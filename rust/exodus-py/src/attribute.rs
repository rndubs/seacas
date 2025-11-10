//! Attribute operations

use pyo3::prelude::*;
use exodus_rs::AttributeData as RustAttributeData;
use crate::error::IntoPyResult;
use crate::file::{ExodusWriter, ExodusAppender, ExodusReader};
use crate::types::{EntityType, AttributeType};

/// Python wrapper for AttributeData
#[pyclass(name = "AttributeData")]
#[derive(Clone)]
pub struct AttributeData {
    inner: RustAttributeData,
}

#[pymethods]
impl AttributeData {
    /// Create an integer attribute
    #[staticmethod]
    fn integer(values: Vec<i64>) -> Self {
        AttributeData {
            inner: RustAttributeData::Integer(values),
        }
    }

    /// Create a double attribute
    #[staticmethod]
    fn double(values: Vec<f64>) -> Self {
        AttributeData {
            inner: RustAttributeData::Double(values),
        }
    }

    /// Create a character/string attribute
    #[staticmethod]
    fn char(value: String) -> Self {
        AttributeData {
            inner: RustAttributeData::Char(value),
        }
    }

    /// Get the integer values (if this is an integer attribute)
    fn as_integer(&self) -> PyResult<Vec<i64>> {
        match &self.inner {
            RustAttributeData::Integer(values) => Ok(values.clone()),
            _ => Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Attribute is not an integer type"
            )),
        }
    }

    /// Get the double values (if this is a double attribute)
    fn as_double(&self) -> PyResult<Vec<f64>> {
        match &self.inner {
            RustAttributeData::Double(values) => Ok(values.clone()),
            _ => Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Attribute is not a double type"
            )),
        }
    }

    /// Get the string value (if this is a character attribute)
    fn as_char(&self) -> PyResult<String> {
        match &self.inner {
            RustAttributeData::Char(value) => Ok(value.clone()),
            _ => Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                "Attribute is not a character type"
            )),
        }
    }

    fn __repr__(&self) -> String {
        match &self.inner {
            RustAttributeData::Integer(values) => format!("AttributeData.Integer({:?})", values),
            RustAttributeData::Double(values) => format!("AttributeData.Double({:?})", values),
            RustAttributeData::Char(value) => format!("AttributeData.Char('{}')", value),
        }
    }
}

impl AttributeData {
    pub fn to_rust(&self) -> RustAttributeData {
        self.inner.clone()
    }

    pub fn from_rust(data: &RustAttributeData) -> Self {
        AttributeData {
            inner: data.clone(),
        }
    }
}

#[pymethods]
impl ExodusWriter {
    /// Write an attribute to an entity
    ///
    /// Args:
    ///     entity_type: Type of entity (ElemBlock, NodeSet, etc.)
    ///     entity_id: ID of the specific entity
    ///     name: Attribute name (max 32 characters)
    ///     attr_type: Type of attribute data
    ///     data: Attribute data to write
    ///
    /// Example:
    ///     >>> file.put_attribute(
    ///     ...     EntityType.ELEM_BLOCK,
    ///     ...     100,
    ///     ...     "material_id",
    ///     ...     AttributeType.INTEGER,
    ///     ...     AttributeData.integer([42])
    ///     ... )
    fn put_attribute(
        &mut self,
        entity_type: EntityType,
        entity_id: i64,
        name: String,
        attr_type: AttributeType,
        data: AttributeData,
    ) -> PyResult<()> {
        self.file_mut()?.put_attribute(
            entity_type.to_rust(),
            entity_id,
            name,
            attr_type.to_rust(),
            data.to_rust(),
        ).into_py()?;
        Ok(())
    }
}

#[pymethods]
impl ExodusAppender {
    /// Read an attribute (NOTE: Not available in Append mode)
    fn get_attribute(&self, _entity_type: EntityType, _entity_id: i64, _name: String) -> PyResult<AttributeData> {
        Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>(
            "get_attribute not available in Append mode - use ExodusReader instead"
        ))
    }

    /// Get attribute names (NOTE: Not available in Append mode)
    fn get_attribute_names(&self, _entity_type: EntityType, _entity_id: i64) -> PyResult<Vec<String>> {
        Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>(
            "get_attribute_names not available in Append mode - use ExodusReader instead"
        ))
    }
}

#[pymethods]
impl ExodusReader {
    /// Read an attribute from an entity
    ///
    /// Args:
    ///     entity_type: Type of entity
    ///     entity_id: ID of the entity
    ///     name: Attribute name
    ///
    /// Returns:
    ///     AttributeData containing the attribute value(s)
    ///
    /// Example:
    ///     >>> attr = file.get_attribute(EntityType.ELEM_BLOCK, 100, "material_id")
    ///     >>> material_id = attr.as_integer()[0]
    fn get_attribute(
        &self,
        entity_type: EntityType,
        entity_id: i64,
        name: String,
    ) -> PyResult<AttributeData> {
        let attr = self.file_ref().attribute(
            entity_type.to_rust(),
            entity_id,
            name,
        ).into_py()?;
        Ok(AttributeData::from_rust(&attr))
    }

    /// Get all attribute names for an entity
    ///
    /// Args:
    ///     entity_type: Type of entity
    ///     entity_id: ID of the entity
    ///
    /// Returns:
    ///     List of attribute names
    ///
    /// Example:
    ///     >>> names = file.get_attribute_names(EntityType.ELEM_BLOCK, 100)
    ///     >>> for name in names:
    ///     ...     attr = file.get_attribute(EntityType.ELEM_BLOCK, 100, name)
    fn get_attribute_names(
        &self,
        entity_type: EntityType,
        entity_id: i64,
    ) -> PyResult<Vec<String>> {
        self.file_ref().attribute_names(
            entity_type.to_rust(),
            entity_id,
        ).into_py()
    }
}
