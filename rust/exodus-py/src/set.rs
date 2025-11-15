//! Set operations for Exodus files

use pyo3::prelude::*;
use crate::error::IntoPyResult;
use crate::file::{ExodusWriter, ExodusReader};
use crate::types::{NodeSet, SideSet, EntitySet, EntityType};

#[pymethods]
impl ExodusWriter {
    /// Define a set
    ///
    /// This creates the NetCDF dimensions and variables for a set.
    ///
    /// Args:
    ///     entity_type: Type of set (NodeSet, EdgeSet, FaceSet, ElemSet, or SideSet)
    ///     set_id: Unique identifier for the set
    ///     num_entries: Number of entries in the set
    ///     num_dist_factors: Number of distribution factors (0 if none)
    ///
    /// Raises:
    ///     ExodusError: If the set cannot be defined
    fn put_set(
        &mut self,
        entity_type: &EntityType,
        set_id: i64,
        num_entries: usize,
        num_dist_factors: usize,
    ) -> PyResult<()> {
        let set = exodus_rs::Set {
            id: set_id,
            entity_type: entity_type.to_rust(),
            num_entries,
            num_dist_factors,
        };
        self.file_mut()?.put_set(&set).into_py()
    }

    /// Write a node set
    ///
    /// Args:
    ///     set_id: ID of the set
    ///     nodes: List of node IDs in the set
    ///     dist_factors: Optional list of distribution factors (one per node)
    ///
    /// Raises:
    ///     ExodusError: If the set has not been defined or write fails
    ///
    /// Example:
    ///     >>> # Define the set first
    ///     >>> exo.put_set(EntityType.NODE_SET, 10, 5, 0)
    ///     >>> # Write the node set data
    ///     >>> exo.put_node_set(10, [1, 2, 3, 4, 5], None)
    fn put_node_set(
        &mut self,
        set_id: i64,
        nodes: Vec<i64>,
        dist_factors: Option<Vec<f64>>,
    ) -> PyResult<()> {
        let df = dist_factors.as_deref();
        self.file_mut()?.put_node_set(set_id, &nodes, df).into_py()
    }

    /// Write a side set
    ///
    /// Args:
    ///     set_id: ID of the set
    ///     elements: List of element IDs that define the sides
    ///     sides: List of side numbers within each element (topology dependent)
    ///     dist_factors: Optional list of distribution factors (one per node-on-side)
    ///
    /// Raises:
    ///     ExodusError: If the set has not been defined or write fails
    ///
    /// Example:
    ///     >>> # Define the set first
    ///     >>> exo.put_set(EntityType.SIDE_SET, 100, 2, 0)
    ///     >>> # Write the side set data
    ///     >>> exo.put_side_set(100, [1, 2], [3, 4], None)
    fn put_side_set(
        &mut self,
        set_id: i64,
        elements: Vec<i64>,
        sides: Vec<i64>,
        dist_factors: Option<Vec<f64>>,
    ) -> PyResult<()> {
        let df = dist_factors.as_deref();
        self.file_mut()?.put_side_set(set_id, &elements, &sides, df).into_py()
    }

    /// Write an entity set (edge, face, or element set)
    ///
    /// Args:
    ///     entity_type: Type of set (EdgeSet, FaceSet, or ElemSet)
    ///     set_id: ID of the set
    ///     entities: List of entity IDs in the set
    ///
    /// Raises:
    ///     ExodusError: If the set has not been defined or write fails
    ///
    /// Example:
    ///     >>> # Define the set first
    ///     >>> exo.put_set(EntityType.ELEM_SET, 500, 5, 0)
    ///     >>> # Write the element set data
    ///     >>> exo.put_entity_set(EntityType.ELEM_SET, 500, [2, 4, 6, 8, 10])
    fn put_entity_set(
        &mut self,
        entity_type: &EntityType,
        set_id: i64,
        entities: Vec<i64>,
    ) -> PyResult<()> {
        self.file_mut()?.put_entity_set(entity_type.to_rust(), set_id, &entities).into_py()
    }

    /// Get all set IDs of a given type
    ///
    /// Args:
    ///     entity_type: Type of set (NodeSet, EdgeSet, FaceSet, ElemSet, or SideSet)
    ///
    /// Returns:
    ///     List of set IDs
    fn get_set_ids(&self, entity_type: &EntityType) -> PyResult<Vec<i64>> {
        self.file_ref()?.set_ids(entity_type.to_rust()).into_py()
    }
}

// Note: ExodusAppender does not have set operations implemented yet in the Rust library.
// Set operations are only available in Write and Read modes.

#[pymethods]
impl ExodusReader {
    /// Read a node set
    ///
    /// Args:
    ///     set_id: ID of the set to read
    ///
    /// Returns:
    ///     NodeSet object containing node IDs and distribution factors
    ///
    /// Example:
    ///     >>> node_set = exo.get_node_set(10)
    ///     >>> print(f"Nodes: {node_set.nodes}")
    ///     >>> print(f"Distribution factors: {node_set.dist_factors}")
    fn get_node_set(&self, set_id: i64) -> PyResult<NodeSet> {
        let node_set = self.file_ref().node_set(set_id).into_py()?;
        Ok(NodeSet {
            id: node_set.id,
            nodes: node_set.nodes,
            dist_factors: node_set.dist_factors,
        })
    }

    /// Read a side set
    ///
    /// Args:
    ///     set_id: ID of the set to read
    ///
    /// Returns:
    ///     SideSet object containing element-side pairs and distribution factors
    fn get_side_set(&self, set_id: i64) -> PyResult<SideSet> {
        let side_set = self.file_ref().side_set(set_id).into_py()?;
        Ok(SideSet {
            id: side_set.id,
            elements: side_set.elements,
            sides: side_set.sides,
            dist_factors: side_set.dist_factors,
        })
    }

    /// Read an entity set (edge, face, or element set)
    ///
    /// Args:
    ///     entity_type: Type of set (EdgeSet, FaceSet, or ElemSet)
    ///     set_id: ID of the set to read
    ///
    /// Returns:
    ///     EntitySet object containing entity IDs
    fn get_entity_set(&self, entity_type: &EntityType, set_id: i64) -> PyResult<EntitySet> {
        let entity_set = self.file_ref().entity_set(entity_type.to_rust(), set_id).into_py()?;
        Ok(EntitySet {
            id: entity_set.id,
            entity_type: EntityType::from_rust(entity_set.entity_type),
            entities: entity_set.entities,
        })
    }

    /// Get node set IDs
    ///
    /// Returns:
    ///     List of all node set IDs
    fn get_node_set_ids(&self) -> PyResult<Vec<i64>> {
        self.file_ref().set_ids(exodus_rs::EntityType::NodeSet).into_py()
    }

    /// Get side set IDs
    ///
    /// Returns:
    ///     List of all side set IDs
    fn get_side_set_ids(&self) -> PyResult<Vec<i64>> {
        self.file_ref().set_ids(exodus_rs::EntityType::SideSet).into_py()
    }

    /// Get element set IDs
    ///
    /// Returns:
    ///     List of all element set IDs
    fn get_elem_set_ids(&self) -> PyResult<Vec<i64>> {
        self.file_ref().set_ids(exodus_rs::EntityType::ElemSet).into_py()
    }

    /// Get all set IDs of a given type
    ///
    /// Args:
    ///     entity_type: Type of set
    ///
    /// Returns:
    ///     List of set IDs
    fn get_set_ids(&self, entity_type: &EntityType) -> PyResult<Vec<i64>> {
        self.file_ref().set_ids(entity_type.to_rust()).into_py()
    }
}
