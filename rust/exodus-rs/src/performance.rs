//! Performance tuning for HDF5/NetCDF I/O operations
//!
//! This module provides configuration options for optimizing I/O performance
//! on HPC systems, including chunk cache tuning and automatic node detection.

use std::env;

/// Type of compute node
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum NodeType {
    /// Login node (shared, limited resources)
    Login,
    /// Compute node (dedicated, full resources)
    Compute,
    /// Unknown/local development machine
    Unknown,
}

impl NodeType {
    /// Detect the current node type based on environment variables
    ///
    /// Checks for job scheduler environment variables:
    /// - SLURM_JOB_ID: Indicates running in a Slurm job
    /// - FLUX_URI: Indicates running in a Flux instance
    /// - PBS_JOBID: Indicates running in a PBS job
    /// - LSB_JOBID: Indicates running in an LSF job
    ///
    /// # Returns
    ///
    /// - `NodeType::Compute` if running inside a job scheduler
    /// - `NodeType::Login` if on a system with schedulers but not in a job
    /// - `NodeType::Unknown` if no scheduler detected
    pub fn detect() -> Self {
        // Check if we're inside a job
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

    /// Get recommended cache size in bytes for this node type
    ///
    /// # Returns
    ///
    /// - Login nodes: 4 MB (conservative for shared resources)
    /// - Compute nodes: 128 MB (aggressive for dedicated resources)
    /// - Unknown: 16 MB (moderate default)
    pub fn default_cache_size(&self) -> usize {
        match self {
            NodeType::Login => 4 * 1024 * 1024,     // 4 MB
            NodeType::Compute => 128 * 1024 * 1024, // 128 MB
            NodeType::Unknown => 16 * 1024 * 1024,  // 16 MB
        }
    }

    /// Get recommended chunk size for nodal data
    ///
    /// This determines how many nodes are stored per HDF5 chunk.
    /// Larger chunks reduce metadata overhead but increase memory usage.
    ///
    /// # Returns
    ///
    /// - Login nodes: 1,000 nodes per chunk
    /// - Compute nodes: 10,000 nodes per chunk
    /// - Unknown: 5,000 nodes per chunk
    pub fn default_chunk_nodes(&self) -> usize {
        match self {
            NodeType::Login => 1_000,
            NodeType::Compute => 10_000,
            NodeType::Unknown => 5_000,
        }
    }

    /// Get recommended chunk size for element data
    ///
    /// # Returns
    ///
    /// - Login nodes: 1,000 elements per chunk
    /// - Compute nodes: 10,000 elements per chunk
    /// - Unknown: 5,000 elements per chunk
    pub fn default_chunk_elements(&self) -> usize {
        match self {
            NodeType::Login => 1_000,
            NodeType::Compute => 10_000,
            NodeType::Unknown => 5_000,
        }
    }
}

/// HDF5 chunk cache configuration
///
/// The chunk cache significantly affects I/O performance. HDF5 reads and writes
/// data in chunks, and caching frequently-accessed chunks can provide dramatic
/// speedups (up to 1000x in some cases).
///
/// # Performance Guidelines
///
/// - **cache_size**: Should be large enough to hold frequently-accessed chunks.
///   On login nodes, keep this modest (4-16 MB). On compute nodes with large
///   memory (256+ GB), use 100-500 MB or more.
///
/// - **num_slots**: Hash table size for chunk lookup. Should be a prime number
///   ~100x the number of chunks that fit in cache_size. Use `auto_slots()` for
///   automatic calculation.
///
/// - **preemption**: Controls cache eviction policy (0.0-1.0). Default 0.75
///   works well for most cases. Lower values (0.0-0.5) favor write-heavy
///   workloads; higher values (0.75-1.0) favor read-heavy workloads.
///
/// # Example
///
/// ```rust,ignore
/// use exodus_rs::performance::CacheConfig;
///
/// // Conservative settings for login node
/// let cache = CacheConfig::new(4 * 1024 * 1024); // 4 MB
///
/// // Aggressive settings for compute node
/// let cache = CacheConfig::new(128 * 1024 * 1024)  // 128 MB
///     .with_preemption(0.5);  // Favor writes
/// ```
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Size of chunk cache in bytes
    pub cache_size: usize,

    /// Number of hash table slots (0 = auto-calculate)
    pub num_slots: usize,

    /// Chunk preemption policy (0.0 = favor writes, 1.0 = favor reads)
    pub preemption: f64,
}

impl CacheConfig {
    /// Create a new cache configuration with the specified size
    ///
    /// # Arguments
    ///
    /// * `cache_size` - Cache size in bytes
    ///
    /// # Returns
    ///
    /// A new `CacheConfig` with auto-calculated slots and default preemption (0.75)
    pub fn new(cache_size: usize) -> Self {
        Self {
            cache_size,
            num_slots: 0, // Auto-calculate
            preemption: 0.75,
        }
    }

