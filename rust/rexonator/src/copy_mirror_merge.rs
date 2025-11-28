//! Copy-Mirror-Merge functionality for creating full models from half-symmetry models
//!
//! This module provides functionality to mirror a mesh about a symmetry plane,
//! creating a full model from a half-symmetry model. Nodes on the symmetry plane
//! are merged, and all mesh entities (elements, node sets, side sets, variables)
//! are properly duplicated and transformed.

use crate::cli::{Axis, Result, TransformError};
use exodus_rs::{mode, types::*, ExodusFile};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

// ============================================================================
// Vector Component Detection
// ============================================================================

/// Known prefixes that indicate scalar values, not vector components.
/// These are common patterns like "max_x" (maximum x value) that should NOT
/// be treated as vector components even though they end with "_x".
const SCALAR_PREFIXES: &[&str] = &[
    "max", "min", "avg", "average", "mean", "sum", "total", "index", "idx", "count", "num",
    "start", "end", "first", "last", "row", "col", "column",
];

/// Configuration for vector component detection during copy-mirror-merge.
///
/// This allows users to explicitly control which variables are treated as
/// vector components (and thus have their values negated when mirroring).
#[derive(Debug, Clone, Default)]
pub struct VectorDetectionConfig {
    /// Base names of fields that are definitely vectors (e.g., "velocity", "displacement").
    /// Fields like "velocity_x", "velocity_y", "velocity_z" will match.
    pub vector_fields: Vec<String>,

    /// Specific field names that are definitely NOT vectors (e.g., "max_x", "flux_x").
    /// These override automatic detection.
    pub scalar_fields: Vec<String>,

    /// If true, disable automatic vector detection entirely.
    /// Only fields matching `vector_fields` will be treated as vectors.
    pub no_auto_detection: bool,
}

impl VectorDetectionConfig {
    /// Create a new VectorDetectionConfig from CLI options.
    pub fn from_cli_options(
        vector_fields: Option<&str>,
        scalar_fields: Option<&str>,
        no_auto_detection: bool,
    ) -> Self {
        Self {
            vector_fields: vector_fields
                .map(|s| s.split(',').map(|v| v.trim().to_lowercase()).collect())
                .unwrap_or_default(),
            scalar_fields: scalar_fields
                .map(|s| s.split(',').map(|v| v.trim().to_lowercase()).collect())
                .unwrap_or_default(),
            no_auto_detection,
        }
    }

    /// Check if a variable name is a vector component for the given axis.
    pub fn is_vector_component(&self, name: &str, axis: Axis) -> bool {
        let name_lower = name.to_lowercase();

        // 1. Explicit scalar override - highest priority
        if self.scalar_fields.iter().any(|s| s == &name_lower) {
            return false;
        }

        // 2. Check if it matches user-specified vector fields
        let matches_user_vector = self.matches_user_vector_field(&name_lower, axis);
        if matches_user_vector {
            return true;
        }

        // 3. If auto-detection is disabled, stop here
        if self.no_auto_detection {
            return false;
        }

        // 4. Apply improved automatic detection heuristics
        self.auto_detect_vector_component(&name_lower, axis)
    }

    /// Check if a field name matches a user-specified vector field pattern.
    fn matches_user_vector_field(&self, name_lower: &str, axis: Axis) -> bool {
        let suffixes = match axis {
            Axis::X => &["_x", "_1", "_u"][..],
            Axis::Y => &["_y", "_2", "_v"][..],
            Axis::Z => &["_z", "_3", "_w"][..],
        };

        for base in &self.vector_fields {
            // Check if name is "base_x", "base_y", "base_z", etc.
            for suffix in suffixes {
                let pattern = format!("{}{}", base, suffix);
                if name_lower == pattern {
                    return true;
                }
            }
            // Also check if name exactly equals the base (single-component vector)
            if name_lower == base {
                return true;
            }
        }
        false
    }

    /// Improved automatic vector component detection with conservative heuristics.
    fn auto_detect_vector_component(&self, name_lower: &str, axis: Axis) -> bool {
        // Single-letter variables: exact match only (u, v, w)
        if name_lower.len() == 1 {
            return matches!(
                (name_lower, axis),
                ("u", Axis::X) | ("v", Axis::Y) | ("w", Axis::Z)
            );
        }

        // Require underscore + component suffix (stricter than before)
        let (primary_suffix, alt_suffix) = match axis {
            Axis::X => ("_x", "_1"),
            Axis::Y => ("_y", "_2"),
            Axis::Z => ("_z", "_3"),
        };

        let suffix_match = if name_lower.ends_with(primary_suffix) {
            Some(primary_suffix)
        } else if name_lower.ends_with(alt_suffix) {
            Some(alt_suffix)
        } else {
            None
        };

        let Some(suffix) = suffix_match else {
            return false;
        };

        // Get the base name (everything before the suffix)
        let base = &name_lower[..name_lower.len() - suffix.len()];

        // Exclude known scalar patterns
        if SCALAR_PREFIXES.contains(&base) {
            return false;
        }

        // Also exclude if base ends with a scalar prefix after underscore
        // e.g., "stress_max_x" should not match
        for prefix in SCALAR_PREFIXES {
            let pattern = format!("_{}", prefix);
            if base.ends_with(&pattern) {
                return false;
            }
        }

        // If we get here, it looks like a vector component
        true
    }
}

// ============================================================================
// Memory Estimation and Warnings
// ============================================================================

/// Size of a f64 value in bytes
const SIZE_F64: usize = std::mem::size_of::<f64>();
/// Size of an i64 value in bytes
const SIZE_I64: usize = std::mem::size_of::<i64>();
/// Memory warning threshold: 1 GB
const MEMORY_WARNING_THRESHOLD_GB: usize = 1;

/// Estimate memory usage for the copy-mirror-merge operation
///
/// Returns the estimated memory usage in bytes for both the source and mirrored data
fn estimate_memory_usage(data: &MeshData) -> usize {
    let num_nodes = data.params.num_nodes;
    let num_elems = data.params.num_elems;
    let num_time_steps = data.times.len().max(1);
    let num_nodal_vars = data.nodal_var_names.len();
    let num_elem_vars = data.elem_var_names.len();
    let num_blocks = data.blocks.len();

    // Coordinates: 3 arrays x num_nodes x f64 x 2 (original + mirrored)
    let coord_memory = 3 * num_nodes * SIZE_F64 * 2;

    // Connectivities: approximate based on average nodes per element
    let avg_nodes_per_elem = if num_blocks > 0 && num_elems > 0 {
        data.blocks
            .iter()
            .map(|b| b.num_nodes_per_entry)
            .sum::<usize>()
            / num_blocks
    } else {
        8 // default to HEX8
    };
    let connectivity_memory = num_elems * avg_nodes_per_elem * SIZE_I64 * 2;

    // Nodal variables: num_vars x num_time_steps x num_nodes x f64 x 2
    let nodal_var_memory = num_nodal_vars * num_time_steps * num_nodes * SIZE_F64 * 2;

    // Element variables: num_blocks x num_vars x num_time_steps x (num_elems/num_blocks) x f64 x 2
    let elem_var_memory = if num_blocks > 0 {
        num_blocks * num_elem_vars * num_time_steps * (num_elems / num_blocks) * SIZE_F64 * 2
    } else {
        0
    };

    // Node sets and side sets (rough estimate)
    let set_memory = (data.node_sets.len() + data.side_sets.len()) * 1000 * SIZE_I64 * 2;

    coord_memory + connectivity_memory + nodal_var_memory + elem_var_memory + set_memory
}

