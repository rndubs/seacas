//! Integration tests for copy-mirror-merge (CMM) operations
//!
//! These tests verify:
//! - Basic CMM functionality for different element types (HEX8, TET4, WEDGE6, PYRAMID5, QUAD4, TRI3)
//! - Node merging at symmetry plane
//! - Node set and side set duplication with _mirror suffix
//! - Vector field component negation
//! - Custom merge tolerance
//! - CMM combined with other operations (pre/post transforms)

mod fixtures;
use fixtures::*;

use exodus_rs::EntityType;
use std::process::Command;

const TOLERANCE: f64 = 1e-10;

fn rexonator_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rexonator"))
}

// ========================================================================
// Basic CMM Tests
// ========================================================================

#[test]
fn test_cmm_hex8_basic() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_hex8_mesh(&input).expect("Failed to create HEX8 mesh");

    // Get original mesh stats
    let orig_params = read_params(&input).expect("Failed to read params");

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--copy-mirror-merge",
            "x",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success(), "CMM should succeed");

    // Verify mesh doubled
    let new_params = read_params(&output).expect("Failed to read new params");

    // Elements should double
    assert_eq!(
        new_params.num_elems,
        orig_params.num_elems * 2,
        "Elements should double"
    );

    // Nodes should approximately double minus symmetry plane nodes
    // Original: 18 nodes, 6 on symmetry plane
    // New: 18 + (18-6) = 30 nodes
    assert!(
        new_params.num_nodes > orig_params.num_nodes,
        "Node count should increase"
    );
    assert!(
        new_params.num_nodes < orig_params.num_nodes * 2,
        "Node count should be less than doubled due to merging"
    );
}

#[test]
fn test_cmm_hex8_coordinate_bounds() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_hex8_mesh(&input).expect("Failed to create HEX8 mesh");

    // Original mesh: x in [0, 1]
    let (orig_x, _, _) = read_coord_bounds(&input).unwrap();
    assert!((orig_x[0] - 0.0).abs() < TOLERANCE);
    assert!((orig_x[1] - 1.0).abs() < TOLERANCE);

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--copy-mirror-merge",
            "x",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    // After CMM about x: x should be in [-1, 1]
    let (new_x, new_y, new_z) = read_coord_bounds(&output).unwrap();

    assert!(
        (new_x[0] - (-1.0)).abs() < TOLERANCE,
        "x_min should be -1, got {}",
        new_x[0]
    );
    assert!(
        (new_x[1] - 1.0).abs() < TOLERANCE,
        "x_max should be 1, got {}",
        new_x[1]
    );
    // Y and Z bounds should be unchanged
    assert!((new_y[0] - 0.0).abs() < TOLERANCE);
    assert!((new_y[1] - 1.0).abs() < TOLERANCE);
    assert!((new_z[0] - 0.0).abs() < TOLERANCE);
    assert!((new_z[1] - 1.0).abs() < TOLERANCE);
}

#[test]
fn test_cmm_y_axis() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_hex8_mesh(&input).expect("Failed to create HEX8 mesh");

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--copy-mirror-merge",
            "y",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let (new_x, new_y, _) = read_coord_bounds(&output).unwrap();

    // X bounds unchanged (symmetry about Y)
    assert!((new_x[0] - 0.0).abs() < TOLERANCE);
    assert!((new_x[1] - 1.0).abs() < TOLERANCE);
    // Y should span [-1, 1] (assuming original [0, 1])
    assert!((new_y[0] - (-1.0)).abs() < TOLERANCE, "y_min should be -1");
    assert!((new_y[1] - 1.0).abs() < TOLERANCE, "y_max should be 1");
}

#[test]
fn test_cmm_z_axis() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_hex8_mesh(&input).expect("Failed to create HEX8 mesh");

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--copy-mirror-merge",
            "z",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let (_, _, new_z) = read_coord_bounds(&output).unwrap();

    // Z should span [-1, 1] (original [0, 1] mirrored)
    assert!((new_z[0] - (-1.0)).abs() < TOLERANCE, "z_min should be -1");
    assert!((new_z[1] - 1.0).abs() < TOLERANCE, "z_max should be 1");
}

