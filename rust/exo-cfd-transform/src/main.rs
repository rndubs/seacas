//! exo-cfd-transform: CLI tool for transforming Exodus mesh files
//!
//! This tool applies geometric transformations to Exodus II mesh files,
//! including translation, rotation, scaling, and mirroring. Transformations
//! are applied in the order they appear on the command line.

use clap::Parser;
use exodus_rs::{mode, transformations::rotation_matrix_from_euler, types::*, ExodusFile};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::{env, fmt};
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
    #[arg(value_name = "INPUT", required_unless_present_any = ["man", "show_perf_config"])]
    input: Option<PathBuf>,

    /// Output Exodus file
    #[arg(value_name = "OUTPUT", required_unless_present_any = ["man", "show_perf_config"])]
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

    /// Scale individual field variables by name.
    /// Format: "field_name,scale_factor"
    /// Examples: "stress,1.23" or "temperature,0.5"
    /// Can be specified multiple times to scale different fields
    #[arg(long = "scale-field", value_name = "NAME,FACTOR")]
    scale_field: Vec<String>,

    /// Copy, mirror, and merge mesh about a symmetry plane.
    /// Creates a full model from a half-symmetry model by mirroring about the
    /// specified axis and merging nodes on the symmetry plane.
    /// Nodes within merge-tolerance of the symmetry plane (axis=0) are merged.
    #[arg(long = "copy-mirror-merge", value_name = "AXIS")]
    copy_mirror_merge: Vec<String>,

    /// Tolerance for merging nodes on the symmetry plane (default: 0.001)
    #[arg(
        long = "merge-tolerance",
        value_name = "VALUE",
        default_value = "0.001"
    )]
    merge_tolerance: f64,

    /// Normalize time values so the first time step is zero
    #[arg(short = 'z', long = "zero-time")]
    zero_time: bool,

    /// Print verbose output
    #[arg(short, long)]
    verbose: bool,

    // --- NetCDF5/HDF5 Performance Options ---
    /// HDF5 chunk cache size in megabytes.
    /// Larger cache improves I/O performance for large files.
    /// Default: auto-detected based on environment (4-128 MB).
    #[arg(long, value_name = "MB")]
    cache_size: Option<usize>,

    /// HDF5 cache preemption policy (0.0 to 1.0).
    /// 0.0 = favor write performance, 1.0 = favor read performance.
    /// Default: 0.75 (balanced).
    #[arg(long, value_name = "VALUE")]
    preemption: Option<f64>,

    /// Node chunk size: number of nodes per HDF5 chunk.
    /// Affects chunking for nodal data in new files.
    /// Default: 1,000-10,000 based on environment.
    #[arg(long, value_name = "SIZE")]
    node_chunk: Option<usize>,

    /// Element chunk size: number of elements per HDF5 chunk.
    /// Affects chunking for element data in new files.
    /// Default: 1,000-10,000 based on environment.
    #[arg(long, value_name = "SIZE")]
    element_chunk: Option<usize>,

    /// Time step chunk size: number of time steps per HDF5 chunk.
    /// 0 = no time chunking (mesh-oriented I/O, default).
    /// 1+ = chunk multiple time steps together.
    #[arg(long, value_name = "SIZE")]
    time_chunk: Option<usize>,

    /// Print performance configuration and exit
    #[arg(long)]
    show_perf_config: bool,

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
    /// Scale a specific field variable (name, scale_factor)
    ScaleField(String, f64),
    /// Copy, mirror, and merge about symmetry plane (axis, tolerance)
    CopyMirrorMerge(Axis, f64),
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

/// Performance configuration for HDF5/NetCDF I/O
#[derive(Debug, Clone)]
struct PerformanceOptions {
    /// Cache size in bytes
    cache_size: usize,
    /// Number of hash table slots (0 = auto)
    num_slots: usize,
    /// Preemption policy (0.0-1.0)
    preemption: f64,
    /// Node chunk size
    node_chunk_size: usize,
    /// Element chunk size
    element_chunk_size: usize,
    /// Time chunk size
    time_chunk_size: usize,
}

impl PerformanceOptions {
    /// Build performance options from CLI arguments
    fn from_cli(cli: &Cli) -> Self {
        // Start with auto-detected defaults based on environment
        let node_type = detect_node_type();
        let (default_cache, default_chunk) = match node_type {
            "compute" => (128 * 1024 * 1024, 10_000), // 128 MB, 10k nodes/elements
            "login" => (4 * 1024 * 1024, 1_000),      // 4 MB, 1k nodes/elements
            _ => (16 * 1024 * 1024, 5_000),           // 16 MB, 5k nodes/elements
        };

        let cache_size = cli
            .cache_size
            .map(|mb| mb * 1024 * 1024)
            .unwrap_or(default_cache);

        let preemption = cli.preemption.unwrap_or(0.75).clamp(0.0, 1.0);

        let node_chunk_size = cli.node_chunk.unwrap_or(default_chunk);
        let element_chunk_size = cli.element_chunk.unwrap_or(default_chunk);
        let time_chunk_size = cli.time_chunk.unwrap_or(0);

        // Auto-calculate hash slots based on cache size
        // Target: ~100x the number of chunks that fit in cache
        let num_slots = {
            let typical_chunk_bytes = 1024 * 1024; // 1 MB typical chunk
            let chunks_in_cache = cache_size / typical_chunk_bytes;
            let target = chunks_in_cache * 100;
            next_prime(target.max(521))
        };

        Self {
            cache_size,
            num_slots,
            preemption,
            node_chunk_size,
            element_chunk_size,
            time_chunk_size,
        }
    }

    /// Apply HDF5 environment variables for performance tuning
    fn apply_env_vars(&self) {
        // These must be set before HDF5 library is initialized
        // Only set if not already set by user
        if env::var("HDF5_CHUNK_CACHE_NBYTES").is_err() {
            env::set_var("HDF5_CHUNK_CACHE_NBYTES", self.cache_size.to_string());
        }

        if env::var("HDF5_CHUNK_CACHE_W0").is_err() {
            env::set_var("HDF5_CHUNK_CACHE_W0", self.preemption.to_string());
        }

        if env::var("HDF5_CHUNK_CACHE_NSLOTS").is_err() {
            env::set_var("HDF5_CHUNK_CACHE_NSLOTS", self.num_slots.to_string());
        }
    }
}

