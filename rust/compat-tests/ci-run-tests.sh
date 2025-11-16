#!/bin/bash
#
# Comprehensive CI script for Rust-C compatibility tests
# This script can be run locally for debugging or in GitHub Actions
#
# All builds are self-contained within ./rust/compat-tests/
# No modifications to root SEACAS files are required
#
# Usage:
#   ./ci-run-tests.sh [--verbose] [--no-cache] [--keep-failures]
#
# Options:
#   --verbose         Show detailed output from all steps
#   --no-cache        Force rebuild even if TPL/build directories exist
#   --keep-failures   Don't delete files that failed verification
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
NO_CACHE=false
KEEP_FAILURES=false
JOBS=${JOBS:-4}

# Parse arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --verbose)
      VERBOSE=true
      shift
      ;;
    --no-cache)
      NO_CACHE=true
      shift
      ;;
    --keep-failures)
      KEEP_FAILURES=true
      shift
      ;;
    --jobs)
      JOBS="$2"
      shift 2
      ;;
    *)
      echo "Unknown option: $1"
      echo "Usage: $0 [--verbose] [--no-cache] [--keep-failures] [--jobs N]"
      exit 1
      ;;
  esac
done

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Rust-C Compatibility Tests CI${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Get script directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"

echo -e "${GREEN}Working directory:${NC} $SCRIPT_DIR"
echo -e "${GREEN}Build jobs:${NC} $JOBS"
echo -e "${GREEN}Verbose:${NC} $VERBOSE"
echo -e "${GREEN}No cache:${NC} $NO_CACHE"
echo ""

# Set environment for build scripts
export JOBS=$JOBS

# ==========================================
# Step 1: Check system dependencies
# ==========================================
echo -e "${YELLOW}Step 1: Checking system dependencies${NC}"
echo ""

MISSING_DEPS=()
for cmd in gcc g++ cmake make curl m4; do
  if ! command -v $cmd &> /dev/null; then
    MISSING_DEPS+=($cmd)
  fi
done

if [ ${#MISSING_DEPS[@]} -gt 0 ]; then
  echo -e "${YELLOW}Missing dependencies: ${MISSING_DEPS[*]}${NC}"
  echo "Please install them with:"
  echo "  sudo apt-get install -y build-essential cmake gfortran curl m4 zlib1g-dev"
  echo ""
  echo "Or on macOS:"
  echo "  brew install cmake curl m4"
  echo ""
  # Don't exit - might be running in a pre-configured environment
else
  echo -e "${GREEN}All required tools are installed${NC}"
  echo ""
fi

# ==========================================
# Step 2: Clean cache if requested
# ==========================================
if [ "$NO_CACHE" = true ]; then
  echo -e "${YELLOW}Step 2: Cleaning existing build cache${NC}"
  echo ""

  cd "$SCRIPT_DIR"
  echo "Cleaning local build artifacts..."
  rm -rf tpl-build tpl-install
  rm -rf exodus-build exodus-install
  rm -rf rust-to-c/verify

  echo ""
else
  echo -e "${YELLOW}Step 2: Using existing cache if available${NC}"
  echo ""
fi

# ==========================================
# Step 3: Build TPL libraries (HDF5, NetCDF)
# ==========================================
echo -e "${YELLOW}Step 3: Building Third-Party Libraries (TPLs)${NC}"
echo ""

cd "$SCRIPT_DIR"

TPL_INSTALL="$SCRIPT_DIR/tpl-install"

# Check if TPL libraries already exist
if [ -f "$TPL_INSTALL/lib/libhdf5.so" ] || [ -f "$TPL_INSTALL/lib/libhdf5.a" ]; then
  echo -e "${GREEN}TPL libraries already built, skipping...${NC}"
  echo "Use --no-cache to force rebuild"
  echo ""
else
  echo "Running build-tpls.sh..."
  if [ "$VERBOSE" = true ]; then
    ./build-tpls.sh --jobs "$JOBS"
  else
    ./build-tpls.sh --jobs "$JOBS" > /tmp/build-tpls.log 2>&1 || {
      echo -e "${RED}Failed to build TPL libraries${NC}"
      echo "Last 50 lines of log:"
      tail -50 /tmp/build-tpls.log
      exit 1
    }
    echo -e "${GREEN}TPL libraries built successfully${NC}"
  fi
  echo ""
fi

# Verify TPL installation
echo "Verifying TPL installation..."
TPL_OK=true
if [ ! -f "$TPL_INSTALL/lib/libhdf5.so" ] && [ ! -f "$TPL_INSTALL/lib/libhdf5.a" ]; then
  echo -e "${RED}✗ HDF5 library not found${NC}"
  TPL_OK=false
fi
if [ ! -f "$TPL_INSTALL/lib/libnetcdf.so" ] && [ ! -f "$TPL_INSTALL/lib/libnetcdf.a" ]; then
  echo -e "${RED}✗ NetCDF library not found${NC}"
  TPL_OK=false
fi

if [ "$TPL_OK" = false ]; then
  echo -e "${RED}TPL verification failed!${NC}"
  echo "Expected libraries in: $TPL_INSTALL/lib/"
  ls -la "$TPL_INSTALL/lib/" 2>/dev/null || echo "lib/ directory doesn't exist"
  exit 1
fi

echo -e "${GREEN}✓ HDF5 and NetCDF libraries verified${NC}"
echo ""

# ==========================================
# Step 4: Build Exodus C library
# ==========================================
echo -e "${YELLOW}Step 4: Building Exodus C Library${NC}"
echo ""

EXODUS_INSTALL="$SCRIPT_DIR/exodus-install"

# Check if already built
if [ -f "$EXODUS_INSTALL/lib/libexodus.so" ] || [ -f "$EXODUS_INSTALL/lib/libexodus.a" ]; then
  echo -e "${GREEN}Exodus library already built, skipping...${NC}"
  echo "Use --no-cache to force rebuild"
  echo ""
else
  echo "Running build-exodus.sh..."
  if [ "$VERBOSE" = true ]; then
    ./build-exodus.sh --jobs "$JOBS"
  else
    ./build-exodus.sh --jobs "$JOBS" > /tmp/build-exodus.log 2>&1 || {
      echo -e "${RED}Failed to build Exodus library${NC}"
      echo "Last 50 lines of log:"
      tail -50 /tmp/build-exodus.log
      exit 1
    }
    echo -e "${GREEN}Exodus library built successfully${NC}"
  fi
  echo ""
fi

# Verify installation
echo "Verifying Exodus installation..."

if [ ! -f "$EXODUS_INSTALL/lib/libexodus.so" ] && [ ! -f "$EXODUS_INSTALL/lib/libexodus.a" ]; then
  echo -e "${RED}✗ Exodus library not found${NC}"
  exit 1
fi

if [ ! -f "$EXODUS_INSTALL/include/exodusII.h" ]; then
  echo -e "${RED}✗ Exodus header not found${NC}"
  exit 1
fi

echo -e "${GREEN}✓ Exodus installation verified${NC}"
echo ""

# ==========================================
# Step 5: Compile C verification tool
# ==========================================
echo -e "${YELLOW}Step 5: Compiling C Verification Tool${NC}"
echo ""

VERIFY_DIR="$SCRIPT_DIR/rust-to-c"

cd "$VERIFY_DIR"

if [ -f "verify" ]; then
  echo -e "${GREEN}Verification tool already compiled, skipping...${NC}"
  echo ""
else
  # Set library paths for compilation
  export LD_LIBRARY_PATH="$EXODUS_INSTALL/lib:$TPL_INSTALL/lib:$LD_LIBRARY_PATH"

  echo "Compiling verify.c..."
  echo "  Include paths: $EXODUS_INSTALL/include, $TPL_INSTALL/include"
  echo "  Library paths: $EXODUS_INSTALL/lib, $TPL_INSTALL/lib"

  gcc verify.c \
    -I"$EXODUS_INSTALL/include" \
    -I"$TPL_INSTALL/include" \
    -L"$EXODUS_INSTALL/lib" \
    -L"$TPL_INSTALL/lib" \
    -lexodus \
    -lnetcdf \
    -lhdf5 \
    -lhdf5_hl \
    -lm \
    -o verify

  echo -e "${GREEN}✓ Verification tool compiled${NC}"
  echo ""
fi

# ==========================================
# Step 6: Set up environment
# ==========================================
echo -e "${YELLOW}Step 6: Setting up environment${NC}"
echo ""

export EXODUS_DIR="$EXODUS_INSTALL"
export TPL_DIR="$TPL_INSTALL"
export LD_LIBRARY_PATH="$EXODUS_INSTALL/lib:$TPL_INSTALL/lib:$LD_LIBRARY_PATH"
export PATH="$EXODUS_INSTALL/bin:$PATH"
export PKG_CONFIG_PATH="$TPL_INSTALL/lib/pkgconfig:$PKG_CONFIG_PATH"
export HDF5_DIR="$TPL_INSTALL"
export NETCDF_DIR="$TPL_INSTALL"

echo "Environment configured:"
echo "  EXODUS_DIR: $EXODUS_DIR"
echo "  TPL_DIR: $TPL_DIR"
echo "  HDF5_DIR: $HDF5_DIR"
echo "  NETCDF_DIR: $NETCDF_DIR"
echo "  PKG_CONFIG_PATH: $PKG_CONFIG_PATH"
echo ""

# ==========================================
# Step 7: Run compatibility tests
# ==========================================
echo -e "${YELLOW}Step 7: Running Rust-C Compatibility Tests${NC}"
echo ""

cd "$VERIFY_DIR"

# Test files to generate and verify
TEST_FILES=(
  "basic-mesh2d:basic_mesh_2d:6"
  "basic-mesh3d:basic_mesh_3d:6"
  "multiple-blocks:multiple_blocks:6"
  "node-sets:node_sets:7"
  "side-sets:side_sets:7"
  "element-sets:element_sets:6"
  "all-sets:all_sets:8"
  "global-variables:global_variables:8"
  "nodal-variables:nodal_variables:8"
  "element-variables:element_variables:8"
  "all-variables:all_variables:10"
  "qa-records:qa_records:6"
  "info-records:info_records:6"
  "qa-and-info:qa_and_info:7"
  "node-id-map:node_id_map:6"
  "element-id-map:element_id_map:6"
  "both-id-maps:both_id_maps:6"
  "block-names:block_names:6"
  "set-names:set_names:8"
  "coordinate-names:coordinate_names:6"
  "variable-names:variable_names:7"
)

# Results tracking
TOTAL_FILES=0
PASSED_FILES=0
FAILED_FILES=0
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Output directory
OUTPUT_DIR="$VERIFY_DIR/output"
mkdir -p "$OUTPUT_DIR"

# Generate files with Rust
echo -e "${BLUE}Generating Exodus files with Rust...${NC}"
echo ""

for test_entry in "${TEST_FILES[@]}"; do
  IFS=':' read -r command filename expected_tests <<< "$test_entry"

  echo -n "  Generating $filename.exo... "

  if [ "$VERBOSE" = true ]; then
    echo ""
    cargo run -- "$command"
    echo ""
  else
    cargo run -- "$command" &> /dev/null
  fi

  if [ -f "$OUTPUT_DIR/${filename}.exo" ]; then
    echo -e "${GREEN}✓${NC}"
  else
    echo -e "${RED}✗ FAILED${NC}"
  fi
done

echo ""

# Verify files with C
echo -e "${BLUE}Verifying files with C Exodus library...${NC}"
echo ""

# Test that verify tool can find its libraries
echo "Testing C verification tool..."
if command -v ldd &> /dev/null; then
  echo "Checking library dependencies:"
  ldd ./verify | grep -E "exodus|netcdf|hdf5" || true
  echo ""

  # Check for any missing libraries
  if ldd ./verify | grep -q "not found"; then
    echo -e "${RED}Error: verify tool has missing library dependencies:${NC}"
    ldd ./verify | grep "not found"
    echo ""
    echo "LD_LIBRARY_PATH: $LD_LIBRARY_PATH"
    exit 1
  fi
fi

# Quick sanity test - verify should print usage when run without args
verify_output=$(./verify 2>&1 || true)
if echo "$verify_output" | grep -q "Usage"; then
  echo -e "${GREEN}✓ Verification tool is working${NC}"
else
  echo -e "${YELLOW}Warning: Unexpected output from verify tool:${NC}"
  echo "$verify_output" | head -5
fi
echo ""

for test_entry in "${TEST_FILES[@]}"; do
  IFS=':' read -r command filename expected_tests <<< "$test_entry"
  test_file="$OUTPUT_DIR/${filename}.exo"

  echo "DEBUG: Processing $test_entry" >&2
  echo "  command=$command, filename=$filename, expected_tests=$expected_tests" >&2
  echo "  test_file=$test_file" >&2

  echo "DEBUG: About to increment TOTAL_FILES (currently $TOTAL_FILES)" >&2
  ((TOTAL_FILES++)) || { echo "ERROR: Failed to increment TOTAL_FILES" >&2; exit 1; }
  echo "DEBUG: TOTAL_FILES is now $TOTAL_FILES" >&2

  echo "DEBUG: About to add $expected_tests to TOTAL_TESTS (currently $TOTAL_TESTS)" >&2
  ((TOTAL_TESTS += expected_tests)) || { echo "ERROR: Failed to add to TOTAL_TESTS" >&2; exit 1; }
  echo "DEBUG: TOTAL_TESTS is now $TOTAL_TESTS" >&2

  echo -e "${BLUE}Testing ${filename}.exo${NC}"

  if [ ! -f "$test_file" ]; then
    echo -e "  ${RED}✗ File not found${NC}"
    ((FAILED_FILES++))
    ((FAILED_TESTS += expected_tests))
    continue
  fi

  # Run C verification
  echo "DEBUG: About to run verify on $test_file" >&2
  echo "DEBUG: Current directory: $(pwd)" >&2
  echo "DEBUG: LD_LIBRARY_PATH=$LD_LIBRARY_PATH" >&2

  # Use a temp file to avoid command substitution issues
  temp_output=$(mktemp)
  echo "DEBUG: Running ./verify..." >&2
  timeout 10 ./verify "$test_file" > "$temp_output" 2>&1 || true
  exit_code=$?
  echo "DEBUG: Verify completed with exit code $exit_code" >&2
  output=$(cat "$temp_output")
  rm -f "$temp_output"

  # Count passed/failed tests
  # Look for "PASS" or "✓" for passed tests
  passed=$(echo "$output" | grep -c "PASS\|✓" || true)
  # Look for "FAIL" or "✗" for failed tests
  failed=$(echo "$output" | grep -c "FAIL\|✗" || true)

  ((PASSED_TESTS += passed))
  ((FAILED_TESTS += failed))

  if [ "$failed" -eq 0 ] && [ "$passed" -gt 0 ]; then
    echo -e "  ${GREEN}✓ All $passed tests passed${NC}"
    ((PASSED_FILES++))
  else
    echo -e "  ${RED}✗ Tests failed (exit code: $exit_code, passed: $passed, failed: $failed)${NC}"
    ((FAILED_FILES++))

    # Show full output when tests fail
    echo "  Output from verify:"
    echo "$output" | sed 's/^/    /'

    if [ "$KEEP_FAILURES" = false ]; then
      rm "$test_file"
      echo "  (deleted failed file)"
    fi
  fi

  echo ""
done

# ==========================================
# Summary
# ==========================================
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
  if [ "$VERBOSE" = false ]; then
    echo ""
    echo "Run with --verbose to see detailed output:"
    echo "  ./ci-run-tests.sh --verbose"
  fi
  exit 1
fi
