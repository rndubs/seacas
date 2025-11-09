//! Set operations
//!
//! This module contains operations for node, edge, face, element, and side sets.
//! Implemented in Phase 5.

use crate::error::{EntityId, ExodusError, Result};
use crate::types::{EntitySet, EntityType, NodeSet, Set, SideSet};
use crate::{mode, ExodusFile, FileMode};

// ====================
// Write Operations
// ====================

impl<M: FileMode> ExodusFile<M> {
    /// Get all set IDs of a given type
    ///
    /// # Arguments
    ///
    /// * `entity_type` - Type of set (NodeSet, EdgeSet, FaceSet, ElemSet, or SideSet)
    ///
    /// # Returns
    ///
    /// Vector of set IDs
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file is not initialized
    /// - Invalid entity type (not a set type)
    /// - NetCDF read fails
    pub fn set_ids(&self, entity_type: EntityType) -> Result<Vec<EntityId>> {
        // Validate that this is a set type
        match entity_type {
            EntityType::NodeSet
            | EntityType::EdgeSet
            | EntityType::FaceSet
            | EntityType::ElemSet
            | EntityType::SideSet => {}
            _ => {
                return Err(ExodusError::InvalidEntityType(format!(
                    "Expected a set type, got {}",
                    entity_type
                )))
            }
        }

        // Get the variable name for set IDs
        let var_name = match entity_type {
            EntityType::NodeSet => "ns_prop1",
            EntityType::EdgeSet => "es_prop1",
            EntityType::FaceSet => "fs_prop1",
            EntityType::ElemSet => "els_prop1",
            EntityType::SideSet => "ss_prop1",
            _ => unreachable!(),
        };

        // Try to get the variable
        match self.nc_file.variable(var_name) {
            Some(var) => {
                // Read the IDs
                let ids: Vec<i64> = var.get_values(..)?;
                Ok(ids)
            }
            None => {
                // Variable doesn't exist, return empty vector
                Ok(Vec::new())
            }
        }
    }

    /// Get set parameters
    ///
    /// # Arguments
    ///
    /// * `entity_type` - Type of set
    /// * `set_id` - ID of the set
    ///
    /// # Returns
    ///
    /// Set parameters
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The set is not found
    /// - NetCDF read fails
    pub fn set(&self, entity_type: EntityType, set_id: EntityId) -> Result<Set> {
        // Find the index of this set
        let ids = self.set_ids(entity_type)?;
        let index = ids
            .iter()
            .position(|&id| id == set_id)
            .ok_or_else(|| ExodusError::EntityNotFound {
                entity_type: entity_type.to_string(),
                id: set_id,
            })?;

        // Get dimension name for number of entries
        let dim_name = match entity_type {
            EntityType::NodeSet => format!("num_nod_ns{}", index + 1),
            EntityType::EdgeSet => format!("num_edge_es{}", index + 1),
            EntityType::FaceSet => format!("num_face_fs{}", index + 1),
            EntityType::ElemSet => format!("num_ele_els{}", index + 1),
            EntityType::SideSet => format!("num_side_ss{}", index + 1),
            _ => {
                return Err(ExodusError::InvalidEntityType(format!(
                    "Not a set type: {}",
                    entity_type
                )))
            }
        };

        // Get the number of entries
        let num_entries = match self.nc_file.dimension(&dim_name) {
            Some(dim) => dim.len(),
            None => 0,
        };

        // Get dimension name for distribution factors
        let df_dim_name = match entity_type {
            EntityType::NodeSet => format!("num_df_ns{}", index + 1),
            EntityType::EdgeSet => format!("num_df_es{}", index + 1),
            EntityType::FaceSet => format!("num_df_fs{}", index + 1),
            EntityType::ElemSet => format!("num_df_els{}", index + 1),
            EntityType::SideSet => format!("num_df_ss{}", index + 1),
            _ => unreachable!(),
        };

        // Get the number of distribution factors
        let num_dist_factors = match self.nc_file.dimension(&df_dim_name) {
            Some(dim) => dim.len(),
            None => 0,
        };

        Ok(Set {
            id: set_id,
            entity_type,
            num_entries,
            num_dist_factors,
        })
    }
}

