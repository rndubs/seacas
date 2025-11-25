//! Map and naming operations for Exodus II files.
//!
//! This module provides functionality for:
//! - Entity ID maps (mapping internal indices to user-defined IDs)
//! - Element order maps
//! - Entity naming
//! - Property arrays

use crate::error::{ExodusError, Result};
use crate::types::EntityType;
use crate::{mode, ExodusFile, FileMode};

// ============================================================================
// ID Maps
// ============================================================================

impl<M: FileMode> ExodusFile<M> {
    /// Get the NetCDF variable name for an entity ID map
    fn id_map_var_name(entity_type: EntityType) -> Result<&'static str> {
        match entity_type {
            EntityType::NodeMap => Ok("node_num_map"),
            EntityType::ElemMap => Ok("elem_num_map"),
            EntityType::EdgeMap => Ok("edge_num_map"),
            EntityType::FaceMap => Ok("face_num_map"),
            EntityType::ElemBlock => Ok("elem_map"), // Global element map
            _ => Err(ExodusError::InvalidEntityType(format!(
                "Entity type {:?} does not support ID maps",
                entity_type
            ))),
        }
    }

    /// Get the NetCDF dimension name for an entity ID map
    fn id_map_dim_name(entity_type: EntityType) -> Result<&'static str> {
        match entity_type {
            EntityType::NodeMap | EntityType::Nodal => Ok("num_nodes"),
            EntityType::ElemMap | EntityType::ElemBlock => Ok("num_elem"),
            EntityType::EdgeMap => Ok("num_edge"),
            EntityType::FaceMap => Ok("num_face"),
            _ => Err(ExodusError::InvalidEntityType(format!(
                "Entity type {:?} does not support ID maps",
                entity_type
            ))),
        }
    }
}

#[cfg(feature = "netcdf4")]
impl ExodusFile<mode::Write> {
    /// Set entity ID map
    ///
    /// ID maps allow mapping from internal 1-based indices to user-defined entity IDs.
    /// If no ID map is defined, entities are numbered sequentially starting from 1.
    ///
    /// # Arguments
    ///
    /// * `entity_type` - Type of entity (NodeMap, ElemMap, EdgeMap, or FaceMap)
    /// * `map` - Array of entity IDs (1-based)
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if:
    /// - The entity type is not valid for ID maps
    /// - The map length doesn't match the number of entities
    /// - NetCDF write fails
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use exodus_rs::*;
    /// # let mut file = ExodusFile::create_default("test.exo").unwrap();
    /// // Custom node numbering starting from 100
    /// let node_map = vec![100, 101, 102, 103];
    /// file.put_id_map(EntityType::NodeMap, &node_map)?;
    /// # Ok::<(), ExodusError>(())
    /// ```
    pub fn put_id_map(&mut self, entity_type: EntityType, map: &[i64]) -> Result<()> {
        let var_name = Self::id_map_var_name(entity_type)?;
        let dim_name = Self::id_map_dim_name(entity_type)?;

        // Get the dimension to validate map length
        let dim_size = self
            .nc_file
            .dimension(dim_name)
            .ok_or_else(|| ExodusError::Other(format!("Dimension {} not found", dim_name)))?
            .len();

        if map.len() != dim_size {
            return Err(ExodusError::InvalidArrayLength {
                expected: dim_size,
                actual: map.len(),
            });
        }

        // Create the variable if it doesn't exist
        if self.nc_file.variable(var_name).is_none() {
            // Verify dimension exists
            self.nc_file
                .dimension(dim_name)
                .ok_or_else(|| ExodusError::Other(format!("Dimension {} not found", dim_name)))?;

            self.nc_file
                .add_variable::<i64>(var_name, &[dim_name])
                .map_err(ExodusError::NetCdf)?;
        }

        // Write the map
        let mut var = self
            .nc_file
            .variable_mut(var_name)
            .ok_or_else(|| ExodusError::VariableNotDefined(var_name.to_string()))?;

        var.put_values(map, ..).map_err(ExodusError::NetCdf)?;

        Ok(())
    }

