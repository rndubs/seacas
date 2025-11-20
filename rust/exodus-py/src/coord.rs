//! Coordinate operations for Exodus files

use pyo3::prelude::*;
use crate::error::IntoPyResult;
use crate::file::{ExodusWriter, ExodusAppender, ExodusReader};

#[cfg(feature = "numpy")]
use numpy::{PyArray1, PyArray2, PyReadonlyArray1};

#[pymethods]
impl ExodusWriter {
    /// Write nodal coordinates to the file (accepts NumPy arrays or lists)
    ///
    /// Args:
    ///     x: X coordinates as NumPy array or list (required)
    ///     y: Y coordinates as NumPy array or list (optional for 1D, required for 2D/3D)
    ///     z: Z coordinates as NumPy array or list (optional for 1D/2D, required for 3D)
    ///
    /// Example:
    ///     >>> import numpy as np
    ///     >>> writer.put_coords(
    ///     ...     x=np.array([0.0, 1.0, 1.0, 0.0]),
    ///     ...     y=np.array([0.0, 0.0, 1.0, 1.0]),
    ///     ...     z=np.array([])
    ///     ... )
    #[pyo3(signature = (x, y=None, z=None))]
    #[cfg(feature = "numpy")]
    fn put_coords(
        &mut self,
        py: Python<'_>,
        x: Bound<'_, PyAny>,
        y: Option<Bound<'_, PyAny>>,
        z: Option<Bound<'_, PyAny>>,
    ) -> PyResult<()> {
        // Convert NumPy arrays or lists to Vec
        let x_vec = if let Ok(arr) = x.downcast::<PyArray1<f64>>() {
            arr.readonly().as_slice()?.to_vec()
        } else {
            x.extract::<Vec<f64>>()?
        };

        let y_vec = if let Some(y_any) = y {
            if let Ok(arr) = y_any.downcast::<PyArray1<f64>>() {
                Some(arr.readonly().as_slice()?.to_vec())
            } else {
                Some(y_any.extract::<Vec<f64>>()?)
            }
        } else {
            None
        };

        let z_vec = if let Some(z_any) = z {
            if let Ok(arr) = z_any.downcast::<PyArray1<f64>>() {
                Some(arr.readonly().as_slice()?.to_vec())
            } else {
                Some(z_any.extract::<Vec<f64>>()?)
            }
        } else {
            None
        };

        let y_slice = y_vec.as_deref();
        let z_slice = z_vec.as_deref();

        self.file_mut()?
            .put_coords(&x_vec, y_slice, z_slice)
            .into_py()?;
        Ok(())
    }

    /// Write nodal coordinates to the file (list version, no NumPy)
    #[pyo3(signature = (x, y=None, z=None))]
    #[cfg(not(feature = "numpy"))]
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
    /// Write nodal coordinates to the file (accepts NumPy arrays or lists)
    #[pyo3(signature = (x, y=None, z=None))]
    #[cfg(feature = "numpy")]
    fn put_coords(
        &mut self,
        py: Python<'_>,
        x: Bound<'_, PyAny>,
        y: Option<Bound<'_, PyAny>>,
        z: Option<Bound<'_, PyAny>>,
    ) -> PyResult<()> {
        // Convert NumPy arrays or lists to Vec
        let x_vec = if let Ok(arr) = x.downcast::<PyArray1<f64>>() {
            arr.readonly().as_slice()?.to_vec()
        } else {
            x.extract::<Vec<f64>>()?
        };

        let y_vec = if let Some(y_any) = y {
            if let Ok(arr) = y_any.downcast::<PyArray1<f64>>() {
                Some(arr.readonly().as_slice()?.to_vec())
            } else {
                Some(y_any.extract::<Vec<f64>>()?)
            }
        } else {
            None
        };

        let z_vec = if let Some(z_any) = z {
            if let Ok(arr) = z_any.downcast::<PyArray1<f64>>() {
                Some(arr.readonly().as_slice()?.to_vec())
            } else {
                Some(z_any.extract::<Vec<f64>>()?)
            }
        } else {
            None
        };

        let y_slice = y_vec.as_deref();
        let z_slice = z_vec.as_deref();

        self.file_mut()?
            .put_coords(&x_vec, y_slice, z_slice)
            .into_py()?;
        Ok(())
    }

