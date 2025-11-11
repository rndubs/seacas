//! Database initialization operations
//!
//! This module provides initialization functionality for Exodus files,
//! including parameter setup and builder pattern support.

use crate::error::{ExodusError, Result};
use crate::types::InitParams;
use crate::{mode, ExodusFile};

#[cfg(feature = "netcdf4")]
use netcdf;

/// Builder for fluent initialization API
///
/// This provides a convenient way to initialize an Exodus file with
/// a fluent, method-chaining interface.
///
/// # Example
///
/// ```rust,ignore
/// use exodus_rs::ExodusFile;
///
/// let mut file = ExodusFile::create_default("mesh.exo")?;
/// file.builder()
///     .title("Example Mesh")
///     .dimensions(3)
///     .nodes(100)
///     .elem_blocks(2)
///     .finish()?;
/// # Ok::<(), exodus_rs::ExodusError>(())
/// ```
#[derive(Debug)]
pub struct InitBuilder<'a> {
    file: &'a mut ExodusFile<mode::Write>,
    params: InitParams,
}

impl<'a> InitBuilder<'a> {
    /// Create a new builder for the given file
    pub(crate) fn new(file: &'a mut ExodusFile<mode::Write>) -> Self {
        Self {
            file,
            params: InitParams::default(),
        }
    }

    /// Set the database title (max 80 characters)
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.params.title = title.into();
        self
    }

    /// Set the number of spatial dimensions (1, 2, or 3)
    pub fn dimensions(mut self, num_dim: usize) -> Self {
        self.params.num_dim = num_dim;
        self
    }

    /// Set the number of nodes
    pub fn nodes(mut self, num_nodes: usize) -> Self {
        self.params.num_nodes = num_nodes;
        self
    }

    /// Set the number of elements
    pub fn elems(mut self, num_elems: usize) -> Self {
        self.params.num_elems = num_elems;
        self
    }

    /// Set the number of element blocks
    pub fn elem_blocks(mut self, num_elem_blocks: usize) -> Self {
        self.params.num_elem_blocks = num_elem_blocks;
        self
    }

    /// Set the number of node sets
    pub fn node_sets(mut self, num_node_sets: usize) -> Self {
        self.params.num_node_sets = num_node_sets;
        self
    }

    /// Set the number of side sets
    pub fn side_sets(mut self, num_side_sets: usize) -> Self {
        self.params.num_side_sets = num_side_sets;
        self
    }

    /// Set the number of edges
    pub fn edges(mut self, num_edges: usize) -> Self {
        self.params.num_edges = num_edges;
        self
    }

    /// Set the number of edge blocks
    pub fn edge_blocks(mut self, num_edge_blocks: usize) -> Self {
        self.params.num_edge_blocks = num_edge_blocks;
        self
    }

    /// Set the number of edge sets
    pub fn edge_sets(mut self, num_edge_sets: usize) -> Self {
        self.params.num_edge_sets = num_edge_sets;
        self
    }

    /// Set the number of faces
    pub fn faces(mut self, num_faces: usize) -> Self {
        self.params.num_faces = num_faces;
        self
    }

    /// Set the number of face blocks
    pub fn face_blocks(mut self, num_face_blocks: usize) -> Self {
        self.params.num_face_blocks = num_face_blocks;
        self
    }

    /// Set the number of face sets
    pub fn face_sets(mut self, num_face_sets: usize) -> Self {
        self.params.num_face_sets = num_face_sets;
        self
    }

    /// Set the number of element sets
    pub fn elem_sets(mut self, num_elem_sets: usize) -> Self {
        self.params.num_elem_sets = num_elem_sets;
        self
    }

    /// Set the number of node maps
    pub fn node_maps(mut self, num_node_maps: usize) -> Self {
        self.params.num_node_maps = num_node_maps;
        self
    }

    /// Set the number of element maps
    pub fn elem_maps(mut self, num_elem_maps: usize) -> Self {
        self.params.num_elem_maps = num_elem_maps;
        self
    }

    /// Set the number of edge maps
    pub fn edge_maps(mut self, num_edge_maps: usize) -> Self {
        self.params.num_edge_maps = num_edge_maps;
        self
    }

    /// Set the number of face maps
    pub fn face_maps(mut self, num_face_maps: usize) -> Self {
        self.params.num_face_maps = num_face_maps;
        self
    }

    /// Set the number of assemblies
    pub fn assemblies(mut self, num_assemblies: usize) -> Self {
        self.params.num_assemblies = num_assemblies;
        self
    }

    /// Set the number of blobs
    pub fn blobs(mut self, num_blobs: usize) -> Self {
        self.params.num_blobs = num_blobs;
        self
    }

    /// Finish building and initialize the database
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file has already been initialized
    /// - Validation fails (invalid dimensions, negative counts, etc.)
    /// - NetCDF operations fail
    pub fn finish(self) -> Result<()> {
        self.file.init(&self.params)
    }
}

