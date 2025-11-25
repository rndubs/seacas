//! exo-cfd-transform: CLI tool for transforming Exodus mesh files
//!
//! This tool applies geometric transformations to Exodus II mesh files,
//! including translation, rotation, scaling, and mirroring. Transformations
//! are applied in the order they appear on the command line.

use clap::Parser;
use exodus_rs::{transformations::rotation_matrix_from_euler, ExodusFile};
use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during mesh transformation
#[derive(Error, Debug)]
pub enum TransformError {
    /// Exodus file error
    #[error("Exodus error: {0}")]
    Exodus(#[from] exodus_rs::ExodusError),

    /// Invalid argument format
    #[error("Invalid argument format: {0}")]
    InvalidFormat(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

type Result<T> = std::result::Result<T, TransformError>;

/// Transform Exodus mesh files with translation, rotation, scaling, and mirroring.
///
/// Transformations are applied in the order they appear on the command line.
/// For example: `--translate 1,0,0 --rotate "Z,90" --scale-len 2` will first
/// translate, then rotate, then scale.
#[derive(Parser, Debug)]
#[command(name = "exo-cfd-transform")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Input Exodus file
    #[arg(value_name = "INPUT", required_unless_present = "man")]
    input: Option<PathBuf>,

    /// Output Exodus file
    #[arg(value_name = "OUTPUT", required_unless_present = "man")]
    output: Option<PathBuf>,

    /// Scale mesh coordinates uniformly by a factor
    #[arg(long = "scale-len", value_name = "FACTOR")]
    scale_len: Vec<f64>,

    /// Mirror (reflect) mesh about an axis (x, y, or z)
    #[arg(long, value_name = "AXIS")]
    mirror: Vec<String>,

    /// Translate mesh by x,y,z offset (e.g., "1.0,2.0,3.0")
    #[arg(long, value_name = "X,Y,Z")]
    translate: Vec<String>,

    /// Rotate mesh using Euler angles.
    /// Format: "SEQUENCE,angle1,angle2,angle3" where SEQUENCE is 1-3 axes.
    /// Uppercase (XYZ) = extrinsic (fixed frame), lowercase (xyz) = intrinsic (body frame).
    /// Examples: "Z,90" or "XYZ,30,45,60" or "zyx,10,20,30"
    #[arg(long, value_name = "SEQ,ANGLES")]
    rotate: Vec<String>,

    /// Normalize time values so the first time step is zero
    #[arg(short = 'z', long = "zero-time")]
    zero_time: bool,

    /// Print verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Display the man page
    #[arg(long)]
    man: bool,
}

/// Represents a transformation operation
#[derive(Debug, Clone)]
enum Operation {
    /// Scale coordinates uniformly
    ScaleLen(f64),
    /// Mirror about an axis
    Mirror(Axis),
    /// Translate by offset
    Translate([f64; 3]),
    /// Rotate using Euler angles (sequence, angles in degrees)
    Rotate(String, Vec<f64>),
}

/// Axis for mirroring
#[derive(Debug, Clone, Copy)]
enum Axis {
    X,
    Y,
    Z,
}

impl std::str::FromStr for Axis {
    type Err = TransformError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "x" => Ok(Axis::X),
            "y" => Ok(Axis::Y),
            "z" => Ok(Axis::Z),
            _ => Err(TransformError::InvalidFormat(format!(
                "Invalid axis '{}', must be x, y, or z",
                s
            ))),
        }
    }
}

