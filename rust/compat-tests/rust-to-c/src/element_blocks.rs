//! Element block generation for compatibility testing

use anyhow::Result;
use exodus_rs::{Block, CreateMode, CreateOptions, EntityType, ExodusFile, InitParams, Topology};
use std::path::Path;

/// Generate a mesh with multiple element blocks of different types
pub fn generate_multiple_blocks(path: &Path) -> Result<()> {
    let mut opts = CreateOptions::default();
    opts.mode = CreateMode::Clobber;

    let mut file = ExodusFile::create(path, opts)?;

    // Mesh with 2 quads and 1 triangle
    let params = InitParams {
        title: "Multi-block mesh for C compatibility test".to_string(),
        num_dim: 2,
        num_nodes: 7,
        num_elems: 3,
        num_elem_blocks: 2, // 1 quad block, 1 tri block
        ..Default::default()
    };

    file.init(&params)?;

    // Coordinates for two quads and one triangle
    // Quad 1: nodes 0,1,2,3
    // Quad 2: nodes 1,4,5,2
    // Tri 1:  nodes 2,5,6
    let x_coords = vec![0.0_f64, 1.0, 1.0, 0.0, 2.0, 2.0, 1.5];
    let y_coords = vec![0.0_f64, 0.0, 1.0, 1.0, 0.0, 1.0, 1.5];
    let z_coords: Vec<f64> = vec![];

    file.put_coords(&x_coords, Some(&y_coords), None)?;

    // Block 1: Quads
    let block1 = Block {
        id: 10,
        entity_type: EntityType::ElemBlock,
        topology: Topology::Quad4.to_string(),
        num_entries: 2,
        num_nodes_per_entry: 4,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    file.put_block(&block1)?;
    let quad_conn = vec![
        1_i64, 2, 3, 4, // Quad 1
        2, 5, 6, 3, // Quad 2
    ];
    file.put_connectivity(10, &quad_conn)?;

    // Block 2: Triangle
    let block2 = Block {
        id: 20,
        entity_type: EntityType::ElemBlock,
        topology: Topology::Tri3.to_string(),
        num_entries: 1,
        num_nodes_per_entry: 3,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    file.put_block(&block2)?;
    let tri_conn = vec![3_i64, 6, 7]; // Triangle
    file.put_connectivity(20, &tri_conn)?;

    Ok(())
}
