//! Entity naming compatibility tests
//!
//! Tests that entity names (blocks, sets, coordinates, variables)
//! are correctly written and can be read by the C Exodus library.

use anyhow::Result;
use exodus_rs::{CreateMode, CreateOptions, ExodusFile, InitParams};
use std::path::Path;

/// Generate a file with named element blocks
pub fn generate_block_names(path: &Path) -> Result<()> {
    let opts = CreateOptions {
        mode: CreateMode::Clobber,
        ..Default::default()
    };

    let mut file = ExodusFile::create(path, opts)?;

    // Initialize with 2D mesh with multiple blocks
    let params = InitParams {
        title: "Block Names Test".to_string(),
        num_dim: 2,
        num_nodes: 8,
        num_elem: 4,
        num_elem_blk: 2,
        num_node_sets: 0,
        num_side_sets: 0,
        ..Default::default()
    };

    file.init(&params)?;

    // Write coordinates for two separate regions
    let x = vec![
        0.0, 1.0, 1.0, 0.0, // Block 1 (quad)
        2.0, 3.0, 3.0, 2.0, // Block 2 (quad)
    ];
    let y = vec![
        0.0, 0.0, 1.0, 1.0, // Block 1
        0.0, 0.0, 1.0, 1.0, // Block 2
    ];
    file.put_coords(&x, &y, &[])?;

    // Block 1: Named "Material_Steel"
    file.put_block(
        exodus_rs::types::EntityType::ElemBlock,
        1,
        "QUAD4",
        2,
        4,
        0,
        0,
    )?;

    file.put_entity_name(exodus_rs::types::EntityType::ElemBlock, 1, "Material_Steel")?;

    let connectivity1 = vec![
        1, 2, 3, 4, // Element 1
        1, 2, 3, 4, // Element 2 (duplicate for demo)
    ];
    file.put_connectivity(exodus_rs::types::EntityType::ElemBlock, 1, &connectivity1)?;

    // Block 2: Named "Material_Aluminum"
    file.put_block(
        exodus_rs::types::EntityType::ElemBlock,
        2,
        "QUAD4",
        2,
        4,
        0,
        0,
    )?;

    file.put_entity_name(
        exodus_rs::types::EntityType::ElemBlock,
        2,
        "Material_Aluminum",
    )?;

    let connectivity2 = vec![
        5, 6, 7, 8, // Element 3
        5, 6, 7, 8, // Element 4 (duplicate for demo)
    ];
    file.put_connectivity(exodus_rs::types::EntityType::ElemBlock, 2, &connectivity2)?;

    Ok(())
}

/// Generate a file with named node and side sets
pub fn generate_set_names(path: &Path) -> Result<()> {
    let opts = CreateOptions {
        mode: CreateMode::Clobber,
        ..Default::default()
    };

    let mut file = ExodusFile::create(path, opts)?;

    // Initialize with 2D mesh
    let params = InitParams {
        title: "Set Names Test".to_string(),
        num_dim: 2,
        num_nodes: 6,
        num_elem: 2,
        num_elem_blk: 1,
        num_node_sets: 2,
        num_side_sets: 2,
        ..Default::default()
    };

    file.init(&params)?;

    // Write coordinates (two quads side by side)
    let x = vec![0.0, 1.0, 2.0, 0.0, 1.0, 2.0];
    let y = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    file.put_coords(&x, &y, &[])?;

    // Write element block
    file.put_block(
        exodus_rs::types::EntityType::ElemBlock,
        1,
        "QUAD4",
        2,
        4,
        0,
        0,
    )?;

    let connectivity = vec![
        1, 2, 5, 4, // Element 1
        2, 3, 6, 5, // Element 2
    ];
    file.put_connectivity(exodus_rs::types::EntityType::ElemBlock, 1, &connectivity)?;

    // Node Set 1: "Bottom_Boundary"
    file.put_set(exodus_rs::types::EntityType::NodeSet, 1, 3)?;
    file.put_entity_name(exodus_rs::types::EntityType::NodeSet, 1, "Bottom_Boundary")?;

    let ns1_nodes = vec![1, 2, 3]; // Bottom nodes
    file.put_set_members(exodus_rs::types::EntityType::NodeSet, 1, &ns1_nodes)?;

    // Node Set 2: "Top_Boundary"
    file.put_set(exodus_rs::types::EntityType::NodeSet, 2, 3)?;
    file.put_entity_name(exodus_rs::types::EntityType::NodeSet, 2, "Top_Boundary")?;

    let ns2_nodes = vec![4, 5, 6]; // Top nodes
    file.put_set_members(exodus_rs::types::EntityType::NodeSet, 2, &ns2_nodes)?;

    // Side Set 1: "Left_Wall"
    file.put_set(exodus_rs::types::EntityType::SideSet, 1, 1)?;
    file.put_entity_name(exodus_rs::types::EntityType::SideSet, 1, "Left_Wall")?;

    let ss1_elems = vec![1];
    let ss1_sides = vec![4]; // Left side of element 1
    file.put_side_set_members(1, &ss1_elems, &ss1_sides)?;

    // Side Set 2: "Right_Wall"
    file.put_set(exodus_rs::types::EntityType::SideSet, 2, 1)?;
    file.put_entity_name(exodus_rs::types::EntityType::SideSet, 2, "Right_Wall")?;

    let ss2_elems = vec![2];
    let ss2_sides = vec![2]; // Right side of element 2
    file.put_side_set_members(2, &ss2_elems, &ss2_sides)?;

    Ok(())
}

