//! Helper functions for generating NetCDF variable and dimension names.
//!
//! This module centralizes the naming conventions used in Exodus II files
//! to reduce string duplication and prevent typos.

#![allow(dead_code)] // Many functions provided for future use

use crate::types::EntityType;

// =============================================================================
// Dimension Names
// =============================================================================

/// Get the dimension name for the count of entities of a type.
pub fn num_dimension(entity_type: EntityType) -> &'static str {
    match entity_type {
        EntityType::Global => "num_glo_var",
        EntityType::Nodal => "num_nodes",
        EntityType::ElemBlock => "num_el_blk",
        EntityType::EdgeBlock => "num_ed_blk",
        EntityType::FaceBlock => "num_fa_blk",
        EntityType::NodeSet => "num_node_sets",
        EntityType::EdgeSet => "num_edge_sets",
        EntityType::FaceSet => "num_face_sets",
        EntityType::SideSet => "num_side_sets",
        EntityType::ElemSet => "num_elem_sets",
        EntityType::Assembly => "num_assembly",
        EntityType::Blob => "num_blob",
        EntityType::NodeMap => "num_node_maps",
        EntityType::ElemMap => "num_elem_maps",
        EntityType::EdgeMap => "num_edge_maps",
        EntityType::FaceMap => "num_face_maps",
    }
}

/// Get the dimension name for elements in a block.
pub fn block_entries_dim(entity_type: EntityType, block_index: usize) -> String {
    match entity_type {
        EntityType::ElemBlock => format!("num_el_in_blk{}", block_index + 1),
        EntityType::EdgeBlock => format!("num_ed_in_blk{}", block_index + 1),
        EntityType::FaceBlock => format!("num_fa_in_blk{}", block_index + 1),
        _ => panic!("Not a block type: {}", entity_type),
    }
}

/// Get the dimension name for nodes per entry in a block.
pub fn block_nodes_per_entry_dim(entity_type: EntityType, block_index: usize) -> String {
    match entity_type {
        EntityType::ElemBlock => format!("num_nod_per_el{}", block_index + 1),
        EntityType::EdgeBlock => format!("num_nod_per_ed{}", block_index + 1),
        EntityType::FaceBlock => format!("num_nod_per_fa{}", block_index + 1),
        _ => panic!("Not a block type: {}", entity_type),
    }
}

/// Get the dimension name for entries in a set.
pub fn set_entries_dim(entity_type: EntityType, set_index: usize) -> String {
    match entity_type {
        EntityType::NodeSet => format!("num_nod_ns{}", set_index + 1),
        EntityType::EdgeSet => format!("num_edge_es{}", set_index + 1),
        EntityType::FaceSet => format!("num_face_fs{}", set_index + 1),
        EntityType::SideSet => format!("num_side_ss{}", set_index + 1),
        EntityType::ElemSet => format!("num_ele_els{}", set_index + 1),
        _ => panic!("Not a set type: {}", entity_type),
    }
}

/// Get the dimension name for distribution factors in a set.
pub fn set_dist_factors_dim(entity_type: EntityType, set_index: usize) -> String {
    match entity_type {
        EntityType::NodeSet => format!("num_df_ns{}", set_index + 1),
        EntityType::EdgeSet => format!("num_df_es{}", set_index + 1),
        EntityType::FaceSet => format!("num_df_fs{}", set_index + 1),
        EntityType::SideSet => format!("num_df_ss{}", set_index + 1),
        EntityType::ElemSet => format!("num_df_els{}", set_index + 1),
        _ => panic!("Not a set type: {}", entity_type),
    }
}

/// Get the dimension name for attributes in a block.
pub fn block_attributes_dim(block_index: usize) -> String {
    format!("num_att_in_blk{}", block_index + 1)
}

/// Get the dimension name for number of variables of a given type.
pub fn num_variables_dim(entity_type: EntityType) -> &'static str {
    match entity_type {
        EntityType::Global => "num_glo_var",
        EntityType::Nodal => "num_nod_var",
        EntityType::ElemBlock => "num_elem_var",
        EntityType::EdgeBlock => "num_edge_var",
        EntityType::FaceBlock => "num_face_var",
        EntityType::NodeSet => "num_nset_var",
        EntityType::EdgeSet => "num_eset_var",
        EntityType::FaceSet => "num_fset_var",
        EntityType::SideSet => "num_sset_var",
        EntityType::ElemSet => "num_elset_var",
        _ => panic!("Invalid variable type: {}", entity_type),
    }
}

