//! Python bindings for performance configuration

use pyo3::prelude::*;
use exodus_rs::performance as rust_perf;

/// Node type detection for HPC environments
///
/// Automatically detects whether code is running on:
/// - Compute node (inside a job scheduler)
/// - Login node (on HPC system but not in a job)
/// - Unknown/local development machine
#[pyclass(name = "NodeType")]
#[derive(Clone, Debug)]
pub struct PyNodeType {
    pub(crate) inner: rust_perf::NodeType,
}

#[pymethods]
impl PyNodeType {
    /// Detect the current node type
    ///
    /// Returns:
    ///     NodeType: Detected node type (Compute, Login, or Unknown)
    ///
    /// Example:
    ///     >>> node_type = NodeType.detect()
    ///     >>> print(node_type)
    ///     NodeType.Compute
    #[staticmethod]
    fn detect() -> Self {
        Self {
            inner: rust_perf::NodeType::detect(),
        }
    }

    /// Create a Compute node type
    #[staticmethod]
    fn compute() -> Self {
        Self {
            inner: rust_perf::NodeType::Compute,
        }
    }

    /// Create a Login node type
    #[staticmethod]
    fn login() -> Self {
        Self {
            inner: rust_perf::NodeType::Login,
        }
    }

    /// Create an Unknown node type
    #[staticmethod]
    fn unknown() -> Self {
        Self {
            inner: rust_perf::NodeType::Unknown,
        }
    }

    /// Get default cache size for this node type (in bytes)
    fn default_cache_size(&self) -> usize {
        self.inner.default_cache_size()
    }

    /// Get default chunk size for nodal data
    fn default_chunk_nodes(&self) -> usize {
        self.inner.default_chunk_nodes()
    }

    /// Get default chunk size for element data
    fn default_chunk_elements(&self) -> usize {
        self.inner.default_chunk_elements()
    }

    fn __repr__(&self) -> String {
        format!("{:?}", self.inner)
    }

    fn __str__(&self) -> String {
        format!("{:?}", self.inner)
    }
}

/// HDF5 chunk cache configuration
///
/// Controls HDF5's chunk caching behavior for improved I/O performance.
///
/// Args:
///     cache_size (int): Cache size in bytes (default: auto-detected)
///     num_slots (int): Number of hash table slots (0 = auto-calculate)
///     preemption (float): Cache preemption policy, 0.0-1.0 (default: 0.75)
///
/// Example:
///     >>> cache = CacheConfig(cache_size=256*1024*1024)  # 256 MB
///     >>> cache = CacheConfig(cache_size=128*1024*1024, preemption=0.5)
#[pyclass(name = "CacheConfig")]
#[derive(Clone, Debug)]
pub struct PyCacheConfig {
    pub(crate) inner: rust_perf::CacheConfig,
}

#[pymethods]
impl PyCacheConfig {
    #[new]
    #[pyo3(signature = (cache_size=None, num_slots=0, preemption=0.75))]
    fn new(cache_size: Option<usize>, num_slots: usize, preemption: f64) -> Self {
        let size = cache_size.unwrap_or_else(|| {
            rust_perf::NodeType::detect().default_cache_size()
        });

        Self {
            inner: rust_perf::CacheConfig::new(size)
                .with_slots(num_slots)
                .with_preemption(preemption),
        }
    }

    /// Set cache size in bytes
    fn with_cache_size(&self, size: usize) -> Self {
        let mut config = self.inner.clone();
        config.cache_size = size;
        Self { inner: config }
    }

    /// Set cache size in megabytes (convenience)
    fn with_cache_mb(&self, mb: usize) -> Self {
        self.with_cache_size(mb * 1024 * 1024)
    }

    /// Set number of hash table slots
    fn with_slots(&self, num_slots: usize) -> Self {
        Self {
            inner: self.inner.clone().with_slots(num_slots),
        }
    }

    /// Set preemption policy (0.0 = favor writes, 1.0 = favor reads)
    fn with_preemption(&self, preemption: f64) -> Self {
        Self {
            inner: self.inner.clone().with_preemption(preemption),
        }
    }

    #[getter]
    fn cache_size(&self) -> usize {
        self.inner.cache_size
    }

    #[getter]
    fn num_slots(&self) -> usize {
        self.inner.num_slots
    }

    #[getter]
    fn preemption(&self) -> f64 {
        self.inner.preemption
    }

    fn __repr__(&self) -> String {
        format!(
            "CacheConfig(cache_size={}, num_slots={}, preemption={})",
            self.inner.cache_size, self.inner.num_slots, self.inner.preemption
        )
    }
}

/// HDF5 chunk size configuration
///
/// Controls how data is divided and stored in HDF5 files.
///
/// Args:
///     node_chunk_size (int): Nodes per chunk (0 = auto-detect)
///     element_chunk_size (int): Elements per chunk (0 = auto-detect)
///     time_chunk_size (int): Time steps per chunk (default: 0 for mesh I/O)
///
/// Example:
///     >>> chunks = ChunkConfig(node_chunk_size=20000)
///     >>> chunks = ChunkConfig(node_chunk_size=10000, element_chunk_size=10000)
#[pyclass(name = "ChunkConfig")]
#[derive(Clone, Debug)]
pub struct PyChunkConfig {
    pub(crate) inner: rust_perf::ChunkConfig,
}

