//! Integration tests for basic rexonator transformations
//!
//! These tests verify the CLI behavior for:
//! - Translation (--translate)
//! - Rotation (--rotate)
//! - Scaling (--scale-len)
//! - Mirroring (--mirror)
//! - Field scaling (--scale-field)
//! - Time normalization (--zero-time)
//! - Operation ordering

mod fixtures;
use fixtures::*;

use serial_test::serial;
use std::process::Command;

const TOLERANCE: f64 = 1e-10;

fn rexonator_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rexonator"))
}

// ========================================================================
// Dry Run Tests
// ========================================================================

#[test]
#[serial]
fn test_dry_run_mode() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_simple_cube(&input).expect("Failed to create test mesh");

    let result = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--translate",
            "1,0,0",
            "--dry-run",
        ])
        .output()
        .expect("Failed to run rexonator in dry-run mode");

    assert!(result.status.success());

    let stdout = String::from_utf8_lossy(&result.stdout);

    assert!(stdout.contains("Dry-Run Mode Enabled:"));
    assert!(stdout.contains(&format!("Input:  {}", input.display())));
    assert!(stdout.contains(&format!("Output: {}", output.display())));
    assert!(stdout.contains("Operations to apply: 1"));
    assert!(stdout.contains("  1: Translate([1.0, 0.0, 0.0])"));
    assert!(stdout.contains("Input Mesh Statistics:"));
    assert!(stdout.contains("Nodes:"));
    assert!(stdout.contains("Elements:"));
    assert!(stdout.contains("Dimensions:"));
    assert!(stdout.contains("Time Steps:"));
    assert!(stdout.contains("No output file will be written in dry-run mode."));

    // Verify that the output file was NOT created
    assert!(
        !output.exists(),
        "Output file should not exist in dry-run mode"
    );
}

// ========================================================================
// Translation Tests
// ========================================================================

#[test]
#[serial]
fn test_translate_positive_x() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_simple_cube(&input).expect("Failed to create test mesh");

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--translate",
            "10,0,0",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success(), "rexonator failed");

    let (x_bounds, y_bounds, z_bounds) = read_coord_bounds(&output).expect("Failed to read output");

    // Original: [0,1] x [0,1] x [0,1]
    // After translate by (10,0,0): [10,11] x [0,1] x [0,1]
    assert!((x_bounds[0] - 10.0).abs() < TOLERANCE, "x_min should be 10");
    assert!((x_bounds[1] - 11.0).abs() < TOLERANCE, "x_max should be 11");
    assert!((y_bounds[0] - 0.0).abs() < TOLERANCE, "y unchanged");
    assert!((y_bounds[1] - 1.0).abs() < TOLERANCE, "y unchanged");
    assert!((z_bounds[0] - 0.0).abs() < TOLERANCE, "z unchanged");
    assert!((z_bounds[1] - 1.0).abs() < TOLERANCE, "z unchanged");
}

#[test]
#[serial]
fn test_translate_negative_offset() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_simple_cube(&input).expect("Failed to create test mesh");

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--translate",
            "-5,-3,-2",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let (x_bounds, y_bounds, z_bounds) = read_coord_bounds(&output).unwrap();

    // After translate by (-5,-3,-2): [-5,-4] x [-3,-2] x [-2,-1]
    assert!((x_bounds[0] - (-5.0)).abs() < TOLERANCE);
    assert!((x_bounds[1] - (-4.0)).abs() < TOLERANCE);
    assert!((y_bounds[0] - (-3.0)).abs() < TOLERANCE);
    assert!((y_bounds[1] - (-2.0)).abs() < TOLERANCE);
    assert!((z_bounds[0] - (-2.0)).abs() < TOLERANCE);
    assert!((z_bounds[1] - (-1.0)).abs() < TOLERANCE);
}