/// Generate a file with coordinate names
pub fn generate_coordinate_names(path: &Path) -> Result<()> {
    let opts = CreateOptions {
        mode: CreateMode::Clobber,
        ..Default::default()
    };

    let mut file = ExodusFile::create(path, opts)?;

    // Initialize with 3D mesh
    let params = InitParams {
        title: "Coordinate Names Test".to_string(),
        num_dim: 3,
        num_nodes: 8, // Single hex element
        num_elem: 1,
        num_elem_blk: 1,
        num_node_sets: 0,
        num_side_sets: 0,
        ..Default::default()
    };

    file.init(&params)?;

    // Set custom coordinate names
    let coord_names = vec!["X_Position", "Y_Position", "Z_Position"];
    file.put_coord_names(&coord_names)?;

    // Write coordinates for a unit cube
    let x = vec![0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0];
    let y = vec![0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0];
    let z = vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0];
    file.put_coords(&x, &y, &z)?;

    // Write element block
    file.put_block(
        exodus_rs::types::EntityType::ElemBlock,
        1,
        "HEX8",
        1,
        8,
        0,
        0,
    )?;

    let connectivity = vec![1, 2, 3, 4, 5, 6, 7, 8];
    file.put_connectivity(exodus_rs::types::EntityType::ElemBlock, 1, &connectivity)?;

    Ok(())
}

/// Generate a file with named variables
pub fn generate_variable_names(path: &Path) -> Result<()> {
    let opts = CreateOptions {
        mode: CreateMode::Clobber,
        ..Default::default()
    };

    let mut file = ExodusFile::create(path, opts)?;

    // Initialize with 2D mesh
    let params = InitParams {
        title: "Variable Names Test".to_string(),
        num_dim: 2,
        num_nodes: 4,
        num_elem: 1,
        num_elem_blk: 1,
        num_node_sets: 0,
        num_side_sets: 0,
        ..Default::default()
    };

    file.init(&params)?;

    // Write coordinates
    let x = vec![0.0, 1.0, 1.0, 0.0];
    let y = vec![0.0, 0.0, 1.0, 1.0];
    file.put_coords(&x, &y, &[])?;

    // Write element block
    file.put_block(
        exodus_rs::types::EntityType::ElemBlock,
        1,
        "QUAD4",
        1,
        4,
        0,
        0,
    )?;

    let connectivity = vec![1, 2, 3, 4];
    file.put_connectivity(exodus_rs::types::EntityType::ElemBlock, 1, &connectivity)?;

    // Define variables with descriptive names
    let global_var_names = vec!["Total_Energy", "Kinetic_Energy", "Potential_Energy"];
    file.define_global_variables(&global_var_names)?;

    let nodal_var_names = vec!["Temperature_Kelvin", "Pressure_Pascal"];
    file.define_nodal_variables(&nodal_var_names)?;

    let element_var_names = vec!["Von_Mises_Stress", "Max_Principal_Strain"];
    file.define_element_variables(&element_var_names)?;

    // Write a single time step with variable data
    file.put_time_value(1, 0.0)?;

    // Global variables
    let global_vals = vec![100.0, 60.0, 40.0];
    file.put_global_vars(1, &global_vals)?;

    // Nodal variables
    let temp_vals = vec![300.0, 310.0, 320.0, 330.0];
    file.put_nodal_var(1, 1, &temp_vals)?;

    let pressure_vals = vec![101325.0, 101330.0, 101335.0, 101340.0];
    file.put_nodal_var(1, 2, &pressure_vals)?;

    // Element variables
    let stress_vals = vec![50e6];
    file.put_element_var(1, 1, 1, &stress_vals)?;

    let strain_vals = vec![0.001];
    file.put_element_var(1, 2, 1, &strain_vals)?;

    Ok(())
}

