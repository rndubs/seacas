# HDF5 Chunking Benchmark - Merlin Workflow

This directory contains a Merlin workflow specification for running the HDF5 chunking benchmark suite on RZHound with parallel job execution.

## Overview

The Merlin workflow distributes each benchmark configuration to a separate node, allowing many configurations to run in parallel. This significantly reduces total wall time compared to the serial `run_all.sh` approach.

## Files

| File | Description |
|------|-------------|
| `chunking_benchmark.yaml` | Main Merlin workflow specification |
| `pgen_configs.py` | Parameter generator script (pgen) |
| `collect_results.py` | Aggregates individual results into single JSON |

## Prerequisites

1. **Merlin installed and configured**
   ```bash
   pip install merlin
   ```

2. **exodus-py built** (or let the workflow build it)
   ```bash
   cd ../..  # rust/exodus-py
   ./install_and_test.sh
   ```

3. **Lustre access** at `/p/lustre1/$(USER)/`

## Quick Start

```bash
# Navigate to this directory
cd rust/exodus-py/benchmarks/chunking_benchmark/merlin

# Generate parameter configurations (preview)
python pgen_configs.py --list

# Run the workflow
merlin run chunking_benchmark.yaml --pgen pgen_configs.py

# Monitor status
merlin status chunking_benchmark.yaml

# Check worker status
merlin query-workers chunking_benchmark.yaml
```

## Workflow Steps

1. **setup** - Creates shared venv on Lustre, builds exodus-py, installs dependencies
2. **generate_mesh** - Creates the ~100GB benchmark mesh on Lustre
3. **run_benchmark** - Runs transform for each configuration (PARALLELIZED)
4. **collect_results** - Aggregates all results into single JSON
5. **generate_plots** - Creates matplotlib visualizations

### Design Considerations

- **Shared venv**: A virtual environment is created on Lustre at `$(OUTPUT_PATH)/benchmark-venv` and shared across all steps
- **Script copying**: Each step copies the required scripts to its working directory before execution
- **Error handling**: All steps use `set -euo pipefail` to fail fast on errors
- **UV package manager**: All Python operations use `uv` for speed and reliability

## Parameter Generation

The `pgen_configs.py` script generates configurations using a **one-at-a-time** experimental design:

- Start with baseline configuration
- Vary each parameter individually while keeping others at baseline
- This allows understanding individual parameter effects

```bash
# Full parameter sweep (21 configurations)
python pgen_configs.py --outfile samples.csv

# Quick test (fewer configurations)
python pgen_configs.py --outfile samples.csv --quick

# Full factorial (WARNING: 3000+ configurations!)
python pgen_configs.py --outfile samples.csv --full

# Preview configurations
python pgen_configs.py --list
```

### Default Parameters

| Parameter | Baseline | Test Values |
|-----------|----------|-------------|
| `CACHE_MB` | 256 | 64, 128, 256, 512, 1024, 2048 |
| `NODE_CHUNK_SIZE` | 25000 | 10k, 25k, 50k, 75k, 100k |
| `ELEMENT_CHUNK_SIZE` | 25000 | 10k, 25k, 50k, 75k |
| `TIME_CHUNK_SIZE` | 100 | 10, 50, 100, 250, 500 |
| `PREEMPTION` | 0.75 | 0.0, 0.25, 0.5, 0.75, 1.0 |

## Output Structure

```
/p/lustre1/$(USER)/chunking_benchmark_merlin/
├── benchmark-venv/             # Shared virtual environment
├── build/                      # exodus-py build directory
├── input_mesh.exo              # Generated benchmark mesh (~100GB)
├── results/
│   ├── benchmark_results.json  # Combined results
│   └── result_*.json           # Individual run results
└── plots/
    ├── runtime_vs_*.png
    ├── breakdown_vs_*.png
    ├── all_params_comparison.png
    └── benchmark_report.txt
```

## Running on RZHound

### Interactive Setup

```bash
# Get an interactive allocation
salloc -N 1 -t 1:00:00 -p pbatch

# Load required modules
module load python

# Run workflow
merlin run chunking_benchmark.yaml --pgen pgen_configs.py
```

### Batch Submission

Merlin handles batch submission automatically through its worker system. The workflow will:

1. Submit setup and mesh generation jobs first
2. Fan out benchmark runs to multiple nodes in parallel
3. Collect and plot results after all benchmarks complete

### Scaling

For 21 configurations (one-at-a-time design):
- **Serial execution**: ~42 hours (2 hours × 21)
- **Parallel execution**: ~2 hours (all run simultaneously)

## Customization

### Change Output Directory

Edit `chunking_benchmark.yaml`:
```yaml
env:
    variables:
        OUTPUT_PATH: /p/lustre1/$(USER)/my_custom_path
```

### Change Mesh Size

Edit `chunking_benchmark.yaml`:
```yaml
env:
    variables:
        NUM_NODES: 75000
        NUM_TIMESTEPS: 18500
```

### Add More Configurations

Edit `pgen_configs.py` to modify `PARAM_RANGES` dictionary.

## Troubleshooting

### Workers not starting

Check Celery/RabbitMQ configuration:
```bash
merlin info
```

### Jobs timing out

Increase walltime in `chunking_benchmark.yaml`:
```yaml
- name: run_benchmark
  run:
      walltime: "04:00:00"  # Increase from 02:00:00
```

### Out of disk space

The workflow automatically removes output meshes after each run. If you need to keep them, remove this line from the run_benchmark step:
```bash
rm -f "$OUTPUT_MESH"
```

## Comparison with run_all.sh

| Feature | run_all.sh | Merlin Workflow |
|---------|------------|-----------------|
| Execution | Serial | Parallel |
| Job scheduling | Manual | Automatic |
| Fault tolerance | None | Retry support |
| Scaling | Single node | Multi-node |
| Time (21 configs) | ~42 hours | ~2 hours |
