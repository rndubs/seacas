//! Coordinate and field transformations for Exodus meshes.
//!
//! This module provides utilities for applying spatial transformations to meshes
//! and their associated field data (vectors, tensors). Transformations can be applied
//! to coordinate systems and propagated to field variables to maintain consistency.
//!
//! # Supported Transformations
//!
//! - **Translation**: Move mesh by a vector offset
//! - **Rotation**: Rotate mesh around X, Y, Z axes or using Euler angles
//! - **Scaling**: Uniform or non-uniform scaling
//! - **Field Transformations**: Apply rotations to vector and tensor fields
//!
//! # Memory Efficiency
//!
//! All transformations process data in-place where possible to minimize memory usage.
//! For large datasets with time-history data, transformations are applied one time step
//! at a time to avoid loading all data into memory simultaneously.
//!
//! # Examples
//!
//! ```rust,ignore
//! use exodus_rs::ExodusFile;
//!
//! // Open file in append mode
//! let mut file = ExodusFile::open_append("mesh.exo")?;
//!
//! // Translate mesh by 10 units in X direction
//! file.translate(&[10.0, 0.0, 0.0])?;
//!
//! // Rotate 45 degrees around Z axis
//! file.rotate_z(45.0)?;
//!
//! // Rotate using Euler angles (extrinsic XYZ sequence)
//! file.rotate_euler("XYZ", &[30.0, 45.0, 60.0], true)?;
//!
//! // Scale uniformly by factor of 2
//! file.scale_uniform(2.0)?;
//! # Ok::<(), exodus_rs::ExodusError>(())
//! ```

use crate::error::{ExodusError, Result};
use std::f64::consts::PI;

/// A 3x3 rotation matrix in row-major order
///
/// Stored as a flat array: [m00, m01, m02, m10, m11, m12, m20, m21, m22]
pub type Matrix3x3 = [f64; 9];

/// Convert degrees to radians
#[inline]
pub fn deg_to_rad(degrees: f64) -> f64 {
    degrees * PI / 180.0
}

/// Create a rotation matrix for rotation around the X axis
///
/// # Arguments
///
/// * `angle_rad` - Rotation angle in radians (positive = counterclockwise when looking along +X)
///
/// # Returns
///
/// A 3x3 rotation matrix
pub fn rotation_matrix_x(angle_rad: f64) -> Matrix3x3 {
    let c = angle_rad.cos();
    let s = angle_rad.sin();

    [1.0, 0.0, 0.0, 0.0, c, -s, 0.0, s, c]
}

/// Create a rotation matrix for rotation around the Y axis
///
/// # Arguments
///
/// * `angle_rad` - Rotation angle in radians (positive = counterclockwise when looking along +Y)
///
/// # Returns
///
/// A 3x3 rotation matrix
pub fn rotation_matrix_y(angle_rad: f64) -> Matrix3x3 {
    let c = angle_rad.cos();
    let s = angle_rad.sin();

    [c, 0.0, s, 0.0, 1.0, 0.0, -s, 0.0, c]
}

/// Create a rotation matrix for rotation around the Z axis
///
/// # Arguments
///
/// * `angle_rad` - Rotation angle in radians (positive = counterclockwise when looking along +Z)
///
/// # Returns
///
/// A 3x3 rotation matrix
pub fn rotation_matrix_z(angle_rad: f64) -> Matrix3x3 {
    let c = angle_rad.cos();
    let s = angle_rad.sin();

    [c, -s, 0.0, s, c, 0.0, 0.0, 0.0, 1.0]
}

/// Multiply two 3x3 matrices: result = a * b
///
/// # Arguments
///
/// * `a` - Left matrix
/// * `b` - Right matrix
///
/// # Returns
///
/// The product matrix a * b
pub fn multiply_matrices(a: &Matrix3x3, b: &Matrix3x3) -> Matrix3x3 {
    let mut result = [0.0; 9];

    for i in 0..3 {
        for j in 0..3 {
            let mut sum = 0.0;
            for k in 0..3 {
                sum += a[i * 3 + k] * b[k * 3 + j];
            }
            result[i * 3 + j] = sum;
        }
    }

    result
}