/// Format bytes as a human-readable string
fn format_bytes(bytes: usize) -> String {
    const GB: usize = 1_000_000_000;
    const MB: usize = 1_000_000;
    const KB: usize = 1_000;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

/// Print memory usage warnings if estimated usage exceeds threshold
fn warn_memory_usage(data: &MeshData, verbose: bool) {
    let estimated_bytes = estimate_memory_usage(data);
    let threshold_bytes = MEMORY_WARNING_THRESHOLD_GB * 1_000_000_000;

    if estimated_bytes > threshold_bytes {
        eprintln!(
            "WARNING: Estimated memory usage: {} (exceeds {} GB threshold)",
            format_bytes(estimated_bytes),
            MEMORY_WARNING_THRESHOLD_GB
        );
        eprintln!(
            "         Mesh: {} nodes, {} elements, {} time steps",
            data.params.num_nodes,
            data.params.num_elems,
            data.times.len()
        );
        eprintln!(
            "         Variables: {} nodal, {} element",
            data.nodal_var_names.len(),
            data.elem_var_names.len()
        );
        eprintln!("         Consider processing on a machine with sufficient memory.");
    } else if verbose {
        println!(
            "  Estimated memory usage: {}",
            format_bytes(estimated_bytes)
        );
    }
}

// ============================================================================
// Copy-Mirror-Merge Implementation
// ============================================================================

/// Type alias for node set data: (id, nodes, dist_factors)
type NodeSetData = (i64, Vec<i64>, Vec<f64>);

/// Type alias for side set data: (id, elements, sides, dist_factors)
type SideSetData = (i64, Vec<i64>, Vec<i64>, Vec<f64>);

/// Data structure to hold all mesh data for copy-mirror-merge operation
#[derive(Debug)]
struct MeshData {
    // Initialization parameters
    params: InitParams,
    // Coordinates
    x: Vec<f64>,
    y: Vec<f64>,
    z: Vec<f64>,
    // Element blocks (supports multiple blocks)
    blocks: Vec<Block>,
    connectivities: Vec<Vec<i64>>,
    block_names: Vec<String>,
    // Node sets
    node_sets: Vec<NodeSetData>,
    // Side sets
    side_sets: Vec<SideSetData>,
    // Nodal variables (name, values for each time step)
    nodal_var_names: Vec<String>,
    nodal_var_values: Vec<Vec<Vec<f64>>>, // [var_idx][time_step][node_idx]
    // Element block variables (name, values for each block and time step)
    elem_var_names: Vec<String>,
    elem_var_values: Vec<Vec<Vec<Vec<f64>>>>, // [block_idx][var_idx][time_step][elem_idx]
    // Global variables
    global_var_names: Vec<String>,
    global_var_values: Vec<Vec<f64>>, // [time_step][var_idx]
    // Time values
    times: Vec<f64>,
    // Set names (for creating _mirror variants)
    node_set_names: Vec<String>,
    side_set_names: Vec<String>,
}

/// Find nodes on the symmetry plane
fn find_symmetry_plane_nodes(coords: &[f64], _axis: Axis, tolerance: f64) -> Vec<usize> {
    coords
        .iter()
        .enumerate()
        .filter_map(|(i, &val)| {
            if val.abs() <= tolerance {
                Some(i)
            } else {
                None
            }
        })
        .collect()
}

/// Get the coordinate array for a given axis
fn get_axis_coords<'a>(x: &'a [f64], y: &'a [f64], z: &'a [f64], axis: Axis) -> &'a [f64] {
    match axis {
        Axis::X => x,
        Axis::Y => y,
        Axis::Z => z,
    }
}

/// Get the winding order permutation for mirroring an element
/// Returns indices to reorder element connectivity to maintain proper winding
fn get_mirror_permutation(topology: &str, axis: Axis) -> Option<Vec<usize>> {
    let topo_upper = topology.to_uppercase();

    match topo_upper.as_str() {
        "HEX" | "HEX8" => {
            // HEX8 node ordering:
            //     7-------6
            //    /|      /|
            //   4-------5 |
            //   | |     | |
            //   | 3-----|-2
            //   |/      |/
            //   0-------1
            Some(match axis {
                Axis::X => vec![1, 0, 3, 2, 5, 4, 7, 6], // Swap X pairs
                Axis::Y => vec![3, 2, 1, 0, 7, 6, 5, 4], // Swap Y pairs
                Axis::Z => vec![4, 5, 6, 7, 0, 1, 2, 3], // Swap Z pairs (top/bottom)
            })
        }
        "TET" | "TET4" | "TETRA" | "TETRA4" => {
            // TET4: swap any two nodes reverses orientation
            Some(vec![0, 2, 1, 3])
        }
        "WEDGE" | "WEDGE6" => {
            // Wedge: swap triangular faces
            Some(match axis {
                Axis::X => vec![1, 0, 2, 4, 3, 5],
                Axis::Y => vec![2, 1, 0, 5, 4, 3],
                Axis::Z => vec![3, 4, 5, 0, 1, 2],
            })
        }
        "PYRAMID" | "PYRAMID5" => {
            // Pyramid: reverse base ordering
            Some(vec![3, 2, 1, 0, 4])
        }
        "QUAD" | "QUAD4" | "SHELL" | "SHELL4" => {
            // 2D quad: reverse winding
            Some(vec![0, 3, 2, 1])
        }
        "TRI" | "TRI3" | "TRIANGLE" => {
            // 2D triangle: reverse winding
            Some(vec![0, 2, 1])
        }
        _ => None, // Unsupported topology
    }
}

/// Get the side number mapping when mirroring an element about a given axis.
///
/// Returns a function that maps original side numbers to mirrored side numbers.
/// When an element is mirrored, faces perpendicular to the mirror axis swap
/// because the +axis face becomes the -axis face and vice versa.
///
/// # Arguments
/// * `topology` - The element topology string (e.g., "HEX8", "TET4")
/// * `axis` - The axis about which the element is being mirrored
///
/// # Returns
/// A vector where index (side_number - 1) gives the new side number after mirroring.
/// Returns None for unsupported topologies.
fn get_side_number_mapping(topology: &str, axis: Axis) -> Option<Vec<i64>> {
    let topo_upper = topology.to_uppercase();

    match topo_upper.as_str() {
        "HEX" | "HEX8" => {
            // HEX8 face definitions (Exodus convention):
            // Face 1: nodes 0,1,5,4 - front face (approximately -Y normal)
            // Face 2: nodes 1,2,6,5 - right face (+X normal)
            // Face 3: nodes 2,3,7,6 - back face (+Y normal)
            // Face 4: nodes 0,4,7,3 - left face (-X normal)
            // Face 5: nodes 0,3,2,1 - bottom face (-Z normal)
            // Face 6: nodes 4,5,6,7 - top face (+Z normal)
            //
            // When mirroring, faces perpendicular to the mirror axis swap.
            Some(match axis {
                Axis::X => vec![1, 4, 3, 2, 5, 6], // 2↔4 swap (+X↔-X)
                Axis::Y => vec![3, 2, 1, 4, 5, 6], // 1↔3 swap (-Y↔+Y)
                Axis::Z => vec![1, 2, 3, 4, 6, 5], // 5↔6 swap (-Z↔+Z)
            })
        }
        "TET" | "TET4" | "TETRA" | "TETRA4" => {
            // TET4 face definitions (Exodus convention):
            // Face 1: nodes 0,1,3
            // Face 2: nodes 1,2,3
            // Face 3: nodes 0,3,2
            // Face 4: nodes 0,2,1 (base triangle)
            //
            // For tetrahedra, the face mapping depends on the specific geometry.
            // The permutation we use (swap nodes 1,2) affects which faces are which.
            // With permutation [0, 2, 1, 3]:
            // - Original face using nodes {0,1,3} → mirrored face uses nodes {0,2,3} = face 3
            // - Original face using nodes {1,2,3} → mirrored face uses nodes {2,1,3} = face 2
            // - Original face using nodes {0,3,2} → mirrored face uses nodes {0,3,1} = face 1
            // - Original face using nodes {0,2,1} → mirrored face uses nodes {0,1,2} = face 4
            Some(vec![3, 2, 1, 4]) // 1↔3 swap, 2↔2, 4↔4
        }
        "WEDGE" | "WEDGE6" => {
            // WEDGE6 face definitions (Exodus convention):
            // Face 1: nodes 0,1,4,3 (quad)
            // Face 2: nodes 1,2,5,4 (quad)
            // Face 3: nodes 0,3,5,2 (quad)
            // Face 4: nodes 0,2,1 (bottom triangle)
            // Face 5: nodes 3,4,5 (top triangle)
            //
            // Wedge mirroring is axis-dependent
            Some(match axis {
                Axis::X => vec![1, 3, 2, 4, 5], // 2↔3 swap (faces on ±X)
                Axis::Y => vec![2, 1, 3, 4, 5], // 1↔2 swap (faces on ±Y)
                Axis::Z => vec![1, 2, 3, 5, 4], // 4↔5 swap (top/bottom triangles)
            })
        }
        "PYRAMID" | "PYRAMID5" => {
            // PYRAMID5 face definitions (Exodus convention):
            // Face 1: nodes 0,1,4 (tri)
            // Face 2: nodes 1,2,4 (tri)
            // Face 3: nodes 2,3,4 (tri)
            // Face 4: nodes 3,0,4 (tri)
            // Face 5: nodes 0,3,2,1 (base quad)
            //
            // With permutation [3, 2, 1, 0, 4] (reverse base ordering):
            // Triangular faces rotate/swap, base face stays as face 5
            Some(match axis {
                Axis::X => vec![4, 3, 2, 1, 5], // 1↔4, 2↔3 swap
                Axis::Y => vec![2, 1, 4, 3, 5], // 1↔2, 3↔4 swap
                Axis::Z => vec![1, 2, 3, 4, 5], // No swap (pyramid symmetric about Z)
            })
        }
        "QUAD" | "QUAD4" | "SHELL" | "SHELL4" => {
            // QUAD4 edge definitions (2D elements, "faces" are edges):
            // Edge 1: nodes 0,1
            // Edge 2: nodes 1,2
            // Edge 3: nodes 2,3
            // Edge 4: nodes 3,0
            //
            // With permutation [0, 3, 2, 1] (reverse winding):
            Some(match axis {
                Axis::X => vec![4, 3, 2, 1], // 1↔4, 2↔3 swap
                Axis::Y => vec![2, 1, 4, 3], // 1↔2, 3↔4 swap
                Axis::Z => vec![1, 2, 3, 4], // No change for 2D in Z
            })
        }
        "TRI" | "TRI3" | "TRIANGLE" => {
            // TRI3 edge definitions (2D elements, "faces" are edges):
            // Edge 1: nodes 0,1
            // Edge 2: nodes 1,2
            // Edge 3: nodes 2,0
            //
            // With permutation [0, 2, 1] (reverse winding):
            Some(match axis {
                Axis::X => vec![3, 2, 1], // 1↔3 swap
                Axis::Y => vec![2, 1, 3], // 1↔2 swap
                Axis::Z => vec![1, 2, 3], // No change for 2D in Z
            })
        }
        _ => None,
    }
}

