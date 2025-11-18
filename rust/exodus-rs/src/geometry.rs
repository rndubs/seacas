//! Geometric utilities for mesh analysis and normal computation.
//!
//! This module provides functions for:
//! - Computing face normals using the cross product
//! - Calculating mesh center of mass
//! - Verifying normal orientations
//! - Checking normal consistency
//! - Computing element volumes (hex, tet, wedge, pyramid)
//! - Computing element centroids

use crate::{mode, types::Topology, ExodusFile, Result};

/// 3D point or vector
pub type Vec3 = [f64; 3];

/// Compute the normal vector for a face using the cross product.
///
/// Uses the first three nodes of the face to compute the normal via the cross product.
/// The winding order of the face nodes (following Exodus II right-hand rule) determines
/// the direction of the normal.
///
/// # Arguments
///
/// * `coords` - Slice of 3D coordinates for the face nodes
///
/// # Returns
///
/// Normalized normal vector pointing in the direction determined by the right-hand rule
///
/// # Panics
///
/// Panics if `coords` has fewer than 3 nodes
pub fn compute_face_normal(coords: &[Vec3]) -> Vec3 {
    assert!(
        coords.len() >= 3,
        "Need at least 3 nodes to compute face normal"
    );

    // Use first 3 nodes to compute normal via cross product
    let p0 = coords[0];
    let p1 = coords[1];
    let p2 = coords[2];

    // Two edge vectors
    let v1 = sub(p1, p0);
    let v2 = sub(p2, p0);

    // Cross product (right-hand rule)
    let normal = cross(v1, v2);

    normalize(normal)
}

/// Compute the geometric center (centroid) of a face.
///
/// # Arguments
///
/// * `coords` - Slice of 3D coordinates for the face nodes
///
/// # Returns
///
/// The centroid position (average of all node positions)
pub fn compute_face_center(coords: &[Vec3]) -> Vec3 {
    let n = coords.len() as f64;
    let sum_x: f64 = coords.iter().map(|c| c[0]).sum();
    let sum_y: f64 = coords.iter().map(|c| c[1]).sum();
    let sum_z: f64 = coords.iter().map(|c| c[2]).sum();

    [sum_x / n, sum_y / n, sum_z / n]
}

/// Compute the mesh center of mass (average of all node positions).
///
/// # Arguments
///
/// * `file` - Exodus file to analyze
///
/// # Returns
///
/// The center of mass position
pub fn compute_center_of_mass(file: &ExodusFile<mode::Read>) -> Result<Vec3> {
    let coords = file.coords()?;
    let num_nodes = coords.x.len();

    if num_nodes == 0 {
        return Ok([0.0, 0.0, 0.0]);
    }

    let sum_x: f64 = coords.x.iter().sum();
    let sum_y: f64 = coords.y.iter().sum();
    let sum_z: f64 = coords.z.iter().sum();

    Ok([
        sum_x / num_nodes as f64,
        sum_y / num_nodes as f64,
        sum_z / num_nodes as f64,
    ])
}

/// Check if a face normal points away from the mesh center of mass.
///
/// A face is considered "outward-facing" if the dot product of the face normal
/// with the vector from the mesh center to the face center is positive.
///
/// # Arguments
///
/// * `face_center` - Centroid of the face
/// * `face_normal` - Normal vector of the face
/// * `mesh_center` - Center of mass of the mesh
///
/// # Returns
///
/// `true` if the normal points away from the mesh center (outward)
pub fn is_outward_facing(face_center: Vec3, face_normal: Vec3, mesh_center: Vec3) -> bool {
    // Vector from mesh center to face center
    let to_face = sub(face_center, mesh_center);

    // Dot product: positive means normal points away from center
    dot(face_normal, to_face) > 0.0
}

/// Check if two normals point in similar directions.
///
/// # Arguments
///
/// * `n1` - First normal vector
/// * `n2` - Second normal vector
/// * `threshold` - Minimum dot product for consistency (typically -0.5 to 1.0)
///   - 1.0: same direction
///   - 0.0: perpendicular
///   - -1.0: opposite directions
///
/// # Returns
///
/// `true` if dot product > threshold (normals are consistent)
pub fn normals_consistent(n1: Vec3, n2: Vec3, threshold: f64) -> bool {
    dot(n1, n2) > threshold
}

