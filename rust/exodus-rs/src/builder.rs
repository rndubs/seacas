//! High-level mesh builder API
//!
//! This module provides a fluent, ergonomic interface for creating Exodus meshes
//! without dealing with low-level file operations.

use crate::error::Result;
use crate::types::{Block, CreateMode, CreateOptions, EntityType, InitParams};
use crate::{mode, ExodusFile};
use std::path::Path;

/// High-level mesh builder with fluent API
///
/// # Example
///
/// ```rust,ignore
/// use exodus_rs::MeshBuilder;
///
/// MeshBuilder::new("My Mesh")
///     .dimensions(3)
///     .coordinates(x_coords, y_coords, z_coords)
///     .add_block(
///         BlockBuilder::new(1, "HEX8")
///             .connectivity(conn)
///             .build()
///     )
///     .write("mesh.exo")?;
/// ```
#[derive(Debug)]
pub struct MeshBuilder {
    title: String,
    num_dim: usize,
    coords: Option<(Vec<f64>, Vec<f64>, Vec<f64>)>,
    blocks: Vec<BlockBuilder>,
    qa_records: Vec<(String, String, String, String)>,
    info_records: Vec<String>,
}

impl MeshBuilder {
    /// Create a new mesh builder with a title
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mesh = MeshBuilder::new("Test Mesh");
    /// ```
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            num_dim: 3,
            coords: None,
            blocks: Vec::new(),
            qa_records: Vec::new(),
            info_records: Vec::new(),
        }
    }

    /// Set the number of spatial dimensions (1, 2, or 3)
    ///
    /// Default: 3
    pub fn dimensions(mut self, num_dim: usize) -> Self {
        self.num_dim = num_dim;
        self
    }

    /// Set nodal coordinates
    ///
    /// The coordinate vectors must all have the same length (number of nodes).
    /// For 2D meshes, `z` can be empty. For 1D, both `y` and `z` can be empty.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// builder.coordinates(
    ///     vec![0.0, 1.0, 1.0, 0.0],  // x
    ///     vec![0.0, 0.0, 1.0, 1.0],  // y
    ///     vec![0.0, 0.0, 0.0, 0.0],  // z
    /// )
    /// ```
    pub fn coordinates(mut self, x: Vec<f64>, y: Vec<f64>, z: Vec<f64>) -> Self {
        self.coords = Some((x, y, z));
        self
    }

    /// Add an element block
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// builder.add_block(
    ///     BlockBuilder::new(1, "HEX8")
    ///         .connectivity(vec![1, 2, 3, 4, 5, 6, 7, 8])
    ///         .build()
    /// )
    /// ```
    pub fn add_block(mut self, block: BlockBuilder) -> Self {
        self.blocks.push(block);
        self
    }

    /// Add a QA record for provenance tracking
    ///
    /// # Arguments
    ///
    /// * `code_name` - Name of the code/application
    /// * `version` - Version string
    /// * `date` - Date string (e.g., "2025-01-15")
    /// * `time` - Time string (e.g., "14:30:00")
    pub fn qa_record(
        mut self,
        code_name: impl Into<String>,
        version: impl Into<String>,
        date: impl Into<String>,
        time: impl Into<String>,
    ) -> Self {
        self.qa_records
            .push((code_name.into(), version.into(), date.into(), time.into()));
        self
    }

    /// Add an information record
    ///
    /// Information records are arbitrary text strings (max 80 characters each).
    pub fn info(mut self, info: impl Into<String>) -> Self {
        self.info_records.push(info.into());
        self
    }

    /// Write the mesh to a file
    ///
    /// This method creates the Exodus file, initializes it with the mesh
    /// parameters, and writes all the data.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// mesh_builder.write("output.exo")?;
    /// ```
    pub fn write<P: AsRef<Path>>(self, path: P) -> Result<()> {
        self.write_with_options(
            path,
            CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            },
        )
    }

    /// Write the mesh with custom creation options
    ///
    /// Allows specifying file format, compression, etc.
    pub fn write_with_options<P: AsRef<Path>>(self, path: P, options: CreateOptions) -> Result<()> {
        let mut file = ExodusFile::<mode::Write>::create(path, options)?;

        // Determine counts
        let num_nodes = self.coords.as_ref().map(|(x, _, _)| x.len()).unwrap_or(0);

        let mut num_elems = 0;
        for block in &self.blocks {
            num_elems += block.connectivity.len() / block.nodes_per_elem;
        }

        // Initialize
        let params = InitParams {
            title: self.title.clone(),
            num_dim: self.num_dim,
            num_nodes,
            num_elems,
            num_elem_blocks: self.blocks.len(),
            ..Default::default()
        };
        file.init(&params)?;

        // Write coordinates if provided
        if let Some((x, y, z)) = self.coords {
            let y_opt = if y.is_empty() { None } else { Some(&y[..]) };
            let z_opt = if z.is_empty() { None } else { Some(&z[..]) };
            file.put_coords(&x, y_opt, z_opt)?;
        }

        // Write blocks
        for block_builder in self.blocks {
            let block = Block {
                id: block_builder.id,
                entity_type: EntityType::ElemBlock,
                topology: block_builder.topology.clone(),
                num_entries: block_builder.connectivity.len() / block_builder.nodes_per_elem,
                num_nodes_per_entry: block_builder.nodes_per_elem,
                num_edges_per_entry: 0,
                num_faces_per_entry: 0,
                num_attributes: block_builder.attributes.len()
                    / (block_builder.connectivity.len() / block_builder.nodes_per_elem).max(1),
            };

            file.put_block(&block)?;
            file.put_connectivity(block.id, &block_builder.connectivity)?;

            if !block_builder.attributes.is_empty() {
                file.put_block_attributes(block.id, &block_builder.attributes)?;
            }

            if !block_builder.attribute_names.is_empty() {
                file.put_block_attribute_names(block.id, &block_builder.attribute_names)?;
            }
        }

        // Write QA records if any (skip for now - not fully implemented)
        #[cfg(feature = "netcdf4")]
        if !self.qa_records.is_empty() {
            use crate::types::QaRecord;
            let qa: Vec<QaRecord> = self
                .qa_records
                .into_iter()
                .map(|(code_name, code_version, date, time)| QaRecord {
                    code_name,
                    code_version,
                    date,
                    time,
                })
                .collect();
            // Ignore errors - QA records are optional and not yet fully implemented
            let _ = file.put_qa_records(&qa);
        }

        // Write info records if any (skip for now - not fully implemented)
        if !self.info_records.is_empty() {
            // Ignore errors - info records are optional and not yet fully implemented
            let _ = file.put_info_records(&self.info_records);
        }

        Ok(())
    }
}

