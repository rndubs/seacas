//! Metadata operations for Exodus files

use pyo3::prelude::*;
use crate::error::IntoPyResult;
use crate::file::{ExodusWriter, ExodusAppender, ExodusReader};
use crate::types::QaRecord;

#[pymethods]
impl ExodusWriter {
    /// Write information records
    fn put_info_records(&mut self, info: Vec<String>) -> PyResult<()> {
        self.file_mut()?.put_info_records(&info).into_py()?;
        Ok(())
    }

    /// Write QA records (NOTE: Not yet implemented in exodus-rs)
    fn put_qa_records(&mut self, _qa_records: Vec<QaRecord>) -> PyResult<()> {
        Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>(
            "put_qa_records not yet implemented in exodus-rs"
        ))
    }
}

#[pymethods]
impl ExodusAppender {
    /// Read QA records (NOTE: Not yet implemented in exodus-rs)
    fn get_qa_records(&self) -> PyResult<Vec<QaRecord>> {
        Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>(
            "get_qa_records not yet implemented in exodus-rs"
        ))
    }

    /// Read information records (NOTE: Not yet implemented in exodus-rs)
    fn get_info_records(&self) -> PyResult<Vec<String>> {
        Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>(
            "get_info_records not yet implemented in exodus-rs"
        ))
    }
}

#[pymethods]
impl ExodusReader {
    /// Read QA records (NOTE: Not yet implemented in exodus-rs)
    fn get_qa_records(&self) -> PyResult<Vec<QaRecord>> {
        Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>(
            "get_qa_records not yet implemented in exodus-rs"
        ))
    }

    /// Read information records (NOTE: Not yet implemented in exodus-rs)
    fn get_info_records(&self) -> PyResult<Vec<String>> {
        Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>(
            "get_info_records not yet implemented in exodus-rs"
        ))
    }
}