#[test]
#[serial]
fn test_translate_3d_combined() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_simple_cube(&input).expect("Failed to create test mesh");

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--translate",
            "1.5,2.5,3.5",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let (x_bounds, y_bounds, z_bounds) = read_coord_bounds(&output).unwrap();

    assert!((x_bounds[0] - 1.5).abs() < TOLERANCE);
    assert!((x_bounds[1] - 2.5).abs() < TOLERANCE);
    assert!((y_bounds[0] - 2.5).abs() < TOLERANCE);
    assert!((y_bounds[1] - 3.5).abs() < TOLERANCE);
    assert!((z_bounds[0] - 3.5).abs() < TOLERANCE);
    assert!((z_bounds[1] - 4.5).abs() < TOLERANCE);
}

#[test]
#[serial]
fn test_translate_2d_mesh() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_quad4_mesh(&input).expect("Failed to create test mesh");

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--translate",
            "5,5,0",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let (x_bounds, y_bounds, _) = read_coord_bounds(&output).unwrap();

    // Original: [0,1] x [0,1]
    // After: [5,6] x [5,6]
    assert!((x_bounds[0] - 5.0).abs() < TOLERANCE);
    assert!((x_bounds[1] - 6.0).abs() < TOLERANCE);
    assert!((y_bounds[0] - 5.0).abs() < TOLERANCE);
    assert!((y_bounds[1] - 6.0).abs() < TOLERANCE);
}

// ========================================================================
// Scaling Tests
// ========================================================================

#[test]
#[serial]
fn test_scale_up() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_simple_cube(&input).expect("Failed to create test mesh");

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--scale-len",
            "2",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let (x_bounds, y_bounds, z_bounds) = read_coord_bounds(&output).unwrap();

    // Original: [0,1] -> scaled by 2 -> [0,2]
    assert!((x_bounds[0] - 0.0).abs() < TOLERANCE);
    assert!((x_bounds[1] - 2.0).abs() < TOLERANCE);
    assert!((y_bounds[0] - 0.0).abs() < TOLERANCE);
    assert!((y_bounds[1] - 2.0).abs() < TOLERANCE);
    assert!((z_bounds[0] - 0.0).abs() < TOLERANCE);
    assert!((z_bounds[1] - 2.0).abs() < TOLERANCE);
}

#[test]
#[serial]
fn test_scale_down() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_simple_cube(&input).expect("Failed to create test mesh");

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--scale-len",
            "0.5",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let (x_bounds, y_bounds, z_bounds) = read_coord_bounds(&output).unwrap();

    // Original: [0,1] -> scaled by 0.5 -> [0,0.5]
    assert!((x_bounds[0] - 0.0).abs() < TOLERANCE);
    assert!((x_bounds[1] - 0.5).abs() < TOLERANCE);
    assert!((y_bounds[0] - 0.0).abs() < TOLERANCE);
    assert!((y_bounds[1] - 0.5).abs() < TOLERANCE);
    assert!((z_bounds[0] - 0.0).abs() < TOLERANCE);
    assert!((z_bounds[1] - 0.5).abs() < TOLERANCE);
}

#[test]
#[serial]
fn test_scale_unit_conversion() {
    // Test converting mm to m (scale by 0.001)
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_simple_cube(&input).expect("Failed to create test mesh");

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--scale-len",
            "0.001",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let (x_bounds, _, _) = read_coord_bounds(&output).unwrap();

    assert!((x_bounds[0] - 0.0).abs() < TOLERANCE);
    assert!((x_bounds[1] - 0.001).abs() < TOLERANCE);
}

// ========================================================================
// Mirror Tests
// ========================================================================

#[test]
#[serial]
fn test_mirror_x() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_simple_cube(&input).expect("Failed to create test mesh");

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--mirror",
            "x",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let (x_bounds, y_bounds, z_bounds) = read_coord_bounds(&output).unwrap();

    // Original: [0,1] -> mirror about YZ plane -> [-1,0]
    assert!((x_bounds[0] - (-1.0)).abs() < TOLERANCE);
    assert!((x_bounds[1] - 0.0).abs() < TOLERANCE);
    // Y and Z unchanged
    assert!((y_bounds[0] - 0.0).abs() < TOLERANCE);
    assert!((y_bounds[1] - 1.0).abs() < TOLERANCE);
    assert!((z_bounds[0] - 0.0).abs() < TOLERANCE);
    assert!((z_bounds[1] - 1.0).abs() < TOLERANCE);
}

