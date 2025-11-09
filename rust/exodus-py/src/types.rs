//! Core type bindings for Exodus II data structures

use pyo3::prelude::*;
use pyo3::types::PyDict;
use exodus_rs::types as rs;

/// Entity types in Exodus II
#[pyclass]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityType {
    /// Element block
    ElemBlock,
    /// Node set
    NodeSet,
    /// Side set
    SideSet,
    /// Node map
    NodeMap,
    /// Element map
    ElemMap,
    /// Edge block
    EdgeBlock,
    /// Edge set
    EdgeSet,
    /// Face block
    FaceBlock,
    /// Face set
    FaceSet,
    /// Element set
    ElemSet,
    /// Edge map
    EdgeMap,
    /// Face map
    FaceMap,
    /// Global variables
    Global,
    /// Nodal variables
    Nodal,
    /// Assembly
    Assembly,
    /// Blob
    Blob,
}

#[pymethods]
impl EntityType {
    fn __str__(&self) -> &'static str {
        self.to_rust().as_str()
    }

    fn __repr__(&self) -> String {
        format!("EntityType.{:?}", self)
    }
}

impl EntityType {
    /// Convert to Rust enum
    pub fn to_rust(&self) -> rs::EntityType {
        match self {
            EntityType::ElemBlock => rs::EntityType::ElemBlock,
            EntityType::NodeSet => rs::EntityType::NodeSet,
            EntityType::SideSet => rs::EntityType::SideSet,
            EntityType::NodeMap => rs::EntityType::NodeMap,
            EntityType::ElemMap => rs::EntityType::ElemMap,
            EntityType::EdgeBlock => rs::EntityType::EdgeBlock,
            EntityType::EdgeSet => rs::EntityType::EdgeSet,
            EntityType::FaceBlock => rs::EntityType::FaceBlock,
            EntityType::FaceSet => rs::EntityType::FaceSet,
            EntityType::ElemSet => rs::EntityType::ElemSet,
            EntityType::EdgeMap => rs::EntityType::EdgeMap,
            EntityType::FaceMap => rs::EntityType::FaceMap,
            EntityType::Global => rs::EntityType::Global,
            EntityType::Nodal => rs::EntityType::Nodal,
            EntityType::Assembly => rs::EntityType::Assembly,
            EntityType::Blob => rs::EntityType::Blob,
        }
    }

    /// Convert from Rust enum
    pub fn from_rust(entity_type: rs::EntityType) -> Self {
        match entity_type {
            rs::EntityType::ElemBlock => EntityType::ElemBlock,
            rs::EntityType::NodeSet => EntityType::NodeSet,
            rs::EntityType::SideSet => EntityType::SideSet,
            rs::EntityType::NodeMap => EntityType::NodeMap,
            rs::EntityType::ElemMap => EntityType::ElemMap,
            rs::EntityType::EdgeBlock => EntityType::EdgeBlock,
            rs::EntityType::EdgeSet => EntityType::EdgeSet,
            rs::EntityType::FaceBlock => EntityType::FaceBlock,
            rs::EntityType::FaceSet => EntityType::FaceSet,
            rs::EntityType::ElemSet => EntityType::ElemSet,
            rs::EntityType::EdgeMap => EntityType::EdgeMap,
            rs::EntityType::FaceMap => EntityType::FaceMap,
            rs::EntityType::Global => EntityType::Global,
            rs::EntityType::Nodal => EntityType::Nodal,
            rs::EntityType::Assembly => EntityType::Assembly,
            rs::EntityType::Blob => EntityType::Blob,
        }
    }
}

/// File creation mode
#[pyclass]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreateMode {
    /// Overwrite existing file
    Clobber,
    /// Fail if file exists
    NoClobber,
}

#[pymethods]
impl CreateMode {
    fn __str__(&self) -> &'static str {
        match self {
            CreateMode::Clobber => "Clobber",
            CreateMode::NoClobber => "NoClobber",
        }
    }
}

