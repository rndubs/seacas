//! Tests for variable storage format detection and combined format reading.
//!
//! This module tests the ability to detect and read from files using the combined
//! 3D variable storage format (e.g., `vals_nod_var(time_step, num_vars, num_nodes)`).

#![cfg(feature = "netcdf4")]

use exodus_rs::{
    mode, Block, CreateMode, CreateOptions, EntityType, ExodusFile, InitParams, VarStorageMode,
};
use tempfile::NamedTempFile;

/// Helper to create a test file with clobber mode
fn create_test_file(path: &std::path::Path) -> exodus_rs::Result<ExodusFile<mode::Write>> {
    ExodusFile::create(
        path,
        CreateOptions {
            mode: CreateMode::Clobber,
            ..Default::default()
        },
    )
}

#[test]
fn test_storage_format_detection_separate() {
    // Create a file with separate variable format (default)
    let tmp = NamedTempFile::new().unwrap();
    {
        let mut file = create_test_file(tmp.path()).unwrap();

        // Initialize with some nodes
        file.init(&InitParams {
            title: "Test file".to_string(),
            num_dim: 3,
            num_nodes: 4,
            num_elems: 1,
            num_elem_blocks: 1,
            ..Default::default()
        })
        .unwrap();

        // Add a block
        file.put_block(&Block {
            id: 1,
            entity_type: EntityType::ElemBlock,
            topology: "TET4".to_string(),
            num_entries: 1,
            num_nodes_per_entry: 4,
            num_edges_per_entry: 0,
            num_faces_per_entry: 0,
            num_attributes: 0,
        })
        .unwrap();

        // Add coordinates (y and z are Option<&[T]>)
        file.put_coords(
            &[0.0, 1.0, 0.0, 0.5],
            Some(&[0.0, 0.0, 1.0, 0.5]),
            Some(&[0.0, 0.0, 0.0, 1.0]),
        )
        .unwrap();

        // Add connectivity (only block_id and connectivity, no EntityType)
        file.put_connectivity(1, &[1, 2, 3, 4]).unwrap();

        // Define nodal variables - this creates separate format variables
        file.define_variables(EntityType::Nodal, &["temperature", "pressure"])
            .unwrap();

        // Write time step and variable data
        file.put_time(0, 0.0).unwrap();
        file.put_var(0, EntityType::Nodal, 0, 0, &[100.0, 200.0, 300.0, 400.0])
            .unwrap();
        file.put_var(0, EntityType::Nodal, 0, 1, &[1.0, 2.0, 3.0, 4.0])
            .unwrap();
    }

    // Open and check storage format detection
    let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
    let format = file.storage_format();

    // Should detect separate format since we created with define_variables
    assert_eq!(format.nodal, VarStorageMode::Separate);

    // Read variable values to verify they work
    let temp_values = file.var(0, EntityType::Nodal, 0, 0).unwrap();
    assert_eq!(temp_values.len(), 4);
    assert!((temp_values[0] - 100.0).abs() < 1e-6);

    let pressure_values = file.var(0, EntityType::Nodal, 0, 1).unwrap();
    assert_eq!(pressure_values.len(), 4);
    assert!((pressure_values[0] - 1.0).abs() < 1e-6);
}

#[test]
fn test_storage_format_detection_no_vars() {
    // Create a file without variables
    let tmp = NamedTempFile::new().unwrap();
    {
        let mut file = create_test_file(tmp.path()).unwrap();

        file.init(&InitParams {
            title: "No vars file".to_string(),
            num_dim: 3,
            num_nodes: 4,
            num_elems: 1,
            num_elem_blocks: 1,
            ..Default::default()
        })
        .unwrap();

        file.put_block(&Block {
            id: 1,
            entity_type: EntityType::ElemBlock,
            topology: "TET4".to_string(),
            num_entries: 1,
            num_nodes_per_entry: 4,
            num_edges_per_entry: 0,
            num_faces_per_entry: 0,
            num_attributes: 0,
        })
        .unwrap();

        file.put_coords(
            &[0.0, 1.0, 0.0, 0.5],
            Some(&[0.0, 0.0, 1.0, 0.5]),
            Some(&[0.0, 0.0, 0.0, 1.0]),
        )
        .unwrap();

        file.put_connectivity(1, &[1, 2, 3, 4]).unwrap();
    }

    // Open and check storage format detection
    let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
    let format = file.storage_format();

    // Should detect None since no variables were defined
    assert_eq!(format.nodal, VarStorageMode::None);
    assert_eq!(format.elem_block, VarStorageMode::None);
    assert_eq!(format.global, VarStorageMode::None);
}

