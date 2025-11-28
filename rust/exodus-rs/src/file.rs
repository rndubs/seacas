//! File handle implementation for Exodus II files.
//!
//! This module provides the core file operations including creating, opening,
//! and closing Exodus files.

use crate::error::Result;
use crate::types::{
    CreateMode, CreateOptions, FileFormat, FileStorageFormat, FloatSize, Int64Mode, VarStorageMode,
};
use crate::{mode, FileMode, WritableMode};
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
    /// Performance configuration for HDF5/NetCDF optimization
    pub performance: Option<crate::performance::PerformanceConfig>,
    /// Detected storage format for variable data
    pub storage_format: FileStorageFormat,
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
            storage_format: FileStorageFormat::default(),
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

        // Create metadata and store performance config
        let mut metadata = FileMetadata::new();
        metadata.performance = perf_config;

        Ok(Self {
            nc_file,
            path: path.to_path_buf(),
            metadata,
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
            env::set_var("HDF5_CHUNK_CACHE_W0", config.cache.preemption.to_string());
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

        // Detect storage format for this file
        let storage_format = detect_storage_format(&nc_file);

        let mut metadata = FileMetadata::new();
        metadata.storage_format = storage_format;

        Ok(Self {
            nc_file,
            path: path.to_path_buf(),
            metadata,
            _mode: std::marker::PhantomData,
        })
    }
}

// =============================================================================
// Shared Write Operations (available in both Write and Append modes)
// =============================================================================

#[cfg(feature = "netcdf4")]
impl<M: WritableMode> ExodusFile<M> {
    /// Sync file to ensure all data is written
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
    pub fn end_define(&mut self) -> Result<()> {
        self.nc_file.sync()?;
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
    pub fn reenter_define(&mut self) -> Result<()> {
        self.nc_file.sync()?;
        self.metadata.define_mode = DefineMode::Define;
        Ok(())
    }

    /// Check if file is currently in define mode
    ///
    /// # Returns
    ///
    /// `true` if in define mode, `false` if in data mode
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

        // Detect storage format for this file
        let storage_format = detect_storage_format(&nc_file);

        // Load metadata from the existing file
        let mut metadata = FileMetadata::new();
        metadata.storage_format = storage_format;

        // Check if file is initialized by checking for num_dim dimension
        if let Some(dim) = nc_file.dimension("num_dim") {
            metadata.initialized = true;
            metadata.num_dim = Some(dim.len());

            // Load dimension cache
            if let Some(nodes_dim) = nc_file.dimension("num_nodes") {
                metadata
                    .dim_cache
                    .insert("num_nodes".to_string(), nodes_dim.len());
            }
        }

        Ok(Self {
            nc_file,
            path: path.to_path_buf(),
            metadata,
            _mode: std::marker::PhantomData,
        })
    }

    /// Convert a nodeset to a sideset and write it to the file with explicit ID.
    ///
    /// This is a convenience method that combines reading the nodeset, converting it
    /// to a sideset based on boundary faces, and writing the result to the file.
    ///
    /// For automatic ID assignment, use [`create_sideset_from_nodeset_auto`].
    ///
    /// # Arguments
    ///
    /// * `nodeset_id` - ID of the existing nodeset
    /// * `new_sideset_id` - ID for the new sideset
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The nodeset doesn't exist
    /// - Unable to read coordinates or connectivity
    /// - Unable to write the sideset to the file
    /// - File I/O errors occur
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut file = ExodusFile::<mode::Append>::append("mesh.exo")?;
    ///
    /// // Convert nodeset 10 to sideset 100 and write it
    /// file.create_sideset_from_nodeset(10, 100)?;
    /// file.sync()?;
    /// ```
    ///
    /// [`create_sideset_from_nodeset_auto`]: ExodusFile::create_sideset_from_nodeset_auto
    pub fn create_sideset_from_nodeset(
        &mut self,
        nodeset_id: i64,
        new_sideset_id: i64,
    ) -> Result<()> {
        // Note: Append mode has both read and write capabilities, but Rust's type
        // system requires separate impl blocks. We'll use internal methods that work
        // with the generic mode.

        // Read and convert the nodeset (uses read operations)
        let sideset = crate::sideset_utils::convert_nodeset_to_sideset(
            // Safe cast since Append mode supports reads
            unsafe { &*(self as *const _ as *const ExodusFile<mode::Read>) },
            nodeset_id,
            new_sideset_id,
        )?;

        // Write the sideset (uses write operations)
        let set = crate::types::Set {
            id: new_sideset_id,
            entity_type: crate::EntityType::SideSet,
            num_entries: sideset.elements.len(),
            num_dist_factors: 0,
        };

        // Safe cast since Append mode supports writes
        let writer = unsafe { &mut *(self as *mut _ as *mut ExodusFile<mode::Write>) };
        writer.put_set(&set)?;
        writer.put_side_set(new_sideset_id, &sideset.elements, &sideset.sides, None)?;

        Ok(())
    }

