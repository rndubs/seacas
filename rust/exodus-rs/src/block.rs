//! Block operations
//!
//! This module provides operations for element, edge, and face blocks.
//! Blocks define the topology and connectivity of mesh entities.

use crate::error::{EntityId, ExodusError, Result};
use crate::types::{Block, Connectivity, EntityType, Topology};
use crate::utils::naming;
use crate::{mode, ExodusFile, FileMode};

// Common block operations (available in all modes)
#[cfg(feature = "netcdf4")]
impl<M: FileMode> ExodusFile<M> {
    /// Get all block IDs of a given type
    ///
    /// # Arguments
    ///
    /// * `entity_type` - Type of block (ElemBlock, EdgeBlock, or FaceBlock)
    ///
    /// # Returns
    ///
    /// Vector of block IDs
    ///
    /// # Errors
    ///
    /// Returns an error if entity type is not a block type
    pub fn block_ids(&self, entity_type: EntityType) -> Result<Vec<EntityId>> {
        let id_var_name = naming::prop_id_var(entity_type);

        if let Some(var) = self.nc_file.variable(id_var_name) {
            let ids: Vec<i64> = var.get_values(..)?;
            Ok(ids)
        } else {
            Ok(Vec::new())
        }
    }

    /// Get block parameters
    ///
    /// # Arguments
    ///
    /// * `block_id` - ID of the block to query
    ///
    /// # Returns
    ///
    /// Block parameters including topology, sizes, etc.
    pub fn block(&self, block_id: EntityId) -> Result<Block> {
        // Try to find the block in elem blocks first
        if let Ok(block) = self.get_block_info(EntityType::ElemBlock, block_id) {
            return Ok(block);
        }

        // Try edge blocks
        if let Ok(block) = self.get_block_info(EntityType::EdgeBlock, block_id) {
            return Ok(block);
        }

        // Try face blocks
        if let Ok(block) = self.get_block_info(EntityType::FaceBlock, block_id) {
            return Ok(block);
        }

        Err(ExodusError::EntityNotFound {
            entity_type: EntityType::ElemBlock.to_string(),
            id: block_id,
        })
    }

    /// Get element connectivity for a block
    ///
    /// # Arguments
    ///
    /// * `block_id` - ID of the block
    ///
    /// # Returns
    ///
    /// Flat array of node IDs
    pub fn connectivity(&self, block_id: EntityId) -> Result<Vec<i64>> {
        let block_index = self.find_block_index(EntityType::ElemBlock, block_id)?;
        let conn_var_name = naming::connectivity_var(block_index);

        let var = self
            .nc_file
            .variable(&conn_var_name)
            .ok_or_else(|| ExodusError::VariableNotDefined(conn_var_name.clone()))?;

        let conn_i32: Vec<i32> = var.get_values(..)?;
        let conn: Vec<i64> = conn_i32.iter().map(|&x| x as i64).collect();

        Ok(conn)
    }

    /// Get connectivity as structured data
    ///
    /// # Arguments
    ///
    /// * `block_id` - ID of the block
    ///
    /// # Returns
    ///
    /// Connectivity struct with shape information
    pub fn connectivity_structured(&self, block_id: EntityId) -> Result<Connectivity> {
        let block = self.block(block_id)?;
        let data = self.connectivity(block_id)?;

        Ok(Connectivity {
            block_id,
            topology: Topology::from_string(&block.topology),
            data,
            num_entries: block.num_entries,
            nodes_per_entry: block.num_nodes_per_entry,
        })
    }

