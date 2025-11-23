//! Integration tests for ndarray support
//!
//! These tests verify the ndarray integration for NumPy compatibility.

#![cfg(all(test, feature = "ndarray"))]

use exodus_rs::{mode, Block, CreateMode, EntityType, ExodusFile, InitParams};
use tempfile::NamedTempFile;

/// Create a basic test file with mesh data
fn create_test_file_with_mesh() -> (NamedTempFile, usize, usize) {
    let tmp = NamedTempFile::new().unwrap();

    let num_nodes = 8;
    let num_elem = 2;

    // Create file with mesh
    {
        let mut file = ExodusFile::<mode::Write>::create(
            tmp.path(),
            exodus_rs::CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            },
        )
        .unwrap();

        // Initialize
        file.init(&InitParams {
            num_dim: 3,
            num_nodes,
            num_elems: num_elem,
            num_elem_blocks: 1,
            num_node_sets: 0,
            num_side_sets: 0,
            title: "Test Mesh".to_string(),
            ..Default::default()
        })
        .unwrap();

        // Write coordinates (simple cube)
        let x = vec![0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0];
        let y = vec![0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0];
        let z = vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0];
        file.put_coords(&x, Some(&y), Some(&z)).unwrap();

        // Write element block
        file.put_block(&Block {
            id: 1,
            entity_type: EntityType::ElemBlock,
            topology: "HEX8".to_string(),
            num_entries: num_elem,
            num_nodes_per_entry: 8,
            num_edges_per_entry: 0,
            num_faces_per_entry: 0,
            num_attributes: 0,
        })
        .unwrap();

        // Write connectivity
        let conn = vec![
            1, 2, 3, 4, 5, 6, 7, 8, // Element 1
            1, 2, 6, 5, 4, 3, 7, 8, // Element 2
        ];
        file.put_connectivity(1, &conn).unwrap();
    }

    (tmp, num_nodes, num_elem)
}

/// Create a test file with variables
fn create_test_file_with_vars() -> (NamedTempFile, usize, usize, usize) {
    let tmp = NamedTempFile::new().unwrap();

    let num_nodes = 10;
    let num_elem = 5;
    let num_steps = 3;

    // Create file with variables
    {
        let mut file = ExodusFile::<mode::Write>::create(
            tmp.path(),
            exodus_rs::CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            },
        )
        .unwrap();

        // Initialize
        file.init(&InitParams {
            num_dim: 2,
            num_nodes,
            num_elems: num_elem,
            num_elem_blocks: 1,
            num_node_sets: 0,
            num_side_sets: 0,
            title: "Test Vars".to_string(),
            ..Default::default()
        })
        .unwrap();

        // Write coordinates
        let x: Vec<f64> = (0..num_nodes).map(|i| i as f64).collect();
        let y: Vec<f64> = (0..num_nodes).map(|i| (i as f64) * 0.5).collect();
        file.put_coords(&x, Some(&y), None).unwrap();

        // Write element block
        file.put_block(&Block {
            id: 1,
            entity_type: EntityType::ElemBlock,
            topology: "QUAD4".to_string(),
            num_entries: num_elem,
            num_nodes_per_entry: 4,
            num_edges_per_entry: 0,
            num_faces_per_entry: 0,
            num_attributes: 0,
        })
        .unwrap();

        // Write connectivity
        let mut conn = Vec::new();
        for i in 0..num_elem {
            let base = (i * 2 + 1) as i64;
            conn.extend_from_slice(&[base, base + 1, base + 3, base + 2]);
        }
        file.put_connectivity(1, &conn).unwrap();

        // Define nodal variables
        file.define_variables(EntityType::Nodal, &["temperature", "pressure"])
            .unwrap();

        // Write time steps with variables
        for step in 0..num_steps {
            file.put_time(step, (step as f64) * 0.1).unwrap();

            // Temperature increases with time
            let temp: Vec<f64> = (0..num_nodes)
                .map(|i| 20.0 + (step as f64) * 10.0 + (i as f64))
                .collect();
            file.put_var(step, EntityType::Nodal, 0, 0, &temp).unwrap();

            // Pressure decreases with time
            let pressure: Vec<f64> = (0..num_nodes)
                .map(|i| 100.0 - (step as f64) * 5.0 + (i as f64) * 0.1)
                .collect();
            file.put_var(step, EntityType::Nodal, 0, 1, &pressure)
                .unwrap();
        }
    }

    (tmp, num_nodes, num_elem, num_steps)
}

#[test]
fn test_coords_array_shape() {
    let (tmp, num_nodes, _) = create_test_file_with_mesh();

    let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
    let coords = file.coords_array().unwrap();

    assert_eq!(coords.shape(), &[num_nodes, 3]);
    assert!(coords.is_standard_layout()); // C-contiguous for NumPy
}

