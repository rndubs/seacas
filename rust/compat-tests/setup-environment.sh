#!/bin/bash
#
# Setup script for C compatibility testing environment
#
# This script:
# 1. Installs third-party libraries (HDF5, NetCDF) via install-tpl.sh
# 2. Builds the SEACAS C Exodus library
# 3. Compiles the C verification tool
#
# Usage:
#   ./setup-environment.sh [--jobs N] [--clean]
#
# Options:
#   --jobs N    Number of parallel build jobs (default: 4)
#   --clean     Remove existing build directories before building
#

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
JOBS=4
CLEAN_BUILD=false

# Parse arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --jobs)
      JOBS="$2"
      shift 2
      ;;
    --clean)
      CLEAN_BUILD=true
      shift
      ;;
    *)
      echo "Unknown option: $1"
      echo "Usage: $0 [--jobs N] [--clean]"
      exit 1
      ;;
  esac
done

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}C Compatibility Test Environment Setup${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Get script directory and SEACAS root
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
SEACAS_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo -e "${GREEN}SEACAS root:${NC} $SEACAS_ROOT"
echo -e "${GREEN}Build jobs:${NC} $JOBS"
echo ""

# Step 1: Install TPLs
echo -e "${YELLOW}Step 1: Installing Third-Party Libraries (TPLs)${NC}"
echo "Building HDF5, NetCDF, and other dependencies..."
echo ""

cd "$SEACAS_ROOT"

if [ "$CLEAN_BUILD" = true ]; then
  echo "Cleaning TPL build directory..."
  rm -rf TPL
fi

# Set environment variables for TPL build
export CGNS=NO
export MATIO=NO
export JOBS=$JOBS

# Run TPL installer
if [ ! -d "TPL" ]; then
  echo "Running install-tpl.sh..."
  bash ./install-tpl.sh
  echo ""
else
  echo "TPL directory already exists, skipping TPL build."
  echo "Use --clean to rebuild TPLs."
  echo ""
fi

# Step 2: Build SEACAS C Exodus library
echo -e "${YELLOW}Step 2: Building SEACAS C Exodus Library${NC}"
echo ""

BUILD_DIR="$SEACAS_ROOT/build-compat"

if [ "$CLEAN_BUILD" = true ]; then
  echo "Cleaning build directory..."
  rm -rf "$BUILD_DIR"
fi

mkdir -p "$BUILD_DIR"
cd "$BUILD_DIR"

# Configure with CMake
if [ ! -f "CMakeCache.txt" ]; then
  echo "Configuring SEACAS build..."
  cmake \
    -DCMAKE_INSTALL_PREFIX="$BUILD_DIR/install" \
    -DTPL_ENABLE_Netcdf=ON \
    -DTPL_ENABLE_HDF5=ON \
    -DNetCDF_ROOT="$SEACAS_ROOT/TPL/netcdf-4.9.2" \
    -DHDF5_ROOT="$SEACAS_ROOT/TPL/hdf5-1.14.6" \
    -DSEACASProj_ENABLE_ALL_PACKAGES=OFF \
    -DSEACASProj_ENABLE_SEACASExodus=ON \
    -DSEACASProj_ENABLE_TESTS=OFF \
    ..
  echo ""
else
  echo "CMake cache exists, skipping configuration."
  echo "Use --clean to reconfigure."
  echo ""
fi

# Build
echo "Building Exodus library..."
make -j"$JOBS"
echo ""

# Install
echo "Installing Exodus library..."
make install
echo ""

# Step 3: Compile C verification tool
echo -e "${YELLOW}Step 3: Compiling C Verification Tool${NC}"
echo ""

VERIFY_DIR="$SCRIPT_DIR/rust-to-c"
cd "$VERIFY_DIR"

# Set library paths for compilation
export EXODUS_DIR="$BUILD_DIR/install"
export LD_LIBRARY_PATH="$EXODUS_DIR/lib:$SEACAS_ROOT/TPL/netcdf-4.9.2/lib:$SEACAS_ROOT/TPL/hdf5-1.14.6/lib:$LD_LIBRARY_PATH"

echo "Compiling verify.c..."
gcc verify.c \
  -I"$EXODUS_DIR/include" \
  -L"$EXODUS_DIR/lib" \
  -lexodus \
  -lnetcdf \
  -lhdf5 \
  -lm \
  -o verify

echo ""

# Step 4: Create environment setup file
echo -e "${YELLOW}Step 4: Creating Environment Setup File${NC}"
echo ""

ENV_FILE="$SCRIPT_DIR/env-compat.sh"
cat > "$ENV_FILE" << EOF
# Source this file to set up environment for C compatibility tests
#
# Usage:
#   source ./env-compat.sh
#

export EXODUS_DIR="$BUILD_DIR/install"
export LD_LIBRARY_PATH="$EXODUS_DIR/lib:$SEACAS_ROOT/TPL/netcdf-4.9.2/lib:$SEACAS_ROOT/TPL/hdf5-1.14.6/lib:\$LD_LIBRARY_PATH"
export PATH="$EXODUS_DIR/bin:\$PATH"

echo "C compatibility test environment configured"
echo "  Exodus library: \$EXODUS_DIR"
EOF

chmod +x "$ENV_FILE"

echo "Created environment file: $ENV_FILE"
echo ""

# Summary
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Setup Complete!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "TPL libraries installed in: $SEACAS_ROOT/TPL"
echo "  - HDF5 1.14.6"
echo "  - NetCDF 4.9.2"
echo ""
echo "SEACAS C Exodus library installed in: $EXODUS_DIR"
echo ""
echo "C verification tool compiled: $VERIFY_DIR/verify"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo "  1. Source the environment file:"
echo "     ${BLUE}source $ENV_FILE${NC}"
echo ""
echo "  2. Run the compatibility tests:"
echo "     ${BLUE}./run-compat-tests.sh${NC}"
echo ""
