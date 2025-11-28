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
use serial_test::serial;
use std::process::Command;

const TOLERANCE: f64 = 1e-10;

fn rexonator_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rexonator"))
}

// ========================================================================
// Basic CMM Tests
// ========================================================================

#[test]
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
// Side Set Side Number Mapping Tests
// ========================================================================

#[test]
#[serial]
fn test_cmm_side_numbers_properly_mapped() {
    // When mirroring elements, the side numbers may need to change
    // depending on the element topology and mirror axis.
    //
    // For HEX8 mirrored about X:
    // - Side 2 (+X face) ↔ Side 4 (-X face) swap
    // - Sides 1, 3, 5, 6 remain unchanged (not perpendicular to X)
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_hex8_mesh(&input).expect("Failed to create mesh");

    // Verify input side set (side 5 = bottom/-Z face for all elements)
    let input_ss_ids = read_side_set_ids(&input).expect("Failed to read input side set IDs");
    assert_eq!(input_ss_ids, vec![1], "Input should have 1 side set");

    let (input_elems, input_sides, _) =
        read_side_set(&input, 1).expect("Failed to read input side set");
    assert_eq!(input_elems, vec![1, 2, 3, 4]);
    assert_eq!(input_sides, vec![5, 5, 5, 5]); // Side 5 is -Z for HEX8

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

    // After CMM, we should have 2 side sets: original and mirrored
    let output_ss_ids = read_side_set_ids(&output).expect("Failed to read output side set IDs");
    assert_eq!(output_ss_ids.len(), 2, "Output should have 2 side sets");

    // Original side set should be unchanged
    let (orig_elems, orig_sides, _) =
        read_side_set(&output, output_ss_ids[0]).expect("Failed to read original side set");
    assert_eq!(orig_elems, vec![1, 2, 3, 4]);
    assert_eq!(orig_sides, vec![5, 5, 5, 5]); // Original sides unchanged

    // Mirrored side set should have elements offset by 4 (original element count)
    let (mirror_elems, mirror_sides, _) =
        read_side_set(&output, output_ss_ids[1]).expect("Failed to read mirrored side set");
    assert_eq!(mirror_elems, vec![5, 6, 7, 8]); // Elements offset by 4

    // For X-axis mirror with HEX8:
    // - Side 5 (-Z) is not perpendicular to X, so it stays as side 5
    assert_eq!(
        mirror_sides,
        vec![5, 5, 5, 5],
        "Side 5 should remain side 5 for X-axis mirror (not perpendicular to X)"
    );
}

