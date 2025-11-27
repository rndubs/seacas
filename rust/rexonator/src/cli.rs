//! CLI argument parsing and types for rexonator
//!
//! This module defines the command-line interface for rexonator,
//! including all argument parsing, error types, and transformation operations.

use clap::Parser;
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

pub type Result<T> = std::result::Result<T, TransformError>;

/// Transform Exodus mesh files with translation, rotation, scaling, and mirroring.
///
/// Transformations are applied in the order they appear on the command line.
/// For example: `--translate 1,0,0 --rotate "Z,90" --scale-len 2` will first
/// translate, then rotate, then scale.
#[derive(Parser, Debug)]
#[command(name = "rexonator")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Input Exodus file
    #[arg(value_name = "INPUT", required_unless_present_any = ["man", "show_perf_config"])]
    pub input: Option<PathBuf>,

    /// Output Exodus file
    #[arg(value_name = "OUTPUT", required_unless_present_any = ["man", "show_perf_config"])]
    pub output: Option<PathBuf>,

    /// Scale mesh coordinates uniformly by a factor
    #[arg(long = "scale-len", value_name = "FACTOR")]
    pub scale_len: Vec<f64>,

    /// Mirror (reflect) mesh about an axis (x, y, or z)
    #[arg(long, value_name = "AXIS")]
    pub mirror: Vec<String>,

    /// Translate mesh by x,y,z offset (e.g., "1.0,2.0,3.0")
    #[arg(long, value_name = "X,Y,Z")]
    pub translate: Vec<String>,

    /// Rotate mesh using Euler angles.
    /// Format: "SEQUENCE,angle1,angle2,angle3" where SEQUENCE is 1-3 axes.
    /// Uppercase (XYZ) = extrinsic (fixed frame), lowercase (xyz) = intrinsic (body frame).
    /// Examples: "Z,90" or "XYZ,30,45,60" or "zyx,10,20,30"
    #[arg(long, value_name = "SEQ,ANGLES")]
    pub rotate: Vec<String>,

    /// Scale individual field variables by name.
    /// Format: "field_name,scale_factor"
    /// Examples: "stress,1.23" or "temperature,0.5"
    /// Can be specified multiple times to scale different fields
    #[arg(long = "scale-field", value_name = "NAME,FACTOR")]
    pub scale_field: Vec<String>,

    /// Copy, mirror, and merge mesh about a symmetry plane.
    /// Creates a full model from a half-symmetry model by mirroring about the
    /// specified axis and merging nodes on the symmetry plane.
    /// Nodes within merge-tolerance of the symmetry plane (axis=0) are merged.
    #[arg(long = "copy-mirror-merge", value_name = "AXIS")]
    pub copy_mirror_merge: Vec<String>,

    /// Tolerance for merging nodes on the symmetry plane (default: 0.001)
    #[arg(
        long = "merge-tolerance",
        value_name = "VALUE",
        default_value = "0.001"
    )]
    pub merge_tolerance: f64,

    /// Normalize time values so the first time step is zero
    #[arg(short = 'z', long = "zero-time")]
    pub zero_time: bool,

    /// Print verbose output
    #[arg(short, long)]
    pub verbose: bool,

    // --- NetCDF5/HDF5 Performance Options ---
    /// HDF5 chunk cache size in megabytes.
    /// Larger cache improves I/O performance for large files.
    /// Default: auto-detected based on environment (4-128 MB).
    #[arg(long, value_name = "MB")]
    pub cache_size: Option<usize>,

    /// HDF5 cache preemption policy (0.0 to 1.0).
    /// 0.0 = favor write performance, 1.0 = favor read performance.
    /// Default: 0.75 (balanced).
    #[arg(long, value_name = "VALUE")]
    pub preemption: Option<f64>,

    /// Node chunk size: number of nodes per HDF5 chunk.
    /// Affects chunking for nodal data in new files.
    /// Default: 1,000-10,000 based on environment.
    #[arg(long, value_name = "SIZE")]
    pub node_chunk: Option<usize>,

    /// Element chunk size: number of elements per HDF5 chunk.
    /// Affects chunking for element data in new files.
    /// Default: 1,000-10,000 based on environment.
    #[arg(long, value_name = "SIZE")]
    pub element_chunk: Option<usize>,

    /// Time step chunk size: number of time steps per HDF5 chunk.
    /// 0 = no time chunking (mesh-oriented I/O, default).
    /// 1+ = chunk multiple time steps together.
    #[arg(long, value_name = "SIZE")]
    pub time_chunk: Option<usize>,

    /// Print performance configuration and exit
    #[arg(long)]
    pub show_perf_config: bool,

    /// Display the man page
    #[arg(long)]
    pub man: bool,
}

/// Represents a transformation operation
#[derive(Debug, Clone)]
pub enum Operation {
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
pub enum Axis {
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
