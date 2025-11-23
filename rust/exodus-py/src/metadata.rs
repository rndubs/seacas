//! Metadata operations for Exodus files

use crate::error::IntoPyResult;
use crate::file::{ExodusAppender, ExodusReader, ExodusWriter};
use crate::types::QaRecord;
use pyo3::prelude::*;

#[pymethods]
impl ExodusWriter {
    /// Write information records
    fn put_info_records(&mut self, info: Vec<String>) -> PyResult<()> {
        self.file_mut()?.put_info_records(&info).into_py()?;
        Ok(())
    }

    /// Write QA records
    ///
    /// Args:
    ///     qa_records: List of QaRecord objects containing code provenance info
    ///
    /// Example:
    ///     >>> qa = QaRecord("MyApp", "1.0", "2025-01-15", "10:30:00")
    ///     >>> writer.put_qa_records([qa])
    fn put_qa_records(&mut self, qa_records: Vec<QaRecord>) -> PyResult<()> {
        let rust_records: Vec<exodus_rs::types::QaRecord> =
            qa_records.iter().map(|qa| qa.to_rust()).collect();
        self.file_mut()?.put_qa_records(&rust_records).into_py()?;
        Ok(())
    }
}

#[pymethods]
impl ExodusAppender {
    /// Read QA records (NOTE: Not available in Append mode - use ExodusReader instead)
    fn get_qa_records(&self) -> PyResult<Vec<QaRecord>> {
        Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>(
            "get_qa_records not available in Append mode - use ExodusReader instead",
        ))
    }

    /// Read information records (NOTE: Not available in Append mode - use ExodusReader instead)
    fn get_info_records(&self) -> PyResult<Vec<String>> {
        Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>(
            "get_info_records not available in Append mode - use ExodusReader instead",
        ))
    }
}

#[pymethods]
impl ExodusReader {
    /// Read QA records
    ///
    /// Returns:
    ///     List of QaRecord objects with code provenance information
    ///
    /// Example:
    ///     >>> reader = ExodusReader.open("mesh.exo")
    ///     >>> qa_records = reader.get_qa_records()
    ///     >>> for qa in qa_records:
    ///     ...     print(f"{qa.code_name} {qa.code_version}")
    fn get_qa_records(&self) -> PyResult<Vec<QaRecord>> {
        let rust_records = self.file.qa_records().into_py()?;
        Ok(rust_records.iter().map(QaRecord::from_rust).collect())
    }

    /// Read information records
    fn get_info_records(&self) -> PyResult<Vec<String>> {
        self.file.info_records().into_py()
    }
}
