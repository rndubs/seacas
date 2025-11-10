//! Coordinate operations for Exodus files

use pyo3::prelude::*;
use crate::error::IntoPyResult;
use crate::file::{ExodusWriter, ExodusAppender, ExodusReader};

#[pymethods]
impl ExodusWriter {
    /// Write nodal coordinates to the file
    ///
    /// Args:
    ///     x: X coordinates (required)
    ///     y: Y coordinates (optional for 1D, required for 2D/3D)
    ///     z: Z coordinates (optional for 1D/2D, required for 3D)
    ///
    /// Example:
    ///     >>> writer.put_coords(
    ///     ...     x=[0.0, 1.0, 1.0, 0.0],
    ///     ...     y=[0.0, 0.0, 1.0, 1.0],
    ///     ...     z=[]
    ///     ... )
    #[pyo3(signature = (x, y=None, z=None))]
    fn put_coords(
        &mut self,
        x: Vec<f64>,
        y: Option<Vec<f64>>,
        z: Option<Vec<f64>>,
    ) -> PyResult<()> {
        let y_slice = y.as_deref();
        let z_slice = z.as_deref();

        self.file_mut()?
            .put_coords(&x, y_slice, z_slice)
            .into_py()?;
        Ok(())
    }
}

#[pymethods]
impl ExodusAppender {
    /// Write nodal coordinates to the file
    #[pyo3(signature = (x, y=None, z=None))]
    fn put_coords(
        &mut self,
        x: Vec<f64>,
        y: Option<Vec<f64>>,
        z: Option<Vec<f64>>,
    ) -> PyResult<()> {
        let y_slice = y.as_deref();
        let z_slice = z.as_deref();

        self.file_mut()?
            .put_coords(&x, y_slice, z_slice)
            .into_py()?;
        Ok(())
    }

    /// Read nodal coordinates
    ///
    /// Returns:
    ///     Tuple of (x, y, z) coordinate arrays
    fn get_coords(&self) -> PyResult<(Vec<f64>, Vec<f64>, Vec<f64>)> {
        let coords = self.file_ref()?.coords::<f64>().into_py()?;
        Ok((coords.x, coords.y, coords.z))
    }
}

#[pymethods]
impl ExodusReader {
    /// Read nodal coordinates
    ///
    /// Returns:
    ///     Tuple of (x, y, z) coordinate arrays
    ///
    /// Example:
    ///     >>> x, y, z = reader.get_coords()
    fn get_coords(&self) -> PyResult<(Vec<f64>, Vec<f64>, Vec<f64>)> {
        let coords = self.file_ref().coords::<f64>().into_py()?;
        Ok((coords.x, coords.y, coords.z))
    }

    /// Read only X coordinates
    ///
    /// Returns:
    ///     List of X coordinates
    ///
    /// Example:
    ///     >>> x = reader.get_coord_x()
    fn get_coord_x(&self) -> PyResult<Vec<f64>> {
        self.file_ref().get_coord_x::<f64>().into_py()
    }

    /// Read only Y coordinates
    ///
    /// Returns:
    ///     List of Y coordinates
    ///
    /// Example:
    ///     >>> y = reader.get_coord_y()
    fn get_coord_y(&self) -> PyResult<Vec<f64>> {
        self.file_ref().get_coord_y::<f64>().into_py()
    }

    /// Read only Z coordinates
    ///
    /// Returns:
    ///     List of Z coordinates
    ///
    /// Example:
    ///     >>> z = reader.get_coord_z()
    fn get_coord_z(&self) -> PyResult<Vec<f64>> {
        self.file_ref().get_coord_z::<f64>().into_py()
    }
}