    /// Convert a nodeset to a sideset with auto-assigned ID and write it to the file.
    ///
    /// This is the recommended method for most use cases. The sideset ID is automatically
    /// assigned as one greater than the maximum existing sideset ID (or 1 if no sidesets exist).
    ///
    /// # Arguments
    ///
    /// * `nodeset_id` - ID of the existing nodeset
    ///
    /// # Returns
    ///
    /// The ID that was assigned to the new sideset
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut file = ExodusFile::<mode::Append>::append("mesh.exo")?;
    ///
    /// // Convert nodeset 10 to a sideset with auto-assigned ID
    /// let sideset_id = file.create_sideset_from_nodeset_auto(10)?;
    /// println!("Created sideset with ID {}", sideset_id);
    /// file.sync()?;
    /// ```
    pub fn create_sideset_from_nodeset_auto(&mut self, nodeset_id: i64) -> Result<i64> {
        // Get the next available sideset ID
        let reader = unsafe { &*(self as *const _ as *const ExodusFile<mode::Read>) };
        let existing_ids = reader.set_ids(crate::EntityType::SideSet)?;
        let new_id = existing_ids.iter().max().map(|&id| id + 1).unwrap_or(1);

        // Use the explicit version to do the actual work
        self.create_sideset_from_nodeset(nodeset_id, new_id)?;

        Ok(new_id)
    }

    /// Convert a nodeset to a sideset by name and write it to the file.
    ///
    /// This method looks up the nodeset by name, converts it to a sideset with
    /// auto-assigned ID, and writes it to the file. If the nodeset has a name,
    /// that name is copied to the new sideset.
    ///
    /// # Arguments
    ///
    /// * `nodeset_name` - Name of the existing nodeset
    ///
    /// # Returns
    ///
    /// The ID that was assigned to the new sideset
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The nodeset name is not found
    /// - Names are not defined for nodesets
    /// - Unable to read coordinates or connectivity
    /// - File I/O errors occur
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut file = ExodusFile::<mode::Append>::append("mesh.exo")?;
    ///
    /// // Convert nodeset named "inlet" to a sideset
    /// let sideset_id = file.create_sideset_from_nodeset_by_name("inlet")?;
    /// println!("Created sideset with ID {}", sideset_id);
    /// ```
    pub fn create_sideset_from_nodeset_by_name(&mut self, nodeset_name: &str) -> Result<i64> {
        // Find the nodeset ID by name
        let reader = unsafe { &*(self as *const _ as *const ExodusFile<mode::Read>) };
        let names = reader.names(crate::EntityType::NodeSet)?;
        let ids = reader.set_ids(crate::EntityType::NodeSet)?;

        let index = names
            .iter()
            .position(|name| name == nodeset_name)
            .ok_or_else(|| {
                crate::ExodusError::Other(format!("Nodeset with name '{}' not found", nodeset_name))
            })?;

        let nodeset_id = ids.get(index).ok_or_else(|| {
            crate::ExodusError::Other(format!(
                "Nodeset index {} out of bounds (found name but no ID)",
                index
            ))
        })?;

        // Create the sideset with auto ID
        let sideset_id = self.create_sideset_from_nodeset_auto(*nodeset_id)?;

        // Copy the name to the new sideset
        self.copy_name_to_sideset(index, sideset_id, nodeset_name)?;

        Ok(sideset_id)
    }

