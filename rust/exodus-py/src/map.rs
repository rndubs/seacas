//! ID map, naming, and property operations for Exodus files

use pyo3::prelude::*;
use crate::error::IntoPyResult as _;
use crate::file::{ExodusWriter, ExodusAppender, ExodusReader};
use exodus_rs::EntityType;

#[pymethods]
impl ExodusWriter {
    /// Write entity ID map
    ///
    /// Args:
    ///     entity_type (str): Type of entity ('node', 'elem', 'edge', or 'face')
    ///     id_map (list[int]): Array of entity IDs
    fn put_id_map(&mut self, entity_type: &str, id_map: Vec<i64>) -> PyResult<()> {
        let entity_type = match entity_type {
            "node" => EntityType::NodeMap,
            "elem" => EntityType::ElemMap,
            "edge" => EntityType::EdgeMap,
            "face" => EntityType::FaceMap,
            _ => return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Invalid entity_type: {}", entity_type)
            )),
        };
        let file = self.file.as_mut().ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("File has been closed"))?;
        file.put_id_map(entity_type, &id_map).into_py()
    }

    /// Write element order map
    ///
    /// Args:
    ///     order (list[int]): Array specifying element processing order
    fn put_elem_order_map(&mut self, order: Vec<i64>) -> PyResult<()> {
        let file = self.file.as_mut().ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("File has been closed"))?;
        file.put_elem_order_map(&order).into_py()
    }

    /// Set name for a single entity
    ///
    /// Args:
    ///     entity_type (str): Type of entity ('elem_block', 'node_set', 'side_set', etc.)
    ///     entity_index (int): 0-based index of the entity
    ///     name (str): Name to assign (max 32 characters)
    fn put_name(&mut self, entity_type: &str, entity_index: usize, name: &str) -> PyResult<()> {
        let entity_type = parse_entity_type(entity_type)?;
        let file = self.file.as_mut().ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("File has been closed"))?;
        file.put_name(entity_type, entity_index, name).into_py()
    }

    /// Set names for all entities of a type
    ///
    /// Args:
    ///     entity_type (str): Type of entity
    ///     names (list[str]): Array of names (max 32 characters each)
    fn put_names(&mut self, entity_type: &str, names: Vec<&str>) -> PyResult<()> {
        let entity_type = parse_entity_type(entity_type)?;
        let file = self.file.as_mut().ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("File has been closed"))?;
        file.put_names(entity_type, &names).into_py()
    }

    /// Set property value for a single entity
    ///
    /// Args:
    ///     entity_type (str): Type of entity
    ///     entity_id (int): Entity ID (not index)
    ///     prop_name (str): Property name
    ///     value (int): Property value
    fn put_property(&mut self, entity_type: &str, entity_id: i64, prop_name: &str, value: i64) -> PyResult<()> {
        let entity_type = parse_entity_type(entity_type)?;
        let file = self.file.as_mut().ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("File has been closed"))?;
        file.put_property(entity_type, entity_id, prop_name, value).into_py()
    }

    /// Set property array for all entities of a type
    ///
    /// Args:
    ///     entity_type (str): Type of entity
    ///     prop_name (str): Property name
    ///     values (list[int]): Array of property values
    fn put_property_array(&mut self, entity_type: &str, prop_name: &str, values: Vec<i64>) -> PyResult<()> {
        let entity_type = parse_entity_type(entity_type)?;
        let file = self.file.as_mut().ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("File has been closed"))?;
        file.put_property_array(entity_type, prop_name, &values).into_py()
    }
}

#[pymethods]
impl ExodusAppender {
    /// Read entity ID map (NOTE: Not available in Append mode)
    fn get_id_map(&self, _entity_type: &str) -> PyResult<Vec<i64>> {
        Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>(
            "get_id_map not available in Append mode - use ExodusReader instead"
        ))
    }

    /// Read element order map (NOTE: Not available in Append mode)
    fn get_elem_order_map(&self) -> PyResult<Vec<i64>> {
        Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>(
            "get_elem_order_map not available in Append mode - use ExodusReader instead"
        ))
    }

    /// Get name for a single entity (NOTE: Not available in Append mode)
    fn get_name(&self, _entity_type: &str, _entity_index: usize) -> PyResult<String> {
        Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>(
            "get_name not available in Append mode - use ExodusReader instead"
        ))
    }

    /// Get all names for entity type (NOTE: Not available in Append mode)
    fn get_names(&self, _entity_type: &str) -> PyResult<Vec<String>> {
        Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>(
            "get_names not available in Append mode - use ExodusReader instead"
        ))
    }

    /// Get property value for a single entity (NOTE: Not available in Append mode)
    fn get_property(&self, _entity_type: &str, _entity_id: i64, _prop_name: &str) -> PyResult<i64> {
        Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>(
            "get_property not available in Append mode - use ExodusReader instead"
        ))
    }

    /// Get property array for all entities of a type (NOTE: Not available in Append mode)
    fn get_property_array(&self, _entity_type: &str, _prop_name: &str) -> PyResult<Vec<i64>> {
        Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>(
            "get_property_array not available in Append mode - use ExodusReader instead"
        ))
    }

    /// Get all property names for an entity type (NOTE: Not available in Append mode)
    fn get_property_names(&self, _entity_type: &str) -> PyResult<Vec<String>> {
        Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>(
            "get_property_names not available in Append mode - use ExodusReader instead"
        ))
    }
}

