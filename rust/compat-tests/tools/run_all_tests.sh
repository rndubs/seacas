#!/bin/bash
#
# Run all compatibility tests
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPAT_DIR="$(dirname "$SCRIPT_DIR")"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Parse arguments
RUN_RUST_TO_C=true
RUN_C_TO_RUST=true

while [[ $# -gt 0 ]]; do
    case $1 in
        --rust-to-c)
            RUN_C_TO_RUST=false
            shift
            ;;
        --c-to-rust)
            RUN_RUST_TO_C=false
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo
            echo "Options:"
            echo "  --rust-to-c    Run only Rust→C tests"
            echo "  --c-to-rust    Run only C→Rust tests"
            echo "  --help         Show this help message"
            echo
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

echo
echo "========================================"
echo "  Exodus Compatibility Test Suite"
echo "========================================"
echo

# Create output directory
mkdir -p "$COMPAT_DIR/output"

TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Function to run a test
run_test() {
    local test_name="$1"
    local test_command="$2"

    echo -e "${BLUE}[TEST]${NC} $test_name"
    TOTAL_TESTS=$((TOTAL_TESTS + 1))

    if eval "$test_command" > /dev/null 2>&1; then
        echo -e "${GREEN}  ✓ PASS${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        return 0
    else
        echo -e "${RED}  ✗ FAIL${NC}"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        return 1
    fi
}

# Rust → C tests
if [ "$RUN_RUST_TO_C" = true ]; then
    echo
    echo "────────────────────────────────────────"
    echo "  Rust → C Compatibility Tests"
    echo "────────────────────────────────────────"
    echo

    cd "$COMPAT_DIR/rust-to-c"

    # Build if needed
    if [ ! -f "target/release/exodus-rust-to-c-tests" ] || [ ! -f "verify" ]; then
        echo "Building Rust→C tests..."
        cargo build --release --features netcdf4 >/dev/null 2>&1
        if [ -f "$COMPAT_DIR/tools/build_c.sh" ]; then
            bash "$COMPAT_DIR/tools/build_c.sh" >/dev/null 2>&1 || true
        fi
    fi

    # Basic 2D test
    echo "Generating basic_mesh_2d.exo..."
    ./target/release/exodus-rust-to-c-tests basic-mesh2d -o output >/dev/null 2>&1
    run_test "Rust→C: Basic 2D mesh" "./verify output/basic_mesh_2d.exo"

    # Basic 3D test
    echo "Generating basic_mesh_3d.exo..."
    ./target/release/exodus-rust-to-c-tests basic-mesh3d -o output >/dev/null 2>&1
    run_test "Rust→C: Basic 3D mesh" "./verify output/basic_mesh_3d.exo"
fi

# C → Rust tests
if [ "$RUN_C_TO_RUST" = true ]; then
    echo
    echo "────────────────────────────────────────"
    echo "  C → Rust Compatibility Tests"
    echo "────────────────────────────────────────"
    echo

    cd "$COMPAT_DIR/c-to-rust"

    # Build if needed
    if [ ! -f "target/release/exodus-c-to-rust-verifier" ] || [ ! -f "writer" ]; then
        echo "Building C→Rust tests..."
        if [ -f "$COMPAT_DIR/tools/build_c.sh" ]; then
            bash "$COMPAT_DIR/tools/build_c.sh" >/dev/null 2>&1 || true
        fi
        cargo build --release --features netcdf4 >/dev/null 2>&1
    fi

    # Basic 2D test
    if [ -f "writer" ]; then
        echo "Generating c_basic_2d.exo..."
        ./writer basic_2d >/dev/null 2>&1
        run_test "C→Rust: Basic 2D mesh" "./target/release/exodus-c-to-rust-verifier output/c_basic_2d.exo"

        # Basic 3D test
        echo "Generating c_basic_3d.exo..."
        ./writer basic_3d >/dev/null 2>&1
        run_test "C→Rust: Basic 3D mesh" "./target/release/exodus-c-to-rust-verifier output/c_basic_3d.exo"

        # Variables test
        echo "Generating c_with_variables.exo..."
        ./writer with_variables >/dev/null 2>&1
        run_test "C→Rust: With variables" "./target/release/exodus-c-to-rust-verifier output/c_with_variables.exo"
    else
        echo -e "${YELLOW}Warning: C writer not built, skipping C→Rust tests${NC}"
        echo "Run './tools/build_c.sh' to build C programs"
    fi
fi

# Summary
echo
echo "========================================"
echo "  Test Summary"
echo "========================================"
echo "  Total tests:  $TOTAL_TESTS"
echo -e "  ${GREEN}Passed:       $PASSED_TESTS${NC}"
echo -e "  ${RED}Failed:       $FAILED_TESTS${NC}"
echo "========================================"
echo

if [ $FAILED_TESTS -eq 0 ] && [ $TOTAL_TESTS -gt 0 ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    echo
    exit 0
elif [ $TOTAL_TESTS -eq 0 ]; then
    echo -e "${YELLOW}No tests were run${NC}"
    echo
    exit 0
else
    echo -e "${RED}✗ Some tests failed${NC}"
    echo
    exit 1
fi
