//! Block operations for Exodus files

use pyo3::prelude::*;
use crate::error::IntoPyResult;
use crate::file::{ExodusWriter, ExodusAppender, ExodusReader};
use crate::types::Block;
use crate::numpy_utils::{extract_i64_vec, extract_f64_vec, vec_to_numpy_i64, vec_to_numpy_f64};

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
    ///     connectivity: Flat array of node IDs (1-based) - accepts list or NumPy array
    ///
    /// Example:
    ///     >>> # Using lists
    ///     >>> writer.put_connectivity(100, [1, 2, 3, 4, 5, 6, 7, 8])
    ///     >>> # Using NumPy
    ///     >>> import numpy as np
    ///     >>> writer.put_connectivity(100, np.array([1, 2, 3, 4, 5, 6, 7, 8]))
    fn put_connectivity(&mut self, py: Python<'_>, block_id: i64, connectivity: &Bound<'_, PyAny>) -> PyResult<()> {
        let connectivity_vec = extract_i64_vec(py, connectivity)?;
        self.file_mut()?
            .put_connectivity(block_id, &connectivity_vec)
            .into_py()?;
        Ok(())
    }

    /// Write block attributes
    ///
    /// Args:
    ///     block_id: Block ID
    ///     attributes: Flat array of attribute values - accepts list or NumPy array
    fn put_block_attributes(&mut self, py: Python<'_>, block_id: i64, attributes: &Bound<'_, PyAny>) -> PyResult<()> {
        let attributes_vec = extract_f64_vec(py, attributes)?;
        self.file_mut()?
            .put_block_attributes(block_id, &attributes_vec)
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
    ///
    /// Returns:
    ///     NumPy array of node IDs (1-based)
    #[cfg(feature = "numpy")]
    fn get_connectivity<'py>(&self, py: Python<'py>, block_id: i64) -> PyResult<Bound<'py, numpy::PyArray1<i64>>> {
        let vec = self.file_ref().connectivity(block_id).into_py()?;
        Ok(vec_to_numpy_i64(py, vec))
    }

    /// Read element connectivity
    #[cfg(not(feature = "numpy"))]
    fn get_connectivity(&self, block_id: i64) -> PyResult<Vec<i64>> {
        self.file_ref().connectivity(block_id).into_py()
    }

    /// Read all block IDs
    ///
    /// Returns:
    ///     NumPy array of block IDs
    #[cfg(feature = "numpy")]
    fn get_block_ids<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, numpy::PyArray1<i64>>> {
        use exodus_rs::types::EntityType;
        let vec = self.file_ref().block_ids(EntityType::ElemBlock).into_py()?;
        Ok(vec_to_numpy_i64(py, vec))
    }

    /// Read all block IDs
    #[cfg(not(feature = "numpy"))]
    fn get_block_ids(&self) -> PyResult<Vec<i64>> {
        use exodus_rs::types::EntityType;
        self.file_ref().block_ids(EntityType::ElemBlock).into_py()
    }

    /// Read block attributes
    ///
    /// Returns:
    ///     NumPy array of attribute values
    #[cfg(feature = "numpy")]
    fn get_block_attributes<'py>(&self, py: Python<'py>, block_id: i64) -> PyResult<Bound<'py, numpy::PyArray1<f64>>> {
        let vec = self.file_ref().block_attributes(block_id).into_py()?;
        Ok(vec_to_numpy_f64(py, vec))
    }

    /// Read block attributes
    #[cfg(not(feature = "numpy"))]
    fn get_block_attributes(&self, block_id: i64) -> PyResult<Vec<f64>> {
        self.file_ref().block_attributes(block_id).into_py()
    }

    /// Read block attribute names
    fn get_block_attribute_names(&self, block_id: i64) -> PyResult<Vec<String>> {
        self.file_ref().block_attribute_names(block_id).into_py()
    }
}
