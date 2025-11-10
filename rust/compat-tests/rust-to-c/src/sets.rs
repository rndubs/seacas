//! Set generation for compatibility testing

use anyhow::Result;
use exodus_rs::{Block, CreateMode, CreateOptions, EntityType, ExodusFile, InitParams, Set, Topology};
use std::path::Path;

/// Generate a mesh with node sets
pub fn generate_node_sets(path: &Path) -> Result<()> {
    let mut opts = CreateOptions::default();
    opts.mode = CreateMode::Clobber;

    let mut file = ExodusFile::create(path, opts)?;

    let params = InitParams {
        title: "Node sets for C compatibility test".to_string(),
        num_dim: 2,
        num_nodes: 9,
        num_elems: 4,
        num_elem_blocks: 1,
        num_node_sets: 2,
        ..Default::default()
    };

    file.init(&params)?;

    // 3x3 grid of nodes
    let x_coords = vec![0.0_f64, 1.0, 2.0, 0.0, 1.0, 2.0, 0.0, 1.0, 2.0];
    let y_coords = vec![0.0_f64, 0.0, 0.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0];
    let z_coords: Vec<f64> = vec![];

    file.put_coords(&x_coords, Some(&y_coords), None)?;

    // Element block with 4 quads
    let block = Block {
        id: 1,
        entity_type: EntityType::ElemBlock,
        topology: Topology::Quad4.to_string(),
        num_entries: 4,
        num_nodes_per_entry: 4,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    file.put_block(&block)?;
    let connectivity = vec![
        1_i64, 2, 5, 4, // Quad 1
        2, 3, 6, 5, // Quad 2
        4, 5, 8, 7, // Quad 3
        5, 6, 9, 8, // Quad 4
    ];
    file.put_connectivity(1, &connectivity)?;

    // Define node set 1
    file.put_set(&Set {
        id: 100,
        entity_type: EntityType::NodeSet,
        num_entries: 3,
        num_dist_factors: 0,
    })?;
    file.put_node_set(100, &[1_i64, 2, 3], None)?;

    // Define node set 2
    file.put_set(&Set {
        id: 200,
        entity_type: EntityType::NodeSet,
        num_entries: 3,
        num_dist_factors: 0,
    })?;
    file.put_node_set(200, &[3_i64, 6, 9], None)?;

    Ok(())
}

/// Generate a mesh with side sets
pub fn generate_side_sets(path: &Path) -> Result<()> {
    let mut opts = CreateOptions::default();
    opts.mode = CreateMode::Clobber;

    let mut file = ExodusFile::create(path, opts)?;

    let params = InitParams {
        title: "Side sets for C compatibility test".to_string(),
        num_dim: 2,
        num_nodes: 4,
        num_elems: 1,
        num_elem_blocks: 1,
        num_side_sets: 1,
        ..Default::default()
    };

    file.init(&params)?;

    let x_coords = vec![0.0_f64, 1.0, 1.0, 0.0];
    let y_coords = vec![0.0_f64, 0.0, 1.0, 1.0];
    let z_coords: Vec<f64> = vec![];
    file.put_coords(&x_coords, Some(&y_coords), None)?;

    let block = Block {
        id: 1,
        entity_type: EntityType::ElemBlock,
        topology: Topology::Quad4.to_string(),
        num_entries: 1,
        num_nodes_per_entry: 4,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    file.put_block(&block)?;
    let connectivity = vec![1_i64, 2, 3, 4];
    file.put_connectivity(1, &connectivity)?;

    // Define side set
    file.put_set(&Set {
        id: 300,
        entity_type: EntityType::SideSet,
        num_entries: 1,
        num_dist_factors: 0,
    })?;
    file.put_side_set(300, &[1_i64], &[1_i64], None)?;

    Ok(())
}

/// Generate a mesh with element sets
pub fn generate_element_sets(path: &Path) -> Result<()> {
    let mut opts = CreateOptions::default();
    opts.mode = CreateMode::Clobber;

    let mut file = ExodusFile::create(path, opts)?;

    let params = InitParams {
        title: "Element sets for C compatibility test".to_string(),
        num_dim: 2,
        num_nodes: 9,
        num_elems: 4,
        num_elem_blocks: 1,
        num_elem_sets: 1,
        ..Default::default()
    };

    file.init(&params)?;

    let x_coords = vec![0.0_f64, 1.0, 2.0, 0.0, 1.0, 2.0, 0.0, 1.0, 2.0];
    let y_coords = vec![0.0_f64, 0.0, 0.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0];
    let z_coords: Vec<f64> = vec![];
    file.put_coords(&x_coords, Some(&y_coords), None)?;

    let block = Block {
        id: 1,
        entity_type: EntityType::ElemBlock,
        topology: Topology::Quad4.to_string(),
        num_entries: 4,
        num_nodes_per_entry: 4,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    file.put_block(&block)?;
    let connectivity = vec![
        1_i64, 2, 5, 4, // Elem 1
        2, 3, 6, 5, // Elem 2
        4, 5, 8, 7, // Elem 3
        5, 6, 9, 8, // Elem 4
    ];
    file.put_connectivity(1, &connectivity)?;

    // Define element set
    file.put_set(&Set {
        id: 400,
        entity_type: EntityType::ElemSet,
        num_entries: 2,
        num_dist_factors: 0,
    })?;
    file.put_entity_set(EntityType::ElemSet, 400, &[1_i64, 2])?;

    Ok(())
}

/// Generate a mesh with all set types
pub fn generate_all_sets(path: &Path) -> Result<()> {
    let mut opts = CreateOptions::default();
    opts.mode = CreateMode::Clobber;

    let mut file = ExodusFile::create(path, opts)?;

    let params = InitParams {
        title: "All set types for C compatibility test".to_string(),
        num_dim: 2,
        num_nodes: 9,
        num_elems: 4,
        num_elem_blocks: 1,
        num_node_sets: 1,
        num_side_sets: 1,
        num_elem_sets: 1,
        ..Default::default()
    };

    file.init(&params)?;

    let x_coords = vec![0.0_f64, 1.0, 2.0, 0.0, 1.0, 2.0, 0.0, 1.0, 2.0];
    let y_coords = vec![0.0_f64, 0.0, 0.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0];
    let z_coords: Vec<f64> = vec![];
    file.put_coords(&x_coords, Some(&y_coords), None)?;

    let block = Block {
        id: 1,
        entity_type: EntityType::ElemBlock,
        topology: Topology::Quad4.to_string(),
        num_entries: 4,
        num_nodes_per_entry: 4,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    file.put_block(&block)?;
    let connectivity = vec![
        1_i64, 2, 5, 4, 2, 3, 6, 5, 4, 5, 8, 7, 5, 6, 9, 8,
    ];
    file.put_connectivity(1, &connectivity)?;

    // Node set
    file.put_set(&Set {
        id: 100,
        entity_type: EntityType::NodeSet,
        num_entries: 3,
        num_dist_factors: 0,
    })?;
    file.put_node_set(100, &[1_i64, 2, 3], None)?;

    // Side set
    file.put_set(&Set {
        id: 200,
        entity_type: EntityType::SideSet,
        num_entries: 1,
        num_dist_factors: 0,
    })?;
    file.put_side_set(200, &[1_i64], &[1_i64], None)?;

    // Element set
    file.put_set(&Set {
        id: 300,
        entity_type: EntityType::ElemSet,
        num_entries: 2,
        num_dist_factors: 0,
    })?;
    file.put_entity_set(EntityType::ElemSet, 300, &[1_i64, 2])?;

    Ok(())
}
