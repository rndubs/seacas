//! Coordinate operations for Exodus files

use pyo3::prelude::*;
use crate::error::IntoPyResult;
use crate::file::{ExodusWriter, ExodusAppender, ExodusReader};
use crate::numpy_utils::{extract_f64_vec, vec_to_numpy_f64, coords_to_numpy};

#[pymethods]
impl ExodusWriter {
    /// Write nodal coordinates to the file
    ///
    /// Args:
    ///     x: X coordinates (required) - accepts list or NumPy array
    ///     y: Y coordinates (optional for 1D, required for 2D/3D) - accepts list or NumPy array
    ///     z: Z coordinates (optional for 1D/2D, required for 3D) - accepts list or NumPy array
    ///
    /// Example:
    ///     >>> # Using lists
    ///     >>> writer.put_coords(
    ///     ...     x=[0.0, 1.0, 1.0, 0.0],
    ///     ...     y=[0.0, 0.0, 1.0, 1.0],
    ///     ...     z=[]
    ///     ... )
    ///     >>> # Using NumPy arrays
    ///     >>> import numpy as np
    ///     >>> writer.put_coords(
    ///     ...     x=np.array([0.0, 1.0, 1.0, 0.0]),
    ///     ...     y=np.array([0.0, 0.0, 1.0, 1.0])
    ///     ... )
    #[pyo3(signature = (x, y=None, z=None))]
    fn put_coords(
        &mut self,
        py: Python<'_>,
        x: &Bound<'_, PyAny>,
        y: Option<&Bound<'_, PyAny>>,
        z: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<()> {
        let x_vec = extract_f64_vec(py, x)?;
        let y_vec = y.map(|v| extract_f64_vec(py, v)).transpose()?;
        let z_vec = z.map(|v| extract_f64_vec(py, v)).transpose()?;

        let y_slice = y_vec.as_deref();
        let z_slice = z_vec.as_deref();

        self.file_mut()?
            .put_coords(&x_vec, y_slice, z_slice)
            .into_py()?;
        Ok(())
    }

    /// Write coordinate names
    ///
    /// Args:
    ///     names: List of coordinate names (length must match num_dim)
    ///
    /// Example:
    ///     >>> writer.put_coord_names(["X", "Y", "Z"])
    fn put_coord_names(&mut self, names: Vec<String>) -> PyResult<()> {
        let names_str: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        self.file_mut()?.put_coord_names(&names_str).into_py()?;
        Ok(())
    }
}

#[pymethods]
impl ExodusAppender {
    /// Write nodal coordinates to the file
    ///
    /// Args:
    ///     x: X coordinates (required) - accepts list or NumPy array
    ///     y: Y coordinates (optional for 1D, required for 2D/3D) - accepts list or NumPy array
    ///     z: Z coordinates (optional for 1D/2D, required for 3D) - accepts list or NumPy array
    #[pyo3(signature = (x, y=None, z=None))]
    fn put_coords(
        &mut self,
        py: Python<'_>,
        x: &Bound<'_, PyAny>,
        y: Option<&Bound<'_, PyAny>>,
        z: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<()> {
        let x_vec = extract_f64_vec(py, x)?;
        let y_vec = y.map(|v| extract_f64_vec(py, v)).transpose()?;
        let z_vec = z.map(|v| extract_f64_vec(py, v)).transpose()?;

        let y_slice = y_vec.as_deref();
        let z_slice = z_vec.as_deref();

        self.file_mut()?
            .put_coords(&x_vec, y_slice, z_slice)
            .into_py()?;
        Ok(())
    }

    /// Read nodal coordinates
    ///
    /// Returns:
    ///     Tuple of (x, y, z) NumPy coordinate arrays
    #[cfg(feature = "numpy")]
    fn get_coords<'py>(&self, py: Python<'py>) -> PyResult<(Bound<'py, numpy::PyArray1<f64>>, Bound<'py, numpy::PyArray1<f64>>, Bound<'py, numpy::PyArray1<f64>>)> {
        let coords = self.file_ref()?.coords::<f64>().into_py()?;
        Ok(coords_to_numpy(py, coords.x, coords.y, coords.z))
    }