impl ExodusFile<mode::Write> {
    /// Define a set
    ///
    /// This creates the NetCDF dimensions and variables for a set.
    ///
    /// # Arguments
    ///
    /// * `set` - Set parameters
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file is not initialized
    /// - Invalid entity type (not a set type)
    /// - NetCDF write fails
    pub fn put_set(&mut self, set: &Set) -> Result<()> {
        // Get the current number of sets of this type
        let ids = self.set_ids(set.entity_type)?;
        let index = ids.len();

        // Create dimension for number of entries
        let entries_dim_name = match set.entity_type {
            EntityType::NodeSet => format!("num_nod_ns{}", index + 1),
            EntityType::EdgeSet => format!("num_edge_es{}", index + 1),
            EntityType::FaceSet => format!("num_face_fs{}", index + 1),
            EntityType::ElemSet => format!("num_ele_els{}", index + 1),
            EntityType::SideSet => format!("num_side_ss{}", index + 1),
            _ => {
                return Err(ExodusError::InvalidEntityType(format!(
                    "Not a set type: {}",
                    set.entity_type
                )))
            }
        };

        if set.num_entries > 0 {
            self.nc_file
                .add_dimension(&entries_dim_name, set.num_entries)?;
        }

        // Create dimension for distribution factors if needed
        if set.num_dist_factors > 0 {
            let df_dim_name = match set.entity_type {
                EntityType::NodeSet => format!("num_df_ns{}", index + 1),
                EntityType::EdgeSet => format!("num_df_es{}", index + 1),
                EntityType::FaceSet => format!("num_df_fs{}", index + 1),
                EntityType::ElemSet => format!("num_df_els{}", index + 1),
                EntityType::SideSet => format!("num_df_ss{}", index + 1),
                _ => unreachable!(),
            };

            self.nc_file
                .add_dimension(&df_dim_name, set.num_dist_factors)?;
        }

        // Create or update the ID property array
        let prop_var_name = match set.entity_type {
            EntityType::NodeSet => "ns_prop1",
            EntityType::EdgeSet => "es_prop1",
            EntityType::FaceSet => "fs_prop1",
            EntityType::ElemSet => "els_prop1",
            EntityType::SideSet => "ss_prop1",
            _ => unreachable!(),
        };

        // If this is the first set of this type, create the property variable
        if index == 0 {
            let dim_name = match set.entity_type {
                EntityType::NodeSet => "num_node_sets",
                EntityType::EdgeSet => "num_edge_sets",
                EntityType::FaceSet => "num_face_sets",
                EntityType::ElemSet => "num_elem_sets",
                EntityType::SideSet => "num_side_sets",
                _ => unreachable!(),
            };

            // Get the number of sets from the dimension
            let num_sets = self
                .nc_file
                .dimension(dim_name)
                .ok_or_else(|| {
                    ExodusError::Other(format!(
                        "Database not initialized for {} sets (dimension {} not found)",
                        set.entity_type, dim_name
                    ))
                })?
                .len();

            if num_sets == 0 {
                return Err(ExodusError::Other(format!(
                    "Database has zero {} sets",
                    set.entity_type
                )));
            }

            // Create the property variable
            let mut var = self
                .nc_file
                .add_variable::<i64>(prop_var_name, &[dim_name])?;
            var.put_attribute("name", "ID")?;
        }

        // Update the ID array at the current index
        if let Some(mut var) = self.nc_file.variable_mut(prop_var_name) {
            var.put_value(set.id, index..index + 1)?;
        }

        Ok(())
    }

