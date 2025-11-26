//! Variable operations
//!
//! This module contains variable definition and I/O operations for time-dependent data.
//! Implemented in Phase 6.

use crate::error::{EntityId, ExodusError, Result};
use crate::types::{EntityType, TruthTable, VarStorageMode};
use crate::{mode, ExodusFile, FileMode};

// ====================
// Common Operations
// ====================

impl<M: FileMode> ExodusFile<M> {
    /// Get variable names for an entity type
    ///
    /// # Arguments
    ///
    /// * `var_type` - Entity type (Global, Nodal, ElemBlock, etc.)
    ///
    /// # Returns
    ///
    /// Vector of variable names
    ///
    /// # Errors
    ///
    /// Returns an error if NetCDF read fails
    pub fn variable_names(&self, var_type: EntityType) -> Result<Vec<String>> {
        let var_name_var = match var_type {
            EntityType::Global => "name_glo_var",
            EntityType::Nodal => "name_nod_var",
            EntityType::ElemBlock => "name_elem_var",
            EntityType::EdgeBlock => "name_edge_var",
            EntityType::FaceBlock => "name_face_var",
            EntityType::NodeSet => "name_nset_var",
            EntityType::EdgeSet => "name_eset_var",
            EntityType::FaceSet => "name_fset_var",
            EntityType::SideSet => "name_sset_var",
            EntityType::ElemSet => "name_elset_var",
            _ => {
                return Err(ExodusError::InvalidEntityType(format!(
                    "Invalid variable type: {}",
                    var_type
                )))
            }
        };

        // Try to get the variable
        match self.nc_file.variable(var_name_var) {
            Some(var) => {
                // Support both classic NetCDF fixed-length char arrays and
                // NetCDF-4 NC_STRING variables for name storage.
                let dims = var.dimensions();

                // If 1D, likely NC_STRING [num_vars]
                if dims.len() == 1 {
                    // NC_STRING case: read each entry using get_string
                    let num_vars = dims[0].len();
                    let mut names = Vec::with_capacity(num_vars);
                    for i in 0..num_vars {
                        let s = var.get_string(i..i + 1)?;
                        names.push(s.trim_end_matches('\0').trim().to_string());
                    }
                    return Ok(names);
                }

                // Otherwise, expect 2D [num_vars, len_string] of NC_CHAR
                // CRITICAL: var.len() returns TOTAL elements (num_vars * len_string)
                // We need the first dimension size (num_vars) instead
                let num_vars = if let Some(dim) = dims.first() {
                    dim.len()
                } else {
                    return Ok(Vec::new());
                };

                if num_vars == 0 {
                    return Ok(Vec::new());
                }

                // Get len_string for fixed-length names
                let len_string = self
                    .nc_file
                    .dimension("len_string")
                    .ok_or_else(|| {
                        ExodusError::Other(
                            "len_string dimension not found when reading variable names"
                                .to_string(),
                        )
                    })?
                    .len();

                let mut names = Vec::new();
                for i in 0..num_vars {
                    // Read one name at a time with explicit dimension bounds (NC_CHAR)
                    let name_chars_i8: Vec<i8> = var.get_values((i..i + 1, 0..len_string))?;
                    // Convert i8 bytes to u8 slice for UTF-8 decoding
                    let name_bytes: Vec<u8> = name_chars_i8.iter().map(|&b| b as u8).collect();
                    // Convert to string, trimming null bytes and whitespace
                    let name = String::from_utf8_lossy(&name_bytes)
                        .trim_end_matches('\0')
                        .trim()
                        .to_string();
                    names.push(name);
                }

                Ok(names)
            }
            None => Ok(Vec::new()),
        }
    }

    /// Get number of time steps
    ///
    /// # Returns
    ///
    /// Number of time steps in the file
    pub fn num_time_steps(&self) -> Result<usize> {
        Ok(self
            .nc_file
            .dimension("time_step")
            .map(|d| d.len())
            .unwrap_or(0))
    }

    // ====================
    // Reduction Variables (Read Operations)
    // ====================

    /// Get reduction variable names for an entity type
    ///
    /// # Arguments
    ///
    /// * `var_type` - Entity type
    ///
    /// # Returns
    ///
    /// Vector of reduction variable names
    ///
    /// # Errors
    ///
    /// Returns an error if NetCDF read fails
    pub fn reduction_variable_names(&self, var_type: EntityType) -> Result<Vec<String>> {
        let var_name_var = match var_type {
            EntityType::Global => "name_glo_var",
            EntityType::ElemBlock => "name_ele_red_var",
            EntityType::EdgeBlock => "name_edg_red_var",
            EntityType::FaceBlock => "name_fac_red_var",
            EntityType::NodeSet => "name_nset_red_var",
            EntityType::EdgeSet => "name_eset_red_var",
            EntityType::FaceSet => "name_fset_red_var",
            EntityType::SideSet => "name_sset_red_var",
            EntityType::ElemSet => "name_elset_red_var",
            EntityType::Assembly => "name_assembly_red_var",
            EntityType::Blob => "name_blob_red_var",
            _ => {
                return Err(ExodusError::InvalidEntityType(format!(
                    "Invalid reduction variable type: {}",
                    var_type
                )))
            }
        };

        // Try to get the variable
        match self.nc_file.variable(var_name_var) {
            Some(var) => {
                // Support NC_STRING [num_vars] and NC_CHAR [num_vars, len_string]
                let dims = var.dimensions();

                if dims.len() == 1 {
                    let num_vars = dims[0].len();
                    let mut names = Vec::with_capacity(num_vars);
                    for i in 0..num_vars {
                        let s = var.get_string(i..i + 1)?;
                        names.push(s.trim_end_matches('\0').trim().to_string());
                    }
                    return Ok(names);
                }

                let num_vars = if let Some(dim) = dims.first() {
                    dim.len()
                } else {
                    return Ok(Vec::new());
                };

                if num_vars == 0 {
                    return Ok(Vec::new());
                }

                let len_string = self
                    .nc_file
                    .dimension("len_string")
                    .ok_or_else(|| {
                        ExodusError::Other(
                            "len_string dimension not found when reading reduction variable names"
                                .to_string(),
                        )
                    })?
                    .len();

                let mut names = Vec::new();
                for i in 0..num_vars {
                    let name_chars_i8: Vec<i8> = var.get_values((i..i + 1, 0..len_string))?;
                    let name_bytes: Vec<u8> = name_chars_i8.iter().map(|&b| b as u8).collect();
                    let name = String::from_utf8_lossy(&name_bytes)
                        .trim_end_matches('\0')
                        .trim()
                        .to_string();
                    names.push(name);
                }

                Ok(names)
            }
            None => Ok(Vec::new()),
        }
    }

    /// Read reduction variable values for a time step
    ///
    /// # Arguments
    ///
    /// * `step` - Time step index (0-based)
    /// * `var_type` - Entity type
    /// * `entity_id` - Entity ID
    ///
    /// # Returns
    ///
    /// Vector of reduction variable values
    ///
    /// # Errors
    ///
    /// Returns an error if NetCDF read fails
    pub fn get_reduction_vars(
        &self,
        step: usize,
        var_type: EntityType,
        entity_id: EntityId,
    ) -> Result<Vec<f64>> {
        let var_name = match var_type {
            EntityType::Global => "vals_glo_var".to_string(),
            EntityType::Assembly => format!("vals_assembly_red{}", entity_id),
            EntityType::Blob => format!("vals_blob_red{}", entity_id),
            EntityType::ElemBlock => {
                let block_ids = self.block_ids(var_type)?;
                let block_index = block_ids
                    .iter()
                    .position(|&id| id == entity_id)
                    .ok_or_else(|| ExodusError::EntityNotFound {
                        entity_type: var_type.to_string(),
                        id: entity_id,
                    })?;
                format!("vals_elem_red_eb{}", block_index + 1)
            }
            EntityType::EdgeBlock => {
                let block_ids = self.block_ids(var_type)?;
                let block_index = block_ids
                    .iter()
                    .position(|&id| id == entity_id)
                    .ok_or_else(|| ExodusError::EntityNotFound {
                        entity_type: var_type.to_string(),
                        id: entity_id,
                    })?;
                format!("vals_edge_red_edgb{}", block_index + 1)
            }
            EntityType::FaceBlock => {
                let block_ids = self.block_ids(var_type)?;
                let block_index = block_ids
                    .iter()
                    .position(|&id| id == entity_id)
                    .ok_or_else(|| ExodusError::EntityNotFound {
                        entity_type: var_type.to_string(),
                        id: entity_id,
                    })?;
                format!("vals_face_red_facb{}", block_index + 1)
            }
            EntityType::NodeSet => {
                let set_ids = self.set_ids(EntityType::NodeSet)?;
                let set_index =
                    set_ids
                        .iter()
                        .position(|&id| id == entity_id)
                        .ok_or_else(|| ExodusError::EntityNotFound {
                            entity_type: EntityType::NodeSet.to_string(),
                            id: entity_id,
                        })?;
                format!("vals_nset_red_ns{}", set_index + 1)
            }
            EntityType::EdgeSet => {
                let set_ids = self.set_ids(EntityType::EdgeSet)?;
                let set_index =
                    set_ids
                        .iter()
                        .position(|&id| id == entity_id)
                        .ok_or_else(|| ExodusError::EntityNotFound {
                            entity_type: EntityType::EdgeSet.to_string(),
                            id: entity_id,
                        })?;
                format!("vals_eset_red_es{}", set_index + 1)
            }
            EntityType::FaceSet => {
                let set_ids = self.set_ids(EntityType::FaceSet)?;
                let set_index =
                    set_ids
                        .iter()
                        .position(|&id| id == entity_id)
                        .ok_or_else(|| ExodusError::EntityNotFound {
                            entity_type: EntityType::FaceSet.to_string(),
                            id: entity_id,
                        })?;
                format!("vals_fset_red_fs{}", set_index + 1)
            }
            EntityType::SideSet => {
                let set_ids = self.set_ids(EntityType::SideSet)?;
                let set_index =
                    set_ids
                        .iter()
                        .position(|&id| id == entity_id)
                        .ok_or_else(|| ExodusError::EntityNotFound {
                            entity_type: EntityType::SideSet.to_string(),
                            id: entity_id,
                        })?;
                format!("vals_sset_red_ss{}", set_index + 1)
            }
            EntityType::ElemSet => {
                let set_ids = self.set_ids(EntityType::ElemSet)?;
                let set_index =
                    set_ids
                        .iter()
                        .position(|&id| id == entity_id)
                        .ok_or_else(|| ExodusError::EntityNotFound {
                            entity_type: EntityType::ElemSet.to_string(),
                            id: entity_id,
                        })?;
                format!("vals_elset_red_els{}", set_index + 1)
            }
            _ => {
                return Err(ExodusError::InvalidEntityType(format!(
                    "Unsupported reduction variable type: {}",
                    var_type
                )))
            }
        };

        if let Some(var) = self.nc_file.variable(&var_name) {
            // Get the number of reduction variables
            let num_vars = if let Some(dim) = var.dimensions().get(1) {
                dim.len()
            } else {
                return Ok(Vec::new());
            };

            let values: Vec<f64> = var.get_values((step..step + 1, 0..num_vars))?;
            Ok(values)
        } else {
            Ok(Vec::new())
        }
    }
}

// ====================
// Write Operations
// ====================

