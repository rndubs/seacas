# HDF5 Chunking Benchmark Suite

This benchmark suite evaluates different HDF5 chunking strategies for exodus-py to find optimal performance configurations for large mesh transformations.

The benchmark supports both **Python** and **Rust** implementations, allowing direct performance comparison between the two.

## Target System

- **RZHound (LLNL)**
  - 112 cores per node (2 x 56-core Intel Sapphire Rapids)
  - 256 GB DDR5 memory per node
  - Lustre parallel filesystem at `/p/lustre1/`
  - SLURM job scheduler

## Quick Start

```bash
# Full benchmark with Python only (default)
./run_all.sh

# Benchmark both Python and Rust for comparison
./run_all.sh --backend both

# Rust-only benchmark
./run_all.sh --backend rust

# Quick test with reduced parameter ranges
./run_all.sh --quick

# Quick comparison of Python vs Rust
./run_all.sh --quick --backend both

# Skip build if already installed
./run_all.sh --skip-build

# Skip mesh generation if mesh already exists
./run_all.sh --skip-build --skip-generate
```

## What It Does

The benchmark performs the following workflow:

1. **Generate Mesh**: Creates a ~100GB Exodus mesh file with:
   - ~75,000 nodes (surface mesh with QUAD4 elements)
   - ~18,500 timesteps
   - 9 scalar field variables (temperature, pressure, velocity_xyz, stress, strain, displacement, heat_flux)

2. **Transform Mesh**: For each performance configuration:
   - Read the input mesh
   - Scale coordinates by 2.0
   - Rotate 90 degrees about Z axis
   - Scale pressure field by 10.0
   - Write transformed mesh to new file

3. **Measure Performance**: Records for each configuration:
   - Read time (init, coords, connectivity, variables)
   - Transform time (coordinates, variables)
   - Write time (init, coords, connectivity, variables, close)
   - Peak memory usage

4. **Generate Reports**: Creates plots and summary reports comparing configurations

## Performance Parameters Evaluated

| Parameter | Description | Test Values |
|-----------|-------------|-------------|
| `cache_mb` | HDF5 chunk cache size (MB) | 64, 128, 256, 512, 1024, 2048 |
| `node_chunk_size` | Nodes per chunk | 10k, 25k, 50k, 75k, 100k |
| `element_chunk_size` | Elements per chunk | 10k, 25k, 50k, 75k |
| `time_chunk_size` | Timesteps per chunk | 10, 50, 100, 250, 500 |
| `preemption` | Cache eviction policy | 0.0, 0.25, 0.5, 0.75, 1.0 |

## Files

| File | Description |
|------|-------------|
| `run_all.sh` | Main entry point - builds, generates mesh, runs benchmarks, plots |
| `generate_mesh.py` | Creates the ~100GB benchmark mesh |
| `transform_mesh.py` | Python implementation - reads, transforms, and writes mesh |
| `run_benchmark_suite.py` | Driver that runs transform_mesh with various configurations |
| `plot_results.py` | Generates matplotlib visualizations from results |

### Rust Binary

The Rust implementation is located at `rust/exodus-rs/src/bin/transform_mesh.rs`. It provides identical functionality to the Python version and uses the same CLI arguments for direct comparison.

```bash
# Build the Rust binary manually
cd rust/exodus-rs
cargo build --release --features "netcdf4,cli" --bin transform_mesh

# Run the Rust binary directly
./target/release/transform_mesh --input input.exo --output output.exo \
    --cache-mb 256 --node-chunk-size 25000
```

## Output Structure

```
/p/lustre1/whitmore/chunking_benchmark/
├── input_mesh.exo              # Generated benchmark mesh (~100GB)
├── results/
│   ├── benchmark_results.json  # All timing data
│   ├── run_000_*.json          # Individual run results
│   └── run_000_*.exo           # Transformed meshes (if not cleaned)
└── plots/
    ├── runtime_vs_cache_mb.png
    ├── runtime_vs_node_chunk_size.png
    ├── breakdown_vs_*.png
    ├── memory_vs_*.png
    ├── all_params_comparison.png
    ├── speedup_comparison.png
    ├── runtime_vs_memory.png
    └── benchmark_report.txt
```

