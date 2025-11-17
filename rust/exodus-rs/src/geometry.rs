//! Geometric utilities for mesh analysis and normal computation.
//!
//! This module provides functions for:
//! - Computing face normals using the cross product
//! - Calculating mesh center of mass
//! - Verifying normal orientations
//! - Checking normal consistency

use crate::{mode, EntityType, ExodusFile, Result};

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
    let num_nodes = coords.0.len();

    if num_nodes == 0 {
        return Ok([0.0, 0.0, 0.0]);
    }

    let sum_x: f64 = coords.0.iter().sum();
    let sum_y: f64 = coords.1.iter().sum();
    let sum_z: f64 = coords.2.iter().sum();

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
        assert!(vec_approx_eq(avg, [expected_val, expected_val, expected_val]));
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
}