    /// Set the number of hash slots manually
    ///
    /// If set to 0, slots will be auto-calculated based on cache size.
    /// Should be a prime number ~100x the number of chunks that fit in cache.
    pub fn with_slots(mut self, num_slots: usize) -> Self {
        self.num_slots = num_slots;
        self
    }

    /// Set the preemption policy
    ///
    /// # Arguments
    ///
    /// * `preemption` - Value between 0.0 and 1.0
    ///   - 0.0: Favor write performance (don't penalize write-only chunks)
    ///   - 0.75: Balanced (default)
    ///   - 1.0: Favor read performance (aggressively evict write-only chunks)
    pub fn with_preemption(mut self, preemption: f64) -> Self {
        self.preemption = preemption.clamp(0.0, 1.0);
        self
    }

    /// Calculate hash slots based on cache size and typical chunk size
    ///
    /// # Arguments
    ///
    /// * `typical_chunk_bytes` - Expected size of a chunk in bytes
    ///
    /// # Returns
    ///
    /// Recommended number of hash slots (prime number)
    pub fn auto_slots(&self, typical_chunk_bytes: usize) -> usize {
        if typical_chunk_bytes == 0 {
            return 521; // Default prime
        }

        let chunks_in_cache = self.cache_size / typical_chunk_bytes;
        let target = chunks_in_cache * 100;

        // Find nearest prime number
        Self::next_prime(target)
    }

    /// Find the next prime number >= n
    fn next_prime(n: usize) -> usize {
        if n <= 2 {
            return 2;
        }

        let mut candidate = if n % 2 == 0 { n + 1 } else { n };

        loop {
            if Self::is_prime(candidate) {
                return candidate;
            }
            candidate += 2;
        }
    }

    /// Check if a number is prime (simple trial division)
    fn is_prime(n: usize) -> bool {
        if n <= 1 {
            return false;
        }
        if n <= 3 {
            return true;
        }
        if n % 2 == 0 || n % 3 == 0 {
            return false;
        }

        let limit = (n as f64).sqrt() as usize;
        let mut i = 5;
        while i <= limit {
            if n % i == 0 || n % (i + 2) == 0 {
                return false;
            }
            i += 6;
        }
        true
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        let node_type = NodeType::detect();
        Self::new(node_type.default_cache_size())
    }
}

/// HDF5 chunk size configuration
///
/// Chunking determines how data is divided and stored in the HDF5 file.
/// Proper chunk sizing is critical for I/O performance.
///
/// # Performance Guidelines
///
/// - **Match access patterns**: If you write entire coordinate arrays at once,
///   use large chunks. If you access individual nodes, use smaller chunks.
///
/// - **Mesh-oriented defaults**: For exodus files without time series, chunk
///   by spatial dimensions (nodes/elements) rather than time.
///
/// - **Typical sizes**: 1,000-10,000 nodes per chunk works well for most cases.
///
/// - **Memory tradeoff**: Larger chunks reduce metadata overhead but require
///   more memory for the cache.
///
/// # Example
///
/// ```rust,ignore
/// use exodus_rs::performance::ChunkConfig;
///
/// // Default: auto-detect based on node type
/// let chunks = ChunkConfig::default();
///
/// // Manual: 10,000 nodes per chunk
/// let chunks = ChunkConfig::new()
///     .with_node_chunk_size(10_000);
/// ```
#[derive(Debug, Clone)]
pub struct ChunkConfig {
    /// Number of nodes per chunk (0 = auto-calculate)
    pub node_chunk_size: usize,

    /// Number of elements per chunk (0 = auto-calculate)
    pub element_chunk_size: usize,

    /// Chunk size for time steps (0 = no chunking on time dimension)
    /// For mesh-oriented I/O, this should typically be 0 or 1
    pub time_chunk_size: usize,
}

impl ChunkConfig {
    /// Create a new chunk configuration with defaults
    pub fn new() -> Self {
        let node_type = NodeType::detect();
        Self {
            node_chunk_size: node_type.default_chunk_nodes(),
            element_chunk_size: node_type.default_chunk_elements(),
            time_chunk_size: 0, // Don't chunk on time by default
        }
    }

    /// Set the node chunk size
    ///
    /// # Arguments
    ///
    /// * `size` - Number of nodes per chunk (0 = auto-calculate based on dataset size)
    pub fn with_node_chunk_size(mut self, size: usize) -> Self {
        self.node_chunk_size = size;
        self
    }

    /// Set the element chunk size
    ///
    /// # Arguments
    ///
    /// * `size` - Number of elements per chunk (0 = auto-calculate based on dataset size)
    pub fn with_element_chunk_size(mut self, size: usize) -> Self {
        self.element_chunk_size = size;
        self
    }

