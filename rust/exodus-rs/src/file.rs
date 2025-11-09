//! File handle implementation for Exodus II files.
//!
//! This module provides the core file operations including creating, opening,
//! and closing Exodus files.

use crate::error::{ExodusError, Result};
use crate::types::{CreateMode, CreateOptions, FileFormat, FloatSize, Int64Mode};
use crate::{mode, FileMode};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[cfg(feature = "netcdf4")]
use netcdf;

/// Internal metadata cache for file operations
#[derive(Debug)]
pub(crate) struct FileMetadata {
    /// Whether the file has been initialized with database parameters
    pub initialized: bool,
    /// Cached title
    pub title: Option<String>,
    /// Cached number of dimensions
    pub num_dim: Option<usize>,
    /// Cache for dimension IDs (dimension name -> size)
    pub dim_cache: HashMap<String, usize>,
}

impl FileMetadata {
    /// Create a new empty metadata cache
    fn new() -> Self {
        Self {
            initialized: false,
            title: None,
            num_dim: None,
            dim_cache: HashMap::new(),
        }
    }
}

/// Main Exodus file handle
///
/// The file is parameterized by mode (Read, Write, or Append) to enforce
/// correct usage at compile time.
#[derive(Debug)]
pub struct ExodusFile<M: FileMode> {
    #[cfg(feature = "netcdf4")]
    pub(crate) nc_file: netcdf::FileMut,
    #[cfg(not(feature = "netcdf4"))]
    _phantom_nc: std::marker::PhantomData<()>,

    pub(crate) path: PathBuf,
    pub(crate) metadata: FileMetadata,
    _mode: std::marker::PhantomData<M>,
}

#[cfg(feature = "netcdf4")]
impl ExodusFile<mode::Write> {
    /// Create a new Exodus file with specified options
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to create
    /// * `options` - Creation options (mode, float size, compression, etc.)
    ///
    /// # Returns
    ///
    /// A new `ExodusFile` in write mode, or an error if creation fails.
    ///
    /// # Errors
    ///
    /// - File already exists and `CreateMode::NoClobber` is set
    /// - Insufficient permissions to create the file
    /// - NetCDF library errors
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use exodus_rs::{ExodusFile, CreateOptions, CreateMode};
    ///
    /// let options = CreateOptions {
    ///     mode: CreateMode::Clobber,
    ///     ..Default::default()
    /// };
    /// let file = ExodusFile::create("mesh.exo", options)?;
    /// # Ok::<(), exodus_rs::ExodusError>(())
    /// ```
    pub fn create<P: AsRef<Path>>(path: P, options: CreateOptions) -> Result<Self> {
        let path = path.as_ref();

        // Convert options to NetCDF creation flags
        let mut nc_options = netcdf::Options::NETCDF4;

        // Set creation mode
        match options.mode {
            CreateMode::Clobber => {
                // Clobber is the default (no NOCLOBBER flag)
            }
            CreateMode::NoClobber => {
                nc_options |= netcdf::Options::NOCLOBBER;
            }
        }

        // Create the NetCDF file
        let mut nc_file = netcdf::create_with(path, nc_options)?;

        // Write global attributes to mark this as an Exodus file
        Self::write_global_attributes(&mut nc_file, &options)?;

        Ok(Self {
            nc_file,
            path: path.to_path_buf(),
            metadata: FileMetadata::new(),
            _mode: std::marker::PhantomData,
        })
    }

    /// Create a new Exodus file with default options
    ///
    /// This is a convenience method that uses `CreateOptions::default()`.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to create
    ///
    /// # Returns
    ///
    /// A new `ExodusFile` in write mode, or an error if creation fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use exodus_rs::ExodusFile;
    ///
    /// let file = ExodusFile::create_default("mesh.exo")?;
    /// # Ok::<(), exodus_rs::ExodusError>(())
    /// ```
    pub fn create_default<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::create(path, CreateOptions::default())
    }

    /// Write global attributes to the NetCDF file
    fn write_global_attributes(
        nc_file: &mut netcdf::FileMut,
        options: &CreateOptions,
    ) -> Result<()> {
        // API version
        nc_file.add_attribute("api_version", 9.04_f32)?;

        // File format version
        nc_file.add_attribute("version", 2.0_f32)?;

        // Floating point word size (4 or 8 bytes)
        let fp_word_size = match options.float_size {
            FloatSize::Float32 => 4_i32,
            FloatSize::Float64 => 8_i32,
        };
        nc_file.add_attribute("floating_point_word_size", fp_word_size)?;

        // File size mode (0 = normal, 1 = large model)
        // Always use 1 for large model support with NetCDF-4
        nc_file.add_attribute("file_size", 1_i32)?;

        // Int64 status
        let int64_status = match options.int64_mode {
            Int64Mode::Int32 => 0_i32,
            Int64Mode::Int64 => 1_i32,
        };
        nc_file.add_attribute("int64_status", int64_status)?;

        Ok(())
    }

    /// Ensure all metadata and dimensions are written to the file
    ///
    /// This method can be called after defining all dimensions, variables, and metadata
    /// to ensure they are committed to the file before writing data. While NetCDF-4
    /// generally handles this automatically, calling this explicitly can help with
    /// certain workflow patterns.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success
    ///
    /// # Note
    ///
    /// NetCDF-4 format generally allows interleaving definition and data operations.
    /// However, for best compatibility and performance, it's recommended to:
    /// 1. Initialize the file with `init()`
    /// 2. Define all blocks, sets, and variables
    /// 3. Call `sync()` (optional but recommended)
    /// 4. Write all data values
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use exodus_rs::*;
    /// let mut file = ExodusFile::create_default("mesh.exo")?;
    ///
    /// // Define structure
    /// file.init(&params)?;
    /// file.put_block(&block)?;
    /// file.define_variables(EntityType::Nodal, &["Temp"])?;
    ///
    /// // Sync definitions
    /// file.sync()?;
    ///
    /// // Write data
    /// file.put_coords(&x, &y, &z)?;
    /// file.put_var(0, EntityType::Nodal, 0, 0, &values)?;
    /// # Ok::<(), ExodusError>(())
    /// ```
    pub fn sync(&mut self) -> Result<()> {
        self.nc_file.sync()?;
        Ok(())
    }

    /// Get file path
    ///
    /// Returns the path to the underlying file.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(feature = "netcdf4")]