impl ExodusFile<mode::Write> {
    /// Define variables for an entity type
    ///
    /// # Arguments
    ///
    /// * `var_type` - Entity type (Global, Nodal, ElemBlock, etc.)
    /// * `names` - Variable names
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - NetCDF write fails
    /// - Invalid entity type
    pub fn define_variables(
        &mut self,
        var_type: EntityType,
        names: &[impl AsRef<str>],
    ) -> Result<()> {
        // Ensure we're in define mode for adding variables
        self.ensure_define_mode()?;

        if names.is_empty() {
            return Ok(());
        }

        let num_vars = names.len();

        // Create dimension for number of variables
        let (num_var_dim, var_name_var) = match var_type {
            EntityType::Global => ("num_glo_var", "name_glo_var"),
            EntityType::Nodal => ("num_nod_var", "name_nod_var"),
            EntityType::ElemBlock => ("num_elem_var", "name_elem_var"),
            EntityType::EdgeBlock => ("num_edge_var", "name_edge_var"),
            EntityType::FaceBlock => ("num_face_var", "name_face_var"),
            EntityType::NodeSet => ("num_nset_var", "name_nset_var"),
            EntityType::EdgeSet => ("num_eset_var", "name_eset_var"),
            EntityType::FaceSet => ("num_fset_var", "name_fset_var"),
            EntityType::SideSet => ("num_sset_var", "name_sset_var"),
            EntityType::ElemSet => ("num_elset_var", "name_elset_var"),
            _ => {
                return Err(ExodusError::InvalidEntityType(format!(
                    "Invalid variable type: {}",
                    var_type
                )))
            }
        };

        // Add dimension
        self.nc_file.add_dimension(num_var_dim, num_vars)?;

        // Add variable name storage (2D char array)
        // Standard Exodus II uses 32 characters for variable names for compatibility
        const STANDARD_NAME_LEN: usize = 32;

        // Use existing len_string if it exists, otherwise use 32
        let len_string = if let Some(dim) = self.nc_file.dimension("len_string") {
            dim.len()
        } else {
            STANDARD_NAME_LEN
        };

        // Create len_string dimension if it doesn't exist
        if self.nc_file.dimension("len_string").is_none() {
            self.nc_file.add_dimension("len_string", len_string)?;
        }

        // Create time_step dimension if it doesn't exist (needed for all variables)
        if self.nc_file.dimension("time_step").is_none() {
            self.nc_file.add_unlimited_dimension("time_step")?;
        }

        // Create the variable name storage variable
        let var_name_exists = self.nc_file.variable(var_name_var).is_some();
        if !var_name_exists {
            self.nc_file
                .add_variable::<u8>(var_name_var, &[num_var_dim, "len_string"])?;
        }

        // Get chunking configuration (raw requested values)
        let time_chunk_req = self
            .metadata
            .performance
            .as_ref()
            .map(|p| p.chunks.time_chunk_size)
            .unwrap_or(0);
        let node_chunk_req = self
            .metadata
            .performance
            .as_ref()
            .map(|p| p.chunks.node_chunk_size)
            .unwrap_or(0);

        // Get dimension sizes for clamping (chunk size can't exceed dimension)
        let num_nodes = self
            .metadata
            .dim_cache
            .get("num_nodes")
            .copied()
            .unwrap_or(0);

        // Create the actual storage variables based on type
        match var_type {
            EntityType::Global => {
                // Global vars: vals_glo_var(time_step, num_glo_var)
                if self.nc_file.variable("vals_glo_var").is_none() {
                    let mut var = self
                        .nc_file
                        .add_variable::<f64>("vals_glo_var", &["time_step", num_var_dim])?;

                    // Apply chunking for global variables (clamp to actual size)
                    // Chunk on time if configured (typically small variable count)
                    if time_chunk_req > 0 && num_vars > 0 {
                        let clamped_num_vars = num_vars.min(num_vars); // num_vars is the dim size
                        var.set_chunking(&[time_chunk_req.max(1), clamped_num_vars])?;
                    }
                }
            }
            EntityType::Nodal => {
                // Nodal vars: vals_nod_var{i}(time_step, num_nodes)
                // Clamp node_chunk to actual num_nodes
                let node_chunk = if node_chunk_req > 0 && num_nodes > 0 {
                    node_chunk_req.min(num_nodes)
                } else {
                    0
                };

                for i in 0..num_vars {
                    let var_name = format!("vals_nod_var{}", i + 1);
                    if self.nc_file.variable(&var_name).is_none() {
                        let mut var = self
                            .nc_file
                            .add_variable::<f64>(&var_name, &["time_step", "num_nodes"])?;

                        // Apply chunking for nodal variables
                        // Chunk on nodes (spatial dimension), optionally on time
                        if node_chunk > 0 {
                            let t_chunk = if time_chunk_req > 0 {
                                time_chunk_req
                            } else {
                                1
                            };
                            var.set_chunking(&[t_chunk, node_chunk])?;
                        }
                    }
                }
            }
            EntityType::ElemBlock => {
                // Element vars: vals_elem_var{var_idx}eb{block_idx}(time_step, num_elem_in_blk{block_idx})
                // We'll create these later when we know which blocks need which variables
                // For now, this is handled in put_var
            }
            _ => {
                // Other types not yet fully supported
            }
        }

        // Now write the variable names (after all dimensions and variables are created)
        // CRITICAL: Use the actual len_string dimension size, not max_name_len!
        // The buffer size must match the variable's second dimension size
        let actual_len_string = self
            .nc_file
            .dimension("len_string")
            .ok_or_else(|| {
                ExodusError::Other("len_string dimension not found after creation".to_string())
            })?
            .len();

        // CRITICAL: Must write the variable names or reading will fail!
        let mut var = self.nc_file.variable_mut(var_name_var).ok_or_else(|| {
            ExodusError::Other(format!(
                "Cannot get mutable reference to variable '{}' after creation. \
                This indicates a NetCDF define/data mode issue.",
                var_name_var
            ))
        })?;

        for (i, name) in names.iter().enumerate() {
            let name_str = name.as_ref();
            // Use actual dimension size for buffer
            let mut buf = vec![0u8; actual_len_string];
            let bytes = name_str.as_bytes();
            let copy_len = bytes.len().min(actual_len_string);
            buf[..copy_len].copy_from_slice(&bytes[..copy_len]);
            var.put_values(&buf, (i..i + 1, ..))?;
        }

        // Force sync to ensure all data is written and file is in consistent state
        // This is critical for proper NetCDF define/data mode transitions
        self.nc_file.sync()?;

        Ok(())
    }

    /// Write time value for a time step
    ///
    /// # Arguments
    ///
    /// * `step` - Time step index (0-based)
    /// * `time` - Time value
    ///
    /// # Errors
    ///
    /// Returns an error if NetCDF write fails
    pub fn put_time(&mut self, step: usize, time: f64) -> Result<()> {
        // Ensure time_step dimension exists (may be created earlier by define_variables)
        if self.nc_file.dimension("time_step").is_none() {
            self.ensure_define_mode()?;
            self.nc_file.add_unlimited_dimension("time_step")?;
        }

        // Ensure time_whole variable exists
        if self.nc_file.variable("time_whole").is_none() {
            self.ensure_define_mode()?;
            let mut var = self
                .nc_file
                .add_variable::<f64>("time_whole", &["time_step"])?;
            var.put_attribute("name", "time_whole")?;
        }

        // Ensure we're in data mode for writing time values
        self.ensure_data_mode()?;

        // Write the time value
        if let Some(mut var) = self.nc_file.variable_mut("time_whole") {
            var.put_value(time, step..step + 1)?;
        }

        Ok(())
    }

    /// Write variable values for a time step
    ///
    /// For Global variables, entity_id is ignored and should be 0.
    /// For Nodal variables, entity_id is ignored and should be 0.
    /// For block variables (element, edge, face), entity_id is the block ID.
    ///
    /// # Arguments
    ///
    /// * `step` - Time step index (0-based)
    /// * `var_type` - Entity type
    /// * `entity_id` - Entity ID (block ID for block variables, 0 for global/nodal)
    /// * `var_index` - Variable index (0-based)
    /// * `values` - Variable values
    ///
    /// # Errors
    ///
    /// Returns an error if NetCDF write fails
    pub fn put_var(
        &mut self,
        step: usize,
        var_type: EntityType,
        entity_id: EntityId,
        var_index: usize,
        values: &[f64],
    ) -> Result<()> {
        let var_name = self.get_var_name(var_type, entity_id, var_index)?;

        // Get or create the variable
        if self.nc_file.variable(&var_name).is_none() {
            // Need to be in define mode to create the variable
            self.ensure_define_mode()?;
            self.create_var_storage(var_type, entity_id, var_index)?;
        }

        // Ensure we're in data mode for writing variable values
        self.ensure_data_mode()?;

        // Write the values
        if let Some(mut var) = self.nc_file.variable_mut(&var_name) {
            match var_type {
                EntityType::Global => {
                    // Global vars: (time_step, num_glo_var)
                    // Write single value at [step, var_index]
                    if values.len() != 1 {
                        return Err(ExodusError::InvalidArrayLength {
                            expected: 1,
                            actual: values.len(),
                        });
                    }
                    var.put_value(values[0], (step..step + 1, var_index..var_index + 1))?;
                }
                EntityType::Nodal => {
                    // Nodal vars: (time_step, num_nodes)
                    var.put_values(values, (step..step + 1, ..))?;
                }
                EntityType::ElemBlock | EntityType::EdgeBlock | EntityType::FaceBlock => {
                    // Block vars: (time_step, num_entries_in_block)
                    var.put_values(values, (step..step + 1, ..))?;
                }
                EntityType::NodeSet
                | EntityType::EdgeSet
                | EntityType::FaceSet
                | EntityType::SideSet
                | EntityType::ElemSet => {
                    // Set vars: (time_step, num_entries_in_set)
                    var.put_values(values, (step..step + 1, ..))?;
                }
                _ => {
                    return Err(ExodusError::InvalidEntityType(format!(
                        "Unsupported variable type: {}",
                        var_type
                    )))
                }
            }
        }

        Ok(())
    }