/// Generate a file with all naming features combined
pub fn generate_all_names(path: &Path) -> Result<()> {
    let opts = CreateOptions {
        mode: CreateMode::Clobber,
        ..Default::default()
    };

    let mut file = ExodusFile::create(path, opts)?;

    // Initialize with 2D mesh
    let params = InitParams {
        title: "All Names Test - Complete Naming Demo".to_string(),
        num_dim: 2,
        num_nodes: 6,
        num_elem: 2,
        num_elem_blk: 2,
        num_node_sets: 1,
        num_side_sets: 1,
        ..Default::default()
    };

    file.init(&params)?;

    // Custom coordinate names
    let coord_names = vec!["X_Coordinate", "Y_Coordinate"];
    file.put_coord_names(&coord_names)?;

    // Write coordinates
    let x = vec![0.0, 1.0, 2.0, 0.0, 1.0, 2.0];
    let y = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
    file.put_coords(&x, &y, &[])?;

    // Block 1 with name
    file.put_block(
        exodus_rs::types::EntityType::ElemBlock,
        1,
        "QUAD4",
        1,
        4,
        0,
        0,
    )?;
    file.put_entity_name(exodus_rs::types::EntityType::ElemBlock, 1, "Region_A")?;

    let connectivity1 = vec![1, 2, 5, 4];
    file.put_connectivity(exodus_rs::types::EntityType::ElemBlock, 1, &connectivity1)?;

    // Block 2 with name
    file.put_block(
        exodus_rs::types::EntityType::ElemBlock,
        2,
        "QUAD4",
        1,
        4,
        0,
        0,
    )?;
    file.put_entity_name(exodus_rs::types::EntityType::ElemBlock, 2, "Region_B")?;

    let connectivity2 = vec![2, 3, 6, 5];
    file.put_connectivity(exodus_rs::types::EntityType::ElemBlock, 2, &connectivity2)?;

    // Node set with name
    file.put_set(exodus_rs::types::EntityType::NodeSet, 1, 3)?;
    file.put_entity_name(exodus_rs::types::EntityType::NodeSet, 1, "Fixed_Nodes")?;
    let ns_nodes = vec![1, 2, 3];
    file.put_set_members(exodus_rs::types::EntityType::NodeSet, 1, &ns_nodes)?;

    // Side set with name
    file.put_set(exodus_rs::types::EntityType::SideSet, 1, 2)?;
    file.put_entity_name(exodus_rs::types::EntityType::SideSet, 1, "Loaded_Surface")?;
    let ss_elems = vec![1, 2];
    let ss_sides = vec![3, 3]; // Top sides
    file.put_side_set_members(1, &ss_elems, &ss_sides)?;

    // Variables with names
    let global_var_names = vec!["Total_Force"];
    file.define_global_variables(&global_var_names)?;

    let nodal_var_names = vec!["Displacement"];
    file.define_nodal_variables(&nodal_var_names)?;

    // Write one time step
    file.put_time_value(1, 1.0)?;
    file.put_global_vars(1, &[1000.0])?;
    file.put_nodal_var(1, 1, &[0.0, 0.1, 0.2, 0.0, 0.05, 0.15])?;

    Ok(())
}