/// Compute the average of multiple normal vectors.
///
/// # Arguments
///
/// * `normals` - Slice of normal vectors to average
///
/// # Returns
///
/// Normalized average normal vector
pub fn average_normals(normals: &[Vec3]) -> Vec3 {
    if normals.is_empty() {
        return [0.0, 0.0, 1.0]; // Default up vector
    }

    let n = normals.len() as f64;
    let sum_x: f64 = normals.iter().map(|v| v[0]).sum();
    let sum_y: f64 = normals.iter().map(|v| v[1]).sum();
    let sum_z: f64 = normals.iter().map(|v| v[2]).sum();

    normalize([sum_x / n, sum_y / n, sum_z / n])
}

// ============================================================================
// Element Volume Calculations
// ============================================================================

/// Compute the volume of a tetrahedron from four vertices.
///
/// Uses the scalar triple product: V = |det(a-d, b-d, c-d)| / 6
/// where a, b, c, d are the four vertices.
///
/// # Arguments
///
/// * `coords` - Array of exactly 4 vertices [a, b, c, d]
///
/// # Returns
///
/// Volume of the tetrahedron (always positive)
///
/// # Panics
///
/// Panics if coords doesn't contain exactly 4 vertices
pub fn tetrahedron_volume(coords: &[Vec3; 4]) -> f64 {
    let a = coords[0];
    let b = coords[1];
    let c = coords[2];
    let d = coords[3];

    // Vectors from d to other vertices
    let ad = sub(a, d);
    let bd = sub(b, d);
    let cd = sub(c, d);

    // Scalar triple product: ad · (bd × cd)
    let cross_bc = cross(bd, cd);
    dot(ad, cross_bc).abs() / 6.0
}

/// Compute the volume of a hexahedral element using tetrahedral decomposition.
///
/// Decomposes the hexahedron into 5 or 6 tetrahedra and sums their volumes.
/// This method works for both regular and irregular (distorted) hexahedra.
///
/// # Arguments
///
/// * `coords` - Array of exactly 8 vertices in Exodus II hex ordering:
///   - Bottom face (z-): 0,1,2,3 (counter-clockwise from bottom)
///   - Top face (z+): 4,5,6,7 (counter-clockwise from top)
///
/// # Returns
///
/// Volume of the hexahedron (always positive)
///
/// # Panics
///
/// Panics if coords doesn't contain exactly 8 vertices
pub fn hexahedron_volume(coords: &[Vec3; 8]) -> f64 {
    // Decompose into 5 tetrahedra using diagonal decomposition
    // This method works for both regular and irregular (distorted) hexahedra
    let mut volume = 0.0;

    // Decomposition using node 0 as common vertex
    // Tet 1: 0,1,2,5
    volume += tetrahedron_volume(&[coords[0], coords[1], coords[2], coords[5]]);
    // Tet 2: 0,2,3,7
    volume += tetrahedron_volume(&[coords[0], coords[2], coords[3], coords[7]]);
    // Tet 3: 0,5,7,4
    volume += tetrahedron_volume(&[coords[0], coords[5], coords[7], coords[4]]);
    // Tet 4: 2,5,7,6
    volume += tetrahedron_volume(&[coords[2], coords[5], coords[7], coords[6]]);
    // Tet 5: 0,2,5,7
    volume += tetrahedron_volume(&[coords[0], coords[2], coords[5], coords[7]]);

    volume
}

/// Compute the volume of a wedge (prism) element using tetrahedral decomposition.
///
/// Decomposes the wedge into 3 tetrahedra.
///
/// # Arguments
///
/// * `coords` - Array of exactly 6 vertices in Exodus II wedge ordering:
///   - Bottom triangle: 0,1,2
///   - Top triangle: 3,4,5
///
/// # Returns
///
/// Volume of the wedge (always positive)
///
/// # Panics
///
/// Panics if coords doesn't contain exactly 6 vertices
pub fn wedge_volume(coords: &[Vec3; 6]) -> f64 {
    // Decompose into 3 tetrahedra
    let mut volume = 0.0;

    // Tet 1: 0,1,2,4
    volume += tetrahedron_volume(&[coords[0], coords[1], coords[2], coords[4]]);
    // Tet 2: 0,2,3,4
    volume += tetrahedron_volume(&[coords[0], coords[2], coords[3], coords[4]]);
    // Tet 3: 2,4,5,3
    volume += tetrahedron_volume(&[coords[2], coords[4], coords[5], coords[3]]);

    volume
}

