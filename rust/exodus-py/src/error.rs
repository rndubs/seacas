//! Error handling for Python bindings

use pyo3::prelude::*;
use pyo3::exceptions::PyRuntimeError;
use exodus_rs::ExodusError as RustExodusError;

/// Trait for converting Rust results to Python results
pub trait IntoPyResult<T> {
    /// Convert a Rust result to a Python result
    fn into_py(self) -> PyResult<T>;
}

impl<T> IntoPyResult<T> for Result<T, RustExodusError> {
    fn into_py(self) -> PyResult<T> {
        self.map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }
}
