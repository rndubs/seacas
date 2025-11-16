#!/bin/bash
#
# Self-contained script to build HDF5 and NetCDF for C compatibility tests
# This script is completely independent of the root SEACAS build system
#
# Usage:
#   ./build-tpls.sh [--jobs N] [--clean]
#

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Versions
HDF5_VERSION="1.14.6"
NETCDF_VERSION="4.9.3"

# Default values
JOBS=${JOBS:-4}
CLEAN=false

# Parse arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --jobs)
      JOBS="$2"
      shift 2
      ;;
    --clean)
      CLEAN=true
      shift
      ;;
    *)
      echo "Unknown option: $1"
      echo "Usage: $0 [--jobs N] [--clean]"
      exit 1
      ;;
  esac
done

# Get script directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
BUILD_ROOT="$SCRIPT_DIR/tpl-build"
INSTALL_DIR="$SCRIPT_DIR/tpl-install"

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Building TPL Libraries${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo "HDF5 version:    $HDF5_VERSION"
echo "NetCDF version:  $NETCDF_VERSION"
echo "Build jobs:      $JOBS"
echo "Install to:      $INSTALL_DIR"
echo ""

# Clean if requested
if [ "$CLEAN" = true ]; then
  echo -e "${YELLOW}Cleaning existing build...${NC}"
  rm -rf "$BUILD_ROOT" "$INSTALL_DIR"
  echo ""
fi

# Create directories
mkdir -p "$BUILD_ROOT"
mkdir -p "$INSTALL_DIR"

# ==========================================
# Build HDF5
# ==========================================
echo -e "${YELLOW}Building HDF5 ${HDF5_VERSION}${NC}"
echo ""

HDF5_BUILD_DIR="$BUILD_ROOT/hdf5"

if [ -f "$INSTALL_DIR/lib/libhdf5.so" ] || [ -f "$INSTALL_DIR/lib/libhdf5.a" ]; then
  echo -e "${GREEN}HDF5 already built, skipping...${NC}"
  echo "Use --clean to rebuild"
  echo ""
else
  # Download HDF5
  HDF5_ARCHIVE="hdf5_${HDF5_VERSION}.tar.gz"
  HDF5_SRC_DIR="$BUILD_ROOT/hdf5-hdf5_${HDF5_VERSION}"

  if [ ! -f "$BUILD_ROOT/$HDF5_ARCHIVE" ]; then
    echo "Downloading HDF5 $HDF5_VERSION..."
    cd "$BUILD_ROOT"
    curl -L -o "$HDF5_ARCHIVE" \
      "https://github.com/HDFGroup/hdf5/archive/refs/tags/hdf5_${HDF5_VERSION}.tar.gz"
    echo ""
  fi

  # Extract
  if [ ! -d "$HDF5_SRC_DIR" ]; then
    echo "Extracting HDF5..."
    cd "$BUILD_ROOT"
    tar -xzf "$HDF5_ARCHIVE"
    echo ""
  fi

  # Configure and build
  mkdir -p "$HDF5_BUILD_DIR"
  cd "$HDF5_BUILD_DIR"

  echo "Configuring HDF5..."
  cmake "$HDF5_SRC_DIR" \
    -DCMAKE_INSTALL_PREFIX="$INSTALL_DIR" \
    -DCMAKE_BUILD_TYPE=Release \
    -DBUILD_SHARED_LIBS=ON \
    -DCMAKE_POSITION_INDEPENDENT_CODE=ON \
    -DBUILD_TESTING=OFF \
    -DHDF5_BUILD_EXAMPLES=OFF \
    -DHDF5_BUILD_TOOLS=OFF \
    -DHDF5_ENABLE_PARALLEL=OFF \
    -DHDF5_ENABLE_Z_LIB_SUPPORT=ON \
    -DHDF5_BUILD_CPP_LIB=OFF \
    -DHDF5_BUILD_FORTRAN=OFF \
    -DHDF5_BUILD_HL_LIB=ON \
    -DHDF5_DISABLE_COMPILER_WARNINGS=ON

  echo ""
  echo "Building HDF5..."
  cmake --build . -j"$JOBS"

  echo ""
  echo "Installing HDF5..."
  cmake --install .

  echo -e "${GREEN}✓ HDF5 built and installed${NC}"
  echo ""
fi

# Verify HDF5 installation
if [ ! -f "$INSTALL_DIR/lib/libhdf5.so" ] && [ ! -f "$INSTALL_DIR/lib/libhdf5.a" ]; then
  echo -e "${RED}✗ HDF5 library not found after installation${NC}"
  exit 1
fi

# ==========================================
# Build NetCDF-C
# ==========================================
echo -e "${YELLOW}Building NetCDF-C ${NETCDF_VERSION}${NC}"
echo ""

NETCDF_BUILD_DIR="$BUILD_ROOT/netcdf"

if [ -f "$INSTALL_DIR/lib/libnetcdf.so" ] || [ -f "$INSTALL_DIR/lib/libnetcdf.a" ]; then
  echo -e "${GREEN}NetCDF already built, skipping...${NC}"
  echo "Use --clean to rebuild"
  echo ""
else
  # Download NetCDF
  NETCDF_ARCHIVE="v${NETCDF_VERSION}.tar.gz"
  NETCDF_SRC_DIR="$BUILD_ROOT/netcdf-c-${NETCDF_VERSION}"

  if [ ! -f "$BUILD_ROOT/$NETCDF_ARCHIVE" ]; then
    echo "Downloading NetCDF $NETCDF_VERSION..."
    cd "$BUILD_ROOT"
    curl -L -o "$NETCDF_ARCHIVE" \
      "https://github.com/Unidata/netcdf-c/archive/refs/tags/v${NETCDF_VERSION}.tar.gz"
    echo ""
  fi

  # Extract
  if [ ! -d "$NETCDF_SRC_DIR" ]; then
    echo "Extracting NetCDF..."
    cd "$BUILD_ROOT"
    tar -xzf "$NETCDF_ARCHIVE"
    echo ""
  fi

  # Configure and build
  mkdir -p "$NETCDF_BUILD_DIR"
  cd "$NETCDF_BUILD_DIR"

  echo "Configuring NetCDF..."

  # Set HDF5 library paths explicitly
  export HDF5_ROOT="$INSTALL_DIR"
  export LD_LIBRARY_PATH="$INSTALL_DIR/lib:$LD_LIBRARY_PATH"

  cmake "$NETCDF_SRC_DIR" \
    -DCMAKE_INSTALL_PREFIX="$INSTALL_DIR" \
    -DCMAKE_BUILD_TYPE=Release \
    -DBUILD_SHARED_LIBS=ON \
    -DCMAKE_POSITION_INDEPENDENT_CODE=ON \
    -DENABLE_TESTS=OFF \
    -DENABLE_DAP=OFF \
    -DENABLE_BYTERANGE=OFF \
    -DENABLE_HDF5=ON \
    -DHDF5_DIR="$INSTALL_DIR" \
    -DHDF5_ROOT="$INSTALL_DIR" \
    -DHDF5_INCLUDE_DIR="$INSTALL_DIR/include" \
    -DHDF5_C_LIBRARY="$INSTALL_DIR/lib/libhdf5.so" \
    -DHDF5_HL_LIBRARY="$INSTALL_DIR/lib/libhdf5_hl.so" \
    -DENABLE_NETCDF4=ON \
    -DENABLE_EXAMPLES=OFF \
    -DBUILD_UTILITIES=OFF

  echo ""
  echo "Building NetCDF..."
  cmake --build . -j"$JOBS"

  echo ""
  echo "Installing NetCDF..."
  cmake --install .

  echo -e "${GREEN}✓ NetCDF built and installed${NC}"
  echo ""
fi

# Verify NetCDF installation
if [ ! -f "$INSTALL_DIR/lib/libnetcdf.so" ] && [ ! -f "$INSTALL_DIR/lib/libnetcdf.a" ]; then
  echo -e "${RED}✗ NetCDF library not found after installation${NC}"
  exit 1
fi

# ==========================================
# Summary
# ==========================================
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}TPL Build Complete${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "Libraries installed to: $INSTALL_DIR"
echo ""
echo "Installed libraries:"
find "$INSTALL_DIR/lib" -name "lib*.so" -o -name "lib*.a" | sort
echo ""
echo "To use these libraries:"
echo "  export LD_LIBRARY_PATH=\"$INSTALL_DIR/lib:\$LD_LIBRARY_PATH\""
echo ""
