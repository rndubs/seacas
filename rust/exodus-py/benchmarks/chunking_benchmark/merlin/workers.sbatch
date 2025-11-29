#!/bin/bash
#SBATCH -N 32
#SBATCH --ntasks=32
#SBATCH --ntasks-per-node=1
#SBATCH --exclusive
#SBATCH -p pbatch
#SBATCH -t 24:00:00
#SBATCH -J merlin_workers
#SBATCH -o merlin_workers_%j.out
#SBATCH -A ecopper

module load python/3.12

cd /usr/WS1/whitmore/pydev/seacas/rust/exodus-py/benchmarks/chunking_benchmark/merlin

# Activate the venv that has merlin installed
source /usr/WS1/whitmore/pydev/seacas/rust/exodus-py/.venv/bin/activate

# Print out the workers command (for debugging)
uv run merlin run-workers chunking_benchmark.yaml --echo

# Run the workers on the allocation
# Workers will naturally exit when all queues are empty
echo "Starting merlin workers..."
uv run merlin run-workers chunking_benchmark.yaml

echo ""
echo "Merlin run-workers has spawned worker processes"
echo ""

# Give workers time to fully start
sleep 15

# Check if srun worker processes are running
echo "Checking for active worker srun processes..."
pgrep -a srun | grep celery || echo "No celery srun processes found"
echo ""

# Monitor for srun processes that are running celery workers
# These processes will exit when workers are done
echo "Monitoring for worker completion..."
MAX_WAIT=86400  # 24 hours max
ELAPSED=0
CHECK_INTERVAL=30

while [ $ELAPSED -lt $MAX_WAIT ]; do
    # Count srun processes running celery workers
    WORKER_COUNT=$(pgrep -f "srun.*celery.*worker" | wc -l)

    if [ "$WORKER_COUNT" -gt 0 ]; then
        echo "[$(date '+%H:%M:%S')] $WORKER_COUNT srun worker process(es) still running..."
        sleep $CHECK_INTERVAL
        ELAPSED=$((ELAPSED + CHECK_INTERVAL))
    else
        echo "[$(date '+%H:%M:%S')] No more srun worker processes detected"
        break
    fi
done

echo ""
echo "All workers have finished. Exiting."
