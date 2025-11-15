//! Assembly operations for Exodus II files.
//!
//! Assemblies provide hierarchical grouping of entities (blocks, sets, etc.)
//! for organizing complex meshes.

use crate::error::{ExodusError, Result};
use crate::types::{Assembly, EntityType};
use crate::{mode, ExodusFile};

#[cfg(feature = "netcdf4")]
// ============================================================================
// Write Operations
// ============================================================================
impl ExodusFile<mode::Write> {
    /// Define an assembly
    ///
    /// Assemblies provide hierarchical grouping of entities. Each assembly contains
    /// a list of entity IDs of a specific type.
    ///
    /// # Arguments
    ///
    /// * `assembly` - Assembly definition with ID, name, type, and entity list
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if:
    /// - The assembly name is too long (max 32 characters)
    /// - NetCDF write fails
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use exodus_rs::*;
    /// # let mut file = ExodusFile::create_default("test.exo").unwrap();
    /// let assembly = Assembly {
    ///     id: 1,
    ///     name: "Region1".into(),
    ///     entity_type: EntityType::ElemBlock,
    ///     entity_list: vec![100, 200, 300],
    /// };
    /// file.put_assembly(&assembly)?;
    /// # Ok::<(), ExodusError>(())
    /// ```
    pub fn put_assembly(&mut self, assembly: &Assembly) -> Result<()> {
        const MAX_NAME_LENGTH: usize = 32;

        if assembly.name.len() > MAX_NAME_LENGTH {
            return Err(ExodusError::StringTooLong {
                max: MAX_NAME_LENGTH,
                actual: assembly.name.len(),
            });
        }

        // Count existing assemblies by checking for assembly variables
        let mut num_assemblies = 0;
        while self
            .nc_file
            .variable(&format!("assembly{}_entity_list", num_assemblies + 1))
            .is_some()
        {
            num_assemblies += 1;
        }

        // Create assembly-specific dimensions and variables
        let assembly_index = num_assemblies;
        let _assembly_var_prefix = format!("assembly{}", assembly_index + 1);

        // Create dimension for entity count
        let entity_dim_name = format!("num_entity_assembly{}", assembly_index + 1);
        self.nc_file
            .add_dimension(&entity_dim_name, assembly.entity_list.len())
            .map_err(ExodusError::NetCdf)?;

        // Create variable for entity list
        let entity_var_name = format!("assembly{}_entity_list", assembly_index + 1);
        let mut entity_var = self
            .nc_file
            .add_variable::<i64>(&entity_var_name, &[&entity_dim_name])
            .map_err(ExodusError::NetCdf)?;

        // Write entity list
        entity_var
            .put_values(&assembly.entity_list, ..)
            .map_err(ExodusError::NetCdf)?;

        // Store assembly metadata as attributes
        entity_var
            .put_attribute("id", assembly.id)
            .map_err(ExodusError::NetCdf)?;

        entity_var
            .put_attribute("name", assembly.name.as_str())
            .map_err(ExodusError::NetCdf)?;

        entity_var
            .put_attribute("entity_type", assembly.entity_type.to_string().as_str())
            .map_err(ExodusError::NetCdf)?;

        Ok(())
    }
}

// ============================================================================
// Read Operations
// ============================================================================

#[cfg(feature = "netcdf4")]
impl ExodusFile<mode::Read> {
    /// Get all assembly IDs
    ///
    /// # Returns
    ///
    /// Vector of assembly IDs, or an error if reading fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use exodus_rs::*;
    /// # let file = ExodusFile::<mode::Read>::open("test.exo").unwrap();
    /// let assembly_ids = file.assembly_ids()?;
    /// # Ok::<(), ExodusError>(())
    /// ```
    pub fn assembly_ids(&self) -> Result<Vec<i64>> {
        // Count assemblies by checking for assembly variables
        let mut num_assemblies = 0;
        while self
            .nc_file
            .variable(&format!("assembly{}_entity_list", num_assemblies + 1))
            .is_some()
        {
            num_assemblies += 1;
        }

        if num_assemblies == 0 {
            return Ok(Vec::new());
        }

        let mut ids = Vec::with_capacity(num_assemblies);

        for i in 0..num_assemblies {
            let entity_var_name = format!("assembly{}_entity_list", i + 1);
            if let Some(var) = self.nc_file.variable(&entity_var_name) {
                if let Some(id_attr) = var.attribute("id") {
                    if let Ok(id_value) = id_attr.value() {
                        match id_value {
                            netcdf::AttributeValue::Short(id) => ids.push(id as i64),
                            netcdf::AttributeValue::Int(id) => ids.push(id as i64),
                            netcdf::AttributeValue::Longlong(id) => ids.push(id),
                            netcdf::AttributeValue::Ulonglong(id) => ids.push(id as i64),
                            netcdf::AttributeValue::Float(id) => ids.push(id as i64),
                            netcdf::AttributeValue::Double(id) => ids.push(id as i64),
                            _ => {}
                        }
                    }
                }
            }
        }

        Ok(ids)
    }

