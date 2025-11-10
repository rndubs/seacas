#!/bin/bash
# Test all Rust-generated files with Rust verifier
# This verifies that Rust-generated files can be read back by Rust

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "======================================"
echo "  Rust Self-Compatibility Test"
echo "======================================"
echo ""
echo "Testing Rust-generated files with Rust verifier..."
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Counters
total=0
passed=0
failed=0

# Test files
test_files=(
    "basic_mesh_2d.exo"
    "basic_mesh_3d.exo"
    "multiple_blocks.exo"
    "node_sets.exo"
    "side_sets.exo"
    "element_sets.exo"
    "all_sets.exo"
    "global_variables.exo"
    "nodal_variables.exo"
    "element_variables.exo"
    "all_variables.exo"
)

# Build verifier if needed
if [ ! -f "$PROJECT_ROOT/c-to-rust/target/debug/exodus-c-to-rust-verifier" ]; then
    echo "Building Rust verifier..."
    cd "$PROJECT_ROOT/c-to-rust"
    cargo build --quiet
    echo ""
fi

# Run tests
for file in "${test_files[@]}"; do
    total=$((total + 1))
    filepath="$PROJECT_ROOT/rust-to-c/output/$file"

    printf "  %-30s " "$file"

    if [ ! -f "$filepath" ]; then
        echo -e "${RED}MISSING${NC}"
        failed=$((failed + 1))
        continue
    fi

    # Run verifier and capture exit code
    if "$PROJECT_ROOT/c-to-rust/target/debug/exodus-c-to-rust-verifier" "$filepath" > /dev/null 2>&1; then
        echo -e "${GREEN}PASS${NC}"
        passed=$((passed + 1))
    else
        echo -e "${RED}FAIL${NC}"
        failed=$((failed + 1))
    fi
done

echo ""
echo "======================================"
echo "  Test Results"
echo "======================================"
echo "  Total:  $total"
echo -e "  ${GREEN}Passed: $passed${NC}"
echo -e "  ${RED}Failed: $failed${NC}"
echo "======================================"
echo ""

if [ $failed -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ Some tests failed${NC}"
    exit 1
fi