/// Compute the volume of a pyramid element using tetrahedral decomposition.
///
/// Decomposes the pyramid into 2 tetrahedra.
///
/// # Arguments
///
/// * `coords` - Array of exactly 5 vertices in Exodus II pyramid ordering:
///   - Base quad: 0,1,2,3
///   - Apex: 4
///
/// # Returns
///
/// Volume of the pyramid (always positive)
///
/// # Panics
///
/// Panics if coords doesn't contain exactly 5 vertices
pub fn pyramid_volume(coords: &[Vec3; 5]) -> f64 {
    // Decompose into 2 tetrahedra using diagonal 0-2
    let mut volume = 0.0;

    // Tet 1: 0,1,2,4
    volume += tetrahedron_volume(&[coords[0], coords[1], coords[2], coords[4]]);
    // Tet 2: 0,2,3,4
    volume += tetrahedron_volume(&[coords[0], coords[2], coords[3], coords[4]]);

    volume
}

/// Compute the volume of an element based on its topology and node coordinates.
///
/// Supports HEX8, TET4, WEDGE6, and PYRAMID5 elements.
/// Higher-order elements use the same corner nodes as their linear counterparts.
///
/// # Arguments
///
/// * `topology` - Element topology type
/// * `coords` - Node coordinates for the element
///
/// # Returns
///
/// Volume of the element, or an error if the topology is unsupported or
/// if there are insufficient coordinates.
///
/// # Examples
///
/// ```
/// use exodus_rs::geometry::{element_volume, Vec3};
/// use exodus_rs::Topology;
///
/// // Unit cube hex
/// let coords = vec![
///     [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0],
///     [0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [1.0, 1.0, 1.0], [0.0, 1.0, 1.0],
/// ];
/// let volume = element_volume(Topology::Hex8, &coords).unwrap();
/// assert!((volume - 1.0).abs() < 1e-10);
/// ```
pub fn element_volume(topology: Topology, coords: &[Vec3]) -> Result<f64> {
    match topology {
        Topology::Hex8 | Topology::Hex20 | Topology::Hex27 => {
            if coords.len() < 8 {
                return Err(crate::ExodusError::Other(format!(
                    "HEX element requires at least 8 coordinates, got {}",
                    coords.len()
                )));
            }
            let hex_coords: [Vec3; 8] = coords[0..8].try_into().unwrap();
            Ok(hexahedron_volume(&hex_coords))
        }
        Topology::Tet4 | Topology::Tet8 | Topology::Tet10 | Topology::Tet14 | Topology::Tet15 => {
            if coords.len() < 4 {
                return Err(crate::ExodusError::Other(format!(
                    "TET element requires at least 4 coordinates, got {}",
                    coords.len()
                )));
            }
            let tet_coords: [Vec3; 4] = coords[0..4].try_into().unwrap();
            Ok(tetrahedron_volume(&tet_coords))
        }
        Topology::Wedge6 | Topology::Wedge15 | Topology::Wedge18 => {
            if coords.len() < 6 {
                return Err(crate::ExodusError::Other(format!(
                    "WEDGE element requires at least 6 coordinates, got {}",
                    coords.len()
                )));
            }
            let wedge_coords: [Vec3; 6] = coords[0..6].try_into().unwrap();
            Ok(wedge_volume(&wedge_coords))
        }
        Topology::Pyramid5 | Topology::Pyramid13 | Topology::Pyramid14 => {
            if coords.len() < 5 {
                return Err(crate::ExodusError::Other(format!(
                    "PYRAMID element requires at least 5 coordinates, got {}",
                    coords.len()
                )));
            }
            let pyramid_coords: [Vec3; 5] = coords[0..5].try_into().unwrap();
            Ok(pyramid_volume(&pyramid_coords))
        }
        _ => Err(crate::ExodusError::Other(format!(
            "Volume calculation not supported for topology: {:?}",
            topology
        ))),
    }
}

/// Compute the centroid (geometric center) of an element.
///
/// The centroid is computed as the average of all node positions.
///
/// # Arguments
///
/// * `coords` - Node coordinates for the element
///
/// # Returns
///
/// Centroid position as [x, y, z]
///
/// # Examples
///
/// ```
/// use exodus_rs::geometry::{element_centroid, Vec3};
///
/// // Unit cube
/// let coords = vec![
///     [0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [0.0, 1.0, 0.0],
///     [0.0, 0.0, 1.0], [1.0, 0.0, 1.0], [1.0, 1.0, 1.0], [0.0, 1.0, 1.0],
/// ];
/// let centroid = element_centroid(&coords);
/// assert!((centroid[0] - 0.5).abs() < 1e-10);
/// assert!((centroid[1] - 0.5).abs() < 1e-10);
/// assert!((centroid[2] - 0.5).abs() < 1e-10);
/// ```
pub fn element_centroid(coords: &[Vec3]) -> Vec3 {
    if coords.is_empty() {
        return [0.0, 0.0, 0.0];
    }

    let n = coords.len() as f64;
    let sum_x: f64 = coords.iter().map(|c| c[0]).sum();
    let sum_y: f64 = coords.iter().map(|c| c[1]).sum();
    let sum_z: f64 = coords.iter().map(|c| c[2]).sum();

    [sum_x / n, sum_y / n, sum_z / n]
}

