//! Argument parsing functions for rexonator transformations
//!
//! This module contains functions for parsing command-line arguments into
//! transformation operations, including translate, rotate, scale-field, and
//! operation ordering.

use crate::cli::{Axis, Cli, Operation, Result, TransformError};

/// Parse a translate argument "x,y,z" into an array of 3 floats
pub fn parse_translate(s: &str) -> Result<[f64; 3]> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 3 {
        return Err(TransformError::InvalidFormat(format!(
            "Translate requires 3 values (x,y,z), got {}",
            parts.len()
        )));
    }

    let x = parts[0]
        .trim()
        .parse::<f64>()
        .map_err(|_| TransformError::InvalidFormat(format!("Invalid x value: {}", parts[0])))?;
    let y = parts[1]
        .trim()
        .parse::<f64>()
        .map_err(|_| TransformError::InvalidFormat(format!("Invalid y value: {}", parts[1])))?;
    let z = parts[2]
        .trim()
        .parse::<f64>()
        .map_err(|_| TransformError::InvalidFormat(format!("Invalid z value: {}", parts[2])))?;

    Ok([x, y, z])
}

/// Parse a rotate argument "SEQUENCE,a1,a2,a3" into (sequence, angles)
pub fn parse_rotate(s: &str) -> Result<(String, Vec<f64>)> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.is_empty() {
        return Err(TransformError::InvalidFormat(
            "Rotate requires at least a sequence".to_string(),
        ));
    }

    let sequence = parts[0].trim().to_string();
    let seq_len = sequence.len();

    if seq_len == 0 || seq_len > 3 {
        return Err(TransformError::InvalidFormat(format!(
            "Euler sequence must be 1-3 characters, got {}",
            seq_len
        )));
    }

    let expected_angles = seq_len;
    let actual_angles = parts.len() - 1;

    if actual_angles != expected_angles {
        return Err(TransformError::InvalidFormat(format!(
            "Sequence '{}' requires {} angle(s), got {}",
            sequence, expected_angles, actual_angles
        )));
    }

    let angles: Result<Vec<f64>> = parts[1..]
        .iter()
        .enumerate()
        .map(|(i, p)| {
            p.trim().parse::<f64>().map_err(|_| {
                TransformError::InvalidFormat(format!("Invalid angle {}: {}", i + 1, p))
            })
        })
        .collect();

    Ok((sequence, angles?))
}

/// Parse a scale-field argument "field_name,scale_factor" into (name, factor)
pub fn parse_scale_field(s: &str) -> Result<(String, f64)> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 2 {
        return Err(TransformError::InvalidFormat(format!(
            "Scale field requires 2 values (name,factor), got {}",
            parts.len()
        )));
    }

    let field_name = parts[0].trim().to_string();
    if field_name.is_empty() {
        return Err(TransformError::InvalidFormat(
            "Field name cannot be empty".to_string(),
        ));
    }

    let scale_factor = parts[1].trim().parse::<f64>().map_err(|_| {
        TransformError::InvalidFormat(format!("Invalid scale factor: {}", parts[1]))
    })?;

    Ok((field_name, scale_factor))
}

/// Check if an argument matches a flag (handles both "--flag" and "--flag=value" forms)
pub fn arg_matches_flag(arg: &str, flag: &str) -> bool {
    arg == flag || arg.starts_with(&format!("{}=", flag))
}

