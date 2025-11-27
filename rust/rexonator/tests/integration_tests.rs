//! Integration tests for the rexonator CLI tool
//!
//! These tests verify the functionality of rexonator transformations on various
//! mesh types and geometries. The test fixtures create simple meshes that can
//! be visually inspected to understand the CLI behavior.
//!
//! Test fixtures are created in a temporary directory and can be examined by
//! running tests with --nocapture to see the file paths.

mod fixtures;

use std::process::Command;
use tempfile::TempDir;

/// Run rexonator with the given arguments and return (success, stdout, stderr)
fn run_rexonator(args: &[&str]) -> (bool, String, String) {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--"])
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to execute rexonator");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (output.status.success(), stdout, stderr)
}

/// Helper to assert floats are approximately equal
fn approx_eq(a: f64, b: f64, tolerance: f64) -> bool {
    (a - b).abs() < tolerance
}

// =============================================================================
// Basic Transformation Tests
// =============================================================================

mod translation_tests {
    use super::*;

    #[test]
    fn test_translate_hex8_positive_x() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("input.exo");
        let output = temp_dir.path().join("output.exo");

        fixtures::create_hex8_mesh(&input).unwrap();

        let (orig_x, orig_y, orig_z) = fixtures::read_coords(&input).unwrap();

        let (success, _, stderr) = run_rexonator(&[
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--translate",
            "10.0,0.0,0.0",
        ]);
        assert!(success, "rexonator failed: {}", stderr);

        let (new_x, new_y, new_z) = fixtures::read_coords(&output).unwrap();

        // X should be shifted by 10
        for (orig, new) in orig_x.iter().zip(new_x.iter()) {
            assert!(
                approx_eq(*new, orig + 10.0, 1e-10),
                "X not translated: {} -> {} (expected {})",
                orig,
                new,
                orig + 10.0
            );
        }
        // Y and Z should be unchanged
        for (orig, new) in orig_y.iter().zip(new_y.iter()) {
            assert!(approx_eq(*new, *orig, 1e-10), "Y changed unexpectedly");
        }
        for (orig, new) in orig_z.iter().zip(new_z.iter()) {
            assert!(approx_eq(*new, *orig, 1e-10), "Z changed unexpectedly");
        }

        println!("Test files preserved at: {:?}", temp_dir.path());
        // Keep temp dir for inspection
        std::mem::forget(temp_dir);
    }

    #[test]
    fn test_translate_all_axes() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("input.exo");
        let output = temp_dir.path().join("output.exo");

        // Use 3D mesh to avoid 2D z-coordinate issues
        fixtures::create_hex8_mesh(&input).unwrap();

        let (orig_x, orig_y, orig_z) = fixtures::read_coords(&input).unwrap();

        let (success, _, stderr) = run_rexonator(&[
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--translate",
            "1.5,2.5,3.5",
        ]);
        assert!(success, "rexonator failed: {}", stderr);

        let (new_x, new_y, new_z) = fixtures::read_coords(&output).unwrap();

        for (orig, new) in orig_x.iter().zip(new_x.iter()) {
            assert!(approx_eq(*new, orig + 1.5, 1e-10));
        }
        for (orig, new) in orig_y.iter().zip(new_y.iter()) {
            assert!(approx_eq(*new, orig + 2.5, 1e-10));
        }
        for (orig, new) in orig_z.iter().zip(new_z.iter()) {
            assert!(approx_eq(*new, orig + 3.5, 1e-10));
        }
    }

    #[test]
    fn test_translate_negative_values() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("input.exo");
        let output = temp_dir.path().join("output.exo");

        fixtures::create_hex8_mesh(&input).unwrap();
        let (orig_x, orig_y, orig_z) = fixtures::read_coords(&input).unwrap();

        // Use --translate= syntax to avoid CLI parsing issues with leading minus
        let (success, _, stderr) = run_rexonator(&[
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--translate=-5.0,-10.0,-15.0",
        ]);
        assert!(success, "rexonator failed: {}", stderr);

        let (new_x, new_y, new_z) = fixtures::read_coords(&output).unwrap();

        for (orig, new) in orig_x.iter().zip(new_x.iter()) {
            assert!(approx_eq(*new, orig - 5.0, 1e-10));
        }
        for (orig, new) in orig_y.iter().zip(new_y.iter()) {
            assert!(approx_eq(*new, orig - 10.0, 1e-10));
        }
        for (orig, new) in orig_z.iter().zip(new_z.iter()) {
            assert!(approx_eq(*new, orig - 15.0, 1e-10));
        }
    }
}

mod scaling_tests {
    use super::*;

    #[test]
    fn test_scale_uniform_hex8() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("input.exo");
        let output = temp_dir.path().join("output.exo");

        fixtures::create_hex8_mesh(&input).unwrap();
        let (orig_x, orig_y, orig_z) = fixtures::read_coords(&input).unwrap();

        let (success, _, stderr) = run_rexonator(&[
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--scale-len",
            "2.0",
        ]);
        assert!(success, "rexonator failed: {}", stderr);