#[test]
#[serial]
fn test_mirror_y() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_simple_cube(&input).expect("Failed to create test mesh");

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--mirror",
            "y",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let (x_bounds, y_bounds, z_bounds) = read_coord_bounds(&output).unwrap();

    assert!((x_bounds[0] - 0.0).abs() < TOLERANCE);
    assert!((x_bounds[1] - 1.0).abs() < TOLERANCE);
    assert!((y_bounds[0] - (-1.0)).abs() < TOLERANCE);
    assert!((y_bounds[1] - 0.0).abs() < TOLERANCE);
    assert!((z_bounds[0] - 0.0).abs() < TOLERANCE);
    assert!((z_bounds[1] - 1.0).abs() < TOLERANCE);
}

#[test]
#[serial]
fn test_mirror_z() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_simple_cube(&input).expect("Failed to create test mesh");

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--mirror",
            "z",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let (x_bounds, y_bounds, z_bounds) = read_coord_bounds(&output).unwrap();

    assert!((x_bounds[0] - 0.0).abs() < TOLERANCE);
    assert!((x_bounds[1] - 1.0).abs() < TOLERANCE);
    assert!((y_bounds[0] - 0.0).abs() < TOLERANCE);
    assert!((y_bounds[1] - 1.0).abs() < TOLERANCE);
    assert!((z_bounds[0] - (-1.0)).abs() < TOLERANCE);
    assert!((z_bounds[1] - 0.0).abs() < TOLERANCE);
}

#[test]
#[serial]
fn test_mirror_uppercase() {
    // Test that uppercase axis works
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_simple_cube(&input).expect("Failed to create test mesh");

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--mirror",
            "X",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let (x_bounds, _, _) = read_coord_bounds(&output).unwrap();
    assert!((x_bounds[0] - (-1.0)).abs() < TOLERANCE);
    assert!((x_bounds[1] - 0.0).abs() < TOLERANCE);
}

// ========================================================================
// Rotation Tests
// ========================================================================

#[test]
#[serial]
fn test_rotate_z_90() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_simple_cube(&input).expect("Failed to create test mesh");

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--rotate",
            "Z,90",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let (x_bounds, y_bounds, z_bounds) = read_coord_bounds(&output).unwrap();

    // Rotation of 90 degrees about Z:
    // (1,0,0) -> (0,1,0)
    // (0,1,0) -> (-1,0,0)
    // Original: [0,1] x [0,1] x [0,1]
    // After 90 deg Z: [-1,0] x [0,1] x [0,1]
    assert!((x_bounds[0] - (-1.0)).abs() < TOLERANCE);
    assert!((x_bounds[1] - 0.0).abs() < TOLERANCE);
    assert!((y_bounds[0] - 0.0).abs() < TOLERANCE);
    assert!((y_bounds[1] - 1.0).abs() < TOLERANCE);
    assert!((z_bounds[0] - 0.0).abs() < TOLERANCE);
    assert!((z_bounds[1] - 1.0).abs() < TOLERANCE);
}

#[test]
#[serial]
fn test_rotate_x_90() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_simple_cube(&input).expect("Failed to create test mesh");

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--rotate",
            "X,90",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let (x_bounds, y_bounds, z_bounds) = read_coord_bounds(&output).unwrap();

    // Rotation of 90 degrees about X:
    // (0,1,0) -> (0,0,1)
    // (0,0,1) -> (0,-1,0)
    // Original: [0,1] x [0,1] x [0,1]
    // After 90 deg X: [0,1] x [-1,0] x [0,1]
    assert!((x_bounds[0] - 0.0).abs() < TOLERANCE);
    assert!((x_bounds[1] - 1.0).abs() < TOLERANCE);
    assert!((y_bounds[0] - (-1.0)).abs() < TOLERANCE);
    assert!((y_bounds[1] - 0.0).abs() < TOLERANCE);
    assert!((z_bounds[0] - 0.0).abs() < TOLERANCE);
    assert!((z_bounds[1] - 1.0).abs() < TOLERANCE);
}

