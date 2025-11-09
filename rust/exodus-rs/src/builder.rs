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
        self.qa_records.push((
            code_name.into(),
            version.into(),
            date.into(),
            time.into(),
        ));
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
    pub fn write_with_options<P: AsRef<Path>>(
        self,
        path: P,
        options: CreateOptions,
    ) -> Result<()> {
        let mut file = ExodusFile::<mode::Write>::create(path, options)?;

        // Determine counts
        let num_nodes = self
            .coords
            .as_ref()
            .map(|(x, _, _)| x.len())
            .unwrap_or(0);

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
            .coordinates(
                vec![0.0, 1.0, 1.0, 0.0],
                vec![0.0, 0.0, 1.0, 1.0],
                vec![],
            )
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
            .coordinates(
                vec![0.0, 1.0, 1.0, 0.0],
                vec![0.0, 0.0, 1.0, 1.0],
                vec![],
            )
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
}