    /// Set element order map
    ///
    /// The element order map specifies the order in which elements should be processed.
    ///
    /// # Arguments
    ///
    /// * `order` - Array of element indices (1-based) specifying the processing order
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if:
    /// - The map length doesn't match the number of elements
    /// - NetCDF write fails
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use exodus_rs::*;
    /// # let mut file = ExodusFile::create_default("test.exo").unwrap();
    /// // Reverse element ordering
    /// let order = vec![4, 3, 2, 1];
    /// file.put_elem_order_map(&order)?;
    /// # Ok::<(), ExodusError>(())
    /// ```
    pub fn put_elem_order_map(&mut self, order: &[i64]) -> Result<()> {
        let var_name = "elem_order_map";
        let dim_name = "num_elem";

        // Get the dimension to validate order length
        let dim_size = self
            .nc_file
            .dimension(dim_name)
            .ok_or_else(|| ExodusError::Other(format!("Dimension {} not found", dim_name)))?
            .len();

        if order.len() != dim_size {
            return Err(ExodusError::InvalidArrayLength {
                expected: dim_size,
                actual: order.len(),
            });
        }

        // Create the variable if it doesn't exist
        if self.nc_file.variable(var_name).is_none() {
            // Verify dimension exists
            self.nc_file
                .dimension(dim_name)
                .ok_or_else(|| ExodusError::Other(format!("Dimension {} not found", dim_name)))?;

            self.nc_file
                .add_variable::<i64>(var_name, &[dim_name])
                .map_err(ExodusError::NetCdf)?;
        }

        // Write the order map
        let mut var = self
            .nc_file
            .variable_mut(var_name)
            .ok_or_else(|| ExodusError::VariableNotDefined(var_name.to_string()))?;

        var.put_values(order, ..).map_err(ExodusError::NetCdf)?;

        Ok(())
    }
}

#[cfg(feature = "netcdf4")]
impl ExodusFile<mode::Read> {
    /// Get entity ID map
    ///
    /// Returns the ID map for the specified entity type, or None if no map is defined.
    /// If no map exists, entities are numbered sequentially starting from 1.
    ///
    /// # Arguments
    ///
    /// * `entity_type` - Type of entity (NodeMap, ElemMap, EdgeMap, or FaceMap)
    ///
    /// # Returns
    ///
    /// The entity ID map, or an error if reading fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use exodus_rs::*;
    /// # let file = ExodusFile::<mode::Read>::open("test.exo").unwrap();
    /// let node_map = file.id_map(EntityType::NodeMap)?;
    /// # Ok::<(), ExodusError>(())
    /// ```
    pub fn id_map(&self, entity_type: EntityType) -> Result<Vec<i64>> {
        let var_name = Self::id_map_var_name(entity_type)?;

        // Check if the variable exists
        let var = self
            .nc_file
            .variable(var_name)
            .ok_or_else(|| ExodusError::VariableNotDefined(var_name.to_string()))?;

        // Read the map
        let map: Vec<i64> = var.get_values(..).map_err(ExodusError::NetCdf)?;

        Ok(map)
    }

    /// Get element order map
    ///
    /// Returns the element order map, or None if no order map is defined.
    ///
    /// # Returns
    ///
    /// The element order map, or an error if reading fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use exodus_rs::*;
    /// # let file = ExodusFile::<mode::Read>::open("test.exo").unwrap();
    /// let order = file.elem_order_map()?;
    /// # Ok::<(), ExodusError>(())
    /// ```
    pub fn elem_order_map(&self) -> Result<Vec<i64>> {
        let var_name = "elem_order_map";

        // Check if the variable exists
        let var = self
            .nc_file
            .variable(var_name)
            .ok_or_else(|| ExodusError::VariableNotDefined(var_name.to_string()))?;

        // Read the order map
        let order: Vec<i64> = var.get_values(..).map_err(ExodusError::NetCdf)?;

        Ok(order)
    }
}

// ============================================================================
// Entity Names
// ============================================================================

impl<M: FileMode> ExodusFile<M> {
    /// Get the NetCDF variable name for entity names
    fn names_var_name(entity_type: EntityType) -> Result<String> {
        match entity_type {
            EntityType::NodeSet => Ok("ns_names".to_string()),
            EntityType::SideSet => Ok("ss_names".to_string()),
            EntityType::ElemBlock => Ok("eb_names".to_string()),
            EntityType::EdgeBlock => Ok("ed_names".to_string()),
            EntityType::FaceBlock => Ok("fa_names".to_string()),
            EntityType::ElemSet => Ok("els_names".to_string()),
            EntityType::EdgeSet => Ok("edges_names".to_string()),
            EntityType::FaceSet => Ok("faces_names".to_string()),
            EntityType::NodeMap => Ok("node_map_names".to_string()),
            EntityType::EdgeMap => Ok("edge_map_names".to_string()),
            EntityType::FaceMap => Ok("face_map_names".to_string()),
            EntityType::ElemMap => Ok("elem_map_names".to_string()),
            _ => Err(ExodusError::InvalidEntityType(format!(
                "Entity type {:?} does not support naming",
                entity_type
            ))),
        }
    }