// ========================================================================
// Element Type Tests
// ========================================================================

#[test]
fn test_cmm_quad4() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_quad4_mesh(&input).expect("Failed to create QUAD4 mesh");

    let orig_params = read_params(&input).unwrap();

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--copy-mirror-merge",
            "x",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let new_params = read_params(&output).unwrap();
    assert_eq!(new_params.num_elems, orig_params.num_elems * 2);

    // Check X bounds extended
    let (new_x, _, _) = read_coord_bounds(&output).unwrap();
    assert!((new_x[0] - (-1.0)).abs() < TOLERANCE);
    assert!((new_x[1] - 1.0).abs() < TOLERANCE);
}

#[test]
fn test_cmm_tri3() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_tri3_mesh(&input).expect("Failed to create TRI3 mesh");

    let orig_params = read_params(&input).unwrap();

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--copy-mirror-merge",
            "x",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let new_params = read_params(&output).unwrap();
    assert_eq!(new_params.num_elems, orig_params.num_elems * 2);
}

#[test]
fn test_cmm_tet4() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_tet4_mesh(&input).expect("Failed to create TET4 mesh");

    let orig_params = read_params(&input).unwrap();

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--copy-mirror-merge",
            "x",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let new_params = read_params(&output).unwrap();
    assert_eq!(new_params.num_elems, orig_params.num_elems * 2);

    // Verify coordinate bounds
    let (new_x, _, _) = read_coord_bounds(&output).unwrap();
    assert!((new_x[0] - (-1.0)).abs() < TOLERANCE);
    assert!((new_x[1] - 1.0).abs() < TOLERANCE);
}

#[test]
fn test_cmm_wedge6() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_wedge6_mesh(&input).expect("Failed to create WEDGE6 mesh");

    let orig_params = read_params(&input).unwrap();

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--copy-mirror-merge",
            "x",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let new_params = read_params(&output).unwrap();
    assert_eq!(new_params.num_elems, orig_params.num_elems * 2);
}

#[test]
fn test_cmm_pyramid5() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_pyramid5_mesh(&input).expect("Failed to create PYRAMID5 mesh");

    let orig_params = read_params(&input).unwrap();

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--copy-mirror-merge",
            "x",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let new_params = read_params(&output).unwrap();
    assert_eq!(new_params.num_elems, orig_params.num_elems * 2);
}

// ========================================================================
// Node Set and Side Set Tests
// ========================================================================

#[test]
fn test_cmm_doubles_node_sets() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_hex8_mesh(&input).expect("Failed to create mesh");

    let orig_params = read_params(&input).unwrap();

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--copy-mirror-merge",
            "x",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let new_params = read_params(&output).unwrap();

    // Node sets should double (original + _mirror variants)
    assert_eq!(
        new_params.num_node_sets,
        orig_params.num_node_sets * 2,
        "Node sets should double"
    );
}

#[test]
fn test_cmm_node_set_names_have_mirror_suffix() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_hex8_mesh(&input).expect("Failed to create mesh");

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--copy-mirror-merge",
            "x",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let names = read_names(&output, EntityType::NodeSet).unwrap();

    // Should have original names plus _mirror variants
    let has_original = names.iter().any(|n| n == "symmetry" || n == "outlet");
    let has_mirror = names.iter().any(|n| n.ends_with("_mirror"));

    assert!(has_original, "Should preserve original names");
    assert!(has_mirror, "Should have _mirror suffix names");
}

#[test]
fn test_cmm_doubles_side_sets() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_hex8_mesh(&input).expect("Failed to create mesh");

    let orig_params = read_params(&input).unwrap();

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--copy-mirror-merge",
            "x",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let new_params = read_params(&output).unwrap();

    assert_eq!(
        new_params.num_side_sets,
        orig_params.num_side_sets * 2,
        "Side sets should double"
    );
}