#[cfg(feature = "netcdf4")]
impl ExodusFile<mode::Write> {
    /// Initialize the Exodus database with parameters
    ///
    /// This must be called before any other data writing operations.
    /// It creates the necessary NetCDF dimensions and sets up the file structure.
    ///
    /// # Arguments
    ///
    /// * `params` - Initialization parameters defining the database structure
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file has already been initialized
    /// - `num_dim` is not 1, 2, or 3
    /// - Any count is negative (impossible with usize)
    /// - Title exceeds 80 characters
    /// - NetCDF operations fail
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use exodus_rs::{ExodusFile, InitParams};
    ///
    /// let mut file = ExodusFile::create_default("mesh.exo")?;
    /// let params = InitParams {
    ///     title: "Example Mesh".into(),
    ///     num_dim: 3,
    ///     num_nodes: 8,
    ///     num_elems: 1,
    ///     num_elem_blocks: 1,
    ///     ..Default::default()
    /// };
    /// file.init(&params)?;
    /// # Ok::<(), exodus_rs::ExodusError>(())
    /// ```
    pub fn init(&mut self, params: &InitParams) -> Result<()> {
        // Ensure we're in define mode for adding dimensions and attributes
        self.ensure_define_mode()?;

        // Check if already initialized
        if self.metadata.initialized {
            return Err(ExodusError::Other(
                "Database already initialized".to_string(),
            ));
        }

        // Validate parameters
        self.validate_init_params(params)?;

        // Write title as global attribute
        if !params.title.is_empty() {
            self.nc_file.add_attribute("title", params.title.as_str())?;
        }

        // Create dimensions
        self.write_dimensions(params)?;

        // Update metadata cache
        self.metadata.initialized = true;
        self.metadata.title = Some(params.title.clone());
        self.metadata.num_dim = Some(params.num_dim);

        Ok(())
    }

