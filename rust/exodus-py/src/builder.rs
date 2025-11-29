//! High-level builder API for creating Exodus meshes

use crate::error::IntoPyResult;
use crate::types::CreateOptions;
use exodus_rs::BlockBuilder as RustBlockBuilder;
use pyo3::prelude::*;

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

    /// Set the number of spatial dimensions (fluent API)
    ///
    /// Args:
    ///     num_dim: Number of dimensions (1, 2, or 3)
    ///
    /// Returns:
    ///     Self for method chaining
    ///
    /// Example:
    ///     >>> builder.dimensions(3).coordinates(...)
    fn dimensions(mut slf: PyRefMut<'_, Self>, num_dim: usize) -> PyRefMut<'_, Self> {
        slf.num_dim = Some(num_dim);
        slf
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

    /// Set nodal coordinates (fluent API)
    ///
    /// Args:
    ///     x: X coordinates (required)
    ///     y: Y coordinates (required for 2D/3D, can be empty for 1D)
    ///     z: Z coordinates (required for 3D, can be empty for 1D/2D)
    ///
    /// Returns:
    ///     Self for method chaining
    ///
    /// Example:
    ///     >>> builder.dimensions(2).coordinates(
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
        slf.coords = Some((x, y, z));
        slf
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

    /// Add an element block (fluent API)
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
    ///     ... ).write("output.exo")
    fn add_block<'a>(mut slf: PyRefMut<'a, Self>, block: &'a BlockBuilder) -> PyRefMut<'a, Self> {
        slf.blocks.push(block.builder.clone());
        slf
    }

    /// Add a QA record for provenance tracking (fluent API)
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
    ///     >>> builder.qa_record("MyApp", "1.0.0", "2025-01-15", "14:30:00").write("output.exo")
    fn qa_record(
        mut slf: PyRefMut<'_, Self>,
        code_name: String,
        version: String,
        date: String,
        time: String,
    ) -> PyRefMut<'_, Self> {
        slf.qa_records.push((code_name, version, date, time));
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
    /// Example:
    ///     >>> builder.add_qa_record("MyApp", "1.0.0", "2025-01-15", "14:30:00")
    fn add_qa_record(&mut self, code_name: String, version: String, date: String, time: String) {
        self.qa_records.push((code_name, version, date, time));
    }

    /// Add an information record (fluent API)
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
    ///     >>> builder.info("Generated by mesh builder").write("output.exo")
    fn info(mut slf: PyRefMut<'_, Self>, info_text: String) -> PyRefMut<'_, Self> {
        slf.info_records.push(info_text);
        slf
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
            rust_builder = rust_builder.qa_record(
                code_name.clone(),
                version.clone(),
                date.clone(),
                time.clone(),
            );
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
            rust_builder = rust_builder.qa_record(
                code_name.clone(),
                version.clone(),
                date.clone(),
                time.clone(),
            );
        }

        for info in &self.info_records {
            rust_builder = rust_builder.info(info.clone());
        }

        rust_builder
            .write_with_options(path, options.to_rust())
            .into_py()?;
        Ok(())
    }

    fn __repr__(&self) -> String {
        format!("MeshBuilder(title=\"{}\")", self.title)
    }
}

/// Python wrapper for NodeSetBuilder
///
/// Provides a fluent interface for building node sets.
///
/// Example:
///     >>> node_set = (NodeSetBuilder(10)
///     ...     .nodes([1, 2, 3, 4])
///     ...     .name("inlet")
///     ...     .dist_factors([1.0, 1.0, 1.0, 1.0])
///     ...     .build())
#[pyclass]
#[derive(Clone)]
pub struct NodeSetBuilder {
    builder: exodus_rs::NodeSetBuilder,
}

#[pymethods]
impl NodeSetBuilder {
    /// Create a new node set builder
    ///
    /// Args:
    ///     id: Node set ID (must be unique)
    ///
    /// Example:
    ///     >>> builder = NodeSetBuilder(10)
    #[new]
    fn new(id: i64) -> Self {
        NodeSetBuilder {
            builder: exodus_rs::NodeSetBuilder::new(id),
        }
    }

    /// Set node IDs in this set
    ///
    /// Args:
    ///     nodes: List of node IDs (1-based)
    ///
    /// Returns:
    ///     Self for method chaining
    ///
    /// Example:
    ///     >>> builder.nodes([1, 2, 3, 4])
    fn nodes(mut slf: PyRefMut<'_, Self>, nodes: Vec<i64>) -> PyRefMut<'_, Self> {
        slf.builder = slf.builder.clone().nodes(nodes);
        slf
    }

    /// Set the name of this node set
    ///
    /// Args:
    ///     name: Name for the node set
    ///
    /// Returns:
    ///     Self for method chaining
    ///
    /// Example:
    ///     >>> builder.name("inlet")
    fn name(mut slf: PyRefMut<'_, Self>, name: String) -> PyRefMut<'_, Self> {
        slf.builder = slf.builder.clone().name(name);
        slf
    }

    /// Set distribution factors (one per node)
    ///
    /// Args:
    ///     factors: List of distribution factors
    ///
    /// Returns:
    ///     Self for method chaining
    ///
    /// Example:
    ///     >>> builder.dist_factors([1.0, 1.0, 1.0, 1.0])
    fn dist_factors(mut slf: PyRefMut<'_, Self>, factors: Vec<f64>) -> PyRefMut<'_, Self> {
        slf.builder = slf.builder.clone().dist_factors(factors);
        slf
    }

    /// Build the node set (for API compatibility)
    ///
    /// Returns:
    ///     Self
    fn build(slf: PyRefMut<'_, Self>) -> PyRefMut<'_, Self> {
        slf
    }

    fn __repr__(&self) -> String {
        format!("NodeSetBuilder({:?})", self.builder)
    }
}

/// Python wrapper for SideSetBuilder
///
/// Provides a fluent interface for building side sets.
///
/// Example:
///     >>> side_set = (SideSetBuilder(20)
///     ...     .sides([(1, 1), (1, 2), (2, 3)])
///     ...     .name("outlet")
///     ...     .build())
#[pyclass]
#[derive(Clone)]
pub struct SideSetBuilder {
    builder: exodus_rs::SideSetBuilder,
}

#[pymethods]
impl SideSetBuilder {
    /// Create a new side set builder
    ///
    /// Args:
    ///     id: Side set ID (must be unique)
    ///
    /// Example:
    ///     >>> builder = SideSetBuilder(20)
    #[new]
    fn new(id: i64) -> Self {
        SideSetBuilder {
            builder: exodus_rs::SideSetBuilder::new(id),
        }
    }

    /// Set element-side pairs in this set
    ///
    /// Args:
    ///     pairs: List of (element_id, side_number) tuples
    ///
    /// Returns:
    ///     Self for method chaining
    ///
    /// Example:
    ///     >>> builder.sides([(1, 1), (1, 2), (2, 3)])
    fn sides(mut slf: PyRefMut<'_, Self>, pairs: Vec<(i64, i64)>) -> PyRefMut<'_, Self> {
        slf.builder = slf.builder.clone().sides(pairs);
        slf
    }

    /// Set elements and sides as separate arrays
    ///
    /// Args:
    ///     elements: List of element IDs
    ///     sides: List of side numbers (must have same length as elements)
    ///
    /// Returns:
    ///     Self for method chaining
    ///
    /// Example:
    ///     >>> builder.elements_and_sides([1, 1, 2], [1, 2, 3])
    fn elements_and_sides(
        mut slf: PyRefMut<'_, Self>,
        elements: Vec<i64>,
        sides: Vec<i64>,
    ) -> PyRefMut<'_, Self> {
        slf.builder = slf.builder.clone().elements_and_sides(elements, sides);
        slf
    }

    /// Set the name of this side set
    ///
    /// Args:
    ///     name: Name for the side set
    ///
    /// Returns:
    ///     Self for method chaining
    ///
    /// Example:
    ///     >>> builder.name("outlet")
    fn name(mut slf: PyRefMut<'_, Self>, name: String) -> PyRefMut<'_, Self> {
        slf.builder = slf.builder.clone().name(name);
        slf
    }

    /// Set distribution factors
    ///
    /// Args:
    ///     factors: List of distribution factors
    ///
    /// Returns:
    ///     Self for method chaining
    fn dist_factors(mut slf: PyRefMut<'_, Self>, factors: Vec<f64>) -> PyRefMut<'_, Self> {
        slf.builder = slf.builder.clone().dist_factors(factors);
        slf
    }

    /// Build the side set (for API compatibility)
    ///
    /// Returns:
    ///     Self
    fn build(slf: PyRefMut<'_, Self>) -> PyRefMut<'_, Self> {
        slf
    }

    fn __repr__(&self) -> String {
        format!("SideSetBuilder({:?})", self.builder)
    }
}

/// Python wrapper for AppendBuilder
///
/// Provides a fluent API for modifying existing Exodus files,
/// allowing you to add new sets and perform conversions.
///
/// Example:
///     >>> (AppendBuilder.open("mesh.exo")
///     ...     .add_node_set(
///     ...         NodeSetBuilder(10)
///     ...             .nodes([1, 2, 3, 4])
///     ...             .name("inlet")
///     ...             .build()
///     ...     )
///     ...     .add_side_set(
///     ...         SideSetBuilder(20)
///     ...             .sides([(1, 1), (1, 2)])
///     ...             .name("wall")
///     ...             .build()
///     ...     )
///     ...     .apply())
#[pyclass]
pub struct AppendBuilder {
    path: String,
    node_sets: Vec<exodus_rs::NodeSetBuilder>,
    side_sets: Vec<exodus_rs::SideSetBuilder>,
    nodeset_conversions: Vec<(i64, Option<String>)>,
}

#[pymethods]
impl AppendBuilder {
    /// Open an existing file for appending
    ///
    /// Args:
    ///     path: Path to the existing Exodus file
    ///
    /// Returns:
    ///     AppendBuilder instance
    ///
    /// Example:
    ///     >>> builder = AppendBuilder.open("mesh.exo")
    #[staticmethod]
    fn open(path: String) -> Self {
        AppendBuilder {
            path,
            node_sets: Vec::new(),
            side_sets: Vec::new(),
            nodeset_conversions: Vec::new(),
        }
    }

    /// Add a node set to the file
    ///
    /// Args:
    ///     node_set: NodeSetBuilder instance
    ///
    /// Returns:
    ///     Self for method chaining
    ///
    /// Example:
    ///     >>> builder.add_node_set(
    ///     ...     NodeSetBuilder(1).nodes([1, 2, 3]).name("boundary").build()
    ///     ... )
    fn add_node_set<'a>(
        mut slf: PyRefMut<'a, Self>,
        node_set: &'a NodeSetBuilder,
    ) -> PyRefMut<'a, Self> {
        slf.node_sets.push(node_set.builder.clone());
        slf
    }

    /// Add a side set to the file
    ///
    /// Args:
    ///     side_set: SideSetBuilder instance
    ///
    /// Returns:
    ///     Self for method chaining
    ///
    /// Example:
    ///     >>> builder.add_side_set(
    ///     ...     SideSetBuilder(1).sides([(1, 1), (1, 2)]).name("surface").build()
    ///     ... )
    fn add_side_set<'a>(
        mut slf: PyRefMut<'a, Self>,
        side_set: &'a SideSetBuilder,
    ) -> PyRefMut<'a, Self> {
        slf.side_sets.push(side_set.builder.clone());
        slf
    }

    /// Convert a node set to a side set
    ///
    /// This creates a side set from all boundary faces that have all their
    /// nodes in the specified node set.
    ///
    /// Args:
    ///     nodeset_id: ID of the node set to convert
    ///     sideset_name: Name for the new side set
    ///
    /// Returns:
    ///     Self for method chaining
    ///
    /// Example:
    ///     >>> builder.convert_nodeset_to_sideset(10, "inlet_surface")
    fn convert_nodeset_to_sideset(
        mut slf: PyRefMut<'_, Self>,
        nodeset_id: i64,
        sideset_name: String,
    ) -> PyRefMut<'_, Self> {
        slf.nodeset_conversions
            .push((nodeset_id, Some(sideset_name)));
        slf
    }

    /// Convert a node set to a side set with auto-generated ID
    ///
    /// Args:
    ///     nodeset_id: ID of the node set to convert
    ///
    /// Returns:
    ///     Self for method chaining
    ///
    /// Example:
    ///     >>> builder.convert_nodeset_to_sideset_auto(10)
    fn convert_nodeset_to_sideset_auto(
        mut slf: PyRefMut<'_, Self>,
        nodeset_id: i64,
    ) -> PyRefMut<'_, Self> {
        slf.nodeset_conversions.push((nodeset_id, None));
        slf
    }

    /// Apply all pending changes to the file
    ///
    /// This method writes all accumulated changes (node sets, side sets,
    /// conversions) to the file.
    ///
    /// Raises:
    ///     ExodusError: If the file cannot be opened or modifications fail
    ///
    /// Example:
    ///     >>> builder.apply()
    fn apply(&self) -> PyResult<()> {
        // Build the Rust AppendBuilder and apply changes
        let mut builder = exodus_rs::AppendBuilder::open(&self.path).into_py()?;

        for ns in &self.node_sets {
            builder = builder.add_node_set(ns.clone());
        }

        for ss in &self.side_sets {
            builder = builder.add_side_set(ss.clone());
        }

        for (nodeset_id, sideset_name) in &self.nodeset_conversions {
            match sideset_name {
                Some(name) => {
                    builder = builder.convert_nodeset_to_sideset(*nodeset_id, name.clone());
                }
                None => {
                    builder = builder.convert_nodeset_to_sideset_auto(*nodeset_id);
                }
            }
        }

        builder.apply().into_py()?;
        Ok(())
    }

    fn __repr__(&self) -> String {
        format!(
            "AppendBuilder(path=\"{}\", node_sets={}, side_sets={}, conversions={})",
            self.path,
            self.node_sets.len(),
            self.side_sets.len(),
            self.nodeset_conversions.len()
        )
    }
}