    /// Read nodal coordinates
    ///
    /// Returns:
    ///     Tuple of (x, y, z) coordinate arrays
    #[cfg(not(feature = "numpy"))]
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
    ///     Tuple of (x, y, z) NumPy coordinate arrays
    ///
    /// Example:
    ///     >>> x, y, z = reader.get_coords()
    #[cfg(feature = "numpy")]
    fn get_coords<'py>(&self, py: Python<'py>) -> PyResult<(Bound<'py, numpy::PyArray1<f64>>, Bound<'py, numpy::PyArray1<f64>>, Bound<'py, numpy::PyArray1<f64>>)> {
        let coords = self.file_ref().coords::<f64>().into_py()?;
        Ok(coords_to_numpy(py, coords.x, coords.y, coords.z))
    }

    /// Read nodal coordinates
    ///
    /// Returns:
    ///     Tuple of (x, y, z) coordinate arrays
    ///
    /// Example:
    ///     >>> x, y, z = reader.get_coords()
    #[cfg(not(feature = "numpy"))]
    fn get_coords(&self) -> PyResult<(Vec<f64>, Vec<f64>, Vec<f64>)> {
        let coords = self.file_ref().coords::<f64>().into_py()?;
        Ok((coords.x, coords.y, coords.z))
    }

    /// Read only X coordinates
    ///
    /// Returns:
    ///     NumPy array of X coordinates
    ///
    /// Example:
    ///     >>> x = reader.get_coord_x()
    #[cfg(feature = "numpy")]
    fn get_coord_x<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, numpy::PyArray1<f64>>> {
        let vec = self.file_ref().get_coord_x::<f64>().into_py()?;
        Ok(vec_to_numpy_f64(py, vec))
    }

    /// Read only X coordinates
    ///
    /// Returns:
    ///     List of X coordinates
    ///
    /// Example:
    ///     >>> x = reader.get_coord_x()
    #[cfg(not(feature = "numpy"))]
    fn get_coord_x(&self) -> PyResult<Vec<f64>> {
        self.file_ref().get_coord_x::<f64>().into_py()
    }

    /// Read only Y coordinates
    ///
    /// Returns:
    ///     NumPy array of Y coordinates
    ///
    /// Example:
    ///     >>> y = reader.get_coord_y()
    #[cfg(feature = "numpy")]
    fn get_coord_y<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, numpy::PyArray1<f64>>> {
        let vec = self.file_ref().get_coord_y::<f64>().into_py()?;
        Ok(vec_to_numpy_f64(py, vec))
    }

    /// Read only Y coordinates
    ///
    /// Returns:
    ///     List of Y coordinates
    ///
    /// Example:
    ///     >>> y = reader.get_coord_y()
    #[cfg(not(feature = "numpy"))]
    fn get_coord_y(&self) -> PyResult<Vec<f64>> {
        self.file_ref().get_coord_y::<f64>().into_py()
    }

    /// Read only Z coordinates
    ///
    /// Returns:
    ///     NumPy array of Z coordinates
    ///
    /// Example:
    ///     >>> z = reader.get_coord_z()
    #[cfg(feature = "numpy")]
    fn get_coord_z<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, numpy::PyArray1<f64>>> {
        let vec = self.file_ref().get_coord_z::<f64>().into_py()?;
        Ok(vec_to_numpy_f64(py, vec))
    }

    /// Read only Z coordinates
    ///
    /// Returns:
    ///     List of Z coordinates
    ///
    /// Example:
    ///     >>> z = reader.get_coord_z()
    #[cfg(not(feature = "numpy"))]
    fn get_coord_z(&self) -> PyResult<Vec<f64>> {
        self.file_ref().get_coord_z::<f64>().into_py()
    }

    /// Read coordinate names
    ///
    /// Returns:
    ///     List of coordinate names (e.g., ["X", "Y", "Z"])
    ///     Empty list if not present in file
    ///
    /// Example:
    ///     >>> names = reader.get_coord_names()
    ///     >>> print(names)  # ["X", "Y", "Z"]
    fn get_coord_names(&self) -> PyResult<Vec<String>> {
        self.file_ref().coord_names().into_py()
    }
}