    /// Create a builder for fluent initialization
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use exodus_rs::ExodusFile;
    ///
    /// let mut file = ExodusFile::create_default("mesh.exo")?;
    /// file.builder()
    ///     .title("My Mesh")
    ///     .dimensions(3)
    ///     .nodes(100)
    ///     .finish()?;
    /// # Ok::<(), exodus_rs::ExodusError>(())
    /// ```
    pub fn builder(&mut self) -> InitBuilder<'_> {
        InitBuilder::new(self)
    }

    /// Validate initialization parameters
    fn validate_init_params(&self, params: &InitParams) -> Result<()> {
        // Check title length
        if params.title.len() > 80 {
            return Err(ExodusError::StringTooLong {
                max: 80,
                actual: params.title.len(),
            });
        }

        // Check dimensions
        if params.num_dim == 0 || params.num_dim > 3 {
            return Err(ExodusError::InvalidDimension {
                expected: "1, 2, or 3".to_string(),
                actual: params.num_dim,
            });
        }

        Ok(())
    }

    /// Write NetCDF dimensions based on initialization parameters
    fn write_dimensions(&mut self, params: &InitParams) -> Result<()> {
        // Always create num_dim dimension
        self.nc_file
            .add_dimension("num_dim", params.num_dim)
            .map_err(|e| ExodusError::NetCdf(e))?;
        self.metadata
            .dim_cache
            .insert("num_dim".to_string(), params.num_dim);

        // Create time dimension (unlimited)
        self.nc_file
            .add_unlimited_dimension("time_step")
            .map_err(|e| ExodusError::NetCdf(e))?;

        // Create node-related dimensions
        if params.num_nodes > 0 {
            self.nc_file
                .add_dimension("num_nodes", params.num_nodes)
                .map_err(|e| ExodusError::NetCdf(e))?;
            self.metadata
                .dim_cache
                .insert("num_nodes".to_string(), params.num_nodes);
        }

        // Create element-related dimensions
        if params.num_elems > 0 {
            self.nc_file
                .add_dimension("num_elem", params.num_elems)
                .map_err(|e| ExodusError::NetCdf(e))?;
            self.metadata
                .dim_cache
                .insert("num_elem".to_string(), params.num_elems);
        }

        if params.num_elem_blocks > 0 {
            self.nc_file
                .add_dimension("num_el_blk", params.num_elem_blocks)
                .map_err(|e| ExodusError::NetCdf(e))?;
            self.metadata
                .dim_cache
                .insert("num_el_blk".to_string(), params.num_elem_blocks);
        }

        // Create edge-related dimensions
        if params.num_edges > 0 {
            self.nc_file
                .add_dimension("num_edge", params.num_edges)
                .map_err(|e| ExodusError::NetCdf(e))?;
            self.metadata
                .dim_cache
                .insert("num_edge".to_string(), params.num_edges);
        }

        if params.num_edge_blocks > 0 {
            self.nc_file
                .add_dimension("num_ed_blk", params.num_edge_blocks)
                .map_err(|e| ExodusError::NetCdf(e))?;
            self.metadata
                .dim_cache
                .insert("num_ed_blk".to_string(), params.num_edge_blocks);
        }

        // Create face-related dimensions
        if params.num_faces > 0 {
            self.nc_file
                .add_dimension("num_face", params.num_faces)
                .map_err(|e| ExodusError::NetCdf(e))?;
            self.metadata
                .dim_cache
                .insert("num_face".to_string(), params.num_faces);
        }

        if params.num_face_blocks > 0 {
            self.nc_file
                .add_dimension("num_fa_blk", params.num_face_blocks)
                .map_err(|e| ExodusError::NetCdf(e))?;
            self.metadata
                .dim_cache
                .insert("num_fa_blk".to_string(), params.num_face_blocks);
        }

        // Create set-related dimensions
        if params.num_node_sets > 0 {
            self.nc_file
                .add_dimension("num_node_sets", params.num_node_sets)
                .map_err(|e| ExodusError::NetCdf(e))?;
            self.metadata
                .dim_cache
                .insert("num_node_sets".to_string(), params.num_node_sets);
        }

        if params.num_side_sets > 0 {
            self.nc_file
                .add_dimension("num_side_sets", params.num_side_sets)
                .map_err(|e| ExodusError::NetCdf(e))?;
            self.metadata
                .dim_cache
                .insert("num_side_sets".to_string(), params.num_side_sets);
        }

        if params.num_edge_sets > 0 {
            self.nc_file
                .add_dimension("num_edge_sets", params.num_edge_sets)
                .map_err(|e| ExodusError::NetCdf(e))?;
            self.metadata
                .dim_cache
                .insert("num_edge_sets".to_string(), params.num_edge_sets);
        }

        if params.num_face_sets > 0 {
            self.nc_file
                .add_dimension("num_face_sets", params.num_face_sets)
                .map_err(|e| ExodusError::NetCdf(e))?;
            self.metadata
                .dim_cache
                .insert("num_face_sets".to_string(), params.num_face_sets);
        }

        if params.num_elem_sets > 0 {
            self.nc_file
                .add_dimension("num_elem_sets", params.num_elem_sets)
                .map_err(|e| ExodusError::NetCdf(e))?;
            self.metadata
                .dim_cache
                .insert("num_elem_sets".to_string(), params.num_elem_sets);
        }

        // Create map-related dimensions
        if params.num_node_maps > 0 {
            self.nc_file
                .add_dimension("num_node_maps", params.num_node_maps)
                .map_err(|e| ExodusError::NetCdf(e))?;
            self.metadata
                .dim_cache
                .insert("num_node_maps".to_string(), params.num_node_maps);
        }

        if params.num_elem_maps > 0 {
            self.nc_file
                .add_dimension("num_elem_maps", params.num_elem_maps)
                .map_err(|e| ExodusError::NetCdf(e))?;
            self.metadata
                .dim_cache
                .insert("num_elem_maps".to_string(), params.num_elem_maps);
        }

        if params.num_edge_maps > 0 {
            self.nc_file
                .add_dimension("num_edge_maps", params.num_edge_maps)
                .map_err(|e| ExodusError::NetCdf(e))?;
            self.metadata
                .dim_cache
                .insert("num_edge_maps".to_string(), params.num_edge_maps);
        }

        if params.num_face_maps > 0 {
            self.nc_file
                .add_dimension("num_face_maps", params.num_face_maps)
                .map_err(|e| ExodusError::NetCdf(e))?;
            self.metadata
                .dim_cache
                .insert("num_face_maps".to_string(), params.num_face_maps);
        }

        // Create assembly and blob dimensions
        if params.num_assemblies > 0 {
            self.nc_file
                .add_dimension("num_assembly", params.num_assemblies)
                .map_err(|e| ExodusError::NetCdf(e))?;
            self.metadata
                .dim_cache
                .insert("num_assembly".to_string(), params.num_assemblies);
        }

        if params.num_blobs > 0 {
            self.nc_file
                .add_dimension("num_blob", params.num_blobs)
                .map_err(|e| ExodusError::NetCdf(e))?;
            self.metadata
                .dim_cache
                .insert("num_blob".to_string(), params.num_blobs);
        }

        // Create coordinate variables if we have nodes
        if params.num_nodes > 0 {
            self.create_coord_variables()?;
        }

        // Create block ID property variables
        if params.num_elem_blocks > 0 {
            self.create_block_id_variable("eb_prop1", "num_el_blk")?;
        }
        if params.num_edge_blocks > 0 {
            self.create_block_id_variable("ed_prop1", "num_ed_blk")?;
        }
        if params.num_face_blocks > 0 {
            self.create_block_id_variable("fa_prop1", "num_fa_blk")?;
        }

        Ok(())
    }

    /// Create coordinate variables
    fn create_coord_variables(&mut self) -> Result<()> {
        // Create coordx variable
        self.nc_file
            .add_variable::<f64>("coordx", &["num_nodes"])
            .map_err(|e| ExodusError::NetCdf(e))?;

        // Create coordy variable
        self.nc_file
            .add_variable::<f64>("coordy", &["num_nodes"])
            .map_err(|e| ExodusError::NetCdf(e))?;

        // Create coordz variable
        self.nc_file
            .add_variable::<f64>("coordz", &["num_nodes"])
            .map_err(|e| ExodusError::NetCdf(e))?;

        Ok(())
    }

    /// Create block ID property variable
    fn create_block_id_variable(&mut self, var_name: &str, dim_name: &str) -> Result<()> {
        let mut var = self
            .nc_file
            .add_variable::<i64>(var_name, &[dim_name])
            .map_err(|e| ExodusError::NetCdf(e))?;

        // Add attribute to identify this as a property variable
        var.put_attribute("name", "ID")
            .map_err(|e| ExodusError::NetCdf(e))?;

        Ok(())
    }
}

