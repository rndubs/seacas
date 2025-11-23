//! Block operations for Exodus files

use crate::error::IntoPyResult;
use crate::file::{ExodusAppender, ExodusReader, ExodusWriter};
use crate::types::Block;
use pyo3::prelude::*;

#[cfg(feature = "numpy")]
use numpy::{PyArray1, PyArray2, PyArrayMethods};

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

    /// Write element connectivity for a block (accepts NumPy arrays or lists)
    ///
    /// Args:
    ///     block_id: Block ID
    ///     connectivity: Flat array of node IDs (1-based) as NumPy array or list
    ///
    /// Example:
    ///     >>> import numpy as np
    ///     >>> # One hex element with 8 nodes
    ///     >>> writer.put_connectivity(100, np.array([1, 2, 3, 4, 5, 6, 7, 8]))
    #[cfg(feature = "numpy")]
    fn put_connectivity(
        &mut self,
        _py: Python<'_>,
        block_id: i64,
        connectivity: Bound<'_, PyAny>,
    ) -> PyResult<()> {
        // Convert NumPy array or list to Vec
        let conn_vec = if let Ok(arr) = connectivity.clone().cast_into::<PyArray1<i64>>() {
            arr.readonly().as_slice()?.to_vec()
        } else if let Ok(arr) = connectivity.clone().cast_into::<PyArray2<i64>>() {
            // Flatten 2D array to 1D
            arr.readonly().as_slice()?.to_vec()
        } else {
            connectivity.extract::<Vec<i64>>()?
        };

        self.file_mut()?
            .put_connectivity(block_id, &conn_vec)
            .into_py()?;
        Ok(())
    }

    /// Write element connectivity for a block (no NumPy)
    #[cfg(not(feature = "numpy"))]
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
            "get_block not fully available in Append mode - use ExodusReader instead",
        ))
    }

    /// Read element connectivity (NOTE: Not available in Append mode)
    fn get_connectivity(&self, _block_id: i64) -> PyResult<Vec<i64>> {
        Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>(
            "get_connectivity not available in Append mode - use ExodusReader instead",
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

    /// Read element connectivity as 2D NumPy array
    ///
    /// Returns:
    ///     2D NumPy array with shape (num_elements, nodes_per_element)
    ///
    /// Example:
    ///     >>> conn = reader.get_connectivity(100)
    ///     >>> print(conn.shape)  # (num_elements, nodes_per_elem)
    #[cfg(feature = "numpy")]
    fn get_connectivity<'py>(
        &self,
        py: Python<'py>,
        block_id: i64,
    ) -> PyResult<Bound<'py, PyArray2<i64>>> {
        // Use optimized connectivity_array() method from Rust
        let arr = self.file_ref().connectivity_array(block_id).into_py()?;

        // Convert to NumPy array (zero-copy transfer)
        Ok(PyArray2::from_owned_array(py, arr))
    }

    /// Read element connectivity as flat list (deprecated)
    ///
    /// .. deprecated::
    ///     Use :meth:`get_connectivity` instead for better performance with NumPy arrays
    fn get_connectivity_list(&self, block_id: i64) -> PyResult<Vec<i64>> {
        self.file_ref().connectivity(block_id).into_py()
    }

    /// Read element connectivity (no NumPy fallback)
    #[cfg(not(feature = "numpy"))]
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