        let (new_x, new_y, new_z) = fixtures::read_coords(&output).unwrap();

        for (orig, new) in orig_x.iter().zip(new_x.iter()) {
            assert!(
                approx_eq(*new, orig * 2.0, 1e-10),
                "X not scaled: {} -> {} (expected {})",
                orig,
                new,
                orig * 2.0
            );
        }
        for (orig, new) in orig_y.iter().zip(new_y.iter()) {
            assert!(approx_eq(*new, orig * 2.0, 1e-10));
        }
        for (orig, new) in orig_z.iter().zip(new_z.iter()) {
            assert!(approx_eq(*new, orig * 2.0, 1e-10));
        }
    }

    #[test]
    fn test_scale_shrink() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("input.exo");
        let output = temp_dir.path().join("output.exo");

        fixtures::create_hex8_mesh(&input).unwrap();
        let (orig_x, _, _) = fixtures::read_coords(&input).unwrap();

        let (success, _, stderr) = run_rexonator(&[
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--scale-len",
            "0.001",
        ]);
        assert!(success, "rexonator failed: {}", stderr);

        let (new_x, _, _) = fixtures::read_coords(&output).unwrap();

        for (orig, new) in orig_x.iter().zip(new_x.iter()) {
            assert!(approx_eq(*new, orig * 0.001, 1e-12));
        }
    }

    #[test]
    #[ignore = "XFAIL: 2D meshes without z-coordinates cause panic in transform_ops (exodus-rs bug)"]
    fn test_scale_quad4() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("input.exo");
        let output = temp_dir.path().join("output.exo");

        fixtures::create_quad4_mesh(&input).unwrap();
        let (orig_x, orig_y, _) = fixtures::read_coords(&input).unwrap();

        let (success, _, stderr) = run_rexonator(&[
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--scale-len",
            "100.0",
        ]);
        assert!(success, "rexonator failed: {}", stderr);

        let (new_x, new_y, _) = fixtures::read_coords(&output).unwrap();

        for (orig, new) in orig_x.iter().zip(new_x.iter()) {
            assert!(approx_eq(*new, orig * 100.0, 1e-8));
        }
        for (orig, new) in orig_y.iter().zip(new_y.iter()) {
            assert!(approx_eq(*new, orig * 100.0, 1e-8));
        }
    }

    #[test]
    fn test_scale_tet4() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("input.exo");
        let output = temp_dir.path().join("output.exo");

        fixtures::create_tet4_mesh(&input).unwrap();
        let (orig_x, orig_y, orig_z) = fixtures::read_coords(&input).unwrap();

        let (success, _, stderr) = run_rexonator(&[
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--scale-len",
            "3.0",
        ]);
        assert!(success, "rexonator failed: {}", stderr);

        let (new_x, new_y, new_z) = fixtures::read_coords(&output).unwrap();

        for (orig, new) in orig_x.iter().zip(new_x.iter()) {
            assert!(approx_eq(*new, orig * 3.0, 1e-10));
        }
        for (orig, new) in orig_y.iter().zip(new_y.iter()) {
            assert!(approx_eq(*new, orig * 3.0, 1e-10));
        }
        for (orig, new) in orig_z.iter().zip(new_z.iter()) {
            assert!(approx_eq(*new, orig * 3.0, 1e-10));
        }
    }
}

mod rotation_tests {
    use super::*;

    #[test]
    fn test_rotate_z_90_degrees() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("input.exo");
        let output = temp_dir.path().join("output.exo");

        fixtures::create_hex8_mesh(&input).unwrap();
        let (orig_x, orig_y, orig_z) = fixtures::read_coords(&input).unwrap();

        let (success, _, stderr) = run_rexonator(&[
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--rotate",
            "Z,90",
        ]);
        assert!(success, "rexonator failed: {}", stderr);

        let (new_x, new_y, new_z) = fixtures::read_coords(&output).unwrap();

