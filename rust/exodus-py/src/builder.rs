//! High-level builder API for creating Exodus meshes

use pyo3::prelude::*;
use exodus_rs::{BlockBuilder as RustBlockBuilder};
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
/// NOTE: Due to API limitations, this builder does not support method chaining.
/// Set all properties before calling write().
///
/// Example:
///     >>> builder = MeshBuilder("My Mesh")
///     >>> builder.set_dimensions(3)
///     >>> builder.set_coordinates([0.0, 1.0], [0.0, 1.0], [])
///     >>> builder.write("mesh.exo")
#[pyclass]
pub struct MeshBuilder {
    title: String,
    num_dim: Option<usize>,
    coords: Option<(Vec<f64>, Vec<f64>, Vec<f64>)>,
    blocks: Vec<RustBlockBuilder>,
    qa_records: Vec<(String, String, String, String)>,
    info_records: Vec<String>,
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
            title,
            num_dim: None,
            coords: None,
            blocks: Vec::new(),
            qa_records: Vec::new(),
            info_records: Vec::new(),
        }
    }

    /// Set the number of spatial dimensions
    ///
    /// Args:
    ///     num_dim: Number of dimensions (1, 2, or 3)
    ///
    /// Example:
    ///     >>> builder.set_dimensions(3)
    fn set_dimensions(&mut self, num_dim: usize) {
        self.num_dim = Some(num_dim);
    }

    /// Set nodal coordinates
    ///
    /// Args:
    ///     x: X coordinates (required)
    ///     y: Y coordinates (required for 2D/3D, can be empty for 1D)
    ///     z: Z coordinates (required for 3D, can be empty for 1D/2D)
    ///
    /// Example:
    ///     >>> builder.set_coordinates(
    ///     ...     x=[0.0, 1.0, 1.0, 0.0],
    ///     ...     y=[0.0, 0.0, 1.0, 1.0],
    ///     ...     z=[]
    ///     ... )
    #[pyo3(signature = (x, y=Vec::new(), z=Vec::new()))]
    fn set_coordinates(&mut self, x: Vec<f64>, y: Vec<f64>, z: Vec<f64>) {
        self.coords = Some((x, y, z));
    }

    /// Add an element block
    ///
    /// Args:
    ///     block: BlockBuilder instance
    ///
    /// Example:
    ///     >>> builder.add_block(
    ///     ...     BlockBuilder(1, "HEX8")
    ///     ...         .connectivity([1, 2, 3, 4, 5, 6, 7, 8])
    ///     ...         .build()
    ///     ... )
    fn add_block(&mut self, block: &BlockBuilder) {
        self.blocks.push(block.builder.clone());
    }

    /// Add a QA record for provenance tracking
    ///
    /// Args:
    ///     code_name: Name of the code/application
    ///     version: Version string
    ///     date: Date string (e.g., "2025-01-15")
    ///     time: Time string (e.g., "14:30:00")
    ///
    /// Example:
    ///     >>> builder.add_qa_record("MyApp", "1.0.0", "2025-01-15", "14:30:00")
    fn add_qa_record(&mut self, code_name: String, version: String, date: String, time: String) {
        self.qa_records.push((code_name, version, date, time));
    }

    /// Add an information record
    ///
    /// Information records are arbitrary text strings (max 80 characters each).
    ///
    /// Args:
    ///     info: Info string
    ///
    /// Example:
    ///     >>> builder.add_info("Generated by mesh builder")
    fn add_info(&mut self, info: String) {
        self.info_records.push(info);
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
        // Build the Rust MeshBuilder
        let mut rust_builder = exodus_rs::MeshBuilder::new(self.title.clone());
        
        if let Some(num_dim) = self.num_dim {
            rust_builder = rust_builder.dimensions(num_dim);
        }
        
        if let Some((ref x, ref y, ref z)) = self.coords {
            rust_builder = rust_builder.coordinates(x.clone(), y.clone(), z.clone());
        }
        
        for block in &self.blocks {
            rust_builder = rust_builder.add_block(block.clone());
        }
        
        for (code_name, version, date, time) in &self.qa_records {
            rust_builder = rust_builder.qa_record(code_name.clone(), version.clone(), date.clone(), time.clone());
        }
        
        for info in &self.info_records {
            rust_builder = rust_builder.info(info.clone());
        }
        
        rust_builder.write(path).into_py()?;
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
        // Build the Rust MeshBuilder
        let mut rust_builder = exodus_rs::MeshBuilder::new(self.title.clone());
        
        if let Some(num_dim) = self.num_dim {
            rust_builder = rust_builder.dimensions(num_dim);
        }
        
        if let Some((ref x, ref y, ref z)) = self.coords {
            rust_builder = rust_builder.coordinates(x.clone(), y.clone(), z.clone());
        }
        
        for block in &self.blocks {
            rust_builder = rust_builder.add_block(block.clone());
        }
        
        for (code_name, version, date, time) in &self.qa_records {
            rust_builder = rust_builder.qa_record(code_name.clone(), version.clone(), date.clone(), time.clone());
        }
        
        for info in &self.info_records {
            rust_builder = rust_builder.info(info.clone());
        }
        
        rust_builder.write_with_options(path, options.to_rust()).into_py()?;
        Ok(())
    }

    fn __repr__(&self) -> String {
        format!("MeshBuilder(title=\"{}\")", self.title)
    }
}