#[pymethods]
impl PyChunkConfig {
    #[new]
    #[pyo3(signature = (node_chunk_size=None, element_chunk_size=None, time_chunk_size=0))]
    fn new(
        node_chunk_size: Option<usize>,
        element_chunk_size: Option<usize>,
        time_chunk_size: usize,
    ) -> Self {
        let node_type = rust_perf::NodeType::detect();
        let node_size = node_chunk_size.unwrap_or_else(|| node_type.default_chunk_nodes());
        let elem_size = element_chunk_size.unwrap_or_else(|| node_type.default_chunk_elements());

        Self {
            inner: rust_perf::ChunkConfig::new()
                .with_node_chunk_size(node_size)
                .with_element_chunk_size(elem_size)
                .with_time_chunk_size(time_chunk_size),
        }
    }

    /// Set node chunk size
    fn with_node_chunk_size(&self, size: usize) -> Self {
        Self {
            inner: self.inner.clone().with_node_chunk_size(size),
        }
    }

    /// Set element chunk size
    fn with_element_chunk_size(&self, size: usize) -> Self {
        Self {
            inner: self.inner.clone().with_element_chunk_size(size),
        }
    }

    /// Set time step chunk size
    fn with_time_chunk_size(&self, size: usize) -> Self {
        Self {
            inner: self.inner.clone().with_time_chunk_size(size),
        }
    }

    #[getter]
    fn node_chunk_size(&self) -> usize {
        self.inner.node_chunk_size
    }

    #[getter]
    fn element_chunk_size(&self) -> usize {
        self.inner.element_chunk_size
    }

    #[getter]
    fn time_chunk_size(&self) -> usize {
        self.inner.time_chunk_size
    }

    fn __repr__(&self) -> String {
        format!(
            "ChunkConfig(node_chunk_size={}, element_chunk_size={}, time_chunk_size={})",
            self.inner.node_chunk_size, self.inner.element_chunk_size, self.inner.time_chunk_size
        )
    }
}

/// Complete performance configuration for Exodus I/O
///
/// Combines cache and chunk settings with automatic node detection.
///
/// Example:
///     >>> # Auto-detect and use defaults
///     >>> perf = PerformanceConfig.auto()
///
///     >>> # Conservative settings for login nodes
///     >>> perf = PerformanceConfig.conservative()
///
///     >>> # Aggressive settings for compute nodes
///     >>> perf = PerformanceConfig.aggressive()
///
///     >>> # Custom configuration
///     >>> perf = PerformanceConfig.auto() \\
///     ...     .with_cache_mb(256) \\
///     ...     .with_node_chunk_size(20000)
#[pyclass(name = "PerformanceConfig")]
#[derive(Clone, Debug)]
pub struct PyPerformanceConfig {
    pub(crate) inner: rust_perf::PerformanceConfig,
}

#[pymethods]
impl PyPerformanceConfig {
    #[new]
    fn new() -> Self {
        Self {
            inner: rust_perf::PerformanceConfig::auto(),
        }
    }

    /// Create configuration with automatic node detection (recommended)
    #[staticmethod]
    fn auto() -> Self {
        Self {
            inner: rust_perf::PerformanceConfig::auto(),
        }
    }

    /// Create conservative configuration (for login nodes)
    #[staticmethod]
    fn conservative() -> Self {
        Self {
            inner: rust_perf::PerformanceConfig::conservative(),
        }
    }

    /// Create aggressive configuration (for compute nodes)
    #[staticmethod]
    fn aggressive() -> Self {
        Self {
            inner: rust_perf::PerformanceConfig::aggressive(),
        }
    }

    /// Create configuration for specific node type
    #[staticmethod]
    fn for_node_type(node_type: &PyNodeType) -> Self {
        Self {
            inner: rust_perf::PerformanceConfig::for_node_type(node_type.inner),
        }
    }

    /// Set cache size in bytes
    fn with_cache_size(&self, bytes: usize) -> Self {
        Self {
            inner: self.inner.clone().with_cache_size(bytes),
        }
    }

    /// Set cache size in megabytes
    fn with_cache_mb(&self, mb: usize) -> Self {
        Self {
            inner: self.inner.clone().with_cache_mb(mb),
        }
    }

    /// Set cache preemption policy (0.0 = favor writes, 1.0 = favor reads)
    fn with_preemption(&self, preemption: f64) -> Self {
        Self {
            inner: self.inner.clone().with_preemption(preemption),
        }
    }

    /// Set node chunk size
    fn with_node_chunk_size(&self, size: usize) -> Self {
        Self {
            inner: self.inner.clone().with_node_chunk_size(size),
        }
    }

    /// Set element chunk size
    fn with_element_chunk_size(&self, size: usize) -> Self {
        Self {
            inner: self.inner.clone().with_element_chunk_size(size),
        }
    }

    /// Set time step chunk size
    fn with_time_chunk_size(&self, size: usize) -> Self {
        Self {
            inner: self.inner.clone().with_time_chunk_size(size),
        }
    }

    /// Get summary of configuration
    fn summary(&self) -> String {
        self.inner.summary()
    }

    #[getter]
    fn node_type(&self) -> PyNodeType {
        PyNodeType {
            inner: self.inner.node_type,
        }
    }

    #[getter]
    fn cache(&self) -> PyCacheConfig {
        PyCacheConfig {
            inner: self.inner.cache.clone(),
        }
    }

    #[getter]
    fn chunks(&self) -> PyChunkConfig {
        PyChunkConfig {
            inner: self.inner.chunks.clone(),
        }
    }

    fn __repr__(&self) -> String {
        format!("PerformanceConfig({:?})", self.inner.node_type)
    }

    fn __str__(&self) -> String {
        self.inner.summary()
    }
}
