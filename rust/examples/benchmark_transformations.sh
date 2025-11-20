#!/usr/bin/env bash
# Benchmark transformation performance with different HDF5 configurations
# This script tests both Python and Rust transformation implementations
# with various caching and chunking strategies

set -e  # Exit on error
set -u  # Exit on undefined variable

# Determine script directory (absolute path)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXODUS_RS_DIR="$SCRIPT_DIR/../exodus-rs"
EXODUS_PY_DIR="$SCRIPT_DIR/../exodus-py"

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Helper functions
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

log_header() {
    echo -e "${BOLD}${CYAN}$1${NC}"
}

# Check if exodus-py is installed
check_exodus_py() {
    log_info "Checking if exodus-py is installed..."
    if python3 -c "import exodus; print(f'exodus-py version: {exodus.__version__}')" 2>/dev/null; then
        log_success "exodus-py is installed"
        return 0
    else
        log_warning "exodus-py is not installed"
        return 1
    fi
}

# Build and install exodus-py
install_exodus_py() {
    log_info "Building and installing exodus-py..."

    cd "$EXODUS_PY_DIR"

    # Check if install script exists
    if [ -f "install_and_test.sh" ]; then
        log_info "Running install_and_test.sh to build exodus-py..."
        bash install_and_test.sh
    else
        # Fallback to manual build
        log_info "Building exodus-py wheel manually..."

        # Check for uv
        if ! command -v uv &> /dev/null; then
            log_info "Installing uv package manager..."
            curl -LsSf https://astral.sh/uv/install.sh | sh
            export PATH="$HOME/.cargo/bin:$PATH"
        fi

        # Build wheel
        uv venv .venv
        uv pip install maturin
        uv run maturin build --release

        # Install wheel
        WHEEL_FILE=$(find target/wheels -name "exodus_py-*.whl" -type f | sort -r | head -n 1)
        if [ -z "$WHEEL_FILE" ]; then
            log_error "Wheel file not found"
            exit 1
        fi

        uv pip install "$WHEEL_FILE"
    fi

    cd "$SCRIPT_DIR"
    log_success "exodus-py installed successfully"
}

# Build Rust transformation example
build_rust_example() {
    log_info "Building Rust transformation example..."

    cd "$EXODUS_RS_DIR"
    cargo build --release --example 13_transform_large_mesh --features netcdf4

    cd "$SCRIPT_DIR"
    log_success "Rust example built successfully"
}

# Generate test file if it doesn't exist
generate_test_file() {
    local test_file="$1"
    local nodes_x="$2"
    local nodes_y="$3"
    local timesteps="$4"

    if [ -f "$test_file" ]; then
        log_info "Test file already exists: $test_file"
        local file_size=$(du -h "$test_file" | cut -f1)
        log_info "File size: $file_size"
    else
        log_info "Generating test file: $test_file"
        log_info "Parameters: ${nodes_x}x${nodes_y} nodes, $timesteps timesteps"

        python3 "$SCRIPT_DIR/generate_large_exodus.py" \
            -o "$test_file" \
            --nodes-x "$nodes_x" \
            --nodes-y "$nodes_y" \
            --timesteps "$timesteps" \
            --cache-mb 128

        local file_size=$(du -h "$test_file" | cut -f1)
        log_success "Test file generated: $file_size"
    fi
}

# Extract total time from output
extract_time() {
    local output="$1"
    # Look for "TOTAL:" line and extract the time
    echo "$output" | grep "TOTAL:" | awk '{print $2}' | head -1
}

# Run Python transformation
run_python_transform() {
    local input="$1"
    local output="$2"
    local cache_mb="$3"
    local node_chunk="$4"
    local elem_chunk="$5"
    local time_chunk="$6"

    log_info "Running Python transformation..."
    log_info "  Cache: ${cache_mb}MB, Node chunk: ${node_chunk}, Elem chunk: ${elem_chunk}, Time chunk: ${time_chunk}"

    local start_time=$(date +%s)
    local transform_output=$(python3 "$SCRIPT_DIR/transform_large_mesh.py" \
        -i "$input" \
        -o "$output" \
        --cache-mb "$cache_mb" \
        --node-chunk-size "$node_chunk" \
        --element-chunk-size "$elem_chunk" \
        --time-chunk-size "$time_chunk" \
        2>&1)
    local end_time=$(date +%s)

    local total_time=$(extract_time "$transform_output")
    if [ -z "$total_time" ]; then
        # Fallback to measured time if parsing failed
        total_time=$((end_time - start_time))
    fi

    echo "$total_time"
}

