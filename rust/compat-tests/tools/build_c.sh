#!/bin/bash
#
# Build C compatibility test programs
#
# This script assumes the Exodus C library is built and available.
# Adjust paths as needed for your system.
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPAT_DIR="$(dirname "$SCRIPT_DIR")"
SEACAS_ROOT="$(dirname "$(dirname "$COMPAT_DIR")")"

echo "===================================="
echo "  Building C Test Programs"
echo "===================================="
echo

# Detect Exodus C library location
EXODUS_INCLUDE="${EXODUS_INCLUDE:-$SEACAS_ROOT/packages/seacas/libraries/exodus/include}"
EXODUS_LIB="${EXODUS_LIB:-$SEACAS_ROOT/build/lib}"

echo "Using Exodus include: $EXODUS_INCLUDE"
echo "Using Exodus library: $EXODUS_LIB"
echo

# Check if exodus library exists
if [ ! -d "$EXODUS_INCLUDE" ]; then
    echo "Warning: Exodus include directory not found: $EXODUS_INCLUDE"
    echo "Set EXODUS_INCLUDE environment variable to correct path"
fi

# Build rust-to-c verifier
echo "Building rust-to-c C verifier..."
cd "$COMPAT_DIR/rust-to-c"
gcc -o verify verify.c \
    -I"$EXODUS_INCLUDE" \
    -L"$EXODUS_LIB" \
    -lexodus \
    -lnetcdf \
    -lhdf5 \
    -lm \
    -Wl,-rpath,"$EXODUS_LIB" || {
    echo "Error: Failed to build C verifier"
    echo "You may need to:"
    echo "  1. Build the C Exodus library first"
    echo "  2. Set EXODUS_INCLUDE and EXODUS_LIB environment variables"
    echo "  3. Install NetCDF and HDF5 development libraries"
    exit 1
}
echo "✓ Built: verify"
echo

# Build c-to-rust writer
echo "Building c-to-rust C writer..."
cd "$COMPAT_DIR/c-to-rust"
gcc -o writer writer.c \
    -I"$EXODUS_INCLUDE" \
    -L"$EXODUS_LIB" \
    -lexodus \
    -lnetcdf \
    -lhdf5 \
    -lm \
    -Wl,-rpath,"$EXODUS_LIB" || {
    echo "Error: Failed to build C writer"
    exit 1
}
echo "✓ Built: writer"
echo

echo "===================================="
echo "  C builds complete!"
echo "===================================="
