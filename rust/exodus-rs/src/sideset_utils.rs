//! Utilities for converting nodesets to sidesets.
//!
//! This module provides functionality to automatically create sidesets from nodesets
//! by identifying element faces where all nodes belong to the nodeset.

use crate::{geometry, mode, topology::FaceDef, EntityId, EntityType, ExodusFile, Result, SideSet, Topology};
use std::collections::{HashMap, HashSet};

/// Convert a nodeset to a sideset by finding all element faces contained in the nodeset.
///
/// This function identifies all element faces where every node is part of the specified
/// nodeset. Only boundary faces (faces appearing in exactly one element) are included,
/// and face normals are verified to point outward from the mesh.
///
/// # Algorithm
///
/// 1. Read the nodeset and store node IDs in a HashSet for O(1) lookup
/// 2. Build a face registry to identify boundary faces (appear only once in mesh)
/// 3. Compute mesh center of mass for normal verification
/// 4. For each element block:
///    - Get topology and face definitions
///    - For each element:
///      - For each face:
///        - Check if all face nodes are in nodeset
///        - Check if face is on boundary
///        - Verify normal points outward from mesh center
///        - Check normal consistency with previously added faces
/// 5. Return SideSet with elements and side numbers
///
/// # Arguments
///
/// * `file` - Exodus file to analyze
/// * `nodeset_id` - ID of the existing nodeset
/// * `new_sideset_id` - ID for the new sideset
///
/// # Returns
///
/// A `SideSet` containing element IDs and side numbers for all matching boundary faces
///
/// # Errors
///
/// Returns an error if:
/// - The nodeset doesn't exist
/// - Unable to read coordinates or connectivity
/// - File I/O errors occur
///
/// # Examples
///
/// ```no_run
/// use exodus_rs::{ExodusFile, mode};
///
/// let file = ExodusFile::<mode::Read>::open("mesh.exo")?;
/// let sideset = exodus_rs::sideset_utils::convert_nodeset_to_sideset(
///     &file,
///     10,   // nodeset ID
///     100   // new sideset ID
/// )?;
///
/// println!("Created sideset with {} faces", sideset.elements.len());
/// # Ok::<(), exodus_rs::ExodusError>(())
/// ```
pub fn convert_nodeset_to_sideset(
    file: &ExodusFile<mode::Read>,
    nodeset_id: EntityId,
    new_sideset_id: EntityId,
) -> Result<SideSet> {
    // 1. Read nodeset and create HashSet for O(1) lookup
    let nodeset = file.node_set(nodeset_id)?;
    let nodeset_nodes: HashSet<i64> = nodeset.nodes.iter().copied().collect();

    if nodeset_nodes.is_empty() {
        eprintln!(
            "Warning: Nodeset {} is empty, cannot create sideset",
            nodeset_id
        );
        return Ok(SideSet {
            id: new_sideset_id,
            elements: Vec::new(),
            sides: Vec::new(),
            dist_factors: Vec::new(),
        });
    }

    // 2. Build face registry for boundary detection
    let face_registry = build_face_registry(file)?;

    // 3. Compute mesh center of mass for normal verification
    let mesh_center = geometry::compute_center_of_mass(file)?;

    // 4. Get coordinates for normal computation
    let coords = file.coords()?;

    // 5. Find all matching faces
    let mut elements = Vec::new();
    let mut sides = Vec::new();
    let mut face_normals = Vec::new(); // Track for consistency checking

    // Iterate through all element blocks
    for block_id in file.block_ids(EntityType::ElemBlock)? {
        let block = file.block(block_id)?;
        let topology = Topology::from_string(&block.topology);

        // Get face definitions for this topology
        let face_defs = match topology.faces() {
            Some(f) => f,
            None => {
                // Skip unsupported topologies
                continue;
            }
        };

        let connectivity = file.connectivity_structured(block_id)?;

        // Check each element in this block
        for (elem_idx, elem_nodes) in connectivity.iter().enumerate() {
            let elem_id = (elem_idx + 1) as i64; // 1-based element ID

            // Check each face of this element
            for face_def in &face_defs {
                // Get actual node IDs for this face
                let face_nodes: Vec<i64> = face_def
                    .node_indices
                    .iter()
                    .map(|&idx| elem_nodes[idx])
                    .collect();

                // Check if ALL face nodes are in nodeset
                if !face_nodes.iter().all(|n| nodeset_nodes.contains(n)) {
                    continue;
                }

                // Check if it's a boundary face
                if !is_boundary_face(&face_nodes, &face_registry) {
                    continue;
                }

                // Get coordinates for normal computation
                let face_coords: Vec<geometry::Vec3> = face_nodes
                    .iter()
                    .map(|&node_id| {
                        let idx = (node_id - 1) as usize; // Convert to 0-based index
                        [coords.0[idx], coords.1[idx], coords.2[idx]]
                    })
                    .collect();

                // Compute face normal and center
                let face_normal = geometry::compute_face_normal(&face_coords);
                let face_center = geometry::compute_face_center(&face_coords);

                // Verify normal points outward from mesh
                if !geometry::is_outward_facing(face_center, face_normal, mesh_center) {
                    eprintln!(
                        "Warning: Face on element {} side {} has inward-pointing normal, skipping",
                        elem_id, face_def.side_number
                    );
                    continue;
                }

                // Check consistency with previously added faces
                if !face_normals.is_empty() {
                    let avg_normal = geometry::average_normals(&face_normals);
                    if !geometry::normals_consistent(face_normal, avg_normal, -0.5) {
                        eprintln!(
                            "Warning: Face on element {} side {} has inconsistent normal direction (dot product with average < -0.5)",
                            elem_id, face_def.side_number
                        );
                        // Still include it, but warn
                    }
                }

                // This face passes all checks, add it to the sideset
                elements.push(elem_id);
                sides.push(face_def.side_number as i64);
                face_normals.push(face_normal);
            }
        }
    }

    // Log warning if no faces found
    if elements.is_empty() {
        eprintln!(
            "Warning: No boundary faces found for nodeset {}. \
             Possible reasons: \
             (1) nodeset doesn't contain complete element faces, \
             (2) nodeset is on interior nodes only, \
             (3) element topology not supported.",
            nodeset_id
        );
    } else {
        println!(
            "Created sideset {} from nodeset {} with {} boundary faces",
            new_sideset_id,
            nodeset_id,
            elements.len()
        );
    }

    Ok(SideSet {
        id: new_sideset_id,
        elements,
        sides,
        dist_factors: Vec::new(), // Don't copy distribution factors
    })
}

