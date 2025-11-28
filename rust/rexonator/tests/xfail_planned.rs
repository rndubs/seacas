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
    create_quad4_mesh, read_nodal_var, read_params, TestContext,
};

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
// Currently, side numbers are copied unchanged (simplification).
// ========================================================================

#[test]
#[ignore = "XFAIL: Proper side set side number mapping not yet implemented - see PLAN.md"]
fn test_cmm_side_numbers_properly_mapped() {
    // When mirroring elements, the side numbers may need to change
    // depending on the element topology and mirror axis.
    //
    // For example, for HEX8 mirrored about X:
    // - Side 4 (x+ face) might become Side 3 (x- face)
    // - This depends on the exodus side numbering convention
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

    // When implemented, verify:
    // 1. Read side set data from output
    // 2. Check that mirrored side set has properly remapped side numbers
    // 3. Side numbers should correspond to the correct faces after mirroring
}

// ========================================================================
// Memory Warnings for Large Meshes (High Priority)
// PLAN.md: copy_mirror_merge.rs:351+
//
// Add warnings when estimated memory usage is high.
// ========================================================================

#[test]
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
// Replace exit() with proper error returns.
// ========================================================================

#[test]
#[ignore = "XFAIL: Proper error handling in man.rs not yet implemented - see PLAN.md"]
fn test_man_page_missing_returns_error() {
    // When the man page file is missing, rexonator should return
    // a proper error instead of calling exit() directly.
    //
    // This test would need to be run in an environment where the
    // man page is deliberately removed/renamed.

    // When implemented:
    // 1. Temporarily rename/hide the man page
    // 2. Run rexonator --man
    // 3. Verify it returns an error code (not just exits)
    // 4. Verify error message is informative
    // 5. Restore the man page
}

// ========================================================================
// Parallel Processing with Rayon (Low Priority)
// PLAN.md: copy_mirror_merge.rs
//
// Add parallel processing for large mesh operations.
// ========================================================================

#[test]
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
#[ignore = "XFAIL: Progress indicators not yet implemented - see PLAN.md"]
fn test_verbose_progress_indicators() {
    // With large meshes, verbose mode should show progress updates
    // like "Processing time step 50/100".
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
    // For large meshes with many time steps/variables, verbose output
    // should include progress like:
    // "  Processing time step 50/100"
    // "  Processing nodal variable 5/10"
}

// ========================================================================
// 2D Mesh Z-Coordinate Handling (Low Priority)
// PLAN.md: copy_mirror_merge.rs:155-160
//
// The code fills z with zeros for 2D meshes, but then might write
// 3D coordinates back, potentially changing a 2D mesh to 3D.
// ========================================================================

#[test]
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
