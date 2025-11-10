//! File operations for Exodus files

use pyo3::prelude::*;
use exodus_rs::{ExodusFile as RustExodusFile, mode};
use std::path::PathBuf;

use crate::error::IntoPyResult;
use crate::types::{CreateOptions, InitParams};

/// Exodus file reader (read-only access)
#[pyclass]
pub struct ExodusReader {
    pub(crate) file: RustExodusFile<mode::Read>,
}

#[pymethods]
impl ExodusReader {
    /// Open an existing Exodus file for reading
    ///
    /// Args:
    ///     path: Path to the Exodus file
    ///
    /// Returns:
    ///     ExodusReader instance
    ///
    /// Example:
    ///     >>> reader = ExodusReader.open("mesh.exo")
    ///     >>> params = reader.init_params()
    ///     >>> print(f"Nodes: {params.num_nodes}")
    #[staticmethod]
    fn open(path: &str) -> PyResult<Self> {
        let file = RustExodusFile::open(path).into_py()?;
        Ok(ExodusReader { file })
    }

    /// Get initialization parameters from the file
    ///
    /// Returns:
    ///     InitParams containing database dimensions and counts
    fn init_params(&self) -> PyResult<InitParams> {
        let params = self.file.init_params().into_py()?;
        Ok(InitParams::from_rust(&params))
    }

    /// Get the file path
    ///
    /// Returns:
    ///     Path to the file as a string
    fn path(&self) -> String {
        self.file.path().to_string_lossy().to_string()
    }

    /// Get Exodus file format version
    ///
    /// Returns:
    ///     tuple: (major_version, minor_version) as integers
    ///
    /// Example:
    ///     >>> reader = ExodusReader.open("mesh.exo")
    ///     >>> version = reader.version()
    ///     >>> print(f"Exodus version: {version[0]}.{version[1]}")
    fn version(&self) -> PyResult<(u32, u32)> {
        self.file.version().into_py()
    }

    /// Get NetCDF file format information
    ///
    /// Returns:
    ///     str: File format as string (e.g., "NetCDF4", "NetCDF3", etc.)
    ///
    /// Example:
    ///     >>> reader = ExodusReader.open("mesh.exo")
    ///     >>> fmt = reader.format()
    ///     >>> print(f"File format: {fmt}")
    fn format(&self) -> PyResult<String> {
        let fmt = self.file.format().into_py()?;
        Ok(format!("{:?}", fmt))
    }

    /// Close the file (no-op for readers, file closes automatically when dropped)
    ///
    /// This method is provided for API consistency with ExodusWriter.
    fn close(&self) -> PyResult<()> {
        // No-op: file closes automatically when dropped
        Ok(())
    }

    /// Enter context manager (for 'with' statement)
    fn __enter__(slf: Py<Self>) -> Py<Self> {
        slf
    }

    /// Exit context manager
    fn __exit__(
        &mut self,
        _exc_type: Option<&PyAny>,
        _exc_value: Option<&PyAny>,
        _traceback: Option<&PyAny>,
    ) -> PyResult<bool> {
        // File will be closed when dropped
        Ok(false)
    }

    fn __repr__(&self) -> String {
        format!("ExodusReader('{}')", self.path())
    }
}

/// Exodus file writer (write-only, for creating new files)
#[pyclass]
pub struct ExodusWriter {
    pub(crate) file: Option<RustExodusFile<mode::Write>>,
}

#[pymethods]
impl ExodusWriter {
    /// Create a new Exodus file for writing
    ///
    /// Args:
    ///     path: Path where the file should be created
    ///     options: CreateOptions (optional, defaults to Clobber mode)
    ///
    /// Returns:
    ///     ExodusWriter instance
    ///
    /// Example:
    ///     >>> from exodus import ExodusWriter, CreateOptions, CreateMode
    ///     >>> writer = ExodusWriter.create("mesh.exo")
    ///     >>> # Or with options:
    ///     >>> opts = CreateOptions(mode=CreateMode.NO_CLOBBER)
    ///     >>> writer = ExodusWriter.create("mesh.exo", opts)
    #[staticmethod]
    #[pyo3(signature = (path, options=None))]
    fn create(path: &str, options: Option<CreateOptions>) -> PyResult<Self> {
        let opts = options
            .map(|o| o.to_rust())
            .unwrap_or_else(|| exodus_rs::types::CreateOptions::default());

        let file = RustExodusFile::create(path, opts).into_py()?;
        Ok(ExodusWriter { file: Some(file) })
    }