// =============================================================================
// Variable Names
// =============================================================================

/// Get the property variable name for entity IDs.
pub fn prop_id_var(entity_type: EntityType) -> &'static str {
    match entity_type {
        EntityType::ElemBlock => "eb_prop1",
        EntityType::EdgeBlock => "ed_prop1",
        EntityType::FaceBlock => "fa_prop1",
        EntityType::NodeSet => "ns_prop1",
        EntityType::EdgeSet => "es_prop1",
        EntityType::FaceSet => "fs_prop1",
        EntityType::SideSet => "ss_prop1",
        EntityType::ElemSet => "els_prop1",
        _ => panic!("No property variable for type: {}", entity_type),
    }
}

/// Get the connectivity variable name for a block.
pub fn connectivity_var(block_index: usize) -> String {
    format!("connect{}", block_index + 1)
}

/// Get the attribute variable name for a block.
pub fn block_attribute_var(block_index: usize) -> String {
    format!("attrib{}", block_index + 1)
}

/// Get the attribute name variable for a block.
pub fn block_attribute_name_var(block_index: usize) -> String {
    format!("attrib_name{}", block_index + 1)
}

/// Get the variable name storage variable for an entity type.
pub fn variable_names_var(entity_type: EntityType) -> &'static str {
    match entity_type {
        EntityType::Global => "name_glo_var",
        EntityType::Nodal => "name_nod_var",
        EntityType::ElemBlock => "name_elem_var",
        EntityType::EdgeBlock => "name_edge_var",
        EntityType::FaceBlock => "name_face_var",
        EntityType::NodeSet => "name_nset_var",
        EntityType::EdgeSet => "name_eset_var",
        EntityType::FaceSet => "name_fset_var",
        EntityType::SideSet => "name_sset_var",
        EntityType::ElemSet => "name_elset_var",
        _ => panic!("Invalid variable type: {}", entity_type),
    }
}

/// Get the reduction variable names storage variable for an entity type.
pub fn reduction_variable_names_var(entity_type: EntityType) -> &'static str {
    match entity_type {
        EntityType::Global => "name_glo_var",
        EntityType::ElemBlock => "name_ele_red_var",
        EntityType::EdgeBlock => "name_edg_red_var",
        EntityType::FaceBlock => "name_fac_red_var",
        EntityType::NodeSet => "name_nset_red_var",
        EntityType::EdgeSet => "name_eset_red_var",
        EntityType::FaceSet => "name_fset_red_var",
        EntityType::SideSet => "name_sset_red_var",
        EntityType::ElemSet => "name_elset_red_var",
        EntityType::Assembly => "name_assembly_red_var",
        EntityType::Blob => "name_blob_red_var",
        _ => panic!("Invalid reduction variable type: {}", entity_type),
    }
}

/// Get the nodal variable storage variable name (separate format).
pub fn nodal_var(var_index: usize) -> String {
    format!("vals_nod_var{}", var_index + 1)
}

/// Get the element variable storage variable name (separate format).
pub fn elem_var(var_index: usize, block_index: usize) -> String {
    format!("vals_elem_var{}eb{}", var_index + 1, block_index + 1)
}

/// Get the edge variable storage variable name (separate format).
pub fn edge_var(var_index: usize, block_index: usize) -> String {
    format!("vals_edge_var{}edb{}", var_index + 1, block_index + 1)
}

/// Get the face variable storage variable name (separate format).
pub fn face_var(var_index: usize, block_index: usize) -> String {
    format!("vals_face_var{}fab{}", var_index + 1, block_index + 1)
}

/// Get the node set variable storage variable name.
pub fn nodeset_var(var_index: usize, set_index: usize) -> String {
    format!("vals_nset_var{}ns{}", var_index + 1, set_index + 1)
}

/// Get the edge set variable storage variable name.
pub fn edgeset_var(var_index: usize, set_index: usize) -> String {
    format!("vals_eset_var{}es{}", var_index + 1, set_index + 1)
}

/// Get the face set variable storage variable name.
pub fn faceset_var(var_index: usize, set_index: usize) -> String {
    format!("vals_fset_var{}fs{}", var_index + 1, set_index + 1)
}

/// Get the side set variable storage variable name.
pub fn sideset_var(var_index: usize, set_index: usize) -> String {
    format!("vals_sset_var{}ss{}", var_index + 1, set_index + 1)
}

/// Get the element set variable storage variable name.
pub fn elemset_var(var_index: usize, set_index: usize) -> String {
    format!("vals_elset_var{}els{}", var_index + 1, set_index + 1)
}

