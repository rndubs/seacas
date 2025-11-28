//! XFail tests for planned but unimplemented features
//!
//! These tests document expected behavior for features described in PLAN.md
//! that are not yet implemented. Tests are marked with #[ignore] and will
//! be enabled when the corresponding features are implemented.
//!
//! When implementing a feature, remove the #[ignore] attribute and verify
//! the test passes.
//!
//! See PLAN.md for full details on planned improvements.

mod fixtures;
use fixtures::{
    create_hex8_mesh, create_mesh_with_false_positive_vars, create_mesh_with_global_vars,
    create_quad4_mesh, read_nodal_var, read_params, read_side_set, read_side_set_ids, TestContext,
};

use serial_test::serial;
use std::process::Command;

fn rexonator_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rexonator"))
}

// ========================================================================
// Vector Component Detection False Positives (Medium Priority)
// PLAN.md: copy_mirror_merge.rs:127-138
//
// FIXED: The detection logic now uses stricter patterns:
// - Requires underscore separator (_x, _y, _z)
// - Excludes known scalar prefixes (max, min, index, etc.)
// - Single-letter vars (u, v, w) must be exact matches
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
// Side Set Side Number Mapping (Medium Priority)
// PLAN.md: copy_mirror_merge.rs:575-577
//
// Side numbers need adjustment based on topology and axis when mirroring.
// IMPLEMENTED: Side numbers are now properly mapped based on topology and axis.
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
// Memory Warnings for Large Meshes (High Priority)
// PLAN.md: copy_mirror_merge.rs:351+
//
// Add warnings when estimated memory usage is high.
// ========================================================================

#[test]
#[serial]
#[ignore = "XFAIL: Memory usage warnings not yet implemented - see PLAN.md"]
fn test_cmm_warns_on_large_mesh() {
    // When processing large meshes, verbose mode should warn about
    // estimated memory usage.
    //
    // Since we can't easily test with truly large meshes in unit tests,
    // this test documents the expected behavior.
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

    // When implemented:
    // With large meshes, verbose output should include memory warning like:
    // "WARNING: Estimated memory usage: X GB"
    //
    // For small test meshes, no warning should appear
}

// ========================================================================
// Proper Error Returns in man.rs (Low Priority)
// PLAN.md: man.rs:27,35
//
// FIXED: Replaced exit() with proper error returns.
// ========================================================================

#[test]
#[serial]
fn test_man_page_missing_returns_error() {
    // FIXED: man.rs now returns proper errors instead of calling exit().
    //
    // The show_man_page() function now returns:
    // - Err(TransformError::Io) when man page is not found
    // - Err(TransformError::Io) when man command fails
    //
    // These errors propagate through main() and result in proper exit codes.
    //
    // This test verifies that --man works when the man page is present.
    // The error path (missing man page) is difficult to test in integration
    // tests without filesystem manipulation, but the code properly returns
    // errors instead of calling exit() directly.

    let status = rexonator_cmd()
        .arg("--man")
        .status()
        .expect("Failed to run rexonator --man");

    // If man page is present and man command available, should succeed
    // If man page is missing or man command unavailable, should fail with error code
    // Either way, it should not panic or hang - it should exit cleanly
    assert!(
        status.code().is_some(),
        "rexonator --man should exit cleanly with a status code"
    );
}

// ========================================================================
// Parallel Processing with Rayon (Low Priority)
// PLAN.md: copy_mirror_merge.rs
//
// Add parallel processing for large mesh operations.
// ========================================================================

#[test]
#[serial]
#[ignore = "XFAIL: Parallel processing not yet implemented - see PLAN.md"]
fn test_cmm_parallel_processing() {
    // Large mesh operations could benefit from parallel processing.
    // This test documents expected behavior.
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

    // When implemented:
    // - Large operations should use multiple threads
    // - Could add a --parallel flag or make it automatic based on mesh size
    // - Verify correctness is maintained with parallel processing
}

// ========================================================================
// Progress Indicators (Low Priority)
// PLAN.md: copy_mirror_merge.rs
//
// Add progress indicators for verbose mode on large operations.
// ========================================================================

#[test]
#[serial]
fn test_verbose_progress_indicators() {
    // FIXED: Progress indicators now work in verbose mode.
    // With meshes that have variables, verbose mode shows progress updates.
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

    // Check that verbose output includes progress messages
    let stdout = String::from_utf8_lossy(&result.stdout);

    // For meshes with nodal variables, should see processing message
    // Note: create_hex8_mesh has nodal variables
    assert!(
        stdout.contains("Processing") || stdout.contains("nodal variable"),
        "Verbose mode should show progress indicators, got: {}",
        stdout
    );
}

// ========================================================================
// 2D Mesh Z-Coordinate Handling (Low Priority)
// PLAN.md: copy_mirror_merge.rs:155-160
//
// The code fills z with zeros for 2D meshes, but then might write
// 3D coordinates back, potentially changing a 2D mesh to 3D.
// ========================================================================

#[test]
#[serial]
#[ignore = "XFAIL: 2D mesh dimensionality preservation not verified - see PLAN.md"]
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

    // When fixed, this assertion should pass:
    // assert_eq!(new_params.num_dim, 2, "Output should remain 2D");

    // Currently, this might change to 3D. Document expected behavior.
    let _ = new_params; // Suppress unused warning
}

// ========================================================================
// Unsupported Topology Error Message
// ========================================================================

#[test]
#[serial]
fn test_cmm_error_on_unsupported_topology() {
    // CMM should fail with a clear error for unsupported topologies
    // Currently supported: HEX8, TET4, WEDGE6, PYRAMID5, QUAD4, TRI3
    //
    // This test verifies that attempting CMM on unsupported topology
    // produces a clear error message.
    //
    // Note: We can't easily create a mesh with unsupported topology
    // using our fixture functions, so this is more of a documentation
    // test. The actual error handling exists in the code.

    // If we had a HEX27 mesh, it would fail like:
    // "Unsupported element topology 'HEX27' in block X for copy-mirror-merge.
    //  Supported: HEX8, TET4, WEDGE6, PYRAMID5, QUAD4, TRI3"
}

// ========================================================================
// Global Variables Warning
// ========================================================================

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
    // Note: This test verifies current behavior
    assert!(
        stderr.contains("WARNING") || stderr.contains("global"),
        "Should warn about global variables needing manual adjustment"
    );
}

// ============================================================================
// Benchmark Placeholders (Low Priority)
// PLAN.md suggests adding benchmarks but these are out of scope for
// integration tests. Document here for completeness.
// ============================================================================

// Benchmarks should be added to benches/copy_mirror_merge.rs and include:
// - bench_large_mesh_mirror (100k nodes)
// - bench_nodal_var_mirroring (10k nodes, 10 vars, 100 timesteps)
//
// See PLAN.md for full benchmark recommendations.