    /// Set truth table (which blocks have which variables)
    ///
    /// # Arguments
    ///
    /// * `var_type` - Entity type (ElemBlock, EdgeBlock, or FaceBlock)
    /// * `table` - Truth table
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - var_type doesn't match table.var_type
    /// - table dimensions don't match actual blocks/variables
    /// - table array length is incorrect
    /// - NetCDF write fails
    pub fn put_truth_table(&mut self, var_type: EntityType, table: &TruthTable) -> Result<()> {
        let var_name = match var_type {
            EntityType::ElemBlock => "elem_var_tab",
            EntityType::EdgeBlock => "edge_var_tab",
            EntityType::FaceBlock => "face_var_tab",
            _ => {
                return Err(ExodusError::InvalidEntityType(format!(
                    "Truth tables only supported for block types, got {}",
                    var_type
                )))
            }
        };

        // Validate that table var_type matches parameter
        if table.var_type != var_type {
            return Err(ExodusError::Other(format!(
                "Truth table var_type {:?} doesn't match parameter {:?}",
                table.var_type, var_type
            )));
        }

        // Get actual counts from file
        let actual_num_blocks = self.block_ids(var_type)?.len();
        let actual_num_vars = self.variable_names(var_type)?.len();

        // Validate dimensions
        if table.num_blocks != actual_num_blocks {
            return Err(ExodusError::Other(format!(
                "Truth table num_blocks {} doesn't match actual blocks {}",
                table.num_blocks, actual_num_blocks
            )));
        }

        if table.num_vars != actual_num_vars {
            return Err(ExodusError::Other(format!(
                "Truth table num_vars {} doesn't match actual variables {}",
                table.num_vars, actual_num_vars
            )));
        }

        // Validate table array length
        let expected_len = actual_num_blocks * actual_num_vars;
        if table.table.len() != expected_len {
            return Err(ExodusError::InvalidArrayLength {
                expected: expected_len,
                actual: table.table.len(),
            });
        }

        let (num_blocks_dim, num_vars_dim) = match var_type {
            EntityType::ElemBlock => ("num_el_blk", "num_elem_var"),
            EntityType::EdgeBlock => ("num_ed_blk", "num_edge_var"),
            EntityType::FaceBlock => ("num_fa_blk", "num_face_var"),
            _ => unreachable!(),
        };

        // Create the truth table variable if it doesn't exist
        if self.nc_file.variable(var_name).is_none() {
            self.nc_file
                .add_variable::<i32>(var_name, &[num_blocks_dim, num_vars_dim])?;
        }

        // Convert bool to i32 and write
        let table_i32: Vec<i32> = table.table.iter().map(|&b| if b { 1 } else { 0 }).collect();

        if let Some(mut var) = self.nc_file.variable_mut(var_name) {
            var.put_values(&table_i32, ..)?;
        }

        Ok(())
    }

    // Helper function to create variable storage
    fn create_var_storage(
        &mut self,
        var_type: EntityType,
        entity_id: EntityId,
        var_index: usize,
    ) -> Result<()> {
        let var_name = self.get_var_name(var_type, entity_id, var_index)?;

        // Get chunking configuration (raw requested values)
        let time_chunk_req = self
            .metadata
            .performance
            .as_ref()
            .map(|p| p.chunks.time_chunk_size)
            .unwrap_or(0);
        let node_chunk_req = self
            .metadata
            .performance
            .as_ref()
            .map(|p| p.chunks.node_chunk_size)
            .unwrap_or(0);
        let elem_chunk_req = self
            .metadata
            .performance
            .as_ref()
            .map(|p| p.chunks.element_chunk_size)
            .unwrap_or(0);

        // Get dimension sizes from cache for clamping
        let num_nodes = self
            .metadata
            .dim_cache
            .get("num_nodes")
            .copied()
            .unwrap_or(0);

        // Helper to clamp chunk size to dimension
        let clamp_chunk = |req: usize, dim_size: usize| -> usize {
            if req > 0 && dim_size > 0 {
                req.min(dim_size)
            } else {
                0
            }
        };

        match var_type {
            EntityType::Global => {
                // Global vars: vals_glo_var(time_step, num_glo_var)
                if self.nc_file.variable("vals_glo_var").is_none() {
                    // Get dimension length before creating variable to avoid borrow conflict
                    let num_glo_var = self
                        .nc_file
                        .dimension("num_glo_var")
                        .map(|d| d.len())
                        .unwrap_or(1);

                    let mut var = self
                        .nc_file
                        .add_variable::<f64>("vals_glo_var", &["time_step", "num_glo_var"])?;

                    // Apply chunking for global variables (clamp to dimension size)
                    if time_chunk_req > 0 && num_glo_var > 0 {
                        var.set_chunking(&[time_chunk_req.max(1), num_glo_var])?;
                    }
                }
            }
            EntityType::Nodal => {
                // Nodal var{i}: vals_nod_var{i}(time_step, num_nodes)
                let mut var = self
                    .nc_file
                    .add_variable::<f64>(&var_name, &["time_step", "num_nodes"])?;

                // Apply chunking for nodal variables (clamp to num_nodes)
                let node_chunk = clamp_chunk(node_chunk_req, num_nodes);
                if node_chunk > 0 {
                    let t_chunk = if time_chunk_req > 0 {
                        time_chunk_req
                    } else {
                        1
                    };
                    var.set_chunking(&[t_chunk, node_chunk])?;
                }
            }
            EntityType::ElemBlock => {
                // Element var: vals_elem_var{var_idx}eb{block_idx}(time_step, num_el_in_blk{block_idx})
                let block_ids = self.block_ids(EntityType::ElemBlock)?;
                let block_index = block_ids
                    .iter()
                    .position(|&id| id == entity_id)
                    .ok_or_else(|| ExodusError::EntityNotFound {
                        entity_type: EntityType::ElemBlock.to_string(),
                        id: entity_id,
                    })?;

                let dim_name = format!("num_el_in_blk{}", block_index + 1);
                // Get block dimension size for clamping
                let block_dim_size = self
                    .nc_file
                    .dimension(&dim_name)
                    .map(|d| d.len())
                    .unwrap_or(0);

                let mut var = self
                    .nc_file
                    .add_variable::<f64>(&var_name, &["time_step", &dim_name])?;

                // Apply chunking for element variables (clamp to block size)
                let elem_chunk = clamp_chunk(elem_chunk_req, block_dim_size);
                if elem_chunk > 0 {
                    let t_chunk = if time_chunk_req > 0 {
                        time_chunk_req
                    } else {
                        1
                    };
                    var.set_chunking(&[t_chunk, elem_chunk])?;
                }
            }
            EntityType::EdgeBlock => {
                let block_ids = self.block_ids(EntityType::EdgeBlock)?;
                let block_index = block_ids
                    .iter()
                    .position(|&id| id == entity_id)
                    .ok_or_else(|| ExodusError::EntityNotFound {
                        entity_type: EntityType::EdgeBlock.to_string(),
                        id: entity_id,
                    })?;

                let dim_name = format!("num_ed_in_blk{}", block_index + 1);
                // Get block dimension size for clamping
                let block_dim_size = self
                    .nc_file
                    .dimension(&dim_name)
                    .map(|d| d.len())
                    .unwrap_or(0);

                let mut var = self
                    .nc_file
                    .add_variable::<f64>(&var_name, &["time_step", &dim_name])?;

                // Apply chunking for edge variables (clamp to block size)
                let elem_chunk = clamp_chunk(elem_chunk_req, block_dim_size);
                if elem_chunk > 0 {
                    let t_chunk = if time_chunk_req > 0 {
                        time_chunk_req
                    } else {
                        1
                    };
                    var.set_chunking(&[t_chunk, elem_chunk])?;
                }
            }
            EntityType::FaceBlock => {
                let block_ids = self.block_ids(EntityType::FaceBlock)?;
                let block_index = block_ids
                    .iter()
                    .position(|&id| id == entity_id)
                    .ok_or_else(|| ExodusError::EntityNotFound {
                        entity_type: EntityType::FaceBlock.to_string(),
                        id: entity_id,
                    })?;

                let dim_name = format!("num_fa_in_blk{}", block_index + 1);
                // Get block dimension size for clamping
                let block_dim_size = self
                    .nc_file
                    .dimension(&dim_name)
                    .map(|d| d.len())
                    .unwrap_or(0);

                let mut var = self
                    .nc_file
                    .add_variable::<f64>(&var_name, &["time_step", &dim_name])?;

                // Apply chunking for face variables (clamp to block size)
                let elem_chunk = clamp_chunk(elem_chunk_req, block_dim_size);
                if elem_chunk > 0 {
                    let t_chunk = if time_chunk_req > 0 {
                        time_chunk_req
                    } else {
                        1
                    };
                    var.set_chunking(&[t_chunk, elem_chunk])?;
                }
            }
            EntityType::NodeSet => {
                let set_ids = self.set_ids(EntityType::NodeSet)?;
                let set_index =
                    set_ids
                        .iter()
                        .position(|&id| id == entity_id)
                        .ok_or_else(|| ExodusError::EntityNotFound {
                            entity_type: EntityType::NodeSet.to_string(),
                            id: entity_id,
                        })?;

                let dim_name = format!("num_nod_ns{}", set_index + 1);
                self.nc_file
                    .add_variable::<f64>(&var_name, &["time_step", &dim_name])?;
            }
            EntityType::EdgeSet => {
                let set_ids = self.set_ids(EntityType::EdgeSet)?;
                let set_index =
                    set_ids
                        .iter()
                        .position(|&id| id == entity_id)
                        .ok_or_else(|| ExodusError::EntityNotFound {
                            entity_type: EntityType::EdgeSet.to_string(),
                            id: entity_id,
                        })?;

                let dim_name = format!("num_edge_es{}", set_index + 1);
                self.nc_file
                    .add_variable::<f64>(&var_name, &["time_step", &dim_name])?;
            }
            EntityType::FaceSet => {
                let set_ids = self.set_ids(EntityType::FaceSet)?;
                let set_index =
                    set_ids
                        .iter()
                        .position(|&id| id == entity_id)
                        .ok_or_else(|| ExodusError::EntityNotFound {
                            entity_type: EntityType::FaceSet.to_string(),
                            id: entity_id,
                        })?;

                let dim_name = format!("num_face_fs{}", set_index + 1);
                self.nc_file
                    .add_variable::<f64>(&var_name, &["time_step", &dim_name])?;
            }
            EntityType::SideSet => {
                let set_ids = self.set_ids(EntityType::SideSet)?;
                let set_index =
                    set_ids
                        .iter()
                        .position(|&id| id == entity_id)
                        .ok_or_else(|| ExodusError::EntityNotFound {
                            entity_type: EntityType::SideSet.to_string(),
                            id: entity_id,
                        })?;

                let dim_name = format!("num_side_ss{}", set_index + 1);
                self.nc_file
                    .add_variable::<f64>(&var_name, &["time_step", &dim_name])?;
            }
            EntityType::ElemSet => {
                let set_ids = self.set_ids(EntityType::ElemSet)?;
                let set_index =
                    set_ids
                        .iter()
                        .position(|&id| id == entity_id)
                        .ok_or_else(|| ExodusError::EntityNotFound {
                            entity_type: EntityType::ElemSet.to_string(),
                            id: entity_id,
                        })?;

                let dim_name = format!("num_ele_els{}", set_index + 1);
                self.nc_file
                    .add_variable::<f64>(&var_name, &["time_step", &dim_name])?;
            }
            _ => {
                return Err(ExodusError::InvalidEntityType(format!(
                    "Unsupported variable type: {}",
                    var_type
                )))
            }
        }

        Ok(())
    }