impl fmt::Display for PerformanceOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "NetCDF5/HDF5 Performance Configuration:")?;
        writeln!(f, "  Node type: {}", detect_node_type())?;
        writeln!(
            f,
            "  Cache size: {} MB ({} bytes)",
            self.cache_size / (1024 * 1024),
            self.cache_size
        )?;
        writeln!(f, "  Cache slots: {} (auto-calculated)", self.num_slots)?;
        writeln!(f, "  Preemption: {:.2}", self.preemption)?;
        writeln!(f, "  Node chunk size: {} nodes", self.node_chunk_size)?;
        writeln!(
            f,
            "  Element chunk size: {} elements",
            self.element_chunk_size
        )?;
        write!(f, "  Time chunk size: {} steps", self.time_chunk_size)?;
        if self.time_chunk_size == 0 {
            write!(f, " (no time chunking)")?;
        }
        Ok(())
    }
}

/// Detect the node type based on environment variables
fn detect_node_type() -> &'static str {
    // Check if we're inside a job scheduler
    if env::var("SLURM_JOB_ID").is_ok()
        || env::var("FLUX_URI").is_ok()
        || env::var("PBS_JOBID").is_ok()
        || env::var("LSB_JOBID").is_ok()
    {
        return "compute";
    }

    // Check if we're on a system with schedulers (but not in a job)
    if env::var("SLURM_CONF").is_ok()
        || env::var("FLUX_EXEC_PATH").is_ok()
        || env::var("PBS_SERVER").is_ok()
        || env::var("LSF_ENVDIR").is_ok()
    {
        return "login";
    }

    "unknown"
}

/// Find the next prime number >= n
fn next_prime(n: usize) -> usize {
    if n <= 2 {
        return 2;
    }

    let mut candidate = if n.is_multiple_of(2) { n + 1 } else { n };

    loop {
        if is_prime(candidate) {
            return candidate;
        }
        candidate += 2;
    }
}