    /// Get the NetCDF dimension name for entity count
    fn entity_count_dim_name(entity_type: EntityType) -> Result<&'static str> {
        match entity_type {
            EntityType::NodeSet => Ok("num_node_sets"),
            EntityType::SideSet => Ok("num_side_sets"),
            EntityType::ElemBlock => Ok("num_el_blk"),
            EntityType::EdgeBlock => Ok("num_edge_blk"),
            EntityType::FaceBlock => Ok("num_face_blk"),
            EntityType::ElemSet => Ok("num_elem_sets"),
            EntityType::EdgeSet => Ok("num_edge_sets"),
            EntityType::FaceSet => Ok("num_face_sets"),
            EntityType::NodeMap => Ok("num_node_maps"),
            EntityType::EdgeMap => Ok("num_edge_maps"),
            EntityType::FaceMap => Ok("num_face_maps"),
            EntityType::ElemMap => Ok("num_elem_maps"),
            _ => Err(ExodusError::InvalidEntityType(format!(
                "Entity type {:?} does not support entity count",
                entity_type
            ))),
        }
    }
}

#[cfg(feature = "netcdf4")]
impl ExodusFile<mode::Write> {
    /// Set name for a single entity
    ///
    /// # Arguments
    ///
    /// * `entity_type` - Type of entity
    /// * `entity_index` - 0-based index of the entity
    /// * `name` - Name to assign (max 32 characters)
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if:
    /// - The entity type doesn't support naming
    /// - The entity index is out of bounds
    /// - The name is too long
    /// - NetCDF write fails
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use exodus_rs::*;
    /// # let mut file = ExodusFile::create_default("test.exo").unwrap();
    /// file.put_name(EntityType::ElemBlock, 0, "MaterialA")?;
    /// # Ok::<(), ExodusError>(())
    /// ```
    pub fn put_name(
        &mut self,
        entity_type: EntityType,
        entity_index: usize,
        name: impl AsRef<str>,
    ) -> Result<()> {
        let name = name.as_ref();
        const MAX_NAME_LENGTH: usize = 32;

        if name.len() > MAX_NAME_LENGTH {
            return Err(ExodusError::StringTooLong {
                max: MAX_NAME_LENGTH,
                actual: name.len(),
            });
        }

        // Read all names, update the specific one, and write back
        let mut names = match self.names_internal(entity_type) {
            Ok(names) => names,
            Err(_) => {
                // If names don't exist, create a vector with the right size
                let dim_name = Self::entity_count_dim_name(entity_type)?;
                let count = self
                    .nc_file
                    .dimension(dim_name)
                    .ok_or_else(|| ExodusError::Other(format!("Dimension {} not found", dim_name)))?
                    .len();
                vec![String::new(); count]
            }
        };

        if entity_index >= names.len() {
            return Err(ExodusError::Other(format!(
                "Entity index {} out of bounds (max {})",
                entity_index,
                names.len() - 1
            )));
        }

        names[entity_index] = name.to_string();
        self.put_names(entity_type, &names)?;

        Ok(())
    }

    /// Set names for all entities of a type
    ///
    /// # Arguments
    ///
    /// * `entity_type` - Type of entity
    /// * `names` - Array of names (max 32 characters each)
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if:
    /// - The entity type doesn't support naming
    /// - The names array length doesn't match the number of entities
    /// - Any name is too long
    /// - NetCDF write fails
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use exodus_rs::*;
    /// # let mut file = ExodusFile::create_default("test.exo").unwrap();
    /// file.put_names(EntityType::ElemBlock, &["Block1", "Block2"])?;
    /// # Ok::<(), ExodusError>(())
    /// ```
    pub fn put_names(&mut self, entity_type: EntityType, names: &[impl AsRef<str>]) -> Result<()> {
        const MAX_NAME_LENGTH: usize = 32;

        // Validate all names
        for name in names {
            let name = name.as_ref();
            if name.len() > MAX_NAME_LENGTH {
                return Err(ExodusError::StringTooLong {
                    max: MAX_NAME_LENGTH,
                    actual: name.len(),
                });
            }
        }

        let var_name = Self::names_var_name(entity_type)?;
        let dim_name = Self::entity_count_dim_name(entity_type)?;

        // Get dimension to validate names length
        let dim_size = self
            .nc_file
            .dimension(dim_name)
            .ok_or_else(|| ExodusError::Other(format!("Dimension {} not found", dim_name)))?
            .len();

        if names.len() != dim_size {
            return Err(ExodusError::InvalidArrayLength {
                expected: dim_size,
                actual: names.len(),
            });
        }

        // Create dimensions if they don't exist
        if self.nc_file.dimension("len_name").is_none() {
            self.nc_file
                .add_dimension("len_name", MAX_NAME_LENGTH + 1)
                .map_err(ExodusError::NetCdf)?;
        }

        // Create the variable if it doesn't exist
        if self.nc_file.variable(&var_name).is_none() {
            // Verify dimensions exist
            self.nc_file
                .dimension(dim_name)
                .ok_or_else(|| ExodusError::Other(format!("Dimension {} not found", dim_name)))?;
            self.nc_file
                .dimension("len_name")
                .ok_or_else(|| ExodusError::Other("Dimension len_name not found".to_string()))?;

            self.nc_file
                .add_variable::<u8>(&var_name, &[dim_name, "len_name"])
                .map_err(ExodusError::NetCdf)?;
        }

        // Convert names to fixed-length byte arrays
        let mut name_bytes = vec![0u8; dim_size * (MAX_NAME_LENGTH + 1)];
        for (i, name) in names.iter().enumerate() {
            let name = name.as_ref();
            let offset = i * (MAX_NAME_LENGTH + 1);
            let bytes = name.as_bytes();
            name_bytes[offset..offset + bytes.len()].copy_from_slice(bytes);
        }

        // Write the names
        let mut var = self
            .nc_file
            .variable_mut(&var_name)
            .ok_or_else(|| ExodusError::VariableNotDefined(var_name.clone()))?;

        var.put_values(&name_bytes, ..)
            .map_err(ExodusError::NetCdf)?;

        Ok(())
    }

