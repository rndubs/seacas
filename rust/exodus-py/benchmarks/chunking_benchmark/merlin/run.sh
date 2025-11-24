#!/bin/bash
# Run the Merlin chunking benchmark workflow
# Usage: ./run.sh [--quick]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Activate venv
source /usr/WS1/whitmore/pydev/seacas/rust/exodus-py/.venv/bin/activate

# Check for quick mode
PARGS=""
if [[ "${1:-}" == "--quick" ]]; then
    echo "Running in QUICK mode (reduced parameter set)"
    PARGS='--pargs "quick:true"'
fi

echo "=== Merlin Chunking Benchmark ==="
echo ""

# Queue tasks to the broker
echo "Step 1: Queueing tasks..."
if [[ -n "$PARGS" ]]; then
    uv run merlin run chunking_benchmark.yaml --pgen pgen_configs.py --pargs "quick:true"
else
    uv run merlin run chunking_benchmark.yaml --pgen pgen_configs.py
fi

echo ""
echo "Step 2: Submitting worker batch job..."
sbatch workers.sbatch

echo ""
echo "=== Workflow submitted ==="
echo ""
echo "Monitor progress with:"
echo "  uv run merlin status chunking_benchmark.yaml"
echo "  tail -f merlin_workers_*.out"
echo ""
echo "Stop the workflow with:"
echo "  ./stop.sh"
