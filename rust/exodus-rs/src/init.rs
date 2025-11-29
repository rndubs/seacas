//! Database initialization operations
//!
//! This module provides initialization functionality for Exodus files,
//! including parameter setup and builder pattern support.

use crate::error::{ExodusError, Result};
use crate::types::InitParams;
use crate::utils::constants::*;
use crate::{mode, ExodusFile};

#[cfg(feature = "netcdf4")]
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
            self.nc_file
                .add_attribute(ATTR_TITLE, params.title.as_str())?;
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
        if params.title.len() > MAX_TITLE_LENGTH {
            return Err(ExodusError::StringTooLong {
                max: MAX_TITLE_LENGTH,
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
            .add_dimension(DIM_NUM_DIM, params.num_dim)
            .map_err(ExodusError::NetCdf)?;
        self.metadata
            .dim_cache
            .insert(DIM_NUM_DIM.to_string(), params.num_dim);

        // Create time dimension (unlimited)
        self.nc_file
            .add_unlimited_dimension(DIM_TIME_STEP)
            .map_err(ExodusError::NetCdf)?;

        // Create node-related dimensions
        if params.num_nodes > 0 {
            self.nc_file
                .add_dimension(DIM_NUM_NODES, params.num_nodes)
                .map_err(ExodusError::NetCdf)?;
            self.metadata
                .dim_cache
                .insert(DIM_NUM_NODES.to_string(), params.num_nodes);
        }

        // Create element-related dimensions
        if params.num_elems > 0 {
            self.nc_file
                .add_dimension(DIM_NUM_ELEM, params.num_elems)
                .map_err(ExodusError::NetCdf)?;
            self.metadata
                .dim_cache
                .insert(DIM_NUM_ELEM.to_string(), params.num_elems);
        }

        if params.num_elem_blocks > 0 {
            self.nc_file
                .add_dimension(DIM_NUM_EL_BLK, params.num_elem_blocks)
                .map_err(ExodusError::NetCdf)?;
            self.metadata
                .dim_cache
                .insert(DIM_NUM_EL_BLK.to_string(), params.num_elem_blocks);
        }

        // Create edge-related dimensions
        if params.num_edges > 0 {
            self.nc_file
                .add_dimension(DIM_NUM_EDGE, params.num_edges)
                .map_err(ExodusError::NetCdf)?;
            self.metadata
                .dim_cache
                .insert(DIM_NUM_EDGE.to_string(), params.num_edges);
        }

        if params.num_edge_blocks > 0 {
            self.nc_file
                .add_dimension(DIM_NUM_ED_BLK, params.num_edge_blocks)
                .map_err(ExodusError::NetCdf)?;
            self.metadata
                .dim_cache
                .insert(DIM_NUM_ED_BLK.to_string(), params.num_edge_blocks);
        }

        // Create face-related dimensions
        if params.num_faces > 0 {
            self.nc_file
                .add_dimension(DIM_NUM_FACE, params.num_faces)
                .map_err(ExodusError::NetCdf)?;
            self.metadata
                .dim_cache
                .insert(DIM_NUM_FACE.to_string(), params.num_faces);
        }

        if params.num_face_blocks > 0 {
            self.nc_file
                .add_dimension(DIM_NUM_FA_BLK, params.num_face_blocks)
                .map_err(ExodusError::NetCdf)?;
            self.metadata
                .dim_cache
                .insert(DIM_NUM_FA_BLK.to_string(), params.num_face_blocks);
        }

        // Create set-related dimensions
        if params.num_node_sets > 0 {
            self.nc_file
                .add_dimension(DIM_NUM_NODE_SETS, params.num_node_sets)
                .map_err(ExodusError::NetCdf)?;
            self.metadata
                .dim_cache
                .insert(DIM_NUM_NODE_SETS.to_string(), params.num_node_sets);
        }

        if params.num_side_sets > 0 {
            self.nc_file
                .add_dimension(DIM_NUM_SIDE_SETS, params.num_side_sets)
                .map_err(ExodusError::NetCdf)?;
            self.metadata
                .dim_cache
                .insert(DIM_NUM_SIDE_SETS.to_string(), params.num_side_sets);
        }

        if params.num_edge_sets > 0 {
            self.nc_file
                .add_dimension(DIM_NUM_EDGE_SETS, params.num_edge_sets)
                .map_err(ExodusError::NetCdf)?;
            self.metadata
                .dim_cache
                .insert(DIM_NUM_EDGE_SETS.to_string(), params.num_edge_sets);
        }

        if params.num_face_sets > 0 {
            self.nc_file
                .add_dimension(DIM_NUM_FACE_SETS, params.num_face_sets)
                .map_err(ExodusError::NetCdf)?;
            self.metadata
                .dim_cache
                .insert(DIM_NUM_FACE_SETS.to_string(), params.num_face_sets);
        }

        if params.num_elem_sets > 0 {
            self.nc_file
                .add_dimension(DIM_NUM_ELEM_SETS, params.num_elem_sets)
                .map_err(ExodusError::NetCdf)?;
            self.metadata
                .dim_cache
                .insert(DIM_NUM_ELEM_SETS.to_string(), params.num_elem_sets);
        }

        // Create map-related dimensions
        if params.num_node_maps > 0 {
            self.nc_file
                .add_dimension(DIM_NUM_NODE_MAPS, params.num_node_maps)
                .map_err(ExodusError::NetCdf)?;
            self.metadata
                .dim_cache
                .insert(DIM_NUM_NODE_MAPS.to_string(), params.num_node_maps);
        }

        if params.num_elem_maps > 0 {
            self.nc_file
                .add_dimension(DIM_NUM_ELEM_MAPS, params.num_elem_maps)
                .map_err(ExodusError::NetCdf)?;
            self.metadata
                .dim_cache
                .insert(DIM_NUM_ELEM_MAPS.to_string(), params.num_elem_maps);
        }

        if params.num_edge_maps > 0 {
            self.nc_file
                .add_dimension(DIM_NUM_EDGE_MAPS, params.num_edge_maps)
                .map_err(ExodusError::NetCdf)?;
            self.metadata
                .dim_cache
                .insert(DIM_NUM_EDGE_MAPS.to_string(), params.num_edge_maps);
        }

        if params.num_face_maps > 0 {
            self.nc_file
                .add_dimension(DIM_NUM_FACE_MAPS, params.num_face_maps)
                .map_err(ExodusError::NetCdf)?;
            self.metadata
                .dim_cache
                .insert(DIM_NUM_FACE_MAPS.to_string(), params.num_face_maps);
        }

        // Create assembly and blob dimensions
        if params.num_assemblies > 0 {
            self.nc_file
                .add_dimension(DIM_NUM_ASSEMBLY, params.num_assemblies)
                .map_err(ExodusError::NetCdf)?;
            self.metadata
                .dim_cache
                .insert(DIM_NUM_ASSEMBLY.to_string(), params.num_assemblies);
        }

        if params.num_blobs > 0 {
            self.nc_file
                .add_dimension(DIM_NUM_BLOB, params.num_blobs)
                .map_err(ExodusError::NetCdf)?;
            self.metadata
                .dim_cache
                .insert(DIM_NUM_BLOB.to_string(), params.num_blobs);
        }

        // Create coordinate variables if we have nodes
        if params.num_nodes > 0 {
            self.create_coord_variables()?;
        }

        // Create block ID property variables
        if params.num_elem_blocks > 0 {
            self.create_block_id_variable("eb_prop1", DIM_NUM_EL_BLK)?;
        }
        if params.num_edge_blocks > 0 {
            self.create_block_id_variable("ed_prop1", DIM_NUM_ED_BLK)?;
        }
        if params.num_face_blocks > 0 {
            self.create_block_id_variable("fa_prop1", DIM_NUM_FA_BLK)?;
        }

        Ok(())
    }

    /// Create coordinate variables
    fn create_coord_variables(&mut self) -> Result<()> {
        // Get chunking configuration from performance config
        let requested_chunk = self
            .metadata
            .performance
            .as_ref()
            .map(|p| p.chunks.node_chunk_size)
            .unwrap_or(0); // 0 means use default/no chunking

        // Get actual num_nodes to clamp chunk size (chunk can't exceed dimension)
        let num_nodes = self
            .metadata
            .dim_cache
            .get(DIM_NUM_NODES)
            .copied()
            .unwrap_or(0);

        // Clamp chunk size: must be > 0 and <= dimension size
        // Only apply chunking if data is large enough to benefit from it
        let chunk_size = if requested_chunk > 0 && num_nodes > 0 {
            requested_chunk.min(num_nodes)
        } else {
            0
        };

        // Create coordx variable
        let mut var_x = self
            .nc_file
            .add_variable::<f64>(VAR_COORD_X, &[DIM_NUM_NODES])
            .map_err(ExodusError::NetCdf)?;

        // Apply chunking if specified and valid
        if chunk_size > 0 {
            var_x
                .set_chunking(&[chunk_size])
                .map_err(ExodusError::NetCdf)?;
        }

        // Create coordy variable
        let mut var_y = self
            .nc_file
            .add_variable::<f64>(VAR_COORD_Y, &[DIM_NUM_NODES])
            .map_err(ExodusError::NetCdf)?;

        if chunk_size > 0 {
            var_y
                .set_chunking(&[chunk_size])
                .map_err(ExodusError::NetCdf)?;
        }

        // Create coordz variable
        let mut var_z = self
            .nc_file
            .add_variable::<f64>(VAR_COORD_Z, &[DIM_NUM_NODES])
            .map_err(ExodusError::NetCdf)?;

        if chunk_size > 0 {
            var_z
                .set_chunking(&[chunk_size])
                .map_err(ExodusError::NetCdf)?;
        }

        Ok(())
    }

    /// Create block ID property variable
    fn create_block_id_variable(&mut self, var_name: &str, dim_name: &str) -> Result<()> {
        let mut var = self
            .nc_file
            .add_variable::<i64>(var_name, &[dim_name])
            .map_err(ExodusError::NetCdf)?;

        // Add attribute to identify this as a property variable
        var.put_attribute("name", "ID")
            .map_err(ExodusError::NetCdf)?;

        Ok(())
    }
}

// =============================================================================
// Read Operations (available in all modes)
// =============================================================================

#[cfg(feature = "netcdf4")]
impl<M: crate::FileMode> ExodusFile<M> {
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
        if let Some(attr) = self.nc_file.attribute(ATTR_TITLE) {
            if let Ok(netcdf::AttributeValue::Str(s)) = attr.value() {
                params.title = s;
            }
        }

        // Read num_dim (required) - use cache-aware helper
        params.num_dim = self.get_dimension_len_required(DIM_NUM_DIM)?;

        // Read optional dimensions using cache-aware helper for efficiency
        // This prioritizes the metadata cache over re-querying the NetCDF file
        params.num_nodes = self.get_dimension_len(DIM_NUM_NODES);
        params.num_elems = self.get_dimension_len(DIM_NUM_ELEM);
        params.num_elem_blocks = self.get_dimension_len(DIM_NUM_EL_BLK);
        params.num_edges = self.get_dimension_len(DIM_NUM_EDGE);
        params.num_edge_blocks = self.get_dimension_len(DIM_NUM_ED_BLK);
        params.num_faces = self.get_dimension_len(DIM_NUM_FACE);
        params.num_face_blocks = self.get_dimension_len(DIM_NUM_FA_BLK);
        params.num_node_sets = self.get_dimension_len(DIM_NUM_NODE_SETS);
        params.num_side_sets = self.get_dimension_len(DIM_NUM_SIDE_SETS);
        params.num_edge_sets = self.get_dimension_len(DIM_NUM_EDGE_SETS);
        params.num_face_sets = self.get_dimension_len(DIM_NUM_FACE_SETS);
        params.num_elem_sets = self.get_dimension_len(DIM_NUM_ELEM_SETS);
        params.num_node_maps = self.get_dimension_len(DIM_NUM_NODE_MAPS);
        params.num_elem_maps = self.get_dimension_len(DIM_NUM_ELEM_MAPS);
        params.num_edge_maps = self.get_dimension_len(DIM_NUM_EDGE_MAPS);
        params.num_face_maps = self.get_dimension_len(DIM_NUM_FACE_MAPS);
        params.num_assemblies = self.get_dimension_len(DIM_NUM_ASSEMBLY);
        params.num_blobs = self.get_dimension_len(DIM_NUM_BLOB);

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