#[test]
#[serial]
fn test_rotate_extrinsic_xyz() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_simple_cube(&input).expect("Failed to create test mesh");

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--rotate",
            "XYZ,90,0,0",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    // XYZ with 90,0,0 should be same as X,90
    let (x_bounds, y_bounds, _) = read_coord_bounds(&output).unwrap();

    assert!((x_bounds[0] - 0.0).abs() < TOLERANCE);
    assert!((x_bounds[1] - 1.0).abs() < TOLERANCE);
    assert!((y_bounds[0] - (-1.0)).abs() < TOLERANCE);
    assert!((y_bounds[1] - 0.0).abs() < TOLERANCE);
}

#[test]
#[serial]
fn test_rotate_intrinsic_lowercase() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_simple_cube(&input).expect("Failed to create test mesh");

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--rotate",
            "z,90",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    // Single axis rotation is the same for intrinsic/extrinsic
    let (x_bounds, y_bounds, _) = read_coord_bounds(&output).unwrap();

    assert!((x_bounds[0] - (-1.0)).abs() < TOLERANCE);
    assert!((x_bounds[1] - 0.0).abs() < TOLERANCE);
    assert!((y_bounds[0] - 0.0).abs() < TOLERANCE);
    assert!((y_bounds[1] - 1.0).abs() < TOLERANCE);
}

// ========================================================================
// Scale Field Tests
// ========================================================================

#[test]
#[serial]
fn test_scale_field_nodal() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_hex8_mesh(&input).expect("Failed to create test mesh");

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--scale-field",
            "temperature,2.0",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    // Read original and output temperature values
    let orig_temp = read_nodal_var(&input, 0, 0).expect("Failed to read original temp");
    let new_temp = read_nodal_var(&output, 0, 0).expect("Failed to read new temp");

    assert_eq!(orig_temp.len(), new_temp.len());
    for (orig, new) in orig_temp.iter().zip(new_temp.iter()) {
        assert!(
            (new - orig * 2.0).abs() < TOLERANCE,
            "Temperature should be scaled by 2: {} * 2 != {}",
            orig,
            new
        );
    }
}

#[test]
#[serial]
fn test_scale_field_multiple() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_hex8_mesh(&input).expect("Failed to create test mesh");

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--scale-field",
            "temperature,1.5",
            "--scale-field",
            "velocity_x,0.5",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let orig_temp = read_nodal_var(&input, 0, 0).unwrap();
    let new_temp = read_nodal_var(&output, 0, 0).unwrap();
    for (orig, new) in orig_temp.iter().zip(new_temp.iter()) {
        assert!((new - orig * 1.5).abs() < TOLERANCE);
    }

    let orig_vx = read_nodal_var(&input, 1, 0).unwrap();
    let new_vx = read_nodal_var(&output, 1, 0).unwrap();
    for (orig, new) in orig_vx.iter().zip(new_vx.iter()) {
        assert!((new - orig * 0.5).abs() < TOLERANCE);
    }
}

#[test]
#[serial]
fn test_scale_field_scientific_notation() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_hex8_mesh(&input).expect("Failed to create test mesh");

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--scale-field",
            "temperature,1e3",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let orig_temp = read_nodal_var(&input, 0, 0).unwrap();
    let new_temp = read_nodal_var(&output, 0, 0).unwrap();
    for (orig, new) in orig_temp.iter().zip(new_temp.iter()) {
        assert!((new - orig * 1000.0).abs() < TOLERANCE);
    }
}

// ========================================================================
// Zero Time Tests
// ========================================================================

