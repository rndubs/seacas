//! Set generation for compatibility testing

use anyhow::Result;
use exodus_rs::{CreateMode, CreateOptions, ExodusFile, InitParams, Topology};
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
        num_elem: 4,
        num_elem_blk: 1,
        num_node_sets: 2,
        num_side_sets: 0,
    };

    file.put_init(&params)?;

    // 3x3 grid of nodes
    let x_coords = vec![0.0_f64, 1.0, 2.0, 0.0, 1.0, 2.0, 0.0, 1.0, 2.0];
    let y_coords = vec![0.0_f64, 0.0, 0.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0];

    file.put_coords(&x_coords, &y_coords, &[])?;
    file.put_coord_names(&["x", "y"])?;

    // Element block with 4 quads
    file.put_block(1, Topology::Quad4, 4, 0, 0)?;
    let connectivity = vec![
        1, 2, 5, 4, // Quad 1
        2, 3, 6, 5, // Quad 2
        4, 5, 8, 7, // Quad 3
        5, 6, 9, 8, // Quad 4
    ];
    file.put_connectivity(1, &connectivity)?;

    // Node set 1: Bottom edge (nodes 1, 2, 3)
    file.put_node_set(100, &[1, 2, 3], None)?;

    // Node set 2: Right edge (nodes 3, 6, 9)
    file.put_node_set(200, &[3, 6, 9], None)?;

    file.put_qa_record(
        "exodus-rust-writer",
        "0.1.0",
        &chrono::Local::now().format("%Y-%m-%d").to_string(),
        &chrono::Local::now().format("%H:%M:%S").to_string(),
    )?;

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
        num_elem: 1,
        num_elem_blk: 1,
        num_node_sets: 0,
        num_side_sets: 1,
    };

    file.put_init(&params)?;

    let x_coords = vec![0.0_f64, 1.0, 1.0, 0.0];
    let y_coords = vec![0.0_f64, 0.0, 1.0, 1.0];
    file.put_coords(&x_coords, &y_coords, &[])?;
    file.put_coord_names(&["x", "y"])?;

    file.put_block(1, Topology::Quad4, 1, 0, 0)?;
    let connectivity = vec![1, 2, 3, 4];
    file.put_connectivity(1, &connectivity)?;

    // Side set: bottom edge (element 1, side 1)
    file.put_side_set(300, &[1], &[1], None)?;

    file.put_qa_record(
        "exodus-rust-writer",
        "0.1.0",
        &chrono::Local::now().format("%Y-%m-%d").to_string(),
        &chrono::Local::now().format("%H:%M:%S").to_string(),
    )?;

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
        num_elem: 4,
        num_elem_blk: 1,
        num_node_sets: 0,
        num_side_sets: 0,
    };

    file.put_init(&params)?;

    let x_coords = vec![0.0_f64, 1.0, 2.0, 0.0, 1.0, 2.0, 0.0, 1.0, 2.0];
    let y_coords = vec![0.0_f64, 0.0, 0.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0];
    file.put_coords(&x_coords, &y_coords, &[])?;
    file.put_coord_names(&["x", "y"])?;

    file.put_block(1, Topology::Quad4, 4, 0, 0)?;
    let connectivity = vec![
        1, 2, 5, 4, // Elem 1
        2, 3, 6, 5, // Elem 2
        4, 5, 8, 7, // Elem 3
        5, 6, 9, 8, // Elem 4
    ];
    file.put_connectivity(1, &connectivity)?;

    // Element set: bottom row (elements 1, 2)
    file.put_elem_set(400, &[1, 2], None)?;

    file.put_qa_record(
        "exodus-rust-writer",
        "0.1.0",
        &chrono::Local::now().format("%Y-%m-%d").to_string(),
        &chrono::Local::now().format("%H:%M:%S").to_string(),
    )?;

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
        num_elem: 4,
        num_elem_blk: 1,
        num_node_sets: 1,
        num_side_sets: 1,
    };

    file.put_init(&params)?;

    let x_coords = vec![0.0_f64, 1.0, 2.0, 0.0, 1.0, 2.0, 0.0, 1.0, 2.0];
    let y_coords = vec![0.0_f64, 0.0, 0.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0];
    file.put_coords(&x_coords, &y_coords, &[])?;
    file.put_coord_names(&["x", "y"])?;

    file.put_block(1, Topology::Quad4, 4, 0, 0)?;
    let connectivity = vec![
        1, 2, 5, 4, 2, 3, 6, 5, 4, 5, 8, 7, 5, 6, 9, 8,
    ];
    file.put_connectivity(1, &connectivity)?;

    // Node set
    file.put_node_set(100, &[1, 2, 3], None)?;

    // Side set
    file.put_side_set(200, &[1], &[1], None)?;

    // Element set
    file.put_elem_set(300, &[1, 2], None)?;

    file.put_qa_record(
        "exodus-rust-writer",
        "0.1.0",
        &chrono::Local::now().format("%Y-%m-%d").to_string(),
        &chrono::Local::now().format("%H:%M:%S").to_string(),
    )?;

    Ok(())
}