// ============================================================================
// High-level API for mesh geometry calculations
// ============================================================================

impl ExodusFile<mode::Read> {
    /// Compute volumes for all elements in a block.
    ///
    /// This method efficiently computes the volume of every element in the specified
    /// block, eliminating the need for nested loops in user code.
    ///
    /// # Arguments
    ///
    /// * `block_id` - ID of the element block
    ///
    /// # Returns
    ///
    /// Vector of volumes, one per element in the block
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Block ID doesn't exist
    /// - Block topology doesn't support volume calculation
    /// - Coordinates cannot be read
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use exodus_rs::{ExodusFile, mode, types::EntityType};
    ///
    /// # fn main() -> exodus_rs::Result<()> {
    /// let file = ExodusFile::<mode::Read>::open("mesh.exo")?;
    /// let block_ids = file.block_ids(EntityType::ElemBlock)?;
    /// let volumes = file.block_element_volumes(block_ids[0])?;
    /// let total_volume: f64 = volumes.iter().sum();
    /// # Ok(())
    /// # }
    /// ```
    pub fn block_element_volumes(&self, block_id: i64) -> Result<Vec<f64>> {
        // Get block information
        let block = self.block(block_id)?;
        let topology = Topology::from_string(&block.topology);
        let num_elems = block.num_entries;
        let nodes_per_elem = block.num_nodes_per_entry;

        // Get connectivity
        let connectivity = self.connectivity(block_id)?;

        // Get all coordinates
        let coords = self.coords()?;

        // Compute volume for each element
        let mut volumes = Vec::with_capacity(num_elems);
        for elem_idx in 0..num_elems {
            // Extract node indices for this element
            let start = elem_idx * nodes_per_elem;
            let end = start + nodes_per_elem;
            let node_indices = &connectivity[start..end];

            // Get coordinates for this element's nodes
            let mut elem_coords = Vec::with_capacity(nodes_per_elem);
            for &node_id in node_indices {
                // Node IDs are 1-based in Exodus, convert to 0-based for array access
                let idx = (node_id - 1) as usize;
                elem_coords.push([coords.x[idx], coords.y[idx], coords.z[idx]]);
            }

            // Compute volume
            let vol = element_volume(topology.clone(), &elem_coords)?;
            volumes.push(vol);
        }

        Ok(volumes)
    }

    /// Compute centroids for all elements in a block.
    ///
    /// This method efficiently computes the centroid of every element in the specified
    /// block, eliminating the need for nested loops in user code.
    ///
    /// # Arguments
    ///
    /// * `block_id` - ID of the element block
    ///
    /// # Returns
    ///
    /// Vector of centroids as [x, y, z] coordinates, one per element in the block
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Block ID doesn't exist
    /// - Coordinates cannot be read
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use exodus_rs::{ExodusFile, mode, types::EntityType};
    ///
    /// # fn main() -> exodus_rs::Result<()> {
    /// let file = ExodusFile::<mode::Read>::open("mesh.exo")?;
    /// let block_ids = file.block_ids(EntityType::ElemBlock)?;
    /// let centroids = file.block_element_centroids(block_ids[0])?;
    /// println!("First element centroid: {:?}", centroids[0]);
    /// # Ok(())
    /// # }
    /// ```
    pub fn block_element_centroids(&self, block_id: i64) -> Result<Vec<Vec3>> {
        // Get block information
        let block = self.block(block_id)?;
        let num_elems = block.num_entries;
        let nodes_per_elem = block.num_nodes_per_entry;

        // Get connectivity
        let connectivity = self.connectivity(block_id)?;

        // Get all coordinates
        let coords = self.coords()?;

        // Compute centroid for each element
        let mut centroids = Vec::with_capacity(num_elems);
        for elem_idx in 0..num_elems {
            // Extract node indices for this element
            let start = elem_idx * nodes_per_elem;
            let end = start + nodes_per_elem;
            let node_indices = &connectivity[start..end];

            // Get coordinates for this element's nodes
            let mut elem_coords = Vec::with_capacity(nodes_per_elem);
            for &node_id in node_indices {
                // Node IDs are 1-based in Exodus, convert to 0-based for array access
                let idx = (node_id - 1) as usize;
                elem_coords.push([coords.x[idx], coords.y[idx], coords.z[idx]]);
            }

            // Compute centroid
            let centroid = element_centroid(&elem_coords);
            centroids.push(centroid);
        }

        Ok(centroids)
    }

