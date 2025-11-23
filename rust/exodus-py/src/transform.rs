//! Transformation operations for Exodus files
//!
//! This module provides Python bindings for spatial transformations of meshes,
//! including translation, rotation, and scaling operations.

use crate::error::IntoPyResult;
use crate::file::ExodusAppender;
use pyo3::prelude::*;

#[pymethods]
impl ExodusAppender {
    /// Translate mesh coordinates by a vector offset
    ///
    /// Args:
    ///     translation: Translation vector [dx, dy, dz]
    ///
    /// Example:
    ///     >>> appender = ExodusAppender.append("mesh.exo")
    ///     >>> appender.translate([10.0, 5.0, 0.0])  # Move 10 units in X, 5 in Y
    fn translate(&mut self, translation: [f64; 3]) -> PyResult<()> {
        self.file_mut()?.translate(&translation).into_py()?;
        Ok(())
    }

    /// Rotate mesh coordinates around the X axis
    ///
    /// Args:
    ///     angle_degrees: Rotation angle in degrees (positive = counterclockwise when looking along +X)
    ///
    /// Example:
    ///     >>> appender = ExodusAppender.append("mesh.exo")
    ///     >>> appender.rotate_x(45.0)  # Rotate 45 degrees around X axis
    fn rotate_x(&mut self, angle_degrees: f64) -> PyResult<()> {
        self.file_mut()?.rotate_x(angle_degrees).into_py()?;
        Ok(())
    }

    /// Rotate mesh coordinates around the Y axis
    ///
    /// Args:
    ///     angle_degrees: Rotation angle in degrees (positive = counterclockwise when looking along +Y)
    ///
    /// Example:
    ///     >>> appender = ExodusAppender.append("mesh.exo")
    ///     >>> appender.rotate_y(90.0)  # Rotate 90 degrees around Y axis
    fn rotate_y(&mut self, angle_degrees: f64) -> PyResult<()> {
        self.file_mut()?.rotate_y(angle_degrees).into_py()?;
        Ok(())
    }

    /// Rotate mesh coordinates around the Z axis
    ///
    /// Args:
    ///     angle_degrees: Rotation angle in degrees (positive = counterclockwise when looking along +Z)
    ///
    /// Example:
    ///     >>> appender = ExodusAppender.append("mesh.exo")
    ///     >>> appender.rotate_z(180.0)  # Rotate 180 degrees around Z axis
    fn rotate_z(&mut self, angle_degrees: f64) -> PyResult<()> {
        self.file_mut()?.rotate_z(angle_degrees).into_py()?;
        Ok(())
    }

    /// Rotate mesh coordinates using Euler angles
    ///
    /// This follows the scipy.spatial.transform.Rotation.from_euler convention.
    ///
    /// Args:
    ///     seq: Euler sequence string (e.g., "XYZ" for extrinsic, "xyz" for intrinsic)
    ///     angles: List of 1-3 rotation angles
    ///     degrees: If True, angles are in degrees; otherwise radians (default: True)
    ///
    /// Example:
    ///     >>> appender = ExodusAppender.append("mesh.exo")
    ///     >>> # Extrinsic XYZ rotation (rotate around fixed axes)
    ///     >>> appender.rotate_euler("XYZ", [30.0, 45.0, 60.0], degrees=True)
    ///     >>> # Intrinsic xyz rotation (rotate around body axes)
    ///     >>> appender.rotate_euler("xyz", [30.0, 45.0, 60.0], degrees=True)
    #[pyo3(signature = (seq, angles, degrees=true))]
    fn rotate_euler(&mut self, seq: &str, angles: Vec<f64>, degrees: bool) -> PyResult<()> {
        self.file_mut()?
            .rotate_euler(seq, &angles, degrees)
            .into_py()?;
        Ok(())
    }

    /// Apply a rotation matrix to mesh coordinates
    ///
    /// Args:
    ///     rotation_matrix: 3x3 rotation matrix as a flat list of 9 values in row-major order
    ///                      [m00, m01, m02, m10, m11, m12, m20, m21, m22]
    ///
    /// Example:
    ///     >>> import math
    ///     >>> appender = ExodusAppender.append("mesh.exo")
    ///     >>> # 90-degree rotation around Z axis
    ///     >>> matrix = [0.0, -1.0, 0.0,
    ///     ...           1.0,  0.0, 0.0,
    ///     ...           0.0,  0.0, 1.0]
    ///     >>> appender.apply_rotation(matrix)
    fn apply_rotation(&mut self, rotation_matrix: [f64; 9]) -> PyResult<()> {
        self.file_mut()?
            .apply_rotation(&rotation_matrix)
            .into_py()?;
        Ok(())
    }

    /// Scale mesh coordinates uniformly
    ///
    /// Args:
    ///     scale_factor: Uniform scale factor (e.g., 2.0 doubles all dimensions)
    ///
    /// Example:
    ///     >>> appender = ExodusAppender.append("mesh.exo")
    ///     >>> appender.scale_uniform(2.0)  # Double all dimensions
    fn scale_uniform(&mut self, scale_factor: f64) -> PyResult<()> {
        self.file_mut()?.scale_uniform(scale_factor).into_py()?;
        Ok(())
    }

    /// Scale mesh coordinates with different factors per axis
    ///
    /// Args:
    ///     scale_factors: Scale factors [sx, sy, sz] for each axis
    ///
    /// Example:
    ///     >>> appender = ExodusAppender.append("mesh.exo")
    ///     >>> appender.scale([2.0, 1.0, 0.5])  # Double X, keep Y, halve Z
    fn scale(&mut self, scale_factors: [f64; 3]) -> PyResult<()> {
        self.file_mut()?.scale(&scale_factors).into_py()?;
        Ok(())
    }
}
