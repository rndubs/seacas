//! Element block generation for compatibility testing

use anyhow::Result;
use exodus_rs::{CreateMode, CreateOptions, ExodusFile, InitParams, Topology};
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
        num_elem: 3,
        num_elem_blk: 2, // 1 quad block, 1 tri block
        num_node_sets: 0,
        num_side_sets: 0,
    };

    file.put_init(&params)?;

    // Coordinates for two quads and one triangle
    // Quad 1: nodes 0,1,2,3
    // Quad 2: nodes 1,4,5,2
    // Tri 1:  nodes 2,5,6
    let x_coords = vec![0.0_f64, 1.0, 1.0, 0.0, 2.0, 2.0, 1.5];
    let y_coords = vec![0.0_f64, 0.0, 1.0, 1.0, 0.0, 1.0, 1.5];

    file.put_coords(&x_coords, &y_coords, &[])?;
    file.put_coord_names(&["x", "y"])?;

    // Block 1: Quads
    file.put_block(10, Topology::Quad4, 2, 0, 0)?;
    let quad_conn = vec![
        1, 2, 3, 4, // Quad 1
        2, 5, 6, 3, // Quad 2
    ];
    file.put_connectivity(10, &quad_conn)?;

    // Block 2: Triangle
    file.put_block(20, Topology::Tri3, 1, 0, 0)?;
    let tri_conn = vec![3, 6, 7]; // Triangle
    file.put_connectivity(20, &tri_conn)?;

    file.put_qa_record(
        "exodus-rust-writer",
        "0.1.0",
        &chrono::Local::now().format("%Y-%m-%d").to_string(),
        &chrono::Local::now().format("%H:%M:%S").to_string(),
    )?;

    Ok(())
}
