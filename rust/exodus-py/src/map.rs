//! ID map operations for Exodus files

use pyo3::prelude::*;
use crate::error::IntoPyResult;
use crate::file::{ExodusWriter, ExodusAppender, ExodusReader};

#[pymethods]
impl ExodusWriter {
    /// Write node ID map (NOTE: Not yet implemented in exodus-rs)
    fn put_node_id_map(&mut self, _id_map: Vec<i64>) -> PyResult<()> {
        Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>(
            "put_node_id_map not yet implemented in exodus-rs"
        ))
    }

    /// Write element ID map (NOTE: Not yet implemented in exodus-rs)
    fn put_elem_id_map(&mut self, _id_map: Vec<i64>) -> PyResult<()> {
        Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>(
            "put_elem_id_map not yet implemented in exodus-rs"
        ))
    }
}

#[pymethods]
impl ExodusAppender {
    /// Read node ID map (NOTE: Not yet implemented in exodus-rs)
    fn get_node_id_map(&self) -> PyResult<Vec<i64>> {
        Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>(
            "get_node_id_map not yet implemented in exodus-rs"
        ))
    }

    /// Read element ID map (NOTE: Not yet implemented in exodus-rs)
    fn get_elem_id_map(&self) -> PyResult<Vec<i64>> {
        Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>(
            "get_elem_id_map not yet implemented in exodus-rs"
        ))
    }
}

#[pymethods]
impl ExodusReader {
    /// Read node ID map (NOTE: Not yet implemented in exodus-rs)
    fn get_node_id_map(&self) -> PyResult<Vec<i64>> {
        Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>(
            "get_node_id_map not yet implemented in exodus-rs"
        ))
    }

    /// Read element ID map (NOTE: Not yet implemented in exodus-rs)
    fn get_elem_id_map(&self) -> PyResult<Vec<i64>> {
        Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>(
            "get_elem_id_map not yet implemented in exodus-rs"
        ))
    }
}