#[test]
fn test_cmm_doubles_element_blocks() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_hex8_mesh(&input).expect("Failed to create mesh");

    let orig_params = read_params(&input).unwrap();

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--copy-mirror-merge",
            "x",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let new_params = read_params(&output).unwrap();

    assert_eq!(
        new_params.num_elem_blocks,
        orig_params.num_elem_blocks * 2,
        "Element blocks should double"
    );

    // Verify block names include _mirror
    let block_names = read_names(&output, EntityType::ElemBlock).unwrap();
    let has_mirror = block_names.iter().any(|n| n.ends_with("_mirror"));
    assert!(has_mirror, "Should have block with _mirror suffix");
}

// ========================================================================
// Vector Field Tests
// ========================================================================

#[test]
fn test_cmm_negates_vector_x_component() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_hex8_mesh(&input).expect("Failed to create mesh");

    // Original velocity_x values (should be non-zero for non-symmetry nodes)
    let orig_vx = read_nodal_var(&input, 1, 0).expect("Failed to read velocity_x");
    let orig_num_nodes = orig_vx.len();

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--copy-mirror-merge",
            "x",
            "-v",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    // Read new velocity_x values
    let new_vx = read_nodal_var(&output, 1, 0).expect("Failed to read new velocity_x");

    // The new mesh should have more nodes
    assert!(new_vx.len() > orig_num_nodes);

    // Original nodes should have unchanged values
    for i in 0..orig_num_nodes {
        assert!(
            (new_vx[i] - orig_vx[i]).abs() < TOLERANCE,
            "Original node {} velocity_x should be unchanged: {} vs {}",
            i,
            new_vx[i],
            orig_vx[i]
        );
    }

    // New (mirrored) nodes should have negated values
    // This is tricky because we need to match mirrored nodes
    // For now, just verify some negated values exist
    let has_negative = new_vx.iter().any(|&v| v < -TOLERANCE);
    // Original mesh has x coords from 0 to 1, so velocity_x is positive
    // Mirrored nodes should have negative velocity_x
    let orig_has_positive = orig_vx.iter().any(|&v| v > TOLERANCE);
    if orig_has_positive {
        assert!(
            has_negative,
            "Mirrored velocity_x should have negative values"
        );
    }
}

#[test]
fn test_cmm_preserves_scalar_field() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_hex8_mesh(&input).expect("Failed to create mesh");

    // Temperature is a scalar, should not be negated
    let orig_temp = read_nodal_var(&input, 0, 0).expect("Failed to read temperature");
    let orig_num_nodes = orig_temp.len();

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--copy-mirror-merge",
            "x",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let new_temp = read_nodal_var(&output, 0, 0).expect("Failed to read new temperature");

    // Original nodes should have unchanged values
    for i in 0..orig_num_nodes {
        assert!(
            (new_temp[i] - orig_temp[i]).abs() < TOLERANCE,
            "Temperature should be unchanged for original nodes"
        );
    }

    // All temperature values should be non-negative (scalar, not negated)
    assert!(
        new_temp.iter().all(|&t| t >= -TOLERANCE),
        "Temperature should not have negative values"
    );
}

// ========================================================================
// Merge Tolerance Tests
// ========================================================================

#[test]
fn test_cmm_custom_tolerance() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_hex8_mesh(&input).expect("Failed to create mesh");

    // Use larger tolerance
    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--copy-mirror-merge",
            "x",
            "--merge-tolerance",
            "0.1",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let params = read_params(&output).unwrap();
    // Should still work, possibly with different node merge behavior
    assert!(params.num_nodes > 0);
}

#[test]
fn test_cmm_very_small_tolerance() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_hex8_mesh(&input).expect("Failed to create mesh");

    // Very small tolerance - should still work, may merge fewer nodes
    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--copy-mirror-merge",
            "x",
            "--merge-tolerance",
            "0.0001",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());
}

// ========================================================================
// CMM with Other Operations Tests
// ========================================================================

#[test]
fn test_cmm_with_pre_translate() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_hex8_mesh(&input).expect("Failed to create mesh");

    // Translate first, then CMM
    // Original x in [0,1], translate by 1 -> [1,2], then CMM about x
    // Result should be approximately [-2, 2]
    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--translate",
            "1,0,0",
            "--copy-mirror-merge",
            "x",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let (new_x, _, _) = read_coord_bounds(&output).unwrap();

    // After translate by 1: x in [1,2]
    // CMM about x (x=0 plane) mirrors [1,2] to [-2,-1]
    // Combined: x in [-2, 2]
    assert!((new_x[0] - (-2.0)).abs() < TOLERANCE);
    assert!((new_x[1] - 2.0).abs() < TOLERANCE);
}

