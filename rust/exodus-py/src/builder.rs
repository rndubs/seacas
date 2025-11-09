//! High-level builder API for creating Exodus meshes

use pyo3::prelude::*;
use exodus_rs::{MeshBuilder as RustMeshBuilder, BlockBuilder as RustBlockBuilder};
use crate::error::IntoPyResult;
use crate::types::CreateOptions;

/// Python wrapper for BlockBuilder
///
/// Provides a fluent interface for building element blocks.
///
/// Example:
///     >>> block = (BlockBuilder(1, "HEX8")
///     ...     .connectivity([1, 2, 3, 4, 5, 6, 7, 8])
///     ...     .attributes([100.0])
///     ...     .attribute_names(["MaterialID"])
///     ...     .build())
#[pyclass]
#[derive(Clone)]
pub struct BlockBuilder {
    builder: RustBlockBuilder,
}

#[pymethods]
impl BlockBuilder {
    /// Create a new block builder
    ///
    /// Args:
    ///     id: Block ID (must be unique)
    ///     topology: Element topology (e.g., "HEX8", "QUAD4", "TET4")
    ///
    /// Example:
    ///     >>> builder = BlockBuilder(100, "HEX8")
    #[new]
    fn new(id: i64, topology: String) -> Self {
        BlockBuilder {
            builder: RustBlockBuilder::new(id, topology),
        }
    }