#[test]
fn test_coords_array_values() {
    let (tmp, _, _) = create_test_file_with_mesh();

    let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
    let coords = file.coords_array().unwrap();

    // Check first node (origin)
    assert!((coords[[0, 0]] - 0.0).abs() < 1e-10);
    assert!((coords[[0, 1]] - 0.0).abs() < 1e-10);
    assert!((coords[[0, 2]] - 0.0).abs() < 1e-10);

    // Check node at (1, 1, 1)
    assert!((coords[[6, 0]] - 1.0).abs() < 1e-10);
    assert!((coords[[6, 1]] - 1.0).abs() < 1e-10);
    assert!((coords[[6, 2]] - 1.0).abs() < 1e-10);
}

#[test]
fn test_coords_array_2d() {
    let tmp = NamedTempFile::new().unwrap();

    // Create 2D mesh
    {
        let mut file = ExodusFile::<mode::Write>::create(
            tmp.path(),
            exodus_rs::CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            },
        )
        .unwrap();

        file.init(&InitParams {
            num_dim: 2,
            num_nodes: 4,
            num_elems: 1,
            num_elem_blocks: 1,
            num_node_sets: 0,
            num_side_sets: 0,
            title: "2D Mesh".to_string(),
            ..Default::default()
        })
        .unwrap();

        let x = vec![0.0, 1.0, 1.0, 0.0];
        let y = vec![0.0, 0.0, 1.0, 1.0];
        file.put_coords(&x, Some(&y), None).unwrap();
    }

    // Read and verify
    let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
    let coords = file.coords_array().unwrap();

    assert_eq!(coords.shape(), &[4, 3]);
    // Z coordinates should be 0 for 2D mesh
    for i in 0..4 {
        assert!((coords[[i, 2]] - 0.0).abs() < 1e-10);
    }
}

#[test]
fn test_connectivity_array_shape() {
    let (tmp, _, num_elem) = create_test_file_with_mesh();

    let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
    let conn = file.connectivity_array(1).unwrap();

    assert_eq!(conn.shape(), &[num_elem, 8]); // 2 HEX8 elements
    assert!(conn.is_standard_layout()); // C-contiguous for NumPy
}

#[test]
fn test_connectivity_array_values() {
    let (tmp, _, _) = create_test_file_with_mesh();

    let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
    let conn = file.connectivity_array(1).unwrap();

    // Check first element connectivity
    let elem0 = conn.row(0);
    assert_eq!(elem0[0], 1);
    assert_eq!(elem0[1], 2);
    assert_eq!(elem0[7], 8);

    // Check second element
    let elem1 = conn.row(1);
    assert_eq!(elem1[0], 1);
    assert_eq!(elem1[5], 3);
}

#[test]
fn test_var_time_series_array_shape() {
    let (tmp, num_nodes, _, num_steps) = create_test_file_with_vars();

    let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();

    // Read temperature time series
    let temps = file
        .var_time_series_array(0, num_steps, EntityType::Nodal, 0, 0)
        .unwrap();

    assert_eq!(temps.shape(), &[num_steps, num_nodes]);
    assert!(temps.is_standard_layout()); // C-contiguous for NumPy
}

#[test]
fn test_var_time_series_array_values() {
    let (tmp, num_nodes, _, num_steps) = create_test_file_with_vars();

    let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();

    // Read temperature time series
    let temps = file
        .var_time_series_array(0, num_steps, EntityType::Nodal, 0, 0)
        .unwrap();

    // Verify values at different time steps
    for step in 0..num_steps {
        for node in 0..num_nodes {
            let expected = 20.0 + (step as f64) * 10.0 + (node as f64);
            let actual = temps[[step, node]];
            assert!(
                (actual - expected).abs() < 1e-10,
                "step={}, node={}, expected={}, actual={}",
                step,
                node,
                expected,
                actual
            );
        }
    }
}

#[test]
fn test_var_time_series_array_subset() {
    let (tmp, num_nodes, _, _) = create_test_file_with_vars();

    let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();

    // Read only steps 1-2 (exclusive end)
    let temps = file
        .var_time_series_array(1, 3, EntityType::Nodal, 0, 0)
        .unwrap();

    assert_eq!(temps.shape(), &[2, num_nodes]);

    // Verify it's the correct subset
    let expected_step1_node0 = 20.0 + 10.0; // step 1, node 0
    assert!((temps[[0, 0]] - expected_step1_node0).abs() < 1e-10);
}

#[test]
fn test_var_time_series_array_column_access() {
    let (tmp, _, _, num_steps) = create_test_file_with_vars();

    let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
    let temps = file
        .var_time_series_array(0, num_steps, EntityType::Nodal, 0, 0)
        .unwrap();

    // Access time history for a single node
    let node_5_history = temps.column(5);
    assert_eq!(node_5_history.len(), num_steps);

    for step in 0..num_steps {
        let expected = 20.0 + (step as f64) * 10.0 + 5.0;
        assert!((node_5_history[step] - expected).abs() < 1e-10);
    }
}

