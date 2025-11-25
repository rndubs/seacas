//! exo-cfd-transform: CLI tool for transforming Exodus mesh files
//!
//! This tool applies geometric transformations to Exodus II mesh files,
//! including translation, rotation, scaling, and mirroring. Transformations
//! are applied in the order they appear on the command line.

use clap::Parser;
use exodus_rs::{transformations::rotation_matrix_from_euler, ExodusFile};
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

/// Check if an argument matches a flag (handles both "--flag" and "--flag=value" forms)
fn arg_matches_flag(arg: &str, flag: &str) -> bool {
    arg == flag || arg.starts_with(&format!("{}=", flag))
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
        if arg_matches_flag(arg, "--scale-len") && scale_idx < cli.scale_len.len() {
            operations.push((pos, Operation::ScaleLen(cli.scale_len[scale_idx])));
            scale_idx += 1;
        } else if arg_matches_flag(arg, "--mirror") && mirror_idx < cli.mirror.len() {
            let axis: Axis = cli.mirror[mirror_idx].parse()?;
            operations.push((pos, Operation::Mirror(axis)));
            mirror_idx += 1;
        } else if arg_matches_flag(arg, "--translate") && translate_idx < cli.translate.len() {
            let offset = parse_translate(&cli.translate[translate_idx])?;
            operations.push((pos, Operation::Translate(offset)));
            translate_idx += 1;
        } else if arg_matches_flag(arg, "--rotate") && rotate_idx < cli.rotate.len() {
            let (seq, angles) = parse_rotate(&cli.rotate[rotate_idx])?;
            operations.push((pos, Operation::Rotate(seq, angles)));
            rotate_idx += 1;
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
    let operations = extract_ordered_operations(&cli)?;

    if cli.verbose {
        println!("Input:  {}", input.display());
        println!("Output: {}", output.display());
        println!("Operations to apply: {}", operations.len());
        println!();
        println!("{}", perf_config);
        println!();
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
}