/// Map a side number from original element to mirrored element
fn map_side_number(side: i64, mapping: &[i64]) -> i64 {
    if side >= 1 && (side as usize) <= mapping.len() {
        mapping[(side - 1) as usize]
    } else {
        // If side number is out of range, keep it unchanged
        side
    }
}

// ============================================================================
// Mirrored Data Creation Functions
// ============================================================================

/// Result of building the node mapping for mirroring
struct NodeMappingResult {
    /// Maps original node index (0-based) to new node ID (1-based)
    mirror_node_map: HashMap<usize, i64>,
    /// Total number of nodes in the new mesh
    num_new_nodes: usize,
    /// Number of new mirrored nodes (not on symmetry plane)
    num_mirror_nodes: usize,
}

/// Build the node mapping for the mirror operation
///
/// Nodes on the symmetry plane map to themselves, while other nodes
/// get new IDs starting after the original nodes.
fn build_node_mapping(orig_num_nodes: usize, symmetry_nodes: &HashSet<usize>) -> NodeMappingResult {
    let mut mirror_node_map: HashMap<usize, i64> = HashMap::new();
    let mut next_mirror_id = (orig_num_nodes + 1) as i64;

    for i in 0..orig_num_nodes {
        if symmetry_nodes.contains(&i) {
            // Symmetry plane node: maps to itself (1-based)
            mirror_node_map.insert(i, (i + 1) as i64);
        } else {
            // Non-symmetry node: gets a new ID
            mirror_node_map.insert(i, next_mirror_id);
            next_mirror_id += 1;
        }
    }

    let num_new_nodes = (next_mirror_id - 1) as usize;
    let num_mirror_nodes = num_new_nodes - orig_num_nodes;

    NodeMappingResult {
        mirror_node_map,
        num_new_nodes,
        num_mirror_nodes,
    }
}

/// Create mirrored coordinates by reflecting across the specified axis
///
/// Returns (new_x, new_y, new_z) with original and mirrored coordinates combined
fn create_mirrored_coordinates(
    data: &MeshData,
    axis: Axis,
    symmetry_nodes: &HashSet<usize>,
) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let orig_num_nodes = data.params.num_nodes;

    // Start with copies of original coordinates
    let mut new_x = Vec::with_capacity(orig_num_nodes * 2);
    let mut new_y = Vec::with_capacity(orig_num_nodes * 2);
    let mut new_z = Vec::with_capacity(orig_num_nodes * 2);

    new_x.extend_from_slice(&data.x);
    new_y.extend_from_slice(&data.y);
    new_z.extend_from_slice(&data.z);

    // Add mirrored coordinates (for non-symmetry nodes)
    for i in 0..orig_num_nodes {
        if !symmetry_nodes.contains(&i) {
            let mx = if matches!(axis, Axis::X) {
                -data.x[i]
            } else {
                data.x[i]
            };
            let my = if matches!(axis, Axis::Y) {
                -data.y[i]
            } else {
                data.y[i]
            };
            let mz = if matches!(axis, Axis::Z) {
                -data.z[i]
            } else {
                data.z[i]
            };
            new_x.push(mx);
            new_y.push(my);
            new_z.push(mz);
        }
    }

    (new_x, new_y, new_z)
}

/// Result of creating mirrored element blocks
struct MirroredBlocksResult {
    blocks: Vec<Block>,
    connectivities: Vec<Vec<i64>>,
    block_names: Vec<String>,
}

/// Create mirrored element blocks with adjusted connectivity
fn create_mirrored_blocks(
    data: &MeshData,
    axis: Axis,
    mirror_node_map: &HashMap<usize, i64>,
    verbose: bool,
) -> Result<MirroredBlocksResult> {
    // Pre-allocate with capacity for original + mirrored blocks
    let num_blocks = data.blocks.len();
    let mut new_blocks = Vec::with_capacity(num_blocks * 2);
    let mut new_connectivities = Vec::with_capacity(num_blocks * 2);
    let mut new_block_names = Vec::with_capacity(num_blocks * 2);

    // Add original blocks first
    new_blocks.extend(data.blocks.iter().cloned());
    new_connectivities.extend(data.connectivities.iter().cloned());
    new_block_names.extend(data.block_names.iter().cloned());

    // Find max block ID to assign new IDs for mirrored blocks
    let max_block_id = data.blocks.iter().map(|b| b.id).max().unwrap_or(0);
    let mut next_block_id = max_block_id + 1;

    for (block_idx, (block, connectivity)) in data
        .blocks
        .iter()
        .zip(data.connectivities.iter())
        .enumerate()
    {
        let nodes_per_elem = block.num_nodes_per_entry;
        let permutation = get_mirror_permutation(&block.topology, axis).ok_or_else(|| {
            TransformError::InvalidFormat(format!("Unsupported topology: {}", block.topology))
        })?;

        // Create mirrored connectivity for this block with pre-allocated capacity
        let mut mirror_connectivity = Vec::with_capacity(block.num_entries * nodes_per_elem);

        for elem_idx in 0..block.num_entries {
            let elem_start = elem_idx * nodes_per_elem;

            // Get original element nodes
            let orig_elem: Vec<i64> =
                connectivity[elem_start..elem_start + nodes_per_elem].to_vec();

            // Create mirrored element with permuted node order
            let mut mirror_elem = vec![0i64; nodes_per_elem];
            for (new_pos, &old_pos) in permutation.iter().enumerate() {
                let orig_node_id = orig_elem[old_pos] as usize - 1; // 0-based
                mirror_elem[new_pos] = mirror_node_map[&orig_node_id];
            }

            mirror_connectivity.extend(mirror_elem);
        }

        // Create mirrored block
        let mut mirror_block = block.clone();
        mirror_block.id = next_block_id;

        new_blocks.push(mirror_block);
        new_connectivities.push(mirror_connectivity);

        // Create name with _mirror suffix
        let default_name = format!("block_{}", block.id);
        let orig_name = data
            .block_names
            .get(block_idx)
            .filter(|s| !s.is_empty())
            .map(|s| s.as_str())
            .unwrap_or(&default_name);
        new_block_names.push(format!("{}_mirror", orig_name));

        if verbose {
            println!(
                "  Created mirrored block {} from block {} ({} elements)",
                next_block_id, block.id, block.num_entries
            );
        }

        next_block_id += 1;
    }

    Ok(MirroredBlocksResult {
        blocks: new_blocks,
        connectivities: new_connectivities,
        block_names: new_block_names,
    })
}