/// Apply a rotation matrix to a 3D vector
///
/// # Arguments
///
/// * `matrix` - The 3x3 rotation matrix
/// * `vec` - The input vector [x, y, z]
///
/// # Returns
///
/// The rotated vector
pub fn apply_rotation_to_vector(matrix: &Matrix3x3, vec: &[f64; 3]) -> [f64; 3] {
    [
        matrix[0] * vec[0] + matrix[1] * vec[1] + matrix[2] * vec[2],
        matrix[3] * vec[0] + matrix[4] * vec[1] + matrix[5] * vec[2],
        matrix[6] * vec[0] + matrix[7] * vec[1] + matrix[8] * vec[2],
    ]
}

/// Euler angle sequence type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EulerSequence {
    /// Intrinsic rotations (lowercase axes: 'x', 'y', 'z')
    /// Rotations are applied in the body frame
    Intrinsic {
        /// Axes of rotation in order
        axes: [char; 3],
    },

    /// Extrinsic rotations (uppercase axes: 'X', 'Y', 'Z')
    /// Rotations are applied in the fixed frame
    Extrinsic {
        /// Axes of rotation in order
        axes: [char; 3],
    },
}

impl EulerSequence {
    /// Parse an Euler sequence string
    ///
    /// # Arguments
    ///
    /// * `seq` - Sequence string (e.g., "XYZ" for extrinsic, "xyz" for intrinsic)
    ///
    /// # Returns
    ///
    /// Parsed EulerSequence or error if invalid
    ///
    /// # Errors
    ///
    /// - Invalid sequence length (must be 1-3 characters)
    /// - Mixed case (cannot mix intrinsic and extrinsic)
    /// - Invalid axis characters (must be X/Y/Z or x/y/z)
    pub fn parse(seq: &str) -> Result<Self> {
        let chars: Vec<char> = seq.chars().collect();

        if chars.is_empty() || chars.len() > 3 {
            return Err(ExodusError::Other(format!(
                "Euler sequence must be 1-3 characters, got {}",
                chars.len()
            )));
        }

        // Check if all uppercase (extrinsic) or all lowercase (intrinsic)
        let all_upper = chars.iter().all(|c| c.is_uppercase());
        let all_lower = chars.iter().all(|c| c.is_lowercase());

        if !all_upper && !all_lower {
            return Err(ExodusError::Other(
                "Cannot mix intrinsic (lowercase) and extrinsic (uppercase) rotations".to_string(),
            ));
        }

        // Validate axes
        for c in &chars {
            let upper = c.to_uppercase().next().unwrap();
            if upper != 'X' && upper != 'Y' && upper != 'Z' {
                return Err(ExodusError::Other(format!(
                    "Invalid axis '{}', must be X/Y/Z or x/y/z",
                    c
                )));
            }
        }

        // Pad with identity if less than 3 rotations
        let mut axes = ['X'; 3];
        for (i, c) in chars.iter().enumerate() {
            axes[i] = c.to_uppercase().next().unwrap();
        }

        // Fill remaining slots with 'X' but mark angles as 0
        if all_upper {
            Ok(EulerSequence::Extrinsic { axes })
        } else {
            Ok(EulerSequence::Intrinsic { axes })
        }
    }
}