    /// Internal helper to read names
    fn names_internal(&self, entity_type: EntityType) -> Result<Vec<String>> {
        let var_name = Self::names_var_name(entity_type)?;

        // Check if the variable exists
        let var = self
            .nc_file
            .variable(&var_name)
            .ok_or_else(|| ExodusError::VariableNotDefined(var_name.clone()))?;

        // Support both classic NetCDF fixed-length char arrays (NC_CHAR) and
        // NetCDF-4 NC_STRING variables for name storage.
        let dims = var.dimensions();

        // If 1D, likely NC_STRING [num_names]
        if dims.len() == 1 {
            let num_names = dims[0].len();
            let mut names = Vec::with_capacity(num_names);
            for i in 0..num_names {
                let s = var.get_string(i..i + 1).map_err(ExodusError::NetCdf)?;
                names.push(s.trim_end_matches('\0').trim().to_string());
            }
            return Ok(names);
        }

        // Otherwise, expect 2D [num_names, len_name] of NC_CHAR
        const MAX_NAME_LENGTH: usize = 32;
        let num_names = dims.first().map(|d| d.len()).unwrap_or(0);
        let len_name = dims.get(1).map(|d| d.len()).unwrap_or(MAX_NAME_LENGTH + 1);

        let mut names = Vec::with_capacity(num_names);

        // Read each name (NC_CHAR stored as i8 in older files)
        for i in 0..num_names {
            let name_chars_i8: Vec<i8> =
                var.get_values((i..i + 1, 0..len_name)).map_err(ExodusError::NetCdf)?;
            // Convert i8 bytes to u8 slice for UTF-8 decoding
            let name_bytes: Vec<u8> = name_chars_i8.iter().map(|&b| b as u8).collect();

            // Find the null terminator or end of string
            let end = name_bytes
                .iter()
                .position(|&b| b == 0)
                .unwrap_or(name_bytes.len());
            let name = String::from_utf8_lossy(&name_bytes[..end]).to_string();
            names.push(name);
        }

        Ok(names)
    }
}

#[cfg(feature = "netcdf4")]
impl ExodusFile<mode::Read> {
    /// Get name for a single entity
    ///
    /// # Arguments
    ///
    /// * `entity_type` - Type of entity
    /// * `entity_index` - 0-based index of the entity
    ///
    /// # Returns
    ///
    /// The entity name, or an error if reading fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use exodus_rs::*;
    /// # let file = ExodusFile::<mode::Read>::open("test.exo").unwrap();
    /// let name = file.name(EntityType::ElemBlock, 0)?;
    /// # Ok::<(), ExodusError>(())
    /// ```
    pub fn name(&self, entity_type: EntityType, entity_index: usize) -> Result<String> {
        let names = self.names(entity_type)?;

        names.get(entity_index).cloned().ok_or_else(|| {
            ExodusError::Other(format!(
                "Entity index {} out of bounds (max {})",
                entity_index,
                names.len() - 1
            ))
        })
    }