#[test]
#[serial]
fn test_zero_time_normalization() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_mesh_with_time_steps(&input).expect("Failed to create test mesh");

    // Verify original times start at 10.0
    let orig_times = read_times(&input).expect("Failed to read original times");
    assert!((orig_times[0] - 10.0).abs() < TOLERANCE);

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--zero-time",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let new_times = read_times(&output).expect("Failed to read new times");

    // Times should now start at 0
    assert!(
        (new_times[0] - 0.0).abs() < TOLERANCE,
        "First time should be 0"
    );
    // Subsequent times should be shifted
    for (i, time) in new_times.iter().enumerate() {
        let expected = i as f64 * 0.1; // 0.0, 0.1, 0.2, ...
        assert!(
            (time - expected).abs() < TOLERANCE,
            "Time {} should be {}, got {}",
            i,
            expected,
            time
        );
    }
}

#[test]
#[serial]
fn test_zero_time_with_other_ops() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_mesh_with_time_steps(&input).expect("Failed to create test mesh");

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--translate",
            "1,0,0",
            "--zero-time",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let new_times = read_times(&output).unwrap();
    assert!((new_times[0] - 0.0).abs() < TOLERANCE);

    let (x_bounds, _, _) = read_coord_bounds(&output).unwrap();
    assert!((x_bounds[0] - 1.0).abs() < TOLERANCE);
}

// ========================================================================
// Operation Ordering Tests
// ========================================================================

#[test]
#[serial]
fn test_order_translate_then_scale() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_simple_cube(&input).expect("Failed to create test mesh");

    // First translate to [10,11], then scale by 2 -> [20,22]
    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--translate",
            "10,0,0",
            "--scale-len",
            "2",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let (x_bounds, _, _) = read_coord_bounds(&output).unwrap();

    assert!((x_bounds[0] - 20.0).abs() < TOLERANCE);
    assert!((x_bounds[1] - 22.0).abs() < TOLERANCE);
}

#[test]
#[serial]
fn test_order_scale_then_translate() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_simple_cube(&input).expect("Failed to create test mesh");

    // First scale by 2 -> [0,2], then translate by 10 -> [10,12]
    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--scale-len",
            "2",
            "--translate",
            "10,0,0",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let (x_bounds, _, _) = read_coord_bounds(&output).unwrap();

    assert!((x_bounds[0] - 10.0).abs() < TOLERANCE);
    assert!((x_bounds[1] - 12.0).abs() < TOLERANCE);
}

#[test]
#[serial]
fn test_order_translate_rotate_translate() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_simple_cube(&input).expect("Failed to create test mesh");

    // Translate to (5,0,0), rotate 90 about Z, translate again
    // Original: [0,1] x [0,1]
    // After T(5,0,0): [5,6] x [0,1]
    // After R(Z,90): [-1,0] x [5,6]
    // After T(10,0,0): [9,10] x [5,6]
    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--translate",
            "5,0,0",
            "--rotate",
            "Z,90",
            "--translate",
            "10,0,0",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let (x_bounds, y_bounds, _) = read_coord_bounds(&output).unwrap();

    assert!((x_bounds[0] - 9.0).abs() < TOLERANCE);
    assert!((x_bounds[1] - 10.0).abs() < TOLERANCE);
    assert!((y_bounds[0] - 5.0).abs() < TOLERANCE);
    assert!((y_bounds[1] - 6.0).abs() < TOLERANCE);
}

#[test]
#[serial]
fn test_multiple_rotations() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_simple_cube(&input).expect("Failed to create test mesh");

    // Three 90-degree rotations about Z should result in 270 degrees = -90 degrees
    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--rotate",
            "Z,90",
            "--rotate",
            "Z,90",
            "--rotate",
            "Z,90",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let (x_bounds, y_bounds, _) = read_coord_bounds(&output).unwrap();

    // 270 degrees about Z: (1,0) -> (0,-1)
    // Original: [0,1] x [0,1] -> [0,1] x [-1,0]
    assert!((x_bounds[0] - 0.0).abs() < TOLERANCE);
    assert!((x_bounds[1] - 1.0).abs() < TOLERANCE);
    assert!((y_bounds[0] - (-1.0)).abs() < TOLERANCE);
    assert!((y_bounds[1] - 0.0).abs() < TOLERANCE);
}