        // 90-degree Z rotation: (x,y) -> (-y, x)
        for i in 0..orig_x.len() {
            assert!(
                approx_eq(new_x[i], -orig_y[i], 1e-10),
                "X rotation wrong at {}: expected {} got {}",
                i,
                -orig_y[i],
                new_x[i]
            );
            assert!(
                approx_eq(new_y[i], orig_x[i], 1e-10),
                "Y rotation wrong at {}: expected {} got {}",
                i,
                orig_x[i],
                new_y[i]
            );
            assert!(
                approx_eq(new_z[i], orig_z[i], 1e-10),
                "Z should be unchanged"
            );
        }
    }

    #[test]
    fn test_rotate_x_90_degrees() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("input.exo");
        let output = temp_dir.path().join("output.exo");

        fixtures::create_hex8_mesh(&input).unwrap();
        let (orig_x, orig_y, orig_z) = fixtures::read_coords(&input).unwrap();

        let (success, _, stderr) = run_rexonator(&[
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--rotate",
            "X,90",
        ]);
        assert!(success, "rexonator failed: {}", stderr);

        let (new_x, new_y, new_z) = fixtures::read_coords(&output).unwrap();

        // 90-degree X rotation: (y,z) -> (-z, y)
        for i in 0..orig_x.len() {
            assert!(
                approx_eq(new_x[i], orig_x[i], 1e-10),
                "X should be unchanged"
            );
            assert!(
                approx_eq(new_y[i], -orig_z[i], 1e-10),
                "Y rotation wrong at {}: expected {} got {}",
                i,
                -orig_z[i],
                new_y[i]
            );
            assert!(
                approx_eq(new_z[i], orig_y[i], 1e-10),
                "Z rotation wrong at {}: expected {} got {}",
                i,
                orig_y[i],
                new_z[i]
            );
        }
    }

    #[test]
    fn test_rotate_xyz_euler() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("input.exo");
        let output = temp_dir.path().join("output.exo");

        fixtures::create_hex8_mesh(&input).unwrap();

        let (success, _, stderr) = run_rexonator(&[
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--rotate",
            "XYZ,30,45,60",
        ]);
        assert!(success, "rexonator failed: {}", stderr);

        // Just verify it runs - exact values depend on matrix implementation
        let params = fixtures::read_params(&output).unwrap();
        assert_eq!(params.num_nodes, 18);
    }

    #[test]
    fn test_rotate_extrinsic() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("input.exo");
        let output = temp_dir.path().join("output.exo");

        fixtures::create_hex8_mesh(&input).unwrap();
        let (orig_x, orig_y, orig_z) = fixtures::read_coords(&input).unwrap();

        // Extrinsic (uppercase)
        let (success, _, stderr) = run_rexonator(&[
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--rotate",
            "XYZ,30,45,60",
        ]);
        assert!(success, "Extrinsic rotation failed: {}", stderr);

        let (new_x, new_y, new_z) = fixtures::read_coords(&output).unwrap();

        // Verify coordinates have changed (rotation happened)
        let mut any_changed = false;
        for i in 0..orig_x.len() {
            if !approx_eq(new_x[i], orig_x[i], 1e-10)
                || !approx_eq(new_y[i], orig_y[i], 1e-10)
                || !approx_eq(new_z[i], orig_z[i], 1e-10)
            {
                any_changed = true;
                break;
            }
        }
        assert!(any_changed, "Rotation should change coordinates");
    }

    #[test]
    fn test_rotate_intrinsic() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("input.exo");
        let output = temp_dir.path().join("output.exo");

        fixtures::create_hex8_mesh(&input).unwrap();
        let (orig_x, orig_y, orig_z) = fixtures::read_coords(&input).unwrap();

        // Intrinsic (lowercase)
        let (success, _, stderr) = run_rexonator(&[
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--rotate",
            "xyz,30,45,60",
        ]);
        assert!(success, "Intrinsic rotation failed: {}", stderr);

        let (new_x, new_y, new_z) = fixtures::read_coords(&output).unwrap();

        // Verify coordinates have changed (rotation happened)
        let mut any_changed = false;
        for i in 0..orig_x.len() {
            if !approx_eq(new_x[i], orig_x[i], 1e-10)
                || !approx_eq(new_y[i], orig_y[i], 1e-10)
                || !approx_eq(new_z[i], orig_z[i], 1e-10)
            {
                any_changed = true;
                break;
            }
        }
        assert!(any_changed, "Rotation should change coordinates");
    }
}

mod mirror_tests {
    use super::*;

    #[test]
    fn test_mirror_x() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("input.exo");
        let output = temp_dir.path().join("output.exo");

        fixtures::create_hex8_mesh(&input).unwrap();
        let (orig_x, orig_y, orig_z) = fixtures::read_coords(&input).unwrap();

        let (success, _, stderr) = run_rexonator(&[
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--mirror",
            "x",
        ]);
        assert!(success, "rexonator failed: {}", stderr);

        let (new_x, new_y, new_z) = fixtures::read_coords(&output).unwrap();