/// Parse a translate argument "x,y,z" into [f64; 3]
fn parse_translate(s: &str) -> Result<[f64; 3]> {
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
fn parse_rotate(s: &str) -> Result<(String, Vec<f64>)> {
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

/// Extract operations from command-line args in the order they appear
fn extract_ordered_operations(cli: &Cli) -> Result<Vec<Operation>> {
    let args: Vec<String> = std::env::args().collect();
    let mut operations: Vec<(usize, Operation)> = Vec::new();

    // Track indices for each operation type
    let mut scale_idx = 0;
    let mut mirror_idx = 0;
    let mut translate_idx = 0;
    let mut rotate_idx = 0;

    for (pos, arg) in args.iter().enumerate() {
        match arg.as_str() {
            "--scale-len" => {
                if scale_idx < cli.scale_len.len() {
                    operations.push((pos, Operation::ScaleLen(cli.scale_len[scale_idx])));
                    scale_idx += 1;
                }
            }
            "--mirror" => {
                if mirror_idx < cli.mirror.len() {
                    let axis: Axis = cli.mirror[mirror_idx].parse()?;
                    operations.push((pos, Operation::Mirror(axis)));
                    mirror_idx += 1;
                }
            }
            "--translate" => {
                if translate_idx < cli.translate.len() {
                    let offset = parse_translate(&cli.translate[translate_idx])?;
                    operations.push((pos, Operation::Translate(offset)));
                    translate_idx += 1;
                }
            }
            "--rotate" => {
                if rotate_idx < cli.rotate.len() {
                    let (seq, angles) = parse_rotate(&cli.rotate[rotate_idx])?;
                    operations.push((pos, Operation::Rotate(seq, angles)));
                    rotate_idx += 1;
                }
            }
            _ => {}
        }
    }

    // Sort by position to preserve command-line order
    operations.sort_by_key(|(pos, _)| *pos);

    Ok(operations.into_iter().map(|(_, op)| op).collect())
}

/// Apply a single transformation operation to the mesh
fn apply_operation(
    file: &mut ExodusFile<exodus_rs::mode::Append>,
    op: &Operation,
    verbose: bool,
) -> Result<()> {
    match op {
        Operation::ScaleLen(factor) => {
            if verbose {
                println!("  Scaling coordinates by factor {}", factor);
            }
            file.scale_uniform(*factor)?;
        }
        Operation::Mirror(axis) => {
            let scale = match axis {
                Axis::X => [-1.0, 1.0, 1.0],
                Axis::Y => [1.0, -1.0, 1.0],
                Axis::Z => [1.0, 1.0, -1.0],
            };
            if verbose {
                println!("  Mirroring about {:?} axis", axis);
            }
            file.scale(&scale)?;
        }
        Operation::Translate(offset) => {
            if verbose {
                println!(
                    "  Translating by [{}, {}, {}]",
                    offset[0], offset[1], offset[2]
                );
            }
            file.translate(offset)?;
        }
        Operation::Rotate(sequence, angles) => {
            if verbose {
                let rotation_type = if sequence.chars().next().unwrap().is_uppercase() {
                    "extrinsic"
                } else {
                    "intrinsic"
                };
                println!(
                    "  Rotating {} '{}' by {:?} degrees",
                    rotation_type, sequence, angles
                );
            }
            // Get the rotation matrix and apply it
            let matrix = rotation_matrix_from_euler(sequence, angles, true)?;
            file.apply_rotation(&matrix)?;
        }
    }
    Ok(())
}

/// Display the man page by looking for it relative to the executable
fn show_man_page() -> Result<()> {
    use std::process::Command;

    // Get the executable path
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path.parent().ok_or_else(|| {
        TransformError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not determine executable directory",
        ))
    })?;

    // Look for the man page in the same directory as the executable
    let man_page = exe_dir.join("exo-cfd-transform.1");

    if !man_page.exists() {
        eprintln!("Man page not found at: {}", man_page.display());
        eprintln!("Please ensure exo-cfd-transform.1 is in the same directory as the executable.");
        eprintln!("\nYou can view it with: man {}", man_page.display());
        std::process::exit(1);
    }

    // Use the man command to display it
    let status = Command::new("man").arg(man_page.as_os_str()).status()?;

    if !status.success() {
        eprintln!("Failed to display man page");
        std::process::exit(1);
    }

    Ok(())
}

/// Normalize time values so the first time step is zero
fn normalize_time(file: &mut ExodusFile<exodus_rs::mode::Append>, verbose: bool) -> Result<()> {
    let times = file.times()?;

    if times.is_empty() {
        if verbose {
            println!("  No time steps to normalize");
        }
        return Ok(());
    }

    let first_time = times[0];
    if verbose {
        println!(
            "  Normalizing time: subtracting {} from all {} time steps",
            first_time,
            times.len()
        );
    }

    for (step, time) in times.iter().enumerate() {
        let normalized = time - first_time;
        file.put_time(step, normalized)?;
    }

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle --man flag
    if cli.man {
        show_man_page()?;
        return Ok(());
    }

    // Unwrap input and output (guaranteed to be present due to required_unless_present)
    let input = cli.input.as_ref().unwrap();
    let output = cli.output.as_ref().unwrap();

    // Extract operations in command-line order
    let operations = extract_ordered_operations(&cli)?;

    if cli.verbose {
        println!("Input:  {}", input.display());
        println!("Output: {}", output.display());
        println!("Operations to apply: {}", operations.len());
    }

    // Copy input file to output location
    if cli.verbose {
        println!("Copying input file to output location...");
    }
    std::fs::copy(input, output)?;

    // Open the output file in append mode for modifications
    let mut file = ExodusFile::append(output)?;

    if cli.verbose {
        let params = file.init_params()?;
        println!(
            "Mesh: {} nodes, {} elements, {} dimensions",
            params.num_nodes, params.num_elems, params.num_dim
        );
    }

    // Apply transformations in order
    if cli.verbose && !operations.is_empty() {
        println!("Applying transformations:");
    }

    for op in &operations {
        apply_operation(&mut file, op, cli.verbose)?;
    }

    // Apply time normalization if requested
    if cli.zero_time {
        if cli.verbose {
            println!("Normalizing time values:");
        }
        normalize_time(&mut file, cli.verbose)?;
    }

    if cli.verbose {
        println!("Done!");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