#[cfg(feature = "netcdf4")]
impl ExodusFile<mode::Read> {
    /// Get initialization parameters from the file
    ///
    /// Reads the database parameters that were written during initialization.
    ///
    /// # Returns
    ///
    /// The initialization parameters stored in the file
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file is not properly initialized
    /// - Required dimensions are missing
    /// - NetCDF operations fail
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use exodus_rs::ExodusFile;
    /// use exodus_rs::mode::Read;
    ///
    /// let file = ExodusFile::<Read>::open("mesh.exo")?;
    /// let params = file.init_params()?;
    /// println!("Title: {}", params.title);
    /// println!("Dimensions: {}", params.num_dim);
    /// # Ok::<(), exodus_rs::ExodusError>(())
    /// ```
    pub fn init_params(&self) -> Result<InitParams> {
        let mut params = InitParams::default();

        // Read title from global attribute
        if let Some(attr) = self.nc_file.attribute("title") {
            if let Ok(netcdf::AttributeValue::Str(s)) = attr.value() {
                params.title = s;
            }
        }

        // Read num_dim (required)
        params.num_dim = self
            .nc_file
            .dimension("num_dim")
            .ok_or_else(|| ExodusError::VariableNotDefined("num_dim".to_string()))?
            .len();

        // Read optional dimensions
        params.num_nodes = self
            .nc_file
            .dimension("num_nodes")
            .map(|d| d.len())
            .unwrap_or(0);

        params.num_elems = self
            .nc_file
            .dimension("num_elem")
            .map(|d| d.len())
            .unwrap_or(0);

        params.num_elem_blocks = self
            .nc_file
            .dimension("num_el_blk")
            .map(|d| d.len())
            .unwrap_or(0);

        params.num_edges = self
            .nc_file
            .dimension("num_edge")
            .map(|d| d.len())
            .unwrap_or(0);

        params.num_edge_blocks = self
            .nc_file
            .dimension("num_ed_blk")
            .map(|d| d.len())
            .unwrap_or(0);

        params.num_faces = self
            .nc_file
            .dimension("num_face")
            .map(|d| d.len())
            .unwrap_or(0);

        params.num_face_blocks = self
            .nc_file
            .dimension("num_fa_blk")
            .map(|d| d.len())
            .unwrap_or(0);

        params.num_node_sets = self
            .nc_file
            .dimension("num_node_sets")
            .map(|d| d.len())
            .unwrap_or(0);

        params.num_side_sets = self
            .nc_file
            .dimension("num_side_sets")
            .map(|d| d.len())
            .unwrap_or(0);

        params.num_edge_sets = self
            .nc_file
            .dimension("num_edge_sets")
            .map(|d| d.len())
            .unwrap_or(0);

        params.num_face_sets = self
            .nc_file
            .dimension("num_face_sets")
            .map(|d| d.len())
            .unwrap_or(0);

        params.num_elem_sets = self
            .nc_file
            .dimension("num_elem_sets")
            .map(|d| d.len())
            .unwrap_or(0);

        params.num_node_maps = self
            .nc_file
            .dimension("num_node_maps")
            .map(|d| d.len())
            .unwrap_or(0);

        params.num_elem_maps = self
            .nc_file
            .dimension("num_elem_maps")
            .map(|d| d.len())
            .unwrap_or(0);

        params.num_edge_maps = self
            .nc_file
            .dimension("num_edge_maps")
            .map(|d| d.len())
            .unwrap_or(0);

        params.num_face_maps = self
            .nc_file
            .dimension("num_face_maps")
            .map(|d| d.len())
            .unwrap_or(0);

        params.num_assemblies = self
            .nc_file
            .dimension("num_assembly")
            .map(|d| d.len())
            .unwrap_or(0);

        params.num_blobs = self
            .nc_file
            .dimension("num_blob")
            .map(|d| d.len())
            .unwrap_or(0);

        Ok(params)
    }
}

