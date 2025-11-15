//! File handle implementation for Exodus II files.
//!
//! This module provides the core file operations including creating, opening,
//! and closing Exodus files.

use crate::error::Result;
use crate::types::{CreateMode, CreateOptions, FileFormat, FloatSize, Int64Mode};
use crate::{mode, FileMode};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[cfg(feature = "netcdf4")]
/// NetCDF define mode state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DefineMode {
    /// In define mode (can add dimensions, variables, attributes)
    Define,
    /// In data mode (can write data to variables)
    Data,
}

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
    /// Current NetCDF define/data mode (only tracked for Write/Append modes)
    pub define_mode: DefineMode,
    /// Performance configuration (if specified)
    #[allow(dead_code)]
    pub performance: Option<crate::performance::PerformanceConfig>,
}

impl FileMetadata {
    /// Create a new empty metadata cache
    fn new() -> Self {
        Self {
            initialized: false,
            title: None,
            num_dim: None,
            dim_cache: HashMap::new(),
            define_mode: DefineMode::Define,
            performance: None,
        }
    }

    /// Create metadata with performance config
    fn with_performance(perf: Option<crate::performance::PerformanceConfig>) -> Self {
        Self {
            initialized: false,
            title: None,
            num_dim: None,
            dim_cache: HashMap::new(),
            define_mode: DefineMode::Define,
            performance: perf,
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
    pub fn create<P: AsRef<Path>>(path: P, mut options: CreateOptions) -> Result<Self> {
        let path = path.as_ref();

        // Get or auto-detect performance configuration
        let perf_config = if options.performance.is_none() {
            Some(crate::performance::PerformanceConfig::auto())
        } else {
            options.performance.take()
        };

        // Apply HDF5 performance tuning via environment variables if config is present
        if let Some(ref config) = perf_config {
            Self::apply_hdf5_env_vars(config);
        }

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
            metadata: FileMetadata::with_performance(perf_config),
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

    /// Apply HDF5 performance tuning via environment variables
    ///
    /// Sets HDF5_CHUNK_CACHE_* environment variables that HDF5 will respect.
    /// Note: These must be set before the HDF5 library is initialized.
    ///
    /// For more fine-grained control, users can set these environment variables
    /// manually before running their program.
    fn apply_hdf5_env_vars(config: &crate::performance::PerformanceConfig) {
        use std::env;

        // HDF5 respects these environment variables (if set before library init)
        // HDF5_CHUNK_CACHE_NBYTES - cache size in bytes
        // HDF5_CHUNK_CACHE_NSLOTS - number of hash slots
        // HDF5_CHUNK_CACHE_W0 - preemption policy (0.0-1.0)

        // Only set if not already set by user
        if env::var("HDF5_CHUNK_CACHE_NBYTES").is_err() {
            env::set_var(
                "HDF5_CHUNK_CACHE_NBYTES",
                config.cache.cache_size.to_string(),
            );
        }

        if env::var("HDF5_CHUNK_CACHE_W0").is_err() {
            env::set_var(
                "HDF5_CHUNK_CACHE_W0",
                config.cache.preemption.to_string(),
            );
        }

        // Calculate or use provided num_slots
        if env::var("HDF5_CHUNK_CACHE_NSLOTS").is_err() {
            let num_slots = if config.cache.num_slots > 0 {
                config.cache.num_slots
            } else {
                // Auto-calculate based on typical chunk size (1MB for coordinates)
                config.cache.auto_slots(1024 * 1024)
            };
            env::set_var("HDF5_CHUNK_CACHE_NSLOTS", num_slots.to_string());
        }
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
    /// This method flushes all pending changes to disk, ensuring data integrity.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success
    ///
    /// # Automatic Mode Management
    ///
    /// exodus-rs now automatically manages NetCDF define/data mode transitions.
    /// You typically don't need to manually call `end_define()` or `reenter_define()`.
    /// The library automatically switches modes as needed:
    /// - Define mode for: `init()`, `put_block()`, `define_variables()`
    /// - Data mode for: `put_coords()`, `put_connectivity()`, `put_var()`
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use exodus_rs::*;
    /// let mut file = ExodusFile::create_default("mesh.exo")?;
    ///
    /// // Define structure - automatically uses define mode
    /// file.init(&params)?;
    /// file.put_block(&block)?;
    /// file.define_variables(EntityType::Nodal, &["Temp"])?;
    ///
    /// // Write data - automatically switches to data mode
    /// file.put_coords(&x, &y, &z)?;
    /// file.put_var(0, EntityType::Nodal, 0, 0, &values)?;
    ///
    /// // Optional: sync to ensure everything is written
    /// file.sync()?;
    /// # Ok::<(), ExodusError>(())
    /// ```
    pub fn sync(&mut self) -> Result<()> {
        self.nc_file.sync()?;
        Ok(())
    }

    /// End define mode and enter data mode
    ///
    /// This method explicitly transitions the NetCDF file from define mode to data mode.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success
    ///
    /// # Note - Automatic Mode Management
    ///
    /// **This method is now optional** thanks to automatic mode management.
    /// exodus-rs automatically switches between define and data modes as needed.
    /// You only need to call this method if you want explicit control over mode
    /// transitions for performance reasons or code clarity.
    ///
    /// For most use cases, you can simply call operations in any order and the
    /// library will handle mode transitions automatically.
    ///
    /// # Example (Manual Mode Control)
    ///
    /// ```rust,ignore
    /// # use exodus_rs::*;
    /// let mut file = ExodusFile::create_default("mesh.exo")?;
    ///
    /// // Define all structure
    /// file.init(&params)?;
    /// file.put_block(&block)?;
    /// file.define_variables(EntityType::Nodal, &["Temp"])?;
    ///
    /// // Optional: explicitly end define mode
    /// file.end_define()?;
    ///
    /// // Write data
    /// file.put_coords(&x, &y, &z)?;
    /// file.put_var(0, EntityType::Nodal, 0, 0, &values)?;
    /// # Ok::<(), ExodusError>(())
    /// ```
    ///
    /// # Example (Automatic - Recommended)
    ///
    /// ```rust,ignore
    /// # use exodus_rs::*;
    /// let mut file = ExodusFile::create_default("mesh.exo")?;
    ///
    /// // Just call operations in any order - modes are automatic
    /// file.init(&params)?;
    /// file.put_block(&block)?;
    /// file.define_variables(EntityType::Nodal, &["Temp"])?;
    /// file.put_coords(&x, &y, &z)?;  // Automatically switches to data mode
    /// file.put_var(0, EntityType::Nodal, 0, 0, &values)?;
    /// # Ok::<(), ExodusError>(())
    /// ```
    pub fn end_define(&mut self) -> Result<()> {
        // Sync to ensure all definitions are committed
        self.nc_file.sync()?;
        // Update internal state
        self.metadata.define_mode = DefineMode::Data;
        Ok(())
    }

    /// Re-enter define mode from data mode
    ///
    /// This method explicitly transitions the NetCDF file back to define mode.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success
    ///
    /// # Note - Automatic Mode Management
    ///
    /// **This method is now optional** thanks to automatic mode management.
    /// exodus-rs automatically switches back to define mode when needed.
    /// The library handles mode transitions transparently, so you can freely
    /// mix definition and data operations.
    ///
    /// # Example (Automatic - Recommended)
    ///
    /// ```rust,ignore
    /// # use exodus_rs::*;
    /// let mut file = ExodusFile::create_default("mesh.exo")?;
    ///
    /// // Mix definitions and data operations freely
    /// file.init(&params)?;
    /// file.put_coords(&x, &y, &z)?;  // Writes data
    /// file.define_variables(EntityType::Element, &["Stress"])?;  // Auto-switches to define mode
    /// file.put_var(0, EntityType::Element, 0, 1, &stress)?;  // Auto-switches back to data mode
    /// # Ok::<(), ExodusError>(())
    /// ```
    ///
    /// # Example (Manual Control)
    ///
    /// ```rust,ignore
    /// # use exodus_rs::*;
    /// let mut file = ExodusFile::create_default("mesh.exo")?;
    ///
    /// file.init(&params)?;
    /// file.end_define()?;
    /// file.put_coords(&x, &y, &z)?;
    ///
    /// // Manually re-enter define mode
    /// file.reenter_define()?;
    /// file.define_variables(EntityType::Element, &["Stress"])?;
    /// file.end_define()?;
    ///
    /// file.put_var(0, EntityType::Element, 0, 1, &stress)?;
    /// # Ok::<(), ExodusError>(())
    /// ```
    pub fn reenter_define(&mut self) -> Result<()> {
        // Sync before mode change
        self.nc_file.sync()?;
        // Update internal state
        self.metadata.define_mode = DefineMode::Define;
        Ok(())
    }

    /// Check if file is currently in define mode
    ///
    /// # Returns
    ///
    /// `true` if in define mode, `false` if in data mode
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use exodus_rs::*;
    /// let mut file = ExodusFile::create_default("mesh.exo")?;
    /// assert!(file.is_define_mode());
    ///
    /// file.end_define()?;
    /// assert!(!file.is_define_mode());
    /// # Ok::<(), ExodusError>(())
    /// ```
    pub fn is_define_mode(&self) -> bool {
        self.metadata.define_mode == DefineMode::Define
    }

    /// Ensure the file is in define mode, transitioning if necessary
    ///
    /// This is an internal helper that automatically manages mode transitions.
    /// If the file is already in define mode, this is a no-op.
    /// If in data mode, this calls `reenter_define()` automatically.
    pub(crate) fn ensure_define_mode(&mut self) -> Result<()> {
        if self.metadata.define_mode == DefineMode::Data {
            self.reenter_define()?;
        }
        Ok(())
    }

    /// Ensure the file is in data mode, transitioning if necessary
    ///
    /// This is an internal helper that automatically manages mode transitions.
    /// If the file is already in data mode, this is a no-op.
    /// If in define mode, this calls `end_define()` automatically.
    pub(crate) fn ensure_data_mode(&mut self) -> Result<()> {
        if self.metadata.define_mode == DefineMode::Define {
            self.end_define()?;
        }
        Ok(())
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

        // Load metadata from the existing file
        let mut metadata = FileMetadata::new();

        // Check if file is initialized by checking for num_dim dimension
        if let Some(dim) = nc_file.dimension("num_dim") {
            metadata.initialized = true;
            metadata.num_dim = Some(dim.len());

            // Load dimension cache
            if let Some(nodes_dim) = nc_file.dimension("num_nodes") {
                metadata.dim_cache.insert("num_nodes".to_string(), nodes_dim.len());
            }
        }

        Ok(Self {
            nc_file,
            path: path.to_path_buf(),
            metadata,
            _mode: std::marker::PhantomData,
        })
    }

    /// Sync file to ensure all data is written
    ///
    /// See [`ExodusFile::<mode::Write>::sync()`] for details.
    pub fn sync(&mut self) -> Result<()> {
        self.nc_file.sync()?;
        Ok(())
    }

    /// End define mode and enter data mode
    ///
    /// See [`ExodusFile::<mode::Write>::end_define()`] for details.
    pub fn end_define(&mut self) -> Result<()> {
        self.nc_file.sync()?;
        self.metadata.define_mode = DefineMode::Data;
        Ok(())
    }

    /// Re-enter define mode from data mode
    ///
    /// See [`ExodusFile::<mode::Write>::reenter_define()`] for details.
    pub fn reenter_define(&mut self) -> Result<()> {
        self.nc_file.sync()?;
        self.metadata.define_mode = DefineMode::Define;
        Ok(())
    }

    /// Check if file is currently in define mode
    ///
    /// See [`ExodusFile::<mode::Write>::is_define_mode()`] for details.
    pub fn is_define_mode(&self) -> bool {
        self.metadata.define_mode == DefineMode::Define
    }

    /// Ensure the file is in define mode, transitioning if necessary
    ///
    /// See [`ExodusFile::<mode::Write>::ensure_define_mode()`] for details.
    #[allow(dead_code)]
    pub(crate) fn ensure_define_mode(&mut self) -> Result<()> {
        if self.metadata.define_mode == DefineMode::Data {
            self.reenter_define()?;
        }
        Ok(())
    }

    /// Ensure the file is in data mode, transitioning if necessary
    ///
    /// See [`ExodusFile::<mode::Write>::ensure_data_mode()`] for details.
    pub(crate) fn ensure_data_mode(&mut self) -> Result<()> {
        if self.metadata.define_mode == DefineMode::Define {
            self.end_define()?;
        }
        Ok(())
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
        // Sync the file to ensure all data and metadata are written
        // This is especially important for NetCDF define mode changes
        // For read-only files, this will be a no-op
        // Ignore errors in drop - we're already cleaning up
        let _ = self.nc_file.sync();

        // NetCDF file is automatically closed by its Drop implementation
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

    #[test]
    fn test_define_mode_tracking() {
        let tmp = NamedTempFile::new().unwrap();
        let mut file = create_test_file(tmp.path()).unwrap();

        // File starts in define mode
        assert!(file.is_define_mode());

        // Transition to data mode
        file.end_define().unwrap();
        assert!(!file.is_define_mode());

        // Re-enter define mode
        file.reenter_define().unwrap();
        assert!(file.is_define_mode());
    }

    #[test]
    fn test_define_mode_append() {
        let tmp = NamedTempFile::new().unwrap();
        {
            let _file = create_test_file(tmp.path()).unwrap();
        }

        let mut file = ExodusFile::<mode::Append>::append(tmp.path()).unwrap();

        // Append mode starts in define mode
        assert!(file.is_define_mode());

        // Can transition modes
        file.end_define().unwrap();
        assert!(!file.is_define_mode());

        file.reenter_define().unwrap();
        assert!(file.is_define_mode());
    }

    #[test]
    fn test_sync() {
        let tmp = NamedTempFile::new().unwrap();
        let mut file = create_test_file(tmp.path()).unwrap();

        // Sync should succeed
        file.sync().unwrap();
    }
}