/// Get the reduction variable storage variable name for blocks.
pub fn block_reduction_var(entity_type: EntityType, block_index: usize) -> String {
    match entity_type {
        EntityType::ElemBlock => format!("vals_elem_red_eb{}", block_index + 1),
        EntityType::EdgeBlock => format!("vals_edge_red_edgb{}", block_index + 1),
        EntityType::FaceBlock => format!("vals_face_red_facb{}", block_index + 1),
        _ => panic!("Not a block type: {}", entity_type),
    }
}

/// Get the reduction variable storage variable name for sets.
pub fn set_reduction_var(entity_type: EntityType, set_index: usize) -> String {
    match entity_type {
        EntityType::NodeSet => format!("vals_nset_red_ns{}", set_index + 1),
        EntityType::EdgeSet => format!("vals_eset_red_es{}", set_index + 1),
        EntityType::FaceSet => format!("vals_fset_red_fs{}", set_index + 1),
        EntityType::SideSet => format!("vals_sset_red_ss{}", set_index + 1),
        EntityType::ElemSet => format!("vals_elset_red_els{}", set_index + 1),
        _ => panic!("Not a set type: {}", entity_type),
    }
}

// =============================================================================
// Set-specific Variable Names
// =============================================================================

/// Get the node set node variable name.
pub fn nodeset_nodes_var(set_index: usize) -> String {
    format!("node_ns{}", set_index + 1)
}

/// Get the node set distribution factors variable name.
pub fn nodeset_dist_factors_var(set_index: usize) -> String {
    format!("dist_fact_ns{}", set_index + 1)
}

/// Get the side set element variable name.
pub fn sideset_elem_var(set_index: usize) -> String {
    format!("elem_ss{}", set_index + 1)
}

/// Get the side set side variable name.
pub fn sideset_side_var(set_index: usize) -> String {
    format!("side_ss{}", set_index + 1)
}

/// Get the side set distribution factors variable name.
pub fn sideset_dist_factors_var(set_index: usize) -> String {
    format!("dist_fact_ss{}", set_index + 1)
}

/// Get the entity set entities variable name (for edge, face, elem sets).
pub fn entity_set_var(entity_type: EntityType, set_index: usize) -> String {
    match entity_type {
        EntityType::EdgeSet => format!("edge_es{}", set_index + 1),
        EntityType::FaceSet => format!("face_fs{}", set_index + 1),
        EntityType::ElemSet => format!("elem_els{}", set_index + 1),
        _ => panic!("Not an entity set type: {}", entity_type),
    }
}

// =============================================================================
// Truth Table Variable Names
// =============================================================================

/// Get the truth table variable name for a block type.
pub fn truth_table_var(entity_type: EntityType) -> &'static str {
    match entity_type {
        EntityType::ElemBlock => "elem_var_tab",
        EntityType::EdgeBlock => "edge_var_tab",
        EntityType::FaceBlock => "face_var_tab",
        _ => panic!(
            "Truth tables only supported for block types, got {}",
            entity_type
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_entries_dim() {
        assert_eq!(
            block_entries_dim(EntityType::ElemBlock, 0),
            "num_el_in_blk1"
        );
        assert_eq!(
            block_entries_dim(EntityType::EdgeBlock, 2),
            "num_ed_in_blk3"
        );
        assert_eq!(
            block_entries_dim(EntityType::FaceBlock, 1),
            "num_fa_in_blk2"
        );
    }

    #[test]
    fn test_set_entries_dim() {
        assert_eq!(set_entries_dim(EntityType::NodeSet, 0), "num_nod_ns1");
        assert_eq!(set_entries_dim(EntityType::SideSet, 4), "num_side_ss5");
    }

    #[test]
    fn test_prop_id_var() {
        assert_eq!(prop_id_var(EntityType::ElemBlock), "eb_prop1");
        assert_eq!(prop_id_var(EntityType::NodeSet), "ns_prop1");
    }

    #[test]
    fn test_variable_names() {
        assert_eq!(nodal_var(0), "vals_nod_var1");
        assert_eq!(elem_var(1, 0), "vals_elem_var2eb1");
        assert_eq!(nodeset_var(0, 2), "vals_nset_var1ns3");
    }

    #[test]
    fn test_set_specific_vars() {
        assert_eq!(nodeset_nodes_var(0), "node_ns1");
        assert_eq!(sideset_elem_var(1), "elem_ss2");
        assert_eq!(sideset_side_var(1), "side_ss2");
    }
}