impl CreateMode {
    pub fn to_rust(&self) -> rs::CreateMode {
        match self {
            CreateMode::Clobber => rs::CreateMode::Clobber,
            CreateMode::NoClobber => rs::CreateMode::NoClobber,
        }
    }
}

/// Floating point precision
#[pyclass]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatSize {
    /// 32-bit floats
    Float32,
    /// 64-bit floats (default)
    Float64,
}

#[pymethods]
impl FloatSize {
    fn __str__(&self) -> &'static str {
        match self {
            FloatSize::Float32 => "Float32",
            FloatSize::Float64 => "Float64",
        }
    }
}

impl FloatSize {
    pub fn to_rust(&self) -> rs::FloatSize {
        match self {
            FloatSize::Float32 => rs::FloatSize::Float32,
            FloatSize::Float64 => rs::FloatSize::Float64,
        }
    }
}

/// Integer storage mode
#[pyclass]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Int64Mode {
    /// 32-bit integers
    Int32,
    /// 64-bit integers (default)
    Int64,
}

#[pymethods]
impl Int64Mode {
    fn __str__(&self) -> &'static str {
        match self {
            Int64Mode::Int32 => "Int32",
            Int64Mode::Int64 => "Int64",
        }
    }
}

impl Int64Mode {
    pub fn to_rust(&self) -> rs::Int64Mode {
        match self {
            Int64Mode::Int32 => rs::Int64Mode::Int32,
            Int64Mode::Int64 => rs::Int64Mode::Int64,
        }
    }
}

/// Attribute value type
#[pyclass]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeType {
    /// Integer attribute
    Integer,
    /// Double precision float
    Double,
    /// Character/string
    Char,
}

#[pymethods]
impl AttributeType {
    fn __str__(&self) -> &'static str {
        match self {
            AttributeType::Integer => "Integer",
            AttributeType::Double => "Double",
            AttributeType::Char => "Char",
        }
    }
}

impl AttributeType {
    pub fn to_rust(&self) -> rs::AttributeType {
        match self {
            AttributeType::Integer => rs::AttributeType::Integer,
            AttributeType::Double => rs::AttributeType::Double,
            AttributeType::Char => rs::AttributeType::Char,
        }
    }

    pub fn from_rust(attr_type: rs::AttributeType) -> Self {
        match attr_type {
            rs::AttributeType::Integer => AttributeType::Integer,
            rs::AttributeType::Double => AttributeType::Double,
            rs::AttributeType::Char => AttributeType::Char,
        }
    }
}

/// Initialization parameters for Exodus database
#[pyclass]
#[derive(Debug, Clone)]
pub struct InitParams {
    #[pyo3(get, set)]
    /// Database title (max 80 characters)
    pub title: String,
    #[pyo3(get, set)]
    /// Number of spatial dimensions (1, 2, or 3)
    pub num_dim: usize,
    #[pyo3(get, set)]
    /// Number of nodes
    pub num_nodes: usize,
    #[pyo3(get, set)]
    /// Number of edges
    pub num_edges: usize,
    #[pyo3(get, set)]
    /// Number of edge blocks
    pub num_edge_blocks: usize,
    #[pyo3(get, set)]
    /// Number of faces
    pub num_faces: usize,
    #[pyo3(get, set)]
    /// Number of face blocks
    pub num_face_blocks: usize,
    #[pyo3(get, set)]
    /// Number of elements
    pub num_elems: usize,
    #[pyo3(get, set)]
    /// Number of element blocks
    pub num_elem_blocks: usize,
    #[pyo3(get, set)]
    /// Number of node sets
    pub num_node_sets: usize,
    #[pyo3(get, set)]
    /// Number of edge sets
    pub num_edge_sets: usize,
    #[pyo3(get, set)]
    /// Number of face sets
    pub num_face_sets: usize,
    #[pyo3(get, set)]
    /// Number of side sets
    pub num_side_sets: usize,
    #[pyo3(get, set)]
    /// Number of element sets
    pub num_elem_sets: usize,
    #[pyo3(get, set)]
    /// Number of node maps
    pub num_node_maps: usize,
    #[pyo3(get, set)]
    /// Number of edge maps
    pub num_edge_maps: usize,
    #[pyo3(get, set)]
    /// Number of face maps
    pub num_face_maps: usize,
    #[pyo3(get, set)]
    /// Number of element maps
    pub num_elem_maps: usize,
    #[pyo3(get, set)]
    /// Number of assemblies
    pub num_assemblies: usize,
    #[pyo3(get, set)]
    /// Number of blobs
    pub num_blobs: usize,
}