    /// Get connectivity as a 2D ndarray (NumPy-compatible)
    ///
    /// Returns connectivity as a 2D ndarray with shape (num_elements, nodes_per_element).
    /// This is more efficient for NumPy integration via PyO3 as it provides a contiguous
    /// memory layout compatible with NumPy arrays.
    ///
    /// # Arguments
    ///
    /// * `block_id` - ID of the block
    ///
    /// # Returns
    ///
    /// An `Array2<i64>` with shape (num_elements, nodes_per_element) where each row
    /// contains the node IDs for one element.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Block is not found
    /// - Connectivity variable is not defined
    /// - NetCDF read fails
    /// - Data cannot be reshaped (inconsistent dimensions)
    #[cfg(feature = "ndarray")]
    pub fn connectivity_array(&self, block_id: EntityId) -> Result<ndarray::Array2<i64>> {
        use ndarray::Array2;

        // Get block info for dimensions (avoids reading through connectivity_structured)
        let block = self.block(block_id)?;

        // Handle empty case
        if block.num_entries == 0 || block.num_nodes_per_entry == 0 {
            return Ok(Array2::zeros((0, 0)));
        }

        // Read connectivity data directly (avoids intermediate Connectivity struct allocation)
        let data = self.connectivity(block_id)?;

        // Reshape flat vector directly into 2D array (num_elements, nodes_per_element)
        // from_shape_vec takes ownership without copying the data
        Array2::from_shape_vec((block.num_entries, block.num_nodes_per_entry), data).map_err(|e| {
            ExodusError::Other(format!(
                "Failed to reshape connectivity array for block {}: {}",
                block_id, e
            ))
        })
    }

    /// Get block attributes
    ///
    /// # Arguments
    ///
    /// * `block_id` - ID of the block
    ///
    /// # Returns
    ///
    /// Flat array of attribute values
    pub fn block_attributes(&self, block_id: EntityId) -> Result<Vec<f64>> {
        let block_index = self.find_block_index(EntityType::ElemBlock, block_id)?;
        let attr_var_name = naming::block_attribute_var(block_index);

        if let Some(var) = self.nc_file.variable(&attr_var_name) {
            let attrs: Vec<f64> = var.get_values(..)?;
            Ok(attrs)
        } else {
            Ok(Vec::new())
        }
    }

    /// Get block attribute names
    ///
    /// # Arguments
    ///
    /// * `block_id` - ID of the block
    ///
    /// # Returns
    ///
    /// Vector of attribute names
    pub fn block_attribute_names(&self, block_id: EntityId) -> Result<Vec<String>> {
        let block_index = self.find_block_index(EntityType::ElemBlock, block_id)?;
        let attr_name_var = naming::block_attribute_name_var(block_index);

        if let Some(var) = self.nc_file.variable(&attr_name_var) {
            let block = self.block(block_id)?;
            let mut names = Vec::with_capacity(block.num_attributes);

            for i in 0..block.num_attributes {
                let bytes: Vec<i8> = var.get_values((i..i + 1, 0..33))?;
                let name = String::from_utf8_lossy(
                    &bytes
                        .iter()
                        .take_while(|&&b| b != 0)
                        .map(|&b| b as u8)
                        .collect::<Vec<u8>>(),
                )
                .trim()
                .to_string();
                names.push(name);
            }

            Ok(names)
        } else {
            Ok(Vec::new())
        }
    }