    /// Convert a nodeset to a sideset with explicit name and write it to the file.
    ///
    /// Creates a sideset from a nodeset with auto-assigned ID and writes both the
    /// sideset data and its name to the file.
    ///
    /// # Arguments
    ///
    /// * `nodeset_id` - ID of the existing nodeset
    /// * `sideset_name` - Name to assign to the new sideset
    ///
    /// # Returns
    ///
    /// The ID that was assigned to the new sideset
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut file = ExodusFile::<mode::Append>::append("mesh.exo")?;
    ///
    /// // Convert nodeset 10 to a sideset named "outlet"
    /// let sideset_id = file.create_sideset_from_nodeset_named(10, "outlet")?;
    /// println!("Created sideset '{}' with ID {}", "outlet", sideset_id);
    /// ```
    pub fn create_sideset_from_nodeset_named(
        &mut self,
        nodeset_id: i64,
        sideset_name: &str,
    ) -> Result<i64> {
        // Create the sideset with auto ID
        let sideset_id = self.create_sideset_from_nodeset_auto(nodeset_id)?;

        // Find the index of the new sideset (it's the last one)
        let reader = unsafe { &*(self as *const _ as *const ExodusFile<mode::Read>) };
        let ss_ids = reader.set_ids(crate::EntityType::SideSet)?;
        let ss_index = ss_ids.len().saturating_sub(1);

        // Set the name
        let writer = unsafe { &mut *(self as *mut _ as *mut ExodusFile<mode::Write>) };
        writer.put_name(crate::EntityType::SideSet, ss_index, sideset_name)?;

        Ok(sideset_id)
    }

