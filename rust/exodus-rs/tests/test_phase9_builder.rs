//! Integration tests for Phase 9: High-Level Builder API

use exodus_rs::{mode, BlockBuilder, ExodusFile, MeshBuilder};
use tempfile::NamedTempFile;

#[test]
#[cfg(feature = "netcdf4")]
fn test_phase9_2d_quad_mesh() {
    let tmp = NamedTempFile::new().unwrap();

    // Create using builder API
    MeshBuilder::new("Phase 9 Test - 2D Quad")
        .dimensions(2)
        .coordinates(vec![0.0, 1.0, 1.0, 0.0], vec![0.0, 0.0, 1.0, 1.0], vec![])
        .add_block(
            BlockBuilder::new(1, "QUAD4")
                .connectivity(vec![1, 2, 3, 4])
                .build(),
        )
        .write(tmp.path())
        .expect("Failed to write 2D mesh");

    // Read back and verify
    let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
    let params = file.init_params().unwrap();

    assert_eq!(params.num_dim, 2);
    assert_eq!(params.num_nodes, 4);
    assert_eq!(params.num_elems, 1);
    assert_eq!(params.num_elem_blocks, 1);
}

#[test]
#[cfg(feature = "netcdf4")]
fn test_phase9_3d_hex_with_metadata() {
    let tmp = NamedTempFile::new().unwrap();

    // Create using builder API with metadata
    MeshBuilder::new("Phase 9 Test - 3D Hex")
        .dimensions(3)
        .coordinates(
            vec![0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0],
            vec![0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0],
            vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0],
        )
        .add_block(
            BlockBuilder::new(100, "HEX8")
                .connectivity(vec![1, 2, 3, 4, 5, 6, 7, 8])
                .build(),
        )
        .qa_record("exodus-rs", "0.1.0", "2025-11-10", "12:00:00")
        .info("Phase 9 integration test")
        .write(tmp.path())
        .expect("Failed to write 3D mesh");

    // Read back and verify
    let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
    let params = file.init_params().unwrap();

    assert_eq!(params.num_dim, 3);
    assert_eq!(params.num_nodes, 8);
    assert_eq!(params.num_elems, 1);
    assert_eq!(params.num_elem_blocks, 1);
    assert_eq!(params.title, "Phase 9 Test - 3D Hex");
}

#[test]
#[cfg(feature = "netcdf4")]
fn test_phase9_multi_block_with_attributes() {
    let tmp = NamedTempFile::new().unwrap();

    // Create multi-block mesh with attributes
    MeshBuilder::new("Phase 9 Test - Multi-Block")
        .dimensions(3)
        .coordinates(
            vec![0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 2.0, 2.0],
            vec![0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0],
            vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0],
        )
        .add_block(
            BlockBuilder::new(1, "HEX8")
                .connectivity(vec![1, 2, 3, 4, 5, 6, 7, 8])
                .attributes(vec![1.0])
                .attribute_names(vec!["MaterialID"])
                .build(),
        )
        .add_block(
            BlockBuilder::new(2, "TRI3")
                .connectivity(vec![2, 9, 10])
                .attributes(vec![2.0])
                .attribute_names(vec!["MaterialID"])
                .build(),
        )
        .write(tmp.path())
        .expect("Failed to write multi-block mesh");

    // Read back and verify
    let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
    let params = file.init_params().unwrap();

    assert_eq!(params.num_dim, 3);
    assert_eq!(params.num_nodes, 10);
    assert_eq!(params.num_elems, 2);
    assert_eq!(params.num_elem_blocks, 2);

    // Verify blocks
    let block_ids = file
        .block_ids(exodus_rs::types::EntityType::ElemBlock)
        .unwrap();
    assert_eq!(block_ids.len(), 2);
    assert!(block_ids.contains(&1));
    assert!(block_ids.contains(&2));
}

#[test]
#[cfg(feature = "netcdf4")]
fn test_phase9_coordinates_readback() {
    let tmp = NamedTempFile::new().unwrap();

    let x_coords = vec![0.0, 1.0, 1.0, 0.0];
    let y_coords = vec![0.0, 0.0, 1.0, 1.0];

    // Create mesh
    MeshBuilder::new("Coordinate Test")
        .dimensions(2)
        .coordinates(x_coords.clone(), y_coords.clone(), vec![])
        .add_block(
            BlockBuilder::new(1, "QUAD4")
                .connectivity(vec![1, 2, 3, 4])
                .build(),
        )
        .write(tmp.path())
        .unwrap();

    // Read back coordinates
    let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
    let coords = file.coords::<f64>().unwrap();

    assert_eq!(coords.x, x_coords);
    assert_eq!(coords.y, y_coords);
}

#[test]
#[cfg(feature = "netcdf4")]
fn test_phase9_connectivity_readback() {
    let tmp = NamedTempFile::new().unwrap();

    let connectivity = vec![1, 2, 3, 4, 5, 6, 7, 8];

    // Create mesh
    MeshBuilder::new("Connectivity Test")
        .dimensions(3)
        .coordinates(
            vec![0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0],
            vec![0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0],
            vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0],
        )
        .add_block(
            BlockBuilder::new(1, "HEX8")
                .connectivity(connectivity.clone())
                .build(),
        )
        .write(tmp.path())
        .unwrap();

    // Read back connectivity
    let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
    let conn = file.connectivity(1).unwrap();

    assert_eq!(conn, connectivity);
}