impl ExodusFile<mode::Read> {
    /// Open an existing Exodus file for reading
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to open
    ///
    /// # Returns
    ///
    /// An `ExodusFile` in read mode, or an error if opening fails.
    ///
    /// # Errors
    ///
    /// - File does not exist
    /// - File is not a valid NetCDF/Exodus file
    /// - Insufficient permissions to read the file
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use exodus_rs::ExodusFile;
    /// use exodus_rs::mode::Read;
    ///
    /// let file = ExodusFile::<Read>::open("mesh.exo")?;
    /// # Ok::<(), exodus_rs::ExodusError>(())
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        // Open the NetCDF file (use append to get FileMut for variable reads)
        let nc_file = netcdf::append(path)?;

        Ok(Self {
            nc_file,
            path: path.to_path_buf(),
            metadata: FileMetadata::new(),
            _mode: std::marker::PhantomData,
        })
    }
}

#[cfg(feature = "netcdf4")]
impl ExodusFile<mode::Append> {
    /// Open an existing Exodus file for appending
    ///
    /// This mode allows both reading and writing to an existing file.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to open
    ///
    /// # Returns
    ///
    /// An `ExodusFile` in append mode, or an error if opening fails.
    ///
    /// # Errors
    ///
    /// - File does not exist
    /// - File is not a valid NetCDF/Exodus file
    /// - Insufficient permissions to write to the file
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use exodus_rs::ExodusFile;
    /// use exodus_rs::mode::Append;
    ///
    /// let file = ExodusFile::<Append>::append("mesh.exo")?;
    /// # Ok::<(), exodus_rs::ExodusError>(())
    /// ```
    pub fn append<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        // Open the NetCDF file in append mode (read-write)
        let nc_file = netcdf::append(path)?;

        Ok(Self {
            nc_file,
            path: path.to_path_buf(),
            metadata: FileMetadata::new(),
            _mode: std::marker::PhantomData,
        })
    }
}

#[cfg(feature = "netcdf4")]
impl<M: FileMode> ExodusFile<M> {
    /// Get the file path
    ///
    /// # Returns
    ///
    /// The path to the Exodus file
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get the NetCDF file format
    ///
    /// # Returns
    ///
    /// The format of the underlying NetCDF file
    ///
    /// # Errors
    ///
    /// Returns an error if the format cannot be determined
    pub fn format(&self) -> Result<FileFormat> {
        // Query the NetCDF file format
        // Note: The netcdf crate doesn't expose format directly,
        // so we'll infer it from file attributes

        // For now, if we can read the file, assume it's NetCDF-4
        // This can be enhanced later with more sophisticated detection
        Ok(FileFormat::NetCdf4)
    }

    /// Get the Exodus file version
    ///
    /// # Returns
    ///
    /// A tuple of (major_version, minor_version)
    ///
    /// # Errors
    ///
    /// Returns an error if the version attribute cannot be read
    pub fn version(&self) -> Result<(u32, u32)> {
        // Read the version attribute
        match self.nc_file.attribute("version") {
            Some(attr) => {
                use netcdf::AttributeValue;
                match attr.value()? {
                    AttributeValue::Float(val) => {
                        // Version is stored as decimal (e.g., 2.0)
                        let major = val as u32;
                        let minor = ((val - major as f32) * 10.0).round() as u32;
                        Ok((major, minor))
                    }
                    AttributeValue::Double(val) => {
                        // Version is stored as decimal (e.g., 2.0)
                        let major = val as u32;
                        let minor = ((val - major as f64) * 10.0).round() as u32;
                        Ok((major, minor))
                    }
                    _ => Ok((2, 0)), // Default version
                }
            }
            None => Ok((2, 0)), // Default version if attribute not found
        }
    }