        for (orig, new) in orig_x.iter().zip(new_x.iter()) {
            assert!(
                approx_eq(*new, -orig, 1e-10),
                "X should be negated: {} -> {} (expected {})",
                orig,
                new,
                -orig
            );
        }
        for (orig, new) in orig_y.iter().zip(new_y.iter()) {
            assert!(approx_eq(*new, *orig, 1e-10), "Y should be unchanged");
        }
        for (orig, new) in orig_z.iter().zip(new_z.iter()) {
            assert!(approx_eq(*new, *orig, 1e-10), "Z should be unchanged");
        }
    }

    #[test]
    fn test_mirror_y() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("input.exo");
        let output = temp_dir.path().join("output.exo");

        fixtures::create_hex8_mesh(&input).unwrap();
        let (orig_x, orig_y, orig_z) = fixtures::read_coords(&input).unwrap();

        let (success, _, stderr) = run_rexonator(&[
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--mirror",
            "Y",
        ]);
        assert!(success, "rexonator failed: {}", stderr);

        let (new_x, new_y, new_z) = fixtures::read_coords(&output).unwrap();

        for (orig, new) in orig_x.iter().zip(new_x.iter()) {
            assert!(approx_eq(*new, *orig, 1e-10), "X should be unchanged");
        }
        for (orig, new) in orig_y.iter().zip(new_y.iter()) {
            assert!(approx_eq(*new, -orig, 1e-10), "Y should be negated");
        }
        for (orig, new) in orig_z.iter().zip(new_z.iter()) {
            assert!(approx_eq(*new, *orig, 1e-10), "Z should be unchanged");
        }
    }

    #[test]
    fn test_mirror_z() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("input.exo");
        let output = temp_dir.path().join("output.exo");

        fixtures::create_hex8_mesh(&input).unwrap();
        let (orig_x, orig_y, orig_z) = fixtures::read_coords(&input).unwrap();

        let (success, _, stderr) = run_rexonator(&[
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--mirror",
            "z",
        ]);
        assert!(success, "rexonator failed: {}", stderr);

        let (new_x, new_y, new_z) = fixtures::read_coords(&output).unwrap();

        for (orig, new) in orig_x.iter().zip(new_x.iter()) {
            assert!(approx_eq(*new, *orig, 1e-10));
        }
        for (orig, new) in orig_y.iter().zip(new_y.iter()) {
            assert!(approx_eq(*new, *orig, 1e-10));
        }
        for (orig, new) in orig_z.iter().zip(new_z.iter()) {
            assert!(approx_eq(*new, -orig, 1e-10), "Z should be negated");
        }
    }
}

// =============================================================================
// Copy-Mirror-Merge Tests
// =============================================================================

mod copy_mirror_merge_tests {
    use super::*;

    #[test]
    fn test_cmm_hex8_x_axis() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("half.exo");
        let output = temp_dir.path().join("full.exo");

        fixtures::create_half_symmetry_hex8(&input).unwrap();
        let orig_params = fixtures::read_params(&input).unwrap();

        let (success, _, stderr) = run_rexonator(&[
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--copy-mirror-merge",
            "x",
        ]);
        assert!(success, "rexonator failed: {}", stderr);

        let new_params = fixtures::read_params(&output).unwrap();

        // Elements should double (minus shared nodes)
        assert_eq!(
            new_params.num_elems,
            orig_params.num_elems * 2,
            "Elements should double"
        );
        // Nodes should increase (but less than double due to merging on symmetry plane)
        assert!(
            new_params.num_nodes > orig_params.num_nodes,
            "Nodes should increase"
        );
        assert!(
            new_params.num_nodes < orig_params.num_nodes * 2,
            "Nodes should be less than doubled due to merging"
        );

        // X range should be symmetric about 0
        let (new_x, _, _) = fixtures::read_coords(&output).unwrap();
        let x_min = new_x.iter().copied().fold(f64::INFINITY, f64::min);
        let x_max = new_x.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        assert!(
            approx_eq(x_min.abs(), x_max.abs(), 0.01),
            "X range should be symmetric: min={}, max={}",
            x_min,
            x_max
        );

