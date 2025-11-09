//! Set operations (node sets, side sets, etc.)

use pyo3::prelude::*;
use crate::error::IntoPyResult;
use crate::file::{ExodusWriter, ExodusAppender, ExodusReader};
use crate::types::{NodeSet, SideSet, EntitySet};

#[pymethods]
impl ExodusWriter {
    /// Write a node set
    fn put_node_set(&mut self, node_set: &NodeSet) -> PyResult<()> {
        self.file_mut()?.put_node_set(&node_set.to_rust()).into_py()?;
        Ok(())
    }

    /// Write a side set
    fn put_side_set(&mut self, side_set: &SideSet) -> PyResult<()> {
        self.file_mut()?.put_side_set(&side_set.to_rust()).into_py()?;
        Ok(())
    }

    /// Write an entity set (edge/face/elem set)
    fn put_entity_set(&mut self, entity_set: &EntitySet) -> PyResult<()> {
        self.file_mut()?.put_entity_set(&entity_set.to_rust()).into_py()?;
        Ok(())
    }
}

#[pymethods]
impl ExodusAppender {
    /// Write a node set
    fn put_node_set(&mut self, node_set: &NodeSet) -> PyResult<()> {
        self.file_mut()?.put_node_set(&node_set.to_rust()).into_py()?;
        Ok(())
    }

    /// Write a side set
    fn put_side_set(&mut self, side_set: &SideSet) -> PyResult<()> {
        self.file_mut()?.put_side_set(&side_set.to_rust()).into_py()?;
        Ok(())
    }

    /// Write an entity set
    fn put_entity_set(&mut self, entity_set: &EntitySet) -> PyResult<()> {
        self.file_mut()?.put_entity_set(&entity_set.to_rust()).into_py()?;
        Ok(())
    }

    /// Read a node set
    fn get_node_set(&self, set_id: i64) -> PyResult<NodeSet> {
        let ns = self.file.as_ref()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("File closed"))?
            .get_node_set(set_id)
            .into_py()?;
        Ok(NodeSet::from_rust(&ns))
    }

    /// Read a side set
    fn get_side_set(&self, set_id: i64) -> PyResult<SideSet> {
        let ss = self.file.as_ref()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("File closed"))?
            .get_side_set(set_id)
            .into_py()?;
        Ok(SideSet::from_rust(&ss))
    }

    /// Read an entity set
    fn get_entity_set(&self, set_id: i64) -> PyResult<EntitySet> {
        let es = self.file.as_ref()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("File closed"))?
            .get_entity_set(set_id)
            .into_py()?;
        Ok(EntitySet::from_rust(&es))
    }
}

#[pymethods]
impl ExodusReader {
    /// Read a node set
    fn get_node_set(&self, set_id: i64) -> PyResult<NodeSet> {
        let ns = self.file_ref().get_node_set(set_id).into_py()?;
        Ok(NodeSet::from_rust(&ns))
    }

    /// Read a side set
    fn get_side_set(&self, set_id: i64) -> PyResult<SideSet> {
        let ss = self.file_ref().get_side_set(set_id).into_py()?;
        Ok(SideSet::from_rust(&ss))
    }

    /// Read an entity set
    fn get_entity_set(&self, set_id: i64) -> PyResult<EntitySet> {
        let es = self.file_ref().get_entity_set(set_id).into_py()?;
        Ok(EntitySet::from_rust(&es))
    }

    /// Get all node set IDs
    fn get_node_set_ids(&self) -> PyResult<Vec<i64>> {
        self.file_ref().get_node_set_ids().into_py()
    }

    /// Get all side set IDs
    fn get_side_set_ids(&self) -> PyResult<Vec<i64>> {
        self.file_ref().get_side_set_ids().into_py()
    }
}
