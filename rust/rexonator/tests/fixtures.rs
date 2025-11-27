//! Test fixtures for rexonator integration tests
//!
//! This module provides functions to create various test exodus mesh files
//! for testing different element types, field types, and geometries.
//!
//! All fixtures create small, simple meshes suitable for quick visual inspection
//! and verification of CLI behavior.

use exodus_rs::{types::*, ExodusFile};
use std::path::PathBuf;
use tempfile::TempDir;

/// Test context that manages temporary files and provides cleanup
pub struct TestContext {
    pub temp_dir: TempDir,
}

impl TestContext {
    pub fn new() -> Self {
        Self {
            temp_dir: TempDir::new().expect("Failed to create temp directory"),
        }
    }

    pub fn path(&self, name: &str) -> PathBuf {
        self.temp_dir.path().join(name)
    }
}

/// Create a simple 2D QUAD4 mesh (4 elements in a 2x2 grid)
///
/// Geometry: Unit square [0,1] x [0,1]
/// ```text
///     y=1  7---8---9
///          |   |   |
///     y=0.5 4---5---6
///          |   |   |
///     y=0  1---2---3
///        x=0 x=0.5 x=1
/// ```
pub fn create_quad4_mesh(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let x_coords: Vec<f64> = vec![
        0.0, 0.5, 1.0, // row 1 (y=0)
        0.0, 0.5, 1.0, // row 2 (y=0.5)
        0.0, 0.5, 1.0, // row 3 (y=1)
    ];

    let y_coords: Vec<f64> = vec![
        0.0, 0.0, 0.0, // row 1
        0.5, 0.5, 0.5, // row 2
        1.0, 1.0, 1.0, // row 3
    ];

    // QUAD4 connectivity (CCW winding)
    let connectivity: Vec<i64> = vec![
        1, 2, 5, 4, // Element 1: lower-left
        2, 3, 6, 5, // Element 2: lower-right
        4, 5, 8, 7, // Element 3: upper-left
        5, 6, 9, 8, // Element 4: upper-right
    ];

    let options = CreateOptions {
        mode: CreateMode::Clobber,
        ..Default::default()
    };
    let mut file = ExodusFile::create(path, options)?;

    let params = InitParams {
        title: "QUAD4 test mesh".to_string(),
        num_dim: 2,
        num_nodes: 9,
        num_elems: 4,
        num_elem_blocks: 1,
        num_node_sets: 2,
        num_side_sets: 1,
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

    // Node set: left edge (x=0)
    let left_nodes: Vec<i64> = vec![1, 4, 7];
    file.put_node_set(1, &left_nodes, None)?;
    file.put_name(EntityType::NodeSet, 0, "left_edge")?;

    // Node set: right edge (x=1)
    let right_nodes: Vec<i64> = vec![3, 6, 9];
    file.put_node_set(2, &right_nodes, None)?;
    file.put_name(EntityType::NodeSet, 1, "right_edge")?;

    // Side set: bottom edge (y=0)
    let bottom_elems: Vec<i64> = vec![1, 2];
    let bottom_sides: Vec<i64> = vec![1, 1]; // Side 1 is typically bottom
    file.put_side_set(1, &bottom_elems, &bottom_sides, None)?;
    file.put_name(EntityType::SideSet, 0, "bottom_edge")?;

    // Add scalar and vector nodal variables
    file.define_variables(
        EntityType::Nodal,
        &["temperature", "velocity_x", "velocity_y"],
    )?;

    file.put_time(0, 0.0)?;

    // Temperature: varies with y
    let temperature: Vec<f64> = y_coords.iter().map(|&y| y * 100.0).collect();
    file.put_var(0, EntityType::Nodal, 0, 0, &temperature)?;

    // Velocity: parabolic profile
    let velocity_x: Vec<f64> = x_coords.iter().map(|&x| x * (1.0 - x) * 4.0).collect();
    file.put_var(0, EntityType::Nodal, 0, 1, &velocity_x)?;

    let velocity_y: Vec<f64> = vec![0.0; 9];
    file.put_var(0, EntityType::Nodal, 0, 2, &velocity_y)?;

    file.sync()?;
    Ok(())
}

/// Create a simple 2D TRI3 mesh (2 triangles forming a square)
///
/// Geometry: Unit square [0,1] x [0,1]
/// ```text
///     y=1  3---4
///          |\  |
///          | \ |
///          |  \|
///     y=0  1---2
///        x=0  x=1
/// ```
pub fn create_tri3_mesh(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let x_coords: Vec<f64> = vec![0.0, 1.0, 0.0, 1.0];
    let y_coords: Vec<f64> = vec![0.0, 0.0, 1.0, 1.0];

    // TRI3 connectivity (CCW winding)
    let connectivity: Vec<i64> = vec![
        1, 2, 3, // Element 1: lower triangle
        2, 4, 3, // Element 2: upper triangle
    ];

    let options = CreateOptions {
        mode: CreateMode::Clobber,
        ..Default::default()
    };
    let mut file = ExodusFile::create(path, options)?;

    let params = InitParams {
        title: "TRI3 test mesh".to_string(),
        num_dim: 2,
        num_nodes: 4,
        num_elems: 2,
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
        num_entries: 2,
        num_nodes_per_entry: 3,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    file.put_block(&block)?;
    file.put_connectivity(1, &connectivity)?;
    file.put_name(EntityType::ElemBlock, 0, "tri_block")?;

    // Node set: left edge (x=0)
    let left_nodes: Vec<i64> = vec![1, 3];
    file.put_node_set(1, &left_nodes, None)?;
    file.put_name(EntityType::NodeSet, 0, "left_edge")?;

    // Add scalar nodal variable
    file.define_variables(EntityType::Nodal, &["pressure"])?;
    file.put_time(0, 0.0)?;
    let pressure: Vec<f64> = vec![1.0, 2.0, 1.5, 2.5];
    file.put_var(0, EntityType::Nodal, 0, 0, &pressure)?;

    file.sync()?;
    Ok(())
}

/// Create a simple 3D HEX8 mesh (4 elements in a 2x2x1 grid)
///
/// This is a half-symmetry mesh with symmetry plane at x=0.
/// ```text
/// View from +Z:
///     y=1  7---8---9
///          |   |   |
///     y=0.5 4---5---6
///          |   |   |
///     y=0  1---2---3
///        x=0 x=0.5 x=1
///
/// Two layers: z=0 and z=1
/// ```
pub fn create_hex8_mesh(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // 18 nodes: 9 per z-layer
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

    // HEX8 connectivity: bottom face CCW, top face CCW
    let connectivity: Vec<i64> = vec![
        1, 2, 5, 4, 10, 11, 14, 13, // Element 1
        2, 3, 6, 5, 11, 12, 15, 14, // Element 2
        4, 5, 8, 7, 13, 14, 17, 16, // Element 3
        5, 6, 9, 8, 14, 15, 18, 17, // Element 4
    ];

    let options = CreateOptions {
        mode: CreateMode::Clobber,
        ..Default::default()
    };
    let mut file = ExodusFile::create(path, options)?;

    let params = InitParams {
        title: "HEX8 half-symmetry test mesh".to_string(),
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
    file.put_name(EntityType::ElemBlock, 0, "hex_block")?;

    // Node set: symmetry plane (x=0)
    let sym_nodes: Vec<i64> = vec![1, 4, 7, 10, 13, 16];
    file.put_node_set(1, &sym_nodes, None)?;
    file.put_name(EntityType::NodeSet, 0, "symmetry")?;

    // Node set: outlet (x=1)
    let outlet_nodes: Vec<i64> = vec![3, 6, 9, 12, 15, 18];
    file.put_node_set(2, &outlet_nodes, None)?;
    file.put_name(EntityType::NodeSet, 1, "outlet")?;

    // Side set: bottom face (z=0)
    let bottom_elems: Vec<i64> = vec![1, 2, 3, 4];
    let bottom_sides: Vec<i64> = vec![5, 5, 5, 5]; // Side 5 is -Z for HEX8
    file.put_side_set(1, &bottom_elems, &bottom_sides, None)?;
    file.put_name(EntityType::SideSet, 0, "bottom")?;

    // Add nodal variables including vector components
    file.define_variables(
        EntityType::Nodal,
        &["temperature", "velocity_x", "velocity_y", "velocity_z"],
    )?;

    file.put_time(0, 0.0)?;

    // Temperature: varies with y
    let temperature: Vec<f64> = y_coords.iter().map(|&y| y * 100.0).collect();
    file.put_var(0, EntityType::Nodal, 0, 0, &temperature)?;

    // Velocity: symmetric flow (vx=x, so 0 at symmetry plane)
    let velocity_x: Vec<f64> = x_coords.clone();
    file.put_var(0, EntityType::Nodal, 0, 1, &velocity_x)?;

    let velocity_y: Vec<f64> = vec![0.0; 18];
    file.put_var(0, EntityType::Nodal, 0, 2, &velocity_y)?;

    let velocity_z: Vec<f64> = vec![0.5; 18];
    file.put_var(0, EntityType::Nodal, 0, 3, &velocity_z)?;

    // Second time step
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

/// Create a simple 3D TET4 mesh (6 tetrahedra forming a unit cube)
///
/// The cube is decomposed into 6 tetrahedra.
pub fn create_tet4_mesh(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // 8 vertices of a unit cube
    let x_coords: Vec<f64> = vec![0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0];
    let y_coords: Vec<f64> = vec![0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0];
    let z_coords: Vec<f64> = vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0];

    // Decompose cube into 6 tetrahedra (nodes are 1-based in connectivity)
    let connectivity: Vec<i64> = vec![
        1, 2, 4, 5, // Tet 1
        2, 3, 4, 7, // Tet 2
        2, 5, 6, 7, // Tet 3
        2, 4, 5, 7, // Tet 4
        4, 5, 7, 8, // Tet 5
        5, 6, 7, 8, // Tet 6 - fixed
    ];

    let options = CreateOptions {
        mode: CreateMode::Clobber,
        ..Default::default()
    };
    let mut file = ExodusFile::create(path, options)?;

    let params = InitParams {
        title: "TET4 test mesh".to_string(),
        num_dim: 3,
        num_nodes: 8,
        num_elems: 6,
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
        num_entries: 6,
        num_nodes_per_entry: 4,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    file.put_block(&block)?;
    file.put_connectivity(1, &connectivity)?;
    file.put_name(EntityType::ElemBlock, 0, "tet_block")?;

    // Node set: x=0 face
    let x0_nodes: Vec<i64> = vec![1, 4, 5, 8];
    file.put_node_set(1, &x0_nodes, None)?;
    file.put_name(EntityType::NodeSet, 0, "x0_face")?;

    // Add scalar nodal variable
    file.define_variables(EntityType::Nodal, &["scalar_field", "u"])?;
    file.put_time(0, 0.0)?;

    let scalar: Vec<f64> = (0..8).map(|i| i as f64 * 0.5).collect();
    file.put_var(0, EntityType::Nodal, 0, 0, &scalar)?;

    // u (vector x-component)
    let u: Vec<f64> = x_coords.clone();
    file.put_var(0, EntityType::Nodal, 0, 1, &u)?;

    file.sync()?;
    Ok(())
}

/// Create a 3D WEDGE6 mesh (2 wedges forming a triangular prism)
pub fn create_wedge6_mesh(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // 6 nodes forming two triangular layers
    let x_coords: Vec<f64> = vec![
        0.0, 1.0, 0.5, // z=0 layer
        0.0, 1.0, 0.5, // z=1 layer
    ];

    let y_coords: Vec<f64> = vec![
        0.0, 0.0, 1.0, // z=0 layer (triangle)
        0.0, 0.0, 1.0, // z=1 layer
    ];

    let z_coords: Vec<f64> = vec![
        0.0, 0.0, 0.0, // z=0 layer
        1.0, 1.0, 1.0, // z=1 layer
    ];

    // Two wedges sharing the middle edge
    // Actually just 1 wedge for simplicity
    let connectivity: Vec<i64> = vec![
        1, 2, 3, 4, 5, 6, // Single wedge
    ];

    let options = CreateOptions {
        mode: CreateMode::Clobber,
        ..Default::default()
    };
    let mut file = ExodusFile::create(path, options)?;

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

    // Node set: x=0 nodes
    let x0_nodes: Vec<i64> = vec![1, 4];
    file.put_node_set(1, &x0_nodes, None)?;
    file.put_name(EntityType::NodeSet, 0, "x0_nodes")?;

    file.sync()?;
    Ok(())
}

/// Create a 3D PYRAMID5 mesh (single pyramid element)
pub fn create_pyramid5_mesh(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // 5 nodes: 4 corners of base + apex
    let x_coords: Vec<f64> = vec![0.0, 1.0, 1.0, 0.0, 0.5];
    let y_coords: Vec<f64> = vec![0.0, 0.0, 1.0, 1.0, 0.5];
    let z_coords: Vec<f64> = vec![0.0, 0.0, 0.0, 0.0, 1.0];

    let connectivity: Vec<i64> = vec![1, 2, 3, 4, 5];

    let options = CreateOptions {
        mode: CreateMode::Clobber,
        ..Default::default()
    };
    let mut file = ExodusFile::create(path, options)?;

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

    // Node set: base nodes
    let base_nodes: Vec<i64> = vec![1, 2, 3, 4];
    file.put_node_set(1, &base_nodes, None)?;
    file.put_name(EntityType::NodeSet, 0, "base")?;

    file.sync()?;
    Ok(())
}

/// Create a HEX8 mesh with element variables
pub fn create_hex8_with_elem_vars(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // Same geometry as create_hex8_mesh but with element variables
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
        1, 2, 5, 4, 10, 11, 14, 13, // Element 1
        2, 3, 6, 5, 11, 12, 15, 14, // Element 2
        4, 5, 8, 7, 13, 14, 17, 16, // Element 3
        5, 6, 9, 8, 14, 15, 18, 17, // Element 4
    ];

    let options = CreateOptions {
        mode: CreateMode::Clobber,
        ..Default::default()
    };
    let mut file = ExodusFile::create(path, options)?;

    let params = InitParams {
        title: "HEX8 with element variables".to_string(),
        num_dim: 3,
        num_nodes: 18,
        num_elems: 4,
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
        num_entries: 4,
        num_nodes_per_entry: 8,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    file.put_block(&block)?;
    file.put_connectivity(1, &connectivity)?;
    file.put_name(EntityType::ElemBlock, 0, "hex_block")?;

    // Node set
    let sym_nodes: Vec<i64> = vec![1, 4, 7, 10, 13, 16];
    file.put_node_set(1, &sym_nodes, None)?;
    file.put_name(EntityType::NodeSet, 0, "symmetry")?;

    // Nodal variables
    file.define_variables(EntityType::Nodal, &["temperature", "velocity_x"])?;

    // Element variables
    file.define_variables(EntityType::ElemBlock, &["stress_xx", "stress_xy"])?;

    // Write truth table (all elements have all variables)
    let truth_table = TruthTable::new(EntityType::ElemBlock, 1, 2);
    file.put_truth_table(EntityType::ElemBlock, &truth_table)?;

    file.put_time(0, 0.0)?;

    // Nodal data
    let temperature: Vec<f64> = y_coords.iter().map(|&y| y * 100.0).collect();
    file.put_var(0, EntityType::Nodal, 0, 0, &temperature)?;
    file.put_var(0, EntityType::Nodal, 0, 1, &x_coords)?;

    // Element data
    let stress_xx: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0];
    file.put_var(0, EntityType::ElemBlock, 1, 0, &stress_xx)?;
    let stress_xy: Vec<f64> = vec![0.5, 0.5, 0.5, 0.5];
    file.put_var(0, EntityType::ElemBlock, 1, 1, &stress_xy)?;

    file.sync()?;
    Ok(())
}

/// Create a mesh with multiple time steps for zero-time testing
pub fn create_mesh_with_time_steps(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let x_coords: Vec<f64> = vec![0.0, 1.0, 0.0, 1.0];
    let y_coords: Vec<f64> = vec![0.0, 0.0, 1.0, 1.0];

    let connectivity: Vec<i64> = vec![1, 2, 4, 3];

    let options = CreateOptions {
        mode: CreateMode::Clobber,
        ..Default::default()
    };
    let mut file = ExodusFile::create(path, options)?;

    let params = InitParams {
        title: "Multi-timestep test mesh".to_string(),
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

    file.define_variables(EntityType::Nodal, &["field"])?;

    // Write time steps starting at non-zero time
    for i in 0..5 {
        let time = 10.0 + i as f64 * 0.1; // Times: 10.0, 10.1, 10.2, 10.3, 10.4
        file.put_time(i, time)?;
        let values: Vec<f64> = vec![time; 4];
        file.put_var(i, EntityType::Nodal, 0, 0, &values)?;
    }

    file.sync()?;
    Ok(())
}

/// Create a simple mesh for basic transformation testing
pub fn create_simple_cube(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // Single HEX8 element (unit cube)
    let x_coords: Vec<f64> = vec![0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0];
    let y_coords: Vec<f64> = vec![0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0];
    let z_coords: Vec<f64> = vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0];

    let connectivity: Vec<i64> = vec![1, 2, 3, 4, 5, 6, 7, 8];

    let options = CreateOptions {
        mode: CreateMode::Clobber,
        ..Default::default()
    };
    let mut file = ExodusFile::create(path, options)?;

    let params = InitParams {
        title: "Simple unit cube".to_string(),
        num_dim: 3,
        num_nodes: 8,
        num_elems: 1,
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
        num_entries: 1,
        num_nodes_per_entry: 8,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    file.put_block(&block)?;
    file.put_connectivity(1, &connectivity)?;

    file.sync()?;
    Ok(())
}

/// Create mesh with global variables for testing
pub fn create_mesh_with_global_vars(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let x_coords: Vec<f64> = vec![0.0, 1.0, 0.0, 1.0];
    let y_coords: Vec<f64> = vec![0.0, 0.0, 1.0, 1.0];

    let connectivity: Vec<i64> = vec![1, 2, 4, 3];

    let options = CreateOptions {
        mode: CreateMode::Clobber,
        ..Default::default()
    };
    let mut file = ExodusFile::create(path, options)?;

    let params = InitParams {
        title: "Mesh with global variables".to_string(),
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

    // Define global variables
    file.define_variables(EntityType::Global, &["total_mass", "time_step"])?;

    file.put_time(0, 0.0)?;
    file.put_var(0, EntityType::Global, 0, 0, &[100.0])?;
    file.put_var(0, EntityType::Global, 0, 1, &[0.01])?;

    file.sync()?;
    Ok(())
}

/// Read coordinate bounds from a file for verification
pub fn read_coord_bounds(
    path: &PathBuf,
) -> Result<([f64; 2], [f64; 2], [f64; 2]), Box<dyn std::error::Error>> {
    let file = ExodusFile::<exodus_rs::mode::Read>::open(path)?;
    let coords = file.coords::<f64>()?;

    let x_min = coords.x.iter().cloned().fold(f64::INFINITY, f64::min);
    let x_max = coords.x.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let y_min = coords.y.iter().cloned().fold(f64::INFINITY, f64::min);
    let y_max = coords.y.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let z_min = if coords.z.is_empty() {
        0.0
    } else {
        coords.z.iter().cloned().fold(f64::INFINITY, f64::min)
    };
    let z_max = if coords.z.is_empty() {
        0.0
    } else {
        coords.z.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
    };

    Ok(([x_min, x_max], [y_min, y_max], [z_min, z_max]))
}

/// Read all coordinates from a file
pub fn read_coords(
    path: &PathBuf,
) -> Result<(Vec<f64>, Vec<f64>, Vec<f64>), Box<dyn std::error::Error>> {
    let file = ExodusFile::<exodus_rs::mode::Read>::open(path)?;
    let coords = file.coords::<f64>()?;
    Ok((coords.x, coords.y, coords.z))
}

/// Read initialization parameters from a file
pub fn read_params(path: &PathBuf) -> Result<InitParams, Box<dyn std::error::Error>> {
    let file = ExodusFile::<exodus_rs::mode::Read>::open(path)?;
    let params = file.init_params()?;
    Ok(params)
}

/// Read time steps from a file
pub fn read_times(path: &PathBuf) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
    let file = ExodusFile::<exodus_rs::mode::Read>::open(path)?;
    let times = file.times()?;
    Ok(times)
}

/// Read nodal variable data
pub fn read_nodal_var(
    path: &PathBuf,
    var_idx: usize,
    time_step: usize,
) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
    let file = ExodusFile::<exodus_rs::mode::Read>::open(path)?;
    let values = file.var(time_step, EntityType::Nodal, 0, var_idx)?;
    Ok(values)
}

/// Read nodal variable names
pub fn read_nodal_var_names(path: &PathBuf) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let file = ExodusFile::<exodus_rs::mode::Read>::open(path)?;
    let names = file.variable_names(EntityType::Nodal)?;
    Ok(names)
}

/// Read node set IDs
pub fn read_node_set_ids(path: &PathBuf) -> Result<Vec<i64>, Box<dyn std::error::Error>> {
    let file = ExodusFile::<exodus_rs::mode::Read>::open(path)?;
    let ids = file.set_ids(EntityType::NodeSet)?;
    Ok(ids)
}

/// Read entity names
pub fn read_names(
    path: &PathBuf,
    entity_type: EntityType,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let file = ExodusFile::<exodus_rs::mode::Read>::open(path)?;
    let names = file.names(entity_type)?;
    Ok(names)
}

/// Read element block IDs
pub fn read_block_ids(path: &PathBuf) -> Result<Vec<i64>, Box<dyn std::error::Error>> {
    let file = ExodusFile::<exodus_rs::mode::Read>::open(path)?;
    let ids = file.block_ids(EntityType::ElemBlock)?;
    Ok(ids)
}

/// Create a HEX8 mesh with both real vector components and false positive field names
///
/// This is used to test that CMM correctly identifies vector components and avoids
/// false positives like "max_x", "index_x", or "matrix".
///
/// Fields included:
/// - velocity_x: Real vector component (should be negated on X-axis mirror)
/// - max_x: NOT a vector component (should NOT be negated)
/// - index_x: NOT a vector component (should NOT be negated)
/// - matrix: Ends in 'x' but NOT a vector component (should NOT be negated)
/// - temperature: Scalar field (should NOT be negated)
pub fn create_hex8_with_vector_false_positives(
    path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    // Simple 2-element mesh with x from 0 to 1
    let x_coords: Vec<f64> = vec![
        0.0, 0.5, 1.0, // y=0, z=0
        0.0, 0.5, 1.0, // y=1, z=0
        0.0, 0.5, 1.0, // y=0, z=1
        0.0, 0.5, 1.0, // y=1, z=1
    ];

    let y_coords: Vec<f64> = vec![
        0.0, 0.0, 0.0, // row 1
        1.0, 1.0, 1.0, // row 2
        0.0, 0.0, 0.0, // row 3
        1.0, 1.0, 1.0, // row 4
    ];

    let z_coords: Vec<f64> = vec![
        0.0, 0.0, 0.0, // z=0
        0.0, 0.0, 0.0, // z=0
        1.0, 1.0, 1.0, // z=1
        1.0, 1.0, 1.0, // z=1
    ];

    // Two HEX8 elements
    let connectivity: Vec<i64> = vec![
        1, 2, 5, 4, 7, 8, 11, 10, // Element 1
        2, 3, 6, 5, 8, 9, 12, 11, // Element 2
    ];

    let options = CreateOptions {
        mode: CreateMode::Clobber,
        ..Default::default()
    };
    let mut file = ExodusFile::create(path, options)?;

    let params = InitParams {
        title: "HEX8 with vector false positives".to_string(),
        num_dim: 3,
        num_nodes: 12,
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
        topology: "HEX8".to_string(),
        num_entries: 2,
        num_nodes_per_entry: 8,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    file.put_block(&block)?;
    file.put_connectivity(1, &connectivity)?;
    file.put_name(EntityType::ElemBlock, 0, "hex_block")?;

    // Node set: symmetry plane (x=0)
    let sym_nodes: Vec<i64> = vec![1, 4, 7, 10];
    file.put_node_set(1, &sym_nodes, None)?;
    file.put_name(EntityType::NodeSet, 0, "symmetry")?;

    // Define nodal variables with both vector and false positive names
    file.define_variables(
        EntityType::Nodal,
        &["velocity_x", "max_x", "index_x", "matrix", "temperature"],
    )?;

    file.put_time(0, 0.0)?;

    // velocity_x: Real vector X-component (positive values that should be negated)
    // Values = x coordinate (so symmetry plane nodes have 0, others positive)
    let velocity_x: Vec<f64> = x_coords.clone();
    file.put_var(0, EntityType::Nodal, 0, 0, &velocity_x)?;

    // max_x: NOT a vector - these are all positive values representing maxima
    // Should NOT be negated
    let max_x: Vec<f64> = vec![5.0; 12]; // All 5.0
    file.put_var(0, EntityType::Nodal, 0, 1, &max_x)?;

    // index_x: NOT a vector - these are indices
    // Should NOT be negated
    let index_x: Vec<f64> = (0..12).map(|i| i as f64).collect();
    file.put_var(0, EntityType::Nodal, 0, 2, &index_x)?;

    // matrix: Ends in 'x' but NOT a vector - could be any matrix value
    // Should NOT be negated
    let matrix: Vec<f64> = vec![1.0; 12]; // All 1.0
    file.put_var(0, EntityType::Nodal, 0, 3, &matrix)?;

    // temperature: Scalar field
    // Should NOT be negated
    let temperature: Vec<f64> = y_coords.iter().map(|&y| y * 100.0).collect();
    file.put_var(0, EntityType::Nodal, 0, 4, &temperature)?;

    file.sync()?;
    Ok(())
}
