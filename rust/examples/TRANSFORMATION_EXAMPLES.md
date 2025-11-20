# Large Mesh Transformation Examples

This directory contains examples for generating and transforming large Exodus files with comprehensive performance monitoring.

## Overview

These examples demonstrate:
1. Generating large Exodus files (~97GB) with many timesteps
2. Applying transformations (rotation, translation, scaling) to meshes
3. Transforming tensor variables (stress) alongside mesh coordinates
4. Performance tuning with HDF5 caching and chunking options
5. Performance monitoring and throughput analysis

## Files

- **`generate_large_exodus.py`**: Python script to generate large test files
- **`transform_large_mesh.py`**: Python transformation script with performance monitoring
- **`../exodus-rs/examples/13_transform_large_mesh.rs`**: Rust transformation example

## Quick Start

### 1. Generate a Large Test File

```bash
# Generate a ~97GB file with default settings (274x274 nodes = ~75k nodes, 43k timesteps)
python3 generate_large_exodus.py -o test_mesh.exo

# Generate a smaller test file for quick testing (50x50 nodes = 2.5k nodes, 100 timesteps)
python3 generate_large_exodus.py -o small_test.exo \
    --nodes-x 50 --nodes-y 50 --timesteps 100

# Generate with custom performance settings
python3 generate_large_exodus.py -o test_mesh.exo \
    --cache-mb 256 \
    --node-chunk-size 20000 \
    --element-chunk-size 15000 \
    --time-chunk-size 20
```

### 2. Transform the File (Python)

```bash
# Transform with default settings
python3 transform_large_mesh.py -i test_mesh.exo -o transformed.exo

# Transform with custom parameters
python3 transform_large_mesh.py \
    -i test_mesh.exo -o transformed.exo \
    --rotation 90 \
    --mesh-scale 5.0 \
    --scalar-scale 10.0 \
    --cache-mb 256 \
    --node-chunk-size 20000 \
    --element-chunk-size 15000 \
    --time-chunk-size 20
```

### 3. Transform the File (Rust)

```bash
# Build the Rust example
cd ../exodus-rs
cargo build --release --example 13_transform_large_mesh --features netcdf4

# Run with default settings
cargo run --release --example 13_transform_large_mesh --features netcdf4 -- \
    --input ../examples/test_mesh.exo \
    --output ../examples/transformed_rust.exo

# Run with custom parameters
cargo run --release --example 13_transform_large_mesh --features netcdf4 -- \
    --input ../examples/test_mesh.exo \
    --output ../examples/transformed_rust.exo \
    --rotation 90 \
    --mesh-scale 5.0 \
    --scalar-scale 10.0 \
    --cache-mb 256 \
    --node-chunk-size 20000 \
    --element-chunk-size 15000 \
    --time-chunk-size 20
```

## Transformations Applied

Both transformation scripts apply the following operations:

1. **Coordinate Rotation**: Rotate all node coordinates by specified angle around Z-axis (default: 90°)
2. **Coordinate Scaling**: Scale all coordinates by mesh scale factor (default: 5x)
3. **Scalar Variable Scaling**: Scale all nodal scalar variables (default: 10x)
4. **Stress Tensor Rotation**: Rotate stress tensor components using proper tensor transformation

The stress tensor transformation uses the formula: σ' = R σ R^T, where:
- σ is the original stress tensor
- R is the rotation matrix
- σ' is the transformed stress tensor

## Performance Tuning

### HDF5 Cache Size (`--cache-mb`)

Controls the size of the HDF5 chunk cache:
- **Small (4-16 MB)**: Conservative, suitable for login nodes or systems with limited RAM
- **Medium (32-128 MB)**: Good for most compute nodes
- **Large (256+ MB)**: Optimal for large memory systems and I/O intensive workloads

**Rule of thumb**: Larger cache = better performance (up to available memory)

### Chunk Sizes

#### Node Chunk Size (`--node-chunk-size`)
- Controls how many nodes are stored contiguously
- Optimal range: 5,000 - 50,000 depending on total nodes
- Larger values better for sequential access, smaller for random access

#### Element Chunk Size (`--element-chunk-size`)
- Controls how many elements are stored contiguously
- Optimal range: 5,000 - 50,000 depending on total elements
- Similar trade-offs as node chunk size

#### Time Chunk Size (`--time-chunk-size`)
- Controls how many time steps are stored contiguously
- Optimal range: 5 - 50
- Larger values better for time-series access
- Smaller values better for accessing single time steps

### Cache Preemption (`--preemption`)

Controls which chunks get evicted from cache first (0.0 - 1.0):
- **0.0**: Never evict write-only chunks (best for write-heavy workloads)
- **0.75** (default): Balanced eviction policy
- **1.0**: Aggressively evict write-only chunks (best for read-heavy workloads)

## Performance Benchmarking

To benchmark different configurations:

```bash
# Test different cache sizes
for cache in 32 64 128 256 512; do
    echo "Testing cache size: ${cache}MB"
    python3 transform_large_mesh.py \
        -i test_mesh.exo -o transformed_${cache}mb.exo \
        --cache-mb $cache 2>&1 | grep "TOTAL:"
done

# Test different chunk sizes
for chunk in 5000 10000 20000 40000; do
    echo "Testing chunk size: ${chunk}"
    python3 transform_large_mesh.py \
        -i test_mesh.exo -o transformed_${chunk}chunk.exo \
        --node-chunk-size $chunk \
        --element-chunk-size $chunk 2>&1 | grep "TOTAL:"
done

# Test different time chunk sizes
for time_chunk in 1 5 10 20 50; do
    echo "Testing time chunk size: ${time_chunk}"
    python3 transform_large_mesh.py \
        -i test_mesh.exo -o transformed_time${time_chunk}.exo \
        --time-chunk-size $time_chunk 2>&1 | grep "TOTAL:"
done
```

