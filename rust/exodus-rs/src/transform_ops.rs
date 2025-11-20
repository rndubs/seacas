//! High-level transformation operations for ExodusFile
//!
//! This module provides transformation methods on ExodusFile for translating,
//! rotating, and scaling mesh coordinates.

use crate::error::Result;
use crate::file::ExodusFile;
use crate::mode;
use crate::transformations::{
    apply_rotation_to_vector, rotation_matrix_from_euler, rotation_matrix_x, rotation_matrix_y,
    rotation_matrix_z, Matrix3x3,
};
use std::f64::consts::PI;

/// Convert degrees to radians
#[inline]
fn deg_to_rad(degrees: f64) -> f64 {
    degrees * PI / 180.0
}

/// Methods for transforming mesh coordinates
///
/// These methods are only available in Append mode since they modify the file
impl ExodusFile<mode::Append> {
    /// Translate mesh coordinates by a vector offset
    ///
    /// # Arguments
    ///
    /// * `translation` - Translation vector [dx, dy, dz]
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    ///
    /// # Errors
    ///
    /// - File not initialized
    /// - Coordinate read/write errors
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use exodus_rs::ExodusFile;
    ///
    /// let mut file = ExodusFile::append("mesh.exo")?;
    /// file.translate(&[10.0, 5.0, 0.0])?; // Translate 10 units in X, 5 in Y
    /// # Ok::<(), exodus_rs::ExodusError>(())
    /// ```
    pub fn translate(&mut self, translation: &[f64; 3]) -> Result<()> {
        // Read current coordinates
        let mut coords = self.coords::<f64>()?;

        // Apply translation
        for i in 0..coords.x.len() {
            coords.x[i] += translation[0];
            coords.y[i] += translation[1];
            coords.z[i] += translation[2];
        }

        // Write back
        self.put_coords(&coords.x, Some(&coords.y), Some(&coords.z))?;

        Ok(())
    }

    /// Rotate mesh coordinates around the X axis
    ///
    /// # Arguments
    ///
    /// * `angle_degrees` - Rotation angle in degrees (positive = counterclockwise when looking along +X)
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    ///
    /// # Errors
    ///
    /// - File not initialized
    /// - Coordinate read/write errors
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use exodus_rs::ExodusFile;
    ///
    /// let mut file = ExodusFile::append("mesh.exo")?;
    /// file.rotate_x(45.0)?; // Rotate 45 degrees around X axis
    /// # Ok::<(), exodus_rs::ExodusError>(())
    /// ```
    pub fn rotate_x(&mut self, angle_degrees: f64) -> Result<()> {
        let matrix = rotation_matrix_x(deg_to_rad(angle_degrees));
        self.apply_rotation(&matrix)
    }

    /// Rotate mesh coordinates around the Y axis
    ///
    /// # Arguments
    ///
    /// * `angle_degrees` - Rotation angle in degrees (positive = counterclockwise when looking along +Y)
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    ///
    /// # Errors
    ///
    /// - File not initialized
    /// - Coordinate read/write errors
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use exodus_rs::ExodusFile;
    ///
    /// let mut file = ExodusFile::append("mesh.exo")?;
    /// file.rotate_y(90.0)?; // Rotate 90 degrees around Y axis
    /// # Ok::<(), exodus_rs::ExodusError>(())
    /// ```
    pub fn rotate_y(&mut self, angle_degrees: f64) -> Result<()> {
        let matrix = rotation_matrix_y(deg_to_rad(angle_degrees));
        self.apply_rotation(&matrix)
    }

    /// Rotate mesh coordinates around the Z axis
    ///
    /// # Arguments
    ///
    /// * `angle_degrees` - Rotation angle in degrees (positive = counterclockwise when looking along +Z)
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    ///
    /// # Errors
    ///
    /// - File not initialized
    /// - Coordinate read/write errors
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use exodus_rs::ExodusFile;
    ///
    /// let mut file = ExodusFile::append("mesh.exo")?;
    /// file.rotate_z(180.0)?; // Rotate 180 degrees around Z axis
    /// # Ok::<(), exodus_rs::ExodusError>(())
    /// ```
    pub fn rotate_z(&mut self, angle_degrees: f64) -> Result<()> {
        let matrix = rotation_matrix_z(deg_to_rad(angle_degrees));
        self.apply_rotation(&matrix)
    }