#[pymethods]
impl InitParams {
    #[new]
    #[pyo3(signature = (title="", num_dim=3, num_nodes=0, num_elems=0, num_elem_blocks=0, **kwargs))]
    fn new(
        title: &str,
        num_dim: usize,
        num_nodes: usize,
        num_elems: usize,
        num_elem_blocks: usize,
        kwargs: Option<&PyDict>,
    ) -> PyResult<Self> {
        let mut params = InitParams {
            title: title.to_string(),
            num_dim,
            num_nodes,
            num_elems,
            num_elem_blocks,
            num_edges: 0,
            num_edge_blocks: 0,
            num_faces: 0,
            num_face_blocks: 0,
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
        };

        // Process kwargs for optional fields
        if let Some(kw) = kwargs {
            if let Some(val) = kw.get_item("num_edges")? {
                params.num_edges = val.extract()?;
            }
            if let Some(val) = kw.get_item("num_edge_blocks")? {
                params.num_edge_blocks = val.extract()?;
            }
            if let Some(val) = kw.get_item("num_faces")? {
                params.num_faces = val.extract()?;
            }
            if let Some(val) = kw.get_item("num_face_blocks")? {
                params.num_face_blocks = val.extract()?;
            }
            if let Some(val) = kw.get_item("num_node_sets")? {
                params.num_node_sets = val.extract()?;
            }
            if let Some(val) = kw.get_item("num_side_sets")? {
                params.num_side_sets = val.extract()?;
            }
            if let Some(val) = kw.get_item("num_assemblies")? {
                params.num_assemblies = val.extract()?;
            }
            if let Some(val) = kw.get_item("num_blobs")? {
                params.num_blobs = val.extract()?;
            }
        }

        Ok(params)
    }

    fn __repr__(&self) -> String {
        format!(
            "InitParams(title='{}', num_dim={}, num_nodes={}, num_elems={}, num_elem_blocks={})",
            self.title, self.num_dim, self.num_nodes, self.num_elems, self.num_elem_blocks
        )
    }
}

impl InitParams {
    pub fn to_rust(&self) -> rs::InitParams {
        rs::InitParams {
            title: self.title.clone(),
            num_dim: self.num_dim,
            num_nodes: self.num_nodes,
            num_edges: self.num_edges,
            num_edge_blocks: self.num_edge_blocks,
            num_faces: self.num_faces,
            num_face_blocks: self.num_face_blocks,
            num_elems: self.num_elems,
            num_elem_blocks: self.num_elem_blocks,
            num_node_sets: self.num_node_sets,
            num_edge_sets: self.num_edge_sets,
            num_face_sets: self.num_face_sets,
            num_side_sets: self.num_side_sets,
            num_elem_sets: self.num_elem_sets,
            num_node_maps: self.num_node_maps,
            num_edge_maps: self.num_edge_maps,
            num_face_maps: self.num_face_maps,
            num_elem_maps: self.num_elem_maps,
            num_assemblies: self.num_assemblies,
            num_blobs: self.num_blobs,
        }
    }

    pub fn from_rust(params: &rs::InitParams) -> Self {
        InitParams {
            title: params.title.clone(),
            num_dim: params.num_dim,
            num_nodes: params.num_nodes,
            num_edges: params.num_edges,
            num_edge_blocks: params.num_edge_blocks,
            num_faces: params.num_faces,
            num_face_blocks: params.num_face_blocks,
            num_elems: params.num_elems,
            num_elem_blocks: params.num_elem_blocks,
            num_node_sets: params.num_node_sets,
            num_edge_sets: params.num_edge_sets,
            num_face_sets: params.num_face_sets,
            num_side_sets: params.num_side_sets,
            num_elem_sets: params.num_elem_sets,
            num_node_maps: params.num_node_maps,
            num_edge_maps: params.num_edge_maps,
            num_face_maps: params.num_face_maps,
            num_elem_maps: params.num_elem_maps,
            num_assemblies: params.num_assemblies,
            num_blobs: params.num_blobs,
        }
    }
}