        println!("Test files at: {:?}", temp_dir.path());
        std::mem::forget(temp_dir);
    }

    #[test]
    fn test_cmm_hex8_y_axis() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("half.exo");
        let output = temp_dir.path().join("full.exo");

        // Create a half-symmetry mesh positioned for Y-axis mirroring
        fixtures::create_hex8_mesh(&input).unwrap();

        let (success, _, stderr) = run_rexonator(&[
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--copy-mirror-merge",
            "y",
        ]);
        assert!(success, "rexonator failed: {}", stderr);

        let new_params = fixtures::read_params(&output).unwrap();
        assert_eq!(
            new_params.num_elems, 8,
            "Elements should double from 4 to 8"
        );
    }

    #[test]
    fn test_cmm_creates_mirror_node_sets() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("half.exo");
        let output = temp_dir.path().join("full.exo");

        fixtures::create_hex8_mesh(&input).unwrap();
        let orig_ns_names = fixtures::read_node_set_names(&input).unwrap();

        let (success, _, stderr) = run_rexonator(&[
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--copy-mirror-merge",
            "x",
        ]);
        assert!(success, "rexonator failed: {}", stderr);

        let new_ns_names = fixtures::read_node_set_names(&output).unwrap();

        // Should have original + mirrored node sets
        assert_eq!(
            new_ns_names.len(),
            orig_ns_names.len() * 2,
            "Should have double the node sets"
        );

        // Check for _mirror suffix
        let has_mirror = new_ns_names.iter().any(|n| n.ends_with("_mirror"));
        assert!(has_mirror, "Should have _mirror node set names");
    }

    #[test]
    fn test_cmm_creates_mirror_blocks() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("half.exo");
        let output = temp_dir.path().join("full.exo");

        fixtures::create_hex8_mesh(&input).unwrap();

        let (success, _, stderr) = run_rexonator(&[
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--copy-mirror-merge",
            "x",
        ]);
        assert!(success, "rexonator failed: {}", stderr);

        let new_block_names = fixtures::read_block_names(&output).unwrap();

        // Check for _mirror block
        let has_mirror = new_block_names.iter().any(|n| n.ends_with("_mirror"));
        assert!(has_mirror, "Should have _mirror block names");
    }

    #[test]
    fn test_cmm_negates_vector_components() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("half.exo");
        let output = temp_dir.path().join("full.exo");

        fixtures::create_half_symmetry_hex8(&input).unwrap();

        // velocity_x should be 0 at x=0, positive for x>0
        // (We read but don't use orig_vx/orig_x here - we verify on the output)
        let _orig_vx = fixtures::read_nodal_var(&input, "velocity_x", 0).unwrap();
        let (_orig_x, _, _) = fixtures::read_coords(&input).unwrap();

        let (success, _, stderr) = run_rexonator(&[
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--copy-mirror-merge",
            "x",
        ]);
        assert!(success, "rexonator failed: {}", stderr);

        let new_vx = fixtures::read_nodal_var(&output, "velocity_x", 0).unwrap();
        let (new_x, _, _) = fixtures::read_coords(&output).unwrap();

        // Find nodes in negative x region (mirrored nodes)
        for (i, &x) in new_x.iter().enumerate() {
            if x < -0.01 {
                // This is a mirrored node - velocity_x should be negative
                assert!(
                    new_vx[i] < 0.0,
                    "velocity_x should be negative for x<0: node {} at x={} has vx={}",
                    i,
                    x,
                    new_vx[i]
                );
            }
        }

        // Check nodes on symmetry plane have vx=0
        for (i, &x) in new_x.iter().enumerate() {
            if x.abs() < 0.01 {
                assert!(
                    approx_eq(new_vx[i], 0.0, 0.01),
                    "velocity_x should be ~0 on symmetry plane: node {} at x={} has vx={}",
                    i,
                    x,
                    new_vx[i]
                );
            }
        }

        println!("Test files at: {:?}", temp_dir.path());
        std::mem::forget(temp_dir);
    }

    #[test]
    fn test_cmm_preserves_scalar_variables() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("input.exo");
        let output = temp_dir.path().join("output.exo");

        fixtures::create_scalar_only_mesh(&input).unwrap();

        let (success, _, stderr) = run_rexonator(&[
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--copy-mirror-merge",
            "x",
        ]);
        assert!(success, "rexonator failed: {}", stderr);

        // Temperature is a scalar - should not be negated
        let temp = fixtures::read_nodal_var(&output, "temperature", 0).unwrap();
        for val in &temp {
            assert!(
                *val >= 0.0,
                "Temperature should remain non-negative: {}",
                val
            );
        }
    }

    #[test]
    fn test_cmm_with_custom_tolerance() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("input.exo");
        let output = temp_dir.path().join("output.exo");

        fixtures::create_hex8_mesh(&input).unwrap();

        let (success, _, stderr) = run_rexonator(&[
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--copy-mirror-merge",
            "x",
            "--merge-tolerance",
            "0.01",
        ]);
        assert!(success, "rexonator failed: {}", stderr);
    }

    #[test]
    fn test_cmm_tet4() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("input.exo");
        let output = temp_dir.path().join("output.exo");

        fixtures::create_tet4_mesh(&input).unwrap();
        let orig_params = fixtures::read_params(&input).unwrap();

        let (success, _, stderr) = run_rexonator(&[
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--copy-mirror-merge",
            "z",
        ]);
        assert!(success, "rexonator failed: {}", stderr);

        let new_params = fixtures::read_params(&output).unwrap();
        assert_eq!(new_params.num_elems, orig_params.num_elems * 2);
    }

    #[test]
    fn test_cmm_wedge6() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("input.exo");
        let output = temp_dir.path().join("output.exo");

        fixtures::create_wedge6_mesh(&input).unwrap();

        let (success, _, stderr) = run_rexonator(&[
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--copy-mirror-merge",
            "z",
        ]);
        assert!(success, "rexonator failed: {}", stderr);

        let new_params = fixtures::read_params(&output).unwrap();
        assert_eq!(new_params.num_elems, 2); // 1 * 2
    }

    #[test]
    fn test_cmm_pyramid5() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("input.exo");
        let output = temp_dir.path().join("output.exo");

        fixtures::create_pyramid5_mesh(&input).unwrap();

        let (success, _, stderr) = run_rexonator(&[
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--copy-mirror-merge",
            "z",
        ]);
        assert!(success, "rexonator failed: {}", stderr);

        let new_params = fixtures::read_params(&output).unwrap();
        assert_eq!(new_params.num_elems, 2); // 1 * 2
    }

    #[test]
    fn test_cmm_quad4_2d() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("input.exo");
        let output = temp_dir.path().join("output.exo");

        fixtures::create_quad4_mesh(&input).unwrap();
        let orig_params = fixtures::read_params(&input).unwrap();

        let (success, _, stderr) = run_rexonator(&[
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--copy-mirror-merge",
            "x",
        ]);
        assert!(success, "rexonator failed: {}", stderr);

        let new_params = fixtures::read_params(&output).unwrap();
        assert_eq!(new_params.num_elems, orig_params.num_elems * 2);
    }

    #[test]
    fn test_cmm_tri3_2d() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("input.exo");
        let output = temp_dir.path().join("output.exo");

        fixtures::create_tri3_mesh(&input).unwrap();
        let orig_params = fixtures::read_params(&input).unwrap();

        let (success, _, stderr) = run_rexonator(&[
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--copy-mirror-merge",
            "x",
        ]);
        assert!(success, "rexonator failed: {}", stderr);

        let new_params = fixtures::read_params(&output).unwrap();
        assert_eq!(new_params.num_elems, orig_params.num_elems * 2);
    }
}