// ========================================================================
// Verbose Mode Test
// ========================================================================

#[test]
#[serial]
fn test_verbose_output() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_simple_cube(&input).expect("Failed to create test mesh");

    let result = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--translate",
            "1,0,0",
            "-v",
        ])
        .output()
        .expect("Failed to run rexonator");

    assert!(result.status.success());

    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(stdout.contains("Input:"), "Should print input path");
    assert!(stdout.contains("Output:"), "Should print output path");
    assert!(stdout.contains("Translating"), "Should describe operation");
}

// ========================================================================
// Element Type Tests
// ========================================================================

#[test]
#[serial]
fn test_transform_tri3_mesh() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_tri3_mesh(&input).expect("Failed to create TRI3 mesh");

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--translate",
            "5,5,0",
            "--scale-len",
            "2",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let (x_bounds, y_bounds, _) = read_coord_bounds(&output).unwrap();

    // Original: [0,1] x [0,1]
    // After translate: [5,6] x [5,6]
    // After scale: [10,12] x [10,12]
    assert!((x_bounds[0] - 10.0).abs() < TOLERANCE);
    assert!((x_bounds[1] - 12.0).abs() < TOLERANCE);
    assert!((y_bounds[0] - 10.0).abs() < TOLERANCE);
    assert!((y_bounds[1] - 12.0).abs() < TOLERANCE);
}

#[test]
#[serial]
fn test_transform_tet4_mesh() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_tet4_mesh(&input).expect("Failed to create TET4 mesh");

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--mirror",
            "x",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let (x_bounds, _, _) = read_coord_bounds(&output).unwrap();

    // Original: [0,1] x ... -> after mirror: [-1,0]
    assert!((x_bounds[0] - (-1.0)).abs() < TOLERANCE);
    assert!((x_bounds[1] - 0.0).abs() < TOLERANCE);
}

#[test]
#[serial]
fn test_transform_wedge6_mesh() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_wedge6_mesh(&input).expect("Failed to create WEDGE6 mesh");

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--rotate",
            "Z,180",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    // Just verify the operation completed successfully
    let params = read_params(&output).expect("Failed to read params");
    assert_eq!(params.num_elems, 1);
}

#[test]
#[serial]
fn test_transform_pyramid5_mesh() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_pyramid5_mesh(&input).expect("Failed to create PYRAMID5 mesh");

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--scale-len",
            "3",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let (_, _, z_bounds) = read_coord_bounds(&output).unwrap();

    // Original apex at (0.5, 0.5, 1) -> (1.5, 1.5, 3)
    assert!((z_bounds[1] - 3.0).abs() < TOLERANCE);
}

// ========================================================================
// Error Handling Tests
// ========================================================================

#[test]
#[serial]
fn test_error_missing_input() {
    let ctx = TestContext::new();
    let output = ctx.path("output.exo");

    let status = rexonator_cmd()
        .args(["nonexistent.exo", output.to_str().unwrap()])
        .status()
        .expect("Failed to run rexonator");

    assert!(!status.success(), "Should fail with missing input");
}

#[test]
#[serial]
fn test_error_invalid_translate() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_simple_cube(&input).expect("Failed to create test mesh");

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--translate",
            "1,2", // Missing z value
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(!status.success(), "Should fail with invalid translate");
}

#[test]
#[serial]
fn test_error_invalid_axis() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_simple_cube(&input).expect("Failed to create test mesh");

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--mirror",
            "w", // Invalid axis
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(!status.success(), "Should fail with invalid axis");
}

#[test]
#[serial]
fn test_error_invalid_rotate() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_simple_cube(&input).expect("Failed to create test mesh");

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--rotate",
            "XYZ,90", // Wrong number of angles
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(!status.success(), "Should fail with wrong number of angles");
}

// ========================================================================
// Preserves Mesh Data Tests
// ========================================================================

