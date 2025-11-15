#!/bin/bash
#
# Comprehensive CI script for Rust-C compatibility tests
# This script can be run locally for debugging or in GitHub Actions
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

# Get script directory and SEACAS root
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
SEACAS_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo -e "${GREEN}SEACAS root:${NC} $SEACAS_ROOT"
echo -e "${GREEN}Build jobs:${NC} $JOBS"
echo -e "${GREEN}Verbose:${NC} $VERBOSE"
echo -e "${GREEN}No cache:${NC} $NO_CACHE"
echo ""

# ==========================================
# Step 1: Install system dependencies (if needed)
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

  cd "$SEACAS_ROOT"
  echo "Cleaning TPL build artifacts..."
  find TPL -mindepth 2 -maxdepth 2 -type d -exec rm -rf {} + 2>/dev/null || true
  rm -rf lib include bin

  echo "Cleaning SEACAS build directory..."
  rm -rf build-compat

  echo ""
else
  echo -e "${YELLOW}Step 2: Checking for incomplete cache${NC}"
  echo ""

  cd "$SEACAS_ROOT"

  # Check if we have a partial build that needs cleaning
  HDF5_INCOMPLETE=false
  if [ -d "TPL/hdf5/hdf5_1.14.6" ]; then
    if [ ! -f "lib/libhdf5.a" ] && [ ! -f "lib/libhdf5.so" ]; then
      HDF5_INCOMPLETE=true
    fi
  fi

  if [ "$HDF5_INCOMPLETE" = true ]; then
    echo "TPL build exists but appears incomplete, cleaning built artifacts..."
    find TPL -mindepth 2 -maxdepth 2 -type d -exec rm -rf {} + 2>/dev/null || true
    rm -rf build-compat lib include bin
    echo ""
  else
    echo "No incomplete cache detected"
    echo ""
  fi
fi

# ==========================================
# Step 3: Build TPL libraries (HDF5, NetCDF)
# ==========================================
echo -e "${YELLOW}Step 3: Building Third-Party Libraries (TPLs)${NC}"
echo ""

cd "$SEACAS_ROOT"

# Set environment variables for TPL build
export CGNS=NO
export MATIO=NO
export JOBS=$JOBS

# Check if TPL libraries already exist
if [ -f "lib/libhdf5.a" ] || [ -f "lib/libhdf5.so" ]; then
  echo -e "${GREEN}TPL libraries already built, skipping...${NC}"
  echo "Use --no-cache to force rebuild"
  echo ""
else
  echo "Running install-tpl.sh to build HDF5 and NetCDF..."
  if [ "$VERBOSE" = true ]; then
    bash ./install-tpl.sh
  else
    bash ./install-tpl.sh > /tmp/install-tpl.log 2>&1 || {
      echo -e "${RED}Failed to build TPL libraries${NC}"
      echo "Last 50 lines of log:"
      tail -50 /tmp/install-tpl.log
      exit 1
    }
    echo -e "${GREEN}TPL libraries built successfully${NC}"
  fi
  echo ""
fi

# Verify TPL installation
echo "Verifying TPL installation..."
TPL_OK=true
if [ ! -f "$SEACAS_ROOT/lib/libhdf5.a" ] && [ ! -f "$SEACAS_ROOT/lib/libhdf5.so" ]; then
  echo -e "${RED}✗ HDF5 library not found${NC}"
  TPL_OK=false
fi
if [ ! -f "$SEACAS_ROOT/lib/libnetcdf.a" ] && [ ! -f "$SEACAS_ROOT/lib/libnetcdf.so" ]; then
  echo -e "${RED}✗ NetCDF library not found${NC}"
  TPL_OK=false
fi

if [ "$TPL_OK" = false ]; then
  echo -e "${RED}TPL verification failed!${NC}"
  echo "Expected libraries in: $SEACAS_ROOT/lib/"
  ls -la "$SEACAS_ROOT/lib/" 2>/dev/null || echo "lib/ directory doesn't exist"
  exit 1
fi

echo -e "${GREEN}✓ HDF5 and NetCDF libraries verified${NC}"
echo ""

# ==========================================
# Step 4: Build SEACAS C Exodus library
# ==========================================
echo -e "${YELLOW}Step 4: Building SEACAS C Exodus Library${NC}"
echo ""

BUILD_DIR="$SEACAS_ROOT/build-compat"

# Check if already built
if [ -f "$BUILD_DIR/install/lib/libexodus.a" ] || [ -f "$BUILD_DIR/install/lib/libexodus.so" ]; then
  echo -e "${GREEN}SEACAS Exodus library already built, skipping...${NC}"
  echo "Use --no-cache to force rebuild"
  echo ""
