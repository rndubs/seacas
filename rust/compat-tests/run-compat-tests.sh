#!/bin/bash
#
# Run C compatibility tests for exodus-rs
#
# This script:
# 1. Generates Exodus files using Rust exodus-rs library
# 2. Verifies each file with the C Exodus library
# 3. Reports results
#
# Usage:
#   ./run-compat-tests.sh [--verbose] [--keep-failures]
#
# Options:
#   --verbose         Show detailed output from each test
#   --keep-failures   Don't delete files that failed verification
#
# Prerequisites:
#   Run ./setup-environment.sh first to build the C library and verification tool
#

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
VERBOSE=false
KEEP_FAILURES=false

# Parse arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --verbose)
      VERBOSE=true
      shift
      ;;
    --keep-failures)
      KEEP_FAILURES=true
      shift
      ;;
    *)
      echo "Unknown option: $1"
      echo "Usage: $0 [--verbose] [--keep-failures]"
      exit 1
      ;;
  esac
done

# Get script directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
SEACAS_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}C Compatibility Tests for exodus-rs${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Check prerequisites
VERIFY_TOOL="$SCRIPT_DIR/rust-to-c/verify"
if [ ! -f "$VERIFY_TOOL" ]; then
  echo -e "${RED}Error: C verification tool not found${NC}"
  echo "Please run ./setup-environment.sh first"
  exit 1
fi

# Check if environment is set up
if [ -z "$EXODUS_DIR" ]; then
  echo -e "${YELLOW}Warning: EXODUS_DIR not set${NC}"
  echo "Sourcing environment file..."
  ENV_FILE="$SCRIPT_DIR/env-compat.sh"
  if [ -f "$ENV_FILE" ]; then
    source "$ENV_FILE"
  else
    echo -e "${RED}Error: Environment file not found${NC}"
    echo "Please run ./setup-environment.sh first"
    exit 1
  fi
  echo ""
fi

# Test files to generate and verify
# Format: "test_name:number_of_expected_tests"
TEST_FILES=(
  "basic_mesh_2d:6"
  "basic_mesh_3d:6"
  "multiple_blocks:6"
  "node_sets:7"
  "side_sets:7"
  "element_sets:6"
  "all_sets:8"
  "global_variables:8"
  "nodal_variables:8"
  "element_variables:8"
  "all_variables:10"
)

# Results tracking
TOTAL_FILES=0
PASSED_FILES=0
FAILED_FILES=0
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Output directory
OUTPUT_DIR="$SCRIPT_DIR/rust-to-c/output"
mkdir -p "$OUTPUT_DIR"

# Step 1: Generate files with Rust
echo -e "${YELLOW}Step 1: Generating Exodus files with Rust${NC}"
echo ""

cd "$SCRIPT_DIR/rust-to-c"

for test_entry in "${TEST_FILES[@]}"; do
  test_name="${test_entry%%:*}"

  echo -n "  Generating $test_name.exo... "

  if [ "$VERBOSE" = true ]; then
    echo ""
    cargo run --features netcdf4 -- "$test_name"
    echo ""
  else
    cargo run --features netcdf4 -- "$test_name" &> /dev/null
  fi

  if [ -f "$OUTPUT_DIR/${test_name}.exo" ]; then
    echo -e "${GREEN}✓${NC}"
  else
    echo -e "${RED}✗ FAILED${NC}"
  fi
done

echo ""

# Step 2: Verify files with C
echo -e "${YELLOW}Step 2: Verifying files with C Exodus library${NC}"
echo ""

for test_entry in "${TEST_FILES[@]}"; do
  test_name="${test_entry%%:*}"
  expected_tests="${test_entry##*:}"
  test_file="$OUTPUT_DIR/${test_name}.exo"

  ((TOTAL_FILES++))
  ((TOTAL_TESTS += expected_tests))

  echo -e "${BLUE}Testing ${test_name}.exo${NC}"

  if [ ! -f "$test_file" ]; then
    echo -e "  ${RED}✗ File not found${NC}"
    ((FAILED_FILES++))
    ((FAILED_TESTS += expected_tests))
    continue
  fi

  # Run C verification
  if [ "$VERBOSE" = true ]; then
    output=$("$VERIFY_TOOL" "$test_file" 2>&1)
    echo "$output"
  else
    output=$("$VERIFY_TOOL" "$test_file" 2>&1 || true)
  fi

  # Count passed/failed tests
  passed=$(echo "$output" | grep -c "✓" || true)
  failed=$(echo "$output" | grep -c "✗" || true)

  ((PASSED_TESTS += passed))
  ((FAILED_TESTS += failed))

  if [ "$failed" -eq 0 ]; then
    echo -e "  ${GREEN}✓ All $passed tests passed${NC}"
    ((PASSED_FILES++))
  else
    echo -e "  ${RED}✗ $failed of $expected_tests tests failed${NC}"
    ((FAILED_FILES++))

    if [ "$VERBOSE" = false ]; then
      echo "$output" | grep "✗"
    fi

    if [ "$KEEP_FAILURES" = false ]; then
      rm "$test_file"
      echo "  (deleted failed file)"
    fi
  fi

  echo ""
done

# Step 3: Summary
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Test Summary${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

echo "Files:"
echo "  Total:  $TOTAL_FILES"
echo -e "  Passed: ${GREEN}$PASSED_FILES${NC}"
if [ "$FAILED_FILES" -gt 0 ]; then
  echo -e "  Failed: ${RED}$FAILED_FILES${NC}"
else
  echo "  Failed: 0"
fi
echo ""

echo "Individual Tests:"
echo "  Total:  $TOTAL_TESTS"
echo -e "  Passed: ${GREEN}$PASSED_TESTS${NC}"
if [ "$FAILED_TESTS" -gt 0 ]; then
  echo -e "  Failed: ${RED}$FAILED_TESTS${NC}"
else
  echo "  Failed: 0"
fi
echo ""

# Overall result
if [ "$FAILED_TESTS" -eq 0 ]; then
  echo -e "${GREEN}========================================${NC}"
  echo -e "${GREEN}ALL TESTS PASSED ✓${NC}"
  echo -e "${GREEN}========================================${NC}"
  echo ""
  echo "The Rust exodus-rs library is 100% compatible"
  echo "with the C Exodus library for all tested features."
  exit 0
else
  echo -e "${RED}========================================${NC}"
  echo -e "${RED}SOME TESTS FAILED ✗${NC}"
  echo -e "${RED}========================================${NC}"
  echo ""
  echo "Please review the failures above."
  exit 1
fi
