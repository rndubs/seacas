//! Basic mesh generation for compatibility testing
//!
//! Creates simple meshes to test fundamental Exodus II operations:
//! - File creation
//! - Initialization
//! - Coordinate writing
//! - Element block definition

use anyhow::Result;
use exodus_rs::{Block, CreateMode, CreateOptions, EntityType, ExodusFile, InitParams, Topology};
use std::path::Path;

/// Generate a simple 2D mesh with a single quad element
///
/// Mesh structure:
/// ```text
///   3------2
///   |      |
///   |      |
///   0------1
/// ```
pub fn generate_2d(path: &Path) -> Result<()> {
    // Create file with clobber mode
    let opts = CreateOptions {
        mode: CreateMode::Clobber,
        ..Default::default()
    };

    let mut file = ExodusFile::create(path, opts)?;

    // Initialize with 2D parameters
    let params = InitParams {
        title: "Rust-generated 2D mesh for C compatibility test".to_string(),
        num_dim: 2,
        num_nodes: 4,
        num_elems: 1,
        num_elem_blocks: 1,
        ..Default::default()
    };

    file.init(&params)?;

    // Write coordinates
    let x_coords = vec![0.0_f64, 1.0, 1.0, 0.0];
    let y_coords = vec![0.0_f64, 0.0, 1.0, 1.0];
    file.put_coords(&x_coords, Some(&y_coords), None)?;

    // Define element block
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

    // Write connectivity (1-indexed)
    let connectivity = vec![1_i64, 2, 3, 4];
    file.put_connectivity(1, &connectivity)?;

    Ok(())
}

/// Generate a simple 3D mesh with a single hex element
///
/// Mesh structure:
/// ```text
///        7------6
///       /|     /|
///      4------5 |
///      | 3----|-2
///      |/     |/
///      0------1
/// ```
pub fn generate_3d(path: &Path) -> Result<()> {
    // Create file with clobber mode
    let opts = CreateOptions {
        mode: CreateMode::Clobber,
        ..Default::default()
    };

    let mut file = ExodusFile::create(path, opts)?;

    // Initialize with 3D parameters
    let params = InitParams {
        title: "Rust-generated 3D mesh for C compatibility test".to_string(),
        num_dim: 3,
        num_nodes: 8,
        num_elems: 1,
        num_elem_blocks: 1,
        ..Default::default()
    };

    file.init(&params)?;

    // Write coordinates (unit cube)
    let x_coords = vec![0.0_f64, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0];
    let y_coords = vec![0.0_f64, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0];
    let z_coords = vec![0.0_f64, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0];
    file.put_coords(&x_coords, Some(&y_coords), Some(&z_coords))?;

    // Define element block
    let block = Block {
        id: 1,
        entity_type: EntityType::ElemBlock,
        topology: Topology::Hex8.to_string(),
        num_entries: 1,
        num_nodes_per_entry: 8,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    file.put_block(&block)?;

    // Write connectivity (1-indexed)
    let connectivity = vec![1_i64, 2, 3, 4, 5, 6, 7, 8];
    file.put_connectivity(1, &connectivity)?;

    Ok(())
}
