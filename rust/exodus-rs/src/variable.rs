//! Variable operations
//!
//! This module contains variable definition and I/O operations for time-dependent data.
//! Implemented in Phase 6.

use crate::error::{EntityId, ExodusError, Result};
use crate::types::{EntityType, TruthTable};
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
                // Read as 2D array of chars and convert to strings
                let num_vars = var.len();
                let mut names = Vec::new();

                for i in 0..num_vars {
                    // Read one name at a time
                    let name_chars: Vec<u8> = var.get_values((i..i+1, ..))?;
                    // Convert to string, trimming null bytes and whitespace
                    let name = String::from_utf8_lossy(&name_chars)
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
        let max_name_len = names
            .iter()
            .map(|n| n.as_ref().len())
            .max()
            .unwrap_or(32)
            .max(32); // Minimum 32 characters

        // Create or get len_string dimension
        if self.nc_file.dimension("len_string").is_none() {
            self.nc_file
                .add_dimension("len_string", max_name_len)?;
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

        // Create the actual storage variables based on type
        match var_type {
            EntityType::Global => {
                // Global vars: vals_glo_var(time_step, num_glo_var)
                if self.nc_file.variable("vals_glo_var").is_none() {
                    self.nc_file
                        .add_variable::<f64>("vals_glo_var", &["time_step", num_var_dim])?;
                }
            }
            EntityType::Nodal => {
                // Nodal vars: vals_nod_var{i}(time_step, num_nodes)
                for i in 0..num_vars {
                    let var_name = format!("vals_nod_var{}", i + 1);
                    if self.nc_file.variable(&var_name).is_none() {
                        self.nc_file
                            .add_variable::<f64>(&var_name, &["time_step", "num_nodes"])?;
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
        if let Some(mut var) = self.nc_file.variable_mut(var_name_var) {
            for (i, name) in names.iter().enumerate() {
                let name_str = name.as_ref();
                let mut buf = vec![0u8; max_name_len];
                let bytes = name_str.as_bytes();
                let copy_len = bytes.len().min(max_name_len);
                buf[..copy_len].copy_from_slice(&bytes[..copy_len]);
                var.put_values(&buf, (i..i + 1, ..))?;
            }
        }

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
            self.nc_file.add_unlimited_dimension("time_step")?;
        }

        // Ensure time_whole variable exists
        if self.nc_file.variable("time_whole").is_none() {
            let mut var = self
                .nc_file
                .add_variable::<f64>("time_whole", &["time_step"])?;
            var.put_attribute("name", "time_whole")?;
        }

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
            self.create_var_storage(var_type, entity_id, var_index)?;
        }

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
                    // Block vars: (time_step, num_elem_in_blk)
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
    /// Returns an error if NetCDF write fails
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

        let (num_blocks_dim, num_vars_dim) = match var_type {
            EntityType::ElemBlock => ("num_elem_blocks", "num_elem_var"),
            EntityType::EdgeBlock => ("num_edge_blocks", "num_edge_var"),
            EntityType::FaceBlock => ("num_face_blocks", "num_face_var"),
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

        match var_type {
            EntityType::Global => {
                // Global vars: vals_glo_var(time_step, num_glo_var)
                if self.nc_file.variable("vals_glo_var").is_none() {
                    self.nc_file
                        .add_variable::<f64>("vals_glo_var", &["time_step", "num_glo_var"])?;
                }
            }
            EntityType::Nodal => {
                // Nodal var{i}: vals_nod_var{i}(time_step, num_nodes)
                self.nc_file
                    .add_variable::<f64>(&var_name, &["time_step", "num_nodes"])?;
            }
            EntityType::ElemBlock => {
                // Element var: vals_elem_var{var_idx}eb{block_idx}(time_step, num_elem_in_blk{block_idx})
                let block_ids = self.block_ids(EntityType::ElemBlock)?;
                let block_index = block_ids
                    .iter()
                    .position(|&id| id == entity_id)
                    .ok_or_else(|| ExodusError::EntityNotFound {
                        entity_type: EntityType::ElemBlock.to_string(),
                        id: entity_id,
                    })?;

                let dim_name = format!("num_el_in_blk{}", block_index + 1);
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
                // Block vars: (time_step, num_elem_in_blk)
                Ok(var.get_values((step..step + 1, ..))?)
            }
            _ => Err(ExodusError::InvalidEntityType(format!(
                "Unsupported variable type: {}",
                var_type
            ))),
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
    /// Returns an error if NetCDF read fails
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

        // Read truth table if it exists
        let table_values = if let Some(var) = self.nc_file.variable(var_name) {
            let table_i32: Vec<i32> = var.get_values(..)?;
            table_i32.iter().map(|&v| v != 0).collect()
        } else {
            // Default: all true
            vec![true; num_blocks * num_vars]
        };

        Ok(TruthTable {
            var_type,
            num_vars,
            num_blocks,
            table: table_values,
        })
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
            _ => {
                return Err(ExodusError::InvalidEntityType(format!(
                    "Unsupported variable type: {}",
                    var_type
                )))
            }
        })
    }
}