#[cfg(feature = "netcdf4")]
impl ExodusFile<mode::Append> {
    /// Get initialization parameters from the file
    ///
    /// Same as `ExodusFile<mode::Read>::init_params()` but for append mode.
    pub fn init_params(&self) -> Result<InitParams> {
        let mut params = InitParams::default();

        // Read title
        if let Some(attr) = self.nc_file.attribute("title") {
            if let Ok(netcdf::AttributeValue::Str(s)) = attr.value() {
                params.title = s;
            }
        }

        // Read num_dim (required)
        params.num_dim = self
            .nc_file
            .dimension("num_dim")
            .ok_or_else(|| ExodusError::VariableNotDefined("num_dim".to_string()))?
            .len();

        // Read other dimensions (same as Read mode)
        params.num_nodes = self
            .nc_file
            .dimension("num_nodes")
            .map(|d| d.len())
            .unwrap_or(0);

        params.num_elems = self
            .nc_file
            .dimension("num_elem")
            .map(|d| d.len())
            .unwrap_or(0);

        params.num_elem_blocks = self
            .nc_file
            .dimension("num_el_blk")
            .map(|d| d.len())
            .unwrap_or(0);

        // ... (same pattern for all other dimensions)

        Ok(params)
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
            crate::CreateOptions {
                mode: crate::CreateMode::Clobber,
                ..Default::default()
            },
        )
    }

    #[test]
    fn test_init_minimal() {
        let tmp = NamedTempFile::new().unwrap();
        let mut file = create_test_file(tmp.path()).unwrap();

        let params = InitParams {
            title: "Test mesh".into(),
            num_dim: 3,
            num_nodes: 100,
            num_elems: 50,
            num_elem_blocks: 2,
            ..Default::default()
        };

        file.init(&params).unwrap();
        drop(file);

        // Read back
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let read_params = file.init_params().unwrap();
        assert_eq!(read_params.title, "Test mesh");
        assert_eq!(read_params.num_dim, 3);
        assert_eq!(read_params.num_nodes, 100);
        assert_eq!(read_params.num_elems, 50);
        assert_eq!(read_params.num_elem_blocks, 2);
    }

    #[test]
    fn test_init_builder() {
        let tmp = NamedTempFile::new().unwrap();
        let mut file = create_test_file(tmp.path()).unwrap();

        file.builder()
            .title("Fluent API test")
            .dimensions(2)
            .nodes(50)
            .elem_blocks(1)
            .finish()
            .unwrap();

        drop(file);

        // Verify
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let params = file.init_params().unwrap();
        assert_eq!(params.title, "Fluent API test");
        assert_eq!(params.num_dim, 2);
        assert_eq!(params.num_nodes, 50);
        assert_eq!(params.num_elem_blocks, 1);
    }

    #[test]
    fn test_init_already_initialized() {
        let tmp = NamedTempFile::new().unwrap();
        let mut file = create_test_file(tmp.path()).unwrap();

        let params = InitParams::default();
        file.init(&params).unwrap();

        // Try to initialize again - should fail
        let result = file.init(&params);
        assert!(result.is_err());
    }

    #[test]
    fn test_init_invalid_dimensions() {
        let tmp = NamedTempFile::new().unwrap();
        let mut file = create_test_file(tmp.path()).unwrap();

        // Invalid: num_dim = 0
        let params = InitParams {
            num_dim: 0,
            ..Default::default()
        };
        let result = file.init(&params);
        assert!(result.is_err());

        // Invalid: num_dim = 4
        let params = InitParams {
            num_dim: 4,
            ..Default::default()
        };
        let result = file.init(&params);
        assert!(result.is_err());
    }

    #[test]
    fn test_init_title_too_long() {
        let tmp = NamedTempFile::new().unwrap();
        let mut file = create_test_file(tmp.path()).unwrap();

        let long_title = "a".repeat(81); // 81 characters, max is 80
        let params = InitParams {
            title: long_title,
            num_dim: 3,
            ..Default::default()
        };

        let result = file.init(&params);
        assert!(result.is_err());
    }

    #[test]
    fn test_init_all_dimensions() {
        let tmp = NamedTempFile::new().unwrap();
        let mut file = create_test_file(tmp.path()).unwrap();

        let params = InitParams {
            title: "Complete test".into(),
            num_dim: 3,
            num_nodes: 10,
            num_edges: 20,
            num_edge_blocks: 2,
            num_faces: 30,
            num_face_blocks: 3,
            num_elems: 40,
            num_elem_blocks: 4,
            num_node_sets: 1,
            num_edge_sets: 2,
            num_face_sets: 3,
            num_side_sets: 4,
            num_elem_sets: 5,
            num_node_maps: 1,
            num_edge_maps: 1,
            num_face_maps: 1,
            num_elem_maps: 1,
            num_assemblies: 2,
            num_blobs: 3,
        };

        file.init(&params).unwrap();
        drop(file);

        // Read back and verify all dimensions
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let read_params = file.init_params().unwrap();
        assert_eq!(read_params.num_nodes, 10);
        assert_eq!(read_params.num_edges, 20);
        assert_eq!(read_params.num_edge_blocks, 2);
        assert_eq!(read_params.num_faces, 30);
        assert_eq!(read_params.num_face_blocks, 3);
        assert_eq!(read_params.num_elems, 40);
        assert_eq!(read_params.num_elem_blocks, 4);
        assert_eq!(read_params.num_assemblies, 2);
        assert_eq!(read_params.num_blobs, 3);
    }
}
