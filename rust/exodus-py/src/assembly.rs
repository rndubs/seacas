//! Assembly and blob operations

use pyo3::prelude::*;
use crate::error::IntoPyResult;
use crate::file::{ExodusWriter, ExodusAppender, ExodusReader};
use crate::types::{Assembly, Blob};

#[pymethods]
impl ExodusWriter {
    /// Write an assembly
    fn put_assembly(&mut self, assembly: &Assembly) -> PyResult<()> {
        self.file_mut()?.put_assembly(&assembly.to_rust()).into_py()?;
        Ok(())
    }

    /// Write a blob with binary data
    ///
    /// Args:
    ///     blob: Blob metadata (ID and name)
    ///     data: Binary data as bytes
    fn put_blob(&mut self, blob: &Blob, data: Vec<u8>) -> PyResult<()> {
        self.file_mut()?.put_blob(&blob.to_rust(), &data).into_py()?;
        Ok(())
    }
}

#[pymethods]
impl ExodusAppender {
    /// Read an assembly (NOTE: Not available in Append mode)
    fn get_assembly(&self, _assembly_id: i64) -> PyResult<Assembly> {
        Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>(
            "get_assembly not available in Append mode - use ExodusReader instead"
        ))
    }

    /// Read a blob (NOTE: Not available in Append mode)
    fn get_blob(&self, _blob_id: i64) -> PyResult<Blob> {
        Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>(
            "get_blob not available in Append mode - use ExodusReader instead"
        ))
    }

    /// Get all assembly IDs (NOTE: Not available in Append mode)
    fn get_assembly_ids(&self) -> PyResult<Vec<i64>> {
        Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>(
            "get_assembly_ids not available in Append mode - use ExodusReader instead"
        ))
    }

    /// Get all blob IDs (NOTE: Not available in Append mode)
    fn get_blob_ids(&self) -> PyResult<Vec<i64>> {
        Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>(
            "get_blob_ids not available in Append mode - use ExodusReader instead"
        ))
    }
}

#[pymethods]
impl ExodusReader {
    /// Read an assembly
    fn get_assembly(&self, assembly_id: i64) -> PyResult<Assembly> {
        let asm = self.file_ref().assembly(assembly_id).into_py()?;
        Ok(Assembly::from_rust(&asm))
    }

    /// Read a blob (returns blob metadata only, not the data)
    fn get_blob(&self, blob_id: i64) -> PyResult<Blob> {
        let (blob, _data) = self.file_ref().blob(blob_id).into_py()?;
        Ok(Blob::from_rust(&blob))
    }

    /// Get all assembly IDs
    fn get_assembly_ids(&self) -> PyResult<Vec<i64>> {
        self.file_ref().assembly_ids().into_py()
    }

    /// Get all blob IDs
    fn get_blob_ids(&self) -> PyResult<Vec<i64>> {
        self.file_ref().blob_ids().into_py()
    }
}
