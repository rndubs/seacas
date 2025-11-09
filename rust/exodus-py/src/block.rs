//! Block operations for Exodus files

use pyo3::prelude::*;
use crate::error::IntoPyResult;
use crate::file::{ExodusWriter, ExodusAppender, ExodusReader};
use crate::types::Block;

#[pymethods]
impl ExodusWriter {
    /// Write a block definition
    ///
    /// Args:
    ///     block: Block object defining the block parameters
    ///
    /// Example:
    ///     >>> block = Block(
    ///     ...     id=100,
    ///     ...     entity_type=EntityType.ELEM_BLOCK,
    ///     ...     topology="HEX8",
    ///     ...     num_entries=10,
    ///     ...     num_nodes_per_entry=8
    ///     ... )
    ///     >>> writer.put_block(block)
    fn put_block(&mut self, block: &Block) -> PyResult<()> {
        self.file_mut()?.put_block(&block.to_rust()).into_py()?;
        Ok(())
    }

    /// Write element connectivity for a block
    ///
    /// Args:
    ///     block_id: Block ID
    ///     connectivity: Flat array of node IDs (1-based)
    ///
    /// Example:
    ///     >>> # One hex element with 8 nodes
    ///     >>> writer.put_connectivity(100, [1, 2, 3, 4, 5, 6, 7, 8])
    fn put_connectivity(&mut self, block_id: i64, connectivity: Vec<i64>) -> PyResult<()> {
        self.file_mut()?
            .put_connectivity(block_id, &connectivity)
            .into_py()?;
        Ok(())
    }

    /// Write block attributes
    ///
    /// Args:
    ///     block_id: Block ID
    ///     attributes: Flat array of attribute values
    fn put_block_attributes(&mut self, block_id: i64, attributes: Vec<f64>) -> PyResult<()> {
        self.file_mut()?
            .put_block_attributes(block_id, &attributes)
            .into_py()?;
        Ok(())
    }

    /// Write block attribute names
    ///
    /// Args:
    ///     block_id: Block ID
    ///     names: List of attribute names
    fn put_block_attribute_names(&mut self, block_id: i64, names: Vec<String>) -> PyResult<()> {
        let name_refs: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        self.file_mut()?
            .put_block_attribute_names(block_id, &name_refs)
            .into_py()?;
        Ok(())
    }
}

#[pymethods]
impl ExodusAppender {
    /// Read a block definition (NOTE: Not fully available in Append mode)
    fn get_block(&self, _block_id: i64) -> PyResult<Block> {
        Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>(
            "get_block not fully available in Append mode - use ExodusReader instead"
        ))
    }

    /// Read element connectivity (NOTE: Not available in Append mode)
    fn get_connectivity(&self, _block_id: i64) -> PyResult<Vec<i64>> {
        Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>(
            "get_connectivity not available in Append mode - use ExodusReader instead"
        ))
    }

    /// Read block IDs
    fn get_block_ids(&self) -> PyResult<Vec<i64>> {
        use exodus_rs::types::EntityType;
        self.file_ref()?.block_ids(EntityType::ElemBlock).into_py()
    }
}

#[pymethods]
impl ExodusReader {
    /// Read a block definition
    fn get_block(&self, block_id: i64) -> PyResult<Block> {
        let block = self.file_ref().block(block_id).into_py()?;
        Ok(Block::from_rust(&block))
    }

    /// Read element connectivity
    fn get_connectivity(&self, block_id: i64) -> PyResult<Vec<i64>> {
        self.file_ref().connectivity(block_id).into_py()
    }

    /// Read all block IDs
    fn get_block_ids(&self) -> PyResult<Vec<i64>> {
        use exodus_rs::types::EntityType;
        self.file_ref().block_ids(EntityType::ElemBlock).into_py()
    }

    /// Read block attributes
    fn get_block_attributes(&self, block_id: i64) -> PyResult<Vec<f64>> {
        self.file_ref().block_attributes(block_id).into_py()
    }

    /// Read block attribute names
    fn get_block_attribute_names(&self, block_id: i64) -> PyResult<Vec<String>> {
        self.file_ref().block_attribute_names(block_id).into_py()
    }
}