    /// Write nodal coordinates to the file (list version, no NumPy)
    #[pyo3(signature = (x, y=None, z=None))]
    #[cfg(not(feature = "numpy"))]
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

    /// Read nodal coordinates as NumPy array
    ///
    /// Returns:
    ///     NumPy array of shape (num_nodes, 3) with columns [x, y, z]
    #[cfg(feature = "numpy")]
    fn get_coords<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let arr = self.file_ref()?.coords_array::<f64>().into_py()?;
        Ok(PyArray2::from_owned_array_bound(py, arr))
    }

    /// Read nodal coordinates as lists (deprecated)
    ///
    /// Returns:
    ///     Tuple of (x, y, z) coordinate lists
    fn get_coords_list(&self) -> PyResult<(Vec<f64>, Vec<f64>, Vec<f64>)> {
        let coords = self.file_ref()?.coords::<f64>().into_py()?;
        Ok((coords.x, coords.y, coords.z))
    }
}

#[pymethods]
impl ExodusReader {
    /// Read nodal coordinates as NumPy array
    ///
    /// Returns:
    ///     NumPy array of shape (num_nodes, 3) with columns [x, y, z]
    ///
    /// Example:
    ///     >>> coords = reader.get_coords()
    ///     >>> print(coords.shape)  # (num_nodes, 3)
    ///     >>> x = coords[:, 0]  # X coordinates
    #[cfg(feature = "numpy")]
    fn get_coords<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyArray2<f64>>> {
        let arr = self.file_ref().coords_array::<f64>().into_py()?;
        Ok(PyArray2::from_owned_array_bound(py, arr))
    }

    /// Read nodal coordinates as lists (deprecated, for backward compatibility)
    ///
    /// Returns:
    ///     Tuple of (x, y, z) coordinate lists
    ///
    /// Example:
    ///     >>> x, y, z = reader.get_coords_list()
    ///
    /// .. deprecated::
    ///     Use :meth:`get_coords` instead for better performance with NumPy arrays
    fn get_coords_list(&self) -> PyResult<(Vec<f64>, Vec<f64>, Vec<f64>)> {
        let coords = self.file_ref().coords::<f64>().into_py()?;
        Ok((coords.x, coords.y, coords.z))
    }

    /// Read only X coordinates as NumPy array
    ///
    /// Returns:
    ///     1D NumPy array of X coordinates
    ///
    /// Example:
    ///     >>> x = reader.get_coord_x()
    #[cfg(feature = "numpy")]
    fn get_coord_x<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let coords = self.file_ref().get_coord_x::<f64>().into_py()?;
        Ok(PyArray1::from_vec_bound(py, coords))
    }

    /// Read only X coordinates as list (deprecated)
    ///
    /// .. deprecated::
    ///     Use :meth:`get_coord_x` instead for better performance with NumPy arrays
    #[cfg(not(feature = "numpy"))]
    fn get_coord_x(&self) -> PyResult<Vec<f64>> {
        self.file_ref().get_coord_x::<f64>().into_py()
    }

    /// Read only Y coordinates as NumPy array
    ///
    /// Returns:
    ///     1D NumPy array of Y coordinates
    ///
    /// Example:
    ///     >>> y = reader.get_coord_y()
    #[cfg(feature = "numpy")]
    fn get_coord_y<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let coords = self.file_ref().get_coord_y::<f64>().into_py()?;
        Ok(PyArray1::from_vec_bound(py, coords))
    }

    /// Read only Y coordinates as list (deprecated)
    #[cfg(not(feature = "numpy"))]
    fn get_coord_y(&self) -> PyResult<Vec<f64>> {
        self.file_ref().get_coord_y::<f64>().into_py()
    }

    /// Read only Z coordinates as NumPy array
    ///
    /// Returns:
    ///     1D NumPy array of Z coordinates
    ///
    /// Example:
    ///     >>> z = reader.get_coord_z()
    #[cfg(feature = "numpy")]
    fn get_coord_z<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyArray1<f64>>> {
        let coords = self.file_ref().get_coord_z::<f64>().into_py()?;
        Ok(PyArray1::from_vec_bound(py, coords))
    }

    /// Read only Z coordinates as list (deprecated)
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