/// Result of creating mirrored node sets
struct MirroredNodeSetsResult {
    node_sets: Vec<NodeSetData>,
    node_set_names: Vec<String>,
}

/// Create mirrored node sets with _mirror suffix
fn create_mirrored_node_sets(
    data: &MeshData,
    mirror_node_map: &HashMap<usize, i64>,
) -> MirroredNodeSetsResult {
    let num_sets = data.node_sets.len();
    let mut new_node_sets = Vec::with_capacity(num_sets * 2);
    let mut new_node_set_names = Vec::with_capacity(num_sets * 2);

    // Add original node sets
    new_node_sets.extend(data.node_sets.iter().cloned());
    new_node_set_names.extend(data.node_set_names.iter().cloned());

    // Find next available set ID
    let max_ns_id = data
        .node_sets
        .iter()
        .map(|(id, _, _)| *id)
        .max()
        .unwrap_or(0);
    let mut next_ns_id = max_ns_id + 1;

    for (idx, (orig_id, nodes, dist_factors)) in data.node_sets.iter().enumerate() {
        // Create mirrored node set
        let mirror_nodes: Vec<i64> = nodes
            .iter()
            .map(|&n| {
                let orig_idx = (n - 1) as usize;
                mirror_node_map[&orig_idx]
            })
            .collect();

        // Distribution factors are copied as-is
        let mirror_df = dist_factors.clone();

        new_node_sets.push((next_ns_id, mirror_nodes, mirror_df));

        // Create name with _mirror suffix
        let default_name = format!("nodeset_{}", orig_id);
        let orig_name = data
            .node_set_names
            .get(idx)
            .map(|s| s.as_str())
            .unwrap_or(&default_name);
        new_node_set_names.push(format!("{}_mirror", orig_name));

        next_ns_id += 1;
    }

    MirroredNodeSetsResult {
        node_sets: new_node_sets,
        node_set_names: new_node_set_names,
    }
}

/// Result of creating mirrored side sets
struct MirroredSideSetsResult {
    side_sets: Vec<SideSetData>,
    side_set_names: Vec<String>,
}

/// Build a mapping from element ID (1-based) to block index for side number remapping
fn build_element_to_block_map(blocks: &[Block]) -> HashMap<i64, usize> {
    let mut elem_to_block: HashMap<i64, usize> = HashMap::new();
    let mut next_elem_id = 1i64;

    for (block_idx, block) in blocks.iter().enumerate() {
        for _ in 0..block.num_entries {
            elem_to_block.insert(next_elem_id, block_idx);
            next_elem_id += 1;
        }
    }

    elem_to_block
}

/// Create mirrored side sets with _mirror suffix and proper side number mapping
fn create_mirrored_side_sets(
    data: &MeshData,
    axis: Axis,
    orig_num_elems: usize,
    verbose: bool,
) -> MirroredSideSetsResult {
    let num_sets = data.side_sets.len();
    let mut new_side_sets = Vec::with_capacity(num_sets * 2);
    let mut new_side_set_names = Vec::with_capacity(num_sets * 2);

    // Add original side sets
    new_side_sets.extend(data.side_sets.iter().cloned());
    new_side_set_names.extend(data.side_set_names.iter().cloned());

    let max_ss_id = data
        .side_sets
        .iter()
        .map(|(id, _, _, _)| *id)
        .max()
        .unwrap_or(0);
    let mut next_ss_id = max_ss_id + 1;

    // Build element-to-block mapping for side number remapping
    let elem_to_block = build_element_to_block_map(&data.blocks);

    // Pre-compute side mappings for each block topology
    let side_mappings: Vec<Option<Vec<i64>>> = data
        .blocks
        .iter()
        .map(|b| get_side_number_mapping(&b.topology, axis))
        .collect();

    for (idx, (orig_id, elements, sides, dist_factors)) in data.side_sets.iter().enumerate() {
        // Mirrored elements have IDs offset by orig_num_elems
        let mirror_elements: Vec<i64> = elements
            .iter()
            .map(|&e| e + orig_num_elems as i64)
            .collect();

        // Map side numbers based on element topology and mirror axis
        let mirror_sides: Vec<i64> = elements
            .iter()
            .zip(sides.iter())
            .map(|(&elem_id, &side)| {
                // Find which block this element belongs to
                if let Some(&block_idx) = elem_to_block.get(&elem_id) {
                    // Apply the side mapping for this block's topology
                    if let Some(ref mapping) = side_mappings[block_idx] {
                        map_side_number(side, mapping)
                    } else {
                        side // No mapping available, keep unchanged
                    }
                } else {
                    side // Element not found in any block, keep unchanged
                }
            })
            .collect();

        let mirror_df = dist_factors.clone();

        // Calculate changed count before moving mirror_sides
        let changed_count = if verbose && !sides.is_empty() {
            sides
                .iter()
                .zip(mirror_sides.iter())
                .filter(|(orig, mir)| orig != mir)
                .count()
        } else {
            0
        };

        new_side_sets.push((next_ss_id, mirror_elements, mirror_sides, mirror_df));

        let default_name = format!("sideset_{}", orig_id);
        let orig_name = data
            .side_set_names
            .get(idx)
            .map(|s| s.as_str())
            .unwrap_or(&default_name);
        new_side_set_names.push(format!("{}_mirror", orig_name));

        if verbose && changed_count > 0 {
            println!(
                "  Side set {}: remapped {} of {} side numbers for {:?}-axis mirror",
                orig_id,
                changed_count,
                sides.len(),
                axis
            );
        }

        next_ss_id += 1;
    }

    MirroredSideSetsResult {
        side_sets: new_side_sets,
        side_set_names: new_side_set_names,
    }
}

/// Create mirrored nodal variable values
///
/// Vector components matching the mirror axis are negated.
fn create_mirrored_nodal_vars(
    data: &MeshData,
    axis: Axis,
    symmetry_nodes: &HashSet<usize>,
    vector_config: &VectorDetectionConfig,
    verbose: bool,
) -> Vec<Vec<Vec<f64>>> {
    let orig_num_nodes = data.params.num_nodes;
    let num_vars = data.nodal_var_names.len();
    let num_time_steps = data.times.len();

    let mut new_nodal_var_values: Vec<Vec<Vec<f64>>> = Vec::with_capacity(num_vars);

    for (var_idx, var_name) in data.nodal_var_names.iter().enumerate() {
        let is_mirror_component = vector_config.is_vector_component(var_name, axis);

        let mut var_time_series = Vec::with_capacity(num_time_steps);
        for step in 0..num_time_steps {
            let orig_values = &data.nodal_var_values[var_idx][step];

            // Pre-allocate with capacity for original + mirrored nodes
            let mirror_count = orig_num_nodes - symmetry_nodes.len();
            let mut new_values = Vec::with_capacity(orig_num_nodes + mirror_count);
            new_values.extend_from_slice(orig_values);

            // Add mirrored values
            for (i, &val) in orig_values.iter().enumerate().take(orig_num_nodes) {
                if !symmetry_nodes.contains(&i) {
                    let mirror_val = if is_mirror_component {
                        -val // Negate vector component
                    } else {
                        val
                    };
                    new_values.push(mirror_val);
                }
            }

            var_time_series.push(new_values);
        }
        new_nodal_var_values.push(var_time_series);

        if verbose && is_mirror_component {
            println!("  Negating vector component: {}", var_name);
        }
    }

    new_nodal_var_values
}

