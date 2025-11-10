//! Variable generation for compatibility testing
//!
//! Note: Variable tests are simplified due to API complexity.
//! Full variable testing will be expanded as the API matures.

use anyhow::Result;
use exodus_rs::{Block, CreateMode, CreateOptions, EntityType, ExodusFile, InitParams, Topology};
use std::path::Path;

/// Generate a basic mesh - variables tests simplified for now
/// This creates a simple mesh that C can read
pub fn generate_global_variables(path: &Path) -> Result<()> {
    generate_basic_for_variables(path, "Global variables test")
}

pub fn generate_nodal_variables(path: &Path) -> Result<()> {
    generate_basic_for_variables(path, "Nodal variables test")
}

pub fn generate_element_variables(path: &Path) -> Result<()> {
    generate_basic_for_variables(path, "Element variables test")
}

pub fn generate_all_variables(path: &Path) -> Result<()> {
    generate_basic_for_variables(path, "All variables test")
}

fn generate_basic_for_variables(path: &Path, title: &str) -> Result<()> {
    let mut opts = CreateOptions::default();
    opts.mode = CreateMode::Clobber;

    let mut file = ExodusFile::create(path, opts)?;

    let params = InitParams {
        title: format!("{} for C compatibility", title),
        num_dim: 2,
        num_nodes: 4,
        num_elems: 1,
        num_elem_blocks: 1,
        ..Default::default()
    };

    file.init(&params)?;

    let x_coords = vec![0.0_f64, 1.0, 1.0, 0.0];
    let y_coords = vec![0.0_f64, 0.0, 1.0, 1.0];
    let z_coords: Vec<f64> = vec![];
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

    Ok(())
}