    /// Write node set members
    ///
    /// # Arguments
    ///
    /// * `set_id` - ID of the set
    /// * `nodes` - Node IDs in the set
    /// * `dist_factors` - Optional distribution factors (one per node)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The set has not been defined
    /// - Array lengths don't match set parameters
    /// - NetCDF write fails
    pub fn put_node_set(
        &mut self,
        set_id: EntityId,
        nodes: &[i64],
        dist_factors: Option<&[f64]>,
    ) -> Result<()> {
        // Find the set index
        let ids = self.set_ids(EntityType::NodeSet)?;
        let index = ids
            .iter()
            .position(|&id| id == set_id)
            .ok_or_else(|| ExodusError::EntityNotFound {
                entity_type: EntityType::NodeSet.to_string(),
                id: set_id,
            })?;

        // Get the set parameters to validate
        let set = self.set(EntityType::NodeSet, set_id)?;

        // Validate node array length
        if nodes.len() != set.num_entries {
            return Err(ExodusError::InvalidArrayLength {
                expected: set.num_entries,
                actual: nodes.len(),
            });
        }

        // Create and write node variable
        let node_var_name = format!("node_ns{}", index + 1);
        let entries_dim_name = format!("num_nod_ns{}", index + 1);

        let mut node_var = self
            .nc_file
            .add_variable::<i64>(&node_var_name, &[&entries_dim_name])?;
        node_var.put_values(nodes, ..)?;

        // Write distribution factors if provided
        if let Some(df) = dist_factors {
            if df.len() != set.num_dist_factors {
                return Err(ExodusError::InvalidArrayLength {
                    expected: set.num_dist_factors,
                    actual: df.len(),
                });
            }

            let df_var_name = format!("dist_fact_ns{}", index + 1);
            let df_dim_name = format!("num_df_ns{}", index + 1);

            let mut df_var = self
                .nc_file
                .add_variable::<f64>(&df_var_name, &[&df_dim_name])?;
            df_var.put_values(df, ..)?;
        }

        Ok(())
    }

    /// Write side set members
    ///
    /// # Arguments
    ///
    /// * `set_id` - ID of the set
    /// * `elements` - Element IDs that define the sides
    /// * `sides` - Side numbers within each element (topology dependent)
    /// * `dist_factors` - Optional distribution factors (one per node-on-side)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The set has not been defined
    /// - Array lengths don't match set parameters
    /// - NetCDF write fails
    pub fn put_side_set(
        &mut self,
        set_id: EntityId,
        elements: &[i64],
        sides: &[i64],
        dist_factors: Option<&[f64]>,
    ) -> Result<()> {
        // Find the set index
        let ids = self.set_ids(EntityType::SideSet)?;
        let index = ids
            .iter()
            .position(|&id| id == set_id)
            .ok_or_else(|| ExodusError::EntityNotFound {
                entity_type: EntityType::SideSet.to_string(),
                id: set_id,
            })?;

        // Get the set parameters to validate
        let set = self.set(EntityType::SideSet, set_id)?;

        // Validate element and side array lengths
        if elements.len() != set.num_entries {
            return Err(ExodusError::InvalidArrayLength {
                expected: set.num_entries,
                actual: elements.len(),
            });
        }

        if sides.len() != set.num_entries {
            return Err(ExodusError::InvalidArrayLength {
                expected: set.num_entries,
                actual: sides.len(),
            });
        }

        // Create and write element variable
        let elem_var_name = format!("elem_ss{}", index + 1);
        let side_var_name = format!("side_ss{}", index + 1);
        let entries_dim_name = format!("num_side_ss{}", index + 1);

        let mut elem_var = self
            .nc_file
            .add_variable::<i64>(&elem_var_name, &[&entries_dim_name])?;
        elem_var.put_values(elements, ..)?;

        let mut side_var = self
            .nc_file
            .add_variable::<i64>(&side_var_name, &[&entries_dim_name])?;
        side_var.put_values(sides, ..)?;

        // Write distribution factors if provided
        if let Some(df) = dist_factors {
            if df.len() != set.num_dist_factors {
                return Err(ExodusError::InvalidArrayLength {
                    expected: set.num_dist_factors,
                    actual: df.len(),
                });
            }

            let df_var_name = format!("dist_fact_ss{}", index + 1);
            let df_dim_name = format!("num_df_ss{}", index + 1);

            let mut df_var = self
                .nc_file
                .add_variable::<f64>(&df_var_name, &[&df_dim_name])?;
            df_var.put_values(df, ..)?;
        }

        Ok(())
    }

