//! Basic mesh generation for compatibility testing
//!
//! Creates simple meshes to test fundamental Exodus II operations:
//! - File creation
//! - Initialization
//! - Coordinate writing
//! - Element block definition

use anyhow::Result;
use exodus_rs::{CreateMode, CreateOptions, ExodusFile, InitParams, Topology};
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
    // Create file with default options but clobber mode
    let mut opts = CreateOptions::default();
    opts.mode = CreateMode::Clobber;

    let mut file = ExodusFile::create(path, opts)?;

    // Initialize with 2D parameters
    let params = InitParams {
        title: "Rust-generated 2D mesh for C compatibility test".to_string(),
        num_dim: 2,
        num_nodes: 4,
        num_elem: 1,
        num_elem_blk: 1,
        num_node_sets: 0,
        num_side_sets: 0,
    };

    file.put_init(&params)?;

    // Write coordinates
    let x_coords = vec![0.0_f64, 1.0, 1.0, 0.0];
    let y_coords = vec![0.0_f64, 0.0, 1.0, 1.0];
    file.put_coords(&x_coords, &y_coords, &[])?;

    // Write coordinate names
    file.put_coord_names(&["x", "y"])?;

    // Define element block
    file.put_block(1, Topology::Quad4, 1, 0, 0)?;

    // Write connectivity (1-indexed)
    let connectivity = vec![1, 2, 3, 4];
    file.put_connectivity(1, &connectivity)?;

    // Add QA record
    file.put_qa_record(
        "exodus-rust-writer",
        "0.1.0",
        &chrono::Local::now().format("%Y-%m-%d").to_string(),
        &chrono::Local::now().format("%H:%M:%S").to_string(),
    )?;

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
    // Create file with default options but clobber mode
    let mut opts = CreateOptions::default();
    opts.mode = CreateMode::Clobber;

    let mut file = ExodusFile::create(path, opts)?;

    // Initialize with 3D parameters
    let params = InitParams {
        title: "Rust-generated 3D mesh for C compatibility test".to_string(),
        num_dim: 3,
        num_nodes: 8,
        num_elem: 1,
        num_elem_blk: 1,
        num_node_sets: 0,
        num_side_sets: 0,
    };

    file.put_init(&params)?;

    // Write coordinates (unit cube)
    let x_coords = vec![0.0_f64, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0];
    let y_coords = vec![0.0_f64, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0];
    let z_coords = vec![0.0_f64, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0];
    file.put_coords(&x_coords, &y_coords, &z_coords)?;

    // Write coordinate names
    file.put_coord_names(&["x", "y", "z"])?;

    // Define element block
    file.put_block(1, Topology::Hex8, 1, 0, 0)?;

    // Write connectivity (1-indexed)
    let connectivity = vec![1, 2, 3, 4, 5, 6, 7, 8];
    file.put_connectivity(1, &connectivity)?;

    // Add QA record
    file.put_qa_record(
        "exodus-rust-writer",
        "0.1.0",
        &chrono::Local::now().format("%Y-%m-%d").to_string(),
        &chrono::Local::now().format("%H:%M:%S").to_string(),
    )?;

    Ok(())
}
