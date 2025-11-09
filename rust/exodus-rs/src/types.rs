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
