#!/bin/bash
# Run the Merlin chunking benchmark workflow
# Usage: ./run.sh [--quick] [--one-at-a-time]
#
# Options:
#   --quick          Use reduced parameter ranges for faster testing
#   --one-at-a-time  Use one-at-a-time design (~21 configs, varies one param at a time)
#
# Default: Full factorial design (~432 configs, all combinations)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Activate venv
source /usr/WS1/whitmore/pydev/seacas/rust/exodus-py/.venv/bin/activate

# Parse arguments
QUICK_MODE=false
FULL_MODE=true
for arg in "$@"; do
    case $arg in
        --quick)         QUICK_MODE=true ;;
        --one-at-a-time) FULL_MODE=false ;;
        *)               echo "Unknown option: $arg"; exit 1 ;;
    esac
done

# Export options as environment variables for pgen_configs.py
export PGEN_QUICK="$QUICK_MODE"
export PGEN_FULL="$FULL_MODE"

if [[ "$QUICK_MODE" == "true" ]]; then
    echo "Running in QUICK mode (reduced parameter set)"
fi
if [[ "$FULL_MODE" == "true" ]]; then
    echo "Running in FULL FACTORIAL mode (all combinations, ~432 configs)"
else
    echo "Running in ONE-AT-A-TIME mode (varying one parameter at a time, ~21 configs)"
fi

echo "=== Merlin Chunking Benchmark ==="
echo ""

# Queue tasks to the broker
echo "Step 1: Queueing tasks..."
echo "Running: uv run merlin run chunking_benchmark.yaml"
uv run merlin run chunking_benchmark.yaml

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