    /// Write all variables for an entity at a time step
    ///
    /// # Arguments
    ///
    /// * `step` - Time step index (0-based)
    /// * `var_type` - Entity type
    /// * `entity_id` - Entity ID (block ID for block variables, 0 for global/nodal)
    /// * `values` - Flat array of all variable values
    ///
    /// # Errors
    ///
    /// Returns an error if NetCDF write fails
    pub fn put_var_multi(
        &mut self,
        step: usize,
        var_type: EntityType,
        entity_id: EntityId,
        values: &[f64],
    ) -> Result<()> {
        let num_vars = self.variable_names(var_type)?.len();

        match var_type {
            EntityType::Global => {
                // Global vars: write all variables at once
                if values.len() != num_vars {
                    return Err(ExodusError::InvalidArrayLength {
                        expected: num_vars,
                        actual: values.len(),
                    });
                }
                for (var_idx, &value) in values.iter().enumerate() {
                    self.put_var(step, var_type, entity_id, var_idx, &[value])?;
                }
            }
            EntityType::Nodal => {
                // Nodal vars: values should be [num_nodes * num_vars]
                let num_nodes = self
                    .nc_file
                    .dimension("num_nodes")
                    .map(|d| d.len())
                    .unwrap_or(0);

                if values.len() != num_nodes * num_vars {
                    return Err(ExodusError::InvalidArrayLength {
                        expected: num_nodes * num_vars,
                        actual: values.len(),
                    });
                }

                // Split values by variable
                for var_idx in 0..num_vars {
                    let start = var_idx * num_nodes;
                    let end = start + num_nodes;
                    self.put_var(step, var_type, entity_id, var_idx, &values[start..end])?;
                }
            }
            EntityType::ElemBlock => {
                // Element vars: values should be [num_elems * num_vars]
                // Get number of elements from dimension
                let block_ids = self.block_ids(EntityType::ElemBlock)?;
                let block_index = block_ids
                    .iter()
                    .position(|&id| id == entity_id)
                    .ok_or_else(|| ExodusError::EntityNotFound {
                        entity_type: EntityType::ElemBlock.to_string(),
                        id: entity_id,
                    })?;

                let dim_name = format!("num_el_in_blk{}", block_index + 1);
                let num_elems = self
                    .nc_file
                    .dimension(&dim_name)
                    .map(|d| d.len())
                    .ok_or_else(|| {
                        ExodusError::Other(format!("Block dimension {} not found", dim_name))
                    })?;

                if values.len() != num_elems * num_vars {
                    return Err(ExodusError::InvalidArrayLength {
                        expected: num_elems * num_vars,
                        actual: values.len(),
                    });
                }

                // Split values by variable
                for var_idx in 0..num_vars {
                    let start = var_idx * num_elems;
                    let end = start + num_elems;
                    self.put_var(step, var_type, entity_id, var_idx, &values[start..end])?;
                }
            }
            _ => {
                return Err(ExodusError::InvalidEntityType(format!(
                    "Unsupported variable type: {}",
                    var_type
                )))
            }
        }

        Ok(())
    }

    /// Write variable across multiple time steps (time series)
    ///
    /// # Arguments
    ///
    /// * `start_step` - Starting time step index (0-based)
    /// * `end_step` - Ending time step index (exclusive)
    /// * `var_type` - Entity type
    /// * `entity_id` - Entity ID (block ID for block variables, 0 for global/nodal)
    /// * `var_index` - Variable index (0-based)
    /// * `values` - Variable values for all time steps
    ///
    /// # Errors
    ///
    /// Returns an error if NetCDF write fails
    pub fn put_var_time_series(
        &mut self,
        start_step: usize,
        end_step: usize,
        var_type: EntityType,
        entity_id: EntityId,
        var_index: usize,
        values: &[f64],
    ) -> Result<()> {
        let num_steps = end_step - start_step;
        let var_name = self.get_var_name(var_type, entity_id, var_index)?;

        // Get or create the variable
        if self.nc_file.variable(&var_name).is_none() {
            self.create_var_storage(var_type, entity_id, var_index)?;
        }

        // Get all needed information BEFORE borrowing variable mutably
        let expected_len = match var_type {
            EntityType::Global => num_steps,
            EntityType::Nodal => {
                let num_nodes = self
                    .nc_file
                    .dimension("num_nodes")
                    .map(|d| d.len())
                    .unwrap_or(0);
                num_steps * num_nodes
            }
            EntityType::ElemBlock | EntityType::EdgeBlock | EntityType::FaceBlock => {
                let block_ids = self.block_ids(var_type)?;
                let block_index = block_ids
                    .iter()
                    .position(|&id| id == entity_id)
                    .ok_or_else(|| ExodusError::EntityNotFound {
                        entity_type: var_type.to_string(),
                        id: entity_id,
                    })?;

                let dim_name = match var_type {
                    EntityType::ElemBlock => format!("num_el_in_blk{}", block_index + 1),
                    EntityType::EdgeBlock => format!("num_ed_in_blk{}", block_index + 1),
                    EntityType::FaceBlock => format!("num_fa_in_blk{}", block_index + 1),
                    _ => unreachable!(),
                };

                let num_entries = self
                    .nc_file
                    .dimension(&dim_name)
                    .map(|d| d.len())
                    .ok_or_else(|| {
                        ExodusError::Other(format!("Block dimension {} not found", dim_name))
                    })?;

                num_steps * num_entries
            }
            _ => {
                return Err(ExodusError::InvalidEntityType(format!(
                    "Unsupported variable type: {}",
                    var_type
                )))
            }
        };

        // Validate array length
        if values.len() != expected_len {
            return Err(ExodusError::InvalidArrayLength {
                expected: expected_len,
                actual: values.len(),
            });
        }

        // Now write the values
        if let Some(mut var) = self.nc_file.variable_mut(&var_name) {
            match var_type {
                EntityType::Global => {
                    // Global vars: (time_step, num_glo_var)
                    // Write time series at [start_step:end_step, var_index]
                    for (i, &value) in values.iter().enumerate() {
                        let step = start_step + i;
                        var.put_value(value, (step..step + 1, var_index..var_index + 1))?;
                    }
                }
                EntityType::Nodal
                | EntityType::ElemBlock
                | EntityType::EdgeBlock
                | EntityType::FaceBlock
                | EntityType::NodeSet
                | EntityType::EdgeSet
                | EntityType::FaceSet
                | EntityType::SideSet
                | EntityType::ElemSet => {
                    // All other types: (time_step, num_entities)
                    var.put_values(values, (start_step..end_step, ..))?;
                }
                _ => unreachable!(),
            }
        }

        Ok(())
    }

    // Helper function to get variable name
    fn get_var_name(
        &self,
        var_type: EntityType,
        entity_id: EntityId,
        var_index: usize,
    ) -> Result<String> {
        Ok(match var_type {
            EntityType::Global => "vals_glo_var".to_string(),
            EntityType::Nodal => format!("vals_nod_var{}", var_index + 1),
            EntityType::ElemBlock => {
                let block_ids = self.block_ids(EntityType::ElemBlock)?;
                let block_index = block_ids
                    .iter()
                    .position(|&id| id == entity_id)
                    .ok_or_else(|| ExodusError::EntityNotFound {
                        entity_type: EntityType::ElemBlock.to_string(),
                        id: entity_id,
                    })?;
                format!("vals_elem_var{}eb{}", var_index + 1, block_index + 1)
            }
            EntityType::EdgeBlock => {
                let block_ids = self.block_ids(EntityType::EdgeBlock)?;
                let block_index = block_ids
                    .iter()
                    .position(|&id| id == entity_id)
                    .ok_or_else(|| ExodusError::EntityNotFound {
                        entity_type: EntityType::EdgeBlock.to_string(),
                        id: entity_id,
                    })?;
                format!("vals_edge_var{}edb{}", var_index + 1, block_index + 1)
            }
            EntityType::FaceBlock => {
                let block_ids = self.block_ids(EntityType::FaceBlock)?;
                let block_index = block_ids
                    .iter()
                    .position(|&id| id == entity_id)
                    .ok_or_else(|| ExodusError::EntityNotFound {
                        entity_type: EntityType::FaceBlock.to_string(),
                        id: entity_id,
                    })?;
                format!("vals_face_var{}fab{}", var_index + 1, block_index + 1)
            }
            EntityType::NodeSet => {
                let set_ids = self.set_ids(EntityType::NodeSet)?;
                let set_index =
                    set_ids
                        .iter()
                        .position(|&id| id == entity_id)
                        .ok_or_else(|| ExodusError::EntityNotFound {
                            entity_type: EntityType::NodeSet.to_string(),
                            id: entity_id,
                        })?;
                format!("vals_nset_var{}ns{}", var_index + 1, set_index + 1)
            }
            EntityType::EdgeSet => {
                let set_ids = self.set_ids(EntityType::EdgeSet)?;
                let set_index =
                    set_ids
                        .iter()
                        .position(|&id| id == entity_id)
                        .ok_or_else(|| ExodusError::EntityNotFound {
                            entity_type: EntityType::EdgeSet.to_string(),
                            id: entity_id,
                        })?;
                format!("vals_eset_var{}es{}", var_index + 1, set_index + 1)
            }
            EntityType::FaceSet => {
                let set_ids = self.set_ids(EntityType::FaceSet)?;
                let set_index =
                    set_ids
                        .iter()
                        .position(|&id| id == entity_id)
                        .ok_or_else(|| ExodusError::EntityNotFound {
                            entity_type: EntityType::FaceSet.to_string(),
                            id: entity_id,
                        })?;
                format!("vals_fset_var{}fs{}", var_index + 1, set_index + 1)
            }
            EntityType::SideSet => {
                let set_ids = self.set_ids(EntityType::SideSet)?;
                let set_index =
                    set_ids
                        .iter()
                        .position(|&id| id == entity_id)
                        .ok_or_else(|| ExodusError::EntityNotFound {
                            entity_type: EntityType::SideSet.to_string(),
                            id: entity_id,
                        })?;
                format!("vals_sset_var{}ss{}", var_index + 1, set_index + 1)
            }
            EntityType::ElemSet => {
                let set_ids = self.set_ids(EntityType::ElemSet)?;
                let set_index =
                    set_ids
                        .iter()
                        .position(|&id| id == entity_id)
                        .ok_or_else(|| ExodusError::EntityNotFound {
                            entity_type: EntityType::ElemSet.to_string(),
                            id: entity_id,
                        })?;
                format!("vals_elset_var{}els{}", var_index + 1, set_index + 1)
            }
            _ => {
                return Err(ExodusError::InvalidEntityType(format!(
                    "Unsupported variable type: {}",
                    var_type
                )))
            }
        })
    }

    // ====================
    // Reduction Variables (Write Operations)
    // ====================