#[test]
#[serial]
fn test_cmm_side_numbers_mapped_for_perpendicular_faces() {
    // Test that faces perpendicular to the mirror axis ARE remapped.
    // We need to create a mesh with a side set on a face perpendicular to X.
    use exodus_rs::{types::*, ExodusFile};

    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    // Create a HEX8 mesh with a side set on the +X face (side 2)
    let x_coords: Vec<f64> = vec![
        0.0, 1.0, 1.0, 0.0, // bottom layer
        0.0, 1.0, 1.0, 0.0, // top layer
    ];
    let y_coords: Vec<f64> = vec![0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 1.0];
    let z_coords: Vec<f64> = vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0];

    // Single HEX8 element
    let connectivity: Vec<i64> = vec![1, 2, 3, 4, 5, 6, 7, 8];

    let options = CreateOptions {
        mode: CreateMode::Clobber,
        ..Default::default()
    };
    let mut file = ExodusFile::create(&input, options).expect("Failed to create file");

    let params = InitParams {
        title: "Side mapping test".to_string(),
        num_dim: 3,
        num_nodes: 8,
        num_elems: 1,
        num_elem_blocks: 1,
        num_node_sets: 0,
        num_side_sets: 1,
        ..Default::default()
    };
    file.init(&params).expect("Failed to init");
    file.put_coords(&x_coords, Some(&y_coords), Some(&z_coords))
        .expect("Failed to put coords");

    let block = Block {
        id: 1,
        entity_type: EntityType::ElemBlock,
        topology: "HEX8".to_string(),
        num_entries: 1,
        num_nodes_per_entry: 8,
        num_edges_per_entry: 0,
        num_faces_per_entry: 0,
        num_attributes: 0,
    };
    file.put_block(&block).expect("Failed to put block");
    file.put_connectivity(1, &connectivity)
        .expect("Failed to put connectivity");

    // Side set on +X face (side 2 for HEX8)
    let side_elems: Vec<i64> = vec![1];
    let side_nums: Vec<i64> = vec![2]; // +X face
    file.put_side_set(1, &side_elems, &side_nums, None)
        .expect("Failed to put side set");

    file.sync().expect("Failed to sync");
    drop(file);

    // Run CMM about X axis
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

    // Read the mirrored side set
    let output_ss_ids = read_side_set_ids(&output).expect("Failed to read output side set IDs");
    assert_eq!(output_ss_ids.len(), 2);

    // Original side set: element 1, side 2 (should be unchanged)
    let (orig_elems, orig_sides, _) =
        read_side_set(&output, output_ss_ids[0]).expect("Failed to read original side set");
    assert_eq!(orig_elems, vec![1]);
    assert_eq!(orig_sides, vec![2]); // Original side 2 (+X)

    // Mirrored side set: element 2 (mirrored), side should be remapped
    // For X-axis mirror: side 2 (+X) → side 4 (-X)
    let (mirror_elems, mirror_sides, _) =
        read_side_set(&output, output_ss_ids[1]).expect("Failed to read mirrored side set");
    assert_eq!(mirror_elems, vec![2]);
    assert_eq!(
        mirror_sides,
        vec![4],
        "Side 2 (+X) should map to side 4 (-X) for X-axis mirror"
    );
}

// ========================================================================
// Vector Field Tests
// ========================================================================

#[test]
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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

#[test]
#[serial]
fn test_cmm_warns_about_global_variables() {
    // When CMM is used on a mesh with global variables, a warning
    // should be printed because some global vars may need manual
    // adjustment (e.g., total mass should double, time step unchanged).
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_mesh_with_global_vars(&input).expect("Failed to create mesh");

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

    let stderr = String::from_utf8_lossy(&result.stderr);

    // The warning about global variables should be present
    assert!(
        stderr.contains("WARNING") || stderr.contains("global"),
        "Should warn about global variables needing manual adjustment"
    );
}

// ========================================================================
// Time Step Preservation Tests
// ========================================================================

#[test]
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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

#[test]
#[serial]
fn test_cmm_preserves_2d_dimensionality() {
    // A 2D mesh should remain 2D after CMM (num_dim should not change).
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_quad4_mesh(&input).expect("Failed to create 2D mesh");

    let orig_params = read_params(&input).unwrap();
    assert_eq!(orig_params.num_dim, 2, "Input should be 2D");

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
    assert_eq!(new_params.num_dim, 2, "Output should remain 2D");
}

// ========================================================================
// Vector Component Detection Tests
// ========================================================================

#[test]
#[serial]
fn test_vector_detection_should_not_match_max_x() {
    // A field named "max_x" should NOT be negated during CMM
    // because it's not a vector component (it's a maximum X value).
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    // Create mesh with both vector and false-positive variable names
    create_mesh_with_false_positive_vars(&input).expect("Failed to create mesh");

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

    // Read max_x values from input and output
    // Variable order: velocity_x (0), max_x (1), index_x (2), temperature (3)
    let input_max_x = read_nodal_var(&input, 1, 0).expect("Failed to read input max_x");
    let output_max_x = read_nodal_var(&output, 1, 0).expect("Failed to read output max_x");

    // max_x should NOT be negated - original nodes should have same values
    let orig_node_count = input_max_x.len();
    for i in 0..orig_node_count {
        assert!(
            (input_max_x[i] - output_max_x[i]).abs() < 1e-10,
            "max_x should NOT be negated at node {}: input={}, output={}",
            i,
            input_max_x[i],
            output_max_x[i]
        );
    }
}

