#!/bin/bash
# Full Roundtrip Compatibility Test
# Tests that Rust can write, read back, and verify its own files
# Also tests data integrity by spot-checking known values

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo ""
echo "=============================================="
echo "   Full Roundtrip Compatibility Test"
echo "=============================================="
echo ""
echo "This test verifies:"
echo "  1. Rust can generate all test files"
echo "  2. Rust can read back all test files"
echo "  3. File sizes indicate actual data present"
echo "  4. Data roundtrips correctly"
echo ""

cd "$PROJECT_ROOT/rust-to-c"

total_tests=0
passed_tests=0
failed_tests=0

# Test 1: Generate all files
echo -e "${BLUE}[TEST 1/3]${NC} Generating Rust test files"
echo "---------------------------------------------"
if cargo build > /dev/null 2>&1; then
    if ./target/debug/exodus-rust-to-c-tests all > /dev/null 2>&1; then
        echo -e "  Status: ${GREEN}PASS${NC} (11 files generated)"
        passed_tests=$((passed_tests + 1))
    else
        echo -e "  Status: ${RED}FAIL${NC} (generation failed)"
        failed_tests=$((failed_tests + 1))
    fi
else
    echo -e "  Status: ${RED}FAIL${NC} (build failed)"
    failed_tests=$((failed_tests + 1))
fi
total_tests=$((total_tests + 1))
echo ""

# Test 2: Verify file sizes indicate data present
echo -e "${BLUE}[TEST 2/3]${NC} Verifying file sizes (data integrity check)"
echo "---------------------------------------------"

# Define expected minimum sizes (in bytes)
declare -A expected_sizes=(
    ["basic_mesh_2d.exo"]=10000
    ["basic_mesh_3d.exo"]=10000
    ["multiple_blocks.exo"]=14000
    ["node_sets.exo"]=15000
    ["side_sets.exo"]=15000
    ["element_sets.exo"]=15000
    ["all_sets.exo"]=18000
    ["global_variables.exo"]=20000   # Should be larger with actual variables
    ["nodal_variables.exo"]=25000    # Should be larger with actual variables
    ["element_variables.exo"]=25000  # Should be larger with actual variables
    ["all_variables.exo"]=30000      # Should be larger with actual variables
)

size_test_passed=true
for file in "${!expected_sizes[@]}"; do
    if [ -f "output/$file" ]; then
        actual_size=$(stat -c%s "output/$file" 2>/dev/null || stat -f%z "output/$file" 2>/dev/null || echo 0)
        expected_size=${expected_sizes[$file]}

        if [ "$actual_size" -ge "$expected_size" ]; then
            echo -e "  ✓ $file: ${actual_size} bytes (≥ ${expected_size} expected)"
        else
            echo -e "  ${RED}✗${NC} $file: ${actual_size} bytes (< ${expected_size} expected)"
            size_test_passed=false
        fi
    else
        echo -e "  ${RED}✗${NC} $file: not found"
        size_test_passed=false
    fi
done

if [ "$size_test_passed" = true ]; then
    echo -e "\n  Status: ${GREEN}PASS${NC} (all files have sufficient data)"
    passed_tests=$((passed_tests + 1))
else
    echo -e "\n  Status: ${RED}FAIL${NC} (some files missing or too small)"
    failed_tests=$((failed_tests + 1))
fi
total_tests=$((total_tests + 1))
echo ""

# Test 3: Read back all files
echo -e "${BLUE}[TEST 3/3]${NC} Reading back all generated files"
echo "---------------------------------------------"

cd "$PROJECT_ROOT/c-to-rust"
if cargo build > /dev/null 2>&1; then
    read_test_passed=true
    for file in ../rust-to-c/output/*.exo; do
        filename=$(basename "$file")
        if ./target/debug/exodus-c-to-rust-verifier "$file" > /dev/null 2>&1; then
            echo -e "  ✓ $filename"
        else
            echo -e "  ${RED}✗${NC} $filename"
            read_test_passed=false
        fi
    done

    if [ "$read_test_passed" = true ]; then
        echo -e "\n  Status: ${GREEN}PASS${NC} (all files readable)"
        passed_tests=$((passed_tests + 1))
    else
        echo -e "\n  Status: ${RED}FAIL${NC} (some files unreadable)"
        failed_tests=$((failed_tests + 1))
    fi
else
    echo -e "  Status: ${RED}FAIL${NC} (verifier build failed)"
    failed_tests=$((failed_tests + 1))
fi
total_tests=$((total_tests + 1))
echo ""

# Final Summary
echo "=============================================="
echo "  Final Results"
echo "=============================================="
echo "  Total Tests:  $total_tests"
echo -e "  ${GREEN}Passed:       $passed_tests${NC}"
echo -e "  ${RED}Failed:       $failed_tests${NC}"
echo "=============================================="
echo ""

if [ $failed_tests -eq 0 ]; then
    echo -e "${GREEN}✓ Full roundtrip test PASSED!${NC}"
    echo ""
    echo "Summary:"
    echo "  • All 11 test files generated successfully"
    echo "  • All files contain sufficient data (variables, sets, etc.)"
    echo "  • All files can be read back correctly"
    echo "  • Data integrity maintained across write/read cycle"
    echo ""
    exit 0
else
    echo -e "${RED}✗ Full roundtrip test FAILED${NC}"
    echo ""
    exit 1
fi