#[test]
#[serial]
fn test_preserves_node_count() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_hex8_mesh(&input).expect("Failed to create test mesh");

    let orig_params = read_params(&input).unwrap();

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--translate",
            "1,2,3",
            "--rotate",
            "Z,45",
            "--scale-len",
            "2",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let new_params = read_params(&output).unwrap();

    assert_eq!(orig_params.num_nodes, new_params.num_nodes);
    assert_eq!(orig_params.num_elems, new_params.num_elems);
    assert_eq!(orig_params.num_dim, new_params.num_dim);
}

#[test]
#[serial]
fn test_preserves_time_steps() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_hex8_mesh(&input).expect("Failed to create test mesh");

    let orig_times = read_times(&input).unwrap();

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--translate",
            "1,0,0",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let new_times = read_times(&output).unwrap();

    assert_eq!(orig_times.len(), new_times.len());
    for (orig, new) in orig_times.iter().zip(new_times.iter()) {
        assert!((orig - new).abs() < TOLERANCE);
    }
}

#[test]
#[serial]
fn test_preserves_variable_names() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_hex8_mesh(&input).expect("Failed to create test mesh");

    let orig_names = read_nodal_var_names(&input).unwrap();

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--scale-len",
            "2",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let new_names = read_nodal_var_names(&output).unwrap();

    assert_eq!(orig_names, new_names);
}

// ========================================================================
// In-Place Mode Tests
// ========================================================================

#[test]
#[serial]
fn test_in_place_translate() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");

    create_simple_cube(&input).expect("Failed to create test mesh");

    // Record original bounds
    let (orig_x, _, _) = read_coord_bounds(&input).expect("Failed to read original bounds");

    let status = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            "--in-place",
            "--translate",
            "10,0,0",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    // Read bounds from the modified input file
    let (x_bounds, y_bounds, z_bounds) = read_coord_bounds(&input).expect("Failed to read output");

    // Original: [0,1] x [0,1] x [0,1]
    // After translate by (10,0,0): [10,11] x [0,1] x [0,1]
    assert!((x_bounds[0] - 10.0).abs() < TOLERANCE, "x_min should be 10");
    assert!((x_bounds[1] - 11.0).abs() < TOLERANCE, "x_max should be 11");
    assert!((y_bounds[0] - 0.0).abs() < TOLERANCE, "y unchanged");
    assert!((y_bounds[1] - 1.0).abs() < TOLERANCE, "y unchanged");
    assert!((z_bounds[0] - 0.0).abs() < TOLERANCE, "z unchanged");
    assert!((z_bounds[1] - 1.0).abs() < TOLERANCE, "z unchanged");
}

#[test]
#[serial]
fn test_in_place_with_short_flag() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");

    create_simple_cube(&input).expect("Failed to create test mesh");

    let status = rexonator_cmd()
        .args([input.to_str().unwrap(), "-i", "--scale-len", "2"])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let (x_bounds, _, _) = read_coord_bounds(&input).unwrap();

    // Original: [0,1] -> scaled by 2 -> [0,2]
    assert!((x_bounds[0] - 0.0).abs() < TOLERANCE);
    assert!((x_bounds[1] - 2.0).abs() < TOLERANCE);
}

#[test]
#[serial]
fn test_in_place_with_explicit_output() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");
    let output = ctx.path("output.exo");

    create_simple_cube(&input).expect("Failed to create test mesh");

    // Copy input to output first (to test in-place on output)
    std::fs::copy(&input, &output).expect("Failed to copy file");

    let status = rexonator_cmd()
        .args([
            output.to_str().unwrap(),
            output.to_str().unwrap(),
            "--in-place",
            "--translate",
            "5,0,0",
        ])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let (x_bounds, _, _) = read_coord_bounds(&output).unwrap();

    assert!((x_bounds[0] - 5.0).abs() < TOLERANCE);
    assert!((x_bounds[1] - 6.0).abs() < TOLERANCE);
}

