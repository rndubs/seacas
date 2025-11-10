//! Attribute operations for Exodus II files.
//!
//! Attributes provide enhanced metadata that can be attached to entities
//! (blocks, sets, etc.). They can store integer, double, or character data.

use crate::error::{ExodusError, Result};
use crate::types::{AttributeType, EntityType};
use crate::{mode, ExodusFile};

#[cfg(feature = "netcdf4")]
use netcdf;

/// Attribute value data
#[derive(Debug, Clone, PartialEq)]
pub enum AttributeData {
    /// Integer attribute values
    Integer(Vec<i64>),
    /// Double precision attribute values
    Double(Vec<f64>),
    /// Character string attribute value
    Char(String),
}

// ============================================================================
// Write Operations
// ============================================================================

#[cfg(feature = "netcdf4")]
impl ExodusFile<mode::Write> {
    /// Write an attribute to an entity
    ///
    /// Attributes provide additional metadata that can be attached to entities.
    ///
    /// # Arguments
    ///
    /// * `entity_type` - Type of entity (ElemBlock, NodeSet, etc.)
    /// * `entity_id` - ID of the specific entity
    /// * `name` - Attribute name (max 32 characters)
    /// * `attr_type` - Type of attribute data
    /// * `data` - Attribute data to write
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or an error if:
    /// - The attribute name is too long (max 32 characters)
    /// - The entity doesn't exist
    /// - NetCDF write fails
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use exodus_rs::*;
    /// # let mut file = ExodusFile::create_default("test.exo").unwrap();
    /// file.put_attribute(
    ///     EntityType::ElemBlock,
    ///     100,
    ///     "material_id",
    ///     AttributeType::Integer,
    ///     AttributeData::Integer(vec![42]),
    /// )?;
    /// # Ok::<(), ExodusError>(())
    /// ```
    pub fn put_attribute(
        &mut self,
        entity_type: EntityType,
        entity_id: i64,
        name: impl AsRef<str>,
        attr_type: AttributeType,
        data: AttributeData,
    ) -> Result<()> {
        const MAX_NAME_LENGTH: usize = 32;
        let name = name.as_ref();

        if name.len() > MAX_NAME_LENGTH {
            return Err(ExodusError::StringTooLong {
                max: MAX_NAME_LENGTH,
                actual: name.len(),
            });
        }

        // Find the NetCDF variable for this entity
        let entity_var_name = self.get_entity_variable_name(entity_type, entity_id)?;

        // Write the attribute based on type
        if let Some(var) = self.nc_file.variable(&entity_var_name) {
            match (attr_type, data) {
                (AttributeType::Integer, AttributeData::Integer(values)) => {
                    if values.len() == 1 {
                        var.put_attribute(name, values[0])
                            .map_err(|e| ExodusError::NetCdf(e))?;
                    } else {
                        var.put_attribute(name, &values[..])
                            .map_err(|e| ExodusError::NetCdf(e))?;
                    }
                }
                (AttributeType::Double, AttributeData::Double(values)) => {
                    if values.len() == 1 {
                        var.put_attribute(name, values[0])
                            .map_err(|e| ExodusError::NetCdf(e))?;
                    } else {
                        var.put_attribute(name, &values[..])
                            .map_err(|e| ExodusError::NetCdf(e))?;
                    }
                }
                (AttributeType::Char, AttributeData::Char(value)) => {
                    var.put_attribute(name, value.as_str())
                        .map_err(|e| ExodusError::NetCdf(e))?;
                }
                _ => {
                    return Err(ExodusError::Other(
                        "Attribute type does not match data type".to_string(),
                    ));
                }
            }
        } else {
            return Err(ExodusError::EntityNotFound {
                entity_type: entity_type.as_str().to_string(),
                id: entity_id,
            });
        }

        Ok(())
    }

    /// Helper to get the NetCDF variable name for an entity
    fn get_entity_variable_name(
        &self,
        entity_type: EntityType,
        entity_id: i64,
    ) -> Result<String> {
        match entity_type {
            EntityType::ElemBlock | EntityType::EdgeBlock | EntityType::FaceBlock => {
                // For blocks, use the connectivity variable
                Ok(format!("connect{}", self.get_entity_index(entity_type, entity_id)?))
            }
            EntityType::NodeSet | EntityType::EdgeSet | EntityType::FaceSet
            | EntityType::ElemSet | EntityType::SideSet => {
                // For sets, use the node/element list variable
                let index = self.get_entity_index(entity_type, entity_id)?;
                Ok(match entity_type {
                    EntityType::NodeSet => format!("node_ns{}", index),
                    EntityType::SideSet => format!("elem_ss{}", index),
                    EntityType::EdgeSet => format!("edge_es{}", index),
                    EntityType::FaceSet => format!("face_fs{}", index),
                    EntityType::ElemSet => format!("elem_els{}", index),
                    _ => unreachable!(),
                })
            }
            _ => Err(ExodusError::InvalidEntityType(
                format!("Attributes not supported for entity type: {}", entity_type.as_str()),
            )),
        }
    }

