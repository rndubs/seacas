use crate::cli::Cli;
use std::env;
use std::fmt;

/// Node type detected from environment variables
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    /// Compute node (inside a job scheduler)
    Compute,
    /// Login node (has scheduler but not in a job)
    Login,
    /// Unknown node type
    Unknown,
}

impl fmt::Display for NodeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeType::Compute => write!(f, "compute"),
            NodeType::Login => write!(f, "login"),
            NodeType::Unknown => write!(f, "unknown"),
        }
    }
}

/// Detect the node type based on environment variables
pub fn detect_node_type() -> NodeType {
    // Check if we're inside a job scheduler
    if env::var("SLURM_JOB_ID").is_ok()
        || env::var("FLUX_URI").is_ok()
        || env::var("PBS_JOBID").is_ok()
        || env::var("LSB_JOBID").is_ok()
    {
        return NodeType::Compute;
    }

    // Check if we're on a system with schedulers (but not in a job)
    if env::var("SLURM_CONF").is_ok()
        || env::var("FLUX_EXEC_PATH").is_ok()
        || env::var("PBS_SERVER").is_ok()
        || env::var("LSF_ENVDIR").is_ok()
    {
        return NodeType::Login;
    }

    NodeType::Unknown
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

/// Performance configuration for HDF5/NetCDF I/O
#[derive(Debug, Clone)]
pub struct PerformanceOptions {
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
    /// Convert to exodus_rs::PerformanceConfig for file operations
    ///
    /// This allows the CLI performance settings to be used when creating
    /// new exodus files during operations like copy-mirror-merge.
    pub fn to_exodus_config(&self) -> exodus_rs::PerformanceConfig {
        exodus_rs::PerformanceConfig::auto()
            .with_cache_size(self.cache_size)
            .with_preemption(self.preemption)
            .with_node_chunk_size(self.node_chunk_size)
            .with_element_chunk_size(self.element_chunk_size)
            .with_time_chunk_size(self.time_chunk_size)
    }

    /// Build performance options from CLI arguments
    pub fn from_cli(cli: &Cli) -> Self {
        // Start with auto-detected defaults based on environment
        let node_type = detect_node_type();
        let (default_cache, default_chunk) = match node_type {
            NodeType::Compute => (128 * 1024 * 1024, 10_000), // 128 MB, 10k nodes/elements
            NodeType::Login => (4 * 1024 * 1024, 1_000),      // 4 MB, 1k nodes/elements
            NodeType::Unknown => (16 * 1024 * 1024, 5_000),   // 16 MB, 5k nodes/elements
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
    pub fn apply_env_vars(&self) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

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
            vector_fields: None,
            scalar_fields: None,
            no_auto_vector_detection: false,
            zero_time: false,
            in_place: false,
            verbose: false,
            cache_size: None,
            preemption: None,
            node_chunk: None,
            element_chunk: None,
            time_chunk: None,
            show_perf_config: false,
            man: false,
            dry_run: false,
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
            vector_fields: None,
            scalar_fields: None,
            no_auto_vector_detection: false,
            zero_time: false,
            in_place: false,
            verbose: false,
            cache_size: Some(256),      // 256 MB
            preemption: Some(0.5),      // Balanced write/read
            node_chunk: Some(20000),    // 20k nodes
            element_chunk: Some(15000), // 15k elements
            time_chunk: Some(10),       // 10 time steps
            show_perf_config: false,
            man: false,
            dry_run: false,
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
            vector_fields: None,
            scalar_fields: None,
            no_auto_vector_detection: false,
            zero_time: false,
            in_place: false,
            verbose: false,
            cache_size: None,
            preemption: Some(1.5), // Out of range (should clamp to 1.0)
            node_chunk: None,
            element_chunk: None,
            time_chunk: None,
            show_perf_config: false,
            man: false,
            dry_run: false,
        };

        let perf = PerformanceOptions::from_cli(&cli);
        assert!((perf.preemption - 1.0).abs() < 0.001);

        cli.preemption = Some(-0.5); // Out of range (should clamp to 0.0)
        let perf = PerformanceOptions::from_cli(&cli);
        assert!(perf.preemption.abs() < 0.001);
    }
}
