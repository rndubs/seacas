#!/usr/bin/env bash
#
# HDF5 Chunking Benchmark Suite - Main Runner Script
#
# This script:
# 1. Builds the exodus-py package (using install_and_test.sh)
# 2. Generates the large benchmark mesh (~100GB)
# 3. Runs the full benchmark suite with various HDF5 chunking configurations
# 4. Generates plots and summary reports
#
# Usage:
#   ./run_all.sh                    # Full benchmark (default)
#   ./run_all.sh --quick            # Quick test with reduced parameters
#   ./run_all.sh --skip-build       # Skip building (use existing installation)
#   ./run_all.sh --skip-generate    # Skip mesh generation (use existing mesh)
#
# Target System: RZHound (LLNL)
#   - 112 cores per node (2 x 56-core Intel Sapphire Rapids)
#   - 256 GB DDR5 memory per node
#   - Lustre parallel filesystem at /p/lustre1/

set -e  # Exit on error

# ==============================================================================
# Configuration
# ==============================================================================

# Lustre output directory
LUSTRE_BASE="/p/lustre1/whitmore"
BENCHMARK_DIR="${LUSTRE_BASE}/chunking_benchmark"

# Mesh configuration (targeting ~100GB file)
NUM_NODES=75000
NUM_TIMESTEPS=18500

# Script locations
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXODUS_PY_DIR="$(dirname "$(dirname "${SCRIPT_DIR}")")"
INSTALL_SCRIPT="${EXODUS_PY_DIR}/install_and_test.sh"

# Output paths
INPUT_MESH="${BENCHMARK_DIR}/input_mesh.exo"
RESULTS_DIR="${BENCHMARK_DIR}/results"
PLOTS_DIR="${BENCHMARK_DIR}/plots"

# Default options
SKIP_BUILD=false
SKIP_GENERATE=false
QUICK_MODE=false
VERBOSE=false
CLEANUP=false
BACKEND="python"  # "python", "rust", or "both"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# ==============================================================================
# Helper Functions
# ==============================================================================

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_section() {
    echo ""
    echo -e "${CYAN}======================================================================${NC}"
    echo -e "${CYAN}$1${NC}"
    echo -e "${CYAN}======================================================================${NC}"
}

print_usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Options:
    --quick           Run quick benchmark with reduced parameter ranges
    --skip-build      Skip building exodus-py (use existing installation)
    --skip-generate   Skip mesh generation (use existing mesh file)
    --backend TYPE    Backend to benchmark: 'python', 'rust', or 'both' (default: python)
    --cleanup         Remove output mesh files after benchmarking
    --verbose, -v     Show verbose output from benchmark runs
    --help, -h        Show this help message

Environment Variables:
    LUSTRE_BASE       Base directory on Lustre (default: /p/lustre1/whitmore)
    NUM_NODES         Target number of mesh nodes (default: 75000)
    NUM_TIMESTEPS     Number of timesteps (default: 18500)

Examples:
    # Full benchmark run (Python only)
    ./run_all.sh

    # Benchmark both Python and Rust
    ./run_all.sh --backend both

    # Quick test run with Rust
    ./run_all.sh --quick --backend rust

    # Skip build and use existing mesh
    ./run_all.sh --skip-build --skip-generate

    # Custom output directory
    LUSTRE_BASE=/p/lustre2/myuser ./run_all.sh
EOF
}

check_lustre() {
    if [ ! -d "$LUSTRE_BASE" ]; then
        log_error "Lustre directory not found: $LUSTRE_BASE"
        log_error "Please ensure you have access to Lustre and set LUSTRE_BASE appropriately."
        exit 1
    fi
    log_success "Lustre directory accessible: $LUSTRE_BASE"
}

get_system_info() {
    log_section "System Information"

    echo "Hostname: $(hostname)"
    echo "Date: $(date)"
    echo "User: $(whoami)"

    # Check if we're on a compute node
    if [ -n "$SLURM_JOB_ID" ]; then
        echo "SLURM Job ID: $SLURM_JOB_ID"
        echo "SLURM Node: $SLURM_NODELIST"
        echo "SLURM CPUs: $SLURM_CPUS_ON_NODE"
    else
        log_warning "Not running under SLURM scheduler"
    fi

    # Memory info
    if [ -f /proc/meminfo ]; then
        total_mem=$(grep MemTotal /proc/meminfo | awk '{print $2}')
        total_mem_gb=$((total_mem / 1024 / 1024))
        echo "Total Memory: ${total_mem_gb} GB"
    fi

    # CPU info
    if [ -f /proc/cpuinfo ]; then
        num_cpus=$(grep -c processor /proc/cpuinfo)
        cpu_model=$(grep "model name" /proc/cpuinfo | head -1 | cut -d: -f2 | xargs)
        echo "CPUs: $num_cpus"
        echo "CPU Model: $cpu_model"
    fi

    echo ""
}

# ==============================================================================
# Parse Arguments
# ==============================================================================