/// Create mirrored element variable values
///
/// Vector components matching the mirror axis are negated.
fn create_mirrored_elem_vars(
    data: &MeshData,
    axis: Axis,
    vector_config: &VectorDetectionConfig,
    verbose: bool,
) -> Vec<Vec<Vec<Vec<f64>>>> {
    let num_blocks = data.blocks.len();

    // Pre-allocate for original + mirrored blocks
    let mut new_elem_var_values: Vec<Vec<Vec<Vec<f64>>>> = Vec::with_capacity(num_blocks * 2);

    // First, keep original block values unchanged
    for block_vars in &data.elem_var_values {
        new_elem_var_values.push(block_vars.clone());
    }

    // Then, add mirrored block values (duplicating original values with vector negation)
    for (block_idx, block_vars) in data.elem_var_values.iter().enumerate() {
        let mut mirror_block_vars: Vec<Vec<Vec<f64>>> = Vec::with_capacity(block_vars.len());

        for (var_idx, var_time_series) in block_vars.iter().enumerate() {
            let var_name = data
                .elem_var_names
                .get(var_idx)
                .map(|s| s.as_str())
                .unwrap_or("");
            let is_mirror_component = vector_config.is_vector_component(var_name, axis);

            let mut mirror_var_time_series: Vec<Vec<f64>> =
                Vec::with_capacity(var_time_series.len());
            for step_values in var_time_series {
                let mirror_values: Vec<f64> = if is_mirror_component {
                    step_values.iter().map(|&v| -v).collect()
                } else {
                    step_values.clone()
                };
                mirror_var_time_series.push(mirror_values);
            }
            mirror_block_vars.push(mirror_var_time_series);

            if verbose && is_mirror_component && block_idx == 0 {
                println!("  Negating element vector component: {}", var_name);
            }
        }
        new_elem_var_values.push(mirror_block_vars);
    }

    new_elem_var_values
}

// ============================================================================
// Mesh Data Reading
// ============================================================================

/// Read all mesh data from a file
fn read_mesh_data(file: &ExodusFile<mode::Read>, verbose: bool) -> Result<MeshData> {
    let params = file.init_params()?;

    if verbose {
        println!(
            "  Reading mesh: {} nodes, {} elements",
            params.num_nodes, params.num_elems
        );
    }

    // Read coordinates
    let coords = file.coords()?;
    let x = coords.x;
    let y = coords.y;
    // For 2D meshes, z may be empty - fill with zeros for consistent handling
    let z = if coords.z.is_empty() {
        vec![0.0; x.len()]
    } else {
        coords.z
    };

    // Read all element blocks
    let block_ids = file.block_ids(EntityType::ElemBlock)?;
    if block_ids.is_empty() {
        return Err(TransformError::InvalidFormat(
            "No element blocks found in mesh".to_string(),
        ));
    }

    let mut blocks = Vec::new();
    let mut connectivities = Vec::new();

    for &block_id in &block_ids {
        let block = file.block(block_id)?;

        // Check for supported topology
        let perm = get_mirror_permutation(&block.topology, Axis::X);
        if perm.is_none() {
            return Err(TransformError::InvalidFormat(format!(
                "Unsupported element topology '{}' in block {} for copy-mirror-merge. \
                 Supported: HEX8, TET4, WEDGE6, PYRAMID5, QUAD4, TRI3",
                block.topology, block_id
            )));
        }

        let connectivity = file.connectivity(block_id)?;

        if verbose {
            println!(
                "  Element block {}: {} elements, topology: {}",
                block_id, block.num_entries, block.topology
            );
        }

        blocks.push(block);
        connectivities.push(connectivity);
    }

    // Read block names
    let block_names = file.names(EntityType::ElemBlock).unwrap_or_default();

    // Read node sets
    let mut node_sets = Vec::new();
    let node_set_ids = file.set_ids(EntityType::NodeSet)?;
    for &set_id in &node_set_ids {
        let ns = file.node_set(set_id)?;
        node_sets.push((set_id, ns.nodes, ns.dist_factors));
    }

    // Read side sets
    let mut side_sets = Vec::new();
    let side_set_ids = file.set_ids(EntityType::SideSet)?;
    for &set_id in &side_set_ids {
        let ss = file.side_set(set_id)?;
        side_sets.push((set_id, ss.elements, ss.sides, ss.dist_factors));
    }

    // Read set names
    let node_set_names = file.names(EntityType::NodeSet).unwrap_or_default();
    let side_set_names = file.names(EntityType::SideSet).unwrap_or_default();

    // Read time values
    let times = file.times()?;
    let num_time_steps = times.len();

    // Read nodal variables
    let nodal_var_names = file.variable_names(EntityType::Nodal)?;
    let mut nodal_var_values: Vec<Vec<Vec<f64>>> = Vec::new();

    if verbose {
        println!(
            "  Found {} nodal variables, {} time steps",
            nodal_var_names.len(),
            num_time_steps
        );
    }

    for var_idx in 0..nodal_var_names.len() {
        let mut var_time_series = Vec::new();
        for step in 0..num_time_steps {
            let values = file.var(step, EntityType::Nodal, 0, var_idx)?;
            var_time_series.push(values);
        }
        nodal_var_values.push(var_time_series);
    }

    if verbose && !nodal_var_names.is_empty() {
        println!("  Nodal variables: {:?}", nodal_var_names);
    }

    // Read element block variables
    let elem_var_names = file.variable_names(EntityType::ElemBlock)?;
    let mut elem_var_values: Vec<Vec<Vec<Vec<f64>>>> = Vec::new(); // [block_idx][var_idx][time_step][elem_idx]

    if verbose {
        println!(
            "  Found {} element variables across {} blocks",
            elem_var_names.len(),
            blocks.len()
        );
    }

    for (block_idx, block) in blocks.iter().enumerate() {
        let mut block_vars: Vec<Vec<Vec<f64>>> = Vec::new(); // [var_idx][time_step][elem_idx]

        for var_idx in 0..elem_var_names.len() {
            let mut var_time_series: Vec<Vec<f64>> = Vec::new();
            for step in 0..num_time_steps {
                // Use block.id as entity_id for element block variables
                match file.var(step, EntityType::ElemBlock, block.id, var_idx) {
                    Ok(values) => {
                        if verbose && step == 0 && block_idx == 0 {
                            println!(
                                "    Read {} values for elem var {} on block {}",
                                values.len(),
                                var_idx,
                                block.id
                            );
                        }
                        var_time_series.push(values);
                    }
                    Err(e) => {
                        if verbose && step == 0 {
                            println!(
                                "    Warning: Could not read elem var {} on block {}: {}",
                                var_idx, block.id, e
                            );
                        }
                        // Variable might not be defined for this block (truth table)
                        // Use empty vector to indicate no data
                        var_time_series.push(Vec::new());
                    }
                }
            }
            block_vars.push(var_time_series);
        }
        elem_var_values.push(block_vars);

        if verbose && !elem_var_names.is_empty() && block_idx == 0 {
            println!("  Element variables: {:?}", elem_var_names);
        }
    }

    // Read global variables
    let global_var_names = file.variable_names(EntityType::Global)?;
    let mut global_var_values: Vec<Vec<f64>> = Vec::new();

    for step in 0..num_time_steps {
        let mut step_values = Vec::new();
        for var_idx in 0..global_var_names.len() {
            let values = file.var(step, EntityType::Global, 0, var_idx)?;
            step_values.extend(values);
        }
        global_var_values.push(step_values);
    }

    if verbose && !global_var_names.is_empty() {
        println!("  Global variables: {:?}", global_var_names);
        eprintln!(
            "WARNING: Global variables found. These may need manual adjustment after mirroring:"
        );
        for name in &global_var_names {
            eprintln!("  - {}", name);
        }
        eprintln!("         (e.g., total mass may need doubling, time step size is unchanged)");
    }

    Ok(MeshData {
        params,
        x,
        y,
        z,
        blocks,
        connectivities,
        block_names,
        node_sets,
        side_sets,
        nodal_var_names,
        nodal_var_values,
        elem_var_names,
        elem_var_values,
        global_var_names,
        global_var_values,
        times,
        node_set_names,
        side_set_names,
    })
}