    /// Helper function to copy a name from nodeset to sideset
    fn copy_name_to_sideset(
        &mut self,
        _nodeset_index: usize,
        sideset_id: i64,
        name: &str,
    ) -> Result<()> {
        // Find the index of the sideset
        let reader = unsafe { &*(self as *const _ as *const ExodusFile<mode::Read>) };
        let ss_ids = reader.set_ids(crate::EntityType::SideSet)?;
        let ss_index = ss_ids
            .iter()
            .position(|&id| id == sideset_id)
            .ok_or_else(|| {
                crate::ExodusError::Other(format!("Sideset with ID {} not found", sideset_id))
            })?;

        // Set the name
        let writer = unsafe { &mut *(self as *mut _ as *mut ExodusFile<mode::Write>) };
        writer.put_name(crate::EntityType::SideSet, ss_index, name)?;

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

    /// Get the detected storage format for this file.
    ///
    /// The storage format indicates how variable data is stored in the NetCDF file.
    /// Exodus II supports two formats:
    /// - **Separate**: Individual variables per index (e.g., `vals_nod_var1`, `vals_nod_var2`)
    /// - **Combined**: A single 3D array (e.g., `vals_nod_var(time_step, num_vars, num_nodes)`)
    ///
    /// The format is detected automatically when opening a file and can vary
    /// per entity type (nodal, element, etc.).
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use exodus_rs::{ExodusFile, mode, VarStorageMode};
    ///
    /// let file = ExodusFile::<mode::Read>::open("mesh.exo")?;
    /// let format = file.storage_format();
    ///
    /// match format.nodal {
    ///     VarStorageMode::Combined => println!("File uses combined nodal variable storage"),
    ///     VarStorageMode::Separate => println!("File uses separate nodal variable storage"),
    ///     VarStorageMode::None => println!("No nodal variables in file"),
    /// }
    /// # Ok::<(), exodus_rs::ExodusError>(())
    /// ```
    pub fn storage_format(&self) -> &FileStorageFormat {
        &self.metadata.storage_format
    }
}

/// Detect the storage format for a single variable type.
///
/// Checks if the combined variable exists first (e.g., `vals_nod_var`),
/// then checks for the separate format (e.g., `vals_nod_var1`).
#[cfg(feature = "netcdf4")]
fn detect_var_storage(
    nc_file: &netcdf::FileMut,
    combined_name: &str,
    separate_prefix: &str,
) -> VarStorageMode {
    // Check for combined 3D format first
    if nc_file.variable(combined_name).is_some() {
        return VarStorageMode::Combined;
    }
    // Check for separate format by looking for the first variable
    let separate_name = format!("{}1", separate_prefix);
    if nc_file.variable(&separate_name).is_some() {
        return VarStorageMode::Separate;
    }
    VarStorageMode::None
}

/// Detect the storage format for element block variables.
///
/// Element variables have a more complex naming pattern: `vals_elem_var{var}eb{block}`
/// for separate format, or `vals_elem_var` for combined format.
#[cfg(feature = "netcdf4")]
fn detect_elem_var_storage(nc_file: &netcdf::FileMut) -> VarStorageMode {
    // Check for combined format
    if nc_file.variable("vals_elem_var").is_some() {
        return VarStorageMode::Combined;
    }
    // Check for separate format (vals_elem_var1eb1)
    if nc_file.variable("vals_elem_var1eb1").is_some() {
        return VarStorageMode::Separate;
    }
    VarStorageMode::None
}

/// Detect the storage format for set variables.
///
/// Set variables have patterns like `vals_nset_var{var}ns{set}` for separate format.
#[cfg(feature = "netcdf4")]
fn detect_set_var_storage(
    nc_file: &netcdf::FileMut,
    combined_name: &str,
    separate_pattern: &str,
) -> VarStorageMode {
    // Check for combined format
    if nc_file.variable(combined_name).is_some() {
        return VarStorageMode::Combined;
    }
    // Check for separate format
    if nc_file.variable(separate_pattern).is_some() {
        return VarStorageMode::Separate;
    }
    VarStorageMode::None
}

/// Detect the storage format for all entity types in a file.
#[cfg(feature = "netcdf4")]
fn detect_storage_format(nc_file: &netcdf::FileMut) -> FileStorageFormat {
    FileStorageFormat {
        // Nodal: vals_nod_var (combined) vs vals_nod_var1 (separate)
        nodal: detect_var_storage(nc_file, "vals_nod_var", "vals_nod_var"),
        // Element block: vals_elem_var (combined) vs vals_elem_var1eb1 (separate)
        elem_block: detect_elem_var_storage(nc_file),
        // Edge block: vals_edge_var (combined) vs vals_edge_var1edb1 (separate)
        edge_block: detect_set_var_storage(nc_file, "vals_edge_var", "vals_edge_var1edb1"),
        // Face block: vals_face_var (combined) vs vals_face_var1fab1 (separate)
        face_block: detect_set_var_storage(nc_file, "vals_face_var", "vals_face_var1fab1"),
        // Node set: vals_nset_var (combined) vs vals_nset_var1ns1 (separate)
        node_set: detect_set_var_storage(nc_file, "vals_nset_var", "vals_nset_var1ns1"),
        // Edge set: vals_eset_var (combined) vs vals_eset_var1es1 (separate)
        edge_set: detect_set_var_storage(nc_file, "vals_eset_var", "vals_eset_var1es1"),
        // Face set: vals_fset_var (combined) vs vals_fset_var1fs1 (separate)
        face_set: detect_set_var_storage(nc_file, "vals_fset_var", "vals_fset_var1fs1"),
        // Side set: vals_sset_var (combined) vs vals_sset_var1ss1 (separate)
        side_set: detect_set_var_storage(nc_file, "vals_sset_var", "vals_sset_var1ss1"),
        // Element set: vals_elset_var (combined) vs vals_elset_var1els1 (separate)
        elem_set: detect_set_var_storage(nc_file, "vals_elset_var", "vals_elset_var1els1"),
        // Global: always uses vals_glo_var (combined format with shape time_step x num_glo_var)
        global: if nc_file.variable("vals_glo_var").is_some() {
            VarStorageMode::Combined
        } else {
            VarStorageMode::None
        },
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
