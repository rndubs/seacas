//! Variable generation for compatibility testing
//!
//! Tests actual variable data writing and time steps

use anyhow::Result;
use exodus_rs::{Block, CreateMode, CreateOptions, EntityType, ExodusFile, InitParams, Topology};
use std::path::Path;

/// Generate mesh with global variables
pub fn generate_global_variables(path: &Path) -> Result<()> {
    let opts = CreateOptions {
        mode: CreateMode::Clobber,
        ..Default::default()
    };

    let mut file = ExodusFile::create(path, opts)?;

    let params = InitParams {
        title: "Global variables test for C compatibility".to_string(),
        num_dim: 2,
        num_nodes: 4,
        num_elems: 1,
        num_elem_blocks: 1,
        ..Default::default()
    };

    file.init(&params)?;

    // Basic mesh
    let x_coords = vec![0.0_f64, 1.0, 1.0, 0.0];
    let y_coords = vec![0.0_f64, 0.0, 1.0, 1.0];
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
    file.put_connectivity(1, &[1_i64, 2, 3, 4])?;

    // Define global variables
    file.define_variables(EntityType::Global, &["time_value", "step_count"])?;

    // Write 3 time steps
    for step in 0..3 {
        let time = step as f64 * 0.1;
        file.put_time(step, time)?;

        // Write global variable values
        file.put_var(step, EntityType::Global, 0, 0, &[time])?; // time_value
        file.put_var(step, EntityType::Global, 0, 1, &[step as f64])?; // step_count
    }

    Ok(())
}

/// Generate mesh with nodal variables
pub fn generate_nodal_variables(path: &Path) -> Result<()> {
    let opts = CreateOptions {
        mode: CreateMode::Clobber,
        ..Default::default()
    };

    let mut file = ExodusFile::create(path, opts)?;

    let params = InitParams {
        title: "Nodal variables test for C compatibility".to_string(),
        num_dim: 2,
        num_nodes: 4,
        num_elems: 1,
        num_elem_blocks: 1,
        ..Default::default()
    };

    file.init(&params)?;

    // Basic mesh
    let x_coords = vec![0.0_f64, 1.0, 1.0, 0.0];
    let y_coords = vec![0.0_f64, 0.0, 1.0, 1.0];
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
    file.put_connectivity(1, &[1_i64, 2, 3, 4])?;

    // Define nodal variables
    file.define_variables(EntityType::Nodal, &["temperature", "pressure"])?;

    // Write 3 time steps
    for step in 0..3 {
        let time = step as f64 * 0.1;
        file.put_time(step, time)?;

        // Write nodal variable values
        let temps = vec![
            100.0 + step as f64 * 10.0,
            110.0 + step as f64 * 10.0,
            120.0 + step as f64 * 10.0,
            130.0 + step as f64 * 10.0,
        ];
        file.put_var(step, EntityType::Nodal, 0, 0, &temps)?;

        let pressures = vec![
            1.0 + step as f64 * 0.1,
            1.1 + step as f64 * 0.1,
            1.2 + step as f64 * 0.1,
            1.3 + step as f64 * 0.1,
        ];
        file.put_var(step, EntityType::Nodal, 0, 1, &pressures)?;
    }

    Ok(())
}

/// Generate mesh with element variables
pub fn generate_element_variables(path: &Path) -> Result<()> {
    let opts = CreateOptions {
        mode: CreateMode::Clobber,
        ..Default::default()
    };

    let mut file = ExodusFile::create(path, opts)?;

    let params = InitParams {
        title: "Element variables test for C compatibility".to_string(),
        num_dim: 2,
        num_nodes: 4,
        num_elems: 1,
        num_elem_blocks: 1,
        ..Default::default()
    };

    file.init(&params)?;

    // Basic mesh
    let x_coords = vec![0.0_f64, 1.0, 1.0, 0.0];
    let y_coords = vec![0.0_f64, 0.0, 1.0, 1.0];
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
    file.put_connectivity(1, &[1_i64, 2, 3, 4])?;

    // Define element variables
    file.define_variables(EntityType::ElemBlock, &["stress", "strain"])?;

    // Write 3 time steps
    for step in 0..3 {
        let time = step as f64 * 0.1;
        file.put_time(step, time)?;

        // Write element variable values (one value per element)
        let stress = vec![1000.0 + step as f64 * 100.0];
        file.put_var(step, EntityType::ElemBlock, 1, 0, &stress)?;

        let strain = vec![0.01 + step as f64 * 0.001];
        file.put_var(step, EntityType::ElemBlock, 1, 1, &strain)?;
    }

    Ok(())
}

/// Generate mesh with all variable types
pub fn generate_all_variables(path: &Path) -> Result<()> {
    let opts = CreateOptions {
        mode: CreateMode::Clobber,
        ..Default::default()
    };

    let mut file = ExodusFile::create(path, opts)?;

    let params = InitParams {
        title: "All variables test for C compatibility".to_string(),
        num_dim: 2,
        num_nodes: 4,
        num_elems: 1,
        num_elem_blocks: 1,
        ..Default::default()
    };

    file.init(&params)?;

    // Basic mesh
    let x_coords = vec![0.0_f64, 1.0, 1.0, 0.0];
    let y_coords = vec![0.0_f64, 0.0, 1.0, 1.0];
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
    file.put_connectivity(1, &[1_i64, 2, 3, 4])?;

    // Define all variable types
    file.define_variables(EntityType::Global, &["time_value"])?;
    file.define_variables(EntityType::Nodal, &["temperature"])?;
    file.define_variables(EntityType::ElemBlock, &["stress"])?;

    // Write 3 time steps
    for step in 0..3 {
        let time = step as f64 * 0.1;
        file.put_time(step, time)?;

        // Global variable
        file.put_var(step, EntityType::Global, 0, 0, &[time])?;

        // Nodal variable
        let temps = vec![
            100.0 + step as f64 * 10.0,
            110.0 + step as f64 * 10.0,
            120.0 + step as f64 * 10.0,
            130.0 + step as f64 * 10.0,
        ];
        file.put_var(step, EntityType::Nodal, 0, 0, &temps)?;

        // Element variable
        let stress = vec![1000.0 + step as f64 * 100.0];
        file.put_var(step, EntityType::ElemBlock, 1, 0, &stress)?;
    }

    Ok(())
}
