//! Error handling for Python bindings

use pyo3::exceptions::{PyException, PyIOError, PyValueError};
use pyo3::prelude::*;
use exodus_rs::ExodusError as RustExodusError;

/// Base exception for all Exodus errors
#[pyclass(extends = PyException)]
#[derive(Debug, Clone)]
pub struct ExodusError {
    #[pyo3(get)]
    message: String,
}

#[pymethods]
impl ExodusError {
    #[new]
    fn new(message: String) -> Self {
        ExodusError { message }
    }

    fn __str__(&self) -> String {
        self.message.clone()
    }

    fn __repr__(&self) -> String {
        format!("ExodusError('{}')", self.message)
    }
}

/// File-related errors
#[pyclass(extends = ExodusError)]
#[derive(Debug, Clone)]
pub struct FileError {
    #[pyo3(get)]
    path: String,
}

#[pymethods]
impl FileError {
    #[new]
    fn new(message: String, path: String) -> (Self, ExodusError) {
        (FileError { path }, ExodusError { message })
    }
}

/// NetCDF library errors
#[pyclass(extends = ExodusError)]
#[derive(Debug, Clone)]
pub struct NetCdfError {}

#[pymethods]
impl NetCdfError {
    #[new]
    fn new(message: String) -> (Self, ExodusError) {
        (NetCdfError {}, ExodusError { message })
    }
}

/// Invalid data errors
#[pyclass(extends = ExodusError)]
#[derive(Debug, Clone)]
pub struct InvalidDataError {}

#[pymethods]
impl InvalidDataError {
    #[new]
    fn new(message: String) -> (Self, ExodusError) {
        (InvalidDataError {}, ExodusError { message })
    }
}

/// Convert Rust ExodusError to Python exception
impl From<RustExodusError> for PyErr {
    fn from(err: RustExodusError) -> PyErr {
        match err {
            RustExodusError::FileNotFound(path) => {
                PyIOError::new_err(format!("File not found: {}", path))
            }
            RustExodusError::FileExists(path) => {
                PyIOError::new_err(format!("File already exists: {}", path))
            }
            RustExodusError::NetCdf(msg) => {
                PyErr::new::<NetCdfError, _>(format!("NetCDF error: {}", msg))
            }
            RustExodusError::InvalidDimension(msg) => {
                PyValueError::new_err(format!("Invalid dimension: {}", msg))
            }
            RustExodusError::InvalidData(msg) => {
                PyErr::new::<InvalidDataError, _>(format!("Invalid data: {}", msg))
            }
            RustExodusError::NotInitialized => {
                PyErr::new::<ExodusError, _>("File not initialized. Call init() first.")
            }
            RustExodusError::InvalidEntityType(msg) => {
                PyValueError::new_err(format!("Invalid entity type: {}", msg))
            }
            RustExodusError::EntityNotFound { entity_type, id } => {
                PyValueError::new_err(format!("{} with ID {} not found", entity_type, id))
            }
        }
    }
}

/// Helper trait to convert Result to PyResult
pub(crate) trait IntoPyResult<T> {
    fn into_py(self) -> PyResult<T>;
}

impl<T> IntoPyResult<T> for exodus_rs::Result<T> {
    fn into_py(self) -> PyResult<T> {
        self.map_err(|e| e.into())
    }
}