/// Builder for element blocks
///
/// # Example
///
/// ```rust,ignore
/// let block = BlockBuilder::new(1, "HEX8")
///     .connectivity(vec![1, 2, 3, 4, 5, 6, 7, 8])
///     .attributes(vec![100.0])
///     .attribute_names(vec!["MaterialID"])
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct BlockBuilder {
    id: i64,
    topology: String,
    connectivity: Vec<i64>,
    nodes_per_elem: usize,
    attributes: Vec<f64>,
    attribute_names: Vec<String>,
}

impl BlockBuilder {
    /// Create a new block builder
    ///
    /// # Arguments
    ///
    /// * `id` - Block ID (must be unique)
    /// * `topology` - Element topology (e.g., "HEX8", "QUAD4", "TET4")
    pub fn new(id: i64, topology: impl Into<String>) -> Self {
        let topology = topology.into();
        let nodes_per_elem = Self::nodes_for_topology(&topology);

        Self {
            id,
            topology,
            connectivity: Vec::new(),
            nodes_per_elem,
            attributes: Vec::new(),
            attribute_names: Vec::new(),
        }
    }

    /// Set element connectivity
    ///
    /// The connectivity array should contain node IDs for each element, in a flat array.
    /// Length must be `num_elements * nodes_per_element`.
    ///
    /// Node IDs are 1-based (first node is ID 1, not 0).
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // One hex element with nodes 1-8
    /// builder.connectivity(vec![1, 2, 3, 4, 5, 6, 7, 8])
    /// ```
    pub fn connectivity(mut self, conn: Vec<i64>) -> Self {
        self.connectivity = conn;
        self
    }