    /// Get all names for entity type
    ///
    /// # Arguments
    ///
    /// * `entity_type` - Type of entity
    ///
    /// # Returns
    ///
    /// Vector of entity names, or an error if reading fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use exodus_rs::*;
    /// # let file = ExodusFile::<mode::Read>::open("test.exo").unwrap();
    /// let names = file.names(EntityType::ElemBlock)?;
    /// # Ok::<(), ExodusError>(())
    /// ```
    pub fn names(&self, entity_type: EntityType) -> Result<Vec<String>> {
        let var_name = Self::names_var_name(entity_type)?;

        // Check if the variable exists
        let var = self
            .nc_file
            .variable(&var_name)
            .ok_or_else(|| ExodusError::VariableNotDefined(var_name.clone()))?;

        // Support both classic NetCDF fixed-length char arrays (NC_CHAR) and
        // NetCDF-4 NC_STRING variables for name storage.
        let dims = var.dimensions();

        // If 1D, likely NC_STRING [num_names]
        if dims.len() == 1 {
            let num_names = dims[0].len();
            let mut names = Vec::with_capacity(num_names);
            for i in 0..num_names {
                let s = var.get_string(i..i + 1).map_err(ExodusError::NetCdf)?;
                names.push(s.trim_end_matches('\0').trim().to_string());
            }
            return Ok(names);
        }

        // Otherwise, expect 2D [num_names, len_name] of NC_CHAR
        const MAX_NAME_LENGTH: usize = 32;
        let num_names = dims.first().map(|d| d.len()).unwrap_or(0);
        let len_name = dims.get(1).map(|d| d.len()).unwrap_or(MAX_NAME_LENGTH + 1);

        let mut names = Vec::with_capacity(num_names);

        // Read each name (NC_CHAR stored as i8 in older files)
        for i in 0..num_names {
            let name_chars_i8: Vec<i8> =
                var.get_values((i..i + 1, 0..len_name)).map_err(ExodusError::NetCdf)?;
            // Convert i8 bytes to u8 slice for UTF-8 decoding
            let name_bytes: Vec<u8> = name_chars_i8.iter().map(|&b| b as u8).collect();

            // Find the null terminator or end of string
            let end = name_bytes
                .iter()
                .position(|&b| b == 0)
                .unwrap_or(name_bytes.len());
            let name = String::from_utf8_lossy(&name_bytes[..end]).to_string();
            names.push(name);
        }

        Ok(names)
    }
}

// ============================================================================
// Property Arrays
// ============================================================================

#[cfg(feature = "netcdf4")]
impl ExodusFile<mode::Write> {
    /// Set property value for a single entity
    ///
    /// Properties are integer values associated with entities, commonly used for
    /// material IDs, processor IDs, or other categorizations.
    ///
    /// # Arguments
    ///
    /// * `entity_type` - Type of entity
    /// * `entity_id` - Entity ID (not index)
    /// * `prop_name` - Property name
    /// * `value` - Property value
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the operation fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use exodus_rs::*;
    /// # let mut file = ExodusFile::create_default("test.exo").unwrap();
    /// file.put_property(EntityType::ElemBlock, 100, "MATERIAL_ID", 42)?;
    /// # Ok::<(), ExodusError>(())
    /// ```
    pub fn put_property(
        &mut self,
        entity_type: EntityType,
        entity_id: i64,
        prop_name: impl AsRef<str>,
        value: i64,
    ) -> Result<()> {
        // Get the current property array
        let mut props = match self.property_array_internal(entity_type, prop_name.as_ref()) {
            Ok(props) => props,
            Err(_) => {
                // Create new property array
                let dim_name = Self::entity_count_dim_name(entity_type)?;
                let count = self
                    .nc_file
                    .dimension(dim_name)
                    .ok_or_else(|| ExodusError::Other(format!("Dimension {} not found", dim_name)))?
                    .len();
                vec![0; count]
            }
        };

        // Find the entity index from ID
        let ids = self.entity_ids_internal(entity_type)?;
        let index = ids.iter().position(|&id| id == entity_id).ok_or_else(|| {
            ExodusError::EntityNotFound {
                entity_type: entity_type.to_string(),
                id: entity_id,
            }
        })?;

        props[index] = value;
        self.put_property_array(entity_type, prop_name, &props)?;

        Ok(())
    }