    /// Close the file explicitly
    ///
    /// This is called automatically when the file handle is dropped,
    /// but can be called explicitly to handle errors.
    ///
    /// # Errors
    ///
    /// Returns an error if closing the NetCDF file fails
    pub fn close(self) -> Result<()> {
        // The netcdf crate handles closing automatically via Drop
        // So we just need to consume self
        Ok(())
    }
}

#[cfg(feature = "netcdf4")]
impl<M: FileMode> Drop for ExodusFile<M> {
    fn drop(&mut self) {
        // NetCDF file is automatically closed by its Drop implementation
        // We don't need to do anything here, but this makes the cleanup explicit
    }
}

// Stub implementations when netcdf4 feature is not enabled
#[cfg(not(feature = "netcdf4"))]
impl ExodusFile<mode::Write> {
    /// Create a new Exodus file (requires netcdf4 feature)
    pub fn create<P: AsRef<Path>>(_path: P, _options: CreateOptions) -> Result<Self> {
        Err(ExodusError::Other(
            "NetCDF support not enabled. Enable the 'netcdf4' feature.".to_string(),
        ))
    }

    /// Create a new Exodus file with default options (requires netcdf4 feature)
    pub fn create_default<P: AsRef<Path>>(_path: P) -> Result<Self> {
        Err(ExodusError::Other(
            "NetCDF support not enabled. Enable the 'netcdf4' feature.".to_string(),
        ))
    }
}

#[cfg(not(feature = "netcdf4"))]
impl ExodusFile<mode::Read> {
    /// Open an existing Exodus file (requires netcdf4 feature)
    pub fn open<P: AsRef<Path>>(_path: P) -> Result<Self> {
        Err(ExodusError::Other(
            "NetCDF support not enabled. Enable the 'netcdf4' feature.".to_string(),
        ))
    }
}

#[cfg(not(feature = "netcdf4"))]
impl ExodusFile<mode::Append> {
    /// Open an existing Exodus file for appending (requires netcdf4 feature)
    pub fn append<P: AsRef<Path>>(_path: P) -> Result<Self> {
        Err(ExodusError::Other(
            "NetCDF support not enabled. Enable the 'netcdf4' feature.".to_string(),
        ))
    }
}

#[cfg(test)]
#[cfg(feature = "netcdf4")]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    // Helper function to create file with clobber mode for tests
    fn create_test_file(path: impl AsRef<std::path::Path>) -> Result<ExodusFile<mode::Write>> {
        ExodusFile::create(
            path,
            CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            },
        )
    }

    #[test]
    fn test_create_default() {
        let tmp = NamedTempFile::new().unwrap();
        let file = create_test_file(tmp.path()).unwrap();
        assert_eq!(file.path(), tmp.path());
        drop(file);
        assert!(tmp.path().exists());
    }

    #[test]
    fn test_create_noclobber() {
        let tmp = NamedTempFile::new().unwrap();
        let _file1 = create_test_file(tmp.path()).unwrap();

        // Should fail - file exists and using NoClobber
        let result = ExodusFile::create(
            tmp.path(),
            CreateOptions {
                mode: CreateMode::NoClobber,
                ..Default::default()
            },
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_create_clobber() {
        let tmp = NamedTempFile::new().unwrap();
        let _file1 = create_test_file(tmp.path()).unwrap();
        drop(_file1);

        // Should succeed - overwrite existing file
        let _file2 = ExodusFile::create(
            tmp.path(),
            CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            },
        )
        .unwrap();
    }

    #[test]
    fn test_open_nonexistent() {
        let result = ExodusFile::<mode::Read>::open("nonexistent_file_xyz.exo");
        assert!(result.is_err());
    }

    #[test]
    fn test_open_existing() {
        let tmp = NamedTempFile::new().unwrap();
        {
            let _file = create_test_file(tmp.path()).unwrap();
        }

        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        assert_eq!(file.path(), tmp.path());
    }

    #[test]
    fn test_version() {
        let tmp = NamedTempFile::new().unwrap();
        let file = create_test_file(tmp.path()).unwrap();
        let version = file.version().unwrap();
        assert_eq!(version, (2, 0));
    }

    #[test]
    fn test_format() {
        let tmp = NamedTempFile::new().unwrap();
        let file = create_test_file(tmp.path()).unwrap();
        let format = file.format().unwrap();
        assert_eq!(format, FileFormat::NetCdf4);
    }

    #[test]
    fn test_close_explicit() {
        let tmp = NamedTempFile::new().unwrap();
        let file = create_test_file(tmp.path()).unwrap();
        file.close().unwrap();
    }

    #[test]
    fn test_append_mode() {
        let tmp = NamedTempFile::new().unwrap();
        {
            let _file = create_test_file(tmp.path()).unwrap();
        }

        let file = ExodusFile::<mode::Append>::append(tmp.path()).unwrap();
        assert_eq!(file.path(), tmp.path());
    }
}