/// File creation options
#[pyclass]
#[derive(Debug, Clone)]
pub struct CreateOptions {
    #[pyo3(get, set)]
    /// Creation mode (Clobber or NoClobber)
    pub mode: CreateMode,
    #[pyo3(get, set)]
    /// Floating point precision
    pub float_size: FloatSize,
    #[pyo3(get, set)]
    /// Integer storage mode
    pub int64_mode: Int64Mode,
}

#[pymethods]
impl CreateOptions {
    #[new]
    #[pyo3(signature = (mode=CreateMode::Clobber, float_size=FloatSize::Float64, int64_mode=Int64Mode::Int64))]
    fn new(mode: CreateMode, float_size: FloatSize, int64_mode: Int64Mode) -> Self {
        CreateOptions {
            mode,
            float_size,
            int64_mode,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "CreateOptions(mode={}, float_size={}, int64_mode={})",
            self.mode.__str__(),
            self.float_size.__str__(),
            self.int64_mode.__str__()
        )
    }
}

impl CreateOptions {
    pub fn to_rust(&self) -> rs::CreateOptions {
        rs::CreateOptions {
            mode: self.mode.to_rust(),
            float_size: self.float_size.to_rust(),
            int64_mode: self.int64_mode.to_rust(),
            ..Default::default()
        }
    }
}

/// Element/Edge/Face block definition
#[pyclass]
#[derive(Debug, Clone)]
pub struct Block {
    #[pyo3(get, set)]
    /// Block ID
    pub id: i64,
    #[pyo3(get, set)]
    /// Entity type (ElemBlock, EdgeBlock, or FaceBlock)
    pub entity_type: EntityType,
    #[pyo3(get, set)]
    /// Topology name (e.g., "HEX8", "QUAD4")
    pub topology: String,
    #[pyo3(get, set)]
    /// Number of entries (elements/edges/faces)
    pub num_entries: usize,
    #[pyo3(get, set)]
    /// Number of nodes per entry
    pub num_nodes_per_entry: usize,
    #[pyo3(get, set)]
    /// Number of edges per entry
    pub num_edges_per_entry: usize,
    #[pyo3(get, set)]
    /// Number of faces per entry
    pub num_faces_per_entry: usize,
    #[pyo3(get, set)]
    /// Number of attributes per entry
    pub num_attributes: usize,
}

#[pymethods]
impl Block {
    #[new]
    #[pyo3(signature = (id, entity_type, topology, num_entries, num_nodes_per_entry, num_edges_per_entry=0, num_faces_per_entry=0, num_attributes=0))]
    fn new(
        id: i64,
        entity_type: EntityType,
        topology: String,
        num_entries: usize,
        num_nodes_per_entry: usize,
        num_edges_per_entry: usize,
        num_faces_per_entry: usize,
        num_attributes: usize,
    ) -> Self {
        Block {
            id,
            entity_type,
            topology,
            num_entries,
            num_nodes_per_entry,
            num_edges_per_entry,
            num_faces_per_entry,
            num_attributes,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "Block(id={}, type={}, topology='{}', num_entries={}, nodes_per_entry={})",
            self.id,
            self.entity_type.__str__(),
            self.topology,
            self.num_entries,
            self.num_nodes_per_entry
        )
    }
}

impl Block {
    pub fn to_rust(&self) -> rs::Block {
        rs::Block {
            id: self.id,
            entity_type: self.entity_type.to_rust(),
            topology: self.topology.clone(),
            num_entries: self.num_entries,
            num_nodes_per_entry: self.num_nodes_per_entry,
            num_edges_per_entry: self.num_edges_per_entry,
            num_faces_per_entry: self.num_faces_per_entry,
            num_attributes: self.num_attributes,
        }
    }