/// Create a rotation matrix from Euler angles
///
/// This follows the scipy convention for Euler angle rotations.
///
/// # Arguments
///
/// * `seq` - Euler sequence (e.g., "XYZ" for extrinsic, "xyz" for intrinsic)
/// * `angles` - Array of 1-3 rotation angles
/// * `degrees` - If true, angles are in degrees; otherwise radians
///
/// # Returns
///
/// A 3x3 rotation matrix representing the composed rotation
///
/// # Errors
///
/// - Invalid sequence string
/// - Wrong number of angles for the sequence
///
/// # Examples
///
/// ```rust,ignore
/// use exodus_rs::transformations::rotation_matrix_from_euler;
///
/// // Extrinsic XYZ rotation: rotate 30° around X, then 45° around Y, then 60° around Z
/// let matrix = rotation_matrix_from_euler("XYZ", &[30.0, 45.0, 60.0], true)?;
///
/// // Intrinsic xyz rotation: rotate around x, then new y, then new z
/// let matrix = rotation_matrix_from_euler("xyz", &[30.0, 45.0, 60.0], true)?;
/// # Ok::<(), exodus_rs::ExodusError>(())
/// ```
pub fn rotation_matrix_from_euler(seq: &str, angles: &[f64], degrees: bool) -> Result<Matrix3x3> {
    let euler_seq = EulerSequence::parse(seq)?;

    // Validate angles length
    let seq_len = seq.len();
    if angles.len() != seq_len {
        return Err(ExodusError::Other(format!(
            "Expected {} angles for sequence '{}', got {}",
            seq_len,
            seq,
            angles.len()
        )));
    }

    // Convert angles to radians if needed
    let angles_rad: Vec<f64> = if degrees {
        angles.iter().map(|&a| deg_to_rad(a)).collect()
    } else {
        angles.to_vec()
    };

    // Pad angles with zeros if less than 3
    let mut full_angles = [0.0; 3];
    for (i, &angle) in angles_rad.iter().enumerate() {
        full_angles[i] = angle;
    }

    match euler_seq {
        EulerSequence::Extrinsic { axes } => {
            // Extrinsic rotations: compose right-to-left (apply in reverse order)
            let mut result = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]; // Identity matrix

            for i in (0..seq_len).rev() {
                let matrix = match axes[i] {
                    'X' => rotation_matrix_x(full_angles[i]),
                    'Y' => rotation_matrix_y(full_angles[i]),
                    'Z' => rotation_matrix_z(full_angles[i]),
                    _ => unreachable!(),
                };
                result = multiply_matrices(&matrix, &result);
            }

            Ok(result)
        }
        EulerSequence::Intrinsic { axes } => {
            // Intrinsic rotations: compose left-to-right
            let mut result = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]; // Identity matrix

            for i in 0..seq_len {
                let matrix = match axes[i] {
                    'X' => rotation_matrix_x(full_angles[i]),
                    'Y' => rotation_matrix_y(full_angles[i]),
                    'Z' => rotation_matrix_z(full_angles[i]),
                    _ => unreachable!(),
                };
                result = multiply_matrices(&result, &matrix);
            }

            Ok(result)
        }
    }
}

