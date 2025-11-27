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
// Copy-Mirror-Merge Implementation
// ============================================================================

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
    // Node sets (id, nodes, dist_factors)
    node_sets: Vec<(i64, Vec<i64>, Vec<f64>)>,
    // Side sets (id, elements, sides, dist_factors)
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

/// Check if a variable name suggests it's a vector component
fn is_vector_component(name: &str, axis: Axis) -> bool {
    let name_lower = name.to_lowercase();
    let suffix = match axis {
        Axis::X => ["_x", "x", "_u", "u"],
        Axis::Y => ["_y", "y", "_v", "v"],
        Axis::Z => ["_z", "z", "_w", "w"],
    };

    suffix
        .iter()
        .any(|s| name_lower.ends_with(s) || (name_lower.len() == 1 && name_lower == *s))
}

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

// ============================================================================
// Helper Functions for Copy-Mirror-Merge
// ============================================================================

/// Node mapping result from build_node_mapping
struct NodeMappingResult {
    /// Maps original node index (0-based) to new node ID (1-based)
    mirror_node_map: HashMap<usize, i64>,
    /// Total number of nodes after mirroring
    num_new_nodes: usize,
    /// Number of newly created mirror nodes (excluding symmetry plane nodes)
    num_mirror_nodes: usize,
}

/// Build node mapping for mirroring operation
///
/// Creates a mapping from original node indices to new node IDs.
/// Nodes on the symmetry plane map to themselves, while other nodes
/// get new IDs starting after the original node count.
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

/// Create mirrored coordinates for nodes not on the symmetry plane
fn create_mirrored_coordinates(
    data: &MeshData,
    axis: Axis,
    symmetry_nodes: &HashSet<usize>,
) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let orig_num_nodes = data.params.num_nodes;
    let mut new_x = data.x.clone();
    let mut new_y = data.y.clone();
    let mut new_z = data.z.clone();

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

/// Create mirrored element blocks with proper connectivity
fn create_mirrored_blocks(
    data: &MeshData,
    axis: Axis,
    mirror_node_map: &HashMap<usize, i64>,
    verbose: bool,
) -> Result<MirroredBlocksResult> {
    let mut new_blocks = data.blocks.clone();
    let mut new_connectivities = data.connectivities.clone();
    let mut new_block_names = data.block_names.clone();

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

        // Create mirrored connectivity for this block
        let mut mirror_connectivity = Vec::new();

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
    node_sets: Vec<(i64, Vec<i64>, Vec<f64>)>,
    node_set_names: Vec<String>,
}