    /// Rotate mesh coordinates using Euler angles
    ///
    /// This follows the scipy.spatial.transform.Rotation.from_euler convention.
    ///
    /// # Arguments
    ///
    /// * `seq` - Euler sequence string (e.g., "XYZ" for extrinsic, "xyz" for intrinsic)
    /// * `angles` - Array of 1-3 rotation angles
    /// * `degrees` - If true, angles are in degrees; otherwise radians
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    ///
    /// # Errors
    ///
    /// - Invalid sequence string
    /// - Wrong number of angles for the sequence
    /// - Coordinate read/write errors
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use exodus_rs::ExodusFile;
    ///
    /// let mut file = ExodusFile::append("mesh.exo")?;
    ///
    /// // Extrinsic XYZ rotation (rotate around fixed axes)
    /// file.rotate_euler("XYZ", &[30.0, 45.0, 60.0], true)?;
    ///
    /// // Intrinsic xyz rotation (rotate around body axes)
    /// file.rotate_euler("xyz", &[30.0, 45.0, 60.0], true)?;
    /// # Ok::<(), exodus_rs::ExodusError>(())
    /// ```
    pub fn rotate_euler(&mut self, seq: &str, angles: &[f64], degrees: bool) -> Result<()> {
        let matrix = rotation_matrix_from_euler(seq, angles, degrees)?;
        self.apply_rotation(&matrix)
    }

    /// Apply a rotation matrix to mesh coordinates
    ///
    /// # Arguments
    ///
    /// * `rotation_matrix` - 3x3 rotation matrix
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    ///
    /// # Errors
    ///
    /// - File not initialized
    /// - Coordinate read/write errors
    pub fn apply_rotation(&mut self, rotation_matrix: &Matrix3x3) -> Result<()> {
        // Read current coordinates
        let coords = self.coords::<f64>()?;

        // Apply rotation to each point
        let num_nodes = coords.x.len();
        let mut new_x = Vec::with_capacity(num_nodes);
        let mut new_y = Vec::with_capacity(num_nodes);
        let mut new_z = Vec::with_capacity(num_nodes);

        for i in 0..num_nodes {
            let point = [coords.x[i], coords.y[i], coords.z[i]];
            let rotated = apply_rotation_to_vector(rotation_matrix, &point);
            new_x.push(rotated[0]);
            new_y.push(rotated[1]);
            new_z.push(rotated[2]);
        }

        // Write back
        self.put_coords(&new_x, Some(&new_y), Some(&new_z))?;

        Ok(())
    }

    /// Scale mesh coordinates uniformly
    ///
    /// # Arguments
    ///
    /// * `scale_factor` - Uniform scale factor (e.g., 2.0 doubles all dimensions)
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    ///
    /// # Errors
    ///
    /// - File not initialized
    /// - Coordinate read/write errors
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use exodus_rs::ExodusFile;
    ///
    /// let mut file = ExodusFile::append("mesh.exo")?;
    /// file.scale_uniform(2.0)?; // Double all dimensions
    /// # Ok::<(), exodus_rs::ExodusError>(())
    /// ```
    pub fn scale_uniform(&mut self, scale_factor: f64) -> Result<()> {
        self.scale(&[scale_factor, scale_factor, scale_factor])
    }