    pub fn from_rust(block: &rs::Block) -> Self {
        Block {
            id: block.id,
            entity_type: EntityType::from_rust(block.entity_type),
            topology: block.topology.clone(),
            num_entries: block.num_entries,
            num_nodes_per_entry: block.num_nodes_per_entry,
            num_edges_per_entry: block.num_edges_per_entry,
            num_faces_per_entry: block.num_faces_per_entry,
            num_attributes: block.num_attributes,
        }
    }
}

/// Node set definition
#[pyclass]
#[derive(Debug, Clone)]
pub struct NodeSet {
    #[pyo3(get, set)]
    /// Set ID
    pub id: i64,
    #[pyo3(get, set)]
    /// Node IDs in the set
    pub nodes: Vec<i64>,
    #[pyo3(get, set)]
    /// Distribution factors (optional)
    pub dist_factors: Vec<f64>,
}

#[pymethods]
impl NodeSet {
    #[new]
    #[pyo3(signature = (id, nodes, dist_factors=Vec::new()))]
    fn new(id: i64, nodes: Vec<i64>, dist_factors: Vec<f64>) -> Self {
        NodeSet {
            id,
            nodes,
            dist_factors,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "NodeSet(id={}, num_nodes={})",
            self.id,
            self.nodes.len()
        )
    }
}

impl NodeSet {
    pub fn to_rust(&self) -> rs::NodeSet {
        rs::NodeSet {
            id: self.id,
            nodes: self.nodes.clone(),
            dist_factors: self.dist_factors.clone(),
        }
    }

    pub fn from_rust(ns: &rs::NodeSet) -> Self {
        NodeSet {
            id: ns.id,
            nodes: ns.nodes.clone(),
            dist_factors: ns.dist_factors.clone(),
        }
    }
}

/// Side set definition
#[pyclass]
#[derive(Debug, Clone)]
pub struct SideSet {
    #[pyo3(get, set)]
    /// Set ID
    pub id: i64,
    #[pyo3(get, set)]
    /// Element IDs
    pub elements: Vec<i64>,
    #[pyo3(get, set)]
    /// Side numbers
    pub sides: Vec<i64>,
    #[pyo3(get, set)]
    /// Distribution factors (optional)
    pub dist_factors: Vec<f64>,
}

#[pymethods]
impl SideSet {
    #[new]
    #[pyo3(signature = (id, elements, sides, dist_factors=Vec::new()))]
    fn new(id: i64, elements: Vec<i64>, sides: Vec<i64>, dist_factors: Vec<f64>) -> Self {
        SideSet {
            id,
            elements,
            sides,
            dist_factors,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "SideSet(id={}, num_sides={})",
            self.id,
            self.elements.len()
        )
    }
}

impl SideSet {
    pub fn to_rust(&self) -> rs::SideSet {
        rs::SideSet {
            id: self.id,
            elements: self.elements.clone(),
            sides: self.sides.clone(),
            dist_factors: self.dist_factors.clone(),
        }
    }

    pub fn from_rust(ss: &rs::SideSet) -> Self {
        SideSet {
            id: ss.id,
            elements: ss.elements.clone(),
            sides: ss.sides.clone(),
            dist_factors: ss.dist_factors.clone(),
        }
    }
}

/// Entity set (edge, face, or element set)
#[pyclass]
#[derive(Debug, Clone)]
pub struct EntitySet {
    #[pyo3(get, set)]
    /// Set ID
    pub id: i64,
    #[pyo3(get, set)]
    /// Entity type
    pub entity_type: EntityType,
    #[pyo3(get, set)]
    /// Entity IDs in the set
    pub entities: Vec<i64>,
}

#[pymethods]
impl EntitySet {
    #[new]
    fn new(id: i64, entity_type: EntityType, entities: Vec<i64>) -> Self {
        EntitySet {
            id,
            entity_type,
            entities,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "EntitySet(id={}, type={}, num_entities={})",
            self.id,
            self.entity_type.__str__(),
            self.entities.len()
        )
    }
}

