//! Map operations (ID maps, order maps)

use pyo3::prelude::*;
use crate::error::IntoPyResult;
use crate::file::{ExodusWriter, ExodusAppender, ExodusReader};

#[pymethods]
impl ExodusWriter {
    /// Write node ID map
    ///
    /// Args:
    ///     id_map: Array of node IDs (1-based)
    fn put_node_id_map(&mut self, id_map: Vec<i64>) -> PyResult<()> {
        self.file_mut()?.put_node_id_map(&id_map).into_py()?;
        Ok(())
    }

    /// Write element ID map
    ///
    /// Args:
    ///     id_map: Array of element IDs (1-based)
    fn put_elem_id_map(&mut self, id_map: Vec<i64>) -> PyResult<()> {
        self.file_mut()?.put_elem_id_map(&id_map).into_py()?;
        Ok(())
    }
}

#[pymethods]
impl ExodusAppender {
    /// Write node ID map
    fn put_node_id_map(&mut self, id_map: Vec<i64>) -> PyResult<()> {
        self.file_mut()?.put_node_id_map(&id_map).into_py()?;
        Ok(())
    }

    /// Write element ID map
    fn put_elem_id_map(&mut self, id_map: Vec<i64>) -> PyResult<()> {
        self.file_mut()?.put_elem_id_map(&id_map).into_py()?;
        Ok(())
    }

    /// Read node ID map
    fn get_node_id_map(&self) -> PyResult<Vec<i64>> {
        self.file.as_ref()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("File closed"))?
            .get_node_id_map()
            .into_py()
    }

    /// Read element ID map
    fn get_elem_id_map(&self) -> PyResult<Vec<i64>> {
        self.file.as_ref()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("File closed"))?
            .get_elem_id_map()
            .into_py()
    }
}

#[pymethods]
impl ExodusReader {
    /// Read node ID map
    fn get_node_id_map(&self) -> PyResult<Vec<i64>> {
        self.file_ref().get_node_id_map().into_py()
    }

    /// Read element ID map
    fn get_elem_id_map(&self) -> PyResult<Vec<i64>> {
        self.file_ref().get_elem_id_map().into_py()
    }
}