// =============================================================================
// Field Scaling Tests
// =============================================================================

mod field_scaling_tests {
    use super::*;

    #[test]
    fn test_scale_field_nodal_scalar() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("input.exo");
        let output = temp_dir.path().join("output.exo");

        fixtures::create_hex8_mesh(&input).unwrap();
        let orig_temp = fixtures::read_nodal_var(&input, "temperature", 0).unwrap();

        let (success, _, stderr) = run_rexonator(&[
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--scale-field",
            "temperature,2.0",
        ]);
        assert!(success, "rexonator failed: {}", stderr);

        let new_temp = fixtures::read_nodal_var(&output, "temperature", 0).unwrap();

        for (orig, new) in orig_temp.iter().zip(new_temp.iter()) {
            assert!(
                approx_eq(*new, orig * 2.0, 1e-10),
                "Temperature not scaled: {} -> {} (expected {})",
                orig,
                new,
                orig * 2.0
            );
        }
    }

    #[test]
    fn test_scale_field_multiple() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("input.exo");
        let output = temp_dir.path().join("output.exo");

        fixtures::create_hex8_mesh(&input).unwrap();
        let orig_temp = fixtures::read_nodal_var(&input, "temperature", 0).unwrap();
        let orig_vx = fixtures::read_nodal_var(&input, "velocity_x", 0).unwrap();

        let (success, _, stderr) = run_rexonator(&[
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--scale-field",
            "temperature,1.5",
            "--scale-field",
            "velocity_x,3.0",
        ]);
        assert!(success, "rexonator failed: {}", stderr);

        let new_temp = fixtures::read_nodal_var(&output, "temperature", 0).unwrap();
        let new_vx = fixtures::read_nodal_var(&output, "velocity_x", 0).unwrap();

        for (orig, new) in orig_temp.iter().zip(new_temp.iter()) {
            assert!(approx_eq(*new, orig * 1.5, 1e-10));
        }
        for (orig, new) in orig_vx.iter().zip(new_vx.iter()) {
            assert!(approx_eq(*new, orig * 3.0, 1e-10));
        }
    }

    #[test]
    fn test_scale_field_scientific_notation() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("input.exo");
        let output = temp_dir.path().join("output.exo");

        fixtures::create_hex8_mesh(&input).unwrap();
        let orig_temp = fixtures::read_nodal_var(&input, "temperature", 0).unwrap();

        let (success, _, stderr) = run_rexonator(&[
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--scale-field",
            "temperature,1e-3",
        ]);
        assert!(success, "rexonator failed: {}", stderr);

        let new_temp = fixtures::read_nodal_var(&output, "temperature", 0).unwrap();

        for (orig, new) in orig_temp.iter().zip(new_temp.iter()) {
            assert!(approx_eq(*new, orig * 1e-3, 1e-13));
        }
    }
}

// =============================================================================
// Time Normalization Tests
// =============================================================================

mod time_tests {
    use super::*;

    #[test]
    fn test_zero_time() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("input.exo");
        let output = temp_dir.path().join("output.exo");

        fixtures::create_multi_timestep_mesh(&input).unwrap();
        let orig_times = fixtures::read_times(&input).unwrap();
        let first_time = orig_times[0];

        let (success, _, stderr) = run_rexonator(&[
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--zero-time",
        ]);
        assert!(success, "rexonator failed: {}", stderr);

        let new_times = fixtures::read_times(&output).unwrap();

        // First time step should be 0
        assert!(
            approx_eq(new_times[0], 0.0, 1e-10),
            "First time step should be 0, got {}",
            new_times[0]
        );