# Run Rust transformation
run_rust_transform() {
    local input="$1"
    local output="$2"
    local cache_mb="$3"
    local node_chunk="$4"
    local elem_chunk="$5"
    local time_chunk="$6"

    log_info "Running Rust transformation..."
    log_info "  Cache: ${cache_mb}MB, Node chunk: ${node_chunk}, Elem chunk: ${elem_chunk}, Time chunk: ${time_chunk}"

    local start_time=$(date +%s)
    local transform_output=$(cargo run --release --quiet --manifest-path "$EXODUS_RS_DIR/Cargo.toml" \
        --example 13_transform_large_mesh --features netcdf4 -- \
        --input "$input" \
        --output "$output" \
        --cache-mb "$cache_mb" \
        --node-chunk-size "$node_chunk" \
        --element-chunk-size "$elem_chunk" \
        --time-chunk-size "$time_chunk" \
        2>&1)
    local end_time=$(date +%s)

    local total_time=$(extract_time "$transform_output")
    if [ -z "$total_time" ]; then
        # Fallback to measured time if parsing failed
        total_time=$((end_time - start_time))
    fi

    echo "$total_time"
}

# Print results table
print_results_table() {
    local -n results=$1

    echo ""
    log_header "================================================================================"
    log_header "BENCHMARK RESULTS"
    log_header "================================================================================"
    echo ""

    # Print table header
    printf "${BOLD}%-15s %-15s %-15s | %-15s %-15s | %-15s${NC}\n" \
        "Cache (MB)" "Node Chunk" "Time Chunk" "Python (s)" "Rust (s)" "Speedup"
    echo "--------------------------------------------------------------------------------"

    # Print each result
    for key in "${!results[@]}"; do
        IFS='|' read -r cache_mb node_chunk time_chunk python_time rust_time <<< "${results[$key]}"

        # Calculate speedup
        local speedup="N/A"
        if [ -n "$python_time" ] && [ -n "$rust_time" ] && [ "$rust_time" != "0" ] && [ "$rust_time" != "0.0" ]; then
            speedup=$(awk "BEGIN {printf \"%.2fx\", $python_time / $rust_time}")
        fi

        printf "%-15s %-15s %-15s | %-15s %-15s | %-15s\n" \
            "$cache_mb" "$node_chunk" "$time_chunk" "$python_time" "$rust_time" "$speedup"
    done

    echo "================================================================================"
    echo ""
}