while [[ $# -gt 0 ]]; do
    case $1 in
        --quick)
            QUICK_MODE=true
            shift
            ;;
        --skip-build)
            SKIP_BUILD=true
            shift
            ;;
        --skip-generate)
            SKIP_GENERATE=true
            shift
            ;;
        --backend)
            BACKEND="$2"
            if [[ "$BACKEND" != "python" && "$BACKEND" != "rust" && "$BACKEND" != "both" ]]; then
                log_error "Invalid backend: $BACKEND (must be 'python', 'rust', or 'both')"
                exit 1
            fi
            shift 2
            ;;
        --cleanup)
            CLEANUP=true
            shift
            ;;
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        --help|-h)
            print_usage
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            print_usage
            exit 1
            ;;
    esac
done

# ==============================================================================
# Main Execution
# ==============================================================================

log_section "HDF5 Chunking Benchmark Suite"
echo ""
log_info "Configuration:"
echo "  Lustre base:     $LUSTRE_BASE"
echo "  Benchmark dir:   $BENCHMARK_DIR"
echo "  Input mesh:      $INPUT_MESH"
echo "  Results dir:     $RESULTS_DIR"
echo "  Plots dir:       $PLOTS_DIR"
echo "  Mesh nodes:      $NUM_NODES"
echo "  Timesteps:       $NUM_TIMESTEPS"
echo "  Backend:         $BACKEND"
echo "  Quick mode:      $QUICK_MODE"
echo "  Skip build:      $SKIP_BUILD"
echo "  Skip generate:   $SKIP_GENERATE"
echo "  Cleanup meshes:  $CLEANUP"
echo "  Verbose:         $VERBOSE"
echo ""

# Check Lustre access
check_lustre

# Get system information
get_system_info

# Create directories
log_info "Creating output directories..."
mkdir -p "$BENCHMARK_DIR"
mkdir -p "$RESULTS_DIR"
mkdir -p "$PLOTS_DIR"
log_success "Directories created"

# ==============================================================================
# Step 1: Build exodus-py
# ==============================================================================

if [ "$SKIP_BUILD" = false ]; then
    log_section "Step 1: Building exodus-py"

    if [ -f "$INSTALL_SCRIPT" ]; then
        log_info "Running install_and_test.sh..."
        cd "$EXODUS_PY_DIR"
        bash "$INSTALL_SCRIPT"
        log_success "exodus-py built successfully"
    else
        log_error "Install script not found: $INSTALL_SCRIPT"
        exit 1
    fi
else
    log_section "Step 1: Skipping Build (--skip-build)"
    log_info "Using existing exodus-py installation"
fi

# Build Rust binary if needed
RUST_BINARY=""
EXODUS_RS_DIR="${EXODUS_PY_DIR}/../exodus-rs"

if [ "$BACKEND" = "rust" ] || [ "$BACKEND" = "both" ]; then
    log_section "Step 1.5: Building Rust transform_mesh binary"

    if [ -d "$EXODUS_RS_DIR" ]; then
        log_info "Building Rust binary with cargo..."
        cd "$EXODUS_RS_DIR"

        # Build in release mode
        cargo build --release --features "netcdf4,cli" --bin transform_mesh || {
            log_error "Failed to build Rust binary"
            if [ "$BACKEND" = "both" ]; then
                log_warning "Continuing with Python-only benchmark"
                BACKEND="python"
            else
                exit 1
            fi
        }

        RUST_BINARY="${EXODUS_RS_DIR}/target/release/transform_mesh"
        if [ -f "$RUST_BINARY" ]; then
            log_success "Rust binary built: $RUST_BINARY"
        else
            log_error "Rust binary not found after build"
            if [ "$BACKEND" = "both" ]; then
                log_warning "Continuing with Python-only benchmark"
                BACKEND="python"
            else
                exit 1
            fi
        fi
    else
        log_error "exodus-rs directory not found: $EXODUS_RS_DIR"
        if [ "$BACKEND" = "both" ]; then
            log_warning "Continuing with Python-only benchmark"
            BACKEND="python"
        else
            exit 1
        fi
    fi
fi

# Activate test environment
TEST_VENV="${EXODUS_PY_DIR}/test-venv"
if [ -d "$TEST_VENV" ]; then
    log_info "Activating test environment: $TEST_VENV"
    source "$TEST_VENV/bin/activate"
else
    log_error "Test environment not found: $TEST_VENV"
    log_error "Please run without --skip-build first"
    exit 1
fi

# Install additional dependencies for benchmarking
log_info "Installing benchmark dependencies..."
uv pip install scipy matplotlib tqdm

# Verify exodus import
python -c "import exodus; print(f'exodus-py version: {exodus.__version__}')" || {
    log_error "Failed to import exodus module"
    exit 1
}
log_success "exodus-py loaded successfully"

# ==============================================================================
# Step 2: Generate Benchmark Mesh
# ==============================================================================