    /// Set the time step chunk size
    ///
    /// # Arguments
    ///
    /// * `size` - Number of time steps per chunk
    ///   - 0: No chunking on time dimension (default for mesh-oriented I/O)
    ///   - 1: Chunk each time step separately
    ///   - 10+: Chunk multiple time steps together (for time-series analysis)
    pub fn with_time_chunk_size(mut self, size: usize) -> Self {
        self.time_chunk_size = size;
        self
    }

    /// Calculate optimal chunk size based on dataset size
    ///
    /// Uses a heuristic to balance chunk size and number of chunks.
    ///
    /// # Arguments
    ///
    /// * `total_size` - Total number of nodes or elements
    /// * `target_chunk_bytes` - Target chunk size in bytes (default: 1 MB)
    ///
    /// # Returns
    ///
    /// Recommended chunk size (number of nodes/elements)
    pub fn calculate_optimal_chunk(total_size: usize, target_chunk_bytes: usize) -> usize {
        const BYTES_PER_COORD: usize = 8; // f64
        const DIMS: usize = 3; // x, y, z

        let bytes_per_node = BYTES_PER_COORD * DIMS;
        let chunk_size = target_chunk_bytes / bytes_per_node;

        // Clamp to reasonable range
        chunk_size.clamp(1000, 100_000).min(total_size)
    }
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Complete performance configuration for Exodus file I/O
///
/// Combines cache and chunk settings with automatic node detection.
///
/// # Example
///
/// ```rust,ignore
/// use exodus_rs::performance::PerformanceConfig;
///
/// // Use automatic detection (recommended)
/// let config = PerformanceConfig::auto();
///
/// // Or customize for your workload
/// let config = PerformanceConfig::auto()
///     .with_cache_size(256 * 1024 * 1024)  // 256 MB cache
///     .with_node_chunk_size(20_000);       // 20k nodes per chunk
///
/// // Apply to file creation
/// let mut options = CreateOptions::default();
/// options.performance = Some(config);
/// let file = ExodusFile::create("mesh.exo", options)?;
/// ```
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// Node type (auto-detected or manual)
    pub node_type: NodeType,

    /// Chunk cache configuration
    pub cache: CacheConfig,

    /// Chunk size configuration
    pub chunks: ChunkConfig,
}

impl PerformanceConfig {
    /// Create a performance configuration with automatic node detection
    ///
    /// This is the recommended way to create a performance config.
    /// It detects whether you're on a login or compute node and sets
    /// appropriate defaults.
    pub fn auto() -> Self {
        let node_type = NodeType::detect();
        Self {
            node_type,
            cache: CacheConfig::default(),
            chunks: ChunkConfig::default(),
        }
    }

    /// Create a performance configuration for a specific node type
    ///
    /// # Arguments
    ///
    /// * `node_type` - The type of node to optimize for
    pub fn for_node_type(node_type: NodeType) -> Self {
        Self {
            node_type,
            cache: CacheConfig::new(node_type.default_cache_size()),
            chunks: ChunkConfig {
                node_chunk_size: node_type.default_chunk_nodes(),
                element_chunk_size: node_type.default_chunk_elements(),
                time_chunk_size: 0,
            },
        }
    }

    /// Create a conservative configuration for shared resources
    ///
    /// Equivalent to `PerformanceConfig::for_node_type(NodeType::Login)`
    pub fn conservative() -> Self {
        Self::for_node_type(NodeType::Login)
    }

    /// Create an aggressive configuration for dedicated resources
    ///
    /// Equivalent to `PerformanceConfig::for_node_type(NodeType::Compute)`
    pub fn aggressive() -> Self {
        Self::for_node_type(NodeType::Compute)
    }

    /// Set the cache size in bytes
    pub fn with_cache_size(mut self, bytes: usize) -> Self {
        self.cache.cache_size = bytes;
        self
    }

    /// Set the cache size in megabytes (convenience method)
    pub fn with_cache_mb(self, mb: usize) -> Self {
        self.with_cache_size(mb * 1024 * 1024)
    }

    /// Set the cache preemption policy
    ///
    /// # Arguments
    ///
    /// * `preemption` - Value between 0.0 (favor writes) and 1.0 (favor reads)
    pub fn with_preemption(mut self, preemption: f64) -> Self {
        self.cache.preemption = preemption.clamp(0.0, 1.0);
        self
    }

    /// Set the node chunk size
    pub fn with_node_chunk_size(mut self, size: usize) -> Self {
        self.chunks.node_chunk_size = size;
        self
    }

    /// Set the element chunk size
    pub fn with_element_chunk_size(mut self, size: usize) -> Self {
        self.chunks.element_chunk_size = size;
        self
    }

