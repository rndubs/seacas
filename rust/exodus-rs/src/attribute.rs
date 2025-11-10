//! Attribute operations for Exodus II files.
//!
//! Attributes provide enhanced metadata that can be attached to entities
//! (blocks, sets, etc.). They can store integer, double, or character data.
//!
//! Note: This is a simplified implementation that stores attributes as properties
//! rather than full NetCDF attributes, since NetCDF requires attributes to be
//! written during define mode.

use crate::error::{ExodusError, Result};
use crate::types::{AttributeType, EntityType};
use crate::{mode, ExodusFile};

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
    /// Note: This implementation stores attributes as entity properties.
    /// For full attribute support, use block attributes or set properties.
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

        // For simplicity, store as properties
        // In a full implementation, this would create separate attribute variables
        match data {
            AttributeData::Integer(values) if values.len() == 1 => {
                // Store single integer as property
                self.put_property(entity_type, entity_id, name, values[0])?;
            }
            _ => {
                // For other types, we would need to create variables
                // For now, return an informative error
                return Err(ExodusError::Other(
                    "Attribute storage not fully implemented. Use block attributes or properties for metadata.".to_string(),
                ));
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
    /// # Ok::<(), ExodusError>(())
    /// ```
    pub fn attribute(
        &self,
        entity_type: EntityType,
        entity_id: i64,
        name: impl AsRef<str>,
    ) -> Result<AttributeData> {
        // Try to read as property first
        match self.property(entity_type, entity_id, name.as_ref()) {
            Ok(value) => Ok(AttributeData::Integer(vec![value])),
            Err(_) => Err(ExodusError::Other(
                format!("Attribute '{}' not found", name.as_ref())
            )),
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
        _entity_id: i64,
    ) -> Result<Vec<String>> {
        // Return property names
        self.property_names(entity_type)
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

            // Write attribute (as property)
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
}