/// Extract operations from args in the order they appear (testable version)
pub fn extract_ordered_operations_from_args(
    args: &[String],
    cli: &Cli,
    verbose: bool,
) -> Result<Vec<Operation>> {
    let mut operations: Vec<(usize, Operation)> = Vec::new();

    if verbose {
        println!("DEBUG: Raw args: {:?}", args);
        println!(
            "DEBUG: Clap parsed - translate: {:?}, rotate: {:?}, scale_len: {:?}, mirror: {:?}, scale_field: {:?}, copy_mirror_merge: {:?}",
            cli.translate, cli.rotate, cli.scale_len, cli.mirror, cli.scale_field, cli.copy_mirror_merge
        );
    }

    // Track indices for each operation type
    let mut scale_idx = 0;
    let mut mirror_idx = 0;
    let mut translate_idx = 0;
    let mut rotate_idx = 0;
    let mut scale_field_idx = 0;
    let mut copy_mirror_merge_idx = 0;

    for (pos, arg) in args.iter().enumerate() {
        if arg_matches_flag(arg, "--scale-len") && scale_idx < cli.scale_len.len() {
            if verbose {
                println!(
                    "DEBUG: Found --scale-len at pos {}, value: {}",
                    pos, cli.scale_len[scale_idx]
                );
            }
            operations.push((pos, Operation::ScaleLen(cli.scale_len[scale_idx])));
            scale_idx += 1;
        } else if arg_matches_flag(arg, "--mirror") && mirror_idx < cli.mirror.len() {
            if verbose {
                println!(
                    "DEBUG: Found --mirror at pos {}, value: {}",
                    pos, cli.mirror[mirror_idx]
                );
            }
            let axis: Axis = cli.mirror[mirror_idx].parse()?;
            operations.push((pos, Operation::Mirror(axis)));
            mirror_idx += 1;
        } else if arg_matches_flag(arg, "--translate") && translate_idx < cli.translate.len() {
            if verbose {
                println!(
                    "DEBUG: Found --translate at pos {}, value: {}",
                    pos, cli.translate[translate_idx]
                );
            }
            let offset = parse_translate(&cli.translate[translate_idx])?;
            operations.push((pos, Operation::Translate(offset)));
            translate_idx += 1;
        } else if arg_matches_flag(arg, "--rotate") && rotate_idx < cli.rotate.len() {
            if verbose {
                println!(
                    "DEBUG: Found --rotate at pos {}, value: {}",
                    pos, cli.rotate[rotate_idx]
                );
            }
            let (seq, angles) = parse_rotate(&cli.rotate[rotate_idx])?;
            operations.push((pos, Operation::Rotate(seq, angles)));
            rotate_idx += 1;
        } else if arg_matches_flag(arg, "--scale-field") && scale_field_idx < cli.scale_field.len()
        {
            if verbose {
                println!(
                    "DEBUG: Found --scale-field at pos {}, value: {}",
                    pos, cli.scale_field[scale_field_idx]
                );
            }
            let (field_name, scale_factor) = parse_scale_field(&cli.scale_field[scale_field_idx])?;
            operations.push((pos, Operation::ScaleField(field_name, scale_factor)));
            scale_field_idx += 1;
        } else if arg_matches_flag(arg, "--copy-mirror-merge")
            && copy_mirror_merge_idx < cli.copy_mirror_merge.len()
        {
            if verbose {
                println!(
                    "DEBUG: Found --copy-mirror-merge at pos {}, value: {}",
                    pos, cli.copy_mirror_merge[copy_mirror_merge_idx]
                );
            }
            let axis: Axis = cli.copy_mirror_merge[copy_mirror_merge_idx].parse()?;
            operations.push((pos, Operation::CopyMirrorMerge(axis, cli.merge_tolerance)));
            copy_mirror_merge_idx += 1;
        }
    }

    // Sort by position to preserve command-line order
    operations.sort_by_key(|(pos, _)| *pos);

    if verbose {
        println!("DEBUG: Final operation order:");
        for (i, op) in operations.iter().enumerate() {
            println!("  {}: pos={}, {:?}", i, op.0, op.1);
        }
    }

    Ok(operations.into_iter().map(|(_, op)| op).collect())
}