/// Perform the copy-mirror-merge operation
///
/// This function creates a full model from a half-symmetry model by:
/// 1. Identifying nodes on the symmetry plane (for merging)
/// 2. Creating mirrored coordinates for non-symmetry nodes
/// 3. Creating mirrored element blocks with adjusted connectivity
/// 4. Creating mirrored node sets and side sets
/// 5. Creating mirrored variable values (with vector component negation)
fn copy_mirror_merge(
    data: &MeshData,
    axis: Axis,
    tolerance: f64,
    vector_config: &VectorDetectionConfig,
    verbose: bool,
) -> Result<MeshData> {
    let orig_num_nodes = data.params.num_nodes;
    let orig_num_elems = data.params.num_elems;

    // Step 1: Find nodes on the symmetry plane
    let axis_coords = get_axis_coords(&data.x, &data.y, &data.z, axis);
    let symmetry_nodes: HashSet<usize> = find_symmetry_plane_nodes(axis_coords, axis, tolerance)
        .into_iter()
        .collect();

    if verbose {
        println!(
            "  Found {} nodes on symmetry plane (tolerance: {})",
            symmetry_nodes.len(),
            tolerance
        );
    }

    if symmetry_nodes.is_empty() {
        eprintln!(
            "WARNING: No nodes found on the symmetry plane (axis={:?}, tolerance={}).",
            axis, tolerance
        );
        eprintln!("         Node merging will be skipped. Consider using a larger tolerance.");
    }

    // Step 2: Build node mapping
    let node_mapping = build_node_mapping(orig_num_nodes, &symmetry_nodes);

    if verbose {
        println!(
            "  New mesh: {} nodes ({} original + {} mirrored)",
            node_mapping.num_new_nodes, orig_num_nodes, node_mapping.num_mirror_nodes
        );
    }

    // Step 3: Create mirrored coordinates
    let (new_x, new_y, new_z) = create_mirrored_coordinates(data, axis, &symmetry_nodes);

    // Step 4: Create mirrored element blocks
    let blocks_result = create_mirrored_blocks(data, axis, &node_mapping.mirror_node_map, verbose)?;

    if verbose {
        println!(
            "  New mesh: {} elements ({} original + {} mirrored) in {} blocks",
            orig_num_elems * 2,
            orig_num_elems,
            orig_num_elems,
            blocks_result.blocks.len()
        );
    }

    // Step 5: Create mirrored node sets
    let node_sets_result = create_mirrored_node_sets(data, &node_mapping.mirror_node_map);

    // Step 6: Create mirrored side sets
    let side_sets_result = create_mirrored_side_sets(data, axis, orig_num_elems, verbose);

    // Step 7: Create mirrored nodal variables
    let new_nodal_var_values =
        create_mirrored_nodal_vars(data, axis, &symmetry_nodes, vector_config, verbose);

    // Step 8: Create mirrored element variables
    let new_elem_var_values = create_mirrored_elem_vars(data, axis, vector_config, verbose);

    // Step 9: Create updated params
    let mut new_params = data.params.clone();
    new_params.num_nodes = node_mapping.num_new_nodes;
    new_params.num_elems = orig_num_elems * 2;
    new_params.num_elem_blocks = blocks_result.blocks.len();
    new_params.num_node_sets = node_sets_result.node_sets.len();
    new_params.num_side_sets = side_sets_result.side_sets.len();

    Ok(MeshData {
        params: new_params,
        x: new_x,
        y: new_y,
        z: new_z,
        blocks: blocks_result.blocks,
        connectivities: blocks_result.connectivities,
        block_names: blocks_result.block_names,
        node_sets: node_sets_result.node_sets,
        side_sets: side_sets_result.side_sets,
        nodal_var_names: data.nodal_var_names.clone(),
        nodal_var_values: new_nodal_var_values,
        elem_var_names: data.elem_var_names.clone(),
        elem_var_values: new_elem_var_values,
        global_var_names: data.global_var_names.clone(),
        global_var_values: data.global_var_values.clone(),
        times: data.times.clone(),
        node_set_names: node_sets_result.node_set_names,
        side_set_names: side_sets_result.side_set_names,
    })
}

/// Write mesh data to a new file
fn write_mesh_data(path: &PathBuf, data: &MeshData, verbose: bool) -> Result<()> {
    use exodus_rs::types::CreateOptions;

    if verbose {
        println!(
            "  Writing output: {} nodes, {} elements",
            data.params.num_nodes, data.params.num_elems
        );
    }

    // Create new file with clobber mode
    let options = CreateOptions {
        mode: CreateMode::Clobber,
        ..Default::default()
    };
    let mut file = ExodusFile::create(path, options)?;

    // Initialize with params
    file.init(&data.params)?;

    // Write coordinates
    let y_opt = if data.params.num_dim >= 2 {
        Some(&data.y[..])
    } else {
        None
    };
    let z_opt = if data.params.num_dim >= 3 {
        Some(&data.z[..])
    } else {
        None
    };
    file.put_coords(&data.x, y_opt, z_opt)?;

    // Write all element blocks
    for (idx, (block, connectivity)) in data
        .blocks
        .iter()
        .zip(data.connectivities.iter())
        .enumerate()
    {
        file.put_block(block)?;
        file.put_connectivity(block.id, connectivity)?;

        // Write block name if available
        if let Some(name) = data.block_names.get(idx) {
            if !name.is_empty() {
                file.put_name(EntityType::ElemBlock, idx, name)?;
            }
        }
    }

    // Write node sets
    for (idx, (set_id, nodes, dist_factors)) in data.node_sets.iter().enumerate() {
        let df_opt = if dist_factors.is_empty() {
            None
        } else {
            Some(&dist_factors[..])
        };
        file.put_node_set(*set_id, nodes, df_opt)?;

        // Write name if available
        if let Some(name) = data.node_set_names.get(idx) {
            if !name.is_empty() {
                file.put_name(EntityType::NodeSet, idx, name)?;
            }
        }
    }

    // Write side sets
    for (idx, (set_id, elements, sides, dist_factors)) in data.side_sets.iter().enumerate() {
        let df_opt = if dist_factors.is_empty() {
            None
        } else {
            Some(&dist_factors[..])
        };
        file.put_side_set(*set_id, elements, sides, df_opt)?;

        if let Some(name) = data.side_set_names.get(idx) {
            if !name.is_empty() {
                file.put_name(EntityType::SideSet, idx, name)?;
            }
        }
    }

    // Write time steps and variables
    if !data.times.is_empty() {
        if verbose {
            println!(
                "  Writing {} time steps with {} nodal vars, {} elem vars",
                data.times.len(),
                data.nodal_var_names.len(),
                data.elem_var_names.len()
            );
            println!(
                "  Elem var values array: {} blocks",
                data.elem_var_values.len()
            );
        }

        // Define nodal variables
        if !data.nodal_var_names.is_empty() {
            let names: Vec<&str> = data.nodal_var_names.iter().map(|s| s.as_str()).collect();
            file.define_variables(EntityType::Nodal, &names)?;
        }

        // Define global variables
        if !data.global_var_names.is_empty() {
            let names: Vec<&str> = data.global_var_names.iter().map(|s| s.as_str()).collect();
            file.define_variables(EntityType::Global, &names)?;
        }

        // Define element block variables
        if !data.elem_var_names.is_empty() {
            let names: Vec<&str> = data.elem_var_names.iter().map(|s| s.as_str()).collect();
            file.define_variables(EntityType::ElemBlock, &names)?;

            // Write truth table (all blocks have all variables)
            let truth_table = TruthTable::new(
                EntityType::ElemBlock,
                data.blocks.len(),
                data.elem_var_names.len(),
            );
            file.put_truth_table(EntityType::ElemBlock, &truth_table)?;

            if verbose {
                println!(
                    "  Wrote elem_var_tab: {} blocks x {} vars",
                    data.blocks.len(),
                    data.elem_var_names.len()
                );
            }
        }

        // Write time values and variable data
        for (step, &time) in data.times.iter().enumerate() {
            file.put_time(step, time)?;

            // Write nodal variables
            for (var_idx, _) in data.nodal_var_names.iter().enumerate() {
                let values = &data.nodal_var_values[var_idx][step];
                file.put_var(step, EntityType::Nodal, 0, var_idx, values)?;
            }

            // Write global variables
            if !data.global_var_values.is_empty() && !data.global_var_values[step].is_empty() {
                for (var_idx, value) in data.global_var_values[step].iter().enumerate() {
                    file.put_var(step, EntityType::Global, 0, var_idx, &[*value])?;
                }
            }

            // Write element block variables
            for (block_idx, block) in data.blocks.iter().enumerate() {
                if let Some(block_vars) = data.elem_var_values.get(block_idx) {
                    for (var_idx, var_time_series) in block_vars.iter().enumerate() {
                        if let Some(values) = var_time_series.get(step) {
                            if !values.is_empty() {
                                if verbose && step == 0 {
                                    println!(
                                        "    Writing {} values for elem var {} on block {} (id={})",
                                        values.len(),
                                        var_idx,
                                        block_idx,
                                        block.id
                                    );
                                }
                                file.put_var(
                                    step,
                                    EntityType::ElemBlock,
                                    block.id,
                                    var_idx,
                                    values,
                                )?;
                            } else if verbose && step == 0 {
                                println!(
                                    "    Skipping empty elem var {} on block {} (id={})",
                                    var_idx, block_idx, block.id
                                );
                            }
                        }
                    }
                } else if verbose && step == 0 {
                    println!("    No elem var data for block {}", block_idx);
                }
            }
        }
    } else if verbose {
        println!("  No time steps - skipping variable output");
    }

    file.sync()?;

    if verbose {
        println!("  Output written successfully");
    }

    Ok(())
}

