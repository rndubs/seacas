//! Edge case tests for exodus-rs
//!
//! This module tests boundary conditions, error handling, and edge cases
//! to ensure robust behavior in production scenarios.

use exodus_rs::types::{Block, CreateMode, CreateOptions, EntityType, InitParams};
use exodus_rs::ExodusFile;
use tempfile::NamedTempFile;

/// Helper to create a test file with clobber mode
fn create_test_file() -> (NamedTempFile, ExodusFile<exodus_rs::mode::Write>) {
    let tmp = NamedTempFile::new().unwrap();
    let file = ExodusFile::create(
        tmp.path(),
        CreateOptions {
            mode: CreateMode::Clobber,
            ..Default::default()
        },
    )
    .unwrap();
    (tmp, file)
}

// ============================================================================
// Boundary Condition Tests
// ============================================================================

#[test]
fn test_empty_coordinates() {
    let (_tmp, mut file) = create_test_file();

    // Initialize with zero nodes
    let params = InitParams {
        title: "Empty Coordinates Test".into(),
        num_dim: 2,
        num_nodes: 0,
        ..Default::default()
    };
    file.init(&params).unwrap();

    // Writing empty coordinates should succeed
    let x: Vec<f64> = vec![];
    let result = file.put_coords(&x, Some(&[]), None);
    assert!(result.is_ok());
}

#[test]
fn test_single_node() {
    let (_tmp, mut file) = create_test_file();

    // Initialize with single node
    let params = InitParams {
        title: "Single Node Test".into(),
        num_dim: 3,
        num_nodes: 1,
        ..Default::default()
    };
    file.init(&params).unwrap();

    // Single coordinate should work
    let x = vec![1.0];
    let y = vec![2.0];
    let z = vec![3.0];
    file.put_coords(&x, Some(&y), Some(&z)).unwrap();
}