    /// Define reduction variables for an entity type
    ///
    /// Reduction variables store aggregated/summary values for entire objects
    /// (e.g., assemblies, blocks, sets) rather than for individual entities within those objects.
    /// For example, you might store total momentum or kinetic energy for an entire assembly.
    ///
    /// # Arguments
    ///
    /// * `var_type` - Entity type (ElemBlock, NodeSet, SideSet, ElemSet, Assembly, Blob, etc.)
    /// * `names` - Variable names
    ///
    /// # Errors
    ///
    /// Returns an error if NetCDF operation fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use exodus_rs::*;
    /// # use exodus_rs::types::*;
    /// # let mut file = ExodusFile::create("test.exo", CreateOptions::default()).unwrap();
    /// // Define reduction variables for assemblies
    /// file.define_reduction_variables(
    ///     EntityType::Assembly,
    ///     &["Momentum_X", "Momentum_Y", "Kinetic_Energy"]
    /// ).unwrap();
    /// ```
    pub fn define_reduction_variables(
        &mut self,
        var_type: EntityType,
        names: &[impl AsRef<str>],
    ) -> Result<()> {
        // Ensure we're in define mode
        self.ensure_define_mode()?;

        if names.is_empty() {
            return Ok(());
        }

        let num_vars = names.len();

        // Get the dimension and variable names based on entity type
        let (num_var_dim, var_name_var) = match var_type {
            EntityType::Global => ("num_glo_var", "name_glo_var"),
            EntityType::ElemBlock => ("num_ele_red_var", "name_ele_red_var"),
            EntityType::EdgeBlock => ("num_edg_red_var", "name_edg_red_var"),
            EntityType::FaceBlock => ("num_fac_red_var", "name_fac_red_var"),
            EntityType::NodeSet => ("num_nset_red_var", "name_nset_red_var"),
            EntityType::EdgeSet => ("num_eset_red_var", "name_eset_red_var"),
            EntityType::FaceSet => ("num_fset_red_var", "name_fset_red_var"),
            EntityType::SideSet => ("num_sset_red_var", "name_sset_red_var"),
            EntityType::ElemSet => ("num_elset_red_var", "name_elset_red_var"),
            EntityType::Assembly => ("num_assembly_red_var", "name_assembly_red_var"),
            EntityType::Blob => ("num_blob_red_var", "name_blob_red_var"),
            _ => {
                return Err(ExodusError::InvalidEntityType(format!(
                    "Invalid reduction variable type: {}",
                    var_type
                )))
            }
        };

        // Add dimension for number of reduction variables
        self.nc_file.add_dimension(num_var_dim, num_vars)?;

        // Ensure len_string dimension exists
        const STANDARD_NAME_LEN: usize = 32;
        let len_string = if let Some(dim) = self.nc_file.dimension("len_string") {
            dim.len()
        } else {
            STANDARD_NAME_LEN
        };

        if self.nc_file.dimension("len_string").is_none() {
            self.nc_file.add_dimension("len_string", len_string)?;
        }

        // Ensure time_step dimension exists
        if self.nc_file.dimension("time_step").is_none() {
            self.nc_file.add_unlimited_dimension("time_step")?;
        }

        // Create the variable name storage variable
        if self.nc_file.variable(var_name_var).is_none() {
            self.nc_file
                .add_variable::<u8>(var_name_var, &[num_var_dim, "len_string"])?;
        }

        // For global reduction variables, create the storage variable now
        // For other types, storage is created per-entity in put_reduction_vars()
        if var_type == EntityType::Global && self.nc_file.variable("vals_glo_var").is_none() {
            self.nc_file
                .add_variable::<f64>("vals_glo_var", &["time_step", num_var_dim])?;
        }

        // Write the variable names
        let actual_len_string = self
            .nc_file
            .dimension("len_string")
            .ok_or_else(|| {
                ExodusError::Other("len_string dimension not found after creation".to_string())
            })?
            .len();

        let mut var = self.nc_file.variable_mut(var_name_var).ok_or_else(|| {
            ExodusError::Other(format!(
                "Cannot get mutable reference to variable '{}' after creation",
                var_name_var
            ))
        })?;

        for (i, name) in names.iter().enumerate() {
            let name_str = name.as_ref();
            let mut buf = vec![0u8; actual_len_string];
            let bytes = name_str.as_bytes();
            let copy_len = bytes.len().min(actual_len_string);
            buf[..copy_len].copy_from_slice(&bytes[..copy_len]);
            var.put_values(&buf, (i..i + 1, ..))?;
        }

        // Force sync to ensure all data is written
        self.nc_file.sync()?;

        Ok(())
    }

    /// Write reduction variable values for a time step
    ///
    /// Reduction variables store aggregate values for entire objects (assemblies, blocks, sets)
    /// rather than for individual entities within those objects.
    ///
    /// # Arguments
    ///
    /// * `step` - Time step index (0-based)
    /// * `var_type` - Entity type
    /// * `entity_id` - Entity ID (e.g., assembly ID, block ID, set ID)
    /// * `values` - Variable values (one per reduction variable)
    ///
    /// # Errors
    ///
    /// Returns an error if NetCDF write fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use exodus_rs::*;
    /// # use exodus_rs::types::*;
    /// # let mut file = ExodusFile::create("test.exo", CreateOptions::default()).unwrap();
    /// // Write reduction variables for assembly 100 at time step 0
    /// file.put_reduction_vars(
    ///     0,
    ///     EntityType::Assembly,
    ///     100,
    ///     &[1.5, 2.3, 45.6] // Momentum_X, Momentum_Y, Kinetic_Energy
    /// ).unwrap();
    /// ```
    pub fn put_reduction_vars(
        &mut self,
        step: usize,
        var_type: EntityType,
        entity_id: EntityId,
        values: &[f64],
    ) -> Result<()> {
        // Get the reduction variable name for this entity
        let var_name = self.get_reduction_var_name(var_type, entity_id)?;

        // Get or create the variable
        if self.nc_file.variable(&var_name).is_none() {
            self.ensure_define_mode()?;
            self.create_reduction_var_storage(var_type, entity_id)?;
        }

        // Ensure we're in data mode for writing
        self.ensure_data_mode()?;

        // Write the values
        if let Some(mut var) = self.nc_file.variable_mut(&var_name) {
            var.put_values(values, (step..step + 1, ..))?;
        }

        Ok(())
    }

    /// Helper: Get reduction variable name for storage
    fn get_reduction_var_name(&self, var_type: EntityType, entity_id: EntityId) -> Result<String> {
        Ok(match var_type {
            EntityType::Global => "vals_glo_var".to_string(),
            EntityType::Assembly => {
                // For assemblies, use entity_id directly as the index
                format!("vals_assembly_red{}", entity_id)
            }
            EntityType::Blob => {
                // For blobs, use entity_id directly as the index
                format!("vals_blob_red{}", entity_id)
            }
            EntityType::ElemBlock => {
                // Find the index of this block
                let block_ids = self.block_ids(var_type)?;
                let block_index = block_ids
                    .iter()
                    .position(|&id| id == entity_id)
                    .ok_or_else(|| ExodusError::EntityNotFound {
                        entity_type: var_type.to_string(),
                        id: entity_id,
                    })?;
                format!("vals_elem_red_eb{}", block_index + 1)
            }
            EntityType::EdgeBlock => {
                let block_ids = self.block_ids(var_type)?;
                let block_index = block_ids
                    .iter()
                    .position(|&id| id == entity_id)
                    .ok_or_else(|| ExodusError::EntityNotFound {
                        entity_type: var_type.to_string(),
                        id: entity_id,
                    })?;
                format!("vals_edge_red_edgb{}", block_index + 1)
            }
            EntityType::FaceBlock => {
                let block_ids = self.block_ids(var_type)?;
                let block_index = block_ids
                    .iter()
                    .position(|&id| id == entity_id)
                    .ok_or_else(|| ExodusError::EntityNotFound {
                        entity_type: var_type.to_string(),
                        id: entity_id,
                    })?;
                format!("vals_face_red_facb{}", block_index + 1)
            }
            EntityType::NodeSet => {
                let set_ids = self.set_ids(EntityType::NodeSet)?;
                let set_index =
                    set_ids
                        .iter()
                        .position(|&id| id == entity_id)
                        .ok_or_else(|| ExodusError::EntityNotFound {
                            entity_type: EntityType::NodeSet.to_string(),
                            id: entity_id,
                        })?;
                format!("vals_nset_red_ns{}", set_index + 1)
            }
            EntityType::EdgeSet => {
                let set_ids = self.set_ids(EntityType::EdgeSet)?;
                let set_index =
                    set_ids
                        .iter()
                        .position(|&id| id == entity_id)
                        .ok_or_else(|| ExodusError::EntityNotFound {
                            entity_type: EntityType::EdgeSet.to_string(),
                            id: entity_id,
                        })?;
                format!("vals_eset_red_es{}", set_index + 1)
            }
            EntityType::FaceSet => {
                let set_ids = self.set_ids(EntityType::FaceSet)?;
                let set_index =
                    set_ids
                        .iter()
                        .position(|&id| id == entity_id)
                        .ok_or_else(|| ExodusError::EntityNotFound {
                            entity_type: EntityType::FaceSet.to_string(),
                            id: entity_id,
                        })?;
                format!("vals_fset_red_fs{}", set_index + 1)
            }
            EntityType::SideSet => {
                let set_ids = self.set_ids(EntityType::SideSet)?;
                let set_index =
                    set_ids
                        .iter()
                        .position(|&id| id == entity_id)
                        .ok_or_else(|| ExodusError::EntityNotFound {
                            entity_type: EntityType::SideSet.to_string(),
                            id: entity_id,
                        })?;
                format!("vals_sset_red_ss{}", set_index + 1)
            }
            EntityType::ElemSet => {
                let set_ids = self.set_ids(EntityType::ElemSet)?;
                let set_index =
                    set_ids
                        .iter()
                        .position(|&id| id == entity_id)
                        .ok_or_else(|| ExodusError::EntityNotFound {
                            entity_type: EntityType::ElemSet.to_string(),
                            id: entity_id,
                        })?;
                format!("vals_elset_red_els{}", set_index + 1)
            }
            _ => {
                return Err(ExodusError::InvalidEntityType(format!(
                    "Unsupported reduction variable type: {}",
                    var_type
                )))
            }
        })
    }

    /// Helper: Create storage variable for reduction variables
    fn create_reduction_var_storage(
        &mut self,
        var_type: EntityType,
        entity_id: EntityId,
    ) -> Result<()> {
        let var_name = self.get_reduction_var_name(var_type, entity_id)?;

        // Get the dimension name for the number of reduction variables
        let num_var_dim = match var_type {
            EntityType::Global => "num_glo_var",
            EntityType::ElemBlock => "num_ele_red_var",
            EntityType::EdgeBlock => "num_edg_red_var",
            EntityType::FaceBlock => "num_fac_red_var",
            EntityType::NodeSet => "num_nset_red_var",
            EntityType::EdgeSet => "num_eset_red_var",
            EntityType::FaceSet => "num_fset_red_var",
            EntityType::SideSet => "num_sset_red_var",
            EntityType::ElemSet => "num_elset_red_var",
            EntityType::Assembly => "num_assembly_red_var",
            EntityType::Blob => "num_blob_red_var",
            _ => {
                return Err(ExodusError::InvalidEntityType(format!(
                    "Unsupported reduction variable type: {}",
                    var_type
                )))
            }
        };

        // Create the storage variable: var_name(time_step, num_*_red_var)
        if self.nc_file.variable(&var_name).is_none() {
            self.nc_file
                .add_variable::<f64>(&var_name, &["time_step", num_var_dim])?;
        }

        Ok(())
    }
}

// ====================
// Read Operations
// ====================

impl ExodusFile<mode::Read> {
    /// Get all time values
    ///
    /// # Returns
    ///
    /// Vector of time values
    ///
    /// # Errors
    ///
    /// Returns an error if NetCDF read fails
    pub fn times(&self) -> Result<Vec<f64>> {
        match self.nc_file.variable("time_whole") {
            Some(var) => Ok(var.get_values(..)?),
            None => Ok(Vec::new()),
        }
    }

    /// Get time value for a step
    ///
    /// # Arguments
    ///
    /// * `step` - Time step index (0-based)
    ///
    /// # Returns
    ///
    /// Time value
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Time step is out of range
    /// - NetCDF read fails
    pub fn time(&self, step: usize) -> Result<f64> {
        let times = self.times()?;
        times
            .get(step)
            .copied()
            .ok_or(ExodusError::InvalidTimeStep(step))
    }

