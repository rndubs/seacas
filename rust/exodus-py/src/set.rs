//! Set operations for Exodus files

use pyo3::prelude::*;
use crate::error::IntoPyResult;
use crate::file::{ExodusWriter, ExodusReader, ExodusAppender};
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
    #[pyo3(signature = (set_id, nodes, dist_factors=None))]
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
    #[pyo3(signature = (set_id, elements, sides, dist_factors=None))]
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

    /// Convert a nodeset to a sideset with explicit ID
    ///
    /// Creates a sideset containing all element faces where every node belongs
    /// to the specified nodeset. Only boundary faces (faces appearing in exactly
    /// one element) are included, and face normals are verified to point outward
    /// from the mesh center of mass.
    ///
    /// For automatic ID assignment, use `convert_nodeset_to_sideset_auto()`.
    ///
    /// Args:
    ///     nodeset_id: ID of the existing nodeset
    ///     new_sideset_id: ID for the new sideset
    ///
    /// Returns:
    ///     SideSet object containing element IDs and side numbers
    ///
    /// Warnings are printed to stderr for:
    ///     - Empty nodeset
    ///     - No boundary faces found
    ///     - Inward-pointing normals
    ///     - Inconsistent normal directions
    ///
    /// Example:
    ///     >>> reader = ExodusReader.open("mesh.exo")
    ///     >>> # Convert nodeset 10 to sideset 100
    ///     >>> sideset = reader.convert_nodeset_to_sideset(10, 100)
    ///     >>> print(f"Created sideset {sideset.id} with {len(sideset.elements)} faces")
    fn convert_nodeset_to_sideset(
        &self,
        nodeset_id: i64,
        new_sideset_id: i64,
    ) -> PyResult<SideSet> {
        let side_set = self.file_ref()
            .convert_nodeset_to_sideset(nodeset_id, new_sideset_id)
            .into_py()?;
        Ok(SideSet {
            id: side_set.id,
            elements: side_set.elements,
            sides: side_set.sides,
            dist_factors: side_set.dist_factors,
        })
    }

    /// Convert a nodeset to a sideset with auto-assigned ID
    ///
    /// This is the recommended method for most use cases. The sideset ID is automatically
    /// assigned as one greater than the maximum existing sideset ID (or 1 if no sidesets exist).
    ///
    /// Args:
    ///     nodeset_id: ID of the existing nodeset
    ///
    /// Returns:
    ///     SideSet object with auto-assigned ID
    ///
    /// Example:
    ///     >>> reader = ExodusReader.open("mesh.exo")
    ///     >>> sideset = reader.convert_nodeset_to_sideset_auto(10)
    ///     >>> print(f"Created sideset {sideset.id} with {len(sideset.elements)} faces")
    fn convert_nodeset_to_sideset_auto(&self, nodeset_id: i64) -> PyResult<SideSet> {
        let side_set = self.file_ref()
            .convert_nodeset_to_sideset_auto(nodeset_id)
            .into_py()?;
        Ok(SideSet {
            id: side_set.id,
            elements: side_set.elements,
            sides: side_set.sides,
            dist_factors: side_set.dist_factors,
        })
    }

    /// Convert a nodeset to a sideset using entity names
    ///
    /// Creates a sideset from a nodeset, looking up the nodeset by name rather than ID.
    /// The new sideset ID is automatically assigned.
    ///
    /// Args:
    ///     nodeset_name: Name of the existing nodeset
    ///
    /// Returns:
    ///     SideSet object with auto-assigned ID
    ///
    /// Example:
    ///     >>> reader = ExodusReader.open("mesh.exo")
    ///     >>> sideset = reader.convert_nodeset_to_sideset_by_name("inlet")
    ///     >>> print(f"Created sideset {sideset.id} with {len(sideset.elements)} faces")
    fn convert_nodeset_to_sideset_by_name(&self, nodeset_name: &str) -> PyResult<SideSet> {
        let side_set = self.file_ref()
            .convert_nodeset_to_sideset_by_name(nodeset_name)
            .into_py()?;
        Ok(SideSet {
            id: side_set.id,
            elements: side_set.elements,
            sides: side_set.sides,
            dist_factors: side_set.dist_factors,
        })
    }

    /// Convert a nodeset to a sideset with explicit name for the new sideset
    ///
    /// Creates a sideset from a nodeset, with automatic ID assignment. Returns
    /// the sideset data along with the assigned ID and name.
    ///
    /// **Note:** This only creates the sideset data structure. To write it to a file,
    /// use ExodusAppender.
    ///
    /// Args:
    ///     nodeset_id: ID of the existing nodeset
    ///     sideset_name: Desired name for the new sideset
    ///
    /// Returns:
    ///     Tuple of (assigned_sideset_id, sideset_name, sideset_data)
    ///
    /// Example:
    ///     >>> reader = ExodusReader.open("mesh.exo")
    ///     >>> ss_id, ss_name, sideset = reader.convert_nodeset_to_sideset_named(10, "outlet")
    ///     >>> print(f"Created sideset '{ss_name}' with ID {ss_id}")
    fn convert_nodeset_to_sideset_named(
        &self,
        nodeset_id: i64,
        sideset_name: &str,
    ) -> PyResult<(i64, String, SideSet)> {
        let (id, name, side_set) = self.file_ref()
            .convert_nodeset_to_sideset_named(nodeset_id, sideset_name)
            .into_py()?;
        Ok((
            id,
            name,
            SideSet {
                id: side_set.id,
                elements: side_set.elements,
                sides: side_set.sides,
                dist_factors: side_set.dist_factors,
            },
        ))
    }
}