#[test]
fn test_zero_element_block() {
    let (_tmp, mut file) = create_test_file();

    let params = InitParams {
        title: "Zero Elements Test".into(),
        num_dim: 2,
        num_nodes: 4,
        num_elems: 0,
        num_elem_blocks: 1,
        ..Default::default()
    };
    file.init(&params).unwrap();

    // Block with zero elements
    let block = Block {
        id: 100,
        entity_type: EntityType::ElemBlock,
        topology: "QUAD4".into(),
        num_entries: 0,
        num_nodes_per_entry: 4,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    file.put_block(&block).unwrap();

    // Writing empty connectivity should succeed
    let conn: Vec<i64> = vec![];
    let result = file.put_connectivity(100, &conn);
    assert!(result.is_ok());
}

#[test]
fn test_max_dimensions() {
    let (_tmp, mut file) = create_test_file();

    // Test maximum practical dimensionality (3D is standard max for Exodus)
    let params = InitParams {
        title: "Max Dimensions Test".into(),
        num_dim: 3,
        num_nodes: 8,
        num_elems: 1,
        num_elem_blocks: 1,
        ..Default::default()
    };
    file.init(&params).unwrap();

    // 3D coordinates
    let coords = vec![0.0; 8];
    file.put_coords(&coords, Some(&coords), Some(&coords))
        .unwrap();
}

#[test]
fn test_very_long_title() {
    let (_tmp, mut file) = create_test_file();

    // Exodus titles are limited to 80 characters
    let long_title = "A".repeat(100);
    let params = InitParams {
        title: long_title,
        num_dim: 2,
        num_nodes: 4,
        ..Default::default()
    };

    // Should return an error for titles that are too long
    let result = file.init(&params);
    assert!(result.is_err());

    // Now try with a valid title (exactly 80 chars)
    let valid_title = "A".repeat(80);
    let params = InitParams {
        title: valid_title,
        num_dim: 2,
        num_nodes: 4,
        ..Default::default()
    };
    file.init(&params).unwrap();

    // Verify file was initialized successfully
    let coords = vec![0.0; 4];
    file.put_coords(&coords, Some(&coords), None).unwrap();
}

#[test]
fn test_very_long_variable_name() {
    let (_tmp, mut file) = create_test_file();

    let params = InitParams {
        title: "Long Var Name Test".into(),
        num_dim: 2,
        num_nodes: 4,
        ..Default::default()
    };
    file.init(&params).unwrap();

    // Variable names are typically limited to 32 characters
    let long_name = "A".repeat(50);
    let result = file.define_variables(EntityType::Nodal, &[&long_name]);

    // Should either truncate or handle gracefully
    assert!(result.is_ok());
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_coordinate_dimension_mismatch() {
    let (_tmp, mut file) = create_test_file();

    let params = InitParams {
        title: "Dimension Mismatch Test".into(),
        num_dim: 2,
        num_nodes: 4,
        ..Default::default()
    };
    file.init(&params).unwrap();

    // Wrong number of coordinates
    let x = vec![0.0, 1.0, 1.0, 0.0];
    let y = vec![0.0, 0.0, 1.0]; // Too few!

    let result = file.put_coords(&x, Some(&y), None);
    assert!(result.is_err());
}

#[test]
fn test_coordinate_count_mismatch() {
    let (_tmp, mut file) = create_test_file();

    let params = InitParams {
        title: "Count Mismatch Test".into(),
        num_dim: 2,
        num_nodes: 4,
        ..Default::default()
    };
    file.init(&params).unwrap();

    // More coordinates than declared nodes
    let x = vec![0.0; 10];
    let y = vec![0.0; 10];

    let result = file.put_coords(&x, Some(&y), None);
    assert!(result.is_err());
}

#[test]
fn test_invalid_block_topology() {
    let (_tmp, mut file) = create_test_file();

    let params = InitParams {
        title: "Invalid Topology Test".into(),
        num_dim: 2,
        num_nodes: 4,
        num_elems: 1,
        num_elem_blocks: 1,
        ..Default::default()
    };
    file.init(&params).unwrap();

    // Nonsense topology name
    let block = Block {
        id: 100,
        entity_type: EntityType::ElemBlock,
        topology: "INVALID_TOPOLOGY_XYZ".into(),
        num_entries: 1,
        num_nodes_per_entry: 4,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };

    // Should succeed - Exodus allows custom topologies
    let result = file.put_block(&block);
    assert!(result.is_ok());
}

#[test]
fn test_multiple_blocks_different_ids() {
    let (_tmp, mut file) = create_test_file();

    let params = InitParams {
        title: "Multiple Blocks Test".into(),
        num_dim: 2,
        num_nodes: 8,
        num_elems: 2,
        num_elem_blocks: 2,
        ..Default::default()
    };
    file.init(&params).unwrap();

    let block1 = Block {
        id: 100,
        entity_type: EntityType::ElemBlock,
        topology: "QUAD4".into(),
        num_entries: 1,
        num_nodes_per_entry: 4,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    file.put_block(&block1).unwrap();

    // Add another block with different ID
    let block2 = Block {
        id: 200,
        entity_type: EntityType::ElemBlock,
        topology: "QUAD4".into(),
        num_entries: 1,
        num_nodes_per_entry: 4,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };

    // Should succeed with different ID
    let result = file.put_block(&block2);
    assert!(result.is_ok());
}

#[test]
fn test_write_before_init() {
    let (_tmp, mut file) = create_test_file();

    // Try to write coordinates without initializing
    let x = vec![0.0, 1.0];
    let result = file.put_coords(&x, Some(&x), None);

    // Should fail - file not initialized
    assert!(result.is_err());
}

#[test]
fn test_connectivity_mismatch() {
    let (_tmp, mut file) = create_test_file();

    let params = InitParams {
        title: "Connectivity Mismatch Test".into(),
        num_dim: 2,
        num_nodes: 4,
        num_elems: 1,
        num_elem_blocks: 1,
        ..Default::default()
    };
    file.init(&params).unwrap();

    let block = Block {
        id: 100,
        entity_type: EntityType::ElemBlock,
        topology: "QUAD4".into(),
        num_entries: 1,
        num_nodes_per_entry: 4,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    file.put_block(&block).unwrap();

    // Wrong connectivity size (3 nodes instead of 4)
    let conn = vec![1_i64, 2, 3];
    let result = file.put_connectivity(100, &conn);

    // Should fail
    assert!(result.is_err());
}

#[test]
fn test_sequential_time_steps() {
    let (_tmp, mut file) = create_test_file();

    let params = InitParams {
        title: "Sequential Time Steps Test".into(),
        num_dim: 2,
        num_nodes: 4,
        ..Default::default()
    };
    file.init(&params).unwrap();

    file.define_variables(EntityType::Nodal, &["Temperature"])
        .unwrap();

    // Write time steps sequentially
    for step in 0..5 {
        file.put_time(step, step as f64 * 0.1).unwrap();
        let vals = vec![100.0 + step as f64; 4];
        file.put_var(step, EntityType::Nodal, 0, 0, &vals).unwrap();
    }
}

// ============================================================================
// Large Data Tests
// ============================================================================

#[test]
fn test_large_node_count() {
    let (_tmp, mut file) = create_test_file();

    // Test with 10,000 nodes
    let num_nodes = 10_000;
    let params = InitParams {
        title: "Large Node Count Test".into(),
        num_dim: 3,
        num_nodes,
        ..Default::default()
    };
    file.init(&params).unwrap();

    let coords = vec![0.0; num_nodes];
    file.put_coords(&coords, Some(&coords), Some(&coords))
        .unwrap();
}

#[test]
fn test_large_element_count() {
    let (_tmp, mut file) = create_test_file();

    // Test with 5,000 elements
    let num_elems = 5_000;
    let num_nodes = num_elems * 4; // QUAD4 elements

    let params = InitParams {
        title: "Large Element Count Test".into(),
        num_dim: 2,
        num_nodes,
        num_elems,
        num_elem_blocks: 1,
        ..Default::default()
    };
    file.init(&params).unwrap();

    let block = Block {
        id: 100,
        entity_type: EntityType::ElemBlock,
        topology: "QUAD4".into(),
        num_entries: num_elems,
        num_nodes_per_entry: 4,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    file.put_block(&block).unwrap();

    // Generate connectivity (simple sequential)
    let mut conn = Vec::with_capacity(num_elems * 4);
    for i in 0..num_elems {
        let base = (i * 4 + 1) as i64;
        conn.extend_from_slice(&[base, base + 1, base + 2, base + 3]);
    }

    file.put_connectivity(100, &conn).unwrap();
}

#[test]
fn test_many_variables() {
    let (_tmp, mut file) = create_test_file();

    let params = InitParams {
        title: "Many Variables Test".into(),
        num_dim: 2,
        num_nodes: 100,
        ..Default::default()
    };
    file.init(&params).unwrap();

    // Define 50 variables
    let var_names: Vec<String> = (0..50).map(|i| format!("var_{}", i)).collect();
    let var_refs: Vec<&str> = var_names.iter().map(|s| s.as_str()).collect();

    file.define_variables(EntityType::Nodal, &var_refs)
        .unwrap();
}

#[test]
fn test_many_time_steps() {
    let (_tmp, mut file) = create_test_file();

    let params = InitParams {
        title: "Many Time Steps Test".into(),
        num_dim: 2,
        num_nodes: 10,
        ..Default::default()
    };
    file.init(&params).unwrap();

    file.define_variables(EntityType::Nodal, &["Temperature"])
        .unwrap();

    // Write 100 time steps
    for step in 0..100 {
        let time = step as f64 * 0.01;
        file.put_time(step, time).unwrap();

        let vals = vec![100.0 + step as f64; 10];
        file.put_var(step, EntityType::Nodal, 0, 0, &vals).unwrap();
    }
}

// ============================================================================
// Special Value Tests
// ============================================================================

#[test]
fn test_negative_coordinates() {
    let (_tmp, mut file) = create_test_file();

    let params = InitParams {
        title: "Negative Coordinates Test".into(),
        num_dim: 2,
        num_nodes: 4,
        ..Default::default()
    };
    file.init(&params).unwrap();

    // Negative coordinates should be valid
    let x = vec![-1.0, -2.0, -3.0, -4.0];
    let y = vec![-10.0, -20.0, -30.0, -40.0];

    file.put_coords(&x, Some(&y), None).unwrap();
}

#[test]
fn test_zero_coordinates() {
    let (_tmp, mut file) = create_test_file();

    let params = InitParams {
        title: "Zero Coordinates Test".into(),
        num_dim: 3,
        num_nodes: 1,
        ..Default::default()
    };
    file.init(&params).unwrap();

    // All zero coordinates
    let zero = vec![0.0];
    file.put_coords(&zero, Some(&zero), Some(&zero)).unwrap();
}

#[test]
fn test_very_large_coordinates() {
    let (_tmp, mut file) = create_test_file();

    let params = InitParams {
        title: "Large Coordinates Test".into(),
        num_dim: 2,
        num_nodes: 2,
        ..Default::default()
    };
    file.init(&params).unwrap();

    // Very large coordinates
    let x = vec![1.0e100, 1.0e-100];
    let y = vec![1.0e200, 1.0e-200];

    file.put_coords(&x, Some(&y), None).unwrap();
}

#[test]
fn test_negative_time() {
    let (_tmp, mut file) = create_test_file();

    let params = InitParams {
        title: "Negative Time Test".into(),
        num_dim: 2,
        num_nodes: 4,
        ..Default::default()
    };
    file.init(&params).unwrap();

    file.define_variables(EntityType::Global, &["Energy"])
        .unwrap();

    // Negative time should be valid (for some simulations)
    file.put_time(0, -10.0).unwrap();
    file.put_var(0, EntityType::Global, 0, 0, &[100.0]).unwrap();
}