    /// Set property array for all entities of a type
    ///
    /// # Arguments
    ///
    /// * `entity_type` - Type of entity
    /// * `prop_name` - Property name
    /// * `values` - Array of property values
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if the operation fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use exodus_rs::*;
    /// # let mut file = ExodusFile::create_default("test.exo").unwrap();
    /// file.put_property_array(EntityType::ElemBlock, "MATERIAL_ID", &[1, 2, 3])?;
    /// # Ok::<(), ExodusError>(())
    /// ```
    pub fn put_property_array(
        &mut self,
        entity_type: EntityType,
        prop_name: impl AsRef<str>,
        values: &[i64],
    ) -> Result<()> {
        let prop_name = prop_name.as_ref();
        let var_name = self.property_var_name(entity_type, prop_name)?;
        let dim_name = Self::entity_count_dim_name(entity_type)?;

        // Get dimension to validate values length
        let dim_size = self
            .nc_file
            .dimension(dim_name)
            .ok_or_else(|| ExodusError::Other(format!("Dimension {} not found", dim_name)))?
            .len();

        if values.len() != dim_size {
            return Err(ExodusError::InvalidArrayLength {
                expected: dim_size,
                actual: values.len(),
            });
        }

        // Create the variable if it doesn't exist
        if self.nc_file.variable(&var_name).is_none() {
            // Verify dimension exists
            self.nc_file
                .dimension(dim_name)
                .ok_or_else(|| ExodusError::Other(format!("Dimension {} not found", dim_name)))?;

            let mut var = self
                .nc_file
                .add_variable::<i64>(&var_name, &[dim_name])
                .map_err(ExodusError::NetCdf)?;

            // Set the name attribute
            var.put_attribute("name", prop_name)
                .map_err(ExodusError::NetCdf)?;
        }

        // Write the properties
        let mut var = self
            .nc_file
            .variable_mut(&var_name)
            .ok_or_else(|| ExodusError::VariableNotDefined(var_name.clone()))?;

        var.put_values(values, ..).map_err(ExodusError::NetCdf)?;

        Ok(())
    }

    /// Helper to get property variable name
    fn property_var_name(&self, entity_type: EntityType, prop_name: &str) -> Result<String> {
        let prefix = match entity_type {
            EntityType::ElemBlock => "eb_prop",
            EntityType::NodeSet => "ns_prop",
            EntityType::SideSet => "ss_prop",
            EntityType::EdgeBlock => "ed_prop",
            EntityType::FaceBlock => "fa_prop",
            EntityType::ElemSet => "els_prop",
            EntityType::EdgeSet => "edges_prop",
            EntityType::FaceSet => "faces_prop",
            _ => {
                return Err(ExodusError::InvalidEntityType(format!(
                    "Entity type {:?} does not support properties",
                    entity_type
                )))
            }
        };

        Ok(format!(
            "{}_{}",
            prefix,
            prop_name.to_lowercase().replace(' ', "_")
        ))
    }

    /// Internal helper to read property array
    fn property_array_internal(
        &self,
        entity_type: EntityType,
        prop_name: &str,
    ) -> Result<Vec<i64>> {
        let var_name = self.property_var_name(entity_type, prop_name)?;

        let var = self
            .nc_file
            .variable(&var_name)
            .ok_or_else(|| ExodusError::VariableNotDefined(var_name.clone()))?;

        let props: Vec<i64> = var.get_values(..).map_err(ExodusError::NetCdf)?;

        Ok(props)
    }

    /// Internal helper to get entity IDs
    fn entity_ids_internal(&self, entity_type: EntityType) -> Result<Vec<i64>> {
        let var_name = match entity_type {
            EntityType::ElemBlock => "eb_prop1",
            EntityType::EdgeBlock => "ed_prop1",
            EntityType::FaceBlock => "fa_prop1",
            EntityType::NodeSet => "ns_prop1",
            EntityType::SideSet => "ss_prop1",
            EntityType::ElemSet => "els_prop1",
            EntityType::EdgeSet => "es_prop1",
            EntityType::FaceSet => "fs_prop1",
            _ => {
                return Err(ExodusError::InvalidEntityType(format!(
                    "Cannot get IDs for entity type {:?}",
                    entity_type
                )))
            }
        };

        // Try to get the variable
        if let Some(var) = self.nc_file.variable(var_name) {
            let ids: Vec<i64> = var.get_values(..).map_err(ExodusError::NetCdf)?;
            // Filter out zeros and NetCDF fill values
            Ok(ids.into_iter().filter(|&id| id > 0).collect())
        } else {
            Ok(Vec::new())
        }
    }
}

