//! Script to create a test exodus mesh for copy-mirror-merge testing
//!
//! Run with: cargo run --example create_test_mesh

use exodus_rs::{types::*, ExodusFile};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a simple half-symmetry mesh: 2x2x1 HEX8 elements
    // The symmetry plane is at x=0
    //
    // View from +Z looking down:
    //
    //     y=1  5---6---7---8
    //          |   |   |   |
    //     y=0  1---2---3---4
    //        x=0  x=0.5 x=1
    //
    // This creates 4 elements in a 2x2 grid (in XY) with depth 1 (in Z)
    // Nodes 1, 5, 9, 13 are on the symmetry plane (x=0)

    let output_path = "test_half_symmetry.exo";

    // Node coordinates (18 nodes for 2x2x2 grid = 3x3x2)
    // Layer z=0:
    //   1(0,0,0)  2(0.5,0,0)  3(1,0,0)
    //   4(0,0.5,0)  5(0.5,0.5,0)  6(1,0.5,0)
    //   7(0,1,0)  8(0.5,1,0)  9(1,1,0)
    // Layer z=1:
    //   10(0,0,1)  11(0.5,0,1)  12(1,0,1)
    //   13(0,0.5,1)  14(0.5,0.5,1)  15(1,0.5,1)
    //   16(0,1,1)  17(0.5,1,1)  18(1,1,1)

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
        0.0, 0.0, 0.0, // row 1
        0.5, 0.5, 0.5, // row 2
        1.0, 1.0, 1.0, // row 3
        // z=1 layer
        0.0, 0.0, 0.0, // row 1
        0.5, 0.5, 0.5, // row 2
        1.0, 1.0, 1.0, // row 3
    ];

    let z_coords: Vec<f64> = vec![
        // z=0 layer
        0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, // z=1 layer
        1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
    ];

    // Element connectivity (4 HEX8 elements)
    // HEX8 node ordering: bottom face (0-3), top face (4-7), CCW when viewed from outside
    //
    // Element 1: lower-left (x=0..0.5, y=0..0.5)
    //   bottom: 1,2,5,4  top: 10,11,14,13
    // Element 2: lower-right (x=0.5..1, y=0..0.5)
    //   bottom: 2,3,6,5  top: 11,12,15,14
    // Element 3: upper-left (x=0..0.5, y=0.5..1)
    //   bottom: 4,5,8,7  top: 13,14,17,16
    // Element 4: upper-right (x=0.5..1, y=0.5..1)
    //   bottom: 5,6,9,8  top: 14,15,18,17

    let connectivity: Vec<i64> = vec![
        // Element 1
        1, 2, 5, 4, 10, 11, 14, 13, // Element 2
        2, 3, 6, 5, 11, 12, 15, 14, // Element 3
        4, 5, 8, 7, 13, 14, 17, 16, // Element 4
        5, 6, 9, 8, 14, 15, 18, 17,
    ];

    // Create the file
    let options = CreateOptions {
        mode: CreateMode::Clobber,
        ..Default::default()
    };
    let mut file = ExodusFile::create(output_path, options)?;

    // Initialize with params
    let params = InitParams {
        title: "Half-symmetry test mesh".to_string(),
        num_dim: 3,
        num_nodes: 18,
        num_elems: 4,
        num_elem_blocks: 1,
        num_node_sets: 2,
        num_side_sets: 1,
        ..Default::default()
    };
    file.init(&params)?;

    // Write coordinates
    file.put_coords(&x_coords, Some(&y_coords), Some(&z_coords))?;

    // Define and write element block
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

    // Node set 1: "inlet" - nodes on x=0 plane (symmetry plane)
    let inlet_nodes: Vec<i64> = vec![1, 4, 7, 10, 13, 16];
    file.put_node_set(1, &inlet_nodes, None)?;
    file.put_name(EntityType::NodeSet, 0, "inlet")?;

    // Node set 2: "outlet" - nodes on x=1 plane
    let outlet_nodes: Vec<i64> = vec![3, 6, 9, 12, 15, 18];
    file.put_node_set(2, &outlet_nodes, None)?;
    file.put_name(EntityType::NodeSet, 1, "outlet")?;

    // Side set 1: "wall" - bottom face (z=0)
    // Elements 1-4, side 1 (bottom face for HEX8 is typically side 1 in Exodus)
    let wall_elements: Vec<i64> = vec![1, 2, 3, 4];
    let wall_sides: Vec<i64> = vec![5, 5, 5, 5]; // Side 5 is -Z face for HEX8
    file.put_side_set(1, &wall_elements, &wall_sides, None)?;
    file.put_name(EntityType::SideSet, 0, "wall")?;

    // Define nodal variables
    file.define_variables(
        EntityType::Nodal,
        &["temperature", "velocity_x", "velocity_y", "velocity_z"],
    )?;

    // Write time step and variable data
    file.put_time(0, 0.0)?;

    // Temperature: varies with y (0 at y=0, 100 at y=1)
    let temperature: Vec<f64> = y_coords.iter().map(|&y| y * 100.0).collect();
    file.put_var(0, EntityType::Nodal, 0, 0, &temperature)?;

    // Velocity: flow in +x direction, symmetric about x=0
    // velocity_x = x (so it's 0 at symmetry plane)
    let velocity_x: Vec<f64> = x_coords.clone();
    file.put_var(0, EntityType::Nodal, 0, 1, &velocity_x)?;

    // velocity_y = 0 (no crossflow)
    let velocity_y: Vec<f64> = vec![0.0; 18];
    file.put_var(0, EntityType::Nodal, 0, 2, &velocity_y)?;

    // velocity_z = 0.5 (constant upward flow)
    let velocity_z: Vec<f64> = vec![0.5; 18];
    file.put_var(0, EntityType::Nodal, 0, 3, &velocity_z)?;

    // Add a second time step with slightly different values
    file.put_time(1, 1.0)?;

    // Temperature at t=1: slightly higher
    let temperature_t1: Vec<f64> = y_coords.iter().map(|&y| y * 100.0 + 10.0).collect();
    file.put_var(1, EntityType::Nodal, 0, 0, &temperature_t1)?;

    // Velocity at t=1: slightly faster
    let velocity_x_t1: Vec<f64> = x_coords.iter().map(|&x| x * 1.2).collect();
    file.put_var(1, EntityType::Nodal, 0, 1, &velocity_x_t1)?;
    file.put_var(1, EntityType::Nodal, 0, 2, &velocity_y)?;
    file.put_var(1, EntityType::Nodal, 0, 3, &velocity_z)?;

    file.sync()?;

    println!("Created test mesh: {}", output_path);
    println!("  Nodes: 18");
    println!("  Elements: 4 HEX8");
    println!("  Node sets: inlet (x=0), outlet (x=1)");
    println!("  Side sets: wall (z=0)");
    println!("  Nodal variables: temperature, velocity_x, velocity_y, velocity_z");
    println!("  Time steps: 2");

    Ok(())
}