## Performance Expectations

Based on testing with various configurations:

### Small Files (~1-10 GB)
- Cache size: 32-64 MB
- Node/Element chunks: 5,000-10,000
- Time chunks: 5-10
- Expected throughput: 0.5-2 GB/s

### Medium Files (~10-50 GB)
- Cache size: 128-256 MB
- Node/Element chunks: 10,000-20,000
- Time chunks: 10-20
- Expected throughput: 1-3 GB/s

### Large Files (~50-200 GB)
- Cache size: 256-512 MB
- Node/Element chunks: 20,000-40,000
- Time chunks: 20-50
- Expected throughput: 2-5 GB/s

**Note**: Actual performance depends on:
- Storage system (SSD vs HDD, local vs network filesystem)
- Available RAM
- CPU performance
- Number of variables and time steps

## Output

Both scripts provide detailed progress information:

```
======================================================================
TRANSFORM LARGE EXODUS MESH
======================================================================
Input:  test_mesh.exo
Output: transformed.exo

Transformations:
  - Rotate 90.0 degrees around Z axis
  - Scale mesh by 5.0
  - Scale scalar variables by 10.0

Performance settings:
  - Cache size: 128 MB
  - Node chunk size: 10000
  - Element chunk size: 8000
  - Time chunk size: 10
  - Cache preemption: 0.75

Step 1: Reading metadata from input file...
  Nodes: 75076
  Elements: 74529
  Element blocks: 1
  Nodal variables: 9
  Element variables: 6
  Time steps: 43000
  Duration: 0.15s

[... progress updates ...]

======================================================================
PERFORMANCE SUMMARY
======================================================================
Read metadata:                          0.15 seconds
Copy mesh:                              2.34 seconds
Transform coordinates:                  0.52 seconds
Transform variables:                  456.78 seconds
Write output:                           1.23 seconds
----------------------------------------------------------------------
TOTAL:                                461.02 seconds
======================================================================

Throughput: 0.21 GB/s

Output file: transformed.exo
```

## Tips for Best Performance

1. **Start with auto-detection**: Use `PerformanceConfig.auto()` and adjust from there
2. **Match workload**: Use larger caches for I/O intensive workloads
3. **Consider filesystem**: Network filesystems benefit from larger chunks
4. **Monitor memory**: Don't exceed available RAM with cache settings
5. **Test first**: Run on smaller files to find optimal settings before processing large files
6. **Use SSD**: Local SSD storage provides best performance
7. **Parallel processing**: For multiple files, process them in parallel

## Troubleshooting

### Out of Memory Errors
- Reduce `--cache-mb`
- Reduce chunk sizes
- Close other applications

### Slow Performance
- Increase `--cache-mb` (if RAM available)
- Increase chunk sizes
- Check filesystem I/O (use `iostat` or similar tools)
- Consider using faster storage (SSD vs HDD)

### File Corruption
- Ensure sufficient disk space (2-3x input file size)
- Check filesystem permissions
- Verify input file is valid: `ncdump -h input.exo`

## Advanced Usage

### Scripted Performance Testing

Create a script to automatically test various configurations:

```bash
#!/bin/bash
# performance_test.sh

INPUT="test_mesh.exo"
CACHE_SIZES=(32 64 128 256 512)
CHUNK_SIZES=(5000 10000 20000)

for cache in "${CACHE_SIZES[@]}"; do
    for chunk in "${CHUNK_SIZES[@]}"; do
        output="result_c${cache}_n${chunk}.exo"
        echo "Testing: cache=${cache}MB, chunk=${chunk}"

        python3 transform_large_mesh.py \
            -i "$INPUT" -o "$output" \
            --cache-mb $cache \
            --node-chunk-size $chunk \
            --element-chunk-size $chunk \
            2>&1 | tee "log_c${cache}_n${chunk}.txt"

        # Clean up output file to save space
        rm -f "$output"
    done
done

# Analyze results
grep "TOTAL:" log_*.txt | sort -k2 -n
grep "Throughput:" log_*.txt | sort -k2 -nr
```

### Batch Processing

Process multiple files with optimal settings:

```bash
#!/bin/bash
# batch_transform.sh

CACHE_MB=256
NODE_CHUNK=20000
ELEM_CHUNK=15000

for input in *.exo; do
    output="transformed_${input}"
    echo "Processing: $input -> $output"

    python3 transform_large_mesh.py \
        -i "$input" -o "$output" \
        --cache-mb $CACHE_MB \
        --node-chunk-size $NODE_CHUNK \
        --element-chunk-size $ELEM_CHUNK &

    # Limit concurrent jobs
    while [ $(jobs -r | wc -l) -ge 4 ]; do
        sleep 1
    done
done

wait
echo "All transformations complete"
```

## References

- [Exodus II Format Specification](https://sandialabs.github.io/seacas-docs/)
- [HDF5 Performance Tuning](https://portal.hdfgroup.org/display/HDF5/HDF5)
- [NetCDF-4 Best Practices](https://www.unidata.ucar.edu/software/netcdf/docs/netcdf_perf_chunking.html)
- [exodus-rs Performance Guide](../exodus-rs/PERFORMANCE.md)