#[cfg(feature = "netcdf4")]
impl ExodusFile<mode::Read> {
    /// Get property value for a single entity
    ///
    /// # Arguments
    ///
    /// * `entity_type` - Type of entity
    /// * `entity_id` - Entity ID (not index)
    /// * `prop_name` - Property name
    ///
    /// # Returns
    ///
    /// The property value, or an error if reading fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use exodus_rs::*;
    /// # let file = ExodusFile::<mode::Read>::open("test.exo").unwrap();
    /// let material_id = file.property(EntityType::ElemBlock, 100, "MATERIAL_ID")?;
    /// # Ok::<(), ExodusError>(())
    /// ```
    pub fn property(
        &self,
        entity_type: EntityType,
        entity_id: i64,
        prop_name: impl AsRef<str>,
    ) -> Result<i64> {
        let props = self.property_array(entity_type, prop_name)?;

        // Get entity IDs to find the index
        let ids = self.entity_ids_internal(entity_type)?;
        let index = ids.iter().position(|&id| id == entity_id).ok_or_else(|| {
            ExodusError::EntityNotFound {
                entity_type: entity_type.to_string(),
                id: entity_id,
            }
        })?;

        props
            .get(index)
            .copied()
            .ok_or_else(|| ExodusError::EntityNotFound {
                entity_type: entity_type.to_string(),
                id: entity_id,
            })
    }

    /// Get property array for all entities of a type
    ///
    /// # Arguments
    ///
    /// * `entity_type` - Type of entity
    /// * `prop_name` - Property name
    ///
    /// # Returns
    ///
    /// Vector of property values, or an error if reading fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use exodus_rs::*;
    /// # let file = ExodusFile::<mode::Read>::open("test.exo").unwrap();
    /// let material_ids = file.property_array(EntityType::ElemBlock, "MATERIAL_ID")?;
    /// # Ok::<(), ExodusError>(())
    /// ```
    pub fn property_array(
        &self,
        entity_type: EntityType,
        prop_name: impl AsRef<str>,
    ) -> Result<Vec<i64>> {
        let prop_name = prop_name.as_ref();
        let var_name = self.property_var_name(entity_type, prop_name)?;

        let var = self
            .nc_file
            .variable(&var_name)
            .ok_or_else(|| ExodusError::VariableNotDefined(var_name.clone()))?;

        let props: Vec<i64> = var.get_values(..).map_err(ExodusError::NetCdf)?;

        Ok(props)
    }

    /// Get all property names for an entity type
    ///
    /// # Arguments
    ///
    /// * `entity_type` - Type of entity
    ///
    /// # Returns
    ///
    /// Vector of property names, or an error if reading fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use exodus_rs::*;
    /// # let file = ExodusFile::<mode::Read>::open("test.exo").unwrap();
    /// let prop_names = file.property_names(EntityType::ElemBlock)?;
    /// # Ok::<(), ExodusError>(())
    /// ```
    pub fn property_names(&self, entity_type: EntityType) -> Result<Vec<String>> {
        let prefix = match entity_type {
            EntityType::ElemBlock => "eb_prop",
            EntityType::NodeSet => "ns_prop",
            EntityType::SideSet => "ss_prop",
            EntityType::EdgeBlock => "ed_prop",
            EntityType::FaceBlock => "fa_prop",
            EntityType::ElemSet => "els_prop",
            EntityType::EdgeSet => "edges_prop",
            EntityType::FaceSet => "faces_prop",
            _ => {
                return Err(ExodusError::InvalidEntityType(format!(
                    "Entity type {:?} does not support properties",
                    entity_type
                )))
            }
        };

        // Find all variables with the matching prefix
        let mut prop_names = Vec::new();
        for var in self.nc_file.variables() {
            if var.name().starts_with(prefix) {
                // Extract property name from variable name
                if let Some(name) = var.name().strip_prefix(&format!("{}_", prefix)) {
                    prop_names.push(name.to_string());
                }
            }
        }

        Ok(prop_names)
    }

    /// Helper to get property variable name
    fn property_var_name(&self, entity_type: EntityType, prop_name: &str) -> Result<String> {
        let prefix = match entity_type {
            EntityType::ElemBlock => "eb_prop",
            EntityType::NodeSet => "ns_prop",
            EntityType::SideSet => "ss_prop",
            EntityType::EdgeBlock => "ed_prop",
            EntityType::FaceBlock => "fa_prop",
            EntityType::ElemSet => "els_prop",
            EntityType::EdgeSet => "edges_prop",
            EntityType::FaceSet => "faces_prop",
            _ => {
                return Err(ExodusError::InvalidEntityType(format!(
                    "Entity type {:?} does not support properties",
                    entity_type
                )))
            }
        };

        Ok(format!(
            "{}_{}",
            prefix,
            prop_name.to_lowercase().replace(' ', "_")
        ))
    }