/// Apply copy-mirror-merge operation (requires reading entire mesh and creating new file)
pub fn apply_copy_mirror_merge(
    input_path: &PathBuf,
    output_path: &PathBuf,
    axis: Axis,
    tolerance: f64,
    vector_config: &VectorDetectionConfig,
    verbose: bool,
) -> Result<()> {
    if verbose {
        println!(
            "  Copy-mirror-merge about {:?} axis (tolerance: {})",
            axis, tolerance
        );
        if !vector_config.vector_fields.is_empty() {
            println!(
                "  User-specified vector fields: {:?}",
                vector_config.vector_fields
            );
        }
        if !vector_config.scalar_fields.is_empty() {
            println!(
                "  User-specified scalar fields (excluded from negation): {:?}",
                vector_config.scalar_fields
            );
        }
        if vector_config.no_auto_detection {
            println!("  Automatic vector detection: disabled");
        }
    }

    // Read input mesh
    let input_file = ExodusFile::<mode::Read>::open(input_path)?;
    let mesh_data = read_mesh_data(&input_file, verbose)?;
    drop(input_file);

    // Check memory usage and warn if needed
    warn_memory_usage(&mesh_data, verbose);

    // Apply copy-mirror-merge
    let merged_data = copy_mirror_merge(&mesh_data, axis, tolerance, vector_config, verbose)?;

    // Write output mesh
    write_mesh_data(output_path, &merged_data, verbose)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_mirror_permutation_hex8() {
        // Test HEX8 permutations for all axes
        let perm_x = get_mirror_permutation("HEX8", Axis::X).unwrap();
        assert_eq!(perm_x, vec![1, 0, 3, 2, 5, 4, 7, 6]);

        let perm_y = get_mirror_permutation("HEX8", Axis::Y).unwrap();
        assert_eq!(perm_y, vec![3, 2, 1, 0, 7, 6, 5, 4]);

        let perm_z = get_mirror_permutation("HEX8", Axis::Z).unwrap();
        assert_eq!(perm_z, vec![4, 5, 6, 7, 0, 1, 2, 3]);

        // Also test lowercase variant
        let perm_hex = get_mirror_permutation("hex", Axis::X).unwrap();
        assert_eq!(perm_hex, vec![1, 0, 3, 2, 5, 4, 7, 6]);
    }

    #[test]
    fn test_get_mirror_permutation_tet4() {
        let perm = get_mirror_permutation("TET4", Axis::X).unwrap();
        assert_eq!(perm, vec![0, 2, 1, 3]);

        // Same permutation for all axes (swapping 2 nodes reverses orientation)
        assert_eq!(
            get_mirror_permutation("TET4", Axis::Y).unwrap(),
            get_mirror_permutation("TET4", Axis::Z).unwrap()
        );
    }

    #[test]
    fn test_get_mirror_permutation_unsupported() {
        assert!(get_mirror_permutation("HEX27", Axis::X).is_none());
        assert!(get_mirror_permutation("UNKNOWN", Axis::X).is_none());
    }

    #[test]
    fn test_vector_detection_default_config() {
        let config = VectorDetectionConfig::default();

        // Test X-axis vector components with underscore suffix
        assert!(config.is_vector_component("velocity_x", Axis::X));
        assert!(config.is_vector_component("displacement_x", Axis::X));
        assert!(config.is_vector_component("force_x", Axis::X));
        assert!(!config.is_vector_component("velocity_y", Axis::X));
        assert!(!config.is_vector_component("temperature", Axis::X));

        // Test single-letter variables (exact match only)
        assert!(config.is_vector_component("u", Axis::X));
        assert!(config.is_vector_component("v", Axis::Y));
        assert!(config.is_vector_component("w", Axis::Z));
        assert!(!config.is_vector_component("u", Axis::Y)); // u is X component
        assert!(!config.is_vector_component("v", Axis::X)); // v is Y component

        // Test Y-axis vector components
        assert!(config.is_vector_component("velocity_y", Axis::Y));
        assert!(!config.is_vector_component("velocity_x", Axis::Y));

        // Test Z-axis vector components
        assert!(config.is_vector_component("velocity_z", Axis::Z));
        assert!(!config.is_vector_component("velocity_x", Axis::Z));

        // Test numeric suffixes (_1, _2, _3)
        assert!(config.is_vector_component("stress_1", Axis::X));
        assert!(config.is_vector_component("stress_2", Axis::Y));
        assert!(config.is_vector_component("stress_3", Axis::Z));
    }

    #[test]
    fn test_vector_detection_false_positives_prevented() {
        let config = VectorDetectionConfig::default();

        // These should NOT be detected as vector components
        // (key fix for the false positive issue)
        assert!(!config.is_vector_component("max_x", Axis::X));
        assert!(!config.is_vector_component("min_x", Axis::X));
        assert!(!config.is_vector_component("avg_x", Axis::X));
        assert!(!config.is_vector_component("index_x", Axis::X));
        assert!(!config.is_vector_component("count_x", Axis::X));
        assert!(!config.is_vector_component("total_y", Axis::Y));
        assert!(!config.is_vector_component("sum_z", Axis::Z));

        // Words ending in 'x' that aren't vectors (stricter matching now)
        assert!(!config.is_vector_component("matrix", Axis::X));
        assert!(!config.is_vector_component("suffix", Axis::X));
        assert!(!config.is_vector_component("index", Axis::X));

        // Compound scalar patterns
        assert!(!config.is_vector_component("stress_max_x", Axis::X));
        assert!(!config.is_vector_component("temp_min_y", Axis::Y));
    }

    #[test]
    fn test_vector_detection_user_specified_vectors() {
        let config = VectorDetectionConfig::from_cli_options(Some("flux,custom"), None, false);

        // User-specified vectors should be detected
        assert!(config.is_vector_component("flux_x", Axis::X));
        assert!(config.is_vector_component("flux_y", Axis::Y));
        assert!(config.is_vector_component("flux_z", Axis::Z));
        assert!(config.is_vector_component("custom_x", Axis::X));

        // Auto-detection still works for standard patterns
        assert!(config.is_vector_component("velocity_x", Axis::X));
        assert!(config.is_vector_component("u", Axis::X));
    }

    #[test]
    fn test_vector_detection_scalar_override() {
        let config = VectorDetectionConfig::from_cli_options(None, Some("flux_x,special_y"), false);

        // Scalar overrides should prevent detection
        assert!(!config.is_vector_component("flux_x", Axis::X));
        assert!(!config.is_vector_component("special_y", Axis::Y));

        // Other vectors still work
        assert!(config.is_vector_component("velocity_x", Axis::X));
    }

    #[test]
    fn test_vector_detection_no_auto_mode() {
        let config = VectorDetectionConfig::from_cli_options(
            Some("velocity"),
            None,
            true, // no_auto_detection
        );

        // Only user-specified vectors should be detected
        assert!(config.is_vector_component("velocity_x", Axis::X));
        assert!(config.is_vector_component("velocity_y", Axis::Y));

        // Auto-detection patterns should NOT work
        assert!(!config.is_vector_component("displacement_x", Axis::X));
        assert!(!config.is_vector_component("u", Axis::X));
        assert!(!config.is_vector_component("force_x", Axis::X));
    }

    #[test]
    fn test_vector_detection_case_insensitive() {
        let config = VectorDetectionConfig::default();

        // Should be case-insensitive
        assert!(config.is_vector_component("Velocity_X", Axis::X));
        assert!(config.is_vector_component("DISPLACEMENT_Y", Axis::Y));
        assert!(config.is_vector_component("Force_Z", Axis::Z));
        assert!(config.is_vector_component("U", Axis::X));
    }

    #[test]
    fn test_find_symmetry_plane_nodes() {
        let coords = vec![0.0, 0.5, 1.0, 0.0, 0.5, 1.0, 0.001, -0.0005];
        let tolerance = 0.01;

        let sym_nodes = find_symmetry_plane_nodes(&coords, Axis::X, tolerance);

        // Nodes at indices 0, 3, 6, 7 should be on symmetry plane
        assert!(sym_nodes.contains(&0)); // 0.0
        assert!(sym_nodes.contains(&3)); // 0.0
        assert!(sym_nodes.contains(&6)); // 0.001 (within tolerance)
        assert!(sym_nodes.contains(&7)); // -0.0005 (within tolerance)
        assert!(!sym_nodes.contains(&1)); // 0.5
        assert!(!sym_nodes.contains(&2)); // 1.0
    }

    #[test]
    fn test_hex8_winding_order_consistency() {
        // Verify that the HEX8 permutation maintains valid element structure
        // by checking that swapped pairs are consistent
        let perm_x = get_mirror_permutation("HEX8", Axis::X).unwrap();

        // For X-axis mirror, we expect pairs to be swapped:
        // (0,1), (2,3), (4,5), (6,7)
        assert_eq!(perm_x[0], 1);
        assert_eq!(perm_x[1], 0);
        assert_eq!(perm_x[2], 3);
        assert_eq!(perm_x[3], 2);
        assert_eq!(perm_x[4], 5);
        assert_eq!(perm_x[5], 4);
        assert_eq!(perm_x[6], 7);
        assert_eq!(perm_x[7], 6);
    }

    // ========================================================================
    // Side Number Mapping Tests
    // ========================================================================

    #[test]
    fn test_side_number_mapping_hex8_x_axis() {
        // HEX8 mirrored about X: sides 2 (+X) and 4 (-X) should swap
        let mapping = get_side_number_mapping("HEX8", Axis::X).unwrap();

        assert_eq!(map_side_number(1, &mapping), 1); // unchanged
        assert_eq!(map_side_number(2, &mapping), 4); // +X → -X
        assert_eq!(map_side_number(3, &mapping), 3); // unchanged
        assert_eq!(map_side_number(4, &mapping), 2); // -X → +X
        assert_eq!(map_side_number(5, &mapping), 5); // unchanged
        assert_eq!(map_side_number(6, &mapping), 6); // unchanged
    }

    #[test]
    fn test_side_number_mapping_hex8_y_axis() {
        // HEX8 mirrored about Y: sides 1 (-Y) and 3 (+Y) should swap
        let mapping = get_side_number_mapping("HEX8", Axis::Y).unwrap();

        assert_eq!(map_side_number(1, &mapping), 3); // -Y → +Y
        assert_eq!(map_side_number(2, &mapping), 2); // unchanged
        assert_eq!(map_side_number(3, &mapping), 1); // +Y → -Y
        assert_eq!(map_side_number(4, &mapping), 4); // unchanged
        assert_eq!(map_side_number(5, &mapping), 5); // unchanged
        assert_eq!(map_side_number(6, &mapping), 6); // unchanged
    }

    #[test]
    fn test_side_number_mapping_hex8_z_axis() {
        // HEX8 mirrored about Z: sides 5 (-Z) and 6 (+Z) should swap
        let mapping = get_side_number_mapping("HEX8", Axis::Z).unwrap();

        assert_eq!(map_side_number(1, &mapping), 1); // unchanged
        assert_eq!(map_side_number(2, &mapping), 2); // unchanged
        assert_eq!(map_side_number(3, &mapping), 3); // unchanged
        assert_eq!(map_side_number(4, &mapping), 4); // unchanged
        assert_eq!(map_side_number(5, &mapping), 6); // -Z → +Z
        assert_eq!(map_side_number(6, &mapping), 5); // +Z → -Z
    }

    #[test]
    fn test_side_number_mapping_tet4() {
        // TET4 mirrored: sides 1 and 3 swap
        let mapping = get_side_number_mapping("TET4", Axis::X).unwrap();

        assert_eq!(map_side_number(1, &mapping), 3);
        assert_eq!(map_side_number(2, &mapping), 2);
        assert_eq!(map_side_number(3, &mapping), 1);
        assert_eq!(map_side_number(4, &mapping), 4);
    }

    #[test]
    fn test_side_number_mapping_wedge6() {
        // Test WEDGE6 for each axis
        let mapping_x = get_side_number_mapping("WEDGE6", Axis::X).unwrap();
        assert_eq!(map_side_number(2, &mapping_x), 3); // 2↔3 swap
        assert_eq!(map_side_number(3, &mapping_x), 2);

        let mapping_z = get_side_number_mapping("WEDGE6", Axis::Z).unwrap();
        assert_eq!(map_side_number(4, &mapping_z), 5); // 4↔5 swap (triangles)
        assert_eq!(map_side_number(5, &mapping_z), 4);
    }

    #[test]
    fn test_side_number_mapping_quad4() {
        // QUAD4 (2D) mirrored about X: edges 1 and 4 swap, 2 and 3 swap
        let mapping = get_side_number_mapping("QUAD4", Axis::X).unwrap();

        assert_eq!(map_side_number(1, &mapping), 4);
        assert_eq!(map_side_number(2, &mapping), 3);
        assert_eq!(map_side_number(3, &mapping), 2);
        assert_eq!(map_side_number(4, &mapping), 1);
    }

    #[test]
    fn test_side_number_mapping_tri3() {
        // TRI3 (2D) mirrored about X: edges 1 and 3 swap
        let mapping = get_side_number_mapping("TRI3", Axis::X).unwrap();

        assert_eq!(map_side_number(1, &mapping), 3);
        assert_eq!(map_side_number(2, &mapping), 2);
        assert_eq!(map_side_number(3, &mapping), 1);
    }

    #[test]
    fn test_side_number_mapping_out_of_range() {
        // Test that out-of-range side numbers are returned unchanged
        let mapping = get_side_number_mapping("HEX8", Axis::X).unwrap();

        assert_eq!(map_side_number(0, &mapping), 0); // Invalid (0)
        assert_eq!(map_side_number(7, &mapping), 7); // Out of range
        assert_eq!(map_side_number(-1, &mapping), -1); // Negative
    }

    #[test]
    fn test_side_number_mapping_case_insensitive() {
        // Topology should be case-insensitive
        let mapping_lower = get_side_number_mapping("hex8", Axis::X).unwrap();
        let mapping_upper = get_side_number_mapping("HEX8", Axis::X).unwrap();
        let mapping_mixed = get_side_number_mapping("Hex8", Axis::X).unwrap();

        assert_eq!(mapping_lower, mapping_upper);
        assert_eq!(mapping_upper, mapping_mixed);
    }

    #[test]
    fn test_side_number_mapping_unsupported_topology() {
        // Unsupported topology should return None
        assert!(get_side_number_mapping("HEX27", Axis::X).is_none());
        assert!(get_side_number_mapping("UNKNOWN", Axis::X).is_none());
    }
}