        // Other time steps should be shifted
        for (orig, new) in orig_times.iter().zip(new_times.iter()) {
            let expected = orig - first_time;
            assert!(
                approx_eq(*new, expected, 1e-10),
                "Time not normalized: {} -> {} (expected {})",
                orig,
                new,
                expected
            );
        }
    }

    #[test]
    fn test_zero_time_with_other_transform() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("input.exo");
        let output = temp_dir.path().join("output.exo");

        // Use hex8 mesh with time steps instead of 2D mesh
        fixtures::create_hex8_mesh(&input).unwrap();
        let (orig_x, _, _) = fixtures::read_coords(&input).unwrap();
        let orig_times = fixtures::read_times(&input).unwrap();

        let (success, _, stderr) = run_rexonator(&[
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--translate",
            "5.0,0.0,0.0",
            "--zero-time",
        ]);
        assert!(success, "rexonator failed: {}", stderr);

        // Both transform and time normalization should be applied
        let (new_x, _, _) = fixtures::read_coords(&output).unwrap();
        for (orig, new) in orig_x.iter().zip(new_x.iter()) {
            assert!(approx_eq(*new, orig + 5.0, 1e-10));
        }

        let new_times = fixtures::read_times(&output).unwrap();
        assert!(approx_eq(new_times[0], 0.0, 1e-10));

        // Time differences should be preserved
        if new_times.len() > 1 {
            let orig_diff = orig_times[1] - orig_times[0];
            let new_diff = new_times[1] - new_times[0];
            assert!(approx_eq(new_diff, orig_diff, 1e-10));
        }
    }
}

// =============================================================================
// Transformation Ordering Tests
// =============================================================================

mod ordering_tests {
    use super::*;

    #[test]
    fn test_translate_then_rotate_vs_rotate_then_translate() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("input.exo");
        let out1 = temp_dir.path().join("translate_rotate.exo");
        let out2 = temp_dir.path().join("rotate_translate.exo");

        fixtures::create_hex8_mesh(&input).unwrap();

        // Translate then rotate
        let (s1, _, _) = run_rexonator(&[
            input.to_str().unwrap(),
            out1.to_str().unwrap(),
            "--translate",
            "10.0,0.0,0.0",
            "--rotate",
            "Z,90",
        ]);
        assert!(s1);

        // Rotate then translate
        let (s2, _, _) = run_rexonator(&[
            input.to_str().unwrap(),
            out2.to_str().unwrap(),
            "--rotate",
            "Z,90",
            "--translate",
            "10.0,0.0,0.0",
        ]);
        assert!(s2);

        let (x1, y1, _) = fixtures::read_coords(&out1).unwrap();
        let (x2, y2, _) = fixtures::read_coords(&out2).unwrap();

        // These should be different
        let mut any_different = false;
        for i in 0..x1.len() {
            if !approx_eq(x1[i], x2[i], 1e-10) || !approx_eq(y1[i], y2[i], 1e-10) {
                any_different = true;
                break;
            }
        }
        assert!(
            any_different,
            "Order of operations should produce different results"
        );
    }

    #[test]
    fn test_multiple_translations() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("input.exo");
        let output = temp_dir.path().join("output.exo");

        fixtures::create_hex8_mesh(&input).unwrap();
        let (orig_x, _, _) = fixtures::read_coords(&input).unwrap();

        let (success, _, _) = run_rexonator(&[
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--translate",
            "1.0,0.0,0.0",
            "--translate",
            "2.0,0.0,0.0",
            "--translate",
            "3.0,0.0,0.0",
        ]);
        assert!(success);

        let (new_x, _, _) = fixtures::read_coords(&output).unwrap();

        // Total translation should be 1+2+3 = 6
        for (orig, new) in orig_x.iter().zip(new_x.iter()) {
            assert!(approx_eq(*new, orig + 6.0, 1e-10));
        }
    }

    #[test]
    fn test_scale_then_translate() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("input.exo");
        let output = temp_dir.path().join("output.exo");

        fixtures::create_hex8_mesh(&input).unwrap();
        let (orig_x, _, _) = fixtures::read_coords(&input).unwrap();

        // Scale by 2, then translate by 10
        let (success, _, _) = run_rexonator(&[
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--scale-len",
            "2.0",
            "--translate",
            "10.0,0.0,0.0",
        ]);
        assert!(success);

        let (new_x, _, _) = fixtures::read_coords(&output).unwrap();

        // Result should be x*2 + 10
        for (orig, new) in orig_x.iter().zip(new_x.iter()) {
            let expected = orig * 2.0 + 10.0;
            assert!(
                approx_eq(*new, expected, 1e-10),
                "Expected {} got {}",
                expected,
                new
            );
        }
    }

    #[test]
    fn test_pre_cmm_and_post_cmm_operations() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("input.exo");
        let output = temp_dir.path().join("output.exo");

        fixtures::create_half_symmetry_hex8(&input).unwrap();

        // Scale first, then CMM, then translate
        let (success, _, stderr) = run_rexonator(&[
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--scale-len",
            "2.0",
            "--copy-mirror-merge",
            "x",
            "--translate",
            "5.0,0.0,0.0",
        ]);
        assert!(success, "rexonator failed: {}", stderr);

        // Verify mesh doubled
        let new_params = fixtures::read_params(&output).unwrap();
        assert_eq!(new_params.num_elems, 2);
    }
}

// =============================================================================
// Verbose Output Tests
// =============================================================================

mod verbose_tests {
    use super::*;

    #[test]
    fn test_verbose_output() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("input.exo");
        let output = temp_dir.path().join("output.exo");

        fixtures::create_hex8_mesh(&input).unwrap();