#[test]
#[serial]
fn test_vector_detection_should_not_match_index_x() {
    // A field named "index_x" should NOT be treated as a vector component
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_mesh_with_false_positive_vars(&input).expect("Failed to create mesh");

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

    // Read index_x values from input and output
    // Variable order: velocity_x (0), max_x (1), index_x (2), temperature (3)
    let input_index_x = read_nodal_var(&input, 2, 0).expect("Failed to read input index_x");
    let output_index_x = read_nodal_var(&output, 2, 0).expect("Failed to read output index_x");

    // index_x should NOT be negated
    let orig_node_count = input_index_x.len();
    for i in 0..orig_node_count {
        assert!(
            (input_index_x[i] - output_index_x[i]).abs() < 1e-10,
            "index_x should NOT be negated at node {}: input={}, output={}",
            i,
            input_index_x[i],
            output_index_x[i]
        );
    }
}

#[test]
#[serial]
fn test_vector_detection_velocity_x_is_negated() {
    // Verify that actual vector components ARE still negated correctly
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_mesh_with_false_positive_vars(&input).expect("Failed to create mesh");

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

    // Read velocity_x values from input and output
    // Variable order: velocity_x (0), max_x (1), index_x (2), temperature (3)
    let input_velocity_x = read_nodal_var(&input, 0, 0).expect("Failed to read input velocity_x");
    let output_velocity_x =
        read_nodal_var(&output, 0, 0).expect("Failed to read output velocity_x");

    // Original nodes should have the same values
    let orig_node_count = input_velocity_x.len();
    for i in 0..orig_node_count {
        assert!(
            (input_velocity_x[i] - output_velocity_x[i]).abs() < 1e-10,
            "velocity_x original values should be preserved at node {}: input={}, output={}",
            i,
            input_velocity_x[i],
            output_velocity_x[i]
        );
    }

    // Mirrored nodes should have negated values
    // The mesh has 18 original nodes, and mirrored nodes are added after
    let output_params = read_params(&output).expect("Failed to read output params");
    assert!(
        output_params.num_nodes > orig_node_count,
        "Should have more nodes after CMM"
    );

    // Check that some mirrored velocity values are negative (for nodes not on symmetry plane)
    let has_negative = output_velocity_x[orig_node_count..]
        .iter()
        .any(|&v| v < 0.0);
    assert!(
        has_negative,
        "Mirrored velocity_x values should include negated values"
    );
}

// ========================================================================
// Progress Indicator Tests
// ========================================================================

/// Test that verbose mode with progress indicators works correctly
#[test]
#[serial]
fn test_verbose_progress_indicators() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    // Create mesh with multiple time steps and variables for meaningful progress
    create_hex8_mesh(&input).expect("Failed to create HEX8 mesh");

    // Run with verbose mode - progress indicators should show without errors
    let result = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--copy-mirror-merge",
            "x",
            "--verbose",
        ])
        .output()
        .expect("Failed to run rexonator");

    assert!(
        result.status.success(),
        "CMM with verbose mode should succeed.\nstderr: {}",
        String::from_utf8_lossy(&result.stderr)
    );

    // Verify the output was created correctly
    let params = read_params(&output).expect("Failed to read output params");
    assert!(params.num_nodes > 0, "Output should have nodes");
    assert!(params.num_elems > 0, "Output should have elements");
}

/// Test verbose mode with element variables (more progress operations)
#[test]
#[serial]
fn test_verbose_progress_with_elem_vars() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    // Create mesh with element variables
    create_hex8_with_elem_vars(&input).expect("Failed to create HEX8 mesh with elem vars");

    let result = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--copy-mirror-merge",
            "x",
            "--verbose",
        ])
        .output()
        .expect("Failed to run rexonator");

    assert!(
        result.status.success(),
        "CMM with verbose mode and element vars should succeed.\nstderr: {}",
        String::from_utf8_lossy(&result.stderr)
    );

    // Verify the output was created correctly
    let params = read_params(&output).expect("Failed to read output params");
    assert_eq!(
        params.num_elems, 8,
        "Should have doubled elements (4 original + 4 mirrored)"
    );
}