    /// Internal helper to get entity IDs
    fn entity_ids_internal(&self, entity_type: EntityType) -> Result<Vec<i64>> {
        let var_name = match entity_type {
            EntityType::ElemBlock => "eb_prop1",
            EntityType::EdgeBlock => "ed_prop1",
            EntityType::FaceBlock => "fa_prop1",
            EntityType::NodeSet => "ns_prop1",
            EntityType::SideSet => "ss_prop1",
            EntityType::ElemSet => "els_prop1",
            EntityType::EdgeSet => "es_prop1",
            EntityType::FaceSet => "fs_prop1",
            _ => {
                return Err(ExodusError::InvalidEntityType(format!(
                    "Cannot get IDs for entity type {:?}",
                    entity_type
                )))
            }
        };

        // Try to get the variable
        if let Some(var) = self.nc_file.variable(var_name) {
            let ids: Vec<i64> = var.get_values(..).map_err(ExodusError::NetCdf)?;
            // Filter out zeros and NetCDF fill values
            Ok(ids.into_iter().filter(|&id| id > 0).collect())
        } else {
            Ok(Vec::new())
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[cfg(feature = "netcdf4")]
mod tests {
    use super::*;
    use crate::types::{CreateMode, CreateOptions, InitParams};
    use tempfile::NamedTempFile;

    #[test]
    fn test_node_id_map() {
        let tmp = NamedTempFile::new().unwrap();

        // Write
        {
            let mut file = ExodusFile::create(
                tmp.path(),
                CreateOptions {
                    mode: CreateMode::Clobber,
                    ..Default::default()
                },
            )
            .unwrap();

            let params = InitParams {
                title: "ID Map Test".into(),
                num_dim: 3,
                num_nodes: 4,
                ..Default::default()
            };
            file.init(&params).unwrap();

            // Custom node numbering starting from 100
            let node_map = vec![100, 101, 102, 103];
            file.put_id_map(EntityType::NodeMap, &node_map).unwrap();
        }

        // Read
        {
            let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
            let node_map = file.id_map(EntityType::NodeMap).unwrap();
            assert_eq!(node_map, vec![100, 101, 102, 103]);
        }
    }

    #[test]
    fn test_elem_order_map() {
        let tmp = NamedTempFile::new().unwrap();

        // Write
        {
            let mut file = ExodusFile::create(
                tmp.path(),
                CreateOptions {
                    mode: CreateMode::Clobber,
                    ..Default::default()
                },
            )
            .unwrap();

            let params = InitParams {
                title: "Order Map Test".into(),
                num_dim: 3,
                num_elems: 4,
                num_elem_blocks: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            // Reverse element ordering
            let order = vec![4, 3, 2, 1];
            file.put_elem_order_map(&order).unwrap();
        }

        // Read
        {
            let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
            let order = file.elem_order_map().unwrap();
            assert_eq!(order, vec![4, 3, 2, 1]);
        }
    }

    #[test]
    fn test_block_names() {
        let tmp = NamedTempFile::new().unwrap();

        // Write
        {
            let mut file = ExodusFile::create(
                tmp.path(),
                CreateOptions {
                    mode: CreateMode::Clobber,
                    ..Default::default()
                },
            )
            .unwrap();

            let params = InitParams {
                title: "Names Test".into(),
                num_dim: 3,
                num_elem_blocks: 2,
                ..Default::default()
            };
            file.init(&params).unwrap();

            file.put_names(EntityType::ElemBlock, &["Block1", "Block2"])
                .unwrap();
        }

        // Read
        {
            let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
            let names = file.names(EntityType::ElemBlock).unwrap();
            assert_eq!(names, vec!["Block1", "Block2"]);

            let name = file.name(EntityType::ElemBlock, 0).unwrap();
            assert_eq!(name, "Block1");
        }
    }

    #[test]
    fn test_property_array() {
        let tmp = NamedTempFile::new().unwrap();

        // Write
        {
            let mut file = ExodusFile::create(
                tmp.path(),
                CreateOptions {
                    mode: CreateMode::Clobber,
                    ..Default::default()
                },
            )
            .unwrap();

            let params = InitParams {
                title: "Property Test".into(),
                num_dim: 3,
                num_elem_blocks: 3,
                ..Default::default()
            };
            file.init(&params).unwrap();

            file.put_property_array(EntityType::ElemBlock, "MATERIAL_ID", &[1, 2, 3])
                .unwrap();
        }

        // Read
        {
            let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
            let props = file
                .property_array(EntityType::ElemBlock, "MATERIAL_ID")
                .unwrap();
            assert_eq!(props, vec![1, 2, 3]);

            let prop_names = file.property_names(EntityType::ElemBlock).unwrap();
            assert!(prop_names.contains(&"material_id".to_string()));
        }
    }
}
