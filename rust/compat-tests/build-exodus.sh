#!/bin/bash
#
# Self-contained script to build the Exodus C library for compatibility tests
# This script is completely independent of the root SEACAS build system
#
# Usage:
#   ./build-exodus.sh [--jobs N] [--clean]
#

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

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

# Get script directory and SEACAS root
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
SEACAS_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
BUILD_ROOT="$SCRIPT_DIR/exodus-build"
INSTALL_DIR="$SCRIPT_DIR/exodus-install"
TPL_INSTALL="$SCRIPT_DIR/tpl-install"

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Building Exodus C Library${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo "SEACAS root:  $SEACAS_ROOT"
echo "Build jobs:   $JOBS"
echo "Install to:   $INSTALL_DIR"
echo "TPL location: $TPL_INSTALL"
echo ""

# Check TPL libraries exist
if [ ! -d "$TPL_INSTALL" ]; then
  echo -e "${RED}✗ TPL libraries not found at: $TPL_INSTALL${NC}"
  echo "Please run ./build-tpls.sh first"
  exit 1
fi

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
# Build Exodus Library
# ==========================================
echo -e "${YELLOW}Building Exodus Library${NC}"
echo ""

EXODUS_SRC="$SEACAS_ROOT/packages/seacas/libraries/exodus"
EXODUS_BUILD="$BUILD_ROOT/build"

if [ -f "$INSTALL_DIR/lib/libexodus.so" ] || [ -f "$INSTALL_DIR/lib/libexodus.a" ]; then
  echo -e "${GREEN}Exodus already built, skipping...${NC}"
  echo "Use --clean to rebuild"
  echo ""
else
  # Copy Exodus source to build directory
  echo "Copying Exodus source..."
  rm -rf "$BUILD_ROOT/src"
  cp -r "$EXODUS_SRC" "$BUILD_ROOT/src"
  echo ""

  # Create a standalone CMakeLists.txt
  echo "Creating standalone CMakeLists.txt..."
  cat > "$BUILD_ROOT/src/CMakeLists.txt.standalone" << 'EOF'
cmake_minimum_required(VERSION 3.12)
project(ExodusII C)

# Find NetCDF
find_path(NETCDF_INCLUDE_DIR netcdf.h
  HINTS ${NETCDF_ROOT} $ENV{NETCDF_ROOT}
  PATH_SUFFIXES include
  NO_DEFAULT_PATH
)

find_library(NETCDF_LIBRARY NAMES netcdf
  HINTS ${NETCDF_ROOT} $ENV{NETCDF_ROOT}
  PATH_SUFFIXES lib
  NO_DEFAULT_PATH
)

if(NOT NETCDF_INCLUDE_DIR OR NOT NETCDF_LIBRARY)
  message(FATAL_ERROR "NetCDF not found. Please set NETCDF_ROOT")
endif()

set(NETCDF_INCLUDE_DIRS ${NETCDF_INCLUDE_DIR})
set(NETCDF_LIBRARIES ${NETCDF_LIBRARY})

message(STATUS "NetCDF include: ${NETCDF_INCLUDE_DIRS}")
message(STATUS "NetCDF library: ${NETCDF_LIBRARIES}")

# Include directories
include_directories(
  ${CMAKE_CURRENT_SOURCE_DIR}/include
  ${CMAKE_CURRENT_BINARY_DIR}
  ${NETCDF_INCLUDE_DIRS}
)

# Collect source files
file(GLOB SOURCES "src/ex_*.c")

# Create config files
configure_file(
  ${CMAKE_CURRENT_SOURCE_DIR}/include/exodusII_cfg.h.in
  ${CMAKE_CURRENT_BINARY_DIR}/exodusII_cfg.h
)

# Create exodus_config.h
file(WRITE ${CMAKE_CURRENT_BINARY_DIR}/exodus_config.h "#pragma once\n")

# Build library
add_library(exodus SHARED ${SOURCES})
set_property(TARGET exodus PROPERTY C_STANDARD 99)
set_property(TARGET exodus PROPERTY C_EXTENSIONS ON)
set_property(TARGET exodus PROPERTY POSITION_INDEPENDENT_CODE ON)

target_include_directories(exodus PUBLIC
  $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
  $<BUILD_INTERFACE:${CMAKE_CURRENT_BINARY_DIR}>
  $<INSTALL_INTERFACE:include>
)

target_link_libraries(exodus PUBLIC ${NETCDF_LIBRARIES})

# Install
install(TARGETS exodus
  LIBRARY DESTINATION lib
  ARCHIVE DESTINATION lib
)

install(FILES
  include/exodusII.h
  include/exodusII_int.h
  include/exodusII_par.h
  ${CMAKE_CURRENT_BINARY_DIR}/exodusII_cfg.h
  ${CMAKE_CURRENT_BINARY_DIR}/exodus_config.h
  DESTINATION include
)
EOF

  # Rename CMakeLists.txt to use standalone version
  cd "$BUILD_ROOT/src"
  mv CMakeLists.txt CMakeLists.txt.tribits
  mv CMakeLists.txt.standalone CMakeLists.txt

  # Configure
  mkdir -p "$EXODUS_BUILD"
  cd "$EXODUS_BUILD"

  echo "Configuring Exodus..."

  # Set paths for NetCDF and HDF5
  export PKG_CONFIG_PATH="$TPL_INSTALL/lib/pkgconfig:$PKG_CONFIG_PATH"
  export LD_LIBRARY_PATH="$TPL_INSTALL/lib:$LD_LIBRARY_PATH"

  cmake "$BUILD_ROOT/src" \
    -DCMAKE_INSTALL_PREFIX="$INSTALL_DIR" \
    -DCMAKE_BUILD_TYPE=Release \
    -DBUILD_SHARED_LIBS=ON \
    -DNETCDF_ROOT="$TPL_INSTALL" \
    -DHDF5_ROOT="$TPL_INSTALL"

  echo ""
  echo "Building Exodus..."
  cmake --build . -j"$JOBS"

  echo ""
  echo "Installing Exodus..."
  cmake --install .

  echo -e "${GREEN}✓ Exodus built and installed${NC}"
  echo ""
fi

# Verify installation
if [ ! -f "$INSTALL_DIR/lib/libexodus.so" ] && [ ! -f "$INSTALL_DIR/lib/libexodus.a" ]; then
  echo -e "${RED}✗ Exodus library not found after installation${NC}"
  exit 1
fi

if [ ! -f "$INSTALL_DIR/include/exodusII.h" ]; then
  echo -e "${RED}✗ Exodus header not found after installation${NC}"
  exit 1
fi

# ==========================================
# Summary
# ==========================================
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Exodus Build Complete${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "Library installed to: $INSTALL_DIR"
echo ""
echo "Installed files:"
echo "  Library: $(ls "$INSTALL_DIR/lib/libexodus."*)"
echo "  Headers: $INSTALL_DIR/include/exodusII*.h"
echo ""
echo "To use this library:"
echo "  export LD_LIBRARY_PATH=\"$INSTALL_DIR/lib:$TPL_INSTALL/lib:\$LD_LIBRARY_PATH\""
echo ""