    /// Get assembly by ID
    ///
    /// # Arguments
    ///
    /// * `assembly_id` - Assembly ID to retrieve
    ///
    /// # Returns
    ///
    /// Assembly structure, or an error if not found or reading fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use exodus_rs::*;
    /// # let file = ExodusFile::<mode::Read>::open("test.exo").unwrap();
    /// let assembly = file.assembly(1)?;
    /// println!("Assembly: {} contains {} entities", assembly.name, assembly.entity_list.len());
    /// # Ok::<(), ExodusError>(())
    /// ```
    pub fn assembly(&self, assembly_id: i64) -> Result<Assembly> {
        // Count assemblies by checking for assembly variables
        let mut num_assemblies = 0;
        while self
            .nc_file
            .variable(&format!("assembly{}_entity_list", num_assemblies + 1))
            .is_some()
        {
            num_assemblies += 1;
        }

        if num_assemblies == 0 {
            return Err(ExodusError::EntityNotFound {
                entity_type: "assembly".to_string(),
                id: assembly_id,
            });
        }

        for i in 0..num_assemblies {
            let entity_var_name = format!("assembly{}_entity_list", i + 1);
            if let Some(var) = self.nc_file.variable(&entity_var_name) {
                // Check if this is the assembly we're looking for
                if let Some(id_attr) = var.attribute("id") {
                    if let Ok(id_value) = id_attr.value() {
                        let id = match id_value {
                            netcdf::AttributeValue::Short(v) => v as i64,
                            netcdf::AttributeValue::Int(v) => v as i64,
                            netcdf::AttributeValue::Longlong(v) => v,
                            netcdf::AttributeValue::Ulonglong(v) => v as i64,
                            netcdf::AttributeValue::Float(v) => v as i64,
                            netcdf::AttributeValue::Double(v) => v as i64,
                            _ => continue,
                        };

                        if id == assembly_id {
                            // Found it! Read all the data
                            let name = if let Some(name_attr) = var.attribute("name") {
                                if let Ok(name_value) = name_attr.value() {
                                    match name_value {
                                        netcdf::AttributeValue::Str(s) => s,
                                        _ => String::new(),
                                    }
                                } else {
                                    String::new()
                                }
                            } else {
                                String::new()
                            };

                            let entity_type_str =
                                if let Some(type_attr) = var.attribute("entity_type") {
                                    if let Ok(type_value) = type_attr.value() {
                                        match type_value {
                                            netcdf::AttributeValue::Str(s) => s,
                                            _ => "elem_block".to_string(),
                                        }
                                    } else {
                                        "elem_block".to_string()
                                    }
                                } else {
                                    "elem_block".to_string()
                                };

                            // Parse entity type string
                            let entity_type = match entity_type_str.as_str() {
                                "elem_block" => EntityType::ElemBlock,
                                "node_set" => EntityType::NodeSet,
                                "side_set" => EntityType::SideSet,
                                _ => EntityType::ElemBlock, // Default
                            };

                            let entity_list: Vec<i64> =
                                var.get_values(..).map_err(ExodusError::NetCdf)?;

                            return Ok(Assembly {
                                id: assembly_id,
                                name,
                                entity_type,
                                entity_list,
                            });
                        }
                    }
                }
            }
        }

        Err(ExodusError::EntityNotFound {
            entity_type: "assembly".to_string(),
            id: assembly_id,
        })
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
    fn test_assembly() {
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
                title: "Assembly Test".into(),
                num_dim: 3,
                num_elem_blocks: 3,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let assembly = Assembly {
                id: 100,
                name: "Region1".into(),
                entity_type: EntityType::ElemBlock,
                entity_list: vec![1, 2, 3],
            };

            file.put_assembly(&assembly).unwrap();
        }

        // Read
        {
            let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
            let ids = file.assembly_ids().unwrap();
            assert_eq!(ids, vec![100]);

            let assembly = file.assembly(100).unwrap();
            assert_eq!(assembly.name, "Region1");
            assert_eq!(assembly.entity_type, EntityType::ElemBlock);
            assert_eq!(assembly.entity_list, vec![1, 2, 3]);
        }
    }

    #[test]
    fn test_multiple_assemblies() {
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
                title: "Multiple Assemblies Test".into(),
                num_dim: 3,
                num_elem_blocks: 5,
                ..Default::default()
            };
            file.init(&params).unwrap();

            // Add multiple assemblies
            file.put_assembly(&Assembly {
                id: 1,
                name: "Region1".into(),
                entity_type: EntityType::ElemBlock,
                entity_list: vec![1, 2],
            })
            .unwrap();

            file.put_assembly(&Assembly {
                id: 2,
                name: "Region2".into(),
                entity_type: EntityType::ElemBlock,
                entity_list: vec![3, 4, 5],
            })
            .unwrap();
        }

        // Read
        {
            let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
            let ids = file.assembly_ids().unwrap();
            assert_eq!(ids.len(), 2);
            assert!(ids.contains(&1));
            assert!(ids.contains(&2));

            let assembly1 = file.assembly(1).unwrap();
            assert_eq!(assembly1.name, "Region1");
            assert_eq!(assembly1.entity_list.len(), 2);

            let assembly2 = file.assembly(2).unwrap();
            assert_eq!(assembly2.name, "Region2");
            assert_eq!(assembly2.entity_list.len(), 3);
        }
    }
}