#[test]
fn test_storage_format_global_variables() {
    // Create a file with global variables
    let tmp = NamedTempFile::new().unwrap();
    {
        let mut file = create_test_file(tmp.path()).unwrap();

        file.init(&InitParams {
            title: "Global vars file".to_string(),
            num_dim: 3,
            num_nodes: 4,
            num_elems: 1,
            num_elem_blocks: 1,
            ..Default::default()
        })
        .unwrap();

        file.put_block(&Block {
            id: 1,
            entity_type: EntityType::ElemBlock,
            topology: "TET4".to_string(),
            num_entries: 1,
            num_nodes_per_entry: 4,
            num_edges_per_entry: 0,
            num_faces_per_entry: 0,
            num_attributes: 0,
        })
        .unwrap();

        file.put_coords(
            &[0.0, 1.0, 0.0, 0.5],
            Some(&[0.0, 0.0, 1.0, 0.5]),
            Some(&[0.0, 0.0, 0.0, 1.0]),
        )
        .unwrap();

        file.put_connectivity(1, &[1, 2, 3, 4]).unwrap();

        // Define global variables
        file.define_variables(EntityType::Global, &["total_energy", "kinetic_energy"])
            .unwrap();

        // Write time step and global variable data
        file.put_time(0, 0.0).unwrap();
        file.put_var(0, EntityType::Global, 0, 0, &[1000.0])
            .unwrap();
        file.put_var(0, EntityType::Global, 0, 1, &[500.0]).unwrap();
    }

    // Open and check storage format detection
    let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
    let format = file.storage_format();

    // Global variables always use combined format (vals_glo_var)
    assert_eq!(format.global, VarStorageMode::Combined);

    // Read values
    let energy = file.var(0, EntityType::Global, 0, 0).unwrap();
    assert_eq!(energy.len(), 1);
    assert!((energy[0] - 1000.0).abs() < 1e-6);
}

#[test]
fn test_storage_format_accessor() {
    // Test that storage_format() accessor returns correct reference
    let tmp = NamedTempFile::new().unwrap();
    {
        let mut file = create_test_file(tmp.path()).unwrap();
        file.init(&InitParams {
            title: "Accessor test".to_string(),
            num_dim: 3,
            num_nodes: 1,
            ..Default::default()
        })
        .unwrap();
        file.put_coords(&[0.0], Some(&[0.0]), Some(&[0.0])).unwrap();
    }

    let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();

    // Test that we can call storage_format() and access fields
    let format = file.storage_format();
    let _ = format.nodal;
    let _ = format.elem_block;
    let _ = format.global;

    // Verify it returns the same reference each time
    let format2 = file.storage_format();
    assert_eq!(format.nodal, format2.nodal);
}

#[test]
fn test_storage_format_append_mode() {
    // Test storage format detection in append mode
    let tmp = NamedTempFile::new().unwrap();
    {
        let mut file = create_test_file(tmp.path()).unwrap();

        file.init(&InitParams {
            title: "Append test".to_string(),
            num_dim: 3,
            num_nodes: 4,
            num_elems: 1,
            num_elem_blocks: 1,
            ..Default::default()
        })
        .unwrap();

        file.put_block(&Block {
            id: 1,
            entity_type: EntityType::ElemBlock,
            topology: "TET4".to_string(),
            num_entries: 1,
            num_nodes_per_entry: 4,
            num_edges_per_entry: 0,
            num_faces_per_entry: 0,
            num_attributes: 0,
        })
        .unwrap();

        file.put_coords(
            &[0.0, 1.0, 0.0, 0.5],
            Some(&[0.0, 0.0, 1.0, 0.5]),
            Some(&[0.0, 0.0, 0.0, 1.0]),
        )
        .unwrap();

        file.put_connectivity(1, &[1, 2, 3, 4]).unwrap();

        file.define_variables(EntityType::Nodal, &["velocity"])
            .unwrap();

        file.put_time(0, 0.0).unwrap();
        file.put_var(0, EntityType::Nodal, 0, 0, &[1.0, 2.0, 3.0, 4.0])
            .unwrap();
    }

    // Open in append mode and verify format detection
    let file = ExodusFile::<mode::Append>::append(tmp.path()).unwrap();
    let format = file.storage_format();

    assert_eq!(format.nodal, VarStorageMode::Separate);

    // Read values using append mode var() method
    let velocity = file.var(0, EntityType::Nodal, 0, 0).unwrap();
    assert_eq!(velocity.len(), 4);
    assert!((velocity[0] - 1.0).abs() < 1e-6);
}

#[test]
fn test_var_storage_mode_default() {
    // Test that VarStorageMode::default() returns Separate
    let mode = VarStorageMode::default();
    assert_eq!(mode, VarStorageMode::Separate);
}

#[test]
fn test_file_storage_format_default() {
    // Test that FileStorageFormat::default() returns all Separate
    let format = exodus_rs::FileStorageFormat::default();
    assert_eq!(format.nodal, VarStorageMode::Separate);
    assert_eq!(format.elem_block, VarStorageMode::Separate);
    assert_eq!(format.global, VarStorageMode::Separate);
}
