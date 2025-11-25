#!/bin/bash
# Stop the Merlin chunking benchmark workflow
# Stops all workers and purges the task queues

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Activate venv
source /usr/WS1/whitmore/pydev/seacas/rust/exodus-py/.venv/bin/activate

echo "=== Stopping Merlin Workflow ==="
echo ""

# Cancel any running Slurm jobs for this workflow
echo "Step 1: Cancelling Slurm jobs..."
scancel --name=merlin_workers 2>/dev/null || echo "  No running jobs found"

# Stop any celery workers
echo ""
echo "Step 2: Stopping Merlin workers..."
uv run merlin stop-workers --spec chunking_benchmark.yaml 2>/dev/null || echo "  No workers to stop"

# Purge task queues (force to avoid interactive prompt)
echo ""
echo "Step 3: Purging task queues..."
uv run merlin purge chunking_benchmark.yaml --force

echo ""
echo "=== Workflow stopped and queues purged ==="