    /// Write element/edge/face set
    ///
    /// # Arguments
    ///
    /// * `entity_type` - Type of set (EdgeSet, FaceSet, or ElemSet)
    /// * `set_id` - ID of the set
    /// * `entities` - Entity IDs in the set
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The set has not been defined
    /// - Invalid entity type (not EdgeSet, FaceSet, or ElemSet)
    /// - Array lengths don't match set parameters
    /// - NetCDF write fails
    pub fn put_entity_set(
        &mut self,
        entity_type: EntityType,
        set_id: EntityId,
        entities: &[i64],
    ) -> Result<()> {
        // Validate entity type
        match entity_type {
            EntityType::EdgeSet | EntityType::FaceSet | EntityType::ElemSet => {}
            _ => {
                return Err(ExodusError::InvalidEntityType(format!(
                    "Expected EdgeSet, FaceSet, or ElemSet, got {}",
                    entity_type
                )))
            }
        }

        // Find the set index
        let ids = self.set_ids(entity_type)?;
        let index = ids
            .iter()
            .position(|&id| id == set_id)
            .ok_or_else(|| ExodusError::EntityNotFound {
                entity_type: entity_type.to_string(),
                id: set_id,
            })?;

        // Get the set parameters to validate
        let set = self.set(entity_type, set_id)?;

        // Validate entities array length
        if entities.len() != set.num_entries {
            return Err(ExodusError::InvalidArrayLength {
                expected: set.num_entries,
                actual: entities.len(),
            });
        }

        // Create and write entity variable
        let (var_name, dim_name) = match entity_type {
            EntityType::EdgeSet => (format!("edge_es{}", index + 1), format!("num_edge_es{}", index + 1)),
            EntityType::FaceSet => (format!("face_fs{}", index + 1), format!("num_face_fs{}", index + 1)),
            EntityType::ElemSet => (format!("elem_els{}", index + 1), format!("num_ele_els{}", index + 1)),
            _ => unreachable!(),
        };

        let mut var = self.nc_file.add_variable::<i64>(&var_name, &[&dim_name])?;
        var.put_values(entities, ..)?;

        Ok(())
    }
}

// ====================
// Read Operations
// ====================

impl ExodusFile<mode::Read> {
    /// Get node set
    ///
    /// # Arguments
    ///
    /// * `set_id` - ID of the set
    ///
    /// # Returns
    ///
    /// Node set with node IDs and distribution factors
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The set is not found
    /// - NetCDF read fails
    pub fn node_set(&self, set_id: EntityId) -> Result<NodeSet> {
        // Find the set index
        let ids = self.set_ids(EntityType::NodeSet)?;
        let index = ids
            .iter()
            .position(|&id| id == set_id)
            .ok_or_else(|| ExodusError::EntityNotFound {
                entity_type: EntityType::NodeSet.to_string(),
                id: set_id,
            })?;

        // Get the set parameters
        let set = self.set(EntityType::NodeSet, set_id)?;

        // Read node IDs
        let node_var_name = format!("node_ns{}", index + 1);
        let nodes: Vec<i64> = self
            .nc_file
            .variable(&node_var_name)
            .ok_or_else(|| ExodusError::VariableNotDefined(node_var_name.clone()))?
            .get_values(..)?;

        // Read distribution factors if present
        let df_var_name = format!("dist_fact_ns{}", index + 1);
        let dist_factors = match self.nc_file.variable(&df_var_name) {
            Some(var) => var.get_values(..)?,
            None => Vec::new(),
        };

        Ok(NodeSet {
            id: set_id,
            nodes,
            dist_factors,
        })
    }

