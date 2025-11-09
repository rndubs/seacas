//! Core type definitions for Exodus II.

use crate::error::EntityId;

/// All entity types supported by Exodus
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum EntityType {
    /// Element block
    ElemBlock = 1,
    /// Node set
    NodeSet = 2,
    /// Side set
    SideSet = 3,
    /// Node map
    NodeMap = 4,
    /// Element map
    ElemMap = 5,
    /// Edge block
    EdgeBlock = 6,
    /// Edge set
    EdgeSet = 7,
    /// Face block
    FaceBlock = 8,
    /// Face set
    FaceSet = 9,
    /// Element set
    ElemSet = 10,
    /// Edge map
    EdgeMap = 11,
    /// Face map
    FaceMap = 12,
    /// Global variables
    Global = 13,
    /// Nodal variables
    Nodal = 14,
    /// Assembly
    Assembly = 16,
    /// Blob
    Blob = 17,
}

impl EntityType {
    /// Get the string representation of the entity type
    pub fn as_str(&self) -> &'static str {
        match self {
            EntityType::ElemBlock => "elem_block",
            EntityType::NodeSet => "node_set",
            EntityType::SideSet => "side_set",
            EntityType::NodeMap => "node_map",
            EntityType::ElemMap => "elem_map",
            EntityType::EdgeBlock => "edge_block",
            EntityType::EdgeSet => "edge_set",
            EntityType::FaceBlock => "face_block",
            EntityType::FaceSet => "face_set",
            EntityType::ElemSet => "elem_set",
            EntityType::EdgeMap => "edge_map",
            EntityType::FaceMap => "face_map",
            EntityType::Global => "global",
            EntityType::Nodal => "nodal",
            EntityType::Assembly => "assembly",
            EntityType::Blob => "blob",
        }
    }
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Initialization parameters for new Exodus files
#[derive(Debug, Clone)]
pub struct InitParams {
    /// Title of the database (max 80 characters)
    pub title: String,
    /// Number of spatial dimensions (1, 2, or 3)
    pub num_dim: usize,
    /// Number of nodes
    pub num_nodes: usize,
    /// Number of edges
    pub num_edges: usize,
    /// Number of edge blocks
    pub num_edge_blocks: usize,
    /// Number of faces
    pub num_faces: usize,
    /// Number of face blocks
    pub num_face_blocks: usize,
    /// Number of elements
    pub num_elems: usize,
    /// Number of element blocks
    pub num_elem_blocks: usize,
    /// Number of node sets
    pub num_node_sets: usize,
    /// Number of edge sets
    pub num_edge_sets: usize,
    /// Number of face sets
    pub num_face_sets: usize,
    /// Number of side sets
    pub num_side_sets: usize,
    /// Number of element sets
    pub num_elem_sets: usize,
    /// Number of node maps
    pub num_node_maps: usize,
    /// Number of edge maps
    pub num_edge_maps: usize,
    /// Number of face maps
    pub num_face_maps: usize,
    /// Number of element maps
    pub num_elem_maps: usize,
    /// Number of assemblies
    pub num_assemblies: usize,
    /// Number of blobs
    pub num_blobs: usize,
}

impl Default for InitParams {
    fn default() -> Self {
        Self {
            title: String::new(),
            num_dim: 3,
            num_nodes: 0,
            num_edges: 0,
            num_edge_blocks: 0,
            num_faces: 0,
            num_face_blocks: 0,
            num_elems: 0,
            num_elem_blocks: 0,
            num_node_sets: 0,
            num_edge_sets: 0,
            num_face_sets: 0,
            num_side_sets: 0,
            num_elem_sets: 0,
            num_node_maps: 0,
            num_edge_maps: 0,
            num_face_maps: 0,
            num_elem_maps: 0,
            num_assemblies: 0,
            num_blobs: 0,
        }
    }
}

/// Block (element/edge/face) parameters
#[derive(Debug, Clone)]
pub struct Block {
    /// Block ID
    pub id: EntityId,
    /// Entity type (ElemBlock, EdgeBlock, or FaceBlock)
    pub entity_type: EntityType,
    /// Topology name (e.g., "HEX8", "QUAD4", "TETRA4")
    pub topology: String,
    /// Number of entries (elements/edges/faces) in this block
    pub num_entries: usize,
    /// Number of nodes per entry
    pub num_nodes_per_entry: usize,
    /// Number of edges per entry
    pub num_edges_per_entry: usize,
    /// Number of faces per entry
    pub num_faces_per_entry: usize,
    /// Number of attributes per entry
    pub num_attributes: usize,
}

