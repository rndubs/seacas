//! Attribute operations for Exodus II files.
//!
//! Attributes provide enhanced metadata that can be attached to entities
//! (blocks, sets, etc.). They can store integer, double, or character data.
//!
//! This implementation stores attributes as NetCDF variables, allowing full
//! support for all data types and multi-value attributes.

use crate::error::{ExodusError, Result};
use crate::types::{AttributeType, EntityType};
use crate::{mode, ExodusFile};

#[cfg(feature = "netcdf4")]
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

impl AttributeData {
    /// Get the attribute type
    pub fn attr_type(&self) -> AttributeType {
        match self {
            AttributeData::Integer(_) => AttributeType::Integer,
            AttributeData::Double(_) => AttributeType::Double,
            AttributeData::Char(_) => AttributeType::Char,
        }
    }
}

// ============================================================================
// Write Operations
// ============================================================================

#[cfg(feature = "netcdf4")]
impl ExodusFile<mode::Write> {
    /// Write an attribute to an entity
    ///
    /// Stores attributes as NetCDF variables, supporting all data types and
    /// multi-value attributes.
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
    /// // Integer attribute
    /// file.put_attribute(
    ///     EntityType::ElemBlock,
    ///     100,
    ///     "material_id",
    ///     AttributeType::Integer,
    ///     AttributeData::Integer(vec![42]),
    /// )?;
    ///
    /// // Double attribute (multi-value)
    /// file.put_attribute(
    ///     EntityType::ElemBlock,
    ///     100,
    ///     "density",
    ///     AttributeType::Double,
    ///     AttributeData::Double(vec![7850.0, 2700.0]),
    /// )?;
    ///
    /// // Character attribute
    /// file.put_attribute(
    ///     EntityType::NodeSet,
    ///     200,
    ///     "boundary_type",
    ///     AttributeType::Char,
    ///     AttributeData::Char("fixed".to_string()),
    /// )?;
    /// # Ok::<(), ExodusError>(())
    /// ```
    pub fn put_attribute(
        &mut self,
        entity_type: EntityType,
        entity_id: i64,
        name: impl AsRef<str>,
        _attr_type: AttributeType,
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

        // Create unique variable name for this attribute
        let entity_type_str = match entity_type {
            EntityType::ElemBlock => "eb",
            EntityType::NodeSet => "ns",
            EntityType::SideSet => "ss",
            EntityType::EdgeBlock => "edge",
            EntityType::FaceBlock => "face",
            _ => "entity",
        };

        // Sanitize attribute name for use in variable name (replace spaces/special chars)
        let safe_name = name.replace([' ', '-'], "_");
        let var_name = format!("{}_{}_{}_attr", entity_type_str, entity_id, safe_name);

        // Store based on data type
        match data {
            AttributeData::Integer(values) => {
                // Create dimension for values
                let dim_name = format!("{}_len", var_name);
                self.nc_file
                    .add_dimension(&dim_name, values.len())
                    .map_err(ExodusError::NetCdf)?;

                // Create variable
                let mut var = self
                    .nc_file
                    .add_variable::<i64>(&var_name, &[&dim_name])
                    .map_err(ExodusError::NetCdf)?;

                // Write values
                var.put_values(&values, ..)
                    .map_err(ExodusError::NetCdf)?;

                // Store metadata
                var.put_attribute("entity_id", entity_id)
                    .map_err(ExodusError::NetCdf)?;
                var.put_attribute("attr_name", name)
                    .map_err(ExodusError::NetCdf)?;
                var.put_attribute("attr_type", "integer")
                    .map_err(ExodusError::NetCdf)?;
            }
            AttributeData::Double(values) => {
                // Create dimension for values
                let dim_name = format!("{}_len", var_name);
                self.nc_file
                    .add_dimension(&dim_name, values.len())
                    .map_err(ExodusError::NetCdf)?;

                // Create variable
                let mut var = self
                    .nc_file
                    .add_variable::<f64>(&var_name, &[&dim_name])
                    .map_err(ExodusError::NetCdf)?;

                // Write values
                var.put_values(&values, ..)
                    .map_err(ExodusError::NetCdf)?;

                // Store metadata
                var.put_attribute("entity_id", entity_id)
                    .map_err(ExodusError::NetCdf)?;
                var.put_attribute("attr_name", name)
                    .map_err(ExodusError::NetCdf)?;
                var.put_attribute("attr_type", "double")
                    .map_err(ExodusError::NetCdf)?;
            }
            AttributeData::Char(text) => {
                // For character attributes, store as a char array
                let chars: Vec<u8> = text.bytes().collect();
                let dim_name = format!("{}_len", var_name);
                self.nc_file
                    .add_dimension(&dim_name, chars.len())
                    .map_err(ExodusError::NetCdf)?;

                // Create variable as u8 (char)
                let mut var = self
                    .nc_file
                    .add_variable::<u8>(&var_name, &[&dim_name])
                    .map_err(ExodusError::NetCdf)?;

                // Write character data
                var.put_values(&chars, ..)
                    .map_err(ExodusError::NetCdf)?;

                // Store metadata
                var.put_attribute("entity_id", entity_id)
                    .map_err(ExodusError::NetCdf)?;
                var.put_attribute("attr_name", name)
                    .map_err(ExodusError::NetCdf)?;
                var.put_attribute("attr_type", "char")
                    .map_err(ExodusError::NetCdf)?;
            }
        }

        Ok(())
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
    /// match attr {
    ///     AttributeData::Integer(values) => println!("Integer: {:?}", values),
    ///     AttributeData::Double(values) => println!("Double: {:?}", values),
    ///     AttributeData::Char(text) => println!("Char: {}", text),
    /// }
    /// # Ok::<(), ExodusError>(())
    /// ```
    pub fn attribute(
        &self,
        entity_type: EntityType,
        entity_id: i64,
        name: impl AsRef<str>,
    ) -> Result<AttributeData> {
        let name = name.as_ref();

        // Create expected variable name
        let entity_type_str = match entity_type {
            EntityType::ElemBlock => "eb",
            EntityType::NodeSet => "ns",
            EntityType::SideSet => "ss",
            EntityType::EdgeBlock => "edge",
            EntityType::FaceBlock => "face",
            _ => "entity",
        };

        let safe_name = name.replace([' ', '-'], "_");
        let var_name = format!("{}_{}_{}_attr", entity_type_str, entity_id, safe_name);

        // Try to find the attribute variable
        if let Some(var) = self.nc_file.variable(&var_name) {
            // Read the type from metadata
            let attr_type = if let Some(type_attr) = var.attribute("attr_type") {
                if let Ok(type_value) = type_attr.value() {
                    match type_value {
                        netcdf::AttributeValue::Str(s) => s,
                        _ => return Err(ExodusError::Other(
                            format!("Invalid attribute type for '{}'", name)
                        )),
                    }
                } else {
                    return Err(ExodusError::Other(
                        format!("Cannot read attribute type for '{}'", name)
                    ));
                }
            } else {
                return Err(ExodusError::Other(
                    format!("Attribute '{}' missing type metadata", name)
                ));
            };

            // Read values based on type
            match attr_type.as_str() {
                "integer" => {
                    let values: Vec<i64> = var
                        .get_values(..)
                        .map_err(ExodusError::NetCdf)?;
                    Ok(AttributeData::Integer(values))
                }
                "double" => {
                    let values: Vec<f64> = var
                        .get_values(..)
                        .map_err(ExodusError::NetCdf)?;
                    Ok(AttributeData::Double(values))
                }
                "char" => {
                    let bytes: Vec<u8> = var
                        .get_values(..)
                        .map_err(ExodusError::NetCdf)?;
                    let text = String::from_utf8(bytes)
                        .map_err(|_| ExodusError::Other(
                            format!("Invalid UTF-8 in char attribute '{}'", name)
                        ))?;
                    Ok(AttributeData::Char(text))
                }
                _ => Err(ExodusError::Other(
                    format!("Unknown attribute type '{}' for '{}'", attr_type, name)
                )),
            }
        } else {
            Err(ExodusError::Other(
                format!("Attribute '{}' not found", name)
            ))
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
        let entity_type_str = match entity_type {
            EntityType::ElemBlock => "eb",
            EntityType::NodeSet => "ns",
            EntityType::SideSet => "ss",
            EntityType::EdgeBlock => "edge",
            EntityType::FaceBlock => "face",
            _ => "entity",
        };

        let prefix = format!("{}_{}_", entity_type_str, entity_id);
        let suffix = "_attr";

        let mut names = Vec::new();

        // Iterate through all variables looking for attributes
        for var_name in self.nc_file.variables().map(|v| v.name()) {
            if var_name.starts_with(&prefix) && var_name.ends_with(suffix) {
                // Extract the attribute name from variable metadata
                if let Some(var) = self.nc_file.variable(&var_name) {
                    if let Some(name_attr) = var.attribute("attr_name") {
                        if let Ok(name_value) = name_attr.value() {
                            if let netcdf::AttributeValue::Str(s) = name_value { names.push(s) }
                        }
                    }
                }
            }
        }

        Ok(names)
    }

    /// Get all attributes for an entity
    ///
    /// Returns a vector of (name, data) tuples for all attributes of the entity.
    ///
    /// # Arguments
    ///
    /// * `entity_type` - Type of entity
    /// * `entity_id` - ID of the entity
    ///
    /// # Returns
    ///
    /// Vector of (attribute_name, AttributeData) tuples
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// # use exodus_rs::*;
    /// # let file = ExodusFile::<mode::Read>::open("test.exo").unwrap();
    /// let attributes = file.entity_attributes(EntityType::ElemBlock, 100)?;
    /// for (name, data) in attributes {
    ///     println!("Attribute {}: {:?}", name, data);
    /// }
    /// # Ok::<(), ExodusError>(())
    /// ```
    pub fn entity_attributes(
        &self,
        entity_type: EntityType,
        entity_id: i64,
    ) -> Result<Vec<(String, AttributeData)>> {
        let names = self.attribute_names(entity_type, entity_id)?;
        let mut attributes = Vec::with_capacity(names.len());

        for name in names {
            let data = self.attribute(entity_type, entity_id, &name)?;
            attributes.push((name, data));
        }

        Ok(attributes)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[cfg(feature = "netcdf4")]
mod tests {
    use super::*;
    use crate::types::{Block, CreateMode, CreateOptions, InitParams, Set};
    use tempfile::NamedTempFile;

    #[test]
    fn test_integer_attribute_single_value() {
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
                title: "Integer Attribute Test".into(),
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

            // Write single integer attribute
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
                    assert_eq!(values.len(), 1);
                    assert_eq!(values[0], 42);
                }
                _ => panic!("Expected integer attribute"),
            }

            let names = file
                .attribute_names(EntityType::ElemBlock, 100)
                .unwrap();
            assert_eq!(names.len(), 1);
            assert!(names.contains(&"material_id".to_string()));
        }
    }

    #[test]
    fn test_integer_attribute_multi_value() {
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
                title: "Multi-value Integer Test".into(),
                num_dim: 3,
                num_elem_blocks: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let block = Block {
                id: 200,
                entity_type: EntityType::ElemBlock,
                topology: "QUAD4".into(),
                num_entries: 1,
                num_nodes_per_entry: 4,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block).unwrap();

            // Write multi-value integer attribute
            file.put_attribute(
                EntityType::ElemBlock,
                200,
                "layer_ids",
                AttributeType::Integer,
                AttributeData::Integer(vec![1, 2, 3, 4, 5]),
            )
            .unwrap();
        }

        // Read
        {
            let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
            let attr = file
                .attribute(EntityType::ElemBlock, 200, "layer_ids")
                .unwrap();

            match attr {
                AttributeData::Integer(values) => {
                    assert_eq!(values.len(), 5);
                    assert_eq!(values, vec![1, 2, 3, 4, 5]);
                }
                _ => panic!("Expected integer attribute"),
            }
        }
    }