/// Create mirrored node sets with proper node ID mapping
fn create_mirrored_node_sets(
    data: &MeshData,
    mirror_node_map: &HashMap<usize, i64>,
) -> MirroredNodeSetsResult {
    let mut new_node_sets = data.node_sets.clone();
    let mut new_node_set_names = data.node_set_names.clone();

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

/// Create mirrored side sets with proper element ID offsets
fn create_mirrored_side_sets(data: &MeshData, orig_num_elems: usize) -> MirroredSideSetsResult {
    let mut new_side_sets = data.side_sets.clone();
    let mut new_side_set_names = data.side_set_names.clone();

    let max_ss_id = data
        .side_sets
        .iter()
        .map(|(id, _, _, _)| *id)
        .max()
        .unwrap_or(0);
    let mut next_ss_id = max_ss_id + 1;

    for (idx, (orig_id, elements, sides, dist_factors)) in data.side_sets.iter().enumerate() {
        // Mirrored elements have IDs offset by orig_num_elems
        let mirror_elements: Vec<i64> = elements
            .iter()
            .map(|&e| e + orig_num_elems as i64)
            .collect();

        // Side numbers need adjustment based on topology and axis
        // For now, keep same side numbers (this is a simplification)
        // TODO: Implement proper side number mapping for different topologies
        let mirror_sides = sides.clone();

        let mirror_df = dist_factors.clone();

        new_side_sets.push((next_ss_id, mirror_elements, mirror_sides, mirror_df));

        let default_name = format!("sideset_{}", orig_id);
        let orig_name = data
            .side_set_names
            .get(idx)
            .map(|s| s.as_str())
            .unwrap_or(&default_name);
        new_side_set_names.push(format!("{}_mirror", orig_name));

        next_ss_id += 1;
    }

    MirroredSideSetsResult {
        side_sets: new_side_sets,
        side_set_names: new_side_set_names,
    }
}

/// Create mirrored nodal variable values
fn create_mirrored_nodal_vars(
    data: &MeshData,
    axis: Axis,
    symmetry_nodes: &HashSet<usize>,
    verbose: bool,
) -> Vec<Vec<Vec<f64>>> {
    let orig_num_nodes = data.params.num_nodes;
    let mut new_nodal_var_values: Vec<Vec<Vec<f64>>> = Vec::new();

    for (var_idx, var_name) in data.nodal_var_names.iter().enumerate() {
        let is_mirror_component = is_vector_component(var_name, axis);

        let mut var_time_series = Vec::new();
        for step in 0..data.times.len() {
            let orig_values = &data.nodal_var_values[var_idx][step];
            let mut new_values = orig_values.clone();

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
fn create_mirrored_elem_vars(
    data: &MeshData,
    axis: Axis,
    verbose: bool,
) -> Vec<Vec<Vec<Vec<f64>>>> {
    // Structure: [block_idx][var_idx][time_step][elem_idx]
    // After mirroring, we have original blocks followed by mirrored blocks
    let mut new_elem_var_values: Vec<Vec<Vec<Vec<f64>>>> = Vec::new();

    // First, keep original block values unchanged
    for block_vars in &data.elem_var_values {
        new_elem_var_values.push(block_vars.clone());
    }

    // Then, add mirrored block values (duplicating original values with vector negation)
    for (block_idx, block_vars) in data.elem_var_values.iter().enumerate() {
        let mut mirror_block_vars: Vec<Vec<Vec<f64>>> = Vec::new();

        for (var_idx, var_time_series) in block_vars.iter().enumerate() {
            let var_name = data
                .elem_var_names
                .get(var_idx)
                .map(|s| s.as_str())
                .unwrap_or("");
            let is_mirror_component = is_vector_component(var_name, axis);

            let mut mirror_var_time_series: Vec<Vec<f64>> = Vec::new();
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
// Main Copy-Mirror-Merge Function
// ============================================================================

/// Perform the copy-mirror-merge operation
fn copy_mirror_merge(
    data: &MeshData,
    axis: Axis,
    tolerance: f64,
    verbose: bool,
) -> Result<MeshData> {
    let orig_num_nodes = data.params.num_nodes;
    let orig_num_elems = data.params.num_elems;

    // Find nodes on the symmetry plane
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

    // Build node mapping for mirrored nodes
    let node_mapping = build_node_mapping(orig_num_nodes, &symmetry_nodes);

    if verbose {
        println!(
            "  New mesh: {} nodes ({} original + {} mirrored)",
            node_mapping.num_new_nodes, orig_num_nodes, node_mapping.num_mirror_nodes
        );
    }

    // Create mirrored coordinates
    let (new_x, new_y, new_z) = create_mirrored_coordinates(data, axis, &symmetry_nodes);

    // Create mirrored element blocks
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

    // Create mirrored node sets
    let node_sets_result = create_mirrored_node_sets(data, &node_mapping.mirror_node_map);

    // Create mirrored side sets
    let side_sets_result = create_mirrored_side_sets(data, orig_num_elems);

    // Create mirrored nodal variables
    let new_nodal_var_values = create_mirrored_nodal_vars(data, axis, &symmetry_nodes, verbose);

    // Create mirrored element variables
    let new_elem_var_values = create_mirrored_elem_vars(data, axis, verbose);

    // Create new params
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
    verbose: bool,
) -> Result<()> {
    if verbose {
        println!(
            "  Copy-mirror-merge about {:?} axis (tolerance: {})",
            axis, tolerance
        );
    }

    // Read input mesh
    let input_file = ExodusFile::<mode::Read>::open(input_path)?;
    let mesh_data = read_mesh_data(&input_file, verbose)?;
    drop(input_file);

    // Apply copy-mirror-merge
    let merged_data = copy_mirror_merge(&mesh_data, axis, tolerance, verbose)?;

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
    fn test_is_vector_component() {
        // Test X-axis vector components
        assert!(is_vector_component("velocity_x", Axis::X));
        assert!(is_vector_component("u", Axis::X));
        assert!(is_vector_component("displacement_x", Axis::X));
        assert!(!is_vector_component("velocity_y", Axis::X));
        assert!(!is_vector_component("temperature", Axis::X));

        // Test Y-axis vector components
        assert!(is_vector_component("velocity_y", Axis::Y));
        assert!(is_vector_component("v", Axis::Y));
        assert!(!is_vector_component("velocity_x", Axis::Y));

        // Test Z-axis vector components
        assert!(is_vector_component("velocity_z", Axis::Z));
        assert!(is_vector_component("w", Axis::Z));
        assert!(!is_vector_component("velocity_x", Axis::Z));
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
}