/// Apply a 3x3 rotation matrix to a 3x3 symmetric tensor (e.g., stress, strain)
///
/// For a symmetric tensor T and rotation matrix R, computes: R * T * R^T
///
/// # Arguments
///
/// * `rotation` - The 3x3 rotation matrix
/// * `tensor` - The symmetric tensor in Voigt notation [T11, T22, T33, T12, T23, T13]
///
/// # Returns
///
/// The rotated tensor in Voigt notation
///
/// # Note
///
/// The input tensor is assumed to be symmetric. The Voigt notation is:
/// - tensor[0] = T11 (XX component)
/// - tensor[1] = T22 (YY component)
/// - tensor[2] = T33 (ZZ component)
/// - tensor[3] = T12 (XY component)
/// - tensor[4] = T23 (YZ component)
/// - tensor[5] = T13 (XZ component)
pub fn rotate_symmetric_tensor(rotation: &Matrix3x3, tensor: &[f64; 6]) -> [f64; 6] {
    // Convert Voigt notation to full 3x3 matrix
    let t = [
        tensor[0], tensor[3], tensor[5], tensor[3], tensor[1], tensor[4], tensor[5], tensor[4],
        tensor[2],
    ];

    // Compute R * T
    let mut rt = [0.0; 9];
    for i in 0..3 {
        for j in 0..3 {
            let mut sum = 0.0;
            for k in 0..3 {
                sum += rotation[i * 3 + k] * t[k * 3 + j];
            }
            rt[i * 3 + j] = sum;
        }
    }

    // Compute (R * T) * R^T
    let mut result_matrix = [0.0; 9];
    for i in 0..3 {
        for j in 0..3 {
            let mut sum = 0.0;
            for k in 0..3 {
                // R^T[j][k] = R[k][j]
                sum += rt[i * 3 + k] * rotation[j * 3 + k];
            }
            result_matrix[i * 3 + j] = sum;
        }
    }

    // Convert back to Voigt notation
    [
        result_matrix[0], // T11
        result_matrix[4], // T22
        result_matrix[8], // T33
        result_matrix[1], // T12
        result_matrix[5], // T23
        result_matrix[2], // T13
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_deg_to_rad() {
        assert_relative_eq!(deg_to_rad(0.0), 0.0);
        assert_relative_eq!(deg_to_rad(90.0), PI / 2.0);
        assert_relative_eq!(deg_to_rad(180.0), PI);
        assert_relative_eq!(deg_to_rad(360.0), 2.0 * PI);
    }

    #[test]
    fn test_rotation_matrix_x() {
        let mat = rotation_matrix_x(PI / 2.0); // 90 degrees
        let vec = [1.0, 0.0, 0.0];
        let result = apply_rotation_to_vector(&mat, &vec);
        assert_relative_eq!(result[0], 1.0, epsilon = 1e-10);
        assert_relative_eq!(result[1], 0.0, epsilon = 1e-10);
        assert_relative_eq!(result[2], 0.0, epsilon = 1e-10);

        let vec = [0.0, 1.0, 0.0];
        let result = apply_rotation_to_vector(&mat, &vec);
        assert_relative_eq!(result[0], 0.0, epsilon = 1e-10);
        assert_relative_eq!(result[1], 0.0, epsilon = 1e-10);
        assert_relative_eq!(result[2], 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_rotation_matrix_y() {
        let mat = rotation_matrix_y(PI / 2.0); // 90 degrees
        let vec = [1.0, 0.0, 0.0];
        let result = apply_rotation_to_vector(&mat, &vec);
        assert_relative_eq!(result[0], 0.0, epsilon = 1e-10);
        assert_relative_eq!(result[1], 0.0, epsilon = 1e-10);
        assert_relative_eq!(result[2], -1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_rotation_matrix_z() {
        let mat = rotation_matrix_z(PI / 2.0); // 90 degrees
        let vec = [1.0, 0.0, 0.0];
        let result = apply_rotation_to_vector(&mat, &vec);
        assert_relative_eq!(result[0], 0.0, epsilon = 1e-10);
        assert_relative_eq!(result[1], 1.0, epsilon = 1e-10);
        assert_relative_eq!(result[2], 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_euler_sequence_parse() {
        let seq = EulerSequence::parse("XYZ").unwrap();
        assert!(matches!(seq, EulerSequence::Extrinsic { .. }));

        let seq = EulerSequence::parse("xyz").unwrap();
        assert!(matches!(seq, EulerSequence::Intrinsic { .. }));

        // Should fail on mixed case
        assert!(EulerSequence::parse("XyZ").is_err());

        // Should fail on invalid characters
        assert!(EulerSequence::parse("ABC").is_err());
    }

    #[test]
    fn test_rotation_from_euler_extrinsic() {
        // Simple single-axis rotation
        let mat = rotation_matrix_from_euler("Z", &[90.0], true).unwrap();
        let expected = rotation_matrix_z(PI / 2.0);
        for i in 0..9 {
            assert_relative_eq!(mat[i], expected[i], epsilon = 1e-10);
        }
    }

    #[test]
    fn test_rotation_from_euler_intrinsic() {
        // Intrinsic rotation should match single axis for one angle
        let mat = rotation_matrix_from_euler("z", &[90.0], true).unwrap();
        let expected = rotation_matrix_z(PI / 2.0);
        for i in 0..9 {
            assert_relative_eq!(mat[i], expected[i], epsilon = 1e-10);
        }
    }

    #[test]
    fn test_multiply_matrices() {
        // Identity * Identity = Identity
        let identity = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
        let result = multiply_matrices(&identity, &identity);
        for i in 0..9 {
            assert_relative_eq!(result[i], identity[i], epsilon = 1e-10);
        }
    }

    #[test]
    fn test_rotate_symmetric_tensor() {
        // Identity rotation should not change tensor
        let identity = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
        let tensor = [1.0, 2.0, 3.0, 0.5, 0.6, 0.7];
        let result = rotate_symmetric_tensor(&identity, &tensor);
        for i in 0..6 {
            assert_relative_eq!(result[i], tensor[i], epsilon = 1e-10);
        }
    }

    #[test]
    fn test_rotate_diagonal_tensor() {
        // 90-degree rotation around Z axis on diagonal tensor
        let mat = rotation_matrix_z(PI / 2.0);
        let tensor = [1.0, 2.0, 3.0, 0.0, 0.0, 0.0]; // Diagonal tensor
        let result = rotate_symmetric_tensor(&mat, &tensor);

        // After 90° rotation around Z: XX becomes YY, YY becomes XX
        assert_relative_eq!(result[0], 2.0, epsilon = 1e-10); // New XX (was YY)
        assert_relative_eq!(result[1], 1.0, epsilon = 1e-10); // New YY (was XX)
        assert_relative_eq!(result[2], 3.0, epsilon = 1e-10); // ZZ unchanged
    }
}
