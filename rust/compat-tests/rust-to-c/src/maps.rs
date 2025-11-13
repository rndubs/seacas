//! ID map compatibility tests
//!
//! Tests that node and element ID maps are correctly written
//! and can be read by the C Exodus library.

use anyhow::Result;
use exodus_rs::{CreateMode, CreateOptions, ExodusFile, InitParams};
use std::path::Path;

/// Generate a file with custom node ID map
pub fn generate_node_id_map(path: &Path) -> Result<()> {
    let opts = CreateOptions {
        mode: CreateMode::Clobber,
        ..Default::default()
    };

    let mut file = ExodusFile::create(path, opts)?;

    // Initialize with 2D mesh
    let params = InitParams {
        title: "Node ID Map Test".to_string(),
        num_dim: 2,
        num_nodes: 9, // 3x3 grid
        num_elem: 4,  // 4 quad elements
        num_elem_blk: 1,
        num_node_sets: 0,
        num_side_sets: 0,
        ..Default::default()
    };

    file.init(&params)?;

    // Custom node numbering (non-sequential)
    // Map internal node index to external node ID
    let node_id_map = vec![
        100, 101, 102, // Bottom row: IDs 100-102
        200, 201, 202, // Middle row: IDs 200-202
        300, 301, 302, // Top row: IDs 300-302
    ];

    file.put_node_id_map(&node_id_map)?;

    // Write coordinates for 3x3 grid
    let x = vec![0.0, 1.0, 2.0, 0.0, 1.0, 2.0, 0.0, 1.0, 2.0];
    let y = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0];
    file.put_coords(&x, &y, &[])?;

    // Write element block
    file.put_block(
        exodus_rs::types::EntityType::ElemBlock,
        1,
        "QUAD4",
        4,
        4,
        0,
        0,
    )?;

    // Connectivity uses internal (1-based) node indices
    let connectivity = vec![
        1, 2, 5, 4, // Element 1
        2, 3, 6, 5, // Element 2
        4, 5, 8, 7, // Element 3
        5, 6, 9, 8, // Element 4
    ];
    file.put_connectivity(exodus_rs::types::EntityType::ElemBlock, 1, &connectivity)?;

    Ok(())
}

/// Generate a file with custom element ID map
pub fn generate_element_id_map(path: &Path) -> Result<()> {
    let opts = CreateOptions {
        mode: CreateMode::Clobber,
        ..Default::default()
    };

    let mut file = ExodusFile::create(path, opts)?;

    // Initialize with 2D mesh
    let params = InitParams {
        title: "Element ID Map Test".to_string(),
        num_dim: 2,
        num_nodes: 6,
        num_elem: 4,
        num_elem_blk: 1,
        num_node_sets: 0,
        num_side_sets: 0,
        ..Default::default()
    };

    file.init(&params)?;

    // Custom element numbering (reverse order)
    let element_id_map = vec![
        1000, // Element 1 -> ID 1000
        2000, // Element 2 -> ID 2000
        3000, // Element 3 -> ID 3000
        4000, // Element 4 -> ID 4000
    ];

    file.put_element_id_map(&element_id_map)?;

    // Write coordinates
    let x = vec![0.0, 1.0, 2.0, 0.0, 1.0, 2.0];
    let y = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    file.put_coords(&x, &y, &[])?;

    // Write element block with triangles
    file.put_block(
        exodus_rs::types::EntityType::ElemBlock,
        1,
        "TRI3",
        4,
        3,
        0,
        0,
    )?;

    let connectivity = vec![
        1, 2, 4, // Triangle 1 (ID 1000)
        2, 5, 4, // Triangle 2 (ID 2000)
        2, 3, 5, // Triangle 3 (ID 3000)
        3, 6, 5, // Triangle 4 (ID 4000)
    ];
    file.put_connectivity(exodus_rs::types::EntityType::ElemBlock, 1, &connectivity)?;

    Ok(())
}

/// Generate a file with both node and element ID maps
pub fn generate_both_id_maps(path: &Path) -> Result<()> {
    let opts = CreateOptions {
        mode: CreateMode::Clobber,
        ..Default::default()
    };

    let mut file = ExodusFile::create(path, opts)?;

    // Initialize with 2D mesh
    let params = InitParams {
        title: "Both ID Maps Test".to_string(),
        num_dim: 2,
        num_nodes: 8, // Hexagonal arrangement
        num_elem: 6,  // 6 triangles
        num_elem_blk: 1,
        num_node_sets: 0,
        num_side_sets: 0,
        ..Default::default()
    };

    file.init(&params)?;

    // Custom node IDs (hexagonal pattern)
    let node_id_map = vec![
        1,   // Center node
        10,  // Node at 0°
        20,  // Node at 60°
        30,  // Node at 120°
        40,  // Node at 180°
        50,  // Node at 240°
        60,  // Node at 300°
        70,  // Outer node
    ];

    file.put_node_id_map(&node_id_map)?;

    // Custom element IDs (non-sequential)
    let element_id_map = vec![
        101, 102, 103, // First three triangles
        201, 202, 203, // Last three triangles
    ];

    file.put_element_id_map(&element_id_map)?;

    // Hexagonal coordinates
    use std::f64::consts::PI;
    let mut x = vec![0.0]; // Center
    let mut y = vec![0.0];

    for i in 0..6 {
        let angle = (i as f64) * PI / 3.0;
        x.push(angle.cos());
        y.push(angle.sin());
    }
    x.push(2.0); // Outer node
    y.push(0.0);

    file.put_coords(&x, &y, &[])?;

    // Write element block
    file.put_block(
        exodus_rs::types::EntityType::ElemBlock,
        1,
        "TRI3",
        6,
        3,
        0,
        0,
    )?;

    // Connectivity for 6 triangles radiating from center
    let connectivity = vec![
        1, 2, 3, // Triangle 1 (ID 101)
        1, 3, 4, // Triangle 2 (ID 102)
        1, 4, 5, // Triangle 3 (ID 103)
        1, 5, 6, // Triangle 4 (ID 201)
        1, 6, 7, // Triangle 5 (ID 202)
        1, 7, 2, // Triangle 6 (ID 203)
    ];
    file.put_connectivity(exodus_rs::types::EntityType::ElemBlock, 1, &connectivity)?;

    Ok(())
}
