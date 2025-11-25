#!/bin/bash
#SBATCH -N 16
#SBATCH -p pbatch
#SBATCH -t 12:00:00
#SBATCH -J merlin_workers
#SBATCH -o merlin_workers_%j.out

module load python/3.12

cd /usr/WS1/whitmore/pydev/seacas/rust/exodus-py/benchmarks/chunking_benchmark/merlin

# Activate the venv that has merlin installed
source /usr/WS1/whitmore/pydev/seacas/rust/exodus-py/.venv/bin/activate

# Print out the workers command (for debugging)
uv run merlin run-workers chunking_benchmark.yaml --echo

# Run the workers on the allocation
uv run merlin run-workers chunking_benchmark.yaml

# Keep script alive until workers finish processing all tasks
uv run merlin monitor chunking_benchmark.yaml