    /// Compute volumes for all elements in the mesh.
    ///
    /// This method efficiently computes the volume of every element in all blocks,
    /// returning a vector indexed by global element number (0-based).
    ///
    /// # Returns
    ///
    /// Vector of volumes, one per element in the entire mesh, ordered by element blocks
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Any block topology doesn't support volume calculation
    /// - Coordinates cannot be read
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use exodus_rs::{ExodusFile, mode};
    ///
    /// # fn main() -> exodus_rs::Result<()> {
    /// let file = ExodusFile::<mode::Read>::open("mesh.exo")?;
    /// let all_volumes = file.all_element_volumes()?;
    /// let total_volume: f64 = all_volumes.iter().sum();
    /// println!("Total mesh volume: {}", total_volume);
    /// # Ok(())
    /// # }
    /// ```
    pub fn all_element_volumes(&self) -> Result<Vec<f64>> {
        use crate::types::EntityType;
        let block_ids = self.block_ids(EntityType::ElemBlock)?;
        let mut all_volumes = Vec::new();

        for block_id in block_ids {
            let volumes = self.block_element_volumes(block_id)?;
            all_volumes.extend(volumes);
        }

        Ok(all_volumes)
    }

    /// Compute centroids for all elements in the mesh.
    ///
    /// This method efficiently computes the centroid of every element in all blocks,
    /// returning a vector indexed by global element number (0-based).
    ///
    /// # Returns
    ///
    /// Vector of centroids as [x, y, z] coordinates, one per element in the entire mesh,
    /// ordered by element blocks
    ///
    /// # Errors
    ///
    /// Returns an error if coordinates cannot be read
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use exodus_rs::{ExodusFile, mode};
    ///
    /// # fn main() -> exodus_rs::Result<()> {
    /// let file = ExodusFile::<mode::Read>::open("mesh.exo")?;
    /// let all_centroids = file.all_element_centroids()?;
    /// println!("Mesh has {} elements", all_centroids.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn all_element_centroids(&self) -> Result<Vec<Vec3>> {
        use crate::types::EntityType;
        let block_ids = self.block_ids(EntityType::ElemBlock)?;
        let mut all_centroids = Vec::new();

        for block_id in block_ids {
            let centroids = self.block_element_centroids(block_id)?;
            all_centroids.extend(centroids);
        }

        Ok(all_centroids)
    }
}

// ============================================================================
// Vector math utilities
// ============================================================================