/// Set (node/edge/face/elem/side) parameters
#[derive(Debug, Clone)]
pub struct Set {
    /// Set ID
    pub id: EntityId,
    /// Entity type (NodeSet, EdgeSet, FaceSet, ElemSet, or SideSet)
    pub entity_type: EntityType,
    /// Number of entries in the set
    pub num_entries: usize,
    /// Number of distribution factors
    pub num_dist_factors: usize,
}

/// Node set with node IDs and optional distribution factors
#[derive(Debug, Clone)]
pub struct NodeSet {
    /// Set ID
    pub id: EntityId,
    /// Node IDs in the set
    pub nodes: Vec<i64>,
    /// Distribution factors (one per node, or empty if not used)
    pub dist_factors: Vec<f64>,
}

/// Side set with element-side pairs and optional distribution factors
#[derive(Debug, Clone)]
pub struct SideSet {
    /// Set ID
    pub id: EntityId,
    /// Element IDs that define the sides
    pub elements: Vec<i64>,
    /// Side numbers within each element (topology dependent)
    pub sides: Vec<i64>,
    /// Distribution factors (one per node-on-side, or empty if not used)
    pub dist_factors: Vec<f64>,
}

/// Entity set (edge, face, or element set)
#[derive(Debug, Clone)]
pub struct EntitySet {
    /// Set ID
    pub id: EntityId,
    /// Entity type (EdgeSet, FaceSet, or ElemSet)
    pub entity_type: EntityType,
    /// Entity IDs in the set
    pub entities: Vec<i64>,
}

/// Assembly (hierarchical grouping)
#[derive(Debug, Clone)]
pub struct Assembly {
    /// Assembly ID
    pub id: EntityId,
    /// Assembly name
    pub name: String,
    /// Entity type of members
    pub entity_type: EntityType,
    /// List of entity IDs in this assembly
    pub entity_list: Vec<EntityId>,
}

/// Blob (arbitrary binary data)
#[derive(Debug, Clone)]
pub struct Blob {
    /// Blob ID
    pub id: EntityId,
    /// Blob name
    pub name: String,
}

/// Attribute metadata
#[derive(Debug, Clone)]
pub struct Attribute {
    /// Entity type this attribute belongs to
    pub entity_type: EntityType,
    /// Entity ID
    pub entity_id: EntityId,
    /// Attribute name
    pub name: String,
    /// Value type
    pub value_type: AttributeType,
}

/// Attribute value types
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AttributeType {
    /// Integer attribute
    Integer,
    /// Double precision floating point attribute
    Double,
    /// Character/string attribute
    Char,
}

/// Truth table for sparse variable storage
///
/// Truth tables indicate which element blocks have which variables defined.
/// This allows for efficient storage when not all blocks have all variables.
#[derive(Debug, Clone)]
pub struct TruthTable {
    /// Entity type this truth table applies to
    pub var_type: EntityType,
    /// Number of variables
    pub num_vars: usize,
    /// Number of blocks
    pub num_blocks: usize,
    /// Flat 2D array: table[block_idx * num_vars + var_idx]
    pub table: Vec<bool>,
}

impl TruthTable {
    /// Create a new truth table with all entries set to true (all variables defined for all blocks)
    ///
    /// # Arguments
    ///
    /// * `var_type` - Entity type (typically ElemBlock, EdgeBlock, or FaceBlock)
    /// * `num_blocks` - Number of blocks
    /// * `num_vars` - Number of variables
    pub fn new(var_type: EntityType, num_blocks: usize, num_vars: usize) -> Self {
        Self {
            var_type,
            num_vars,
            num_blocks,
            table: vec![true; num_blocks * num_vars],
        }
    }

    /// Set whether a variable exists for a block
    ///
    /// # Arguments
    ///
    /// * `block_idx` - Block index (0-based)
    /// * `var_idx` - Variable index (0-based)
    /// * `exists` - True if the variable exists for this block
    pub fn set(&mut self, block_idx: usize, var_idx: usize, exists: bool) {
        if block_idx < self.num_blocks && var_idx < self.num_vars {
            self.table[block_idx * self.num_vars + var_idx] = exists;
        }
    }

    /// Get whether a variable exists for a block
    ///
    /// # Arguments
    ///
    /// * `block_idx` - Block index (0-based)
    /// * `var_idx` - Variable index (0-based)
    ///
    /// # Returns
    ///
    /// True if the variable exists for this block
    pub fn get(&self, block_idx: usize, var_idx: usize) -> bool {
        if block_idx < self.num_blocks && var_idx < self.num_vars {
            self.table[block_idx * self.num_vars + var_idx]
        } else {
            false
        }
    }
}