    /// Scale mesh coordinates with different factors per axis
    ///
    /// # Arguments
    ///
    /// * `scale_factors` - Scale factors [sx, sy, sz] for each axis
    ///
    /// # Returns
    ///
    /// Ok(()) on success
    ///
    /// # Errors
    ///
    /// - File not initialized
    /// - Coordinate read/write errors
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use exodus_rs::ExodusFile;
    ///
    /// let mut file = ExodusFile::append("mesh.exo")?;
    /// file.scale(&[2.0, 1.0, 0.5])?; // Double X, keep Y, halve Z
    /// # Ok::<(), exodus_rs::ExodusError>(())
    /// ```
    pub fn scale(&mut self, scale_factors: &[f64; 3]) -> Result<()> {
        // Read current coordinates
        let mut coords = self.coords::<f64>()?;

        // Apply scaling
        for i in 0..coords.x.len() {
            coords.x[i] *= scale_factors[0];
            coords.y[i] *= scale_factors[1];
            coords.z[i] *= scale_factors[2];
        }

        // Write back
        self.put_coords(&coords.x, Some(&coords.y), Some(&coords.z))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{CreateMode, CreateOptions, InitParams};
    use crate::ExodusFile;
    use approx::assert_relative_eq;
    use tempfile::NamedTempFile;

    #[test]
    fn test_translate() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        // Create a file with some coordinates
        {
            let mut file = ExodusFile::create(
                path,
                CreateOptions {
                    mode: CreateMode::Clobber,
                    ..Default::default()
                },
            )
            .unwrap();

            let params = InitParams {
                num_dim: 3,
                num_nodes: 3,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let x = vec![0.0, 1.0, 2.0];
            let y = vec![0.0, 1.0, 2.0];
            let z = vec![0.0, 1.0, 2.0];
            file.put_coords(&x, Some(&y), Some(&z)).unwrap();
        }

        // Open in append mode and translate
        {
            let mut file = ExodusFile::append(path).unwrap();
            file.translate(&[10.0, 20.0, 30.0]).unwrap();

            let coords = file.coords::<f64>().unwrap();
            assert_relative_eq!(coords.x[0], 10.0, epsilon = 1e-10);
            assert_relative_eq!(coords.y[0], 20.0, epsilon = 1e-10);
            assert_relative_eq!(coords.z[0], 30.0, epsilon = 1e-10);

            assert_relative_eq!(coords.x[1], 11.0, epsilon = 1e-10);
            assert_relative_eq!(coords.y[1], 21.0, epsilon = 1e-10);
            assert_relative_eq!(coords.z[1], 31.0, epsilon = 1e-10);
        }
    }

    #[test]
    fn test_rotate_z() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        // Create a file with a point on the X axis
        {
            let mut file = ExodusFile::create(
                path,
                CreateOptions {
                    mode: CreateMode::Clobber,
                    ..Default::default()
                },
            )
            .unwrap();

            let params = InitParams {
                num_dim: 3,
                num_nodes: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let x = vec![1.0];
            let y = vec![0.0];
            let z = vec![0.0];
            file.put_coords(&x, Some(&y), Some(&z)).unwrap();
        }

        // Rotate 90 degrees around Z
        {
            let mut file = ExodusFile::append(path).unwrap();
            file.rotate_z(90.0).unwrap();

            let coords = file.coords::<f64>().unwrap();
            assert_relative_eq!(coords.x[0], 0.0, epsilon = 1e-10);
            assert_relative_eq!(coords.y[0], 1.0, epsilon = 1e-10);
            assert_relative_eq!(coords.z[0], 0.0, epsilon = 1e-10);
        }
    }

    #[test]
    fn test_scale_uniform() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        // Create a file with some coordinates
        {
            let mut file = ExodusFile::create(
                path,
                CreateOptions {
                    mode: CreateMode::Clobber,
                    ..Default::default()
                },
            )
            .unwrap();

            let params = InitParams {
                num_dim: 3,
                num_nodes: 2,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let x = vec![1.0, 2.0];
            let y = vec![3.0, 4.0];
            let z = vec![5.0, 6.0];
            file.put_coords(&x, Some(&y), Some(&z)).unwrap();
        }

        // Scale by 2
        {
            let mut file = ExodusFile::append(path).unwrap();
            file.scale_uniform(2.0).unwrap();

            let coords = file.coords::<f64>().unwrap();
            assert_relative_eq!(coords.x[0], 2.0, epsilon = 1e-10);
            assert_relative_eq!(coords.y[0], 6.0, epsilon = 1e-10);
            assert_relative_eq!(coords.z[0], 10.0, epsilon = 1e-10);
        }
    }

    #[test]
    fn test_rotate_euler() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        // Create a file with a point
        {
            let mut file = ExodusFile::create(
                path,
                CreateOptions {
                    mode: CreateMode::Clobber,
                    ..Default::default()
                },
            )
            .unwrap();

            let params = InitParams {
                num_dim: 3,
                num_nodes: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let x = vec![1.0];
            let y = vec![0.0];
            let z = vec![0.0];
            file.put_coords(&x, Some(&y), Some(&z)).unwrap();
        }

        // Rotate using Euler angles
        {
            let mut file = ExodusFile::append(path).unwrap();
            file.rotate_euler("Z", &[90.0], true).unwrap();

            let coords = file.coords::<f64>().unwrap();
            assert_relative_eq!(coords.x[0], 0.0, epsilon = 1e-10);
            assert_relative_eq!(coords.y[0], 1.0, epsilon = 1e-10);
        }
    }
}
