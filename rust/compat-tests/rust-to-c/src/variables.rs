//! Variable generation for compatibility testing

use anyhow::Result;
use exodus_rs::{CreateMode, CreateOptions, EntityType, ExodusFile, InitParams, Topology};
use std::path::Path;

/// Generate a mesh with global variables
pub fn generate_global_variables(path: &Path) -> Result<()> {
    let mut opts = CreateOptions::default();
    opts.mode = CreateMode::Clobber;

    let mut file = ExodusFile::create(path, opts)?;

    let params = InitParams {
        title: "Global variables for C compatibility test".to_string(),
        num_dim: 2,
        num_nodes: 4,
        num_elem: 1,
        num_elem_blk: 1,
        num_node_sets: 0,
        num_side_sets: 0,
    };

    file.put_init(&params)?;

    let x_coords = vec![0.0_f64, 1.0, 1.0, 0.0];
    let y_coords = vec![0.0_f64, 0.0, 1.0, 1.0];
    file.put_coords(&x_coords, &y_coords, &[])?;

    file.put_block(1, Topology::Quad4, 1, 0, 0)?;
    file.put_connectivity(1, &[1, 2, 3, 4])?;

    // Define 2 global variables
    file.put_variable_names(EntityType::Global, &["time_value", "max_stress"])?;

    // Write 3 time steps
    for step in 0..3 {
        let time = step as f64 * 0.1;
        file.put_time(step + 1, time)?;

        let values = vec![time, 100.0 + step as f64 * 10.0];
        file.put_global_vars(step + 1, &values)?;
    }

    file.put_qa_record(
        "exodus-rust-writer",
        "0.1.0",
        &chrono::Local::now().format("%Y-%m-%d").to_string(),
        &chrono::Local::now().format("%H:%M:%S").to_string(),
    )?;

    Ok(())
}

/// Generate a mesh with nodal variables
pub fn generate_nodal_variables(path: &Path) -> Result<()> {
    let mut opts = CreateOptions::default();
    opts.mode = CreateMode::Clobber;

    let mut file = ExodusFile::create(path, opts)?;

    let params = InitParams {
        title: "Nodal variables for C compatibility test".to_string(),
        num_dim: 2,
        num_nodes: 4,
        num_elem: 1,
        num_elem_blk: 1,
        num_node_sets: 0,
        num_side_sets: 0,
    };

    file.put_init(&params)?;

    let x_coords = vec![0.0_f64, 1.0, 1.0, 0.0];
    let y_coords = vec![0.0_f64, 0.0, 1.0, 1.0];
    file.put_coords(&x_coords, &y_coords, &[])?;

    file.put_block(1, Topology::Quad4, 1, 0, 0)?;
    file.put_connectivity(1, &[1, 2, 3, 4])?;

    // Define 2 nodal variables
    file.put_variable_names(EntityType::Node, &["temperature", "pressure"])?;

    // Write 2 time steps
    for step in 0..2 {
        let time = step as f64 * 0.1;
        file.put_time(step + 1, time)?;

        // Temperature increases with node number and time
        let temp = vec![
            100.0 + step as f64 * 10.0,
            110.0 + step as f64 * 10.0,
            120.0 + step as f64 * 10.0,
            130.0 + step as f64 * 10.0,
        ];
        file.put_nodal_var(step + 1, 1, &temp)?;

        // Pressure decreases with node number
        let pressure = vec![
            400.0 - step as f64 * 10.0,
            390.0 - step as f64 * 10.0,
            380.0 - step as f64 * 10.0,
            370.0 - step as f64 * 10.0,
        ];
        file.put_nodal_var(step + 1, 2, &pressure)?;
    }

    file.put_qa_record(
        "exodus-rust-writer",
        "0.1.0",
        &chrono::Local::now().format("%Y-%m-%d").to_string(),
        &chrono::Local::now().format("%H:%M:%S").to_string(),
    )?;

    Ok(())
}

/// Generate a mesh with element variables
pub fn generate_element_variables(path: &Path) -> Result<()> {
    let mut opts = CreateOptions::default();
    opts.mode = CreateMode::Clobber;

    let mut file = ExodusFile::create(path, opts)?;

    let params = InitParams {
        title: "Element variables for C compatibility test".to_string(),
        num_dim: 2,
        num_nodes: 6,
        num_elem: 2,
        num_elem_blk: 1,
        num_node_sets: 0,
        num_side_sets: 0,
    };

    file.put_init(&params)?;

    let x_coords = vec![0.0_f64, 1.0, 2.0, 0.0, 1.0, 2.0];
    let y_coords = vec![0.0_f64, 0.0, 0.0, 1.0, 1.0, 1.0];
    file.put_coords(&x_coords, &y_coords, &[])?;

    file.put_block(1, Topology::Quad4, 2, 0, 0)?;
    file.put_connectivity(1, &[1, 2, 5, 4, 2, 3, 6, 5])?;

    // Define element variable
    file.put_variable_names(EntityType::ElemBlock, &["stress"])?;

    // Write 2 time steps
    for step in 0..2 {
        let time = step as f64 * 0.1;
        file.put_time(step + 1, time)?;

        let stress = vec![50.0 + step as f64 * 5.0, 60.0 + step as f64 * 5.0];
        file.put_elem_var(step + 1, 1, 1, &stress)?;
    }

    file.put_qa_record(
        "exodus-rust-writer",
        "0.1.0",
        &chrono::Local::now().format("%Y-%m-%d").to_string(),
        &chrono::Local::now().format("%H:%M:%S").to_string(),
    )?;

    Ok(())
}

/// Generate a mesh with all variable types
pub fn generate_all_variables(path: &Path) -> Result<()> {
    let mut opts = CreateOptions::default();
    opts.mode = CreateMode::Clobber;

    let mut file = ExodusFile::create(path, opts)?;

    let params = InitParams {
        title: "All variable types for C compatibility test".to_string(),
        num_dim: 2,
        num_nodes: 4,
        num_elem: 1,
        num_elem_blk: 1,
        num_node_sets: 0,
        num_side_sets: 0,
    };

    file.put_init(&params)?;

    let x_coords = vec![0.0_f64, 1.0, 1.0, 0.0];
    let y_coords = vec![0.0_f64, 0.0, 1.0, 1.0];
    file.put_coords(&x_coords, &y_coords, &[])?;

    file.put_block(1, Topology::Quad4, 1, 0, 0)?;
    file.put_connectivity(1, &[1, 2, 3, 4])?;

    // Define variables
    file.put_variable_names(EntityType::Global, &["time_value"])?;
    file.put_variable_names(EntityType::Node, &["temperature"])?;
    file.put_variable_names(EntityType::ElemBlock, &["stress"])?;

    // Write 2 time steps
    for step in 0..2 {
        let time = step as f64 * 0.1;
        file.put_time(step + 1, time)?;

        // Global variable
        file.put_global_vars(step + 1, &[time])?;

        // Nodal variable
        let temp = vec![
            100.0 + step as f64 * 10.0,
            110.0 + step as f64 * 10.0,
            120.0 + step as f64 * 10.0,
            130.0 + step as f64 * 10.0,
        ];
        file.put_nodal_var(step + 1, 1, &temp)?;

        // Element variable
        let stress = vec![50.0 + step as f64 * 5.0];
        file.put_elem_var(step + 1, 1, 1, &stress)?;
    }

    file.put_qa_record(
        "exodus-rust-writer",
        "0.1.0",
        &chrono::Local::now().format("%Y-%m-%d").to_string(),
        &chrono::Local::now().format("%H:%M:%S").to_string(),
    )?;

    Ok(())
}