    #[test]
    fn test_double_attribute_single_value() {
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

            // Write double attribute
            file.put_attribute(
                EntityType::ElemBlock,
                100,
                "density",
                AttributeType::Double,
                AttributeData::Double(vec![7850.0]),
            )
            .unwrap();
        }

        // Read
        {
            let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
            let attr = file
                .attribute(EntityType::ElemBlock, 100, "density")
                .unwrap();

            match attr {
                AttributeData::Double(values) => {
                    assert_eq!(values.len(), 1);
                    assert_eq!(values[0], 7850.0);
                }
                _ => panic!("Expected double attribute"),
            }
        }
    }

    #[test]
    fn test_double_attribute_multi_value() {
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
                title: "Multi-value Double Test".into(),
                num_dim: 2,
                num_elem_blocks: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let block = Block {
                id: 300,
                entity_type: EntityType::ElemBlock,
                topology: "TRI3".into(),
                num_entries: 1,
                num_nodes_per_entry: 3,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: 0,
            };
            file.put_block(&block).unwrap();

            // Write multi-value double attribute
            file.put_attribute(
                EntityType::ElemBlock,
                300,
                "elastic_moduli",
                AttributeType::Double,
                AttributeData::Double(vec![200e9, 70e9, 110e9]),
            )
            .unwrap();
        }

        // Read
        {
            let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
            let attr = file
                .attribute(EntityType::ElemBlock, 300, "elastic_moduli")
                .unwrap();

            match attr {
                AttributeData::Double(values) => {
                    assert_eq!(values.len(), 3);
                    assert_eq!(values[0], 200e9);
                    assert_eq!(values[1], 70e9);
                    assert_eq!(values[2], 110e9);
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
                title: "Char Attribute Test".into(),
                num_dim: 3,
                num_node_sets: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            let set = Set {
                id: 500,
                entity_type: EntityType::NodeSet,
                num_entries: 0,
                num_dist_factors: 0,
            };
            file.put_set(&set).unwrap();

            // Write character attribute
            file.put_attribute(
                EntityType::NodeSet,
                500,
                "boundary_type",
                AttributeType::Char,
                AttributeData::Char("fixed_displacement".to_string()),
            )
            .unwrap();
        }

        // Read
        {
            let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
            let attr = file
                .attribute(EntityType::NodeSet, 500, "boundary_type")
                .unwrap();

            match attr {
                AttributeData::Char(text) => {
                    assert_eq!(text, "fixed_displacement");
                }
                _ => panic!("Expected char attribute"),
            }
        }
    }

    #[test]
    fn test_multiple_attributes_same_entity() {
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
                title: "Multiple Attributes Test".into(),
                num_dim: 3,
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

            // Write multiple attributes
            file.put_attribute(
                EntityType::ElemBlock,
                100,
                "material_id",
                AttributeType::Integer,
                AttributeData::Integer(vec![1]),
            )
            .unwrap();

            file.put_attribute(
                EntityType::ElemBlock,
                100,
                "density",
                AttributeType::Double,
                AttributeData::Double(vec![7850.0]),
            )
            .unwrap();

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

            // Check individual attributes
            let mat_id = file
                .attribute(EntityType::ElemBlock, 100, "material_id")
                .unwrap();
            assert!(matches!(mat_id, AttributeData::Integer(_)));

            let density = file
                .attribute(EntityType::ElemBlock, 100, "density")
                .unwrap();
            assert!(matches!(density, AttributeData::Double(_)));

            let name = file
                .attribute(EntityType::ElemBlock, 100, "material_name")
                .unwrap();
            assert!(matches!(name, AttributeData::Char(_)));

            // Check attribute names
            let names = file
                .attribute_names(EntityType::ElemBlock, 100)
                .unwrap();
            assert_eq!(names.len(), 3);
            assert!(names.contains(&"material_id".to_string()));
            assert!(names.contains(&"density".to_string()));
            assert!(names.contains(&"material_name".to_string()));

            // Get all attributes
            let all_attrs = file
                .entity_attributes(EntityType::ElemBlock, 100)
                .unwrap();
            assert_eq!(all_attrs.len(), 3);
        }
    }

    #[test]
    fn test_attribute_name_length_limit() {
        let tmp = NamedTempFile::new().unwrap();

        let mut file = ExodusFile::create(
            tmp.path(),
            CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            },
        )
        .unwrap();

        let params = InitParams {
            title: "Name Length Test".into(),
            num_dim: 3,
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

        // Try to write attribute with name > 32 characters
        let long_name = "this_is_a_very_long_attribute_name_that_exceeds_limit";
        let result = file.put_attribute(
            EntityType::ElemBlock,
            100,
            long_name,
            AttributeType::Integer,
            AttributeData::Integer(vec![1]),
        );

        assert!(result.is_err());
        match result {
            Err(ExodusError::StringTooLong { max, actual }) => {
                assert_eq!(max, 32);
                assert_eq!(actual, long_name.len());
            }
            _ => panic!("Expected StringTooLong error"),
        }
    }

    #[test]
    fn test_attributes_on_different_entity_types() {
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
                title: "Multi-entity Attributes Test".into(),
                num_dim: 3,
                num_elem_blocks: 1,
                num_node_sets: 1,
                num_side_sets: 1,
                ..Default::default()
            };
            file.init(&params).unwrap();

            // Element block
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
                "block_attr",
                AttributeType::Integer,
                AttributeData::Integer(vec![1]),
            )
            .unwrap();

            // Node set
            let nset = Set {
                id: 200,
                entity_type: EntityType::NodeSet,
                num_entries: 0,
                num_dist_factors: 0,
            };
            file.put_set(&nset).unwrap();
            file.put_attribute(
                EntityType::NodeSet,
                200,
                "nset_attr",
                AttributeType::Double,
                AttributeData::Double(vec![2.5]),
            )
            .unwrap();

            // Side set
            let sset = Set {
                id: 300,
                entity_type: EntityType::SideSet,
                num_entries: 0,
                num_dist_factors: 0,
            };
            file.put_set(&sset).unwrap();
            file.put_attribute(
                EntityType::SideSet,
                300,
                "sset_attr",
                AttributeType::Char,
                AttributeData::Char("boundary".to_string()),
            )
            .unwrap();
        }

        // Read
        {
            let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();

            // Verify each entity's attributes
            let block_attr = file
                .attribute(EntityType::ElemBlock, 100, "block_attr")
                .unwrap();
            assert!(matches!(block_attr, AttributeData::Integer(_)));

            let nset_attr = file
                .attribute(EntityType::NodeSet, 200, "nset_attr")
                .unwrap();
            assert!(matches!(nset_attr, AttributeData::Double(_)));

            let sset_attr = file
                .attribute(EntityType::SideSet, 300, "sset_attr")
                .unwrap();
            assert!(matches!(sset_attr, AttributeData::Char(_)));
        }
    }
}
