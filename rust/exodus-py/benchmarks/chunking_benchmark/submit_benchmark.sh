#!/bin/bash
#SBATCH -N 1                          # 1 node
#SBATCH -t 8:00:00                    # 8 hours
#SBATCH -p pbatch                     # Batch partition
#SBATCH -J chunking_benchmark         # Job name
#SBATCH -o benchmark_%j.out           # stdout file
#SBATCH -e benchmark_%j.err           # stderr file
#SBATCH --exclusive                   # Exclusive node access for consistent benchmarking

#
# SLURM Batch Script for HDF5 Chunking Benchmark
#
# Submit with: sbatch submit_benchmark.sh
#
# Options:
#   --quick    Add to run quick benchmark
#   --cleanup  Add to clean up output meshes
#
# Example:
#   sbatch submit_benchmark.sh
#   sbatch --export=QUICK=1 submit_benchmark.sh
#

echo "=============================================="
echo "HDF5 Chunking Benchmark - SLURM Job"
echo "=============================================="
echo "Job ID:     $SLURM_JOB_ID"
echo "Node:       $SLURM_NODELIST"
echo "Start Time: $(date)"
echo "=============================================="

# Navigate to benchmark directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Build command with options
CMD="./run_all.sh"

# Check for QUICK mode (set via sbatch --export=QUICK=1)
if [ -n "$QUICK" ]; then
    CMD="$CMD --quick"
    echo "Running in QUICK mode"
fi

# Check for CLEANUP mode
if [ -n "$CLEANUP" ]; then
    CMD="$CMD --cleanup"
    echo "Cleanup mode enabled"
fi

# Check for SKIP_BUILD mode
if [ -n "$SKIP_BUILD" ]; then
    CMD="$CMD --skip-build"
    echo "Skipping build step"
fi

# Check for SKIP_GENERATE mode
if [ -n "$SKIP_GENERATE" ]; then
    CMD="$CMD --skip-generate"
    echo "Skipping mesh generation"
fi

# Add verbose output
CMD="$CMD --verbose"

echo ""
echo "Running: $CMD"
echo ""

# Run benchmark
$CMD
EXIT_CODE=$?

echo ""
echo "=============================================="
echo "Job Complete"
echo "End Time: $(date)"
echo "Exit Code: $EXIT_CODE"
echo "=============================================="

# Print output locations
if [ $EXIT_CODE -eq 0 ]; then
    echo ""
    echo "Results available at:"
    echo "  /p/lustre1/whitmore/chunking_benchmark/results/"
    echo "  /p/lustre1/whitmore/chunking_benchmark/plots/"
fi

exit $EXIT_CODE