#[test]
fn test_var_time_series_array_row_access() {
    let (tmp, num_nodes, _, num_steps) = create_test_file_with_vars();

    let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
    let temps = file
        .var_time_series_array(0, num_steps, EntityType::Nodal, 0, 0)
        .unwrap();

    // Access single time step
    let step_2 = temps.row(2);
    assert_eq!(step_2.len(), num_nodes);

    for node in 0..num_nodes {
        let expected = 20.0 + 20.0 + (node as f64); // step 2
        assert!((step_2[node] - expected).abs() < 1e-10);
    }
}

#[test]
fn test_coords_array_empty() {
    let tmp = NamedTempFile::new().unwrap();

    // Create file with 0 nodes
    {
        let mut file = ExodusFile::<mode::Write>::create(
            tmp.path(),
            exodus_rs::CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            },
        )
        .unwrap();

        file.init(&InitParams {
            num_dim: 3,
            num_nodes: 0,
            num_elems: 0,
            num_elem_blocks: 0,
            num_node_sets: 0,
            num_side_sets: 0,
            title: "Empty".to_string(),
            ..Default::default()
        })
        .unwrap();
    }

    let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
    let coords = file.coords_array().unwrap();

    assert_eq!(coords.shape(), &[0, 3]);
}

#[test]
fn test_multiple_vars() {
    let (tmp, num_nodes, _, num_steps) = create_test_file_with_vars();

    let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();

    // Read both variables
    let temps = file
        .var_time_series_array(0, num_steps, EntityType::Nodal, 0, 0)
        .unwrap();
    let pressure = file
        .var_time_series_array(0, num_steps, EntityType::Nodal, 0, 1)
        .unwrap();

    assert_eq!(temps.shape(), &[num_steps, num_nodes]);
    assert_eq!(pressure.shape(), &[num_steps, num_nodes]);

    // Verify they have different values
    assert!((temps[[0, 0]] - pressure[[0, 0]]).abs() > 1.0);
}

#[test]
fn test_array_memory_layout() {
    let (tmp, num_nodes, _) = create_test_file_with_mesh();

    let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
    let coords = file.coords_array().unwrap();

    // Verify memory layout is C-contiguous (row-major)
    // This is required for efficient zero-copy with NumPy
    assert!(
        coords.is_standard_layout(),
        "Array must be C-contiguous for NumPy compatibility"
    );

    // Verify shape and strides
    assert_eq!(coords.shape(), &[num_nodes, 3]);
    assert_eq!(coords.strides(), &[3, 1]); // Row-major strides
}

#[test]
fn test_coords_array_1d() {
    // Test 1D mesh edge case for coords_array()
    let tmp = NamedTempFile::new().unwrap();

    // Create 1D mesh
    {
        let mut file = ExodusFile::<mode::Write>::create(
            tmp.path(),
            exodus_rs::CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            },
        )
        .unwrap();

        file.init(&InitParams {
            num_dim: 1,
            num_nodes: 5,
            num_elems: 4,
            num_elem_blocks: 1,
            num_node_sets: 0,
            num_side_sets: 0,
            title: "1D Mesh".to_string(),
            ..Default::default()
        })
        .unwrap();

        let x = vec![0.0, 0.25, 0.5, 0.75, 1.0];
        file.put_coords(&x, None, None).unwrap();
    }

    // Read and verify
    let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
    let coords = file.coords_array().unwrap();

    assert_eq!(coords.shape(), &[5, 3]);
    // X coordinates should match
    for i in 0..5 {
        assert!((coords[[i, 0]] - (i as f64) * 0.25).abs() < 1e-10);
        // Y and Z coordinates should be 0 for 1D mesh
        assert!((coords[[i, 1]] - 0.0).abs() < 1e-10);
        assert!((coords[[i, 2]] - 0.0).abs() < 1e-10);
    }
}

#[test]
fn test_connectivity_array_empty_block() {
    // Test empty connectivity array edge case
    let tmp = NamedTempFile::new().unwrap();

    // Create file with empty block
    {
        let mut file = ExodusFile::<mode::Write>::create(
            tmp.path(),
            exodus_rs::CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            },
        )
        .unwrap();

        file.init(&InitParams {
            num_dim: 3,
            num_nodes: 4,
            num_elems: 0, // No elements
            num_elem_blocks: 1,
            num_node_sets: 0,
            num_side_sets: 0,
            title: "Empty Block Mesh".to_string(),
            ..Default::default()
        })
        .unwrap();

        // Write some coordinates
        let x = vec![0.0, 1.0, 1.0, 0.0];
        let y = vec![0.0, 0.0, 1.0, 1.0];
        let z = vec![0.0, 0.0, 0.0, 0.0];
        file.put_coords(&x, Some(&y), Some(&z)).unwrap();

        // Write empty element block
        file.put_block(&Block {
            id: 1,
            entity_type: EntityType::ElemBlock,
            topology: "QUAD4".to_string(),
            num_entries: 0, // No elements in this block
            num_nodes_per_entry: 4,
            num_edges_per_entry: 0,
            num_faces_per_entry: 0,
            num_attributes: 0,
        })
        .unwrap();
    }

    // Read and verify empty connectivity array
    let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();
    let conn = file.connectivity_array(1).unwrap();

    // Empty block should return array with shape (0, 0)
    assert_eq!(conn.shape(), &[0, 0]);
}