    /// Read variable values at a time step
    ///
    /// This method automatically handles both storage formats:
    /// - **Separate format**: Individual variables (e.g., `vals_nod_var1`, `vals_nod_var2`)
    /// - **Combined format**: Single 3D array (e.g., `vals_nod_var(time_step, num_vars, num_nodes)`)
    ///
    /// The storage format is detected automatically when the file is opened.
    ///
    /// # Arguments
    ///
    /// * `step` - Time step index (0-based)
    /// * `var_type` - Entity type
    /// * `entity_id` - Entity ID (block ID for block variables, 0 for global/nodal)
    /// * `var_index` - Variable index (0-based)
    ///
    /// # Returns
    ///
    /// Variable values
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Variable not found
    /// - NetCDF read fails
    pub fn var(
        &self,
        step: usize,
        var_type: EntityType,
        entity_id: EntityId,
        var_index: usize,
    ) -> Result<Vec<f64>> {
        // Get the storage mode for this entity type
        let storage_mode = match var_type {
            EntityType::Global => self.metadata.storage_format.global,
            EntityType::Nodal => self.metadata.storage_format.nodal,
            EntityType::ElemBlock => self.metadata.storage_format.elem_block,
            EntityType::EdgeBlock => self.metadata.storage_format.edge_block,
            EntityType::FaceBlock => self.metadata.storage_format.face_block,
            EntityType::NodeSet => self.metadata.storage_format.node_set,
            EntityType::EdgeSet => self.metadata.storage_format.edge_set,
            EntityType::FaceSet => self.metadata.storage_format.face_set,
            EntityType::SideSet => self.metadata.storage_format.side_set,
            EntityType::ElemSet => self.metadata.storage_format.elem_set,
            _ => {
                return Err(ExodusError::InvalidEntityType(format!(
                    "Unsupported variable type: {}",
                    var_type
                )))
            }
        };

        match storage_mode {
            VarStorageMode::Combined => {
                self.read_var_combined(step, var_type, entity_id, var_index)
            }
            VarStorageMode::Separate | VarStorageMode::None => {
                self.read_var_separate(step, var_type, entity_id, var_index)
            }
        }
    }

    /// Read variable from separate format (vals_nod_var1, vals_nod_var2, etc.)
    fn read_var_separate(
        &self,
        step: usize,
        var_type: EntityType,
        entity_id: EntityId,
        var_index: usize,
    ) -> Result<Vec<f64>> {
        let var_name = self.get_var_name_read(var_type, entity_id, var_index)?;

        let var = self
            .nc_file
            .variable(&var_name)
            .ok_or_else(|| ExodusError::VariableNotDefined(var_name.clone()))?;

        match var_type {
            EntityType::Global => {
                // Global vars: (time_step, num_glo_var)
                let value: f64 = var.get_value((step, var_index))?;
                Ok(vec![value])
            }
            EntityType::Nodal => {
                // Nodal vars: (time_step, num_nodes)
                Ok(var.get_values((step..step + 1, ..))?)
            }
            EntityType::ElemBlock | EntityType::EdgeBlock | EntityType::FaceBlock => {
                // Block vars: (time_step, num_entries_in_block)
                Ok(var.get_values((step..step + 1, ..))?)
            }
            EntityType::NodeSet
            | EntityType::EdgeSet
            | EntityType::FaceSet
            | EntityType::SideSet
            | EntityType::ElemSet => {
                // Set vars: (time_step, num_entries_in_set)
                Ok(var.get_values((step..step + 1, ..))?)
            }
            _ => Err(ExodusError::InvalidEntityType(format!(
                "Unsupported variable type: {}",
                var_type
            ))),
        }
    }

    /// Read variable from combined 3D format (vals_nod_var, vals_elem_var, etc.)
    ///
    /// Combined format stores all variables in a single 3D array:
    /// - `vals_nod_var(time_step, num_nod_var, num_nodes)` for nodal vars
    /// - `vals_elem_var(time_step, num_elem_var, num_elem)` for element vars
    fn read_var_combined(
        &self,
        step: usize,
        var_type: EntityType,
        _entity_id: EntityId,
        var_index: usize,
    ) -> Result<Vec<f64>> {
        let var_name = match var_type {
            EntityType::Global => "vals_glo_var",
            EntityType::Nodal => "vals_nod_var",
            EntityType::ElemBlock => "vals_elem_var",
            EntityType::EdgeBlock => "vals_edge_var",
            EntityType::FaceBlock => "vals_face_var",
            EntityType::NodeSet => "vals_nset_var",
            EntityType::EdgeSet => "vals_eset_var",
            EntityType::FaceSet => "vals_fset_var",
            EntityType::SideSet => "vals_sset_var",
            EntityType::ElemSet => "vals_elset_var",
            _ => {
                return Err(ExodusError::InvalidEntityType(format!(
                    "Unsupported variable type: {}",
                    var_type
                )))
            }
        };

        let var = self
            .nc_file
            .variable(var_name)
            .ok_or_else(|| ExodusError::VariableNotDefined(var_name.to_string()))?;

        match var_type {
            EntityType::Global => {
                // Global vars in combined format: (time_step, num_glo_var)
                let value: f64 = var.get_value((step, var_index))?;
                Ok(vec![value])
            }
            _ => {
                // Other vars in combined format: (time_step, num_vars, num_entities)
                // Get the number of entities from the last dimension
                let dims = var.dimensions();
                if dims.len() != 3 {
                    return Err(ExodusError::Other(format!(
                        "Expected 3D array for combined variable {}, got {} dimensions",
                        var_name,
                        dims.len()
                    )));
                }
                let num_entities = dims[2].len();

                // Read the slice [step, var_index, 0:num_entities]
                // Try reading as f64 first, fall back to f32 if that fails
                let result: std::result::Result<Vec<f64>, _> =
                    var.get_values((step..step + 1, var_index..var_index + 1, 0..num_entities));

                match result {
                    Ok(values) => Ok(values),
                    Err(_) => {
                        // Try reading as f32 and converting to f64
                        let values_f32: Vec<f32> = var.get_values((
                            step..step + 1,
                            var_index..var_index + 1,
                            0..num_entities,
                        ))?;
                        Ok(values_f32.into_iter().map(|v| v as f64).collect())
                    }
                }
            }
        }
    }

    /// Get truth table
    ///
    /// # Arguments
    ///
    /// * `var_type` - Entity type (ElemBlock, EdgeBlock, or FaceBlock)
    ///
    /// # Returns
    ///
    /// Truth table
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - var_type is not a block type
    /// - truth table size doesn't match expected dimensions
    /// - NetCDF read fails
    pub fn truth_table(&self, var_type: EntityType) -> Result<TruthTable> {
        let var_name = match var_type {
            EntityType::ElemBlock => "elem_var_tab",
            EntityType::EdgeBlock => "edge_var_tab",
            EntityType::FaceBlock => "face_var_tab",
            _ => {
                return Err(ExodusError::InvalidEntityType(format!(
                    "Truth tables only supported for block types, got {}",
                    var_type
                )))
            }
        };

        // Get number of blocks and variables
        let num_blocks = self.block_ids(var_type)?.len();
        let num_vars = self.variable_names(var_type)?.len();
        let expected_len = num_blocks * num_vars;

        // Read truth table if it exists
        let table_values = if let Some(var) = self.nc_file.variable(var_name) {
            let table_i32: Vec<i32> = var.get_values(..)?;

            // Validate size
            if table_i32.len() != expected_len {
                return Err(ExodusError::InvalidArrayLength {
                    expected: expected_len,
                    actual: table_i32.len(),
                });
            }

            table_i32.iter().map(|&v| v != 0).collect()
        } else {
            // Default: all true
            vec![true; expected_len]
        };

        Ok(TruthTable {
            var_type,
            num_vars,
            num_blocks,
            table: table_values,
        })
    }

    /// Check if a variable is enabled in the truth table for a given block
    ///
    /// This is a helper method to check whether a specific variable is defined
    /// for a specific block according to the truth table. This is useful for
    /// sparse variable storage where not all variables are defined on all blocks.
    ///
    /// # Arguments
    ///
    /// * `var_type` - Entity type (must be a block type)
    /// * `block_id` - Block ID
    /// * `var_index` - Variable index (0-based)
    ///
    /// # Returns
    ///
    /// `true` if the variable is enabled for the block, `false` otherwise.
    /// If no truth table exists, returns `true` (all variables enabled by default).
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - var_type is not a block type
    /// - block_id is not found
    /// - var_index is out of range
    /// - truth table read fails
    pub fn is_var_in_truth_table(
        &self,
        var_type: EntityType,
        block_id: EntityId,
        var_index: usize,
    ) -> Result<bool> {
        // Only supported for block types
        if !matches!(
            var_type,
            EntityType::ElemBlock | EntityType::EdgeBlock | EntityType::FaceBlock
        ) {
            return Err(ExodusError::InvalidEntityType(format!(
                "Truth tables only supported for block types, got {}",
                var_type
            )));
        }

        // Get the block index from the block ID
        let block_ids = self.block_ids(var_type)?;
        let block_index = block_ids
            .iter()
            .position(|&id| id == block_id)
            .ok_or_else(|| ExodusError::EntityNotFound {
                entity_type: var_type.to_string(),
                id: block_id,
            })?;

        // Get the truth table
        let truth_table = self.truth_table(var_type)?;

        // Validate var_index
        if var_index >= truth_table.num_vars {
            return Err(ExodusError::Other(format!(
                "Variable index {} out of range (max {})",
                var_index,
                truth_table.num_vars - 1
            )));
        }

        // Truth table is stored as [block0_var0, block0_var1, ..., block1_var0, block1_var1, ...]
        let table_index = block_index * truth_table.num_vars + var_index;
        Ok(truth_table.table[table_index])
    }

    /// Read all variables for an entity at a time step
    ///
    /// # Arguments
    ///
    /// * `step` - Time step index (0-based)
    /// * `var_type` - Entity type
    /// * `entity_id` - Entity ID (block ID for block variables, 0 for global/nodal)
    ///
    /// # Returns
    ///
    /// Flat array of all variable values
    ///
    /// # Errors
    ///
    /// Returns an error if NetCDF read fails
    pub fn var_multi(
        &self,
        step: usize,
        var_type: EntityType,
        entity_id: EntityId,
    ) -> Result<Vec<f64>> {
        let num_vars = self.variable_names(var_type)?.len();
        let mut all_values = Vec::new();

        match var_type {
            EntityType::Global => {
                // Global vars: read all variables at once
                for var_idx in 0..num_vars {
                    let values = self.var(step, var_type, entity_id, var_idx)?;
                    all_values.extend_from_slice(&values);
                }
            }
            EntityType::Nodal => {
                // Nodal vars: concatenate all variable values
                for var_idx in 0..num_vars {
                    let values = self.var(step, var_type, entity_id, var_idx)?;
                    all_values.extend_from_slice(&values);
                }
            }
            EntityType::ElemBlock => {
                // Element vars: concatenate all variable values
                for var_idx in 0..num_vars {
                    let values = self.var(step, var_type, entity_id, var_idx)?;
                    all_values.extend_from_slice(&values);
                }
            }
            _ => {
                return Err(ExodusError::InvalidEntityType(format!(
                    "Unsupported variable type: {}",
                    var_type
                )))
            }
        }

        Ok(all_values)
    }

