//! Test fixtures for rexonator integration tests
//!
//! This module provides functions to create test Exodus meshes with different
//! element types, geometries, and field variables for testing rexonator operations.

use exodus_rs::{types::*, ExodusFile};
use std::path::Path;

/// Create a simple 2x2x1 HEX8 mesh with variables for testing
///
/// Creates a mesh with:
/// - 18 nodes arranged in a 3x3x2 grid
/// - 4 HEX8 elements
/// - 2 node sets (inlet, outlet)
/// - 1 side set (wall)
/// - Nodal variables: temperature, velocity_x, velocity_y, velocity_z
/// - 2 time steps
pub fn create_hex8_mesh<P: AsRef<Path>>(path: P) -> Result<(), Box<dyn std::error::Error>> {
    // Node coordinates (18 nodes for 3x3x2 grid)
    let x_coords: Vec<f64> = vec![
        // z=0 layer
        0.0, 0.5, 1.0, // y=0
        0.0, 0.5, 1.0, // y=0.5
        0.0, 0.5, 1.0, // y=1
        // z=1 layer
        0.0, 0.5, 1.0, // y=0
        0.0, 0.5, 1.0, // y=0.5
        0.0, 0.5, 1.0, // y=1
    ];

    let y_coords: Vec<f64> = vec![
        // z=0 layer
        0.0, 0.0, 0.0, 0.5, 0.5, 0.5, 1.0, 1.0, 1.0, // z=1 layer
        0.0, 0.0, 0.0, 0.5, 0.5, 0.5, 1.0, 1.0, 1.0,
    ];

    let z_coords: Vec<f64> = vec![
        // z=0 layer
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, // z=1 layer
        1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    ];

    // Element connectivity (4 HEX8 elements)
    let connectivity: Vec<i64> = vec![
        // Element 1 (lower-left)
        1, 2, 5, 4, 10, 11, 14, 13, // Element 2 (lower-right)
        2, 3, 6, 5, 11, 12, 15, 14, // Element 3 (upper-left)
        4, 5, 8, 7, 13, 14, 17, 16, // Element 4 (upper-right)
        5, 6, 9, 8, 14, 15, 18, 17,
    ];

    let options = CreateOptions {
        mode: CreateMode::Clobber,
        ..Default::default()
    };
    let mut file = ExodusFile::create(path.as_ref(), options)?;

    let params = InitParams {
        title: "HEX8 test mesh".to_string(),
        num_dim: 3,
        num_nodes: 18,
        num_elems: 4,
        num_elem_blocks: 1,
        num_node_sets: 2,
        num_side_sets: 1,
        ..Default::default()
    };
    file.init(&params)?;
    file.put_coords(&x_coords, Some(&y_coords), Some(&z_coords))?;

    let block = Block {
        id: 1,
        entity_type: EntityType::ElemBlock,
        topology: "HEX8".to_string(),
        num_entries: 4,
        num_nodes_per_entry: 8,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    file.put_block(&block)?;
    file.put_connectivity(1, &connectivity)?;
    file.put_name(EntityType::ElemBlock, 0, "block_1")?;

    // Node sets
    let inlet_nodes: Vec<i64> = vec![1, 4, 7, 10, 13, 16]; // x=0 plane
    file.put_node_set(1, &inlet_nodes, None)?;
    file.put_name(EntityType::NodeSet, 0, "inlet")?;

    let outlet_nodes: Vec<i64> = vec![3, 6, 9, 12, 15, 18]; // x=1 plane
    file.put_node_set(2, &outlet_nodes, None)?;
    file.put_name(EntityType::NodeSet, 1, "outlet")?;

    // Side set (bottom face)
    let wall_elements: Vec<i64> = vec![1, 2, 3, 4];
    let wall_sides: Vec<i64> = vec![5, 5, 5, 5];
    file.put_side_set(1, &wall_elements, &wall_sides, None)?;
    file.put_name(EntityType::SideSet, 0, "wall")?;

    // Variables
    file.define_variables(
        EntityType::Nodal,
        &["temperature", "velocity_x", "velocity_y", "velocity_z"],
    )?;

    // Time step 0
    file.put_time(0, 0.0)?;
    let temperature: Vec<f64> = y_coords.iter().map(|&y| y * 100.0).collect();
    file.put_var(0, EntityType::Nodal, 0, 0, &temperature)?;
    let velocity_x: Vec<f64> = x_coords.clone();
    file.put_var(0, EntityType::Nodal, 0, 1, &velocity_x)?;
    let velocity_y: Vec<f64> = vec![0.0; 18];
    file.put_var(0, EntityType::Nodal, 0, 2, &velocity_y)?;
    let velocity_z: Vec<f64> = vec![0.5; 18];
    file.put_var(0, EntityType::Nodal, 0, 3, &velocity_z)?;

    // Time step 1
    file.put_time(1, 1.0)?;
    let temperature_t1: Vec<f64> = y_coords.iter().map(|&y| y * 100.0 + 10.0).collect();
    file.put_var(1, EntityType::Nodal, 0, 0, &temperature_t1)?;
    let velocity_x_t1: Vec<f64> = x_coords.iter().map(|&x| x * 1.2).collect();
    file.put_var(1, EntityType::Nodal, 0, 1, &velocity_x_t1)?;
    file.put_var(1, EntityType::Nodal, 0, 2, &velocity_y)?;
    file.put_var(1, EntityType::Nodal, 0, 3, &velocity_z)?;

    file.sync()?;
    Ok(())
}

/// Create a simple 2x2 QUAD4 mesh (2D) with variables
///
/// Creates a mesh with:
/// - 9 nodes arranged in a 3x3 grid
/// - 4 QUAD4 elements
/// - Nodal variables: temperature, velocity_x, velocity_y
/// - 1 time step
pub fn create_quad4_mesh<P: AsRef<Path>>(path: P) -> Result<(), Box<dyn std::error::Error>> {
    // Node coordinates (9 nodes for 3x3 grid)
    let x_coords: Vec<f64> = vec![
        0.0, 0.5, 1.0, // y=0
        0.0, 0.5, 1.0, // y=0.5
        0.0, 0.5, 1.0, // y=1
    ];

    let y_coords: Vec<f64> = vec![
        0.0, 0.0, 0.0, // row 1
        0.5, 0.5, 0.5, // row 2
        1.0, 1.0, 1.0, // row 3
    ];

    // QUAD4 connectivity (4 elements in CCW order)
    let connectivity: Vec<i64> = vec![
        1, 2, 5, 4, // Element 1 (lower-left)
        2, 3, 6, 5, // Element 2 (lower-right)
        4, 5, 8, 7, // Element 3 (upper-left)
        5, 6, 9, 8, // Element 4 (upper-right)
    ];

    let options = CreateOptions {
        mode: CreateMode::Clobber,
        ..Default::default()
    };
    let mut file = ExodusFile::create(path.as_ref(), options)?;

    let params = InitParams {
        title: "QUAD4 test mesh".to_string(),
        num_dim: 2,
        num_nodes: 9,
        num_elems: 4,
        num_elem_blocks: 1,
        num_node_sets: 1,
        num_side_sets: 0,
        ..Default::default()
    };
    file.init(&params)?;
    file.put_coords(&x_coords, Some(&y_coords), None)?;

    let block = Block {
        id: 1,
        entity_type: EntityType::ElemBlock,
        topology: "QUAD4".to_string(),
        num_entries: 4,
        num_nodes_per_entry: 4,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    file.put_block(&block)?;
    file.put_connectivity(1, &connectivity)?;
    file.put_name(EntityType::ElemBlock, 0, "quad_block")?;

    // Node set on x=0 boundary
    let boundary_nodes: Vec<i64> = vec![1, 4, 7];
    file.put_node_set(1, &boundary_nodes, None)?;
    file.put_name(EntityType::NodeSet, 0, "left_boundary")?;

    // Variables
    file.define_variables(
        EntityType::Nodal,
        &["temperature", "velocity_x", "velocity_y"],
    )?;

    file.put_time(0, 0.0)?;
    let temperature: Vec<f64> = y_coords.iter().map(|&y| y * 100.0).collect();
    file.put_var(0, EntityType::Nodal, 0, 0, &temperature)?;
    let velocity_x: Vec<f64> = x_coords.clone();
    file.put_var(0, EntityType::Nodal, 0, 1, &velocity_x)?;
    let velocity_y: Vec<f64> = vec![0.0; 9];
    file.put_var(0, EntityType::Nodal, 0, 2, &velocity_y)?;

    file.sync()?;
    Ok(())
}

/// Create a simple TET4 mesh with 2 tetrahedra
///
/// Creates a mesh with:
/// - 5 nodes (forms 2 tetrahedra sharing a face)
/// - 2 TET4 elements
/// - Nodal variables: temperature, velocity_x, velocity_y, velocity_z
/// - 1 time step
pub fn create_tet4_mesh<P: AsRef<Path>>(path: P) -> Result<(), Box<dyn std::error::Error>> {
    // Nodes: Create a simple double-tet (bipyramid)
    // 5 nodes: base triangle at z=0 + apex above + apex below
    let x_coords: Vec<f64> = vec![
        0.0, // node 1: base corner
        1.0, // node 2: base corner
        0.5, // node 3: base corner
        0.5, // node 4: top apex
        0.5, // node 5: bottom apex (at x=0 plane for symmetry tests)
    ];

    let y_coords: Vec<f64> = vec![
        0.0,            // node 1
        0.0,            // node 2
        0.866025403784, // node 3 (sqrt(3)/2)
        0.288675134595, // node 4 (centroid y)
        0.288675134595, // node 5 (centroid y)
    ];

    let z_coords: Vec<f64> = vec![
        0.0,  // node 1
        0.0,  // node 2
        0.0,  // node 3
        0.5,  // node 4 (above)
        -0.5, // node 5 (below, on symmetry plane)
    ];

    // TET4 connectivity (right-hand rule for outward normals)
    let connectivity: Vec<i64> = vec![
        1, 2, 3, 4, // Element 1: base + top apex
        1, 3, 2, 5, // Element 2: base (reversed) + bottom apex
    ];

    let options = CreateOptions {
        mode: CreateMode::Clobber,
        ..Default::default()
    };
    let mut file = ExodusFile::create(path.as_ref(), options)?;

    let params = InitParams {
        title: "TET4 test mesh".to_string(),
        num_dim: 3,
        num_nodes: 5,
        num_elems: 2,
        num_elem_blocks: 1,
        num_node_sets: 1,
        num_side_sets: 0,
        ..Default::default()
    };
    file.init(&params)?;
    file.put_coords(&x_coords, Some(&y_coords), Some(&z_coords))?;

    let block = Block {
        id: 1,
        entity_type: EntityType::ElemBlock,
        topology: "TET4".to_string(),
        num_entries: 2,
        num_nodes_per_entry: 4,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    file.put_block(&block)?;
    file.put_connectivity(1, &connectivity)?;
    file.put_name(EntityType::ElemBlock, 0, "tet_block")?;

    // Node set on base plane (z=0)
    let base_nodes: Vec<i64> = vec![1, 2, 3];
    file.put_node_set(1, &base_nodes, None)?;
    file.put_name(EntityType::NodeSet, 0, "base_plane")?;

    // Variables
    file.define_variables(
        EntityType::Nodal,
        &["temperature", "velocity_x", "velocity_y", "velocity_z"],
    )?;

    file.put_time(0, 0.0)?;
    let temperature: Vec<f64> = vec![100.0, 100.0, 100.0, 150.0, 50.0];
    file.put_var(0, EntityType::Nodal, 0, 0, &temperature)?;
    let velocity_x = x_coords.clone();
    file.put_var(0, EntityType::Nodal, 0, 1, &velocity_x)?;
    let velocity_y: Vec<f64> = vec![0.0; 5];
    file.put_var(0, EntityType::Nodal, 0, 2, &velocity_y)?;
    let velocity_z = z_coords.clone();
    file.put_var(0, EntityType::Nodal, 0, 3, &velocity_z)?;

    file.sync()?;
    Ok(())
}

/// Create a TRI3 mesh (2D triangular elements)
///
/// Creates a mesh with:
/// - 5 nodes forming 4 triangles
/// - 4 TRI3 elements
/// - Nodal variables: temperature, velocity_x, velocity_y
/// - 1 time step
pub fn create_tri3_mesh<P: AsRef<Path>>(path: P) -> Result<(), Box<dyn std::error::Error>> {
    // Create a small triangular grid
    // 5 nodes: 4 corners + 1 center
    let x_coords: Vec<f64> = vec![
        0.0, 1.0, 1.0, 0.0, 0.5, // center
    ];

    let y_coords: Vec<f64> = vec![
        0.0, 0.0, 1.0, 1.0, 0.5, // center
    ];

    // TRI3 connectivity (CCW ordering)
    let connectivity: Vec<i64> = vec![
        1, 2, 5, // Element 1
        2, 3, 5, // Element 2
        3, 4, 5, // Element 3
        4, 1, 5, // Element 4
    ];

    let options = CreateOptions {
        mode: CreateMode::Clobber,
        ..Default::default()
    };
    let mut file = ExodusFile::create(path.as_ref(), options)?;

    let params = InitParams {
        title: "TRI3 test mesh".to_string(),
        num_dim: 2,
        num_nodes: 5,
        num_elems: 4,
        num_elem_blocks: 1,
        num_node_sets: 1,
        num_side_sets: 0,
        ..Default::default()
    };
    file.init(&params)?;
    file.put_coords(&x_coords, Some(&y_coords), None)?;

    let block = Block {
        id: 1,
        entity_type: EntityType::ElemBlock,
        topology: "TRI3".to_string(),
        num_entries: 4,
        num_nodes_per_entry: 3,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    file.put_block(&block)?;
    file.put_connectivity(1, &connectivity)?;
    file.put_name(EntityType::ElemBlock, 0, "tri_block")?;

    // Node set on x=0 boundary
    let boundary_nodes: Vec<i64> = vec![1, 4];
    file.put_node_set(1, &boundary_nodes, None)?;
    file.put_name(EntityType::NodeSet, 0, "left_boundary")?;

    // Variables
    file.define_variables(
        EntityType::Nodal,
        &["temperature", "velocity_x", "velocity_y"],
    )?;

    file.put_time(0, 0.0)?;
    let temperature: Vec<f64> = y_coords.iter().map(|&y| y * 100.0).collect();
    file.put_var(0, EntityType::Nodal, 0, 0, &temperature)?;
    let velocity_x = x_coords.clone();
    file.put_var(0, EntityType::Nodal, 0, 1, &velocity_x)?;
    let velocity_y: Vec<f64> = vec![0.0; 5];
    file.put_var(0, EntityType::Nodal, 0, 2, &velocity_y)?;

    file.sync()?;
    Ok(())
}

/// Create a WEDGE6 (triangular prism) mesh
///
/// Creates a mesh with:
/// - 6 nodes forming a single wedge
/// - 1 WEDGE6 element
/// - Nodal variables: temperature, velocity_x, velocity_y, velocity_z
pub fn create_wedge6_mesh<P: AsRef<Path>>(path: P) -> Result<(), Box<dyn std::error::Error>> {
    // Wedge nodes: bottom triangle (1-3), top triangle (4-6)
    let x_coords: Vec<f64> = vec![0.0, 1.0, 0.5, 0.0, 1.0, 0.5];
    let y_coords: Vec<f64> = vec![0.0, 0.0, 0.866025403784, 0.0, 0.0, 0.866025403784];
    let z_coords: Vec<f64> = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];

    let connectivity: Vec<i64> = vec![1, 2, 3, 4, 5, 6];

    let options = CreateOptions {
        mode: CreateMode::Clobber,
        ..Default::default()
    };
    let mut file = ExodusFile::create(path.as_ref(), options)?;

    let params = InitParams {
        title: "WEDGE6 test mesh".to_string(),
        num_dim: 3,
        num_nodes: 6,
        num_elems: 1,
        num_elem_blocks: 1,
        num_node_sets: 1,
        num_side_sets: 0,
        ..Default::default()
    };
    file.init(&params)?;
    file.put_coords(&x_coords, Some(&y_coords), Some(&z_coords))?;

    let block = Block {
        id: 1,
        entity_type: EntityType::ElemBlock,
        topology: "WEDGE6".to_string(),
        num_entries: 1,
        num_nodes_per_entry: 6,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    file.put_block(&block)?;
    file.put_connectivity(1, &connectivity)?;
    file.put_name(EntityType::ElemBlock, 0, "wedge_block")?;

    // Node set on bottom (z=0)
    let bottom_nodes: Vec<i64> = vec![1, 2, 3];
    file.put_node_set(1, &bottom_nodes, None)?;
    file.put_name(EntityType::NodeSet, 0, "bottom")?;

    // Variables
    file.define_variables(
        EntityType::Nodal,
        &["temperature", "velocity_x", "velocity_y", "velocity_z"],
    )?;

    file.put_time(0, 0.0)?;
    let temperature = z_coords.iter().map(|&z| z * 100.0).collect::<Vec<f64>>();
    file.put_var(0, EntityType::Nodal, 0, 0, &temperature)?;
    file.put_var(0, EntityType::Nodal, 0, 1, &x_coords)?;
    let velocity_y = vec![0.0; 6];
    file.put_var(0, EntityType::Nodal, 0, 2, &velocity_y)?;
    file.put_var(0, EntityType::Nodal, 0, 3, &z_coords)?;

    file.sync()?;
    Ok(())
}

/// Create a PYRAMID5 mesh
///
/// Creates a mesh with:
/// - 5 nodes forming a single pyramid
/// - 1 PYRAMID5 element
/// - Nodal variables: temperature, velocity_x, velocity_y, velocity_z
pub fn create_pyramid5_mesh<P: AsRef<Path>>(path: P) -> Result<(), Box<dyn std::error::Error>> {
    // Pyramid nodes: square base (1-4), apex (5)
    let x_coords: Vec<f64> = vec![0.0, 1.0, 1.0, 0.0, 0.5];
    let y_coords: Vec<f64> = vec![0.0, 0.0, 1.0, 1.0, 0.5];
    let z_coords: Vec<f64> = vec![0.0, 0.0, 0.0, 0.0, 1.0];

    let connectivity: Vec<i64> = vec![1, 2, 3, 4, 5];

    let options = CreateOptions {
        mode: CreateMode::Clobber,
        ..Default::default()
    };
    let mut file = ExodusFile::create(path.as_ref(), options)?;

    let params = InitParams {
        title: "PYRAMID5 test mesh".to_string(),
        num_dim: 3,
        num_nodes: 5,
        num_elems: 1,
        num_elem_blocks: 1,
        num_node_sets: 1,
        num_side_sets: 0,
        ..Default::default()
    };
    file.init(&params)?;
    file.put_coords(&x_coords, Some(&y_coords), Some(&z_coords))?;

    let block = Block {
        id: 1,
        entity_type: EntityType::ElemBlock,
        topology: "PYRAMID5".to_string(),
        num_entries: 1,
        num_nodes_per_entry: 5,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    file.put_block(&block)?;
    file.put_connectivity(1, &connectivity)?;
    file.put_name(EntityType::ElemBlock, 0, "pyramid_block")?;

    // Node set on base (z=0)
    let base_nodes: Vec<i64> = vec![1, 2, 3, 4];
    file.put_node_set(1, &base_nodes, None)?;
    file.put_name(EntityType::NodeSet, 0, "base")?;

    // Variables
    file.define_variables(
        EntityType::Nodal,
        &["temperature", "velocity_x", "velocity_y", "velocity_z"],
    )?;

    file.put_time(0, 0.0)?;
    let temperature = z_coords.iter().map(|&z| z * 100.0).collect::<Vec<f64>>();
    file.put_var(0, EntityType::Nodal, 0, 0, &temperature)?;
    file.put_var(0, EntityType::Nodal, 0, 1, &x_coords)?;
    let velocity_y = vec![0.0; 5];
    file.put_var(0, EntityType::Nodal, 0, 2, &velocity_y)?;
    file.put_var(0, EntityType::Nodal, 0, 3, &z_coords)?;

    file.sync()?;
    Ok(())
}

/// Create a HEX8 mesh with element variables for testing element variable operations
///
/// Creates a mesh with:
/// - 18 nodes arranged in a 3x3x2 grid
/// - 4 HEX8 elements
/// - Element variables: pressure, stress_x, stress_y, stress_z
/// - 2 time steps
pub fn create_hex8_with_elem_vars<P: AsRef<Path>>(
    path: P,
) -> Result<(), Box<dyn std::error::Error>> {
    // Same node coordinates as create_hex8_mesh
    let x_coords: Vec<f64> = vec![
        0.0, 0.5, 1.0, 0.0, 0.5, 1.0, 0.0, 0.5, 1.0, 0.0, 0.5, 1.0, 0.0, 0.5, 1.0, 0.0, 0.5, 1.0,
    ];

    let y_coords: Vec<f64> = vec![
        0.0, 0.0, 0.0, 0.5, 0.5, 0.5, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.5, 0.5, 0.5, 1.0, 1.0, 1.0,
    ];

    let z_coords: Vec<f64> = vec![
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    ];

    let connectivity: Vec<i64> = vec![
        1, 2, 5, 4, 10, 11, 14, 13, 2, 3, 6, 5, 11, 12, 15, 14, 4, 5, 8, 7, 13, 14, 17, 16, 5, 6,
        9, 8, 14, 15, 18, 17,
    ];

    let options = CreateOptions {
        mode: CreateMode::Clobber,
        ..Default::default()
    };
    let mut file = ExodusFile::create(path.as_ref(), options)?;

    let params = InitParams {
        title: "HEX8 with elem vars".to_string(),
        num_dim: 3,
        num_nodes: 18,
        num_elems: 4,
        num_elem_blocks: 1,
        num_node_sets: 0,
        num_side_sets: 0,
        ..Default::default()
    };
    file.init(&params)?;
    file.put_coords(&x_coords, Some(&y_coords), Some(&z_coords))?;

    let block = Block {
        id: 1,
        entity_type: EntityType::ElemBlock,
        topology: "HEX8".to_string(),
        num_entries: 4,
        num_nodes_per_entry: 8,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    file.put_block(&block)?;
    file.put_connectivity(1, &connectivity)?;
    file.put_name(EntityType::ElemBlock, 0, "block_1")?;

    // Define element variables
    file.define_variables(
        EntityType::ElemBlock,
        &["pressure", "stress_x", "stress_y", "stress_z"],
    )?;

    // Create truth table (all vars defined for all blocks)
    let truth_table = TruthTable::new(EntityType::ElemBlock, 1, 4);
    file.put_truth_table(EntityType::ElemBlock, &truth_table)?;

    // Time step 0
    file.put_time(0, 0.0)?;
    let pressure_t0 = vec![1.0, 2.0, 3.0, 4.0];
    let stress_x_t0 = vec![10.0, 20.0, 30.0, 40.0];
    let stress_y_t0 = vec![0.0; 4];
    let stress_z_t0 = vec![5.0, 5.0, 5.0, 5.0];

    file.put_var(0, EntityType::ElemBlock, 1, 0, &pressure_t0)?;
    file.put_var(0, EntityType::ElemBlock, 1, 1, &stress_x_t0)?;
    file.put_var(0, EntityType::ElemBlock, 1, 2, &stress_y_t0)?;
    file.put_var(0, EntityType::ElemBlock, 1, 3, &stress_z_t0)?;

    // Time step 1
    file.put_time(1, 1.0)?;
    let pressure_t1 = vec![1.5, 2.5, 3.5, 4.5];
    let stress_x_t1 = vec![15.0, 25.0, 35.0, 45.0];

    file.put_var(1, EntityType::ElemBlock, 1, 0, &pressure_t1)?;
    file.put_var(1, EntityType::ElemBlock, 1, 1, &stress_x_t1)?;
    file.put_var(1, EntityType::ElemBlock, 1, 2, &stress_y_t0)?;
    file.put_var(1, EntityType::ElemBlock, 1, 3, &stress_z_t0)?;

    file.sync()?;
    Ok(())
}

/// Create a mesh with only scalar variables (no vector components)
/// to test that scalar fields are NOT negated during copy-mirror-merge
pub fn create_scalar_only_mesh<P: AsRef<Path>>(path: P) -> Result<(), Box<dyn std::error::Error>> {
    let x_coords: Vec<f64> = vec![0.0, 0.5, 1.0, 0.0, 0.5, 1.0, 0.0, 0.5, 1.0];
    let y_coords: Vec<f64> = vec![0.0, 0.0, 0.0, 0.5, 0.5, 0.5, 1.0, 1.0, 1.0];

    let connectivity: Vec<i64> = vec![1, 2, 5, 4, 2, 3, 6, 5, 4, 5, 8, 7, 5, 6, 9, 8];

    let options = CreateOptions {
        mode: CreateMode::Clobber,
        ..Default::default()
    };
    let mut file = ExodusFile::create(path.as_ref(), options)?;

    let params = InitParams {
        title: "Scalar only mesh".to_string(),
        num_dim: 2,
        num_nodes: 9,
        num_elems: 4,
        num_elem_blocks: 1,
        num_node_sets: 0,
        num_side_sets: 0,
        ..Default::default()
    };
    file.init(&params)?;
    file.put_coords(&x_coords, Some(&y_coords), None)?;

    let block = Block {
        id: 1,
        entity_type: EntityType::ElemBlock,
        topology: "QUAD4".to_string(),
        num_entries: 4,
        num_nodes_per_entry: 4,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    file.put_block(&block)?;
    file.put_connectivity(1, &connectivity)?;

    // Only scalar variables (should NOT be negated)
    file.define_variables(EntityType::Nodal, &["temperature", "pressure", "density"])?;

    file.put_time(0, 0.0)?;
    let temperature: Vec<f64> = y_coords.iter().map(|&y| y * 100.0).collect();
    file.put_var(0, EntityType::Nodal, 0, 0, &temperature)?;
    let pressure: Vec<f64> = vec![1.0, 2.0, 3.0, 1.0, 2.0, 3.0, 1.0, 2.0, 3.0];
    file.put_var(0, EntityType::Nodal, 0, 1, &pressure)?;
    let density: Vec<f64> = vec![1.0; 9];
    file.put_var(0, EntityType::Nodal, 0, 2, &density)?;

    file.sync()?;
    Ok(())
}

/// Create a mesh with multiple time steps for time normalization testing
pub fn create_multi_timestep_mesh<P: AsRef<Path>>(
    path: P,
) -> Result<(), Box<dyn std::error::Error>> {
    // Simple 4-node quad
    let x_coords: Vec<f64> = vec![0.0, 1.0, 1.0, 0.0];
    let y_coords: Vec<f64> = vec![0.0, 0.0, 1.0, 1.0];

    let connectivity: Vec<i64> = vec![1, 2, 3, 4];

    let options = CreateOptions {
        mode: CreateMode::Clobber,
        ..Default::default()
    };
    let mut file = ExodusFile::create(path.as_ref(), options)?;

    let params = InitParams {
        title: "Multi timestep mesh".to_string(),
        num_dim: 2,
        num_nodes: 4,
        num_elems: 1,
        num_elem_blocks: 1,
        num_node_sets: 0,
        num_side_sets: 0,
        ..Default::default()
    };
    file.init(&params)?;
    file.put_coords(&x_coords, Some(&y_coords), None)?;

    let block = Block {
        id: 1,
        entity_type: EntityType::ElemBlock,
        topology: "QUAD4".to_string(),
        num_entries: 1,
        num_nodes_per_entry: 4,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    file.put_block(&block)?;
    file.put_connectivity(1, &connectivity)?;

    file.define_variables(EntityType::Nodal, &["temperature"])?;

    // Time steps starting at 10.0
    for i in 0..5 {
        let time = 10.0 + i as f64;
        file.put_time(i, time)?;
        let temp: Vec<f64> = vec![100.0 + i as f64 * 10.0; 4];
        file.put_var(i, EntityType::Nodal, 0, 0, &temp)?;
    }

    file.sync()?;
    Ok(())
}

/// Create a half-symmetry HEX8 mesh positioned for copy-mirror-merge testing
/// The mesh is positioned with x >= 0 for X-axis symmetry tests
pub fn create_half_symmetry_hex8<P: AsRef<Path>>(
    path: P,
) -> Result<(), Box<dyn std::error::Error>> {
    // Single HEX8 element with one face on the x=0 symmetry plane
    let x_coords: Vec<f64> = vec![
        0.0, 1.0, 1.0, 0.0, // bottom face
        0.0, 1.0, 1.0, 0.0, // top face
    ];
    let y_coords: Vec<f64> = vec![
        0.0, 0.0, 1.0, 1.0, // bottom
        0.0, 0.0, 1.0, 1.0, // top
    ];
    let z_coords: Vec<f64> = vec![
        0.0, 0.0, 0.0, 0.0, // bottom
        1.0, 1.0, 1.0, 1.0, // top
    ];

    let connectivity: Vec<i64> = vec![1, 2, 3, 4, 5, 6, 7, 8];

    let options = CreateOptions {
        mode: CreateMode::Clobber,
        ..Default::default()
    };
    let mut file = ExodusFile::create(path.as_ref(), options)?;

    let params = InitParams {
        title: "Half symmetry HEX8".to_string(),
        num_dim: 3,
        num_nodes: 8,
        num_elems: 1,
        num_elem_blocks: 1,
        num_node_sets: 1,
        num_side_sets: 0,
        ..Default::default()
    };
    file.init(&params)?;
    file.put_coords(&x_coords, Some(&y_coords), Some(&z_coords))?;

    let block = Block {
        id: 1,
        entity_type: EntityType::ElemBlock,
        topology: "HEX8".to_string(),
        num_entries: 1,
        num_nodes_per_entry: 8,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    file.put_block(&block)?;
    file.put_connectivity(1, &connectivity)?;
    file.put_name(EntityType::ElemBlock, 0, "block_1")?;

    // Node set on symmetry plane (x=0)
    let sym_nodes: Vec<i64> = vec![1, 4, 5, 8];
    file.put_node_set(1, &sym_nodes, None)?;
    file.put_name(EntityType::NodeSet, 0, "symmetry")?;

    // Variables with x-component that should be negated
    file.define_variables(
        EntityType::Nodal,
        &["temperature", "velocity_x", "velocity_y", "velocity_z"],
    )?;

    file.put_time(0, 0.0)?;
    // Temperature varies with x position
    let temperature: Vec<f64> = x_coords.iter().map(|&x| x * 100.0).collect();
    file.put_var(0, EntityType::Nodal, 0, 0, &temperature)?;
    // Velocity_x = x (0 on symmetry plane, positive away from it)
    file.put_var(0, EntityType::Nodal, 0, 1, &x_coords)?;
    let velocity_y: Vec<f64> = vec![0.0; 8];
    file.put_var(0, EntityType::Nodal, 0, 2, &velocity_y)?;
    let velocity_z: Vec<f64> = vec![0.5; 8];
    file.put_var(0, EntityType::Nodal, 0, 3, &velocity_z)?;

    file.sync()?;
    Ok(())
}

/// Helper function to read coordinates from a file
pub fn read_coords<P: AsRef<Path>>(
    path: P,
) -> Result<(Vec<f64>, Vec<f64>, Vec<f64>), Box<dyn std::error::Error>> {
    use exodus_rs::mode;
    let file = ExodusFile::<mode::Read>::open(path.as_ref())?;
    let coords = file.coords()?;
    let z = if coords.z.is_empty() {
        vec![0.0; coords.x.len()]
    } else {
        coords.z
    };
    Ok((coords.x, coords.y, z))
}

/// Helper function to read nodal variable values
pub fn read_nodal_var<P: AsRef<Path>>(
    path: P,
    var_name: &str,
    time_step: usize,
) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
    use exodus_rs::mode;
    let file = ExodusFile::<mode::Read>::open(path.as_ref())?;
    let var_names = file.variable_names(EntityType::Nodal)?;
    let var_idx = var_names
        .iter()
        .position(|n| n == var_name)
        .ok_or_else(|| format!("Variable '{}' not found", var_name))?;
    let values = file.var(time_step, EntityType::Nodal, 0, var_idx)?;
    Ok(values)
}

/// Helper function to read mesh parameters
pub fn read_params<P: AsRef<Path>>(path: P) -> Result<InitParams, Box<dyn std::error::Error>> {
    use exodus_rs::mode;
    let file = ExodusFile::<mode::Read>::open(path.as_ref())?;
    let params = file.init_params()?;
    Ok(params)
}

/// Helper function to read time steps
pub fn read_times<P: AsRef<Path>>(path: P) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
    use exodus_rs::mode;
    let file = ExodusFile::<mode::Read>::open(path.as_ref())?;
    let times = file.times()?;
    Ok(times)
}

/// Helper function to read block names
pub fn read_block_names<P: AsRef<Path>>(
    path: P,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    use exodus_rs::mode;
    let file = ExodusFile::<mode::Read>::open(path.as_ref())?;
    let names = file.names(EntityType::ElemBlock).unwrap_or_default();
    Ok(names)
}

/// Helper function to read node set names
pub fn read_node_set_names<P: AsRef<Path>>(
    path: P,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    use exodus_rs::mode;
    let file = ExodusFile::<mode::Read>::open(path.as_ref())?;
    let names = file.names(EntityType::NodeSet).unwrap_or_default();
    Ok(names)
}