#[test]
fn test_var_time_series_single_step() {
    // Test var_time_series_array with a single time step
    let (tmp, num_nodes, _, _) = create_test_file_with_vars();

    let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();

    // Read only a single step
    let temps = file
        .var_time_series_array(1, 2, EntityType::Nodal, 0, 0)
        .unwrap();

    assert_eq!(temps.shape(), &[1, num_nodes]);

    // Verify values for step 1
    for node in 0..num_nodes {
        let expected = 20.0 + 10.0 + (node as f64); // step 1
        assert!((temps[[0, node]] - expected).abs() < 1e-10);
    }
}

#[test]
fn test_var_time_series_all_steps() {
    // Test reading all time steps at once
    let (tmp, num_nodes, _, num_steps) = create_test_file_with_vars();

    let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();

    // Read all steps
    let pressure = file
        .var_time_series_array(0, num_steps, EntityType::Nodal, 0, 1)
        .unwrap();

    assert_eq!(pressure.shape(), &[num_steps, num_nodes]);
    assert!(pressure.is_standard_layout());

    // Verify some values
    for step in 0..num_steps {
        for node in 0..num_nodes {
            let expected = 100.0 - (step as f64) * 5.0 + (node as f64) * 0.1;
            assert!(
                (pressure[[step, node]] - expected).abs() < 1e-10,
                "Pressure mismatch at step={}, node={}",
                step,
                node
            );
        }
    }
}

#[test]
fn test_connectivity_array_multiple_blocks() {
    // Test connectivity_array with multiple element blocks
    let tmp = NamedTempFile::new().unwrap();

    {
        let mut file = ExodusFile::<mode::Write>::create(
            tmp.path(),
            exodus_rs::CreateOptions {
                mode: CreateMode::Clobber,
                ..Default::default()
            },
        )
        .unwrap();

        file.init(&InitParams {
            num_dim: 2,
            num_nodes: 9,
            num_elems: 4,
            num_elem_blocks: 2,
            num_node_sets: 0,
            num_side_sets: 0,
            title: "Multi-block Mesh".to_string(),
            ..Default::default()
        })
        .unwrap();

        // Write coordinates (3x3 grid)
        let x = vec![0.0, 1.0, 2.0, 0.0, 1.0, 2.0, 0.0, 1.0, 2.0];
        let y = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 2.0, 2.0, 2.0];
        file.put_coords(&x, Some(&y), None).unwrap();

        // Block 1: 2 QUAD4 elements
        file.put_block(&Block {
            id: 1,
            entity_type: EntityType::ElemBlock,
            topology: "QUAD4".to_string(),
            num_entries: 2,
            num_nodes_per_entry: 4,
            num_edges_per_entry: 0,
            num_faces_per_entry: 0,
            num_attributes: 0,
        })
        .unwrap();
        file.put_connectivity(1, &[1, 2, 5, 4, 2, 3, 6, 5]).unwrap();

        // Block 2: 2 QUAD4 elements
        file.put_block(&Block {
            id: 2,
            entity_type: EntityType::ElemBlock,
            topology: "QUAD4".to_string(),
            num_entries: 2,
            num_nodes_per_entry: 4,
            num_edges_per_entry: 0,
            num_faces_per_entry: 0,
            num_attributes: 0,
        })
        .unwrap();
        file.put_connectivity(2, &[4, 5, 8, 7, 5, 6, 9, 8]).unwrap();
    }

    let file = ExodusFile::<mode::Read>::open(tmp.path()).unwrap();

    // Read block 1 connectivity
    let conn1 = file.connectivity_array(1).unwrap();
    assert_eq!(conn1.shape(), &[2, 4]);
    assert_eq!(conn1[[0, 0]], 1);
    assert_eq!(conn1[[0, 3]], 4);
    assert_eq!(conn1[[1, 0]], 2);

    // Read block 2 connectivity
    let conn2 = file.connectivity_array(2).unwrap();
    assert_eq!(conn2.shape(), &[2, 4]);
    assert_eq!(conn2[[0, 0]], 4);
    assert_eq!(conn2[[1, 3]], 8);
}