/// QA Record (software provenance tracking)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QaRecord {
    /// Code name (max 32 characters)
    pub code_name: String,
    /// Code version (max 32 characters)
    pub code_version: String,
    /// Date (max 32 characters)
    pub date: String,
    /// Time (max 32 characters)
    pub time: String,
}

/// Information record (arbitrary text, max 80 chars each)
pub type InfoRecord = String;

/// Element topology types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Topology {
    // 0D
    /// Sphere element (1 node)
    Sphere,

    // 1D
    /// 2-node bar/truss/beam element
    Bar2,
    /// 3-node bar/truss/beam element
    Bar3,

    // 2D
    /// 3-node triangle
    Tri3,
    /// 6-node triangle
    Tri6,
    /// 7-node triangle
    Tri7,
    /// 4-node quadrilateral
    Quad4,
    /// 8-node quadrilateral
    Quad8,
    /// 9-node quadrilateral
    Quad9,

    // 3D
    /// 4-node tetrahedron
    Tet4,
    /// 8-node tetrahedron
    Tet8,
    /// 10-node tetrahedron
    Tet10,
    /// 14-node tetrahedron
    Tet14,
    /// 15-node tetrahedron
    Tet15,
    /// 8-node hexahedron
    Hex8,
    /// 20-node hexahedron
    Hex20,
    /// 27-node hexahedron
    Hex27,
    /// 6-node wedge/prism
    Wedge6,
    /// 15-node wedge/prism
    Wedge15,
    /// 18-node wedge/prism
    Wedge18,
    /// 5-node pyramid
    Pyramid5,
    /// 13-node pyramid
    Pyramid13,
    /// 14-node pyramid
    Pyramid14,

    // Arbitrary
    /// N-sided polygon (variable node count)
    NSided,
    /// N-faced polyhedron (variable node count)
    NFaced,

    // Custom
    /// Custom topology with arbitrary name
    Custom(String),
}

impl Topology {
    /// Parse topology from string
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "SPHERE" => Self::Sphere,
            "BAR2" | "TRUSS2" | "BEAM2" => Self::Bar2,
            "BAR3" | "TRUSS3" | "BEAM3" => Self::Bar3,
            "TRI" | "TRI3" | "TRIANGLE" => Self::Tri3,
            "TRI6" => Self::Tri6,
            "TRI7" => Self::Tri7,
            "QUAD" | "QUAD4" | "SHELL4" => Self::Quad4,
            "QUAD8" | "SHELL8" => Self::Quad8,
            "QUAD9" | "SHELL9" => Self::Quad9,
            "TETRA" | "TET4" | "TETRA4" => Self::Tet4,
            "TET8" | "TETRA8" => Self::Tet8,
            "TETRA10" | "TET10" => Self::Tet10,
            "TET14" | "TETRA14" => Self::Tet14,
            "TET15" | "TETRA15" => Self::Tet15,
            "HEX" | "HEX8" | "HEXAHEDRON" => Self::Hex8,
            "HEX20" => Self::Hex20,
            "HEX27" => Self::Hex27,
            "WEDGE" | "WEDGE6" => Self::Wedge6,
            "WEDGE15" => Self::Wedge15,
            "WEDGE18" => Self::Wedge18,
            "PYRAMID" | "PYRAMID5" => Self::Pyramid5,
            "PYRAMID13" => Self::Pyramid13,
            "PYRAMID14" => Self::Pyramid14,
            "NSIDED" => Self::NSided,
            "NFACED" => Self::NFaced,
            _ => Self::Custom(s.to_string()),
        }
    }

    /// Get string representation
    pub fn as_str(&self) -> &str {
        match self {
            Self::Sphere => "SPHERE",
            Self::Bar2 => "BAR2",
            Self::Bar3 => "BAR3",
            Self::Tri3 => "TRI3",
            Self::Tri6 => "TRI6",
            Self::Tri7 => "TRI7",
            Self::Quad4 => "QUAD4",
            Self::Quad8 => "QUAD8",
            Self::Quad9 => "QUAD9",
            Self::Tet4 => "TET4",
            Self::Tet8 => "TET8",
            Self::Tet10 => "TET10",
            Self::Tet14 => "TET14",
            Self::Tet15 => "TET15",
            Self::Hex8 => "HEX8",
            Self::Hex20 => "HEX20",
            Self::Hex27 => "HEX27",
            Self::Wedge6 => "WEDGE6",
            Self::Wedge15 => "WEDGE15",
            Self::Wedge18 => "WEDGE18",
            Self::Pyramid5 => "PYRAMID5",
            Self::Pyramid13 => "PYRAMID13",
            Self::Pyramid14 => "PYRAMID14",
            Self::NSided => "NSIDED",
            Self::NFaced => "NFACED",
            Self::Custom(s) => s,
        }
    }

    /// Get expected number of nodes for standard topologies
    pub fn expected_nodes(&self) -> Option<usize> {
        match self {
            Self::Sphere => Some(1),
            Self::Bar2 => Some(2),
            Self::Bar3 => Some(3),
            Self::Tri3 => Some(3),
            Self::Tri6 => Some(6),
            Self::Tri7 => Some(7),
            Self::Quad4 => Some(4),
            Self::Quad8 => Some(8),
            Self::Quad9 => Some(9),
            Self::Tet4 => Some(4),
            Self::Tet8 => Some(8),
            Self::Tet10 => Some(10),
            Self::Tet14 => Some(14),
            Self::Tet15 => Some(15),
            Self::Hex8 => Some(8),
            Self::Hex20 => Some(20),
            Self::Hex27 => Some(27),
            Self::Wedge6 => Some(6),
            Self::Wedge15 => Some(15),
            Self::Wedge18 => Some(18),
            Self::Pyramid5 => Some(5),
            Self::Pyramid13 => Some(13),
            Self::Pyramid14 => Some(14),
            Self::NSided | Self::NFaced => None, // Variable
            Self::Custom(_) => None,
        }
    }
}