    /// Initialize the database with parameters
    ///
    /// Must be called before writing data to the file.
    ///
    /// Args:
    ///     params: InitParams containing database dimensions
    ///
    /// Example:
    ///     >>> params = InitParams(
    ///     ...     title="My Mesh",
    ///     ...     num_dim=3,
    ///     ...     num_nodes=8,
    ///     ...     num_elems=1,
    ///     ...     num_elem_blocks=1
    ///     ... )
    ///     >>> writer.put_init_params(params)
    fn put_init_params(&mut self, params: &InitParams) -> PyResult<()> {
        if let Some(ref mut file) = self.file {
            file.init(&params.to_rust()).into_py()?;
            Ok(())
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "File already closed",
            ))
        }
    }

    /// Get the file path
    fn path(&self) -> PyResult<String> {
        if let Some(ref file) = self.file {
            Ok(file.path().to_string_lossy().to_string())
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "File already closed",
            ))
        }
    }

    /// Explicitly flush buffered data to disk
    ///
    /// This ensures all data is written to the file system.
    /// Useful for ensuring data persistence before continuing operations.
    ///
    /// Example:
    ///     >>> writer = ExodusWriter.create("mesh.exo")
    ///     >>> writer.put_init_params(params)
    ///     >>> writer.sync()  # Ensure data is flushed to disk
    fn sync(&mut self) -> PyResult<()> {
        if let Some(ref mut file) = self.file {
            file.sync().into_py()?;
            Ok(())
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "File already closed",
            ))
        }
    }

    /// Close the file explicitly
    fn close(&mut self) -> PyResult<()> {
        if let Some(file) = self.file.take() {
            file.close().into_py()?;
        }
        Ok(())
    }

    /// Enter context manager
    fn __enter__(slf: Py<Self>) -> Py<Self> {
        slf
    }

    /// Exit context manager
    fn __exit__(
        &mut self,
        _exc_type: Option<&PyAny>,
        _exc_value: Option<&PyAny>,
        _traceback: Option<&PyAny>,
    ) -> PyResult<bool> {
        self.close()?;
        Ok(false)
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("ExodusWriter('{}')", self.path()?))
    }
}

/// Exodus file appender (read-write access to existing files)
#[pyclass]
pub struct ExodusAppender {
    pub(crate) file: Option<RustExodusFile<mode::Append>>,
}

#[pymethods]
impl ExodusAppender {
    /// Open an existing Exodus file for appending (read-write)
    ///
    /// Args:
    ///     path: Path to the existing Exodus file
    ///
    /// Returns:
    ///     ExodusAppender instance
    ///
    /// Example:
    ///     >>> appender = ExodusAppender.append("mesh.exo")
    ///     >>> params = appender.init_params()
    ///     >>> # ... modify data ...
    #[staticmethod]
    fn append(path: &str) -> PyResult<Self> {
        let file = RustExodusFile::append(path).into_py()?;
        Ok(ExodusAppender { file: Some(file) })
    }

    /// Get initialization parameters
    fn init_params(&self) -> PyResult<InitParams> {
        if let Some(ref file) = self.file {
            let params = file.init_params().into_py()?;
            Ok(InitParams::from_rust(&params))
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "File already closed",
            ))
        }
    }

    /// Get the file path
    fn path(&self) -> PyResult<String> {
        if let Some(ref file) = self.file {
            Ok(file.path().to_string_lossy().to_string())
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "File already closed",
            ))
        }
    }

    /// Close the file explicitly
    fn close(&mut self) -> PyResult<()> {
        if let Some(file) = self.file.take() {
            file.close().into_py()?;
        }
        Ok(())
    }

    /// Enter context manager
    fn __enter__(slf: Py<Self>) -> Py<Self> {
        slf
    }

    /// Exit context manager
    fn __exit__(
        &mut self,
        _exc_type: Option<&PyAny>,
        _exc_value: Option<&PyAny>,
        _traceback: Option<&PyAny>,
    ) -> PyResult<bool> {
        self.close()?;
        Ok(false)
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("ExodusAppender('{}')", self.path()?))
    }
}

// Internal helper to get mutable reference to the underlying file
impl ExodusWriter {
    pub(crate) fn file_mut(&mut self) -> PyResult<&mut RustExodusFile<mode::Write>> {
        self.file.as_mut().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("File already closed")
        })
    }

    pub(crate) fn file_ref(&self) -> PyResult<&RustExodusFile<mode::Write>> {
        self.file.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("File already closed")
        })
    }
}

impl ExodusAppender {
    pub(crate) fn file_mut(&mut self) -> PyResult<&mut RustExodusFile<mode::Append>> {
        self.file.as_mut().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("File already closed")
        })
    }

    pub(crate) fn file_ref(&self) -> PyResult<&RustExodusFile<mode::Append>> {
        self.file.as_ref().ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("File already closed")
        })
    }
}

impl ExodusReader {
    pub(crate) fn file_ref(&self) -> &RustExodusFile<mode::Read> {
        &self.file
    }
}