impl EntitySet {
    pub fn to_rust(&self) -> rs::EntitySet {
        rs::EntitySet {
            id: self.id,
            entity_type: self.entity_type.to_rust(),
            entities: self.entities.clone(),
        }
    }

    pub fn from_rust(es: &rs::EntitySet) -> Self {
        EntitySet {
            id: es.id,
            entity_type: EntityType::from_rust(es.entity_type),
            entities: es.entities.clone(),
        }
    }
}

/// Assembly (hierarchical grouping)
#[pyclass]
#[derive(Debug, Clone)]
pub struct Assembly {
    #[pyo3(get, set)]
    /// Assembly ID
    pub id: i64,
    #[pyo3(get, set)]
    /// Assembly name
    pub name: String,
    #[pyo3(get, set)]
    /// Entity type of members
    pub entity_type: EntityType,
    #[pyo3(get, set)]
    /// List of entity IDs
    pub entity_list: Vec<i64>,
}

#[pymethods]
impl Assembly {
    #[new]
    fn new(id: i64, name: String, entity_type: EntityType, entity_list: Vec<i64>) -> Self {
        Assembly {
            id,
            name,
            entity_type,
            entity_list,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "Assembly(id={}, name='{}', type={}, num_entities={})",
            self.id,
            self.name,
            self.entity_type.__str__(),
            self.entity_list.len()
        )
    }
}

impl Assembly {
    pub fn to_rust(&self) -> rs::Assembly {
        rs::Assembly {
            id: self.id,
            name: self.name.clone(),
            entity_type: self.entity_type.to_rust(),
            entity_list: self.entity_list.clone(),
        }
    }

    pub fn from_rust(asm: &rs::Assembly) -> Self {
        Assembly {
            id: asm.id,
            name: asm.name.clone(),
            entity_type: EntityType::from_rust(asm.entity_type),
            entity_list: asm.entity_list.clone(),
        }
    }
}

/// Blob (arbitrary binary data)
#[pyclass]
#[derive(Debug, Clone)]
pub struct Blob {
    #[pyo3(get, set)]
    /// Blob ID
    pub id: i64,
    #[pyo3(get, set)]
    /// Blob name
    pub name: String,
}

#[pymethods]
impl Blob {
    #[new]
    fn new(id: i64, name: String) -> Self {
        Blob { id, name }
    }

    fn __repr__(&self) -> String {
        format!("Blob(id={}, name='{}')", self.id, self.name)
    }
}

impl Blob {
    pub fn to_rust(&self) -> rs::Blob {
        rs::Blob {
            id: self.id,
            name: self.name.clone(),
        }
    }

    pub fn from_rust(blob: &rs::Blob) -> Self {
        Blob {
            id: blob.id,
            name: blob.name.clone(),
        }
    }
}

/// QA record for provenance tracking
#[pyclass]
#[derive(Debug, Clone)]
pub struct QaRecord {
    #[pyo3(get, set)]
    /// Code name
    pub code_name: String,
    #[pyo3(get, set)]
    /// Code version
    pub code_version: String,
    #[pyo3(get, set)]
    /// Date string
    pub date: String,
    #[pyo3(get, set)]
    /// Time string
    pub time: String,
}

#[pymethods]
impl QaRecord {
    #[new]
    fn new(code_name: String, code_version: String, date: String, time: String) -> Self {
        QaRecord {
            code_name,
            code_version,
            date,
            time,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "QaRecord(code='{}', version='{}', date='{}', time='{}')",
            self.code_name, self.code_version, self.date, self.time
        )
    }
}

impl QaRecord {
    pub fn to_rust(&self) -> rs::QaRecord {
        rs::QaRecord {
            code_name: self.code_name.clone(),
            code_version: self.code_version.clone(),
            date: self.date.clone(),
            time: self.time.clone(),
        }
    }

    pub fn from_rust(qa: &rs::QaRecord) -> Self {
        QaRecord {
            code_name: qa.code_name.clone(),
            code_version: qa.code_version.clone(),
            date: qa.date.clone(),
            time: qa.time.clone(),
        }
    }
}