    /// Read variable time series
    ///
    /// # Arguments
    ///
    /// * `start_step` - Starting time step index (0-based)
    /// * `end_step` - Ending time step index (exclusive)
    /// * `var_type` - Entity type
    /// * `entity_id` - Entity ID (block ID for block variables, 0 for global/nodal)
    /// * `var_index` - Variable index (0-based)
    ///
    /// # Returns
    ///
    /// Variable values for all time steps
    ///
    /// # Errors
    ///
    /// Returns an error if NetCDF read fails
    pub fn var_time_series(
        &self,
        start_step: usize,
        end_step: usize,
        var_type: EntityType,
        entity_id: EntityId,
        var_index: usize,
    ) -> Result<Vec<f64>> {
        let var_name = self.get_var_name_read(var_type, entity_id, var_index)?;

        let var = self
            .nc_file
            .variable(&var_name)
            .ok_or_else(|| ExodusError::VariableNotDefined(var_name.clone()))?;

        match var_type {
            EntityType::Global => {
                // Global vars: (time_step, num_glo_var)
                // Read time series at [start_step:end_step, var_index]
                let mut values = Vec::new();
                for step in start_step..end_step {
                    let value: f64 = var.get_value((step, var_index))?;
                    values.push(value);
                }
                Ok(values)
            }
            EntityType::Nodal => {
                // Nodal vars: (time_step, num_nodes)
                Ok(var.get_values((start_step..end_step, ..))?)
            }
            EntityType::ElemBlock | EntityType::EdgeBlock | EntityType::FaceBlock => {
                // Block vars: (time_step, num_entries_in_block)
                Ok(var.get_values((start_step..end_step, ..))?)
            }
            EntityType::NodeSet
            | EntityType::EdgeSet
            | EntityType::FaceSet
            | EntityType::SideSet
            | EntityType::ElemSet => {
                // Set vars: (time_step, num_entries_in_set)
                Ok(var.get_values((start_step..end_step, ..))?)
            }
            _ => Err(ExodusError::InvalidEntityType(format!(
                "Unsupported variable type: {}",
                var_type
            ))),
        }
    }

    /// Read variable time series as a 2D ndarray (NumPy-compatible)
    ///
    /// Returns variable values across multiple time steps as a 2D ndarray with shape
    /// (num_steps, num_entities). This is more efficient for NumPy integration via PyO3
    /// as it provides a contiguous memory layout compatible with NumPy arrays.
    ///
    /// # Arguments
    ///
    /// * `start_step` - Starting time step index (0-based)
    /// * `end_step` - Ending time step index (exclusive)
    /// * `var_type` - Type of entity (Nodal, ElemBlock, Global, etc.)
    /// * `entity_id` - Entity ID (block/set ID, or 0 for nodal/global)
    /// * `var_index` - Variable index (0-based)
    ///
    /// # Returns
    ///
    /// An `Array2<f64>` with shape:
    /// - For Global variables: (num_steps, 1)
    /// - For Nodal variables: (num_steps, num_nodes)
    /// - For Block/Set variables: (num_steps, num_entities_in_block/set)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Variable is not defined
    /// - Entity ID is not found
    /// - Time step range is invalid
    /// - NetCDF read fails
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use exodus_rs::{ExodusFile, EntityType};
    /// use exodus_rs::mode::Read;
    ///
    /// let file = ExodusFile::<Read>::open("mesh.exo")?;
    /// let temps = file.var_time_series_array(0, 100, EntityType::Nodal, 0, 0)?;
    /// println!("Shape: {:?}", temps.shape());  // (100, num_nodes)
    ///
    /// // Access specific time step
    /// let step_0 = temps.row(0);
    ///
    /// // Access specific node history
    /// let node_5 = temps.column(5);
    /// # Ok::<(), exodus_rs::ExodusError>(())
    /// ```
    #[cfg(feature = "ndarray")]
    pub fn var_time_series_array(
        &self,
        start_step: usize,
        end_step: usize,
        var_type: EntityType,
        entity_id: EntityId,
        var_index: usize,
    ) -> Result<ndarray::Array2<f64>> {
        use ndarray::Array2;

        // Get the data as a flat vector
        let data = self.var_time_series(start_step, end_step, var_type, entity_id, var_index)?;

        let num_steps = end_step - start_step;

        // Handle empty case
        if data.is_empty() || num_steps == 0 {
            return Ok(Array2::zeros((0, 0)));
        }

        // For Global variables, reshape to (num_steps, num_vars)
        // For other types, reshape to (num_steps, num_entities)
        let num_entities = if var_type == EntityType::Global {
            // Global variables are stored differently - we collected individual values
            // The data vector length is num_steps
            if data.len() != num_steps {
                return Err(ExodusError::Other(format!(
                    "Data length mismatch: expected {} steps, got {} values",
                    num_steps,
                    data.len()
                )));
            }
            1 // Return shape (num_steps, 1) for global vars
        } else {
            // For other types, netcdf returns shape (num_steps * num_entities)
            data.len() / num_steps
        };

        // Reshape flat vector into 2D array
        // Note: Array2::from_shape_vec expects data in row-major (C) order
        Array2::from_shape_vec((num_steps, num_entities), data)
            .map_err(|e| ExodusError::Other(format!("Failed to reshape array: {}", e)))
    }

    // Helper function to get variable name for reading
    fn get_var_name_read(
        &self,
        var_type: EntityType,
        entity_id: EntityId,
        var_index: usize,
    ) -> Result<String> {
        Ok(match var_type {
            EntityType::Global => "vals_glo_var".to_string(),
            EntityType::Nodal => format!("vals_nod_var{}", var_index + 1),
            EntityType::ElemBlock => {
                let block_ids = self.block_ids(EntityType::ElemBlock)?;
                let block_index = block_ids
                    .iter()
                    .position(|&id| id == entity_id)
                    .ok_or_else(|| ExodusError::EntityNotFound {
                        entity_type: EntityType::ElemBlock.to_string(),
                        id: entity_id,
                    })?;
                format!("vals_elem_var{}eb{}", var_index + 1, block_index + 1)
            }
            EntityType::EdgeBlock => {
                let block_ids = self.block_ids(EntityType::EdgeBlock)?;
                let block_index = block_ids
                    .iter()
                    .position(|&id| id == entity_id)
                    .ok_or_else(|| ExodusError::EntityNotFound {
                        entity_type: EntityType::EdgeBlock.to_string(),
                        id: entity_id,
                    })?;
                format!("vals_edge_var{}edb{}", var_index + 1, block_index + 1)
            }
            EntityType::FaceBlock => {
                let block_ids = self.block_ids(EntityType::FaceBlock)?;
                let block_index = block_ids
                    .iter()
                    .position(|&id| id == entity_id)
                    .ok_or_else(|| ExodusError::EntityNotFound {
                        entity_type: EntityType::FaceBlock.to_string(),
                        id: entity_id,
                    })?;
                format!("vals_face_var{}fab{}", var_index + 1, block_index + 1)
            }
            EntityType::NodeSet => {
                let set_ids = self.set_ids(EntityType::NodeSet)?;
                let set_index =
                    set_ids
                        .iter()
                        .position(|&id| id == entity_id)
                        .ok_or_else(|| ExodusError::EntityNotFound {
                            entity_type: EntityType::NodeSet.to_string(),
                            id: entity_id,
                        })?;
                format!("vals_nset_var{}ns{}", var_index + 1, set_index + 1)
            }
            EntityType::EdgeSet => {
                let set_ids = self.set_ids(EntityType::EdgeSet)?;
                let set_index =
                    set_ids
                        .iter()
                        .position(|&id| id == entity_id)
                        .ok_or_else(|| ExodusError::EntityNotFound {
                            entity_type: EntityType::EdgeSet.to_string(),
                            id: entity_id,
                        })?;
                format!("vals_eset_var{}es{}", var_index + 1, set_index + 1)
            }
            EntityType::FaceSet => {
                let set_ids = self.set_ids(EntityType::FaceSet)?;
                let set_index =
                    set_ids
                        .iter()
                        .position(|&id| id == entity_id)
                        .ok_or_else(|| ExodusError::EntityNotFound {
                            entity_type: EntityType::FaceSet.to_string(),
                            id: entity_id,
                        })?;
                format!("vals_fset_var{}fs{}", var_index + 1, set_index + 1)
            }
            EntityType::SideSet => {
                let set_ids = self.set_ids(EntityType::SideSet)?;
                let set_index =
                    set_ids
                        .iter()
                        .position(|&id| id == entity_id)
                        .ok_or_else(|| ExodusError::EntityNotFound {
                            entity_type: EntityType::SideSet.to_string(),
                            id: entity_id,
                        })?;
                format!("vals_sset_var{}ss{}", var_index + 1, set_index + 1)
            }
            EntityType::ElemSet => {
                let set_ids = self.set_ids(EntityType::ElemSet)?;
                let set_index =
                    set_ids
                        .iter()
                        .position(|&id| id == entity_id)
                        .ok_or_else(|| ExodusError::EntityNotFound {
                            entity_type: EntityType::ElemSet.to_string(),
                            id: entity_id,
                        })?;
                format!("vals_elset_var{}els{}", var_index + 1, set_index + 1)
            }
            _ => {
                return Err(ExodusError::InvalidEntityType(format!(
                    "Unsupported variable type: {}",
                    var_type
                )))
            }
        })
    }
}

// ====================
// Append Operations (Read + Write for time values)
// ====================

impl ExodusFile<mode::Append> {
    /// Get all time values
    ///
    /// # Returns
    ///
    /// Vector of time values
    ///
    /// # Errors
    ///
    /// Returns an error if NetCDF read fails
    pub fn times(&self) -> Result<Vec<f64>> {
        match self.nc_file.variable("time_whole") {
            Some(var) => Ok(var.get_values(..)?),
            None => Ok(Vec::new()),
        }
    }

    /// Get time value for a step
    ///
    /// # Arguments
    ///
    /// * `step` - Time step index (0-based)
    ///
    /// # Returns
    ///
    /// Time value
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Time step is out of range
    /// - NetCDF read fails
    pub fn time(&self, step: usize) -> Result<f64> {
        let times = self.times()?;
        times
            .get(step)
            .copied()
            .ok_or(ExodusError::InvalidTimeStep(step))
    }

    /// Write time value for a time step
    ///
    /// # Arguments
    ///
    /// * `step` - Time step index (0-based)
    /// * `time` - Time value
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - NetCDF write fails
    /// - time_whole variable doesn't exist in the file
    pub fn put_time(&mut self, step: usize, time: f64) -> Result<()> {
        // Ensure we're in data mode for writing time values
        self.ensure_data_mode()?;

        // Write the time value - the variable should already exist for append mode
        if let Some(mut var) = self.nc_file.variable_mut("time_whole") {
            var.put_value(time, step..step + 1)?;
            Ok(())
        } else {
            Err(ExodusError::Other(
                "time_whole variable not found in file".to_string(),
            ))
        }
    }