    // Internal helper to get block info
    fn get_block_info(&self, entity_type: EntityType, block_id: EntityId) -> Result<Block> {
        let block_index = self.find_block_index(entity_type, block_id)?;
        let conn_var_name = naming::connectivity_var(block_index);

        let var = self
            .nc_file
            .variable(&conn_var_name)
            .ok_or_else(|| ExodusError::VariableNotDefined(conn_var_name.clone()))?;

        // Get topology from attribute
        let topology = var
            .attribute("elem_type")
            .and_then(|attr| {
                if let Ok(netcdf::AttributeValue::Str(s)) = attr.value() {
                    Some(s)
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "UNKNOWN".to_string());

        // Get dimensions
        let dims = var.dimensions();
        let num_entries = dims.first().map(|d| d.len()).unwrap_or(0);
        let num_nodes_per_entry = dims.get(1).map(|d| d.len()).unwrap_or(0);

        // Check for attributes
        let attr_dim_name = naming::block_attributes_dim(block_index);
        let num_attributes = self
            .nc_file
            .dimension(&attr_dim_name)
            .map(|d| d.len())
            .unwrap_or(0);

        Ok(Block {
            id: block_id,
            entity_type,
            topology,
            num_entries,
            num_nodes_per_entry,
            num_edges_per_entry: 0,
            num_faces_per_entry: 0,
            num_attributes,
        })
    }

    // Internal helper to find block index by ID
    fn find_block_index(&self, entity_type: EntityType, block_id: EntityId) -> Result<usize> {
        let id_var_name = naming::prop_id_var(entity_type);

        if let Some(var) = self.nc_file.variable(id_var_name) {
            let ids: Vec<i64> = var.get_values(..)?;
            ids.iter()
                .position(|&id| id == block_id)
                .ok_or_else(|| ExodusError::EntityNotFound {
                    entity_type: entity_type.to_string(),
                    id: block_id,
                })
        } else {
            Err(ExodusError::EntityNotFound {
                entity_type: entity_type.to_string(),
                id: block_id,
            })
        }
    }
}

// Block operations for write mode
#[cfg(feature = "netcdf4")]
impl ExodusFile<mode::Write> {
    /// Define an element/edge/face block
    ///
    /// This creates the NetCDF dimensions and variables needed to store block data.
    ///
    /// # Arguments
    ///
    /// * `block` - Block parameters including ID, topology, and sizes
    ///
    /// # Errors
    ///
    /// - File not initialized
    /// - Block already exists
    /// - Invalid topology
    /// - NetCDF errors
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use exodus_rs::{ExodusFile, Block, EntityType};
    ///
    /// let block = Block {
    ///     id: 100,
    ///     entity_type: EntityType::ElemBlock,
    ///     topology: "HEX8".into(),
    ///     num_entries: 10,
    ///     num_nodes_per_entry: 8,
    ///     num_edges_per_entry: 0,
    ///     num_faces_per_entry: 0,
    ///     num_attributes: 0,
    /// };
    /// file.put_block(&block)?;
    /// # Ok::<(), exodus_rs::ExodusError>(())
    /// ```
    pub fn put_block(&mut self, block: &Block) -> Result<()> {
        // Ensure we're in define mode for adding block definitions
        self.ensure_define_mode()?;

        if !self.metadata.initialized {
            return Err(ExodusError::NotInitialized);
        }

        // Validate block type
        match block.entity_type {
            EntityType::ElemBlock | EntityType::EdgeBlock | EntityType::FaceBlock => {}
            _ => {
                return Err(ExodusError::InvalidEntityType(format!(
                    "Expected block type, got {:?}",
                    block.entity_type
                )))
            }
        }

        // Validate topology
        let topology = Topology::from_string(&block.topology);
        if let Some(expected) = topology.expected_nodes() {
            if expected != block.num_nodes_per_entry {
                return Err(ExodusError::InvalidTopology(format!(
                    "Topology {} expects {} nodes, got {}",
                    topology, expected, block.num_nodes_per_entry
                )));
            }
        }

        let block_index = self.get_block_index(block.entity_type, block.id)?;

        // Create dimensions for this block using naming helpers
        let dim_name_entries = naming::block_entries_dim(block.entity_type, block_index);
        let dim_name_nodes = naming::block_nodes_per_entry_dim(block.entity_type, block_index);

        self.nc_file
            .add_dimension(&dim_name_entries, block.num_entries)?;
        self.nc_file
            .add_dimension(&dim_name_nodes, block.num_nodes_per_entry)?;

        // Create connectivity variable
        let conn_var_name = naming::connectivity_var(block_index);
        let mut conn_var = self
            .nc_file
            .add_variable::<i32>(&conn_var_name, &[&dim_name_entries, &dim_name_nodes])?;

        // Apply chunking if configured (clamp to dimension size to avoid NC_EBADCHUNK)
        let requested_chunk = self
            .metadata
            .performance
            .as_ref()
            .map(|p| p.chunks.element_chunk_size)
            .unwrap_or(0);
        let chunk_size = if requested_chunk > 0 && block.num_entries > 0 {
            requested_chunk.min(block.num_entries)
        } else {
            0
        };
        if chunk_size > 0 {
            // Chunk along the elements dimension (first dimension)
            // Second dimension is typically small (nodes per element), so use full size
            conn_var.set_chunking(&[chunk_size, block.num_nodes_per_entry])?;
        }

        // Set topology attribute
        conn_var.put_attribute("elem_type", block.topology.as_str())?;

        // Set block ID
        let id_var_name = naming::prop_id_var(block.entity_type);
        if let Some(mut id_var) = self.nc_file.variable_mut(id_var_name) {
            // Use put_values with a slice instead of put_value
            id_var.put_values(&[block.id], block_index..block_index + 1)?;
        }

        // Create attribute variable if needed
        if block.num_attributes > 0 {
            let attr_dim_name = naming::block_attributes_dim(block_index);
            self.nc_file
                .add_dimension(&attr_dim_name, block.num_attributes)?;

            let attr_var_name = naming::block_attribute_var(block_index);
            let mut attr_var = self
                .nc_file
                .add_variable::<f64>(&attr_var_name, &[&dim_name_entries, &attr_dim_name])?;

            // Apply same chunking to attributes as connectivity (chunk_size already clamped above)
            if chunk_size > 0 {
                attr_var.set_chunking(&[chunk_size, block.num_attributes])?;
            }
        }

        Ok(())
    }

    /// Write element connectivity for a block
    ///
    /// # Arguments
    ///
    /// * `block_id` - ID of the block
    /// * `connectivity` - Flat array of node IDs (1-based indexing)
    ///
    /// # Errors
    ///
    /// - Block not found
    /// - Array length mismatch
    /// - NetCDF errors
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // For a single hex element with 8 nodes
    /// let conn = vec![1, 2, 3, 4, 5, 6, 7, 8];
    /// file.put_connectivity(100, &conn)?;
    /// # Ok::<(), exodus_rs::ExodusError>(())
    /// ```
    pub fn put_connectivity(&mut self, block_id: EntityId, connectivity: &[i64]) -> Result<()> {
        // Ensure we're in data mode for writing connectivity values
        self.ensure_data_mode()?;

        // Try to find the block in all block types (ElemBlock, EdgeBlock, FaceBlock)
        let (block_index, _entity_type) = self.find_block_in_any_type_write(block_id)?;
        let conn_var_name = naming::connectivity_var(block_index);

        let mut var = self.nc_file.variable_mut(&conn_var_name).ok_or_else(|| {
            ExodusError::VariableNotDefined(format!("Connectivity variable {}", conn_var_name))
        })?;

        // Convert i64 to i32 for NetCDF
        let conn_i32: Vec<i32> = connectivity.iter().map(|&x| x as i32).collect();

        // Write the full connectivity array
        var.put_values(&conn_i32, ..)?;

        Ok(())
    }

    /// Write attributes for a block
    ///
    /// # Arguments
    ///
    /// * `block_id` - ID of the block
    /// * `attributes` - Flat array of attribute values
    ///
    /// # Errors
    ///
    /// - Block not found
    /// - Array length mismatch
    /// - Block has no attributes defined
    pub fn put_block_attributes(&mut self, block_id: EntityId, attributes: &[f64]) -> Result<()> {
        let (block_index, _entity_type) = self.find_block_in_any_type_write(block_id)?;
        let attr_var_name = naming::block_attribute_var(block_index);

        let mut var = self.nc_file.variable_mut(&attr_var_name).ok_or_else(|| {
            ExodusError::VariableNotDefined(format!("Attribute variable {}", attr_var_name))
        })?;

        var.put_values(attributes, ..)?;

        Ok(())
    }

    /// Set attribute names for a block
    ///
    /// # Arguments
    ///
    /// * `block_id` - ID of the block
    /// * `names` - Slice of attribute names
    pub fn put_block_attribute_names(
        &mut self,
        block_id: EntityId,
        names: &[impl AsRef<str>],
    ) -> Result<()> {
        let block_index = self.find_block_index_write(EntityType::ElemBlock, block_id)?;
        let attr_name_var = naming::block_attribute_name_var(block_index);

        // Create the variable if it doesn't exist
        if self.nc_file.variable(&attr_name_var).is_none() {
            let dim_name = naming::block_attributes_dim(block_index);
            let len_string = "len_string";

            // Add len_string dimension if not present
            if self.nc_file.dimension(len_string).is_none() {
                self.nc_file.add_dimension(len_string, 33)?;
            }

            self.nc_file
                .add_variable::<i8>(&attr_name_var, &[&dim_name, len_string])?;
        }

        // Write attribute names
        let mut var = self.nc_file.variable_mut(&attr_name_var).unwrap();
        for (i, name) in names.iter().enumerate() {
            let name_str = name.as_ref();
            let mut buf = vec![0i8; 33];
            for (j, &byte) in name_str.as_bytes().iter().take(32).enumerate() {
                buf[j] = byte as i8;
            }
            var.put_values(&buf, (i..i + 1, 0..33))?;
        }

        Ok(())
    }

    // Helper methods for Write mode

    /// Get the next available block index for new blocks
    fn get_block_index(&self, _entity_type: EntityType, _block_id: EntityId) -> Result<usize> {
        // Count how many blocks have been written by checking for connectivity variables
        let mut count = 0;
        loop {
            let conn_var_name = naming::connectivity_var(count);
            if self.nc_file.variable(&conn_var_name).is_none() {
                break;
            }
            count += 1;
        }
        Ok(count)
    }

    /// Find block index by ID (Write mode specific - uses naming helper)
    fn find_block_index_write(&self, entity_type: EntityType, block_id: EntityId) -> Result<usize> {
        let id_var_name = naming::prop_id_var(entity_type);

        if let Some(var) = self.nc_file.variable(id_var_name) {
            let ids: Vec<i64> = var.get_values(..)?;
            ids.iter()
                .position(|&id| id == block_id)
                .ok_or_else(|| ExodusError::EntityNotFound {
                    entity_type: entity_type.to_string(),
                    id: block_id,
                })
        } else {
            Err(ExodusError::EntityNotFound {
                entity_type: entity_type.to_string(),
                id: block_id,
            })
        }
    }

    /// Find a block by ID across all block types (ElemBlock, EdgeBlock, FaceBlock)
    fn find_block_in_any_type_write(&self, block_id: EntityId) -> Result<(usize, EntityType)> {
        // Try ElemBlock first
        if let Ok(index) = self.find_block_index_write(EntityType::ElemBlock, block_id) {
            return Ok((index, EntityType::ElemBlock));
        }
        // Try EdgeBlock
        if let Ok(index) = self.find_block_index_write(EntityType::EdgeBlock, block_id) {
            return Ok((index, EntityType::EdgeBlock));
        }
        // Try FaceBlock
        if let Ok(index) = self.find_block_index_write(EntityType::FaceBlock, block_id) {
            return Ok((index, EntityType::FaceBlock));
        }
        // Not found in any block type
        Err(ExodusError::EntityNotFound {
            entity_type: "block (elem/edge/face)".to_string(),
            id: block_id,
        })
    }
}

// Note: Read-only block operations (block, connectivity, block_attributes, etc.)
// are now available in the generic impl<M: FileMode> ExodusFile<M> block above.

// Note: Append mode uses the generic impl<M: FileMode> ExodusFile<M> for read operations.

#[cfg(test)]
#[cfg(feature = "netcdf4")]
mod tests {
    use super::*;
    use crate::types::InitParams;
    use tempfile::NamedTempFile;

    #[test]
    fn test_hex_block() {
        let tmp = NamedTempFile::new().unwrap();

        // Write
        {
            // Use clobber mode since NamedTempFile creates the file
            let options = crate::types::CreateOptions {
                mode: crate::types::CreateMode::Clobber,
                ..Default::default()
            };
            let mut file = ExodusFile::create(tmp.path(), options).unwrap();
            let params = InitParams {
                title: "Test Hex Block".into(),
                num_dim: 3,
                num_nodes: 8,
                num_elems: 1,
                num_elem_blocks: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            // Define hex block
            let block = Block {
                id: 100,
                entity_type: EntityType::ElemBlock,
                topology: "HEX8".into(),
                num_entries: 1,
                num_nodes_per_entry: 8,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block).unwrap();

            // Write connectivity (1-based node IDs)
            let conn = vec![1, 2, 3, 4, 5, 6, 7, 8];
            file.put_connectivity(100, &conn).unwrap();
        }

        // Read
        {
            let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
            let ids = file.block_ids(EntityType::ElemBlock).unwrap();
            assert_eq!(ids, vec![100]);

            let block = file.block(100).unwrap();
            assert_eq!(block.topology, "HEX8");
            assert_eq!(block.num_entries, 1);
            assert_eq!(block.num_nodes_per_entry, 8);

            let conn = file.connectivity(100).unwrap();
            assert_eq!(conn.len(), 8);
            assert_eq!(conn, vec![1, 2, 3, 4, 5, 6, 7, 8]);
        }
    }

    #[test]
    fn test_quad_block() {
        let tmp = NamedTempFile::new().unwrap();

        {
            let options = crate::types::CreateOptions {
                mode: crate::types::CreateMode::Clobber,
                ..Default::default()
            };
            let mut file = ExodusFile::create(tmp.path(), options).unwrap();
            let params = InitParams {
                title: "Test Quad Block".into(),
                num_dim: 2,
                num_nodes: 4,
                num_elems: 1,
                num_elem_blocks: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let block = Block {
                id: 10,
                entity_type: EntityType::ElemBlock,
                topology: "QUAD4".into(),
                num_entries: 1,
                num_nodes_per_entry: 4,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block).unwrap();

            let conn = vec![1, 2, 3, 4];
            file.put_connectivity(10, &conn).unwrap();
        }

        {
            let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
            let block = file.block(10).unwrap();
            assert_eq!(block.topology, "QUAD4");

            let conn_struct = file.connectivity_structured(10).unwrap();
            assert_eq!(conn_struct.num_entries, 1);
            assert_eq!(conn_struct.nodes_per_entry, 4);
            assert_eq!(conn_struct.entry(0).unwrap(), &[1, 2, 3, 4]);
        }
    }

    #[test]
    fn test_topology_parsing() {
        assert_eq!(Topology::from_string("HEX8"), Topology::Hex8);
        assert_eq!(Topology::from_string("hex"), Topology::Hex8);
        assert_eq!(Topology::from_string("QUAD4"), Topology::Quad4);
        assert_eq!(Topology::from_string("TET10"), Topology::Tet10);

        assert_eq!(Topology::Hex8.expected_nodes(), Some(8));
        assert_eq!(Topology::Quad4.expected_nodes(), Some(4));
        assert_eq!(Topology::Tet10.expected_nodes(), Some(10));
        assert_eq!(Topology::NSided.expected_nodes(), None);
    }

    #[test]
    fn test_connectivity_iterator() {
        let conn = Connectivity {
            block_id: 1,
            topology: Topology::Hex8,
            data: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
            num_entries: 2,
            nodes_per_entry: 8,
        };

        let entries: Vec<&[i64]> = conn.iter().collect();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0], &[1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(entries[1], &[9, 10, 11, 12, 13, 14, 15, 16]);
    }
}