/// Build a registry counting how many times each face appears in the mesh.
///
/// Used to identify boundary faces (faces that appear in exactly one element).
///
/// # Arguments
///
/// * `file` - Exodus file to analyze
///
/// # Returns
///
/// HashMap mapping sorted face node IDs to occurrence count
fn build_face_registry(file: &ExodusFile<mode::Read>) -> Result<HashMap<Vec<i64>, usize>> {
    let mut registry = HashMap::new();

    for block_id in file.block_ids(EntityType::ElemBlock)? {
        let block = file.block(block_id)?;
        let topology = Topology::from_string(&block.topology);

        let face_defs = match topology.faces() {
            Some(f) => f,
            None => continue,
        };

        let connectivity = file.connectivity_structured(block_id)?;

        for elem_nodes in connectivity.iter() {
            for face_def in &face_defs {
                // Get node IDs for this face
                let mut face_nodes: Vec<i64> = face_def
                    .node_indices
                    .iter()
                    .map(|&idx| elem_nodes[idx])
                    .collect();

                // Sort nodes to create canonical representation
                // (so same face from different elements matches)
                face_nodes.sort_unstable();

                // Increment count for this face
                *registry.entry(face_nodes).or_insert(0) += 1;
            }
        }
    }

    Ok(registry)
}

/// Check if a face is on the mesh boundary.
///
/// A face is on the boundary if it appears in exactly one element.
///
/// # Arguments
///
/// * `face_nodes` - Node IDs of the face (unsorted)
/// * `registry` - Face registry mapping sorted node IDs to occurrence count
///
/// # Returns
///
/// `true` if the face appears exactly once (is on boundary)
fn is_boundary_face(face_nodes: &[i64], registry: &HashMap<Vec<i64>, usize>) -> bool {
    let mut sorted = face_nodes.to_vec();
    sorted.sort_unstable();
    registry.get(&sorted).copied() == Some(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_boundary_face() {
        let mut registry = HashMap::new();
        registry.insert(vec![1, 2, 3], 1); // Boundary face
        registry.insert(vec![4, 5, 6], 2); // Interior face

        assert!(is_boundary_face(&[3, 1, 2], &registry)); // Unsorted but matches
        assert!(is_boundary_face(&[1, 2, 3], &registry));
        assert!(!is_boundary_face(&[4, 5, 6], &registry)); // Interior
        assert!(!is_boundary_face(&[7, 8, 9], &registry)); // Not in registry
    }
}