/// Check if a number is prime
fn is_prime(n: usize) -> bool {
    if n <= 1 {
        return false;
    }
    if n <= 3 {
        return true;
    }
    if n.is_multiple_of(2) || n.is_multiple_of(3) {
        return false;
    }

    let limit = (n as f64).sqrt() as usize;
    let mut i = 5;
    while i <= limit {
        if n.is_multiple_of(i) || n.is_multiple_of(i + 2) {
            return false;
        }
        i += 6;
    }
    true
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

/// Parse a scale-field argument "field_name,scale_factor" into (name, factor)
fn parse_scale_field(s: &str) -> Result<(String, f64)> {
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
fn arg_matches_flag(arg: &str, flag: &str) -> bool {
    arg == flag || arg.starts_with(&format!("{}=", flag))
}

/// Extract operations from args in the order they appear (testable version)
fn extract_ordered_operations_from_args(
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
fn extract_ordered_operations(cli: &Cli, verbose: bool) -> Result<Vec<Operation>> {
    let args: Vec<String> = std::env::args().collect();
    extract_ordered_operations_from_args(&args, cli, verbose)
}

/// Apply a single simple transformation operation to the mesh
fn apply_simple_operation(
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
        Operation::ScaleField(field_name, scale_factor) => {
            if verbose {
                println!(
                    "  Scaling field variable '{}' by factor {}",
                    field_name, scale_factor
                );
            }
            file.scale_field_variable(field_name, *scale_factor, verbose)?;
        }
        Operation::CopyMirrorMerge(_, _) => {
            // This should be handled separately, not through apply_simple_operation
            return Err(TransformError::InvalidFormat(
                "CopyMirrorMerge must be handled specially".to_string(),
            ));
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

// ============================================================================
// Copy-Mirror-Merge Implementation
// ============================================================================

/// Type alias for side set data: (id, elements, sides, dist_factors)
type SideSetData = (i64, Vec<i64>, Vec<i64>, Vec<f64>);

/// Data structure to hold all mesh data for copy-mirror-merge operation
#[derive(Debug)]
struct MeshData {
    // Initialization parameters
    params: InitParams,
    // Coordinates
    x: Vec<f64>,
    y: Vec<f64>,
    z: Vec<f64>,
    // Element blocks (supports multiple blocks)
    blocks: Vec<Block>,
    connectivities: Vec<Vec<i64>>,
    block_names: Vec<String>,
    // Node sets (id, nodes, dist_factors)
    node_sets: Vec<(i64, Vec<i64>, Vec<f64>)>,
    // Side sets (id, elements, sides, dist_factors)
    side_sets: Vec<SideSetData>,
    // Nodal variables (name, values for each time step)
    nodal_var_names: Vec<String>,
    nodal_var_values: Vec<Vec<Vec<f64>>>, // [var_idx][time_step][node_idx]
    // Element block variables (name, values for each block and time step)
    elem_var_names: Vec<String>,
    elem_var_values: Vec<Vec<Vec<Vec<f64>>>>, // [block_idx][var_idx][time_step][elem_idx]
    // Global variables
    global_var_names: Vec<String>,
    global_var_values: Vec<Vec<f64>>, // [time_step][var_idx]
    // Time values
    times: Vec<f64>,
    // Set names (for creating _mirror variants)
    node_set_names: Vec<String>,
    side_set_names: Vec<String>,
}

/// Find nodes on the symmetry plane
fn find_symmetry_plane_nodes(coords: &[f64], _axis: Axis, tolerance: f64) -> Vec<usize> {
    coords
        .iter()
        .enumerate()
        .filter_map(|(i, &val)| {
            if val.abs() <= tolerance {
                Some(i)
            } else {
                None
            }
        })
        .collect()
}

/// Get the coordinate array for a given axis
fn get_axis_coords<'a>(x: &'a [f64], y: &'a [f64], z: &'a [f64], axis: Axis) -> &'a [f64] {
    match axis {
        Axis::X => x,
        Axis::Y => y,
        Axis::Z => z,
    }
}

/// Get the winding order permutation for mirroring an element
/// Returns indices to reorder element connectivity to maintain proper winding
fn get_mirror_permutation(topology: &str, axis: Axis) -> Option<Vec<usize>> {
    let topo_upper = topology.to_uppercase();

    match topo_upper.as_str() {
        "HEX" | "HEX8" => {
            // HEX8 node ordering:
            //     7-------6
            //    /|      /|
            //   4-------5 |
            //   | |     | |
            //   | 3-----|-2
            //   |/      |/
            //   0-------1
            Some(match axis {
                Axis::X => vec![1, 0, 3, 2, 5, 4, 7, 6], // Swap X pairs
                Axis::Y => vec![3, 2, 1, 0, 7, 6, 5, 4], // Swap Y pairs
                Axis::Z => vec![4, 5, 6, 7, 0, 1, 2, 3], // Swap Z pairs (top/bottom)
            })
        }
        "TET" | "TET4" | "TETRA" | "TETRA4" => {
            // TET4: swap any two nodes reverses orientation
            Some(vec![0, 2, 1, 3])
        }
        "WEDGE" | "WEDGE6" => {
            // Wedge: swap triangular faces
            Some(match axis {
                Axis::X => vec![1, 0, 2, 4, 3, 5],
                Axis::Y => vec![2, 1, 0, 5, 4, 3],
                Axis::Z => vec![3, 4, 5, 0, 1, 2],
            })
        }
        "PYRAMID" | "PYRAMID5" => {
            // Pyramid: reverse base ordering
            Some(vec![3, 2, 1, 0, 4])
        }
        "QUAD" | "QUAD4" | "SHELL" | "SHELL4" => {
            // 2D quad: reverse winding
            Some(vec![0, 3, 2, 1])
        }
        "TRI" | "TRI3" | "TRIANGLE" => {
            // 2D triangle: reverse winding
            Some(vec![0, 2, 1])
        }
        _ => None, // Unsupported topology
    }
}

/// Check if a variable name suggests it's a vector component
fn is_vector_component(name: &str, axis: Axis) -> bool {
    let name_lower = name.to_lowercase();
    let suffix = match axis {
        Axis::X => ["_x", "x", "_u", "u"],
        Axis::Y => ["_y", "y", "_v", "v"],
        Axis::Z => ["_z", "z", "_w", "w"],
    };

    suffix
        .iter()
        .any(|s| name_lower.ends_with(s) || (name_lower.len() == 1 && name_lower == *s))
}

/// Read all mesh data from a file
fn read_mesh_data(file: &ExodusFile<mode::Read>, verbose: bool) -> Result<MeshData> {
    let params = file.init_params()?;

    if verbose {
        println!(
            "  Reading mesh: {} nodes, {} elements",
            params.num_nodes, params.num_elems
        );
    }

    // Read coordinates
    let coords = file.coords()?;
    let x = coords.x;
    let y = coords.y;
    let z = coords.z;

    // Read all element blocks
    let block_ids = file.block_ids(EntityType::ElemBlock)?;
    if block_ids.is_empty() {
        return Err(TransformError::InvalidFormat(
            "No element blocks found in mesh".to_string(),
        ));
    }

    let mut blocks = Vec::new();
    let mut connectivities = Vec::new();

    for &block_id in &block_ids {
        let block = file.block(block_id)?;

        // Check for supported topology
        let perm = get_mirror_permutation(&block.topology, Axis::X);
        if perm.is_none() {
            return Err(TransformError::InvalidFormat(format!(
                "Unsupported element topology '{}' in block {} for copy-mirror-merge. \
                 Supported: HEX8, TET4, WEDGE6, PYRAMID5, QUAD4, TRI3",
                block.topology, block_id
            )));
        }

        let connectivity = file.connectivity(block_id)?;

        if verbose {
            println!(
                "  Element block {}: {} elements, topology: {}",
                block_id, block.num_entries, block.topology
            );
        }

        blocks.push(block);
        connectivities.push(connectivity);
    }

    // Read block names
    let block_names = file.names(EntityType::ElemBlock).unwrap_or_default();

    // Read node sets
    let mut node_sets = Vec::new();
    let node_set_ids = file.set_ids(EntityType::NodeSet)?;
    for &set_id in &node_set_ids {
        let ns = file.node_set(set_id)?;
        node_sets.push((set_id, ns.nodes, ns.dist_factors));
    }

    // Read side sets
    let mut side_sets = Vec::new();
    let side_set_ids = file.set_ids(EntityType::SideSet)?;
    for &set_id in &side_set_ids {
        let ss = file.side_set(set_id)?;
        side_sets.push((set_id, ss.elements, ss.sides, ss.dist_factors));
    }

    // Read set names
    let node_set_names = file.names(EntityType::NodeSet).unwrap_or_default();
    let side_set_names = file.names(EntityType::SideSet).unwrap_or_default();

    // Read time values
    let times = file.times()?;
    let num_time_steps = times.len();

    // Read nodal variables
    let nodal_var_names = file.variable_names(EntityType::Nodal)?;
    let mut nodal_var_values: Vec<Vec<Vec<f64>>> = Vec::new();

    if verbose {
        println!(
            "  Found {} nodal variables, {} time steps",
            nodal_var_names.len(),
            num_time_steps
        );
    }

    for var_idx in 0..nodal_var_names.len() {
        let mut var_time_series = Vec::new();
        for step in 0..num_time_steps {
            let values = file.var(step, EntityType::Nodal, 0, var_idx)?;
            var_time_series.push(values);
        }
        nodal_var_values.push(var_time_series);
    }

    if verbose && !nodal_var_names.is_empty() {
        println!("  Nodal variables: {:?}", nodal_var_names);
    }

    // Read element block variables
    let elem_var_names = file.variable_names(EntityType::ElemBlock)?;
    let mut elem_var_values: Vec<Vec<Vec<Vec<f64>>>> = Vec::new(); // [block_idx][var_idx][time_step][elem_idx]

    if verbose {
        println!(
            "  Found {} element variables across {} blocks",
            elem_var_names.len(),
            blocks.len()
        );
    }

    for (block_idx, block) in blocks.iter().enumerate() {
        let mut block_vars: Vec<Vec<Vec<f64>>> = Vec::new(); // [var_idx][time_step][elem_idx]

        for var_idx in 0..elem_var_names.len() {
            let mut var_time_series: Vec<Vec<f64>> = Vec::new();
            for step in 0..num_time_steps {
                // Use block.id as entity_id for element block variables
                match file.var(step, EntityType::ElemBlock, block.id, var_idx) {
                    Ok(values) => {
                        if verbose && step == 0 && block_idx == 0 {
                            println!(
                                "    Read {} values for elem var {} on block {}",
                                values.len(),
                                var_idx,
                                block.id
                            );
                        }
                        var_time_series.push(values);
                    }
                    Err(e) => {
                        if verbose && step == 0 {
                            println!(
                                "    Warning: Could not read elem var {} on block {}: {}",
                                var_idx, block.id, e
                            );
                        }
                        // Variable might not be defined for this block (truth table)
                        // Use empty vector to indicate no data
                        var_time_series.push(Vec::new());
                    }
                }
            }
            block_vars.push(var_time_series);
        }
        elem_var_values.push(block_vars);

        if verbose && !elem_var_names.is_empty() && block_idx == 0 {
            println!("  Element variables: {:?}", elem_var_names);
        }
    }

    // Read global variables
    let global_var_names = file.variable_names(EntityType::Global)?;
    let mut global_var_values: Vec<Vec<f64>> = Vec::new();

    for step in 0..num_time_steps {
        let mut step_values = Vec::new();
        for var_idx in 0..global_var_names.len() {
            let values = file.var(step, EntityType::Global, 0, var_idx)?;
            step_values.extend(values);
        }
        global_var_values.push(step_values);
    }

    if verbose && !global_var_names.is_empty() {
        println!("  Global variables: {:?}", global_var_names);
        eprintln!(
            "WARNING: Global variables found. These may need manual adjustment after mirroring:"
        );
        for name in &global_var_names {
            eprintln!("  - {}", name);
        }
        eprintln!("         (e.g., total mass may need doubling, time step size is unchanged)");
    }

    Ok(MeshData {
        params,
        x,
        y,
        z,
        blocks,
        connectivities,
        block_names,
        node_sets,
        side_sets,
        nodal_var_names,
        nodal_var_values,
        elem_var_names,
        elem_var_values,
        global_var_names,
        global_var_values,
        times,
        node_set_names,
        side_set_names,
    })
}

/// Perform the copy-mirror-merge operation
fn copy_mirror_merge(
    data: &MeshData,
    axis: Axis,
    tolerance: f64,
    verbose: bool,
) -> Result<MeshData> {
    let orig_num_nodes = data.params.num_nodes;
    let orig_num_elems = data.params.num_elems;

    // Find nodes on the symmetry plane
    let axis_coords = get_axis_coords(&data.x, &data.y, &data.z, axis);
    let symmetry_nodes: HashSet<usize> = find_symmetry_plane_nodes(axis_coords, axis, tolerance)
        .into_iter()
        .collect();

    if verbose {
        println!(
            "  Found {} nodes on symmetry plane (tolerance: {})",
            symmetry_nodes.len(),
            tolerance
        );
    }

    if symmetry_nodes.is_empty() {
        eprintln!(
            "WARNING: No nodes found on the symmetry plane (axis={:?}, tolerance={}).",
            axis, tolerance
        );
        eprintln!("         Node merging will be skipped. Consider using a larger tolerance.");
    }

    // Build node mapping: original node i -> new node id
    // Original nodes: 1..N (1-based in Exodus)
    // Mirrored nodes: start at N+1, but skip symmetry plane nodes
    let mut mirror_node_map: HashMap<usize, i64> = HashMap::new();
    let mut next_mirror_id = (orig_num_nodes + 1) as i64;

    for i in 0..orig_num_nodes {
        if symmetry_nodes.contains(&i) {
            // Symmetry plane node: maps to itself (1-based)
            mirror_node_map.insert(i, (i + 1) as i64);
        } else {
            // Non-symmetry node: gets a new ID
            mirror_node_map.insert(i, next_mirror_id);
            next_mirror_id += 1;
        }
    }

    let num_new_nodes = (next_mirror_id - 1) as usize;
    let num_mirror_nodes = num_new_nodes - orig_num_nodes;

    if verbose {
        println!(
            "  New mesh: {} nodes ({} original + {} mirrored)",
            num_new_nodes, orig_num_nodes, num_mirror_nodes
        );
    }

    // Create new coordinates
    let mut new_x = data.x.clone();
    let mut new_y = data.y.clone();
    let mut new_z = data.z.clone();

    // Add mirrored coordinates (for non-symmetry nodes)
    for i in 0..orig_num_nodes {
        if !symmetry_nodes.contains(&i) {
            let mx = if matches!(axis, Axis::X) {
                -data.x[i]
            } else {
                data.x[i]
            };
            let my = if matches!(axis, Axis::Y) {
                -data.y[i]
            } else {
                data.y[i]
            };
            let mz = if matches!(axis, Axis::Z) {
                -data.z[i]
            } else {
                data.z[i]
            };
            new_x.push(mx);
            new_y.push(my);
            new_z.push(mz);
        }
    }

    // Create mirrored element blocks with connectivity
    let mut new_blocks = data.blocks.clone();
    let mut new_connectivities = data.connectivities.clone();
    let mut new_block_names = data.block_names.clone();

    // Find max block ID to assign new IDs for mirrored blocks
    let max_block_id = data.blocks.iter().map(|b| b.id).max().unwrap_or(0);
    let mut next_block_id = max_block_id + 1;

    for (block_idx, (block, connectivity)) in data
        .blocks
        .iter()
        .zip(data.connectivities.iter())
        .enumerate()
    {
        let nodes_per_elem = block.num_nodes_per_entry;
        let permutation = get_mirror_permutation(&block.topology, axis).ok_or_else(|| {
            TransformError::InvalidFormat(format!("Unsupported topology: {}", block.topology))
        })?;

        // Create mirrored connectivity for this block
        let mut mirror_connectivity = Vec::new();

        for elem_idx in 0..block.num_entries {
            let elem_start = elem_idx * nodes_per_elem;

            // Get original element nodes
            let orig_elem: Vec<i64> =
                connectivity[elem_start..elem_start + nodes_per_elem].to_vec();

            // Create mirrored element with permuted node order
            let mut mirror_elem = vec![0i64; nodes_per_elem];
            for (new_pos, &old_pos) in permutation.iter().enumerate() {
                let orig_node_id = orig_elem[old_pos] as usize - 1; // 0-based
                mirror_elem[new_pos] = mirror_node_map[&orig_node_id];
            }

            mirror_connectivity.extend(mirror_elem);
        }

        // Create mirrored block
        let mut mirror_block = block.clone();
        mirror_block.id = next_block_id;

        new_blocks.push(mirror_block);
        new_connectivities.push(mirror_connectivity);

        // Create name with _mirror suffix
        let default_name = format!("block_{}", block.id);
        let orig_name = data
            .block_names
            .get(block_idx)
            .filter(|s| !s.is_empty())
            .map(|s| s.as_str())
            .unwrap_or(&default_name);
        new_block_names.push(format!("{}_mirror", orig_name));

        if verbose {
            println!(
                "  Created mirrored block {} from block {} ({} elements)",
                next_block_id, block.id, block.num_entries
            );
        }

        next_block_id += 1;
    }

    if verbose {
        println!(
            "  New mesh: {} elements ({} original + {} mirrored) in {} blocks",
            orig_num_elems * 2,
            orig_num_elems,
            orig_num_elems,
            new_blocks.len()
        );
    }

    // Create mirrored node sets with _mirror suffix
    let mut new_node_sets = data.node_sets.clone();
    let mut new_node_set_names = data.node_set_names.clone();

    // Find next available set ID
    let max_ns_id = data
        .node_sets
        .iter()
        .map(|(id, _, _)| *id)
        .max()
        .unwrap_or(0);
    let mut next_ns_id = max_ns_id + 1;

    for (idx, (orig_id, nodes, dist_factors)) in data.node_sets.iter().enumerate() {
        // Create mirrored node set
        let mirror_nodes: Vec<i64> = nodes
            .iter()
            .map(|&n| {
                let orig_idx = (n - 1) as usize;
                mirror_node_map[&orig_idx]
            })
            .collect();

        // Distribution factors are copied as-is
        let mirror_df = dist_factors.clone();

        new_node_sets.push((next_ns_id, mirror_nodes, mirror_df));

        // Create name with _mirror suffix
        let default_name = format!("nodeset_{}", orig_id);
        let orig_name = data
            .node_set_names
            .get(idx)
            .map(|s| s.as_str())
            .unwrap_or(&default_name);
        new_node_set_names.push(format!("{}_mirror", orig_name));

        next_ns_id += 1;
    }

    // Create mirrored side sets with _mirror suffix
    let mut new_side_sets = data.side_sets.clone();
    let mut new_side_set_names = data.side_set_names.clone();

    let max_ss_id = data
        .side_sets
        .iter()
        .map(|(id, _, _, _)| *id)
        .max()
        .unwrap_or(0);
    let mut next_ss_id = max_ss_id + 1;

    for (idx, (orig_id, elements, sides, dist_factors)) in data.side_sets.iter().enumerate() {
        // Mirrored elements have IDs offset by orig_num_elems
        let mirror_elements: Vec<i64> = elements
            .iter()
            .map(|&e| e + orig_num_elems as i64)
            .collect();

        // Side numbers need adjustment based on topology and axis
        // For now, keep same side numbers (this is a simplification)
        // TODO: Implement proper side number mapping for different topologies
        let mirror_sides = sides.clone();

        let mirror_df = dist_factors.clone();

        new_side_sets.push((next_ss_id, mirror_elements, mirror_sides, mirror_df));

        let default_name = format!("sideset_{}", orig_id);
        let orig_name = data
            .side_set_names
            .get(idx)
            .map(|s| s.as_str())
            .unwrap_or(&default_name);
        new_side_set_names.push(format!("{}_mirror", orig_name));

        next_ss_id += 1;
    }

    // Create mirrored nodal variable values
    let mut new_nodal_var_values: Vec<Vec<Vec<f64>>> = Vec::new();

    for (var_idx, var_name) in data.nodal_var_names.iter().enumerate() {
        let is_mirror_component = is_vector_component(var_name, axis);

        let mut var_time_series = Vec::new();
        for step in 0..data.times.len() {
            let orig_values = &data.nodal_var_values[var_idx][step];
            let mut new_values = orig_values.clone();

            // Add mirrored values
            for (i, &val) in orig_values.iter().enumerate().take(orig_num_nodes) {
                if !symmetry_nodes.contains(&i) {
                    let mirror_val = if is_mirror_component {
                        -val // Negate vector component
                    } else {
                        val
                    };
                    new_values.push(mirror_val);
                }
            }

            var_time_series.push(new_values);
        }
        new_nodal_var_values.push(var_time_series);

        if verbose && is_mirror_component {
            println!("  Negating vector component: {}", var_name);
        }
    }

    // Create mirrored element variable values
    // Structure: [block_idx][var_idx][time_step][elem_idx]
    // After mirroring, we have original blocks followed by mirrored blocks
    let mut new_elem_var_values: Vec<Vec<Vec<Vec<f64>>>> = Vec::new();

    // First, keep original block values unchanged
    for block_vars in &data.elem_var_values {
        new_elem_var_values.push(block_vars.clone());
    }

    // Then, add mirrored block values (duplicating original values with vector negation)
    for (block_idx, block_vars) in data.elem_var_values.iter().enumerate() {
        let mut mirror_block_vars: Vec<Vec<Vec<f64>>> = Vec::new();

        for (var_idx, var_time_series) in block_vars.iter().enumerate() {
            let var_name = data
                .elem_var_names
                .get(var_idx)
                .map(|s| s.as_str())
                .unwrap_or("");
            let is_mirror_component = is_vector_component(var_name, axis);

            let mut mirror_var_time_series: Vec<Vec<f64>> = Vec::new();
            for step_values in var_time_series {
                let mirror_values: Vec<f64> = if is_mirror_component {
                    step_values.iter().map(|&v| -v).collect()
                } else {
                    step_values.clone()
                };
                mirror_var_time_series.push(mirror_values);
            }
            mirror_block_vars.push(mirror_var_time_series);

            if verbose && is_mirror_component && block_idx == 0 {
                println!("  Negating element vector component: {}", var_name);
            }
        }
        new_elem_var_values.push(mirror_block_vars);
    }

    // Create new params
    let mut new_params = data.params.clone();
    new_params.num_nodes = num_new_nodes;
    new_params.num_elems = orig_num_elems * 2;
    new_params.num_elem_blocks = new_blocks.len();
    new_params.num_node_sets = new_node_sets.len();
    new_params.num_side_sets = new_side_sets.len();

    Ok(MeshData {
        params: new_params,
        x: new_x,
        y: new_y,
        z: new_z,
        blocks: new_blocks,
        connectivities: new_connectivities,
        block_names: new_block_names,
        node_sets: new_node_sets,
        side_sets: new_side_sets,
        nodal_var_names: data.nodal_var_names.clone(),
        nodal_var_values: new_nodal_var_values,
        elem_var_names: data.elem_var_names.clone(),
        elem_var_values: new_elem_var_values,
        global_var_names: data.global_var_names.clone(),
        global_var_values: data.global_var_values.clone(),
        times: data.times.clone(),
        node_set_names: new_node_set_names,
        side_set_names: new_side_set_names,
    })
}

/// Write mesh data to a new file
fn write_mesh_data(path: &PathBuf, data: &MeshData, verbose: bool) -> Result<()> {
    use exodus_rs::types::CreateOptions;

    if verbose {
        println!(
            "  Writing output: {} nodes, {} elements",
            data.params.num_nodes, data.params.num_elems
        );
    }

    // Create new file with clobber mode
    let options = CreateOptions {
        mode: CreateMode::Clobber,
        ..Default::default()
    };
    let mut file = ExodusFile::create(path, options)?;

    // Initialize with params
    file.init(&data.params)?;

    // Write coordinates
    let y_opt = if data.params.num_dim >= 2 {
        Some(&data.y[..])
    } else {
        None
    };
    let z_opt = if data.params.num_dim >= 3 {
        Some(&data.z[..])
    } else {
        None
    };
    file.put_coords(&data.x, y_opt, z_opt)?;

    // Write all element blocks
    for (idx, (block, connectivity)) in data
        .blocks
        .iter()
        .zip(data.connectivities.iter())
        .enumerate()
    {
        file.put_block(block)?;
        file.put_connectivity(block.id, connectivity)?;

        // Write block name if available
        if let Some(name) = data.block_names.get(idx) {
            if !name.is_empty() {
                file.put_name(EntityType::ElemBlock, idx, name)?;
            }
        }
    }

    // Write node sets
    for (idx, (set_id, nodes, dist_factors)) in data.node_sets.iter().enumerate() {
        let df_opt = if dist_factors.is_empty() {
            None
        } else {
            Some(&dist_factors[..])
        };
        file.put_node_set(*set_id, nodes, df_opt)?;

        // Write name if available
        if let Some(name) = data.node_set_names.get(idx) {
            if !name.is_empty() {
                file.put_name(EntityType::NodeSet, idx, name)?;
            }
        }
    }

    // Write side sets
    for (idx, (set_id, elements, sides, dist_factors)) in data.side_sets.iter().enumerate() {
        let df_opt = if dist_factors.is_empty() {
            None
        } else {
            Some(&dist_factors[..])
        };
        file.put_side_set(*set_id, elements, sides, df_opt)?;

        if let Some(name) = data.side_set_names.get(idx) {
            if !name.is_empty() {
                file.put_name(EntityType::SideSet, idx, name)?;
            }
        }
    }

    // Write time steps and variables
    if !data.times.is_empty() {
        if verbose {
            println!(
                "  Writing {} time steps with {} nodal vars, {} elem vars",
                data.times.len(),
                data.nodal_var_names.len(),
                data.elem_var_names.len()
            );
            println!(
                "  Elem var values array: {} blocks",
                data.elem_var_values.len()
            );
        }

        // Define nodal variables
        if !data.nodal_var_names.is_empty() {
            let names: Vec<&str> = data.nodal_var_names.iter().map(|s| s.as_str()).collect();
            file.define_variables(EntityType::Nodal, &names)?;
        }

        // Define global variables
        if !data.global_var_names.is_empty() {
            let names: Vec<&str> = data.global_var_names.iter().map(|s| s.as_str()).collect();
            file.define_variables(EntityType::Global, &names)?;
        }

        // Define element block variables
        if !data.elem_var_names.is_empty() {
            let names: Vec<&str> = data.elem_var_names.iter().map(|s| s.as_str()).collect();
            file.define_variables(EntityType::ElemBlock, &names)?;

            // Write truth table (all blocks have all variables)
            let truth_table = TruthTable::new(
                EntityType::ElemBlock,
                data.blocks.len(),
                data.elem_var_names.len(),
            );
            file.put_truth_table(EntityType::ElemBlock, &truth_table)?;

            if verbose {
                println!(
                    "  Wrote elem_var_tab: {} blocks x {} vars",
                    data.blocks.len(),
                    data.elem_var_names.len()
                );
            }
        }

        // Write time values and variable data
        for (step, &time) in data.times.iter().enumerate() {
            file.put_time(step, time)?;

            // Write nodal variables
            for (var_idx, _) in data.nodal_var_names.iter().enumerate() {
                let values = &data.nodal_var_values[var_idx][step];
                file.put_var(step, EntityType::Nodal, 0, var_idx, values)?;
            }

            // Write global variables
            if !data.global_var_values.is_empty() && !data.global_var_values[step].is_empty() {
                for (var_idx, value) in data.global_var_values[step].iter().enumerate() {
                    file.put_var(step, EntityType::Global, 0, var_idx, &[*value])?;
                }
            }

            // Write element block variables
            for (block_idx, block) in data.blocks.iter().enumerate() {
                if let Some(block_vars) = data.elem_var_values.get(block_idx) {
                    for (var_idx, var_time_series) in block_vars.iter().enumerate() {
                        if let Some(values) = var_time_series.get(step) {
                            if !values.is_empty() {
                                if verbose && step == 0 {
                                    println!(
                                        "    Writing {} values for elem var {} on block {} (id={})",
                                        values.len(),
                                        var_idx,
                                        block_idx,
                                        block.id
                                    );
                                }
                                file.put_var(
                                    step,
                                    EntityType::ElemBlock,
                                    block.id,
                                    var_idx,
                                    values,
                                )?;
                            } else if verbose && step == 0 {
                                println!(
                                    "    Skipping empty elem var {} on block {} (id={})",
                                    var_idx, block_idx, block.id
                                );
                            }
                        }
                    }
                } else if verbose && step == 0 {
                    println!("    No elem var data for block {}", block_idx);
                }
            }
        }
    } else if verbose {
        println!("  No time steps - skipping variable output");
    }

    file.sync()?;

    if verbose {
        println!("  Output written successfully");
    }

    Ok(())
}

/// Apply copy-mirror-merge operation (requires reading entire mesh and creating new file)
fn apply_copy_mirror_merge(
    input_path: &PathBuf,
    output_path: &PathBuf,
    axis: Axis,
    tolerance: f64,
    verbose: bool,
) -> Result<()> {
    if verbose {
        println!(
            "  Copy-mirror-merge about {:?} axis (tolerance: {})",
            axis, tolerance
        );
    }

    // Read input mesh
    let input_file = ExodusFile::<mode::Read>::open(input_path)?;
    let mesh_data = read_mesh_data(&input_file, verbose)?;
    drop(input_file);

    // Apply copy-mirror-merge
    let merged_data = copy_mirror_merge(&mesh_data, axis, tolerance, verbose)?;

    // Write output mesh
    write_mesh_data(output_path, &merged_data, verbose)?;

    Ok(())
}

// ============================================================================
// End Copy-Mirror-Merge Implementation
// ============================================================================

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

    // Build performance configuration from CLI options
    let perf_config = PerformanceOptions::from_cli(&cli);

    // Handle --man flag
    if cli.man {
        show_man_page()?;
        return Ok(());
    }

    // Handle --show-perf-config
    if cli.show_perf_config {
        println!("{}", perf_config);
        return Ok(());
    }

    // Apply HDF5 environment variables before opening any files
    // These must be set before the HDF5 library is initialized
    perf_config.apply_env_vars();

    // Unwrap input and output (guaranteed to be present due to required_unless_present_any)
    let input = cli.input.as_ref().unwrap();
    let output = cli.output.as_ref().unwrap();

    // Extract operations in command-line order
    let operations = extract_ordered_operations(&cli, cli.verbose)?;

    if cli.verbose {
        println!("Input:  {}", input.display());
        println!("Output: {}", output.display());
        println!("Operations to apply: {}", operations.len());
        println!();
        println!("{}", perf_config);
        println!();
    }

    // Check if any CopyMirrorMerge operations are present
    let has_cmm = operations
        .iter()
        .any(|op| matches!(op, Operation::CopyMirrorMerge(_, _)));

    if has_cmm {
        // Complex path: handle CopyMirrorMerge with pre/post operations
        // Split operations into groups: before CMM, CMM itself, after CMM
        let mut pre_cmm_ops = Vec::new();
        let mut cmm_op: Option<(Axis, f64)> = None;
        let mut post_cmm_ops = Vec::new();
        let mut found_cmm = false;

        for op in &operations {
            if let Operation::CopyMirrorMerge(axis, tolerance) = op {
                if found_cmm {
                    return Err(TransformError::InvalidFormat(
                        "Only one --copy-mirror-merge operation is supported per invocation"
                            .to_string(),
                    ));
                }
                cmm_op = Some((*axis, *tolerance));
                found_cmm = true;
            } else if found_cmm {
                post_cmm_ops.push(op.clone());
            } else {
                pre_cmm_ops.push(op.clone());
            }
        }

        let (cmm_axis, cmm_tolerance) = cmm_op.unwrap();

        // Step 1: Apply pre-CMM operations (if any)
        if !pre_cmm_ops.is_empty() {
            if cli.verbose {
                println!("Copying input file to output location...");
            }
            std::fs::copy(input, output)?;

            let mut file = ExodusFile::append(output)?;
            if cli.verbose {
                let params = file.init_params()?;
                println!(
                    "Mesh: {} nodes, {} elements, {} dimensions",
                    params.num_nodes, params.num_elems, params.num_dim
                );
                println!("Applying pre-merge transformations:");
            }

            for op in &pre_cmm_ops {
                apply_simple_operation(&mut file, op, cli.verbose)?;
            }
            file.sync()?;
            drop(file);

            // Step 2: Apply CopyMirrorMerge (reads from output, writes to output)
            if cli.verbose {
                println!("Applying copy-mirror-merge:");
            }
            apply_copy_mirror_merge(output, output, cmm_axis, cmm_tolerance, cli.verbose)?;
        } else {
            // No pre-CMM ops, apply CMM directly from input to output
            if cli.verbose {
                println!("Applying copy-mirror-merge:");
            }
            apply_copy_mirror_merge(input, output, cmm_axis, cmm_tolerance, cli.verbose)?;
        }

        // Step 3: Apply post-CMM operations (if any)
        if !post_cmm_ops.is_empty() {
            let mut file = ExodusFile::append(output)?;
            if cli.verbose {
                println!("Applying post-merge transformations:");
            }
            for op in &post_cmm_ops {
                apply_simple_operation(&mut file, op, cli.verbose)?;
            }
            file.sync()?;
        }

        // Apply time normalization if requested
        if cli.zero_time {
            let mut file = ExodusFile::append(output)?;
            if cli.verbose {
                println!("Normalizing time values:");
            }
            normalize_time(&mut file, cli.verbose)?;
        }
    } else {
        // Simple path: no CopyMirrorMerge, use existing approach
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
            apply_simple_operation(&mut file, op, cli.verbose)?;
        }

        // Apply time normalization if requested
        if cli.zero_time {
            if cli.verbose {
                println!("Normalizing time values:");
            }
            normalize_time(&mut file, cli.verbose)?;
        }
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

    #[test]
    fn test_is_prime() {
        assert!(is_prime(2));
        assert!(is_prime(3));
        assert!(is_prime(5));
        assert!(is_prime(7));
        assert!(is_prime(521));
        assert!(!is_prime(4));
        assert!(!is_prime(100));
        assert!(!is_prime(1));
    }

    #[test]
    fn test_next_prime() {
        assert_eq!(next_prime(1), 2);
        assert_eq!(next_prime(2), 2);
        assert_eq!(next_prime(3), 3);
        assert_eq!(next_prime(4), 5);
        assert_eq!(next_prime(520), 521);
    }

    #[test]
    fn test_performance_options_defaults() {
        // Create a mock CLI with no performance options set
        let cli = Cli {
            input: Some(PathBuf::from("input.exo")),
            output: Some(PathBuf::from("output.exo")),
            scale_len: vec![],
            mirror: vec![],
            translate: vec![],
            rotate: vec![],
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
        };

        let perf = PerformanceOptions::from_cli(&cli);

        // Check default preemption
        assert!((perf.preemption - 0.75).abs() < 0.001);

        // Check time chunk default (0 = no chunking)
        assert_eq!(perf.time_chunk_size, 0);

        // Check that cache size is reasonable (at least 1 MB)
        assert!(perf.cache_size >= 1024 * 1024);

        // Check that chunk sizes are reasonable
        assert!(perf.node_chunk_size >= 1000);
        assert!(perf.element_chunk_size >= 1000);
    }

    #[test]
    fn test_performance_options_custom() {
        let cli = Cli {
            input: Some(PathBuf::from("input.exo")),
            output: Some(PathBuf::from("output.exo")),
            scale_len: vec![],
            mirror: vec![],
            translate: vec![],
            rotate: vec![],
            scale_field: vec![],
            copy_mirror_merge: vec![],
            merge_tolerance: 0.001,
            zero_time: false,
            verbose: false,
            cache_size: Some(256),      // 256 MB
            preemption: Some(0.5),      // Balanced write/read
            node_chunk: Some(20000),    // 20k nodes
            element_chunk: Some(15000), // 15k elements
            time_chunk: Some(10),       // 10 time steps
            show_perf_config: false,
            man: false,
        };

        let perf = PerformanceOptions::from_cli(&cli);

        assert_eq!(perf.cache_size, 256 * 1024 * 1024);
        assert!((perf.preemption - 0.5).abs() < 0.001);
        assert_eq!(perf.node_chunk_size, 20000);
        assert_eq!(perf.element_chunk_size, 15000);
        assert_eq!(perf.time_chunk_size, 10);
    }

    #[test]
    fn test_preemption_clamping() {
        let mut cli = Cli {
            input: Some(PathBuf::from("input.exo")),
            output: Some(PathBuf::from("output.exo")),
            scale_len: vec![],
            mirror: vec![],
            translate: vec![],
            rotate: vec![],
            scale_field: vec![],
            copy_mirror_merge: vec![],
            merge_tolerance: 0.001,
            zero_time: false,
            verbose: false,
            cache_size: None,
            preemption: Some(1.5), // Out of range (should clamp to 1.0)
            node_chunk: None,
            element_chunk: None,
            time_chunk: None,
            show_perf_config: false,
            man: false,
        };

        let perf = PerformanceOptions::from_cli(&cli);
        assert!((perf.preemption - 1.0).abs() < 0.001);

        cli.preemption = Some(-0.5); // Out of range (should clamp to 0.0)
        let perf = PerformanceOptions::from_cli(&cli);
        assert!(perf.preemption.abs() < 0.001);
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
        // Simulate: exo-cfd-transform in.exo out.exo --translate 1,0,0 --rotate Z,90
        let args: Vec<String> = vec![
            "exo-cfd-transform",
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
        // Simulate: exo-cfd-transform in.exo out.exo --rotate Z,90 --translate 1,0,0
        let args: Vec<String> = vec![
            "exo-cfd-transform",
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
        // Simulate: exo-cfd-transform in.exo out.exo --translate 1,0,0 --rotate Z,90 --translate 2,0,0
        let args: Vec<String> = vec![
            "exo-cfd-transform",
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
        // Simulate: exo-cfd-transform in.exo out.exo --mirror x --translate 1,0,0 --scale-len 2 --rotate Z,90
        let args: Vec<String> = vec![
            "exo-cfd-transform",
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
        // Simulate: exo-cfd-transform in.exo out.exo --translate=1,0,0 --rotate=Z,90
        let args: Vec<String> = vec![
            "exo-cfd-transform",
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
        // Simulate: exo-cfd-transform in.exo out.exo --rotate=Z,90 --translate=1,0,0
        let args: Vec<String> = vec![
            "exo-cfd-transform",
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
    fn test_scale_field_operation_order() {
        // Test that scale-field operations are ordered correctly
        let args: Vec<String> = vec![
            "exo-cfd-transform",
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
            "exo-cfd-transform",
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
            "exo-cfd-transform",
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
            "exo-cfd-transform",
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
    fn test_scale_field_scientific_notation() {
        // Test scale factors in scientific notation
        let result = parse_scale_field("temperature,1.5e-3").unwrap();
        assert_eq!(result.0, "temperature");
        assert!((result.1 - 1.5e-3).abs() < 1e-10);

        let result = parse_scale_field("pressure,2.5E+2").unwrap();
        assert_eq!(result.0, "pressure");
        assert!((result.1 - 2.5e2).abs() < 1e-10);
    }

    // =========================================================================
    // Copy-Mirror-Merge Tests
    // =========================================================================

    #[test]
    fn test_get_mirror_permutation_hex8() {
        // Test HEX8 permutations for all axes
        let perm_x = get_mirror_permutation("HEX8", Axis::X).unwrap();
        assert_eq!(perm_x, vec![1, 0, 3, 2, 5, 4, 7, 6]);

        let perm_y = get_mirror_permutation("HEX8", Axis::Y).unwrap();
        assert_eq!(perm_y, vec![3, 2, 1, 0, 7, 6, 5, 4]);

        let perm_z = get_mirror_permutation("HEX8", Axis::Z).unwrap();
        assert_eq!(perm_z, vec![4, 5, 6, 7, 0, 1, 2, 3]);

        // Also test lowercase variant
        let perm_hex = get_mirror_permutation("hex", Axis::X).unwrap();
        assert_eq!(perm_hex, vec![1, 0, 3, 2, 5, 4, 7, 6]);
    }

    #[test]
    fn test_get_mirror_permutation_tet4() {
        let perm = get_mirror_permutation("TET4", Axis::X).unwrap();
        assert_eq!(perm, vec![0, 2, 1, 3]);

        // Same permutation for all axes (swapping 2 nodes reverses orientation)
        assert_eq!(
            get_mirror_permutation("TET4", Axis::Y).unwrap(),
            get_mirror_permutation("TET4", Axis::Z).unwrap()
        );
    }

    #[test]
    fn test_get_mirror_permutation_unsupported() {
        assert!(get_mirror_permutation("HEX27", Axis::X).is_none());
        assert!(get_mirror_permutation("UNKNOWN", Axis::X).is_none());
    }

    #[test]
    fn test_is_vector_component() {
        // Test X-axis vector components
        assert!(is_vector_component("velocity_x", Axis::X));
        assert!(is_vector_component("u", Axis::X));
        assert!(is_vector_component("displacement_x", Axis::X));
        assert!(!is_vector_component("velocity_y", Axis::X));
        assert!(!is_vector_component("temperature", Axis::X));

        // Test Y-axis vector components
        assert!(is_vector_component("velocity_y", Axis::Y));
        assert!(is_vector_component("v", Axis::Y));
        assert!(!is_vector_component("velocity_x", Axis::Y));

        // Test Z-axis vector components
        assert!(is_vector_component("velocity_z", Axis::Z));
        assert!(is_vector_component("w", Axis::Z));
        assert!(!is_vector_component("velocity_x", Axis::Z));
    }

    #[test]
    fn test_find_symmetry_plane_nodes() {
        let coords = vec![0.0, 0.5, 1.0, 0.0, 0.5, 1.0, 0.001, -0.0005];
        let tolerance = 0.01;

        let sym_nodes = find_symmetry_plane_nodes(&coords, Axis::X, tolerance);

        // Nodes at indices 0, 3, 6, 7 should be on symmetry plane
        assert!(sym_nodes.contains(&0)); // 0.0
        assert!(sym_nodes.contains(&3)); // 0.0
        assert!(sym_nodes.contains(&6)); // 0.001 (within tolerance)
        assert!(sym_nodes.contains(&7)); // -0.0005 (within tolerance)
        assert!(!sym_nodes.contains(&1)); // 0.5
        assert!(!sym_nodes.contains(&2)); // 1.0
    }

    #[test]
    fn test_copy_mirror_merge_operation_parsing() {
        // Simulate: exo-cfd-transform in.exo out.exo --copy-mirror-merge x
        let args: Vec<String> = vec![
            "exo-cfd-transform",
            "in.exo",
            "out.exo",
            "--copy-mirror-merge",
            "x",
        ]
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
        // Simulate: exo-cfd-transform in.exo out.exo --translate 1,0,0 --copy-mirror-merge x --rotate Z,90
        let args: Vec<String> = vec![
            "exo-cfd-transform",
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
        // Simulate: exo-cfd-transform in.exo out.exo --copy-mirror-merge=y
        let args: Vec<String> = vec![
            "exo-cfd-transform",
            "in.exo",
            "out.exo",
            "--copy-mirror-merge=y",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        let cli =
            make_test_cli_with_cmm(vec![], vec![], vec![], vec![], vec!["y".to_string()], 0.001);

        let ops = extract_ordered_operations_from_args(&args, &cli, false).unwrap();

        assert_eq!(ops.len(), 1);
        assert!(matches!(ops[0], Operation::CopyMirrorMerge(Axis::Y, _)));
    }

    #[test]
    fn test_hex8_winding_order_consistency() {
        // Verify that the HEX8 permutation maintains valid element structure
        // by checking that swapped pairs are consistent
        let perm_x = get_mirror_permutation("HEX8", Axis::X).unwrap();

        // For X-axis mirror, we expect pairs to be swapped:
        // (0,1), (2,3), (4,5), (6,7)
        assert_eq!(perm_x[0], 1);
        assert_eq!(perm_x[1], 0);
        assert_eq!(perm_x[2], 3);
        assert_eq!(perm_x[3], 2);
        assert_eq!(perm_x[4], 5);
        assert_eq!(perm_x[5], 4);
        assert_eq!(perm_x[6], 7);
        assert_eq!(perm_x[7], 6);
    }
}