    /// Set element attributes
    ///
    /// Attributes are floating-point values associated with each element.
    /// Length must be `num_elements * num_attributes_per_element`.
    pub fn attributes(mut self, attrs: Vec<f64>) -> Self {
        self.attributes = attrs;
        self
    }

    /// Set attribute names
    ///
    /// Must match the number of attributes per element.
    pub fn attribute_names<S: Into<String>>(mut self, names: Vec<S>) -> Self {
        self.attribute_names = names.into_iter().map(|s| s.into()).collect();
        self
    }

    /// Build the block (consumes the builder)
    pub fn build(self) -> Self {
        self
    }

    /// Get the expected number of nodes for a topology
    fn nodes_for_topology(topology: &str) -> usize {
        match topology.to_uppercase().as_str() {
            "SPHERE" => 1,
            "BAR2" | "TRUSS2" | "BEAM2" => 2,
            "BAR3" | "TRUSS3" | "BEAM3" => 3,
            "TRI" | "TRI3" | "TRIANGLE" => 3,
            "TRI6" => 6,
            "TRI7" => 7,
            "QUAD" | "QUAD4" | "SHELL4" => 4,
            "QUAD8" | "SHELL8" => 8,
            "QUAD9" | "SHELL9" => 9,
            "TETRA" | "TET4" | "TETRA4" => 4,
            "TETRA8" | "TET8" => 8,
            "TETRA10" | "TET10" => 10,
            "TETRA14" | "TET14" => 14,
            "TETRA15" | "TET15" => 15,
            "HEX" | "HEX8" | "HEXAHEDRON" => 8,
            "HEX20" => 20,
            "HEX27" => 27,
            "WEDGE" | "WEDGE6" => 6,
            "WEDGE15" => 15,
            "WEDGE18" => 18,
            "PYRAMID" | "PYRAMID5" => 5,
            "PYRAMID13" => 13,
            "PYRAMID14" => 14,
            _ => {
                // Try to parse number from string like "NSIDED5"
                if let Some(num_str) = topology.strip_prefix("NSIDED") {
                    num_str.parse().unwrap_or(4)
                } else {
                    // Default fallback
                    4
                }
            }
        }
    }
}

/// Builder for node sets
///
/// # Example
///
/// ```rust,ignore
/// let node_set = NodeSetBuilder::new(1)
///     .nodes(vec![1, 2, 3, 4])
///     .name("boundary_nodes")
///     .dist_factors(vec![1.0, 1.0, 1.0, 1.0])
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct NodeSetBuilder {
    id: i64,
    nodes: Vec<i64>,
    name: Option<String>,
    dist_factors: Option<Vec<f64>>,
}

impl NodeSetBuilder {
    /// Create a new node set builder
    ///
    /// # Arguments
    ///
    /// * `id` - Node set ID (must be unique)
    pub fn new(id: i64) -> Self {
        Self {
            id,
            nodes: Vec::new(),
            name: None,
            dist_factors: None,
        }
    }

    /// Set the node IDs in this set
    ///
    /// Node IDs are 1-based (first node is ID 1, not 0).
    pub fn nodes(mut self, nodes: Vec<i64>) -> Self {
        self.nodes = nodes;
        self
    }

    /// Set the name of this node set
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set distribution factors (one per node)
    pub fn dist_factors(mut self, factors: Vec<f64>) -> Self {
        self.dist_factors = Some(factors);
        self
    }

    /// Build the node set (consumes the builder)
    pub fn build(self) -> Self {
        self
    }
}