#[pymethods]
impl ExodusAppender {
    /// Convert a nodeset to a sideset and write it to the file with explicit ID
    ///
    /// This is a convenience method that combines reading the nodeset, converting it
    /// to a sideset based on boundary faces, and writing the result to the file.
    ///
    /// For automatic ID assignment, use `create_sideset_from_nodeset_auto()`.
    ///
    /// Only boundary faces (faces appearing in exactly one element) are included,
    /// and face normals are verified to point outward from the mesh center of mass.
    ///
    /// Args:
    ///     nodeset_id: ID of the existing nodeset
    ///     new_sideset_id: ID for the new sideset
    ///
    /// Warnings are printed to stderr for:
    ///     - Empty nodeset
    ///     - No boundary faces found
    ///     - Inward-pointing normals
    ///     - Inconsistent normal directions
    ///
    /// Example:
    ///     >>> appender = ExodusAppender.append("mesh.exo")
    ///     >>> # Convert nodeset 10 to sideset 100 and write it
    ///     >>> appender.create_sideset_from_nodeset(10, 100)
    ///     >>> appender.close()
    fn create_sideset_from_nodeset(
        &mut self,
        nodeset_id: i64,
        new_sideset_id: i64,
    ) -> PyResult<()> {
        self.file_mut()?
            .create_sideset_from_nodeset(nodeset_id, new_sideset_id)
            .into_py()
    }

    /// Convert a nodeset to a sideset with auto-assigned ID and write it to the file
    ///
    /// This is the recommended method for most use cases. The sideset ID is automatically
    /// assigned as one greater than the maximum existing sideset ID (or 1 if no sidesets exist).
    ///
    /// Args:
    ///     nodeset_id: ID of the existing nodeset
    ///
    /// Returns:
    ///     The ID that was assigned to the new sideset
    ///
    /// Example:
    ///     >>> appender = ExodusAppender.append("mesh.exo")
    ///     >>> # Convert nodeset 10 to a sideset with auto-assigned ID
    ///     >>> sideset_id = appender.create_sideset_from_nodeset_auto(10)
    ///     >>> print(f"Created sideset with ID {sideset_id}")
    ///     >>> appender.close()
    fn create_sideset_from_nodeset_auto(&mut self, nodeset_id: i64) -> PyResult<i64> {
        self.file_mut()?
            .create_sideset_from_nodeset_auto(nodeset_id)
            .into_py()
    }

    /// Convert a nodeset to a sideset by name and write it to the file
    ///
    /// This method looks up the nodeset by name, converts it to a sideset with
    /// auto-assigned ID, and writes it to the file. The nodeset's name is copied
    /// to the new sideset.
    ///
    /// Args:
    ///     nodeset_name: Name of the existing nodeset
    ///
    /// Returns:
    ///     The ID that was assigned to the new sideset
    ///
    /// Example:
    ///     >>> appender = ExodusAppender.append("mesh.exo")
    ///     >>> # Convert nodeset named "inlet" to a sideset
    ///     >>> sideset_id = appender.create_sideset_from_nodeset_by_name("inlet")
    ///     >>> print(f"Created sideset with ID {sideset_id}")
    ///     >>> appender.close()
    fn create_sideset_from_nodeset_by_name(&mut self, nodeset_name: &str) -> PyResult<i64> {
        self.file_mut()?
            .create_sideset_from_nodeset_by_name(nodeset_name)
            .into_py()
    }

    /// Convert a nodeset to a sideset with explicit name and write it to the file
    ///
    /// Creates a sideset from a nodeset with auto-assigned ID and writes both the
    /// sideset data and its name to the file.
    ///
    /// Args:
    ///     nodeset_id: ID of the existing nodeset
    ///     sideset_name: Name to assign to the new sideset
    ///
    /// Returns:
    ///     The ID that was assigned to the new sideset
    ///
    /// Example:
    ///     >>> appender = ExodusAppender.append("mesh.exo")
    ///     >>> # Convert nodeset 10 to a sideset named "outlet"
    ///     >>> sideset_id = appender.create_sideset_from_nodeset_named(10, "outlet")
    ///     >>> print(f"Created sideset 'outlet' with ID {sideset_id}")
    ///     >>> appender.close()
    fn create_sideset_from_nodeset_named(
        &mut self,
        nodeset_id: i64,
        sideset_name: &str,
    ) -> PyResult<i64> {
        self.file_mut()?
            .create_sideset_from_nodeset_named(nodeset_id, sideset_name)
            .into_py()
    }
}