    /// Read a variable at a specific time step (append mode - provides read access)
    ///
    /// This method automatically handles both storage formats:
    /// - **Separate format**: Individual variables (e.g., `vals_nod_var1`, `vals_nod_var2`)
    /// - **Combined format**: Single 3D array (e.g., `vals_nod_var(time_step, num_vars, num_nodes)`)
    ///
    /// The storage format is detected automatically when the file is opened.
    pub fn var(
        &self,
        step: usize,
        var_type: EntityType,
        entity_id: EntityId,
        var_index: usize,
    ) -> Result<Vec<f64>> {
        // Get the storage mode for this entity type
        let storage_mode = match var_type {
            EntityType::Global => self.metadata.storage_format.global,
            EntityType::Nodal => self.metadata.storage_format.nodal,
            EntityType::ElemBlock => self.metadata.storage_format.elem_block,
            EntityType::EdgeBlock => self.metadata.storage_format.edge_block,
            EntityType::FaceBlock => self.metadata.storage_format.face_block,
            EntityType::NodeSet => self.metadata.storage_format.node_set,
            EntityType::EdgeSet => self.metadata.storage_format.edge_set,
            EntityType::FaceSet => self.metadata.storage_format.face_set,
            EntityType::SideSet => self.metadata.storage_format.side_set,
            EntityType::ElemSet => self.metadata.storage_format.elem_set,
            _ => {
                return Err(ExodusError::InvalidEntityType(format!(
                    "Unsupported variable type: {}",
                    var_type
                )))
            }
        };

        match storage_mode {
            VarStorageMode::Combined => self.read_var_combined_append(step, var_type, var_index),
            VarStorageMode::Separate | VarStorageMode::None => {
                self.read_var_separate_append(step, var_type, entity_id, var_index)
            }
        }
    }

    /// Read variable from separate format (vals_nod_var1, vals_nod_var2, etc.)
    fn read_var_separate_append(
        &self,
        step: usize,
        var_type: EntityType,
        entity_id: EntityId,
        var_index: usize,
    ) -> Result<Vec<f64>> {
        // Simplified variable name construction (same logic as Read mode)
        let var_name = match var_type {
            EntityType::Global => "vals_glo_var".to_string(),
            EntityType::Nodal => format!("vals_nod_var{}", var_index + 1),
            EntityType::ElemBlock => {
                let block_ids = self.block_ids(EntityType::ElemBlock)?;
                let block_index = block_ids
                    .iter()
                    .position(|&id| id == entity_id)
                    .ok_or_else(|| ExodusError::EntityNotFound {
                        entity_type: EntityType::ElemBlock.to_string(),
                        id: entity_id,
                    })?;
                format!("vals_elem_var{}eb{}", var_index + 1, block_index + 1)
            }
            EntityType::EdgeBlock => {
                let block_ids = self.block_ids(EntityType::EdgeBlock)?;
                let block_index = block_ids
                    .iter()
                    .position(|&id| id == entity_id)
                    .ok_or_else(|| ExodusError::EntityNotFound {
                        entity_type: EntityType::EdgeBlock.to_string(),
                        id: entity_id,
                    })?;
                format!("vals_edge_var{}edb{}", var_index + 1, block_index + 1)
            }
            EntityType::FaceBlock => {
                let block_ids = self.block_ids(EntityType::FaceBlock)?;
                let block_index = block_ids
                    .iter()
                    .position(|&id| id == entity_id)
                    .ok_or_else(|| ExodusError::EntityNotFound {
                        entity_type: EntityType::FaceBlock.to_string(),
                        id: entity_id,
                    })?;
                format!("vals_face_var{}fab{}", var_index + 1, block_index + 1)
            }
            EntityType::NodeSet => {
                let set_ids = self.set_ids(EntityType::NodeSet)?;
                let set_index =
                    set_ids
                        .iter()
                        .position(|&id| id == entity_id)
                        .ok_or_else(|| ExodusError::EntityNotFound {
                            entity_type: EntityType::NodeSet.to_string(),
                            id: entity_id,
                        })?;
                format!("vals_nset_var{}ns{}", var_index + 1, set_index + 1)
            }
            EntityType::SideSet => {
                let set_ids = self.set_ids(EntityType::SideSet)?;
                let set_index =
                    set_ids
                        .iter()
                        .position(|&id| id == entity_id)
                        .ok_or_else(|| ExodusError::EntityNotFound {
                            entity_type: EntityType::SideSet.to_string(),
                            id: entity_id,
                        })?;
                format!("vals_sset_var{}ss{}", var_index + 1, set_index + 1)
            }
            _ => {
                return Err(ExodusError::Other(format!(
                    "Variable reading not yet fully implemented for {:?} in append mode",
                    var_type
                )))
            }
        };

        let var = self
            .nc_file
            .variable(&var_name)
            .ok_or_else(|| ExodusError::VariableNotDefined(var_name.clone()))?;

        match var_type {
            EntityType::Global => {
                let value: f64 = var.get_value((step, var_index))?;
                Ok(vec![value])
            }
            _ => Ok(var.get_values((step..step + 1, ..))?),
        }
    }

    /// Read variable from combined 3D format (append mode)
    fn read_var_combined_append(
        &self,
        step: usize,
        var_type: EntityType,
        var_index: usize,
    ) -> Result<Vec<f64>> {
        let var_name = match var_type {
            EntityType::Global => "vals_glo_var",
            EntityType::Nodal => "vals_nod_var",
            EntityType::ElemBlock => "vals_elem_var",
            EntityType::EdgeBlock => "vals_edge_var",
            EntityType::FaceBlock => "vals_face_var",
            EntityType::NodeSet => "vals_nset_var",
            EntityType::EdgeSet => "vals_eset_var",
            EntityType::FaceSet => "vals_fset_var",
            EntityType::SideSet => "vals_sset_var",
            EntityType::ElemSet => "vals_elset_var",
            _ => {
                return Err(ExodusError::InvalidEntityType(format!(
                    "Unsupported variable type: {}",
                    var_type
                )))
            }
        };

        let var = self
            .nc_file
            .variable(var_name)
            .ok_or_else(|| ExodusError::VariableNotDefined(var_name.to_string()))?;

        match var_type {
            EntityType::Global => {
                // Global vars in combined format: (time_step, num_glo_var)
                let value: f64 = var.get_value((step, var_index))?;
                Ok(vec![value])
            }
            _ => {
                // Other vars in combined format: (time_step, num_vars, num_entities)
                // Get the number of entities from the last dimension
                let dims = var.dimensions();
                if dims.len() != 3 {
                    return Err(ExodusError::Other(format!(
                        "Expected 3D array for combined variable {}, got {} dimensions",
                        var_name,
                        dims.len()
                    )));
                }
                let num_entities = dims[2].len();

                // Read the slice [step, var_index, 0:num_entities]
                // Try reading as f64 first, fall back to f32 if that fails
                let result: std::result::Result<Vec<f64>, _> =
                    var.get_values((step..step + 1, var_index..var_index + 1, 0..num_entities));

                match result {
                    Ok(values) => Ok(values),
                    Err(_) => {
                        // Try reading as f32 and converting to f64
                        let values_f32: Vec<f32> = var.get_values((
                            step..step + 1,
                            var_index..var_index + 1,
                            0..num_entities,
                        ))?;
                        Ok(values_f32.into_iter().map(|v| v as f64).collect())
                    }
                }
            }
        }
    }

    /// Write a variable at a specific time step (append mode - provides write access)
    /// This is a simplified version that assumes the variable already exists
    pub fn put_var(
        &mut self,
        step: usize,
        var_type: EntityType,
        entity_id: EntityId,
        var_index: usize,
        values: &[f64],
    ) -> Result<()> {
        // Simplified variable name construction (same as var method)
        let var_name = match var_type {
            EntityType::Global => "vals_glo_var".to_string(),
            EntityType::Nodal => format!("vals_nod_var{}", var_index + 1),
            EntityType::ElemBlock => {
                let block_ids = self.block_ids(EntityType::ElemBlock)?;
                let block_index = block_ids
                    .iter()
                    .position(|&id| id == entity_id)
                    .ok_or_else(|| ExodusError::EntityNotFound {
                        entity_type: EntityType::ElemBlock.to_string(),
                        id: entity_id,
                    })?;
                format!("vals_elem_var{}eb{}", var_index + 1, block_index + 1)
            }
            _ => {
                return Err(ExodusError::Other(format!(
                    "Variable writing not yet fully implemented for {:?} in append mode",
                    var_type
                )))
            }
        };

        // Ensure we're in data mode for writing
        self.ensure_data_mode()?;

        // Write the values
        if let Some(mut var) = self.nc_file.variable_mut(&var_name) {
            match var_type {
                EntityType::Global => {
                    if values.len() != 1 {
                        return Err(ExodusError::InvalidArrayLength {
                            expected: 1,
                            actual: values.len(),
                        });
                    }
                    var.put_value(values[0], (step..step + 1, var_index..var_index + 1))?;
                }
                EntityType::Nodal | EntityType::ElemBlock => {
                    var.put_values(values, (step..step + 1, ..))?;
                }
                _ => {
                    return Err(ExodusError::Other(format!(
                        "Unsupported variable type: {:?}",
                        var_type
                    )))
                }
            }
        }

        Ok(())
    }

    /// Check if a variable is in the truth table (append mode)
    pub fn is_var_in_truth_table(
        &self,
        var_type: EntityType,
        entity_id: EntityId,
        var_index: usize,
    ) -> Result<bool> {
        // Only block types use truth tables
        match var_type {
            EntityType::ElemBlock | EntityType::EdgeBlock | EntityType::FaceBlock => {
                let truth_table = self.truth_table(var_type)?;
                let block_ids = self.block_ids(var_type)?;

                let block_idx = block_ids
                    .iter()
                    .position(|&id| id == entity_id)
                    .ok_or_else(|| ExodusError::EntityNotFound {
                        entity_type: var_type.to_string(),
                        id: entity_id,
                    })?;

                Ok(truth_table.get(block_idx, var_index))
            }
            _ => Ok(true), // Non-block types don't use truth tables
        }
    }

    /// Get the truth table for a variable type (append mode - delegates to common implementation)
    pub fn truth_table(&self, var_type: EntityType) -> Result<TruthTable> {
        // Get the dimension name for the number of variables
        let num_var_dim = match var_type {
            EntityType::ElemBlock => "num_elem_var",
            EntityType::EdgeBlock => "num_edge_var",
            EntityType::FaceBlock => "num_face_var",
            _ => {
                return Err(ExodusError::Other(format!(
                    "Truth tables only supported for block types, got: {:?}",
                    var_type
                )))
            }
        };

        // Get number of variables
        let num_vars = self
            .nc_file
            .dimension(num_var_dim)
            .map(|d| d.len())
            .unwrap_or(0);

        if num_vars == 0 {
            return Ok(TruthTable::new(var_type, 0, 0));
        }

        // Get block IDs
        let block_ids = self.block_ids(var_type)?;
        let num_blocks = block_ids.len();

        if num_blocks == 0 {
            return Ok(TruthTable::new(var_type, 0, num_vars));
        }

        // Try to read the truth table variable
        let truth_var_name = match var_type {
            EntityType::ElemBlock => "elem_var_tab",
            EntityType::EdgeBlock => "edge_var_tab",
            EntityType::FaceBlock => "face_var_tab",
            _ => unreachable!(),
        };

        let mut table = TruthTable::new(var_type, num_blocks, num_vars);

        if let Some(var) = self.nc_file.variable(truth_var_name) {
            let truth_data: Vec<i32> = var.get_values((.., ..))?;
            for block_idx in 0..num_blocks {
                for var_idx in 0..num_vars {
                    let value = truth_data[block_idx * num_vars + var_idx];
                    table.set(block_idx, var_idx, value != 0);
                }
            }
        } else {
            // If no truth table exists, assume all variables exist for all blocks
            for block_idx in 0..num_blocks {
                for var_idx in 0..num_vars {
                    table.set(block_idx, var_idx, true);
                }
            }
        }

        Ok(table)
    }
}