    /// Get side set
    ///
    /// # Arguments
    ///
    /// * `set_id` - ID of the set
    ///
    /// # Returns
    ///
    /// Side set with element-side pairs and distribution factors
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The set is not found
    /// - NetCDF read fails
    pub fn side_set(&self, set_id: EntityId) -> Result<SideSet> {
        // Find the set index
        let ids = self.set_ids(EntityType::SideSet)?;
        let index = ids
            .iter()
            .position(|&id| id == set_id)
            .ok_or_else(|| ExodusError::EntityNotFound {
                entity_type: EntityType::SideSet.to_string(),
                id: set_id,
            })?;

        // Get the set parameters
        let set = self.set(EntityType::SideSet, set_id)?;

        // Read element IDs
        let elem_var_name = format!("elem_ss{}", index + 1);
        let elements: Vec<i64> = self
            .nc_file
            .variable(&elem_var_name)
            .ok_or_else(|| ExodusError::VariableNotDefined(elem_var_name.clone()))?
            .get_values(..)?;

        // Read side IDs
        let side_var_name = format!("side_ss{}", index + 1);
        let sides: Vec<i64> = self
            .nc_file
            .variable(&side_var_name)
            .ok_or_else(|| ExodusError::VariableNotDefined(side_var_name.clone()))?
            .get_values(..)?;

        // Read distribution factors if present
        let df_var_name = format!("dist_fact_ss{}", index + 1);
        let dist_factors = match self.nc_file.variable(&df_var_name) {
            Some(var) => var.get_values(..)?,
            None => Vec::new(),
        };

        Ok(SideSet {
            id: set_id,
            elements,
            sides,
            dist_factors,
        })
    }

    /// Get entity set (edge, face, or element set)
    ///
    /// # Arguments
    ///
    /// * `entity_type` - Type of set (EdgeSet, FaceSet, or ElemSet)
    /// * `set_id` - ID of the set
    ///
    /// # Returns
    ///
    /// Entity set with entity IDs
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The set is not found
    /// - Invalid entity type (not EdgeSet, FaceSet, or ElemSet)
    /// - NetCDF read fails
    pub fn entity_set(&self, entity_type: EntityType, set_id: EntityId) -> Result<EntitySet> {
        // Validate entity type
        match entity_type {
            EntityType::EdgeSet | EntityType::FaceSet | EntityType::ElemSet => {}
            _ => {
                return Err(ExodusError::InvalidEntityType(format!(
                    "Expected EdgeSet, FaceSet, or ElemSet, got {}",
                    entity_type
                )))
            }
        }

        // Find the set index
        let ids = self.set_ids(entity_type)?;
        let index = ids
            .iter()
            .position(|&id| id == set_id)
            .ok_or_else(|| ExodusError::EntityNotFound {
                entity_type: entity_type.to_string(),
                id: set_id,
            })?;

        // Read entity IDs
        let var_name = match entity_type {
            EntityType::EdgeSet => format!("edge_es{}", index + 1),
            EntityType::FaceSet => format!("face_fs{}", index + 1),
            EntityType::ElemSet => format!("elem_els{}", index + 1),
            _ => unreachable!(),
        };

        let entities: Vec<i64> = self
            .nc_file
            .variable(&var_name)
            .ok_or_else(|| ExodusError::VariableNotDefined(var_name.clone()))?
            .get_values(..)?;

        Ok(EntitySet {
            id: set_id,
            entity_type,
            entities,
        })
    }
}

// ====================
// Set Iteration
// ====================

impl<M: FileMode> ExodusFile<M> {
    /// Get an iterator over all sets of a given type
    ///
    /// # Arguments
    ///
    /// * `entity_type` - Type of set to iterate over
    ///
    /// # Returns
    ///
    /// Iterator over set IDs
    ///
    /// # Errors
    ///
    /// Returns an error if the entity type is not a set type
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let file = ExodusFile::<mode::Read>::open("mesh.exo")?;
    ///
    /// // Iterate over all node sets
    /// for set_id in file.sets(EntityType::NodeSet)? {
    ///     let node_set = file.node_set(set_id)?;
    ///     println!("Node set {}: {} nodes", set_id, node_set.nodes.len());
    /// }
    /// ```
    pub fn sets(&self, entity_type: EntityType) -> Result<SetIterator> {
        let ids = self.set_ids(entity_type)?;
        Ok(SetIterator {
            ids,
            index: 0,
        })
    }
}

/// Iterator over set IDs
#[derive(Debug)]
pub struct SetIterator {
    ids: Vec<EntityId>,
    index: usize,
}

impl Iterator for SetIterator {
    type Item = EntityId;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.ids.len() {
            let id = self.ids[self.index];
            self.index += 1;
            Some(id)
        } else {
            None
        }
    }
}

impl ExactSizeIterator for SetIterator {
    fn len(&self) -> usize {
        self.ids.len() - self.index
    }
}