else
  mkdir -p "$BUILD_DIR"
  cd "$BUILD_DIR"

  echo "Configuring SEACAS build with CMake..."

  # Use the SEACAS_ROOT as the install path for TPLs
  # This is where install-tpl.sh actually installs them
  if [ "$VERBOSE" = true ]; then
    cmake \
      -DCMAKE_INSTALL_PREFIX="$BUILD_DIR/install" \
      -DTPL_ENABLE_Netcdf=ON \
      -DTPL_ENABLE_HDF5=ON \
      -DNetCDF_ROOT="$SEACAS_ROOT" \
      -DHDF5_ROOT="$SEACAS_ROOT" \
      -DSEACASProj_ENABLE_ALL_PACKAGES=OFF \
      -DSEACASProj_ENABLE_SEACASExodus=ON \
      -DSEACASProj_ENABLE_TESTS=OFF \
      ..
  else
    cmake \
      -DCMAKE_INSTALL_PREFIX="$BUILD_DIR/install" \
      -DTPL_ENABLE_Netcdf=ON \
      -DTPL_ENABLE_HDF5=ON \
      -DNetCDF_ROOT="$SEACAS_ROOT" \
      -DHDF5_ROOT="$SEACAS_ROOT" \
      -DSEACASProj_ENABLE_ALL_PACKAGES=OFF \
      -DSEACASProj_ENABLE_SEACASExodus=ON \
      -DSEACASProj_ENABLE_TESTS=OFF \
      .. > /tmp/cmake-config.log 2>&1 || {
      echo -e "${RED}CMake configuration failed${NC}"
      echo "Last 50 lines of log:"
      tail -50 /tmp/cmake-config.log
      exit 1
    }
  fi

  echo ""
  echo "Building Exodus library..."

  if [ "$VERBOSE" = true ]; then
    make -j"$JOBS"
  else
    make -j"$JOBS" > /tmp/make-build.log 2>&1 || {
      echo -e "${RED}Build failed${NC}"
      echo "Last 50 lines of log:"
      tail -50 /tmp/make-build.log
      exit 1
    }
  fi

  echo ""
  echo "Installing Exodus library..."

  if [ "$VERBOSE" = true ]; then
    make install
  else
    make install > /tmp/make-install.log 2>&1 || {
      echo -e "${RED}Installation failed${NC}"
      echo "Last 50 lines of log:"
      tail -50 /tmp/make-install.log
      exit 1
    }
  fi

  echo -e "${GREEN}✓ SEACAS Exodus library built successfully${NC}"
  echo ""
fi

# ==========================================
# Step 5: Compile C verification tool
# ==========================================
echo -e "${YELLOW}Step 5: Compiling C Verification Tool${NC}"
echo ""

VERIFY_DIR="$SCRIPT_DIR/rust-to-c"
EXODUS_DIR="$BUILD_DIR/install"

cd "$VERIFY_DIR"

if [ -f "verify" ]; then
  echo -e "${GREEN}Verification tool already compiled, skipping...${NC}"
  echo ""
else
  # Set library paths for compilation
  export LD_LIBRARY_PATH="$EXODUS_DIR/lib:$SEACAS_ROOT/lib:$LD_LIBRARY_PATH"

  echo "Compiling verify.c..."
  gcc verify.c \
    -I"$EXODUS_DIR/include" \
    -L"$EXODUS_DIR/lib" \
    -L"$SEACAS_ROOT/lib" \
    -lexodus \
    -lnetcdf \
    -lhdf5 \
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

export EXODUS_DIR="$BUILD_DIR/install"
export LD_LIBRARY_PATH="$EXODUS_DIR/lib:$SEACAS_ROOT/lib:$LD_LIBRARY_PATH"
export PATH="$EXODUS_DIR/bin:$PATH"

echo "Environment configured:"
echo "  EXODUS_DIR: $EXODUS_DIR"
echo "  LD_LIBRARY_PATH: $LD_LIBRARY_PATH"
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

for test_entry in "${TEST_FILES[@]}"; do
  IFS=':' read -r command filename expected_tests <<< "$test_entry"
  test_file="$OUTPUT_DIR/${filename}.exo"

  ((TOTAL_FILES++))
  ((TOTAL_TESTS += expected_tests))

  echo -e "${BLUE}Testing ${filename}.exo${NC}"

  if [ ! -f "$test_file" ]; then
    echo -e "  ${RED}✗ File not found${NC}"
    ((FAILED_FILES++))
    ((FAILED_TESTS += expected_tests))
    continue
  fi

  # Run C verification
  if [ "$VERBOSE" = true ]; then
    output=$("./verify" "$test_file" 2>&1)
    echo "$output"
  else
    output=$("./verify" "$test_file" 2>&1 || true)
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
