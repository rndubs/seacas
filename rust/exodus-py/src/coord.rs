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

    /// Write coordinate names
    ///
    /// Args:
    ///     names: List of coordinate names (e.g., ["X", "Y", "Z"])
    ///
    /// Example:
    ///     >>> writer.put_coord_names(["X", "Y", "Z"])
    fn put_coord_names(&mut self, names: Vec<String>) -> PyResult<()> {
        let name_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        self.file_mut()?.put_coord_names(&name_refs).into_py()?;
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

    /// Write coordinate names
    fn put_coord_names(&mut self, names: Vec<String>) -> PyResult<()> {
        let name_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        self.file_mut()?.put_coord_names(&name_refs).into_py()?;
        Ok(())
    }

    /// Read nodal coordinates
    ///
    /// Returns:
    ///     Tuple of (x, y, z) coordinate arrays
    fn get_coords(&self) -> PyResult<(Vec<f64>, Vec<f64>, Vec<f64>)> {
        let coords = self.file.as_ref()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("File closed"))?
            .get_coords()
            .into_py()?;

        Ok((coords.x, coords.y, coords.z))
    }

    /// Read coordinate names
    fn get_coord_names(&self) -> PyResult<Vec<String>> {
        self.file.as_ref()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("File closed"))?
            .get_coord_names()
            .into_py()
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
        let coords = self.file_ref().get_coords().into_py()?;
        Ok((coords.x, coords.y, coords.z))
    }

    /// Read coordinate names
    ///
    /// Returns:
    ///     List of coordinate names
    fn get_coord_names(&self) -> PyResult<Vec<String>> {
        self.file_ref().get_coord_names().into_py()
    }
}