/// Builder for side sets
///
/// # Example
///
/// ```rust,ignore
/// let side_set = SideSetBuilder::new(1)
///     .sides(vec![(1, 1), (1, 2), (2, 1)])  // (element_id, side_number)
///     .name("pressure_surface")
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct SideSetBuilder {
    id: i64,
    elements: Vec<i64>,
    sides: Vec<i64>,
    name: Option<String>,
    dist_factors: Option<Vec<f64>>,
}

impl SideSetBuilder {
    /// Create a new side set builder
    ///
    /// # Arguments
    ///
    /// * `id` - Side set ID (must be unique)
    pub fn new(id: i64) -> Self {
        Self {
            id,
            elements: Vec::new(),
            sides: Vec::new(),
            name: None,
            dist_factors: None,
        }
    }

    /// Set the element-side pairs in this set
    ///
    /// # Arguments
    ///
    /// * `pairs` - Vector of (element_id, side_number) tuples
    pub fn sides(mut self, pairs: Vec<(i64, i64)>) -> Self {
        self.elements = pairs.iter().map(|(e, _)| *e).collect();
        self.sides = pairs.iter().map(|(_, s)| *s).collect();
        self
    }

    /// Set elements and sides as separate arrays
    ///
    /// # Arguments
    ///
    /// * `elements` - Element IDs
    /// * `sides` - Side numbers (must have same length as elements)
    pub fn elements_and_sides(mut self, elements: Vec<i64>, sides: Vec<i64>) -> Self {
        self.elements = elements;
        self.sides = sides;
        self
    }

    /// Set the name of this side set
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set distribution factors
    pub fn dist_factors(mut self, factors: Vec<f64>) -> Self {
        self.dist_factors = Some(factors);
        self
    }

    /// Build the side set (consumes the builder)
    pub fn build(self) -> Self {
        self
    }
}

/// Builder for appending data to existing Exodus files
///
/// This builder provides a fluent API for modifying existing Exodus files,
/// allowing you to add new blocks, sets, and variables without dealing with
/// low-level file operations.
///
/// # Example
///
/// ```rust,ignore
/// use exodus_rs::{AppendBuilder, NodeSetBuilder, SideSetBuilder};
///
/// AppendBuilder::open("mesh.exo")?
///     .add_node_set(
///         NodeSetBuilder::new(10)
///             .nodes(vec![1, 2, 3, 4])
///             .name("inlet")
///             .build()
///     )
///     .add_side_set(
///         SideSetBuilder::new(20)
///             .sides(vec![(1, 1), (1, 2)])
///             .name("wall")
///             .build()
///     )
///     .convert_nodeset_to_sideset(10, "inlet_surface")
///     .apply()?;
/// ```
#[derive(Debug)]
pub struct AppendBuilder {
    file: ExodusFile<mode::Append>,
    node_sets: Vec<NodeSetBuilder>,
    side_sets: Vec<SideSetBuilder>,
    nodeset_conversions: Vec<(i64, Option<String>)>,
}

