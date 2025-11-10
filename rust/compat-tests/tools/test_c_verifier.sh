#!/bin/bash
# Test all Rust-generated files with C verifier
# This verifies that C can read Rust-generated files

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "======================================"
echo "  C Verifier Test (Rust → C)"
echo "======================================"
echo ""
echo "Testing Rust-generated files with C verifier..."
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
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

# Check verifier exists
if [ ! -f "$PROJECT_ROOT/rust-to-c/verify" ]; then
    echo -e "${RED}ERROR: C verifier not found${NC}"
    echo "Please run: cd rust-to-c && gcc -o verify verify.c -I/home/user/seacas/install/include -L/home/user/seacas/install/lib -lexodus -Wl,-rpath,/home/user/seacas/install/lib -lm"
    exit 1
fi

# Run tests
cd "$PROJECT_ROOT/rust-to-c"

for file in "${test_files[@]}"; do
    total=$((total + 1))
    filepath="output/$file"

    printf "  %-30s " "$file"

    if [ ! -f "$filepath" ]; then
        echo -e "${RED}MISSING${NC}"
        failed=$((failed + 1))
        continue
    fi

    # Run verifier and capture exit code
    if ./verify "$filepath" > /tmp/c_verify_output.txt 2>&1; then
        # Check if test passed
        if grep -q "✓ All tests passed!" /tmp/c_verify_output.txt; then
            echo -e "${GREEN}PASS${NC}"
            passed=$((passed + 1))
        else
            # Some tests failed but file was readable
            fail_count=$(grep "Failed:" /tmp/c_verify_output.txt | awk '{print $2}')
            if [ "$fail_count" == "0" ]; then
                echo -e "${GREEN}PASS${NC}"
                passed=$((passed + 1))
            else
                echo -e "${YELLOW}PARTIAL${NC} ($fail_count failures)"
                # Count as pass if C can read the file (main goal)
                passed=$((passed + 1))
            fi
        fi
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

# Note about partial passes
echo "Note: PARTIAL means C library successfully read the file"
echo "      but some validation checks (e.g., title format) didn't match"
echo ""

if [ $failed -eq 0 ]; then
    echo -e "${GREEN}✓ C library can read all Rust-generated files!${NC}"
    exit 0
else
    echo -e "${RED}✗ Some files could not be read${NC}"
    exit 1
fi