## Running on RZHound

### Interactive Session

```bash
# Request an interactive node
salloc -N 1 -t 4:00:00 -p pbatch

# Run benchmarks
cd /path/to/seacas/rust/exodus-py/benchmarks/chunking_benchmark
./run_all.sh
```

### Batch Job

Create a SLURM batch script:

```bash
#!/bin/bash
#SBATCH -N 1
#SBATCH -t 8:00:00
#SBATCH -p pbatch
#SBATCH -J chunking_benchmark
#SBATCH -o benchmark_%j.out
#SBATCH -e benchmark_%j.err

cd /path/to/seacas/rust/exodus-py/benchmarks/chunking_benchmark
./run_all.sh
```

Submit with:
```bash
sbatch benchmark_job.sh
```

## Individual Script Usage

### Generate Mesh Only

```bash
python generate_mesh.py \
    --output /p/lustre1/whitmore/chunking_benchmark/input_mesh.exo \
    --num-nodes 75000 \
    --num-timesteps 18500 \
    --cache-mb 512
```

### Transform Mesh with Specific Config

```bash
python transform_mesh.py \
    --input input_mesh.exo \
    --output output_mesh.exo \
    --cache-mb 256 \
    --node-chunk-size 50000 \
    --element-chunk-size 25000 \
    --time-chunk-size 100 \
    --preemption 0.75 \
    --output-json results.json
```

### Run Benchmark Suite Only

```bash
python run_benchmark_suite.py \
    --input-mesh input_mesh.exo \
    --output-dir results/ \
    --verbose
```

### Generate Plots Only

```bash
python plot_results.py \
    --results results/benchmark_results.json \
    --output-dir plots/
```

## Understanding Results

### Key Metrics

- **Total Time**: End-to-end time for read → transform → write
- **Read Time**: Time to read all mesh data into memory
- **Transform Time**: Time to apply coordinate and field transformations
- **Write Time**: Time to write transformed mesh to new file
- **Peak Memory**: Maximum memory usage during the run

### Interpreting Plots

- **runtime_vs_*.png**: Shows how total runtime varies with each parameter
- **breakdown_vs_*.png**: Stacked bars showing read/transform/write time breakdown
- **speedup_comparison.png**: Bar chart of best speedup achieved per parameter
- **runtime_vs_memory.png**: Scatter plot showing memory/performance trade-offs

### Python vs Rust Comparison Plots

When running with `--backend both`, additional comparison plots are generated:

- **python_vs_rust_comparison.png**: Side-by-side comparison of best times and speedup by phase
- **backend_comparison_*.png**: Per-parameter comparison showing both backends on the same chart

### Recommendations

After running, check `benchmark_report.txt` for:
- Best overall configuration found
- Per-parameter analysis
- Python vs Rust performance comparison (if both backends were run)
- Specific recommendations for your workload

## Technical Notes

### HDF5 Cache Configuration

The HDF5 chunk cache is configured via environment variables that must be set **before** the HDF5 library is initialized. This is why each benchmark run executes as a separate subprocess - to ensure fresh library initialization with the correct cache settings.

### Chunking vs Caching

- **Chunking** (✅ fully working): Controls how data is stored on disk. Properly sized chunks improve sequential read/write performance.
- **Caching** (⚠️ partially working): Controls in-memory buffering. May not apply if HDF5 was initialized before setting env vars.

For best results, focus on chunking optimization which has the most reliable and significant impact.

### Memory Considerations

With RZHound's 256GB per node, you can use aggressive cache sizes (1-2GB) without concern. The benchmark sweeps cache sizes from 64MB to 2GB to find the optimal point.

## Troubleshooting

### "Out of disk space"

The benchmark requires ~300GB free space (100GB input + 200GB for multiple outputs). Use `--cleanup` to remove output meshes after each run.

### "HDF5 infinite loop" warnings

This is a known HDF5 issue during cleanup. Generally harmless - the data is written correctly.

### Runs timing out

Each run has a 2-hour timeout. If your mesh is larger, increase the timeout in `run_benchmark_suite.py`.

### Import errors

Ensure you've run the build step:
```bash
cd rust/exodus-py
./install_and_test.sh
source test-venv/bin/activate
```