    /// Helper to get the index of an entity by its ID
    fn get_entity_index(&self, entity_type: EntityType, entity_id: i64) -> Result<usize> {
        let ids = match entity_type {
            EntityType::ElemBlock => self.elem_block_ids()?,
            EntityType::EdgeBlock => self.edge_block_ids()?,
            EntityType::FaceBlock => self.face_block_ids()?,
            EntityType::NodeSet => self.node_set_ids()?,
            EntityType::EdgeSet => self.edge_set_ids()?,
            EntityType::FaceSet => self.face_set_ids()?,
            EntityType::SideSet => self.side_set_ids()?,
            EntityType::ElemSet => self.elem_set_ids()?,
            _ => {
                return Err(ExodusError::InvalidEntityType(
                    entity_type.as_str().to_string(),
                ))
            }
        };

        ids.iter()
            .position(|&id| id == entity_id)
            .map(|i| i + 1) // NetCDF indices are 1-based
            .ok_or_else(|| ExodusError::EntityNotFound {
                entity_type: entity_type.as_str().to_string(),
                id: entity_id,
            })
    }
}

// ============================================================================
// Read Operations
// ============================================================================

#[cfg(feature = "netcdf4")]
impl ExodusFile<mode::Read> {
    /// Read an attribute from an entity
    ///
    /// # Arguments
    ///
    /// * `entity_type` - Type of entity
    /// * `entity_id` - ID of the entity
    /// * `name` - Attribute name
    ///
    /// # Returns
    ///
    /// `AttributeData` containing the attribute value(s), or an error if:
    /// - The entity doesn't exist
    /// - The attribute doesn't exist
    /// - NetCDF read fails
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use exodus_rs::*;
    /// # let file = ExodusFile::<mode::Read>::open("test.exo").unwrap();
    /// let attr = file.attribute(EntityType::ElemBlock, 100, "material_id")?;
    /// # Ok::<(), ExodusError>(())
    /// ```
    pub fn attribute(
        &self,
        entity_type: EntityType,
        entity_id: i64,
        name: impl AsRef<str>,
    ) -> Result<AttributeData> {
        let name = name.as_ref();

        // Find the NetCDF variable for this entity
        let entity_var_name = self.get_entity_variable_name_read(entity_type, entity_id)?;

        if let Some(var) = self.nc_file.variable(&entity_var_name) {
            if let Some(attr) = var.attribute(name) {
                let value = attr.value().map_err(|e| ExodusError::NetCdf(e))?;

                match value {
                    netcdf::AttributeValue::Uchar(v) => {
                        Ok(AttributeData::Integer(vec![v as i64]))
                    }
                    netcdf::AttributeValue::Schar(v) => {
                        Ok(AttributeData::Integer(vec![v as i64]))
                    }
                    netcdf::AttributeValue::Ushort(v) => {
                        Ok(AttributeData::Integer(vec![v as i64]))
                    }
                    netcdf::AttributeValue::Short(v) => {
                        Ok(AttributeData::Integer(vec![v as i64]))
                    }
                    netcdf::AttributeValue::Uint(v) => {
                        Ok(AttributeData::Integer(vec![v as i64]))
                    }
                    netcdf::AttributeValue::Int(v) => {
                        Ok(AttributeData::Integer(vec![v as i64]))
                    }
                    netcdf::AttributeValue::Ulonglong(v) => {
                        Ok(AttributeData::Integer(vec![v as i64]))
                    }
                    netcdf::AttributeValue::Longlong(v) => {
                        Ok(AttributeData::Integer(vec![v]))
                    }
                    netcdf::AttributeValue::Float(v) => {
                        Ok(AttributeData::Double(vec![v as f64]))
                    }
                    netcdf::AttributeValue::Double(v) => {
                        Ok(AttributeData::Double(vec![v]))
                    }
                    netcdf::AttributeValue::Str(s) => {
                        Ok(AttributeData::Char(s))
                    }
                    _ => Err(ExodusError::Other(
                        "Unsupported attribute type".to_string(),
                    )),
                }
            } else {
                Err(ExodusError::Other(format!("Attribute '{}' not found", name)))
            }
        } else {
            Err(ExodusError::EntityNotFound {
                entity_type: entity_type.as_str().to_string(),
                id: entity_id,
            })
        }
    }

    /// Get all attribute names for an entity
    ///
    /// # Arguments
    ///
    /// * `entity_type` - Type of entity
    /// * `entity_id` - ID of the entity
    ///
    /// # Returns
    ///
    /// Vector of attribute names
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use exodus_rs::*;
    /// # let file = ExodusFile::<mode::Read>::open("test.exo").unwrap();
    /// let names = file.attribute_names(EntityType::ElemBlock, 100)?;
    /// # Ok::<(), ExodusError>(())
    /// ```
    pub fn attribute_names(
        &self,
        entity_type: EntityType,
        entity_id: i64,
    ) -> Result<Vec<String>> {
        let entity_var_name = self.get_entity_variable_name_read(entity_type, entity_id)?;

        if let Some(var) = self.nc_file.variable(&entity_var_name) {
            let names: Vec<String> = var.attributes()
                .filter_map(|attr| {
                    // Filter out standard NetCDF attributes
                    let name = attr.name();
                    if name.starts_with("_") || name == "id" || name == "name"
                        || name == "entity_type" {
                        None
                    } else {
                        Some(name.to_string())
                    }
                })
                .collect();
            Ok(names)
        } else {
            Err(ExodusError::EntityNotFound {
                entity_type: entity_type.as_str().to_string(),
                id: entity_id,
            })
        }
    }