impl std::fmt::Display for Topology {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Structured connectivity with shape information
#[derive(Debug, Clone)]
pub struct Connectivity {
    /// Block ID this connectivity belongs to
    pub block_id: EntityId,
    /// Topology of elements in this block
    pub topology: Topology,
    /// Connectivity data (flat array)
    pub data: Vec<i64>,
    /// Number of entries (elements/edges/faces)
    pub num_entries: usize,
    /// Number of nodes per entry
    pub nodes_per_entry: usize,
}

impl Connectivity {
    /// Get connectivity for entry i (0-indexed)
    pub fn entry(&self, i: usize) -> Option<&[i64]> {
        if i >= self.num_entries {
            return None;
        }
        let start = i * self.nodes_per_entry;
        Some(&self.data[start..start + self.nodes_per_entry])
    }

    /// Get number of entries
    pub fn len(&self) -> usize {
        self.num_entries
    }

    /// Check if connectivity is empty
    pub fn is_empty(&self) -> bool {
        self.num_entries == 0
    }

    /// Iterator over entries
    pub fn iter(&self) -> ConnectivityIterator<'_> {
        ConnectivityIterator {
            connectivity: self,
            index: 0,
        }
    }
}

/// Iterator over connectivity entries
#[derive(Debug)]
pub struct ConnectivityIterator<'a> {
    connectivity: &'a Connectivity,
    index: usize,
}

impl<'a> Iterator for ConnectivityIterator<'a> {
    type Item = &'a [i64];

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.connectivity.entry(self.index);
        if result.is_some() {
            self.index += 1;
        }
        result
    }
}

/// File format type
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FileFormat {
    /// Classic NetCDF-3 format
    NetCdf3Classic,
    /// NetCDF-3 with 64-bit offsets
    NetCdf364BitOffset,
    /// NetCDF-4 format
    NetCdf4,
    /// NetCDF-4 classic model
    NetCdf4Classic,
    /// CDF-5 format
    NetCdfCdf5,
}

/// File creation mode
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CreateMode {
    /// Overwrite existing file
    Clobber,
    /// Fail if file exists
    NoClobber,
}

/// Floating point word size
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum FloatSize {
    /// 32-bit (4 byte) floats
    Float32,
    /// 64-bit (8 byte) doubles
    Float64,
}

/// Integer ID word size mode
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Int64Mode {
    /// Classic 32-bit IDs
    Int32,
    /// 64-bit IDs
    Int64,
}

/// Compression settings
#[derive(Debug, Copy, Clone)]
pub enum Compression {
    /// No compression
    None,
    /// Gzip compression with level (1-9)
    Gzip(u8),
    /// Szip compression
    Szip,
    /// Zstandard compression with level (1-9)
    Zstd(u8),
}

/// File creation options
#[derive(Debug, Clone)]
pub struct CreateOptions {
    /// Creation mode (clobber or noclobber)
    pub mode: CreateMode,
    /// Floating point size
    pub float_size: FloatSize,
    /// Integer ID size mode
    pub int64_mode: Int64Mode,
    /// Compression settings
    pub compression: Option<Compression>,
    /// Enable parallel I/O
    pub parallel: bool,
}

impl Default for CreateOptions {
    fn default() -> Self {
        Self {
            mode: CreateMode::NoClobber,
            float_size: FloatSize::Float64,
            int64_mode: Int64Mode::Int64,
            compression: None,
            parallel: false,
        }
    }
}