    /// Set element connectivity
    ///
    /// Args:
    ///     conn: Flat array of node IDs (1-based)
    ///           Length must be num_elements * nodes_per_element
    ///
    /// Returns:
    ///     Self for method chaining
    ///
    /// Example:
    ///     >>> builder.connectivity([1, 2, 3, 4, 5, 6, 7, 8])
    fn connectivity(mut slf: PyRefMut<'_, Self>, conn: Vec<i64>) -> PyRefMut<'_, Self> {
        slf.builder = slf.builder.clone().connectivity(conn);
        slf
    }

    /// Set element attributes
    ///
    /// Args:
    ///     attrs: Flat array of attribute values
    ///            Length must be num_elements * num_attributes_per_element
    ///
    /// Returns:
    ///     Self for method chaining
    fn attributes(mut slf: PyRefMut<'_, Self>, attrs: Vec<f64>) -> PyRefMut<'_, Self> {
        slf.builder = slf.builder.clone().attributes(attrs);
        slf
    }

    /// Set attribute names
    ///
    /// Args:
    ///     names: List of attribute names
    ///
    /// Returns:
    ///     Self for method chaining
    fn attribute_names(mut slf: PyRefMut<'_, Self>, names: Vec<String>) -> PyRefMut<'_, Self> {
        slf.builder = slf.builder.clone().attribute_names(names);
        slf
    }

    /// Build and return the block builder
    ///
    /// This is a no-op in Python (for API compatibility with Rust),
    /// but allows for consistent fluent interface.
    ///
    /// Returns:
    ///     Self
    fn build(slf: PyRefMut<'_, Self>) -> PyRefMut<'_, Self> {
        slf
    }

    fn __repr__(&self) -> String {
        format!("BlockBuilder({:?})", self.builder)
    }
}

/// Python wrapper for MeshBuilder
///
/// Provides a fluent interface for creating complete Exodus meshes.
///
/// Example:
///     >>> (MeshBuilder("My Mesh")
///     ...     .dimensions(3)
///     ...     .coordinates(x, y, z)
///     ...     .add_block(BlockBuilder(1, "HEX8").connectivity(conn).build())
///     ...     .write("mesh.exo"))
#[pyclass]
pub struct MeshBuilder {
    builder: RustMeshBuilder,
}

#[pymethods]
impl MeshBuilder {
    /// Create a new mesh builder
    ///
    /// Args:
    ///     title: Mesh title (max 80 characters)
    ///
    /// Example:
    ///     >>> builder = MeshBuilder("Test Mesh")
    #[new]
    fn new(title: String) -> Self {
        MeshBuilder {
            builder: RustMeshBuilder::new(title),
        }
    }

    /// Set the number of spatial dimensions
    ///
    /// Args:
    ///     num_dim: Number of dimensions (1, 2, or 3)
    ///
    /// Returns:
    ///     Self for method chaining
    ///
    /// Example:
    ///     >>> builder.dimensions(3)
    fn dimensions(mut slf: PyRefMut<'_, Self>, num_dim: usize) -> PyRefMut<'_, Self> {
        slf.builder = slf.builder.clone().dimensions(num_dim);
        slf
    }

    /// Set nodal coordinates
    ///
    /// Args:
    ///     x: X coordinates (required)
    ///     y: Y coordinates (required for 2D/3D, can be empty for 1D)
    ///     z: Z coordinates (required for 3D, can be empty for 1D/2D)
    ///
    /// The coordinate arrays must all have the same length (number of nodes).
    ///
    /// Returns:
    ///     Self for method chaining
    ///
    /// Example:
    ///     >>> builder.coordinates(
    ///     ...     x=[0.0, 1.0, 1.0, 0.0],
    ///     ...     y=[0.0, 0.0, 1.0, 1.0],
    ///     ...     z=[]
    ///     ... )
    #[pyo3(signature = (x, y=Vec::new(), z=Vec::new()))]
    fn coordinates(
        mut slf: PyRefMut<'_, Self>,
        x: Vec<f64>,
        y: Vec<f64>,
        z: Vec<f64>,
    ) -> PyRefMut<'_, Self> {
        slf.builder = slf.builder.clone().coordinates(x, y, z);
        slf
    }

    /// Add an element block
    ///
    /// Args:
    ///     block: BlockBuilder instance
    ///
    /// Returns:
    ///     Self for method chaining
    ///
    /// Example:
    ///     >>> builder.add_block(
    ///     ...     BlockBuilder(1, "HEX8")
    ///     ...         .connectivity([1, 2, 3, 4, 5, 6, 7, 8])
    ///     ...         .build()
    ///     ... )
    fn add_block<'a>(mut slf: PyRefMut<'a, Self>, block: &BlockBuilder) -> PyRefMut<'a, Self> {
        slf.builder = slf.builder.clone().add_block(block.builder.clone());
        slf
    }

    /// Add a QA record for provenance tracking
    ///
    /// Args:
    ///     code_name: Name of the code/application
    ///     version: Version string
    ///     date: Date string (e.g., "2025-01-15")
    ///     time: Time string (e.g., "14:30:00")
    ///
    /// Returns:
    ///     Self for method chaining
    ///
    /// Example:
    ///     >>> builder.qa_record("MyApp", "1.0.0", "2025-01-15", "14:30:00")
    fn qa_record(
        mut slf: PyRefMut<'_, Self>,
        code_name: String,
        version: String,
        date: String,
        time: String,
    ) -> PyRefMut<'_, Self> {
        slf.builder = slf.builder.clone().qa_record(code_name, version, date, time);
        slf
    }

    /// Add an information record
    ///
    /// Information records are arbitrary text strings (max 80 characters each).
    ///
    /// Args:
    ///     info: Info string
    ///
    /// Returns:
    ///     Self for method chaining
    ///
    /// Example:
    ///     >>> builder.info("Generated by mesh builder")
    fn info(mut slf: PyRefMut<'_, Self>, info: String) -> PyRefMut<'_, Self> {
        slf.builder = slf.builder.clone().info(info);
        slf
    }

    /// Write the mesh to a file
    ///
    /// This method creates the Exodus file, initializes it with the mesh
    /// parameters, and writes all the data.
    ///
    /// Args:
    ///     path: Output file path
    ///
    /// Example:
    ///     >>> builder.write("output.exo")
    fn write(&self, path: String) -> PyResult<()> {
        self.builder.clone().write(path).into_py()?;
        Ok(())
    }

    /// Write the mesh with custom creation options
    ///
    /// Args:
    ///     path: Output file path
    ///     options: CreateOptions instance
    ///
    /// Example:
    ///     >>> from exodus import CreateOptions, CreateMode
    ///     >>> opts = CreateOptions(mode=CreateMode.NO_CLOBBER)
    ///     >>> builder.write_with_options("output.exo", opts)
    fn write_with_options(&self, path: String, options: &CreateOptions) -> PyResult<()> {
        self.builder
            .clone()
            .write_with_options(path, options.to_rust())
            .into_py()?;
        Ok(())
    }

    fn __repr__(&self) -> String {
        format!("MeshBuilder({:?})", self.builder)
    }
}
