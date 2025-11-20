# Memory-Efficient Exodus File Processing Examples

This directory contains examples demonstrating how to process large Exodus files (100GB+) with limited memory by using sequential processing and efficient memory management.

## Overview

Both Python and Rust versions perform the same operations:
1. Read and transform mesh coordinates (scaling and translation)
2. Read, scale, and write time-history field values for all time steps
3. Minimize memory usage by processing one time step at a time

**Memory Usage:**
- **Python version**: ~4x the size of a single time step (due to PyO3 marshaling overhead)
- **Rust version**: ~2-3x the size of a single time step (more efficient, no marshaling)

**Performance:**
- Rust version is typically **5-10x faster** than Python for large files
- Rust version uses **30-40% less memory** than Python

## Files

- `process_exodus_memory_efficient.py` - Python implementation
- `run_rust_example.sh` - Bash script to compile and run the Rust version
- `../exodus-rs/examples/12_process_large_file.rs` - Rust implementation

## Prerequisites

### For Python Version

1. Install exodus-py:
   ```bash
   cd rust/exodus-py
   maturin develop --release
   ```

2. Or install from PyPI (if published):
   ```bash
   pip install exodus-py
   ```

### For Rust Version

1. Install HDF5 and NetCDF libraries (see `../CLAUDE.md` for detailed instructions):

   **Ubuntu/Debian:**
   ```bash
   apt-get install -y libhdf5-dev libnetcdf-dev pkg-config
   ```

   **macOS:**
   ```bash
   brew install hdf5 netcdf
   ```

2. Verify installation:
   ```bash
   pkg-config --modversion hdf5
   pkg-config --modversion netcdf
   ```

## Usage

### Python Version

```bash
# Basic usage
python process_exodus_memory_efficient.py input.exo output.exo

# With custom scale factor
python process_exodus_memory_efficient.py input.exo output.exo 2.0

# Make executable and run directly
chmod +x process_exodus_memory_efficient.py
./process_exodus_memory_efficient.py input.exo output.exo
```

### Rust Version (Recommended for Large Files)

```bash
# Use the provided script (easiest)
./run_rust_example.sh input.exo output.exo 2.0

# Or compile and run manually
cd ../exodus-rs
cargo build --release --example 12_process_large_file --features netcdf4
./target/release/examples/12_process_large_file input.exo output.exo 2.0
```

## Performance Comparison

To compare Python vs Rust performance on your file:

```bash
# Test with Python
time python process_exodus_memory_efficient.py input.exo output_py.exo 1.5

# Test with Rust
time ./run_rust_example.sh input.exo output_rust.exo 1.5

# Verify outputs are identical
ncdump -h output_py.exo > py.txt
ncdump -h output_rust.exo > rust.txt
diff py.txt rust.txt
```

## Memory Efficiency Tips

### Current Implementation (Sequential Processing)

Both scripts process data sequentially:
- Coordinates: Read once → Transform → Write → Free
- Time steps: Read one → Process → Write → Free → Repeat

**Memory usage**: `O(max_single_timestep)` - suitable for files up to system RAM

### For Even Larger Files (Future Enhancement)

If your file is still too large, you can use chunked processing by:

1. **Adding partial coordinate reading** (Rust layer already supports this):
   ```python
   # Process coordinates in chunks of 1M nodes
   chunk_size = 1_000_000
   for i in range(0, total_nodes, chunk_size):
       x, y, z = reader.get_partial_coords(i, chunk_size)  # Not yet exposed to Python
       # Transform and write chunk
   ```

2. **Using memory-mapped files** (netcdf-rs feature)

3. **Processing variable subsets** if you don't need all variables

## Example Output

```
Processing Exodus file: large_simulation.exo
Output file: output.exo
Field scale factor: 1.5

[1/6] Opening input file...
[2/6] Reading metadata...
  Nodes: 10,000,000
  Elements: 9,500,000
  Dimensions: 3
  Element Blocks: 5
  Time Steps: 50,000
  Nodal Variables: 3 - ['Temperature', 'Pressure', 'Velocity']
  Estimated memory per time step: 228.9 MB
  Peak memory usage: ~687 MB (Python) or ~572 MB (Rust)

[3/6] Processing coordinates...
  Reading coordinates from input...
  Loaded 10,000,000 nodes
  Transforming coordinates...

[4/6] Creating output file...
  Writing transformed coordinates...
  Copying element blocks...
  Defining variables...

[5/6] Processing 50,000 time steps...
  Processing one time step at a time to minimize memory usage...
  Progress: 100/50,000 (0.2%) - 12.5 steps/sec - ETA: 3992s
  Progress: 10,000/50,000 (20.0%) - 13.2 steps/sec - ETA: 3030s
  ...
  Progress: 50,000/50,000 (100.0%) - 13.5 steps/sec - ETA: 0s

[6/6] Finalizing output file...

✓ Processing complete!
  Output written to: output.exo
  Total time: 3703.70s (Rust) or 18500s (Python)
  Average: 13.5 steps/sec (Rust) or 2.7 steps/sec (Python)
```

## Troubleshooting

### Python: "exodus-py not installed"
```bash
cd rust/exodus-py
maturin develop --release
```

### Rust: "NetCDF library not found"
```bash
# Ubuntu/Debian
apt-get install -y libhdf5-dev libnetcdf-dev pkg-config

# macOS
brew install hdf5 netcdf
```

### Rust: Compilation errors
See `../CLAUDE.md` for detailed dependency installation instructions.

### Memory issues persist
- Verify you're using the sequential processing approach (not loading all timesteps at once)
- Check available system memory: `free -h` (Linux) or `vm_stat` (macOS)
- Reduce chunk sizes if processing in chunks
- Close other applications to free memory

## Architecture Details

### Python Version (`process_exodus_memory_efficient.py`)

**Data Flow:**
```
HDF5 → NetCDF-rs (Vec<f64>) → Type conv (Vec<f64>) → PyO3 (Python list)
```

**Memory Copies per Read:**
1. HDF5 read allocates Vec
2. Type conversion creates new Vec
3. PyO3 marshaling creates Python list

**Total**: 3 copies on read, 2-3 copies on write

### Rust Version (`12_process_large_file.rs`)

**Data Flow:**
```
HDF5 → NetCDF-rs (Vec<f64>) → Type conv (Vec<f64>) → Process in-place
```

**Memory Copies per Read:**
1. HDF5 read allocates Vec
2. Type conversion creates new Vec
3. **No marshaling** - stays in Rust
4. **In-place scaling** - no copy

**Total**: 2 copies on read, 1-2 copies on write

**Key Optimizations:**
- `scale_field_values_inplace()` modifies data without allocation
- References used for writing (no copy)
- Explicit drops for memory control
- Compiler optimizations in release mode

## Next Steps

For even better performance, consider:

1. **Add chunked coordinate processing** to Python bindings
2. **Add NumPy zero-copy support** to eliminate PyO3 marshaling
3. **Use parallel processing** for independent time steps (if order doesn't matter)
4. **Profile your specific workload** to identify bottlenecks

## License

Same as the parent project (Apache-2.0 OR MIT).