#[test]
#[serial]
fn test_in_place_same_result_as_copy() {
    let ctx = TestContext::new();
    let input1 = ctx.path("input1.exo");
    let input2 = ctx.path("input2.exo");
    let output = ctx.path("output.exo");

    // Create two identical meshes
    create_simple_cube(&input1).expect("Failed to create test mesh 1");
    create_simple_cube(&input2).expect("Failed to create test mesh 2");

    // Apply transformation with copy mode
    let status1 = rexonator_cmd()
        .args([
            input1.to_str().unwrap(),
            output.to_str().unwrap(),
            "--translate",
            "10,20,30",
            "--rotate",
            "Z,45",
        ])
        .status()
        .expect("Failed to run rexonator (copy mode)");
    assert!(status1.success());

    // Apply same transformation with in-place mode
    let status2 = rexonator_cmd()
        .args([
            input2.to_str().unwrap(),
            "--in-place",
            "--translate",
            "10,20,30",
            "--rotate",
            "Z,45",
        ])
        .status()
        .expect("Failed to run rexonator (in-place mode)");
    assert!(status2.success());

    // Compare results
    let (x1, y1, z1) = read_coord_bounds(&output).unwrap();
    let (x2, y2, z2) = read_coord_bounds(&input2).unwrap();

    assert!((x1[0] - x2[0]).abs() < TOLERANCE, "x_min mismatch");
    assert!((x1[1] - x2[1]).abs() < TOLERANCE, "x_max mismatch");
    assert!((y1[0] - y2[0]).abs() < TOLERANCE, "y_min mismatch");
    assert!((y1[1] - y2[1]).abs() < TOLERANCE, "y_max mismatch");
    assert!((z1[0] - z2[0]).abs() < TOLERANCE, "z_min mismatch");
    assert!((z1[1] - z2[1]).abs() < TOLERANCE, "z_max mismatch");
}

#[test]
#[serial]
fn test_in_place_verbose_output() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");

    create_simple_cube(&input).expect("Failed to create test mesh");

    let result = rexonator_cmd()
        .args([input.to_str().unwrap(), "-i", "--translate", "1,0,0", "-v"])
        .output()
        .expect("Failed to run rexonator");

    assert!(result.status.success());

    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(
        stdout.contains("In-place") || stdout.contains("in-place"),
        "Should mention in-place mode in verbose output"
    );
    assert!(
        stdout.contains("no copy") || stdout.contains("no file copy"),
        "Should mention no copy needed"
    );
}

#[test]
#[serial]
fn test_in_place_with_zero_time() {
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");

    create_mesh_with_time_steps(&input).expect("Failed to create test mesh");

    let orig_times = read_times(&input).expect("Failed to read original times");
    assert!((orig_times[0] - 10.0).abs() < TOLERANCE); // Verify starting time

    let status = rexonator_cmd()
        .args([input.to_str().unwrap(), "-i", "--zero-time"])
        .status()
        .expect("Failed to run rexonator");

    assert!(status.success());

    let new_times = read_times(&input).expect("Failed to read new times");

    // Times should now start at 0
    assert!(
        (new_times[0] - 0.0).abs() < TOLERANCE,
        "First time should be 0"
    );
}

#[test]
#[serial]
fn test_auto_in_place_same_paths() {
    // Test that when input == output, in-place mode is automatically enabled
    let ctx = TestContext::new();
    let input = ctx.path("input.exo");

    create_simple_cube(&input).expect("Failed to create test mesh");

    // Use the same path for input and output (without --in-place flag)
    let result = rexonator_cmd()
        .args([
            input.to_str().unwrap(),
            input.to_str().unwrap(), // Same as input
            "--translate",
            "10,0,0",
            "-v",
        ])
        .output()
        .expect("Failed to run rexonator");

    assert!(result.status.success());

    let stdout = String::from_utf8_lossy(&result.stdout);
    // Should automatically detect same file and use in-place mode
    assert!(
        stdout.contains("In-place") || stdout.contains("in-place"),
        "Should automatically enable in-place mode when paths are same"
    );

    let (x_bounds, _, _) = read_coord_bounds(&input).unwrap();
    assert!((x_bounds[0] - 10.0).abs() < TOLERANCE);
}
