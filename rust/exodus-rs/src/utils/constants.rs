//! Constants used in Exodus files
//!
//! This module centralizes all NetCDF dimension and variable names, as well as
//! Exodus format constants, to prevent typos and improve maintainability.

// =============================================================================
// Exodus Format Constants
// =============================================================================

/// Current API version
pub const API_VERSION: f32 = 9.04;

/// Exodus file format version
pub const FILE_VERSION: f32 = 2.0;

/// Maximum length for title string
pub const MAX_TITLE_LENGTH: usize = 80;

/// Maximum length for QA record strings
pub const MAX_QA_STRING_LENGTH: usize = 32;

/// Maximum length for info record strings
pub const MAX_INFO_STRING_LENGTH: usize = 80;

/// Maximum length for entity/variable names
pub const MAX_NAME_LENGTH: usize = 32;

// =============================================================================
// Global Attribute Names
// =============================================================================

/// Title attribute name
pub const ATTR_TITLE: &str = "title";

/// API version attribute name
pub const ATTR_API_VERSION: &str = "api_version";

/// File version attribute name
pub const ATTR_VERSION: &str = "version";

// =============================================================================
// Common Dimension Names
// =============================================================================

/// Number of spatial dimensions
pub const DIM_NUM_DIM: &str = "num_dim";

/// Number of nodes
pub const DIM_NUM_NODES: &str = "num_nodes";

/// Number of elements
pub const DIM_NUM_ELEM: &str = "num_elem";

/// Number of edges
pub const DIM_NUM_EDGE: &str = "num_edge";

/// Number of faces
pub const DIM_NUM_FACE: &str = "num_face";

/// Number of element blocks
pub const DIM_NUM_EL_BLK: &str = "num_el_blk";

/// Number of edge blocks
pub const DIM_NUM_ED_BLK: &str = "num_ed_blk";

/// Number of face blocks
pub const DIM_NUM_FA_BLK: &str = "num_fa_blk";

/// Number of node sets
pub const DIM_NUM_NODE_SETS: &str = "num_node_sets";

/// Number of edge sets
pub const DIM_NUM_EDGE_SETS: &str = "num_edge_sets";

/// Number of face sets
pub const DIM_NUM_FACE_SETS: &str = "num_face_sets";

/// Number of side sets
pub const DIM_NUM_SIDE_SETS: &str = "num_side_sets";

/// Number of element sets
pub const DIM_NUM_ELEM_SETS: &str = "num_elem_sets";

/// Time step dimension (unlimited)
pub const DIM_TIME_STEP: &str = "time_step";

/// String length dimension (for variable names, etc.)
pub const DIM_LEN_STRING: &str = "len_string";

/// Name length dimension
pub const DIM_LEN_NAME: &str = "len_name";

/// Line length dimension (for info records)
pub const DIM_LEN_LINE: &str = "len_line";

/// Number of QA records
pub const DIM_NUM_QA_REC: &str = "num_qa_rec";

/// Number of QA dimensions (always 4: code, version, date, time)
pub const DIM_NUM_QA_DIM: &str = "num_qa_dim";

/// Number of info records
pub const DIM_NUM_INFO: &str = "num_info";

/// Number of global variables
pub const DIM_NUM_GLO_VAR: &str = "num_glo_var";

/// Number of nodal variables
pub const DIM_NUM_NOD_VAR: &str = "num_nod_var";

/// Number of element variables
pub const DIM_NUM_ELEM_VAR: &str = "num_elem_var";

/// Number of edge variables
pub const DIM_NUM_EDGE_VAR: &str = "num_edge_var";

/// Number of face variables
pub const DIM_NUM_FACE_VAR: &str = "num_face_var";

/// Number of node maps
pub const DIM_NUM_NODE_MAPS: &str = "num_node_maps";

/// Number of element maps
pub const DIM_NUM_ELEM_MAPS: &str = "num_elem_maps";

/// Number of edge maps
pub const DIM_NUM_EDGE_MAPS: &str = "num_edge_maps";

/// Number of face maps
pub const DIM_NUM_FACE_MAPS: &str = "num_face_maps";

/// Number of assemblies
pub const DIM_NUM_ASSEMBLY: &str = "num_assembly";

/// Number of blobs
pub const DIM_NUM_BLOB: &str = "num_blob";

// =============================================================================
// Common Variable Names
// =============================================================================

/// X-coordinate variable name
pub const VAR_COORD_X: &str = "coordx";

/// Y-coordinate variable name
pub const VAR_COORD_Y: &str = "coordy";

/// Z-coordinate variable name
pub const VAR_COORD_Z: &str = "coordz";

/// Coordinate names variable
pub const VAR_COOR_NAMES: &str = "coor_names";

/// Time whole variable (time values for each step)
pub const VAR_TIME_WHOLE: &str = "time_whole";

/// Global variables storage (combined format)
pub const VAR_VALS_GLO_VAR: &str = "vals_glo_var";

/// Global variable names
pub const VAR_NAME_GLO_VAR: &str = "name_glo_var";

/// Nodal variable names
pub const VAR_NAME_NOD_VAR: &str = "name_nod_var";

/// Element variable names
pub const VAR_NAME_ELEM_VAR: &str = "name_elem_var";

/// Edge variable names
pub const VAR_NAME_EDGE_VAR: &str = "name_edge_var";

/// Face variable names
pub const VAR_NAME_FACE_VAR: &str = "name_face_var";

/// Element variable truth table
pub const VAR_ELEM_VAR_TAB: &str = "elem_var_tab";

/// Edge variable truth table
pub const VAR_EDGE_VAR_TAB: &str = "edge_var_tab";

/// Face variable truth table
pub const VAR_FACE_VAR_TAB: &str = "face_var_tab";

/// QA records variable
pub const VAR_QA_RECORDS: &str = "qa_records";

/// Info records variable
pub const VAR_INFO_RECORDS: &str = "info_records";
