# Performance Tuning Guide for exodus-rs

This guide explains how to optimize I/O performance for exodus-rs on HPC systems, particularly for large-node computing without MPI.

## Table of Contents

- [Overview](#overview)
- [Understanding the I/O Stack](#understanding-the-io-stack)
- [Quick Start](#quick-start)
- [Node Type Detection](#node-type-detection)
- [Cache Configuration](#cache-configuration)
- [Chunk Configuration](#chunk-configuration)
- [Best Practices](#best-practices)
- [Advanced Tuning](#advanced-tuning)
- [Troubleshooting](#troubleshooting)

## Overview

exodus-rs uses NetCDF-4 (built on HDF5) for file I/O. The HDF5 layer provides sophisticated caching and chunking mechanisms that can significantly impact performance. This guide helps you configure these features for optimal performance on your hardware.

### Key Performance Factors

1. **HDF5 Chunk Cache**: Buffers frequently-accessed data chunks in memory
2. **Chunk Size**: Determines how data is physically stored on disk
3. **Node Type**: Login nodes require conservative settings; compute nodes can be aggressive
4. **Access Patterns**: Sequential vs random, read-heavy vs write-heavy

### What You Can Optimize (Without MPI)

✅ **Chunk cache size** - Dramatic impact (up to 1000x in documented cases)
✅ **Cache preemption policy** - Optimize for read vs write workloads
✅ **Chunk dimensions** - Match your access patterns
✅ **NUMA-aware allocation** - Reduce memory latency on multi-socket systems

❌ **Parallel writes to single file** - Requires MPI-enabled HDF5
❌ **Multi-threaded HDF5 calls** - Global mutex serializes access

## Quick Start

### Automatic Configuration (Recommended)

```rust
use exodus_rs::*;

// Auto-detect node type and apply appropriate settings
let mut file = ExodusFile::create_default("mesh.exo")?;

// Or explicitly specify auto-detection
let options = CreateOptions {
    performance: Some(PerformanceConfig::auto()),
    ..Default::default()
};
let mut file = ExodusFile::create("mesh.exo", options)?;
```

### Manual Configuration

```rust
use exodus_rs::*;

// Custom settings for large compute node
let perf = PerformanceConfig::auto()
    .with_cache_mb(256)           // 256 MB cache
    .with_node_chunk_size(20_000)  // 20k nodes per chunk
    .with_preemption(0.5);         // Balanced read/write

let options = CreateOptions {
    performance: Some(perf),
    ..Default::default()
};

let mut file = ExodusFile::create("mesh.exo", options)?;
```

## Node Type Detection

exodus-rs automatically detects your compute environment and applies appropriate defaults.

### Detection Logic

```rust
use exodus_rs::NodeType;

let node_type = NodeType::detect();

match node_type {
    NodeType::Compute => {
        // Detected job scheduler (SLURM_JOB_ID, FLUX_URI, etc.)
        // Default: 128 MB cache, 10k node chunks
    }
    NodeType::Login => {
        // On HPC system but not in a job
        // Default: 4 MB cache, 1k node chunks (conservative)
    }
    NodeType::Unknown => {
        // Local development machine
        // Default: 16 MB cache, 5k node chunks (moderate)
    }
}
```

### Supported Job Schedulers

- **Slurm**: `SLURM_JOB_ID` environment variable
- **Flux**: `FLUX_URI` environment variable
- **PBS**: `PBS_JOBID` environment variable
- **LSF**: `LSB_JOBID` environment variable

### Override Detection

```rust
// Force specific node type
let perf = PerformanceConfig::for_node_type(NodeType::Compute);

// Or use convenience methods
let perf = PerformanceConfig::aggressive();  // Compute node settings
let perf = PerformanceConfig::conservative(); // Login node settings
```

## Cache Configuration

The HDF5 chunk cache significantly affects I/O performance.

### Cache Size

**Rule of thumb**: Larger is better (up to available memory)

| Node RAM | Recommended Cache | Use Case |
|----------|-------------------|----------|
| 16 GB | 4-16 MB | Development/login nodes |
| 64 GB | 32-128 MB | Small compute nodes |
| 256 GB | 128-512 MB | Large compute nodes |
| 512+ GB | 1+ GB | HPC nodes |

```rust
let perf = PerformanceConfig::auto()
    .with_cache_mb(512);  // 512 MB cache
```

### Cache Slots (Hash Table Size)

The cache uses a hash table to look up chunks. More slots = better performance for large caches.

```rust
use exodus_rs::CacheConfig;

// Manual slot configuration
let cache = CacheConfig::new(256 * 1024 * 1024)  // 256 MB
    .with_slots(10007);  // Prime number recommended

// Auto-calculate based on typical chunk size
let slots = cache.auto_slots(1024 * 1024);  // 1 MB chunks
```

**Guidelines**:
- Use prime numbers for hash table size
- Target ~100x the number of chunks that fit in cache
- Default auto-calculation works well for most cases

### Preemption Policy

Controls which chunks get evicted from cache first.

```rust
let perf = PerformanceConfig::auto()
    .with_preemption(0.5);  // 0.0 to 1.0
```

| Value | Behavior | Use Case |
|-------|----------|----------|
| 0.0 | Never evict write-only chunks | Write-heavy workloads |
| 0.75 | Balanced (default) | Mixed read/write |
| 1.0 | Aggressively evict write-only | Read-heavy workloads |

## Chunk Configuration

Chunking determines how data is physically stored in the HDF5 file.

### Chunk Size Guidelines

**For mesh-oriented I/O** (no time series):
- Chunk by spatial dimensions (nodes/elements)
- Larger chunks reduce metadata overhead
- Match your typical read/write size

```rust
let perf = PerformanceConfig::auto()
    .with_node_chunk_size(20_000)     // 20k nodes per chunk
    .with_element_chunk_size(15_000)  // 15k elements per chunk
    .with_time_chunk_size(0);         // No time chunking (default)
```

**For time series data**:
```rust
let perf = PerformanceConfig::auto()
    .with_node_chunk_size(10_000)   // Spatial chunking
    .with_time_chunk_size(10);       // 10 time steps per chunk
```

### Optimal Chunk Calculation

```rust
use exodus_rs::ChunkConfig;

// Calculate optimal chunk size for dataset
let num_nodes = 1_000_000;
let target_bytes = 1024 * 1024;  // 1 MB chunks

let chunk_size = ChunkConfig::calculate_optimal_chunk(
    num_nodes,
    target_bytes
);
```

**Target chunk size**: 100 KB - 10 MB per chunk
- Too small: High metadata overhead
- Too large: Wasted cache space, poor random access

## Best Practices

### 1. Profile Before Optimizing

```bash
# Run with default settings first
time cargo run --release --example my_app

# Then tune based on your workload
```

### 2. Match Cache to Workload

```rust
// For sequential writes of entire arrays
let perf = PerformanceConfig::aggressive()
    .with_cache_mb(256)
    .with_preemption(0.0);  // Favor writes

// For random access reads
let perf = PerformanceConfig::aggressive()
    .with_cache_mb(512)
    .with_preemption(1.0);  // Favor reads
```

### 3. Align Chunk Size with Access Patterns

```rust
// If you write 50k nodes at a time
let perf = PerformanceConfig::auto()
    .with_node_chunk_size(50_000);  // Match write size

// If you read individual nodes randomly
let perf = PerformanceConfig::auto()
    .with_node_chunk_size(1_000);   // Smaller chunks
```

### 4. Use Conservative Settings on Login Nodes

```rust
// Detect and respect shared resources
let perf = if NodeType::detect() == NodeType::Login {
    PerformanceConfig::conservative()
} else {
    PerformanceConfig::aggressive()
};
```

### 5. Document Your Configuration

```rust
let perf = PerformanceConfig::auto()
    .with_cache_mb(256)
    .with_node_chunk_size(20_000);

println!("Performance configuration:\n{}", perf.summary());
```

## Advanced Tuning

### NUMA-Aware Memory Allocation

On multi-socket systems (e.g., 2x56-core nodes), allocate memory on the local NUMA node:

```bash
# Pin process to NUMA node 0
numactl --cpunodebind=0 --membind=0 ./my_app

# Or distribute across nodes
numactl --interleave=all ./my_app
```

```rust
// In your code (requires libc)
#[cfg(target_os = "linux")]
unsafe {
    libc::numa_set_preferred(0);  // Prefer NUMA node 0
}
```

### Environment Variable Override

HDF5 respects environment variables (must be set before library init):

```bash
# Set cache parameters
export HDF5_CHUNK_CACHE_NBYTES=268435456  # 256 MB
export HDF5_CHUNK_CACHE_NSLOTS=10007      # Prime number
export HDF5_CHUNK_CACHE_W0=0.75           # Preemption

# Run your application
cargo run --release
```

exodus-rs automatically sets these if not already present, but you can override.

### Process-Level Parallelism

Since HDF5 has a global mutex, use multiple processes instead of threads:

```rust
use rayon::prelude::*;
use std::process::Command;

// Write partitions in parallel (separate files)
(0..num_partitions).into_par_iter().for_each(|i| {
    let output = format!("mesh_part_{:04}.exo", i);
    write_partition(i, &output).unwrap();
});

// Later: merge files if needed (can be done in parallel reads)
```

### Delegation Pattern (Single File)

For single-file output with parallel compute:

```rust
use std::sync::mpsc::channel;
use rayon::prelude::*;

let (tx, rx) = channel();

// I/O thread (serial)
let writer = std::thread::spawn(move || {
    let mut file = ExodusFile::create("mesh.exo", options)?;
    while let Ok(data) = rx.recv() {
        file.write_data(data)?;
    }
    Ok(())
});

// Compute threads (parallel)
(0..num_tasks).into_par_iter().for_each(|i| {
    let result = compute_expensive_data(i);
    tx.send(result).unwrap();
});

writer.join().unwrap()?;
```

## Troubleshooting

### Slow Write Performance

**Symptoms**: Writing coordinates or variables takes a long time

**Solutions**:
1. Increase cache size:
   ```rust
   .with_cache_mb(512)
   ```

2. Match chunk size to write size:
   ```rust
   .with_node_chunk_size(num_nodes)  // If writing all at once
   ```

3. Use write-favoring preemption:
   ```rust
   .with_preemption(0.0)
   ```

### Slow Read Performance

**Symptoms**: Reading data is slower than expected

**Solutions**:
1. Increase cache size
2. Use read-favoring preemption:
   ```rust
   .with_preemption(1.0)
   ```
3. Check chunk alignment with reads

### Out of Memory

**Symptoms**: Process killed or OOM errors

**Solutions**:
1. Reduce cache size:
   ```rust
   .with_cache_mb(64)  // Smaller cache
   ```

2. Use conservative settings:
   ```rust
   PerformanceConfig::conservative()
   ```

### Poor Scaling on Multi-Socket Systems

**Symptoms**: Performance doesn't improve with more cores

**Solutions**:
1. Use NUMA-aware allocation (see Advanced Tuning)
2. Pin threads to specific sockets
3. Consider process-level parallelism

### "HDF5 library not initialized" Errors

**Cause**: Environment variables set after HDF5 initialization

**Solution**: Set HDF5 env vars before importing any HDF5-using library, or use exodus-rs config (which sets them early).

## Performance Expectations

### Realistic Single-Node Limits

For a 112-core node with 256 GB RAM and NVMe storage:

| Operation | Sequential Write | Sequential Read | Random Access |
|-----------|-----------------|-----------------|---------------|
| Coordinates (100M nodes) | 2-5 GB/s | 3-8 GB/s | 100-500 MB/s |
| Variables (100M values) | 1-4 GB/s | 2-6 GB/s | 50-300 MB/s |

**Bottleneck**: Usually storage bandwidth, not CPU or HDF5

### Optimization Impact

| Change | Expected Speedup |
|--------|------------------|
| Increase cache 10x (10MB → 100MB) | 2-5x for cached access |
| Match chunk size to access | 2-10x |
| Optimize preemption | 1.2-2x |
| NUMA-aware allocation | 1.5-3x (multi-socket) |
| **Combined** | **5-50x possible** |

## Example Configurations

### Development (Laptop)

```rust
let perf = PerformanceConfig::auto()
    .with_cache_mb(16)
    .with_node_chunk_size(5_000);
```

### Small HPC Job (16 cores, 64 GB)

```rust
let perf = PerformanceConfig::auto()
    .with_cache_mb(128)
    .with_node_chunk_size(10_000);
```

### Large HPC Job (112 cores, 256 GB)

```rust
let perf = PerformanceConfig::aggressive()
    .with_cache_mb(512)
    .with_node_chunk_size(20_000)
    .with_preemption(0.5);
```

### Massive HPC Job (256+ cores, 1+ TB)

```rust
let perf = PerformanceConfig::for_node_type(NodeType::Compute)
    .with_cache_mb(2048)           // 2 GB cache
    .with_node_chunk_size(50_000)
    .with_preemption(0.3);         // Favor writes
```

## References

- [HDF5 Chunking Guide](https://support.hdfgroup.org/HDF5/doc/Advanced/Chunking/index.html)
- [HDF5 Performance Tuning](https://www.slideshare.net/HDFEOS/hdf5-performancetuning)
- [NetCDF-4 Performance](https://www.unidata.ucar.edu/software/netcdf/docs/netcdf_perf_chunking.html)
- [Exodus II Format Specification](https://sandialabs.github.io/seacas-docs/)

## Getting Help

- **Example**: See `examples/11_performance_tuning.rs`
- **API Docs**: `cargo doc --open --features netcdf4`
- **Issues**: https://github.com/sandialabs/seacas/issues