/// Vector subtraction: a - b
#[inline]
fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// Dot product: a · b
#[inline]
fn dot(a: Vec3, b: Vec3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Cross product: a × b
#[inline]
fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Vector magnitude (length)
#[inline]
fn magnitude(v: Vec3) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

/// Normalize a vector to unit length
#[inline]
fn normalize(v: Vec3) -> Vec3 {
    let mag = magnitude(v);
    if mag < 1e-12 {
        // Degenerate case: zero or near-zero vector
        return [0.0, 0.0, 1.0]; // Return default up vector
    }
    [v[0] / mag, v[1] / mag, v[2] / mag]
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f64 = 1e-10;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < EPSILON
    }

    fn vec_approx_eq(a: Vec3, b: Vec3) -> bool {
        approx_eq(a[0], b[0]) && approx_eq(a[1], b[1]) && approx_eq(a[2], b[2])
    }

    #[test]
    fn test_cross_product() {
        // i × j = k
        let i = [1.0, 0.0, 0.0];
        let j = [0.0, 1.0, 0.0];
        let k = cross(i, j);
        assert!(vec_approx_eq(k, [0.0, 0.0, 1.0]));

        // j × i = -k
        let neg_k = cross(j, i);
        assert!(vec_approx_eq(neg_k, [0.0, 0.0, -1.0]));
    }

    #[test]
    fn test_normalize() {
        let v = [3.0, 4.0, 0.0];
        let n = normalize(v);
        assert!(vec_approx_eq(n, [0.6, 0.8, 0.0]));
        assert!(approx_eq(magnitude(n), 1.0));
    }

    #[test]
    fn test_compute_face_normal() {
        // Square in xy-plane, ccw winding from +z viewpoint
        let coords = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];

        let normal = compute_face_normal(&coords);
        // Should point in +z direction
        assert!(vec_approx_eq(normal, [0.0, 0.0, 1.0]));
    }

    #[test]
    fn test_compute_face_center() {
        let coords = vec![
            [0.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [2.0, 2.0, 0.0],
            [0.0, 2.0, 0.0],
        ];

        let center = compute_face_center(&coords);
        assert!(vec_approx_eq(center, [1.0, 1.0, 0.0]));
    }

    #[test]
    fn test_is_outward_facing() {
        let mesh_center = [0.0, 0.0, 0.0];

        // Face on +x side
        let face_center = [1.0, 0.0, 0.0];
        let face_normal = [1.0, 0.0, 0.0]; // Points in +x
        assert!(is_outward_facing(face_center, face_normal, mesh_center));

        // Same face but normal points inward
        let inward_normal = [-1.0, 0.0, 0.0];
        assert!(!is_outward_facing(face_center, inward_normal, mesh_center));
    }

    #[test]
    fn test_normals_consistent() {
        let n1 = [1.0, 0.0, 0.0];
        let n2 = [1.0, 0.0, 0.0];
        assert!(normals_consistent(n1, n2, 0.9)); // Same direction

        let n3 = [0.0, 1.0, 0.0];
        assert!(!normals_consistent(n1, n3, 0.5)); // Perpendicular

        let n4 = [-1.0, 0.0, 0.0];
        assert!(!normals_consistent(n1, n4, -0.5)); // Opposite
    }

    #[test]
    fn test_average_normals() {
        let normals = vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

        let avg = average_normals(&normals);
        // Should be normalized diagonal
        let expected_val = 1.0 / 3.0_f64.sqrt();
        assert!(vec_approx_eq(
            avg,
            [expected_val, expected_val, expected_val]
        ));
    }

    #[test]
    fn test_dot_product() {
        let a = [1.0, 2.0, 3.0];
        let b = [4.0, 5.0, 6.0];
        assert!(approx_eq(dot(a, b), 32.0)); // 1*4 + 2*5 + 3*6
    }

    #[test]
    fn test_magnitude() {
        let v = [3.0, 4.0, 0.0];
        assert!(approx_eq(magnitude(v), 5.0));
    }

    #[test]
    fn test_tetrahedron_volume() {
        // Unit tetrahedron with base in xy-plane
        let coords = [
            [0.0, 0.0, 0.0], // origin
            [1.0, 0.0, 0.0], // x-axis
            [0.0, 1.0, 0.0], // y-axis
            [0.0, 0.0, 1.0], // z-axis
        ];
        let volume = tetrahedron_volume(&coords);
        // Volume = 1/6 for unit tetrahedron
        assert!(approx_eq(volume, 1.0 / 6.0));
    }

    #[test]
    fn test_hexahedron_volume_unit_cube() {
        // Unit cube
        let coords = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ];
        let volume = hexahedron_volume(&coords);
        assert!(approx_eq(volume, 1.0));
    }

    #[test]
    fn test_hexahedron_volume_scaled() {
        // 2x3x4 box
        let coords = [
            [0.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [2.0, 3.0, 0.0],
            [0.0, 3.0, 0.0],
            [0.0, 0.0, 4.0],
            [2.0, 0.0, 4.0],
            [2.0, 3.0, 4.0],
            [0.0, 3.0, 4.0],
        ];
        let volume = hexahedron_volume(&coords);
        assert!(approx_eq(volume, 24.0)); // 2*3*4
    }

    #[test]
    fn test_wedge_volume() {
        // Unit wedge: triangular base in xy-plane, height 1 in z
        let coords = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [0.0, 1.0, 1.0],
        ];
        let volume = wedge_volume(&coords);
        // Volume = base_area * height = (1/2 * 1 * 1) * 1 = 0.5
        assert!(approx_eq(volume, 0.5));
    }

    #[test]
    fn test_pyramid_volume() {
        // Unit pyramid: square base in xy-plane, apex at (0.5, 0.5, 1)
        let coords = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.5, 0.5, 1.0],
        ];
        let volume = pyramid_volume(&coords);
        // Volume = (1/3) * base_area * height = (1/3) * 1 * 1 = 1/3
        assert!((volume - 1.0 / 3.0).abs() < 0.01); // Allow some numerical error
    }

    #[test]
    fn test_element_volume_hex() {
        use crate::Topology;

        let coords = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ];
        let volume = element_volume(Topology::Hex8, &coords).unwrap();
        assert!(approx_eq(volume, 1.0));
    }

    #[test]
    fn test_element_volume_tet() {
        use crate::Topology;

        let coords = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ];
        let volume = element_volume(Topology::Tet4, &coords).unwrap();
        assert!(approx_eq(volume, 1.0 / 6.0));
    }

    #[test]
    fn test_element_centroid() {
        // Unit cube
        let coords = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ];
        let centroid = element_centroid(&coords);
        assert!(approx_eq(centroid[0], 0.5));
        assert!(approx_eq(centroid[1], 0.5));
        assert!(approx_eq(centroid[2], 0.5));
    }

    #[test]
    fn test_element_centroid_tet() {
        // Tetrahedron
        let coords = vec![
            [0.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
            [0.0, 3.0, 0.0],
            [0.0, 0.0, 3.0],
        ];
        let centroid = element_centroid(&coords);
        // Centroid at (3/4, 3/4, 3/4) for this tet
        assert!(approx_eq(centroid[0], 0.75));
        assert!(approx_eq(centroid[1], 0.75));
        assert!(approx_eq(centroid[2], 0.75));
    }

    #[test]
    #[cfg(feature = "netcdf4")]
    fn test_block_element_volumes() {
        use crate::{
            mode,
            types::{Block, EntityType},
            CreateMode, CreateOptions, ExodusFile, InitParams,
        };
        use tempfile::NamedTempFile;

        // Create a test file with a simple 2-element mesh
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        // Create file and write mesh
        {
            let options = CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            };
            let mut file = ExodusFile::create(path, options).unwrap();

            let params = InitParams {
                title: "Test".into(),
                num_dim: 3,
                num_nodes: 8,
                num_elems: 2,
                num_elem_blocks: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            // Two unit cubes side by side
            let x = vec![0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0];
            let y = vec![0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0];
            let z = vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0];
            file.put_coords(&x, Some(&y), Some(&z)).unwrap();

            // Define block
            let block = Block {
                id: 100,
                entity_type: EntityType::ElemBlock,
                topology: "HEX8".into(),
                num_entries: 2,
                num_nodes_per_entry: 8,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block).unwrap();

            // Two cubes: first is 0-7, second is offset but shares some nodes
            let connectivity = vec![
                1, 2, 3, 4, 5, 6, 7, 8, // First cube
                1, 2, 3, 4, 5, 6, 7, 8, // Second cube (same for simplicity)
            ];
            file.put_connectivity(100, &connectivity).unwrap();
        }

        // Read and compute volumes
        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let volumes = file.block_element_volumes(100).unwrap();

        assert_eq!(volumes.len(), 2);
        // Each cube should have volume 1.0
        assert!(approx_eq(volumes[0], 1.0));
        assert!(approx_eq(volumes[1], 1.0));
    }

    #[test]
    #[cfg(feature = "netcdf4")]
    fn test_block_element_centroids() {
        use crate::{
            mode,
            types::{Block, EntityType},
            CreateMode, CreateOptions, ExodusFile, InitParams,
        };
        use tempfile::NamedTempFile;

        // Create a test file with a simple 2-element mesh
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        // Create file and write mesh
        {
            let options = CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            };
            let mut file = ExodusFile::create(path, options).unwrap();

            let params = InitParams {
                title: "Test".into(),
                num_dim: 3,
                num_nodes: 16,
                num_elems: 2,
                num_elem_blocks: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            // Two unit cubes: one at origin, one offset in x by 2
            let x = vec![
                0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, // First cube
                2.0, 3.0, 3.0, 2.0, 2.0, 3.0, 3.0, 2.0, // Second cube
            ];
            let y = vec![
                0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0,
            ];
            let z = vec![
                0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0,
            ];
            file.put_coords(&x, Some(&y), Some(&z)).unwrap();

            // Define block
            let block = Block {
                id: 100,
                entity_type: EntityType::ElemBlock,
                topology: "HEX8".into(),
                num_entries: 2,
                num_nodes_per_entry: 8,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block).unwrap();

            let connectivity = vec![
                1, 2, 3, 4, 5, 6, 7, 8, // First cube (nodes 1-8)
                9, 10, 11, 12, 13, 14, 15, 16, // Second cube (nodes 9-16)
            ];
            file.put_connectivity(100, &connectivity).unwrap();
        }

        // Read and compute centroids
        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let centroids = file.block_element_centroids(100).unwrap();

        assert_eq!(centroids.len(), 2);
        // First cube centroid at (0.5, 0.5, 0.5)
        assert!(approx_eq(centroids[0][0], 0.5));
        assert!(approx_eq(centroids[0][1], 0.5));
        assert!(approx_eq(centroids[0][2], 0.5));
        // Second cube centroid at (2.5, 0.5, 0.5)
        assert!(approx_eq(centroids[1][0], 2.5));
        assert!(approx_eq(centroids[1][1], 0.5));
        assert!(approx_eq(centroids[1][2], 0.5));
    }

    #[test]
    #[cfg(feature = "netcdf4")]
    fn test_all_element_volumes() {
        use crate::{
            mode,
            types::{Block, EntityType},
            CreateMode, CreateOptions, ExodusFile, InitParams,
        };
        use tempfile::NamedTempFile;

        // Create a test file with two blocks
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        // Create file and write mesh
        {
            let options = CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            };
            let mut file = ExodusFile::create(path, options).unwrap();

            let params = InitParams {
                title: "Test".into(),
                num_dim: 3,
                num_nodes: 12,
                num_elems: 3,
                num_elem_blocks: 2,
                ..Default::default()
            };
            file.init(&params).unwrap();

            // Coordinates for 1 hex and 2 tets
            let x = vec![
                0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, // Hex nodes 1-8
                2.0, 3.0, 2.5, 2.5, // Tet nodes 9-12
            ];
            let y = vec![0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.5];
            let z = vec![
                0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0,
                1.0, // Give the tet some height in z
            ];
            file.put_coords(&x, Some(&y), Some(&z)).unwrap();

            // Block 1: 1 hex
            let block1 = Block {
                id: 100,
                entity_type: EntityType::ElemBlock,
                topology: "HEX8".into(),
                num_entries: 1,
                num_nodes_per_entry: 8,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block1).unwrap();
            file.put_connectivity(100, &[1, 2, 3, 4, 5, 6, 7, 8])
                .unwrap();

            // Block 2: 2 tets
            let block2 = Block {
                id: 200,
                entity_type: EntityType::ElemBlock,
                topology: "TET4".into(),
                num_entries: 2,
                num_nodes_per_entry: 4,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block2).unwrap();
            file.put_connectivity(
                200,
                &[
                    9, 10, 11, 12, // First tet
                    9, 10, 11, 12, // Second tet (same for simplicity)
                ],
            )
            .unwrap();
        }

        // Read and compute all volumes
        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let all_volumes = file.all_element_volumes().unwrap();

        assert_eq!(all_volumes.len(), 3); // 1 hex + 2 tets
                                          // Hex volume should be 1.0
        assert!(approx_eq(all_volumes[0], 1.0));
        // Tets should have positive volumes
        assert!(all_volumes[1] > 0.0);
        assert!(all_volumes[2] > 0.0);
    }

    #[test]
    #[cfg(feature = "netcdf4")]
    fn test_all_element_centroids() {
        use crate::{
            mode,
            types::{Block, EntityType},
            CreateMode, CreateOptions, ExodusFile, InitParams,
        };
        use tempfile::NamedTempFile;

        // Create a test file with two blocks
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_str().unwrap();

        // Create file and write mesh
        {
            let options = CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            };
            let mut file = ExodusFile::create(path, options).unwrap();

            let params = InitParams {
                title: "Test".into(),
                num_dim: 3,
                num_nodes: 12,
                num_elems: 2,
                num_elem_blocks: 2,
                ..Default::default()
            };
            file.init(&params).unwrap();

            // Two unit cubes in different blocks
            let x = vec![
                0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, // First cube
                2.0, 3.0, 3.0, 2.0, // Second cube (only 4 nodes for simplicity)
            ];
            let y = vec![0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0];
            let z = vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0];
            file.put_coords(&x, Some(&y), Some(&z)).unwrap();

            // Block 1: 1 hex
            let block1 = Block {
                id: 100,
                entity_type: EntityType::ElemBlock,
                topology: "HEX8".into(),
                num_entries: 1,
                num_nodes_per_entry: 8,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block1).unwrap();
            file.put_connectivity(100, &[1, 2, 3, 4, 5, 6, 7, 8])
                .unwrap();

            // Block 2: 1 quad (2D)
            let block2 = Block {
                id: 200,
                entity_type: EntityType::ElemBlock,
                topology: "QUAD4".into(),
                num_entries: 1,
                num_nodes_per_entry: 4,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block2).unwrap();
            file.put_connectivity(200, &[9, 10, 11, 12]).unwrap();
        }

        // Read and compute all centroids
        let file = ExodusFile::<mode::Read>::open(path).unwrap();
        let all_centroids = file.all_element_centroids().unwrap();

        assert_eq!(all_centroids.len(), 2); // 1 hex + 1 quad
                                            // First element (hex) centroid at (0.5, 0.5, 0.5)
        assert!(approx_eq(all_centroids[0][0], 0.5));
        assert!(approx_eq(all_centroids[0][1], 0.5));
        assert!(approx_eq(all_centroids[0][2], 0.5));
        // Second element (quad) centroid at (2.5, 0.5, 0.0)
        assert!(approx_eq(all_centroids[1][0], 2.5));
        assert!(approx_eq(all_centroids[1][1], 0.5));
        assert!(approx_eq(all_centroids[1][2], 0.0));
    }
}