# Main benchmark function
run_benchmarks() {
    local test_file="$1"

    # Define caching strategies (cache_mb)
    local cache_strategies=(64 256)

    # Define chunking strategies (node_chunk, elem_chunk, time_chunk)
    # Format: "node_chunk,elem_chunk,time_chunk"
    local chunk_strategies=(
        "5000,4000,5"
        "20000,15000,20"
    )

    # Results storage: key="cache|node_chunk|time_chunk" value="python_time|rust_time"
    declare -A results

    log_header "Starting benchmark suite..."
    log_info "Test file: $test_file"
    log_info "Cache strategies: ${cache_strategies[*]}"
    log_info "Chunk strategies: ${#chunk_strategies[@]} configurations"
    echo ""

    local total_runs=$((${#cache_strategies[@]} * ${#chunk_strategies[@]} * 2))
    local current_run=0

    for cache_mb in "${cache_strategies[@]}"; do
        for chunk_config in "${chunk_strategies[@]}"; do
            IFS=',' read -r node_chunk elem_chunk time_chunk <<< "$chunk_config"

            log_header "Configuration: Cache=${cache_mb}MB, Chunks=(${node_chunk}, ${elem_chunk}, ${time_chunk})"
            echo ""

            # Run Python transformation
            current_run=$((current_run + 1))
            log_info "[$current_run/$total_runs] Running Python transformation..."

            local py_output="$SCRIPT_DIR/benchmark_py_${cache_mb}_${node_chunk}_${time_chunk}.exo"
            local python_time=$(run_python_transform \
                "$test_file" "$py_output" \
                "$cache_mb" "$node_chunk" "$elem_chunk" "$time_chunk")

            log_success "Python completed in ${python_time}s"

            # Clean up output file to save space
            rm -f "$py_output"

            # Run Rust transformation
            current_run=$((current_run + 1))
            log_info "[$current_run/$total_runs] Running Rust transformation..."

            local rs_output="$SCRIPT_DIR/benchmark_rs_${cache_mb}_${node_chunk}_${time_chunk}.exo"
            local rust_time=$(run_rust_transform \
                "$test_file" "$rs_output" \
                "$cache_mb" "$node_chunk" "$elem_chunk" "$time_chunk")

            log_success "Rust completed in ${rust_time}s"

            # Clean up output file to save space
            rm -f "$rs_output"

            # Store results
            local key="${cache_mb}|${node_chunk}|${time_chunk}"
            results[$key]="${cache_mb}|${node_chunk}|${time_chunk}|${python_time}|${rust_time}"

            echo ""
        done
    done

    # Print results table
    print_results_table results

    # Save results to CSV
    local csv_file="$SCRIPT_DIR/benchmark_results_$(date +%Y%m%d_%H%M%S).csv"
    log_info "Saving results to: $csv_file"

    echo "Cache_MB,Node_Chunk,Time_Chunk,Python_Time_s,Rust_Time_s,Speedup" > "$csv_file"
    for key in "${!results[@]}"; do
        IFS='|' read -r cache_mb node_chunk time_chunk python_time rust_time <<< "${results[$key]}"
        local speedup="N/A"
        if [ -n "$python_time" ] && [ -n "$rust_time" ] && [ "$rust_time" != "0" ] && [ "$rust_time" != "0.0" ]; then
            speedup=$(awk "BEGIN {printf \"%.2f\", $python_time / $rust_time}")
        fi
        echo "$cache_mb,$node_chunk,$time_chunk,$python_time,$rust_time,$speedup" >> "$csv_file"
    done

    log_success "Results saved to: $csv_file"
}

# Main execution
main() {
    log_header "================================================================================"
    log_header "EXODUS TRANSFORMATION BENCHMARK SUITE"
    log_header "================================================================================"
    echo ""

    # Parse command line arguments
    local test_file="${1:-$SCRIPT_DIR/benchmark_test.exo}"
    local nodes_x="${2:-50}"
    local nodes_y="${3:-50}"
    local timesteps="${4:-100}"

    log_info "Configuration:"
    log_info "  Test file: $test_file"
    log_info "  Mesh size: ${nodes_x}x${nodes_y} = $((nodes_x * nodes_y)) nodes"
    log_info "  Time steps: $timesteps"
    echo ""

    # Step 1: Check and install exodus-py if needed
    if ! check_exodus_py; then
        log_warning "exodus-py not found, installing..."
        install_exodus_py
    fi
    echo ""

    # Step 2: Build Rust example
    build_rust_example
    echo ""

    # Step 3: Generate test file if needed
    generate_test_file "$test_file" "$nodes_x" "$nodes_y" "$timesteps"
    echo ""

    # Step 4: Run benchmarks
    run_benchmarks "$test_file"

    log_header "================================================================================"
    log_success "Benchmark suite complete!"
    log_header "================================================================================"
    echo ""
    log_info "To run again with custom parameters:"
    log_info "  $0 <test_file> <nodes_x> <nodes_y> <timesteps>"
    log_info ""
    log_info "Example:"
    log_info "  $0 custom_test.exo 100 100 200"
    echo ""
}

# Run main function
main "$@"