#[pymethods]
impl ExodusReader {
    /// Read entity ID map
    ///
    /// Args:
    ///     entity_type (str): Type of entity ('node', 'elem', 'edge', or 'face')
    ///
    /// Returns:
    ///     list[int]: Entity ID map
    fn get_id_map(&self, entity_type: &str) -> PyResult<Vec<i64>> {
        let entity_type = match entity_type {
            "node" => EntityType::NodeMap,
            "elem" => EntityType::ElemMap,
            "edge" => EntityType::EdgeMap,
            "face" => EntityType::FaceMap,
            _ => return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Invalid entity_type: {}", entity_type)
            )),
        };
        self.file.id_map(entity_type).into_py()
    }

    /// Read element order map
    ///
    /// Returns:
    ///     list[int]: Element order map
    fn get_elem_order_map(&self) -> PyResult<Vec<i64>> {
        self.file.elem_order_map().into_py()
    }

    /// Get name for a single entity
    ///
    /// Args:
    ///     entity_type (str): Type of entity
    ///     entity_index (int): 0-based index of the entity
    ///
    /// Returns:
    ///     str: Entity name
    fn get_name(&self, entity_type: &str, entity_index: usize) -> PyResult<String> {
        let entity_type = parse_entity_type(entity_type)?;
        self.file.name(entity_type, entity_index).into_py()
    }

    /// Get all names for entity type
    ///
    /// Args:
    ///     entity_type (str): Type of entity
    ///
    /// Returns:
    ///     list[str]: Vector of entity names
    fn get_names(&self, entity_type: &str) -> PyResult<Vec<String>> {
        let entity_type = parse_entity_type(entity_type)?;
        self.file.names(entity_type).into_py()
    }

    /// Get property value for a single entity
    ///
    /// Args:
    ///     entity_type (str): Type of entity
    ///     entity_id (int): Entity ID (not index)
    ///     prop_name (str): Property name
    ///
    /// Returns:
    ///     int: Property value
    fn get_property(&self, entity_type: &str, entity_id: i64, prop_name: &str) -> PyResult<i64> {
        let entity_type = parse_entity_type(entity_type)?;
        self.file.property(entity_type, entity_id, prop_name).into_py()
    }

    /// Get property array for all entities of a type
    ///
    /// Args:
    ///     entity_type (str): Type of entity
    ///     prop_name (str): Property name
    ///
    /// Returns:
    ///     list[int]: Vector of property values
    fn get_property_array(&self, entity_type: &str, prop_name: &str) -> PyResult<Vec<i64>> {
        let entity_type = parse_entity_type(entity_type)?;
        self.file.property_array(entity_type, prop_name).into_py()
    }

    /// Get all property names for an entity type
    ///
    /// Args:
    ///     entity_type (str): Type of entity
    ///
    /// Returns:
    ///     list[str]: Vector of property names
    fn get_property_names(&self, entity_type: &str) -> PyResult<Vec<String>> {
        let entity_type = parse_entity_type(entity_type)?;
        self.file.property_names(entity_type).into_py()
    }
}

/// Helper function to parse entity type string
fn parse_entity_type(entity_type: &str) -> PyResult<EntityType> {
    match entity_type {
        "elem_block" => Ok(EntityType::ElemBlock),
        "edge_block" => Ok(EntityType::EdgeBlock),
        "face_block" => Ok(EntityType::FaceBlock),
        "node_set" => Ok(EntityType::NodeSet),
        "edge_set" => Ok(EntityType::EdgeSet),
        "face_set" => Ok(EntityType::FaceSet),
        "elem_set" => Ok(EntityType::ElemSet),
        "side_set" => Ok(EntityType::SideSet),
        "node_map" => Ok(EntityType::NodeMap),
        "edge_map" => Ok(EntityType::EdgeMap),
        "face_map" => Ok(EntityType::FaceMap),
        "elem_map" => Ok(EntityType::ElemMap),
        _ => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            format!("Invalid entity_type: {}", entity_type)
        )),
    }
}