/// Extract operations from command-line args in the order they appear
pub fn extract_ordered_operations(cli: &Cli, verbose: bool) -> Result<Vec<Operation>> {
    let args: Vec<String> = std::env::args().collect();
    extract_ordered_operations_from_args(&args, cli, verbose)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_translate() {
        let result = parse_translate("1.0,2.0,3.0").unwrap();
        assert_eq!(result, [1.0, 2.0, 3.0]);

        let result = parse_translate("  1.5 , -2.5 , 0  ").unwrap();
        assert_eq!(result, [1.5, -2.5, 0.0]);
    }

    #[test]
    fn test_parse_translate_invalid() {
        assert!(parse_translate("1,2").is_err());
        assert!(parse_translate("1,2,3,4").is_err());
        assert!(parse_translate("a,b,c").is_err());
    }

    #[test]
    fn test_parse_rotate() {
        let (seq, angles) = parse_rotate("Z,90").unwrap();
        assert_eq!(seq, "Z");
        assert_eq!(angles, vec![90.0]);

        let (seq, angles) = parse_rotate("XYZ,30,45,60").unwrap();
        assert_eq!(seq, "XYZ");
        assert_eq!(angles, vec![30.0, 45.0, 60.0]);

        let (seq, angles) = parse_rotate("xyz,10,20,30").unwrap();
        assert_eq!(seq, "xyz");
        assert_eq!(angles, vec![10.0, 20.0, 30.0]);
    }

    #[test]
    fn test_parse_rotate_invalid() {
        // Wrong number of angles
        assert!(parse_rotate("XYZ,30,45").is_err());
        assert!(parse_rotate("Z,30,45").is_err());

        // Empty sequence
        assert!(parse_rotate(",90").is_err());

        // Too many axes
        assert!(parse_rotate("XYZW,1,2,3,4").is_err());
    }

    #[test]
    fn test_axis_parse() {
        assert!(matches!("x".parse::<Axis>().unwrap(), Axis::X));
        assert!(matches!("Y".parse::<Axis>().unwrap(), Axis::Y));
        assert!(matches!("z".parse::<Axis>().unwrap(), Axis::Z));
        assert!("w".parse::<Axis>().is_err());
    }

    #[test]
    fn test_parse_scale_field_valid() {
        // Valid inputs
        let result = parse_scale_field("temperature,1.5").unwrap();
        assert_eq!(result.0, "temperature");
        assert!((result.1 - 1.5).abs() < 1e-10);

        let result = parse_scale_field("stress,2.0").unwrap();
        assert_eq!(result.0, "stress");
        assert!((result.1 - 2.0).abs() < 1e-10);

        let result = parse_scale_field("pressure,0.5").unwrap();
        assert_eq!(result.0, "pressure");
        assert!((result.1 - 0.5).abs() < 1e-10);

        // With spaces
        let result = parse_scale_field("  velocity  ,  3.14  ").unwrap();
        assert_eq!(result.0, "velocity");
        assert!((result.1 - 3.14).abs() < 1e-10);

        // Negative scale factor
        let result = parse_scale_field("displacement,-1.0").unwrap();
        assert_eq!(result.0, "displacement");
        assert!((result.1 - (-1.0)).abs() < 1e-10);
    }

    #[test]
    fn test_parse_scale_field_invalid() {
        // Missing scale factor
        assert!(parse_scale_field("temperature").is_err());

        // Too many parts
        assert!(parse_scale_field("temperature,1.5,extra").is_err());

        // Empty field name
        assert!(parse_scale_field(",1.5").is_err());
        assert!(parse_scale_field("  ,1.5").is_err());

        // Invalid scale factor
        assert!(parse_scale_field("temperature,abc").is_err());
        assert!(parse_scale_field("temperature,").is_err());
        assert!(parse_scale_field("temperature,1.5.6").is_err());

        // Empty string
        assert!(parse_scale_field("").is_err());
    }

    #[test]
    fn test_parse_scale_field_scientific_notation() {
        // Test scale factors in scientific notation
        let result = parse_scale_field("temperature,1.5e-3").unwrap();
        assert_eq!(result.0, "temperature");
        assert!((result.1 - 1.5e-3).abs() < 1e-10);

        let result = parse_scale_field("pressure,2.5E+2").unwrap();
        assert_eq!(result.0, "pressure");
        assert!((result.1 - 2.5e2).abs() < 1e-10);
    }

    #[test]
    fn test_arg_matches_flag() {
        // Exact match (space-separated form: --flag value)
        assert!(arg_matches_flag("--translate", "--translate"));
        assert!(arg_matches_flag("--scale-len", "--scale-len"));
        assert!(arg_matches_flag("--mirror", "--mirror"));
        assert!(arg_matches_flag("--rotate", "--rotate"));

        // Equals form (--flag=value)
        assert!(arg_matches_flag("--translate=1,0,0", "--translate"));
        assert!(arg_matches_flag("--scale-len=2.0", "--scale-len"));
        assert!(arg_matches_flag("--mirror=x", "--mirror"));
        assert!(arg_matches_flag("--rotate=Z,90", "--rotate"));

        // Non-matches
        assert!(!arg_matches_flag("--translatex", "--translate")); // No equals sign
        assert!(!arg_matches_flag("--trans", "--translate")); // Partial match
        assert!(!arg_matches_flag("-t", "--translate")); // Short form (not supported)
        assert!(!arg_matches_flag("translate", "--translate")); // Missing dashes
    }

    /// Helper to create a test CLI with specific operations
    fn make_test_cli(
        translate: Vec<String>,
        rotate: Vec<String>,
        scale_len: Vec<f64>,
        mirror: Vec<String>,
    ) -> Cli {
        Cli {
            input: Some(PathBuf::from("input.exo")),
            output: Some(PathBuf::from("output.exo")),
            scale_len,
            mirror,
            translate,
            rotate,
            scale_field: vec![],
            copy_mirror_merge: vec![],
            merge_tolerance: 0.001,
            zero_time: false,
            verbose: false,
            cache_size: None,
            preemption: None,
            node_chunk: None,
            element_chunk: None,
            time_chunk: None,
            show_perf_config: false,
            man: false,
        }
    }

    /// Helper to create a test CLI with copy-mirror-merge
    fn make_test_cli_with_cmm(
        translate: Vec<String>,
        rotate: Vec<String>,
        scale_len: Vec<f64>,
        mirror: Vec<String>,
        copy_mirror_merge: Vec<String>,
        merge_tolerance: f64,
    ) -> Cli {
        Cli {
            input: Some(PathBuf::from("input.exo")),
            output: Some(PathBuf::from("output.exo")),
            scale_len,
            mirror,
            translate,
            rotate,
            scale_field: vec![],
            copy_mirror_merge,
            merge_tolerance,
            zero_time: false,
            verbose: false,
            cache_size: None,
            preemption: None,
            node_chunk: None,
            element_chunk: None,
            time_chunk: None,
            show_perf_config: false,
            man: false,
        }
    }

    #[test]
    fn test_operation_order_translate_then_rotate() {
        // Simulate: rexonator in.exo out.exo --translate 1,0,0 --rotate Z,90
        let args: Vec<String> = vec![
            "rexonator",
            "in.exo",
            "out.exo",
            "--translate",
            "1,0,0",
            "--rotate",
            "Z,90",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        let cli = make_test_cli(
            vec!["1,0,0".to_string()],
            vec!["Z,90".to_string()],
            vec![],
            vec![],
        );

        let ops = extract_ordered_operations_from_args(&args, &cli, false).unwrap();

        assert_eq!(ops.len(), 2);
        assert!(matches!(ops[0], Operation::Translate(_)));
        assert!(matches!(ops[1], Operation::Rotate(_, _)));
    }

    #[test]
    fn test_operation_order_rotate_then_translate() {
        // Simulate: rexonator in.exo out.exo --rotate Z,90 --translate 1,0,0
        let args: Vec<String> = vec![
            "rexonator",
            "in.exo",
            "out.exo",
            "--rotate",
            "Z,90",
            "--translate",
            "1,0,0",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        let cli = make_test_cli(
            vec!["1,0,0".to_string()],
            vec!["Z,90".to_string()],
            vec![],
            vec![],
        );

        let ops = extract_ordered_operations_from_args(&args, &cli, false).unwrap();

        assert_eq!(ops.len(), 2);
        assert!(matches!(ops[0], Operation::Rotate(_, _)));
        assert!(matches!(ops[1], Operation::Translate(_)));
    }

    #[test]
    fn test_operation_order_interleaved() {
        // Simulate: rexonator in.exo out.exo --translate 1,0,0 --rotate Z,90 --translate 2,0,0
        let args: Vec<String> = vec![
            "rexonator",
            "in.exo",
            "out.exo",
            "--translate",
            "1,0,0",
            "--rotate",
            "Z,90",
            "--translate",
            "2,0,0",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        let cli = make_test_cli(
            vec!["1,0,0".to_string(), "2,0,0".to_string()],
            vec!["Z,90".to_string()],
            vec![],
            vec![],
        );

        let ops = extract_ordered_operations_from_args(&args, &cli, false).unwrap();

        assert_eq!(ops.len(), 3);
        assert!(matches!(ops[0], Operation::Translate([1.0, 0.0, 0.0])));
        assert!(matches!(ops[1], Operation::Rotate(_, _)));
        assert!(matches!(ops[2], Operation::Translate([2.0, 0.0, 0.0])));
    }

    #[test]
    fn test_operation_order_all_types() {
        // Simulate: rexonator in.exo out.exo --mirror x --translate 1,0,0 --scale-len 2 --rotate Z,90
        let args: Vec<String> = vec![
            "rexonator",
            "in.exo",
            "out.exo",
            "--mirror",
            "x",
            "--translate",
            "1,0,0",
            "--scale-len",
            "2",
            "--rotate",
            "Z,90",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        let cli = make_test_cli(
            vec!["1,0,0".to_string()],
            vec!["Z,90".to_string()],
            vec![2.0],
            vec!["x".to_string()],
        );

        let ops = extract_ordered_operations_from_args(&args, &cli, false).unwrap();

        assert_eq!(ops.len(), 4);
        assert!(matches!(ops[0], Operation::Mirror(Axis::X)));
        assert!(matches!(ops[1], Operation::Translate(_)));
        assert!(matches!(ops[2], Operation::ScaleLen(2.0)));
        assert!(matches!(ops[3], Operation::Rotate(_, _)));
    }

    #[test]
    fn test_operation_order_equals_syntax() {
        // Simulate: rexonator in.exo out.exo --translate=1,0,0 --rotate=Z,90
        let args: Vec<String> = vec![
            "rexonator",
            "in.exo",
            "out.exo",
            "--translate=1,0,0",
            "--rotate=Z,90",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        let cli = make_test_cli(
            vec!["1,0,0".to_string()],
            vec!["Z,90".to_string()],
            vec![],
            vec![],
        );

        let ops = extract_ordered_operations_from_args(&args, &cli, false).unwrap();

        assert_eq!(ops.len(), 2);
        assert!(matches!(ops[0], Operation::Translate(_)));
        assert!(matches!(ops[1], Operation::Rotate(_, _)));
    }

    #[test]
    fn test_operation_order_equals_syntax_reversed() {
        // Simulate: rexonator in.exo out.exo --rotate=Z,90 --translate=1,0,0
        let args: Vec<String> = vec![
            "rexonator",
            "in.exo",
            "out.exo",
            "--rotate=Z,90",
            "--translate=1,0,0",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        let cli = make_test_cli(
            vec!["1,0,0".to_string()],
            vec!["Z,90".to_string()],
            vec![],
            vec![],
        );

        let ops = extract_ordered_operations_from_args(&args, &cli, false).unwrap();

        assert_eq!(ops.len(), 2);
        assert!(matches!(ops[0], Operation::Rotate(_, _)));
        assert!(matches!(ops[1], Operation::Translate(_)));
    }

    #[test]
    fn test_scale_field_operation_order() {
        // Test that scale-field operations are ordered correctly
        let args: Vec<String> = vec![
            "rexonator",
            "in.exo",
            "out.exo",
            "--scale-field",
            "temperature,1.5",
            "--scale-len",
            "2.0",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        let mut cli = make_test_cli(vec![], vec![], vec![2.0], vec![]);
        cli.scale_field = vec!["temperature,1.5".to_string()];

        let ops = extract_ordered_operations_from_args(&args, &cli, false).unwrap();

        assert_eq!(ops.len(), 2);
        assert!(matches!(
            ops[0],
            Operation::ScaleField(ref name, factor) if name == "temperature" && (factor - 1.5).abs() < 1e-10
        ));
        assert!(matches!(ops[1], Operation::ScaleLen(_)));
    }

    #[test]
    fn test_multiple_scale_field_operations() {
        // Test multiple field scaling operations
        let args: Vec<String> = vec![
            "rexonator",
            "in.exo",
            "out.exo",
            "--scale-field",
            "temperature,1.5",
            "--scale-field",
            "pressure,0.5",
            "--scale-field",
            "velocity,2.0",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        let mut cli = make_test_cli(vec![], vec![], vec![], vec![]);
        cli.scale_field = vec![
            "temperature,1.5".to_string(),
            "pressure,0.5".to_string(),
            "velocity,2.0".to_string(),
        ];

        let ops = extract_ordered_operations_from_args(&args, &cli, false).unwrap();

        assert_eq!(ops.len(), 3);
        assert!(matches!(
            ops[0],
            Operation::ScaleField(ref name, factor) if name == "temperature" && (factor - 1.5).abs() < 1e-10
        ));
        assert!(matches!(
            ops[1],
            Operation::ScaleField(ref name, factor) if name == "pressure" && (factor - 0.5).abs() < 1e-10
        ));
        assert!(matches!(
            ops[2],
            Operation::ScaleField(ref name, factor) if name == "velocity" && (factor - 2.0).abs() < 1e-10
        ));
    }

    #[test]
    fn test_scale_field_with_other_operations() {
        // Test scale-field mixed with other operations
        let args: Vec<String> = vec![
            "rexonator",
            "in.exo",
            "out.exo",
            "--translate",
            "1,0,0",
            "--scale-field",
            "stress,1.23",
            "--rotate",
            "Z,90",
            "--scale-field",
            "temperature,1.8",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        let mut cli = make_test_cli(
            vec!["1,0,0".to_string()],
            vec!["Z,90".to_string()],
            vec![],
            vec![],
        );
        cli.scale_field = vec!["stress,1.23".to_string(), "temperature,1.8".to_string()];

        let ops = extract_ordered_operations_from_args(&args, &cli, false).unwrap();

        assert_eq!(ops.len(), 4);
        assert!(matches!(ops[0], Operation::Translate(_)));
        assert!(matches!(
            ops[1],
            Operation::ScaleField(ref name, factor) if name == "stress" && (factor - 1.23).abs() < 1e-10
        ));
        assert!(matches!(ops[2], Operation::Rotate(_, _)));
        assert!(matches!(
            ops[3],
            Operation::ScaleField(ref name, factor) if name == "temperature" && (factor - 1.8).abs() < 1e-10
        ));
    }

    #[test]
    fn test_scale_field_equals_syntax() {
        // Test --scale-field=value syntax
        let args: Vec<String> = vec![
            "rexonator",
            "in.exo",
            "out.exo",
            "--scale-field=temperature,1.5",
            "--scale-field=pressure,0.5",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        let mut cli = make_test_cli(vec![], vec![], vec![], vec![]);
        cli.scale_field = vec!["temperature,1.5".to_string(), "pressure,0.5".to_string()];

        let ops = extract_ordered_operations_from_args(&args, &cli, false).unwrap();

        assert_eq!(ops.len(), 2);
        assert!(matches!(
            ops[0],
            Operation::ScaleField(ref name, factor) if name == "temperature" && (factor - 1.5).abs() < 1e-10
        ));
        assert!(matches!(
            ops[1],
            Operation::ScaleField(ref name, factor) if name == "pressure" && (factor - 0.5).abs() < 1e-10
        ));
    }

    #[test]
    fn test_scale_field_with_underscores_and_numbers() {
        // Test field names with underscores and numbers
        let result = parse_scale_field("velocity_x,2.5").unwrap();
        assert_eq!(result.0, "velocity_x");
        assert!((result.1 - 2.5).abs() < 1e-10);

        let result = parse_scale_field("stress_11,1.0").unwrap();
        assert_eq!(result.0, "stress_11");
        assert!((result.1 - 1.0).abs() < 1e-10);

        let result = parse_scale_field("field_var_123,0.75").unwrap();
        assert_eq!(result.0, "field_var_123");
        assert!((result.1 - 0.75).abs() < 1e-10);
    }

    #[test]
    fn test_copy_mirror_merge_operation_parsing() {
        // Simulate: rexonator in.exo out.exo --copy-mirror-merge x
        let args: Vec<String> = vec!["rexonator", "in.exo", "out.exo", "--copy-mirror-merge", "x"]
            .into_iter()
            .map(String::from)
            .collect();

        let cli =
            make_test_cli_with_cmm(vec![], vec![], vec![], vec![], vec!["x".to_string()], 0.001);

        let ops = extract_ordered_operations_from_args(&args, &cli, false).unwrap();

        assert_eq!(ops.len(), 1);
        assert!(
            matches!(ops[0], Operation::CopyMirrorMerge(Axis::X, tol) if (tol - 0.001).abs() < 0.0001)
        );
    }

    #[test]
    fn test_copy_mirror_merge_with_other_ops() {
        // Simulate: rexonator in.exo out.exo --translate 1,0,0 --copy-mirror-merge x --rotate Z,90
        let args: Vec<String> = vec![
            "rexonator",
            "in.exo",
            "out.exo",
            "--translate",
            "1,0,0",
            "--copy-mirror-merge",
            "x",
            "--rotate",
            "Z,90",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        let cli = make_test_cli_with_cmm(
            vec!["1,0,0".to_string()],
            vec!["Z,90".to_string()],
            vec![],
            vec![],
            vec!["x".to_string()],
            0.005,
        );

        let ops = extract_ordered_operations_from_args(&args, &cli, false).unwrap();

        assert_eq!(ops.len(), 3);
        assert!(matches!(ops[0], Operation::Translate(_)));
        assert!(
            matches!(ops[1], Operation::CopyMirrorMerge(Axis::X, tol) if (tol - 0.005).abs() < 0.0001)
        );
        assert!(matches!(ops[2], Operation::Rotate(_, _)));
    }

    #[test]
    fn test_copy_mirror_merge_equals_syntax() {
        // Simulate: rexonator in.exo out.exo --copy-mirror-merge=y
        let args: Vec<String> = vec!["rexonator", "in.exo", "out.exo", "--copy-mirror-merge=y"]
            .into_iter()
            .map(String::from)
            .collect();

        let cli =
            make_test_cli_with_cmm(vec![], vec![], vec![], vec![], vec!["y".to_string()], 0.001);

        let ops = extract_ordered_operations_from_args(&args, &cli, false).unwrap();

        assert_eq!(ops.len(), 1);
        assert!(matches!(ops[0], Operation::CopyMirrorMerge(Axis::Y, _)));
    }
}