if [ "$SKIP_GENERATE" = false ]; then
    log_section "Step 2: Generating Benchmark Mesh (~100GB)"

    # Clean up existing mesh file if it exists
    if [ -f "$INPUT_MESH" ]; then
        log_info "Removing existing mesh file: $INPUT_MESH"
        rm -f "$INPUT_MESH"
    fi

    # Estimate file size
    estimated_gb=$(python -c "
import sys
sys.path.insert(0, '${SCRIPT_DIR}')
from generate_mesh import calculate_file_size_gb
size = calculate_file_size_gb(${NUM_NODES}, ${NUM_TIMESTEPS}, 9)
print(f'{size:.1f}')
")
    log_info "Estimated file size: ${estimated_gb} GB"

    # Check disk space
    available_space=$(df -BG "$BENCHMARK_DIR" | tail -1 | awk '{print $4}' | tr -d 'G')
    required_space=$((${estimated_gb%.*} * 3))  # Need space for input + outputs
    log_info "Available disk space: ${available_space} GB"
    log_info "Required disk space: ~${required_space} GB"

    if [ "$available_space" -lt "$required_space" ]; then
        log_error "Insufficient disk space!"
        log_error "Need at least ${required_space} GB, have ${available_space} GB"
        exit 1
    fi

    log_info "Generating mesh..."
    START_TIME=$(date +%s)

    python "${SCRIPT_DIR}/generate_mesh.py" \
        --output "$INPUT_MESH" \
        --num-nodes "$NUM_NODES" \
        --num-timesteps "$NUM_TIMESTEPS" \
        --cache-mb 512 \
        --node-chunk-size 25000 \
        --element-chunk-size 25000 \
        --time-chunk-size 100

    END_TIME=$(date +%s)
    DURATION=$((END_TIME - START_TIME))

    # Get actual file size
    actual_size=$(ls -lh "$INPUT_MESH" | awk '{print $5}')
    log_success "Mesh generated: $INPUT_MESH"
    log_success "File size: $actual_size"
    log_success "Generation time: ${DURATION} seconds"
else
    log_section "Step 2: Skipping Mesh Generation (--skip-generate)"

    if [ ! -f "$INPUT_MESH" ]; then
        log_error "Input mesh not found: $INPUT_MESH"
        log_error "Please run without --skip-generate first"
        exit 1
    fi

    actual_size=$(ls -lh "$INPUT_MESH" | awk '{print $5}')
    log_info "Using existing mesh: $INPUT_MESH ($actual_size)"
fi

# ==============================================================================
# Step 3: Run Benchmark Suite
# ==============================================================================

log_section "Step 3: Running Benchmark Suite"

# Build benchmark command
BENCH_CMD="python ${SCRIPT_DIR}/run_benchmark_suite.py"
BENCH_CMD="$BENCH_CMD --input-mesh $INPUT_MESH"
BENCH_CMD="$BENCH_CMD --output-dir $RESULTS_DIR"
BENCH_CMD="$BENCH_CMD --backend $BACKEND"

if [ -n "$RUST_BINARY" ] && [ -f "$RUST_BINARY" ]; then
    BENCH_CMD="$BENCH_CMD --rust-binary $RUST_BINARY"
fi

if [ "$QUICK_MODE" = true ]; then
    BENCH_CMD="$BENCH_CMD --quick"
fi

if [ "$CLEANUP" = true ]; then
    BENCH_CMD="$BENCH_CMD --cleanup"
fi

if [ "$VERBOSE" = true ]; then
    BENCH_CMD="$BENCH_CMD --verbose"
fi

log_info "Running command: $BENCH_CMD"
START_TIME=$(date +%s)

eval $BENCH_CMD

END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))
log_success "Benchmark suite completed in ${DURATION} seconds ($((DURATION / 60)) minutes)"

# ==============================================================================
# Step 4: Generate Plots
# ==============================================================================

log_section "Step 4: Generating Plots and Reports"

RESULTS_FILE="${RESULTS_DIR}/benchmark_results.json"

if [ -f "$RESULTS_FILE" ]; then
    log_info "Generating plots from: $RESULTS_FILE"

    python "${SCRIPT_DIR}/plot_results.py" \
        --results "$RESULTS_FILE" \
        --output-dir "$PLOTS_DIR"

    log_success "Plots generated in: $PLOTS_DIR"
else
    log_error "Results file not found: $RESULTS_FILE"
    exit 1
fi

# ==============================================================================
# Summary
# ==============================================================================

log_section "Benchmark Complete!"

echo ""
log_info "Output locations:"
echo "  Input mesh:   $INPUT_MESH"
echo "  Results:      $RESULTS_DIR/benchmark_results.json"
echo "  Plots:        $PLOTS_DIR/"
echo "  Report:       $PLOTS_DIR/benchmark_report.txt"
echo ""

# Print key results
if [ -f "${PLOTS_DIR}/benchmark_report.txt" ]; then
    log_info "Summary Report:"
    echo ""
    head -50 "${PLOTS_DIR}/benchmark_report.txt"
    echo ""
    echo "  (See full report at: ${PLOTS_DIR}/benchmark_report.txt)"
fi

log_success "All done!"
