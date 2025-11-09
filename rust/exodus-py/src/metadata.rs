//! Metadata operations (QA records, info records, names)

use pyo3::prelude::*;
use crate::error::IntoPyResult;
use crate::file::{ExodusWriter, ExodusAppender, ExodusReader};
use crate::types::QaRecord;

#[pymethods]
impl ExodusWriter {
    /// Write QA records
    ///
    /// Args:
    ///     qa_records: List of QaRecord objects
    fn put_qa_records(&mut self, qa_records: Vec<QaRecord>) -> PyResult<()> {
        let rust_qas: Vec<_> = qa_records.iter().map(|q| q.to_rust()).collect();
        self.file_mut()?.put_qa_records(&rust_qas).into_py()?;
        Ok(())
    }

    /// Write info records
    ///
    /// Args:
    ///     info_records: List of info strings
    fn put_info_records(&mut self, info_records: Vec<String>) -> PyResult<()> {
        let info_refs: Vec<&str> = info_records.iter().map(|s| s.as_str()).collect();
        self.file_mut()?.put_info_records(&info_refs).into_py()?;
        Ok(())
    }
}

#[pymethods]
impl ExodusAppender {
    /// Write QA records
    fn put_qa_records(&mut self, qa_records: Vec<QaRecord>) -> PyResult<()> {
        let rust_qas: Vec<_> = qa_records.iter().map(|q| q.to_rust()).collect();
        self.file_mut()?.put_qa_records(&rust_qas).into_py()?;
        Ok(())
    }

    /// Write info records
    fn put_info_records(&mut self, info_records: Vec<String>) -> PyResult<()> {
        let info_refs: Vec<&str> = info_records.iter().map(|s| s.as_str()).collect();
        self.file_mut()?.put_info_records(&info_refs).into_py()?;
        Ok(())
    }

    /// Read QA records
    fn get_qa_records(&self) -> PyResult<Vec<QaRecord>> {
        let qas = self.file.as_ref()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("File closed"))?
            .get_qa_records()
            .into_py()?;
        Ok(qas.iter().map(QaRecord::from_rust).collect())
    }

    /// Read info records
    fn get_info_records(&self) -> PyResult<Vec<String>> {
        self.file.as_ref()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("File closed"))?
            .get_info_records()
            .into_py()
    }
}

#[pymethods]
impl ExodusReader {
    /// Read QA records
    fn get_qa_records(&self) -> PyResult<Vec<QaRecord>> {
        let qas = self.file_ref().get_qa_records().into_py()?;
        Ok(qas.iter().map(QaRecord::from_rust).collect())
    }

    /// Read info records
    fn get_info_records(&self) -> PyResult<Vec<String>> {
        self.file_ref().get_info_records().into_py()
    }
}