#[test]
fn test_cmm_with_post_scale() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_hex8_mesh(&input).expect("Failed to create mesh");

    // CMM first, then scale
    // Original x in [0,1], CMM -> [-1,1], scale by 2 -> [-2,2]
    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--copy-mirror-merge",
            "x",
            "--scale-len",
            "2",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let (new_x, _, _) = read_coord_bounds(&output).unwrap();

    assert!((new_x[0] - (-2.0)).abs() < TOLERANCE);
    assert!((new_x[1] - 2.0).abs() < TOLERANCE);
}

#[test]
fn test_cmm_with_pre_and_post_ops() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_hex8_mesh(&input).expect("Failed to create mesh");

    // Pre: scale by 0.5, CMM, Post: translate by 10
    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--scale-len",
            "0.5",
            "--copy-mirror-merge",
            "x",
            "--translate",
            "10,0,0",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let (new_x, _, _) = read_coord_bounds(&output).unwrap();

    // Original [0,1] -> scale 0.5 -> [0, 0.5]
    // CMM about x -> [-0.5, 0.5]
    // Translate by 10 -> [9.5, 10.5]
    assert!((new_x[0] - 9.5).abs() < TOLERANCE);
    assert!((new_x[1] - 10.5).abs() < TOLERANCE);
}

// ========================================================================
// Verbose Output Tests
// ========================================================================

#[test]
fn test_cmm_verbose_output() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_hex8_mesh(&input).expect("Failed to create mesh");

    let result = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--copy-mirror-merge",
            "x",
            "-v",
        ])
        .output()
        .expect("Failed to run rexonator");

    assert!(result.status.success());

    let stdout = String::from_utf8_lossy(&result.stdout);

    // Check for expected verbose output
    assert!(
        stdout.contains("symmetry plane") || stdout.contains("Copy-mirror-merge"),
        "Should mention symmetry plane or copy-mirror-merge"
    );
}

// ========================================================================
// Time Step Preservation Tests
// ========================================================================

#[test]
fn test_cmm_preserves_time_steps() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_hex8_mesh(&input).expect("Failed to create mesh");

    let orig_times = read_times(&input).unwrap();

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--copy-mirror-merge",
            "x",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let new_times = read_times(&output).unwrap();

    assert_eq!(
        orig_times.len(),
        new_times.len(),
        "Time step count should be preserved"
    );

    for (orig, new) in orig_times.iter().zip(new_times.iter()) {
        assert!(
            (orig - new).abs() < TOLERANCE,
            "Time values should be preserved"
        );
    }
}

#[test]
fn test_cmm_with_zero_time() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_mesh_with_time_steps(&input).expect("Failed to create mesh");

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--copy-mirror-merge",
            "x",
            "--zero-time",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let new_times = read_times(&output).unwrap();
    assert!(
        (new_times[0] - 0.0).abs() < TOLERANCE,
        "First time should be 0"
    );
}

// ========================================================================
// Variable Names Preservation Tests
// ========================================================================

#[test]
fn test_cmm_preserves_variable_names() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_hex8_mesh(&input).expect("Failed to create mesh");

    let orig_names = read_nodal_var_names(&input).unwrap();

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--copy-mirror-merge",
            "x",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let new_names = read_nodal_var_names(&output).unwrap();

    assert_eq!(orig_names, new_names, "Variable names should be preserved");
}

// ========================================================================
// Complex Geometry Tests
// ========================================================================

#[test]
fn test_cmm_with_element_variables() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_hex8_with_elem_vars(&input).expect("Failed to create mesh with elem vars");

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--copy-mirror-merge",
            "x",
            "-v",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    // Verify element count doubled
    let new_params = read_params(&output).unwrap();
    assert_eq!(new_params.num_elems, 8); // 4 * 2
}