impl AppendBuilder {
    /// Open an existing file for appending
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the existing Exodus file
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let builder = AppendBuilder::open("mesh.exo")?;
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = ExodusFile::<mode::Append>::append(path)?;
        Ok(Self {
            file,
            node_sets: Vec::new(),
            side_sets: Vec::new(),
            nodeset_conversions: Vec::new(),
        })
    }

    /// Add a node set to the file
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// builder.add_node_set(
    ///     NodeSetBuilder::new(1)
    ///         .nodes(vec![1, 2, 3])
    ///         .name("boundary")
    ///         .build()
    /// )
    /// ```
    pub fn add_node_set(mut self, node_set: NodeSetBuilder) -> Self {
        self.node_sets.push(node_set);
        self
    }

    /// Add a side set to the file
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// builder.add_side_set(
    ///     SideSetBuilder::new(1)
    ///         .sides(vec![(1, 1), (1, 2)])
    ///         .name("surface")
    ///         .build()
    /// )
    /// ```
    pub fn add_side_set(mut self, side_set: SideSetBuilder) -> Self {
        self.side_sets.push(side_set);
        self
    }

    /// Convert a node set to a side set
    ///
    /// This creates a side set from all boundary faces that have all their
    /// nodes in the specified node set.
    ///
    /// # Arguments
    ///
    /// * `nodeset_id` - ID of the node set to convert
    /// * `sideset_name` - Optional name for the new side set
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// builder.convert_nodeset_to_sideset(10, "inlet_surface")
    /// ```
    pub fn convert_nodeset_to_sideset(
        mut self,
        nodeset_id: i64,
        sideset_name: impl Into<String>,
    ) -> Self {
        self.nodeset_conversions
            .push((nodeset_id, Some(sideset_name.into())));
        self
    }

    /// Convert a node set to a side set with auto-generated name
    ///
    /// # Arguments
    ///
    /// * `nodeset_id` - ID of the node set to convert
    pub fn convert_nodeset_to_sideset_auto(mut self, nodeset_id: i64) -> Self {
        self.nodeset_conversions.push((nodeset_id, None));
        self
    }

    /// Apply all pending changes to the file
    ///
    /// This method writes all accumulated changes (node sets, side sets,
    /// conversions) to the file.
    ///
    /// # Returns
    ///
    /// The modified `ExodusFile<mode::Append>` on success
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let file = builder.apply()?;
    /// // File is now modified and can be used for further operations
    /// ```
    pub fn apply(mut self) -> Result<ExodusFile<mode::Append>> {
        // Take ownership of the vectors to avoid borrow issues
        let node_sets = std::mem::take(&mut self.node_sets);
        let side_sets = std::mem::take(&mut self.side_sets);
        let conversions = std::mem::take(&mut self.nodeset_conversions);

        // Write node sets
        for ns in &node_sets {
            self.write_node_set(ns)?;
        }

        // Write side sets
        for ss in &side_sets {
            self.write_side_set(ss)?;
        }

        // Perform nodeset-to-sideset conversions
        for (nodeset_id, sideset_name) in conversions {
            match sideset_name {
                Some(name) => {
                    self.file
                        .create_sideset_from_nodeset_named(nodeset_id, &name)?;
                }
                None => {
                    self.file.create_sideset_from_nodeset_auto(nodeset_id)?;
                }
            }
        }

        // Sync to ensure all data is written
        self.file.sync()?;

        Ok(self.file)
    }

    /// Write a node set to the file
    fn write_node_set(&mut self, ns: &NodeSetBuilder) -> Result<()> {
        // Cast to Write mode to access put_node_set
        // SAFETY: The Append mode guarantees both read and write access to the file.
        // This cast temporarily reinterprets self.file as ExodusFile<mode::Write> to access
        // write methods. We have exclusive mutable access to self.file (&mut self),
        // ensuring no aliasing violations. The PhantomData marker is zero-sized and
        // doesn't affect memory layout or safety.
        let writer = unsafe { &mut *(&mut self.file as *mut _ as *mut ExodusFile<mode::Write>) };
        writer.put_node_set(ns.id, &ns.nodes, ns.dist_factors.as_deref())?;

        // Set name if provided
        if let Some(ref name) = ns.name {
            // Get the index of the node set we just added
            let ids = self.file.set_ids(EntityType::NodeSet)?;
            if let Some(index) = ids.iter().position(|&id| id == ns.id) {
                writer.put_name(EntityType::NodeSet, index, name)?;
            }
        }

        Ok(())
    }

    /// Write a side set to the file
    fn write_side_set(&mut self, ss: &SideSetBuilder) -> Result<()> {
        // Cast to Write mode to access put_side_set
        // SAFETY: The Append mode guarantees both read and write access to the file.
        // This cast temporarily reinterprets self.file as ExodusFile<mode::Write> to access
        // write methods. We have exclusive mutable access to self.file (&mut self),
        // ensuring no aliasing violations. The PhantomData marker is zero-sized and
        // doesn't affect memory layout or safety.
        let writer = unsafe { &mut *(&mut self.file as *mut _ as *mut ExodusFile<mode::Write>) };
        writer.put_side_set(ss.id, &ss.elements, &ss.sides, ss.dist_factors.as_deref())?;

        // Set name if provided
        if let Some(ref name) = ss.name {
            // Get the index of the side set we just added
            let ids = self.file.set_ids(EntityType::SideSet)?;
            if let Some(index) = ids.iter().position(|&id| id == ss.id) {
                writer.put_name(EntityType::SideSet, index, name)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_simple_mesh_builder() {
        let tmp = NamedTempFile::new().unwrap();

        // Create a simple 2D quad mesh
        let result = MeshBuilder::new("Simple Quad")
            .dimensions(2)
            .coordinates(vec![0.0, 1.0, 1.0, 0.0], vec![0.0, 0.0, 1.0, 1.0], vec![])
            .add_block(
                BlockBuilder::new(1, "QUAD4")
                    .connectivity(vec![1, 2, 3, 4])
                    .build(),
            )
            .write(tmp.path());

        assert!(result.is_ok());
    }

    #[test]
    fn test_hex_mesh_builder() {
        let tmp = NamedTempFile::new().unwrap();

        // Create a single hex element
        let result = MeshBuilder::new("Single Hex")
            .dimensions(3)
            .coordinates(
                vec![0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0],
                vec![0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0],
                vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0],
            )
            .add_block(
                BlockBuilder::new(100, "HEX8")
                    .connectivity(vec![1, 2, 3, 4, 5, 6, 7, 8])
                    .build(),
            )
            .qa_record("exodus-rs", "0.1.0", "2025-01-15", "12:00:00")
            .info("Created by MeshBuilder test")
            .write(tmp.path());

        assert!(result.is_ok());

        // Read back and verify
        let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
        let params = file.init_params().unwrap();
        assert_eq!(params.num_nodes, 8);
        assert_eq!(params.num_elems, 1);
        assert_eq!(params.num_elem_blocks, 1);
    }

    #[test]
    fn test_multiple_blocks() {
        let tmp = NamedTempFile::new().unwrap();

        let result = MeshBuilder::new("Multi-Block")
            .dimensions(3)
            .coordinates(
                vec![0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 2.0, 2.0],
                vec![0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0],
                vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0],
            )
            .add_block(
                BlockBuilder::new(1, "HEX8")
                    .connectivity(vec![1, 2, 3, 4, 5, 6, 7, 8])
                    .build(),
            )
            .add_block(
                BlockBuilder::new(2, "TRI3")
                    .connectivity(vec![2, 9, 10])
                    .build(),
            )
            .write(tmp.path());

        assert!(result.is_ok());
    }

    #[test]
    fn test_block_with_attributes() {
        let tmp = NamedTempFile::new().unwrap();

        let result = MeshBuilder::new("Block with Attributes")
            .dimensions(2)
            .coordinates(vec![0.0, 1.0, 1.0, 0.0], vec![0.0, 0.0, 1.0, 1.0], vec![])
            .add_block(
                BlockBuilder::new(1, "QUAD4")
                    .connectivity(vec![1, 2, 3, 4])
                    .attributes(vec![100.0])
                    .attribute_names(vec!["MaterialID"])
                    .build(),
            )
            .write(tmp.path());

        assert!(result.is_ok());
    }

    #[test]
    fn test_topology_node_counts() {
        assert_eq!(BlockBuilder::nodes_for_topology("HEX8"), 8);
        assert_eq!(BlockBuilder::nodes_for_topology("TET4"), 4);
        assert_eq!(BlockBuilder::nodes_for_topology("QUAD4"), 4);
        assert_eq!(BlockBuilder::nodes_for_topology("TRI3"), 3);
        assert_eq!(BlockBuilder::nodes_for_topology("WEDGE6"), 6);
        assert_eq!(BlockBuilder::nodes_for_topology("PYRAMID5"), 5);
    }

    #[test]
    fn test_node_set_builder() {
        let ns = NodeSetBuilder::new(10)
            .nodes(vec![1, 2, 3, 4])
            .name("boundary")
            .dist_factors(vec![1.0, 1.0, 1.0, 1.0])
            .build();

        assert_eq!(ns.id, 10);
        assert_eq!(ns.nodes, vec![1, 2, 3, 4]);
        assert_eq!(ns.name, Some("boundary".to_string()));
        assert_eq!(ns.dist_factors, Some(vec![1.0, 1.0, 1.0, 1.0]));
    }

    #[test]
    fn test_side_set_builder() {
        let ss = SideSetBuilder::new(20)
            .sides(vec![(1, 1), (1, 2), (2, 3)])
            .name("surface")
            .build();

        assert_eq!(ss.id, 20);
        assert_eq!(ss.elements, vec![1, 1, 2]);
        assert_eq!(ss.sides, vec![1, 2, 3]);
        assert_eq!(ss.name, Some("surface".to_string()));
    }

    #[test]
    fn test_side_set_builder_separate_arrays() {
        let ss = SideSetBuilder::new(30)
            .elements_and_sides(vec![1, 2, 3], vec![4, 5, 6])
            .build();

        assert_eq!(ss.id, 30);
        assert_eq!(ss.elements, vec![1, 2, 3]);
        assert_eq!(ss.sides, vec![4, 5, 6]);
    }

    #[test]
    fn test_append_builder_add_node_set() {
        use crate::types::InitParams;

        let tmp = NamedTempFile::new().unwrap();

        // First create a mesh with proper initialization including node set capacity
        {
            let mut file = ExodusFile::<mode::Write>::create(
                tmp.path(),
                CreateOptions {
                    mode: CreateMode::Clobber,
                    ..Default::default()
                },
            )
            .unwrap();

            let params = InitParams {
                title: "Test Mesh".to_string(),
                num_dim: 2,
                num_nodes: 4,
                num_elems: 1,
                num_elem_blocks: 1,
                num_node_sets: 1, // Reserve space for node sets
                ..Default::default()
            };
            file.init(&params).unwrap();
            file.put_coords(&[0.0, 1.0, 1.0, 0.0], Some(&[0.0, 0.0, 1.0, 1.0]), None)
                .unwrap();
        }

        // Then add a node set using AppendBuilder
        let file = AppendBuilder::open(tmp.path())
            .unwrap()
            .add_node_set(NodeSetBuilder::new(10).nodes(vec![1, 2]).build())
            .apply()
            .unwrap();

        // Verify the node set was added
        let ids = file.set_ids(EntityType::NodeSet).unwrap();
        assert!(ids.contains(&10));
    }

    #[test]
    fn test_append_builder_add_side_set() {
        use crate::types::InitParams;

        let tmp = NamedTempFile::new().unwrap();

        // First create a mesh with proper initialization including side set capacity
        {
            let mut file = ExodusFile::<mode::Write>::create(
                tmp.path(),
                CreateOptions {
                    mode: CreateMode::Clobber,
                    ..Default::default()
                },
            )
            .unwrap();

            let params = InitParams {
                title: "Test Mesh".to_string(),
                num_dim: 2,
                num_nodes: 4,
                num_elems: 1,
                num_elem_blocks: 1,
                num_side_sets: 1, // Reserve space for side sets
                ..Default::default()
            };
            file.init(&params).unwrap();
            file.put_coords(&[0.0, 1.0, 1.0, 0.0], Some(&[0.0, 0.0, 1.0, 1.0]), None)
                .unwrap();
        }

        // Then add a side set using AppendBuilder
        let file = AppendBuilder::open(tmp.path())
            .unwrap()
            .add_side_set(SideSetBuilder::new(20).sides(vec![(1, 1), (1, 2)]).build())
            .apply()
            .unwrap();

        // Verify the side set was added
        let ids = file.set_ids(EntityType::SideSet).unwrap();
        assert!(ids.contains(&20));
    }
}