    /// Set the time step chunk size
    pub fn with_time_chunk_size(mut self, size: usize) -> Self {
        self.chunks.time_chunk_size = size;
        self
    }

    /// Get a summary of the configuration
    pub fn summary(&self) -> String {
        format!(
            "Performance Config:\n\
             - Node Type: {:?}\n\
             - Cache Size: {} MB\n\
             - Cache Slots: {} (auto)\n\
             - Cache Preemption: {:.2}\n\
             - Node Chunk Size: {} nodes\n\
             - Element Chunk Size: {} elements\n\
             - Time Chunk Size: {} steps",
            self.node_type,
            self.cache.cache_size / (1024 * 1024),
            self.cache.num_slots,
            self.cache.preemption,
            self.chunks.node_chunk_size,
            self.chunks.element_chunk_size,
            self.chunks.time_chunk_size
        )
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self::auto()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_type_detection() {
        // Detection should always return something
        let node_type = NodeType::detect();
        assert!(matches!(
            node_type,
            NodeType::Login | NodeType::Compute | NodeType::Unknown
        ));
    }

    #[test]
    fn test_cache_config_defaults() {
        let cache = CacheConfig::default();
        assert!(cache.cache_size > 0);
        assert_eq!(cache.preemption, 0.75);
    }

    #[test]
    fn test_cache_config_builder() {
        let cache = CacheConfig::new(32 * 1024 * 1024)
            .with_slots(10007)
            .with_preemption(0.5);

        assert_eq!(cache.cache_size, 32 * 1024 * 1024);
        assert_eq!(cache.num_slots, 10007);
        assert_eq!(cache.preemption, 0.5);
    }

    #[test]
    fn test_prime_calculation() {
        assert!(CacheConfig::is_prime(2));
        assert!(CacheConfig::is_prime(3));
        assert!(CacheConfig::is_prime(5));
        assert!(CacheConfig::is_prime(7));
        assert!(CacheConfig::is_prime(521));
        assert!(!CacheConfig::is_prime(4));
        assert!(!CacheConfig::is_prime(100));
    }

    #[test]
    fn test_chunk_config_defaults() {
        let chunks = ChunkConfig::default();
        assert!(chunks.node_chunk_size > 0);
        assert!(chunks.element_chunk_size > 0);
        assert_eq!(chunks.time_chunk_size, 0); // Default: no time chunking
    }

    #[test]
    fn test_chunk_config_builder() {
        let chunks = ChunkConfig::new()
            .with_node_chunk_size(15000)
            .with_element_chunk_size(12000)
            .with_time_chunk_size(10);

        assert_eq!(chunks.node_chunk_size, 15000);
        assert_eq!(chunks.element_chunk_size, 12000);
        assert_eq!(chunks.time_chunk_size, 10);
    }

    #[test]
    fn test_performance_config_auto() {
        let config = PerformanceConfig::auto();
        assert!(config.cache.cache_size > 0);
        assert!(config.chunks.node_chunk_size > 0);
    }

    #[test]
    fn test_performance_config_conservative() {
        let config = PerformanceConfig::conservative();
        assert_eq!(config.node_type, NodeType::Login);
        assert_eq!(config.cache.cache_size, 4 * 1024 * 1024);
        assert_eq!(config.chunks.node_chunk_size, 1_000);
    }

    #[test]
    fn test_performance_config_aggressive() {
        let config = PerformanceConfig::aggressive();
        assert_eq!(config.node_type, NodeType::Compute);
        assert_eq!(config.cache.cache_size, 128 * 1024 * 1024);
        assert_eq!(config.chunks.node_chunk_size, 10_000);
    }

    #[test]
    fn test_performance_config_builder() {
        let config = PerformanceConfig::auto()
            .with_cache_mb(256)
            .with_node_chunk_size(20_000)
            .with_preemption(0.3);

        assert_eq!(config.cache.cache_size, 256 * 1024 * 1024);
        assert_eq!(config.chunks.node_chunk_size, 20_000);
        assert_eq!(config.cache.preemption, 0.3);
    }

    #[test]
    fn test_optimal_chunk_calculation() {
        // For a dataset with 100k nodes, 1MB target
        let chunk_size = ChunkConfig::calculate_optimal_chunk(100_000, 1024 * 1024);
        assert!(chunk_size >= 1000);
        assert!(chunk_size <= 100_000);
    }

    #[test]
    fn test_node_type_defaults() {
        let login = NodeType::Login;
        assert_eq!(login.default_cache_size(), 4 * 1024 * 1024);
        assert_eq!(login.default_chunk_nodes(), 1_000);

        let compute = NodeType::Compute;
        assert_eq!(compute.default_cache_size(), 128 * 1024 * 1024);
        assert_eq!(compute.default_chunk_nodes(), 10_000);
    }
}
