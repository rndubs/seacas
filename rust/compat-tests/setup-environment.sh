#!/bin/bash
#
# Setup script for C compatibility testing environment
#
# This script:
# 1. Builds third-party libraries (HDF5, NetCDF) locally
# 2. Builds the Exodus C library locally
# 3. Compiles the C verification tool
#
# All builds are self-contained within ./rust/compat-tests/
# No modifications to root SEACAS files are required
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

# Get script directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"

echo -e "${GREEN}Working directory:${NC} $SCRIPT_DIR"
echo -e "${GREEN}Build jobs:${NC} $JOBS"
echo ""

# Set environment for build scripts
export JOBS=$JOBS

# Step 1: Build TPL libraries (HDF5, NetCDF)
echo -e "${YELLOW}Step 1: Building Third-Party Libraries (TPLs)${NC}"
echo ""

TPL_SCRIPT="$SCRIPT_DIR/build-tpls.sh"

if [ ! -f "$TPL_SCRIPT" ]; then
  echo -e "${RED}✗ TPL build script not found: $TPL_SCRIPT${NC}"
  exit 1
fi

if [ "$CLEAN_BUILD" = true ]; then
  "$TPL_SCRIPT" --jobs "$JOBS" --clean
else
  "$TPL_SCRIPT" --jobs "$JOBS"
fi

# Step 2: Build Exodus C library
echo -e "${YELLOW}Step 2: Building Exodus C Library${NC}"
echo ""

EXODUS_SCRIPT="$SCRIPT_DIR/build-exodus.sh"

if [ ! -f "$EXODUS_SCRIPT" ]; then
  echo -e "${RED}✗ Exodus build script not found: $EXODUS_SCRIPT${NC}"
  exit 1
fi

if [ "$CLEAN_BUILD" = true ]; then
  "$EXODUS_SCRIPT" --jobs "$JOBS" --clean
else
  "$EXODUS_SCRIPT" --jobs "$JOBS"
fi

# Step 3: Compile C verification tool
echo -e "${YELLOW}Step 3: Compiling C Verification Tool${NC}"
echo ""

VERIFY_DIR="$SCRIPT_DIR/rust-to-c"
TPL_INSTALL="$SCRIPT_DIR/tpl-install"
EXODUS_INSTALL="$SCRIPT_DIR/exodus-install"

cd "$VERIFY_DIR"

# Set library paths for compilation
export LD_LIBRARY_PATH="$EXODUS_INSTALL/lib:$TPL_INSTALL/lib:$LD_LIBRARY_PATH"

if [ -f "verify" ]; then
  echo -e "${GREEN}Verification tool already exists, recompiling...${NC}"
  rm -f verify
fi

echo "Compiling verify.c..."
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

if [ ! -f "verify" ]; then
  echo -e "${RED}✗ Failed to compile verification tool${NC}"
  exit 1
fi

echo -e "${GREEN}✓ Verification tool compiled${NC}"
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

export EXODUS_DIR="$EXODUS_INSTALL"
export TPL_DIR="$TPL_INSTALL"
export LD_LIBRARY_PATH="$EXODUS_INSTALL/lib:$TPL_INSTALL/lib:\$LD_LIBRARY_PATH"
export PATH="$EXODUS_INSTALL/bin:\$PATH"

echo "C compatibility test environment configured"
echo "  Exodus library: \$EXODUS_DIR"
echo "  TPL libraries:  \$TPL_DIR"
EOF

chmod +x "$ENV_FILE"

echo "Created environment file: $ENV_FILE"
echo ""

# Summary
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Setup Complete!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "TPL libraries (HDF5, NetCDF) installed in:"
echo "  $TPL_INSTALL"
echo ""
echo "Exodus C library installed in:"
echo "  $EXODUS_INSTALL"
echo ""
echo "C verification tool compiled:"
echo "  $VERIFY_DIR/verify"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo "  1. Source the environment file:"
echo "     ${BLUE}source $ENV_FILE${NC}"
echo ""
echo "  2. Run the compatibility tests:"
echo "     ${BLUE}./run-compat-tests.sh${NC}"
echo ""