        let (success, stdout, _) = run_rexonator(&[
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--translate",
            "1.0,0.0,0.0",
            "-v",
        ]);
        assert!(success);
        assert!(stdout.contains("Input:") || stdout.contains("Translating"));
    }
}

// =============================================================================
// Error Handling Tests
// =============================================================================

mod error_tests {
    use super::*;

    #[test]
    fn test_invalid_input_file() {
        let (success, _, _stderr) = run_rexonator(&["nonexistent.exo", "output.exo"]);
        assert!(!success, "Command should fail for nonexistent input file");
    }

    #[test]
    fn test_invalid_translate_format() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("input.exo");
        let output = temp_dir.path().join("output.exo");

        fixtures::create_hex8_mesh(&input).unwrap();

        let (success, _, _) = run_rexonator(&[
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--translate",
            "1,2", // Missing Z
        ]);
        assert!(!success, "Should fail with invalid translate format");
    }

    #[test]
    fn test_invalid_axis() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("input.exo");
        let output = temp_dir.path().join("output.exo");

        fixtures::create_hex8_mesh(&input).unwrap();

        let (success, _, _) = run_rexonator(&[
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--mirror",
            "w", // Invalid axis
        ]);
        assert!(!success, "Should fail with invalid axis");
    }
}

// =============================================================================
// Tests for Known Issues (xfail tests)
// These tests document known limitations from PLAN.md that should be addressed
// =============================================================================

mod known_issues {
    use super::*;

    /// XFAIL: Vector component detection false positives
    /// Fields like "max_x" or "index_x" might be incorrectly identified as vector components
    /// See PLAN.md: "Vector component detection false positives"
    #[test]
    #[ignore = "XFAIL: Known issue - vector component detection false positives (PLAN.md)"]
    fn test_cmm_scalar_field_ending_with_x_not_negated() {
        // This test documents the known false positive issue
        // A field named "max_x" should NOT be negated, but the current implementation
        // might incorrectly identify it as a vector component
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("input.exo");
        let output = temp_dir.path().join("output.exo");

        // Create mesh with a field that ends in "_x" but is not a vector component
        // (e.g., "max_x" representing a maximum value in x direction)
        // When this issue is fixed, this test should pass
        fixtures::create_half_symmetry_hex8(&input).unwrap();

        let (success, _, _) = run_rexonator(&[
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--copy-mirror-merge",
            "x",
        ]);
        assert!(success);

        // When fixed: verify that "max_x" values are NOT negated in the mirrored region
        // (This would require creating a custom fixture with "max_x" field)
    }

    /// XFAIL: Side set side number mapping is incomplete
    /// See PLAN.md: "Complete side set side number mapping (TODO in code)"
    #[test]
    #[ignore = "XFAIL: Known issue - side set side numbers not properly mapped for mirrored elements (PLAN.md)"]
    fn test_cmm_side_set_side_numbers_mapped() {
        // This test documents that side set side numbers are not currently
        // properly adjusted for mirrored elements
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("input.exo");
        let output = temp_dir.path().join("output.exo");

        fixtures::create_hex8_mesh(&input).unwrap();

        let (success, _, _) = run_rexonator(&[
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--copy-mirror-merge",
            "x",
        ]);
        assert!(success);

        // When fixed: verify that side numbers in mirrored side sets
        // are properly mapped based on topology and mirror axis
    }

    /// XFAIL: 2D mesh handling in copy-mirror-merge
    /// See PLAN.md: "2D Mesh Z-Coordinate Handling"
    #[test]
    #[ignore = "XFAIL: Known issue - 2D mesh might be converted to 3D during CMM (PLAN.md)"]
    fn test_cmm_2d_mesh_stays_2d() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("input.exo");
        let output = temp_dir.path().join("output.exo");

        fixtures::create_quad4_mesh(&input).unwrap();
        let orig_params = fixtures::read_params(&input).unwrap();
        assert_eq!(orig_params.num_dim, 2, "Input should be 2D");

        let (success, _, _) = run_rexonator(&[
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--copy-mirror-merge",
            "x",
        ]);
        assert!(success);

        let new_params = fixtures::read_params(&output).unwrap();
        // When fixed: This should remain 2D
        assert_eq!(new_params.num_dim, 2, "Output should remain 2D after CMM");
    }

    /// XFAIL: Multiple copy-mirror-merge operations not supported
    /// Currently only one CMM operation per invocation is allowed
    #[test]
    fn test_multiple_cmm_not_supported() {
        let temp_dir = TempDir::new().unwrap();
        let input = temp_dir.path().join("input.exo");
        let output = temp_dir.path().join("output.exo");

        fixtures::create_half_symmetry_hex8(&input).unwrap();

        let (success, _, stderr) = run_rexonator(&[
            input.to_str().unwrap(),
            output.to_str().unwrap(),
            "--copy-mirror-merge",
            "x",
            "--copy-mirror-merge",
            "y", // Second CMM - should fail or be rejected
        ]);

        // Current behavior: this should fail with an error
        assert!(
            !success || stderr.contains("Only one"),
            "Multiple CMM operations should be rejected"
        );
    }
}