    /// Helper to get the NetCDF variable name for an entity (read mode)
    fn get_entity_variable_name_read(
        &self,
        entity_type: EntityType,
        entity_id: i64,
    ) -> Result<String> {
        match entity_type {
            EntityType::ElemBlock | EntityType::EdgeBlock | EntityType::FaceBlock => {
                Ok(format!("connect{}", self.get_entity_index_read(entity_type, entity_id)?))
            }
            EntityType::NodeSet | EntityType::EdgeSet | EntityType::FaceSet
            | EntityType::ElemSet | EntityType::SideSet => {
                let index = self.get_entity_index_read(entity_type, entity_id)?;
                Ok(match entity_type {
                    EntityType::NodeSet => format!("node_ns{}", index),
                    EntityType::SideSet => format!("elem_ss{}", index),
                    EntityType::EdgeSet => format!("edge_es{}", index),
                    EntityType::FaceSet => format!("face_fs{}", index),
                    EntityType::ElemSet => format!("elem_els{}", index),
                    _ => unreachable!(),
                })
            }
            _ => Err(ExodusError::InvalidEntityType(
                format!("Attributes not supported for entity type: {}", entity_type.as_str()),
            )),
        }
    }

    /// Helper to get the index of an entity by its ID (read mode)
    fn get_entity_index_read(&self, entity_type: EntityType, entity_id: i64) -> Result<usize> {
        let ids = match entity_type {
            EntityType::ElemBlock => self.elem_block_ids()?,
            EntityType::EdgeBlock => self.edge_block_ids()?,
            EntityType::FaceBlock => self.face_block_ids()?,
            EntityType::NodeSet => self.node_set_ids()?,
            EntityType::EdgeSet => self.edge_set_ids()?,
            EntityType::FaceSet => self.face_set_ids()?,
            EntityType::SideSet => self.side_set_ids()?,
            EntityType::ElemSet => self.elem_set_ids()?,
            _ => {
                return Err(ExodusError::InvalidEntityType(
                    entity_type.as_str().to_string(),
                ))
            }
        };

        ids.iter()
            .position(|&id| id == entity_id)
            .map(|i| i + 1) // NetCDF indices are 1-based
            .ok_or_else(|| ExodusError::EntityNotFound {
                entity_type: entity_type.as_str().to_string(),
                id: entity_id,
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
    use crate::types::{Block, CreateMode, CreateOptions, InitParams};
    use tempfile::NamedTempFile;

    #[test]
    fn test_integer_attribute() {
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
                title: "Attribute Test".into(),
                num_dim: 3,
                num_nodes: 8,
                num_elems: 1,
                num_elem_blocks: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

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

            // Write attribute
            file.put_attribute(
                EntityType::ElemBlock,
                100,
                "material_id",
                AttributeType::Integer,
                AttributeData::Integer(vec![42]),
            )
            .unwrap();
        }

        // Read
        {
            let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
            let attr = file
                .attribute(EntityType::ElemBlock, 100, "material_id")
                .unwrap();

            match attr {
                AttributeData::Integer(values) => {
                    assert_eq!(values[0], 42);
                }
                _ => panic!("Expected integer attribute"),
            }

            let names = file
                .attribute_names(EntityType::ElemBlock, 100)
                .unwrap();
            assert!(names.contains(&"material_id".to_string()));
        }
    }

    #[test]
    fn test_double_attribute() {
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
                title: "Double Attribute Test".into(),
                num_dim: 3,
                num_nodes: 8,
                num_elems: 1,
                num_elem_blocks: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

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

            file.put_attribute(
                EntityType::ElemBlock,
                100,
                "density",
                AttributeType::Double,
                AttributeData::Double(vec![7.85]),
            )
            .unwrap();
        }

        // Read
        {
            let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
            let attr = file.attribute(EntityType::ElemBlock, 100, "density").unwrap();

            match attr {
                AttributeData::Double(values) => {
                    assert!((values[0] - 7.85).abs() < 1e-10);
                }
                _ => panic!("Expected double attribute"),
            }
        }
    }

    #[test]
    fn test_char_attribute() {
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
                title: "String Attribute Test".into(),
                num_dim: 3,
                num_nodes: 8,
                num_elems: 1,
                num_elem_blocks: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

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

            file.put_attribute(
                EntityType::ElemBlock,
                100,
                "material_name",
                AttributeType::Char,
                AttributeData::Char("Steel".to_string()),
            )
            .unwrap();
        }

        // Read
        {
            let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
            let attr = file
                .attribute(EntityType::ElemBlock, 100, "material_name")
                .unwrap();

            match attr {
                AttributeData::Char(s) => {
                    assert_eq!(s, "Steel");
                }
                _ => panic!("Expected char attribute"),
            }
        }
    }
}
