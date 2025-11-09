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

    /// Write a blob
    fn put_blob(&mut self, blob: &Blob) -> PyResult<()> {
        self.file_mut()?.put_blob(&blob.to_rust()).into_py()?;
        Ok(())
    }
}

#[pymethods]
impl ExodusAppender {
    /// Write an assembly
    fn put_assembly(&mut self, assembly: &Assembly) -> PyResult<()> {
        self.file_mut()?.put_assembly(&assembly.to_rust()).into_py()?;
        Ok(())
    }

    /// Write a blob
    fn put_blob(&mut self, blob: &Blob) -> PyResult<()> {
        self.file_mut()?.put_blob(&blob.to_rust()).into_py()?;
        Ok(())
    }

    /// Read an assembly
    fn get_assembly(&self, assembly_id: i64) -> PyResult<Assembly> {
        let asm = self.file.as_ref()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("File closed"))?
            .get_assembly(assembly_id)
            .into_py()?;
        Ok(Assembly::from_rust(&asm))
    }

    /// Read a blob
    fn get_blob(&self, blob_id: i64) -> PyResult<Blob> {
        let blob = self.file.as_ref()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("File closed"))?
            .get_blob(blob_id)
            .into_py()?;
        Ok(Blob::from_rust(&blob))
    }

    /// Get all assembly IDs
    fn get_assembly_ids(&self) -> PyResult<Vec<i64>> {
        self.file.as_ref()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("File closed"))?
            .get_assembly_ids()
            .into_py()
    }

    /// Get all blob IDs
    fn get_blob_ids(&self) -> PyResult<Vec<i64>> {
        self.file.as_ref()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("File closed"))?
            .get_blob_ids()
            .into_py()
    }
}

#[pymethods]
impl ExodusReader {
    /// Read an assembly
    fn get_assembly(&self, assembly_id: i64) -> PyResult<Assembly> {
        let asm = self.file_ref().get_assembly(assembly_id).into_py()?;
        Ok(Assembly::from_rust(&asm))
    }

    /// Read a blob
    fn get_blob(&self, blob_id: i64) -> PyResult<Blob> {
        let blob = self.file_ref().get_blob(blob_id).into_py()?;
        Ok(Blob::from_rust(&blob))
    }

    /// Get all assembly IDs
    fn get_assembly_ids(&self) -> PyResult<Vec<i64>> {
        self.file_ref().get_assembly_ids().into_py()
    }

    /// Get all blob IDs
    fn get_blob_ids(&self) -> PyResult<Vec<i64>> {
        self.file_ref().get_blob_ids().into_py()
    }
}
