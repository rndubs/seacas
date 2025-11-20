#!/bin/bash
#
# Compile and run the Rust Exodus processing example
#
# Usage:
#   ./run_rust_example.sh INPUT.exo OUTPUT.exo [SCALE_FACTOR]
#

set -e  # Exit on error

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXODUS_RS_DIR="$SCRIPT_DIR/../exodus-rs"
EXAMPLE_NAME="12_process_large_file"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Check if we have the required arguments
if [ $# -lt 2 ]; then
    echo "Usage: $0 INPUT.exo OUTPUT.exo [SCALE_FACTOR]"
    echo ""
    echo "Arguments:"
    echo "  INPUT.exo       - Input Exodus file path"
    echo "  OUTPUT.exo      - Output Exodus file path"
    echo "  SCALE_FACTOR    - Optional scale factor for field values (default: 1.5)"
    echo ""
    echo "Example:"
    echo "  $0 input.exo output.exo 2.0"
    exit 1
fi

INPUT_FILE="$1"
OUTPUT_FILE="$2"
SCALE_FACTOR="${3:-1.5}"

# Check if input file exists
if [ ! -f "$INPUT_FILE" ]; then
    echo -e "${RED}ERROR: Input file not found: $INPUT_FILE${NC}"
    exit 1
fi

# Check if exodus-rs directory exists
if [ ! -d "$EXODUS_RS_DIR" ]; then
    echo -e "${RED}ERROR: exodus-rs directory not found at: $EXODUS_RS_DIR${NC}"
    exit 1
fi

echo -e "${GREEN}=== Rust Exodus File Processor ===${NC}"
echo ""
echo "Configuration:"
echo "  Input:  $INPUT_FILE"
echo "  Output: $OUTPUT_FILE"
echo "  Scale:  $SCALE_FACTOR"
echo ""

# Step 1: Compile the Rust example
echo -e "${YELLOW}[Step 1/2] Compiling Rust example (release mode)...${NC}"
echo ""

cd "$EXODUS_RS_DIR"

# Check if netcdf4 feature dependencies are available
echo "Checking dependencies..."
if ! command -v pkg-config &> /dev/null; then
    echo -e "${RED}WARNING: pkg-config not found. You may need to install it.${NC}"
fi

# Try to detect HDF5/NetCDF
if pkg-config --exists hdf5 netcdf 2>/dev/null; then
    echo "✓ HDF5 and NetCDF libraries found"
    HDF5_VERSION=$(pkg-config --modversion hdf5 2>/dev/null || echo "unknown")
    NETCDF_VERSION=$(pkg-config --modversion netcdf 2>/dev/null || echo "unknown")
    echo "  HDF5 version: $HDF5_VERSION"
    echo "  NetCDF version: $NETCDF_VERSION"
else
    echo -e "${YELLOW}WARNING: HDF5/NetCDF libraries may not be properly configured${NC}"
    echo "  If compilation fails, see CLAUDE.md for installation instructions"
fi

echo ""
echo "Building with cargo (this may take a minute)..."

# Build in release mode with netcdf4 feature
if cargo build --release --example "$EXAMPLE_NAME" --features netcdf4; then
    echo -e "${GREEN}✓ Compilation successful!${NC}"
else
    echo -e "${RED}ERROR: Compilation failed${NC}"
    echo ""
    echo "Common issues:"
    echo "  1. Missing HDF5/NetCDF libraries - see rust/CLAUDE.md for installation"
    echo "  2. Missing pkg-config - install with: apt-get install pkg-config"
    echo ""
    exit 1
fi

echo ""

# Step 2: Run the compiled binary
echo -e "${YELLOW}[Step 2/2] Running Rust example...${NC}"
echo ""

BINARY_PATH="$EXODUS_RS_DIR/target/release/examples/$EXAMPLE_NAME"

if [ ! -f "$BINARY_PATH" ]; then
    echo -e "${RED}ERROR: Compiled binary not found at: $BINARY_PATH${NC}"
    exit 1
fi

# Get absolute paths
INPUT_ABS=$(cd "$(dirname "$INPUT_FILE")" && pwd)/$(basename "$INPUT_FILE")
OUTPUT_ABS="$(cd "$(dirname "$OUTPUT_FILE")" && pwd)/$(basename "$OUTPUT_FILE")"

# Run the binary with timing
echo "Executing: $BINARY_PATH"
echo ""

time "$BINARY_PATH" "$INPUT_ABS" "$OUTPUT_ABS" "$SCALE_FACTOR"

EXIT_CODE=$?

echo ""
if [ $EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}✓ Processing complete!${NC}"
    echo ""
    echo "Output file: $OUTPUT_ABS"

    # Show file sizes
    if [ -f "$INPUT_ABS" ] && [ -f "$OUTPUT_ABS" ]; then
        INPUT_SIZE=$(du -h "$INPUT_ABS" | cut -f1)
        OUTPUT_SIZE=$(du -h "$OUTPUT_ABS" | cut -f1)
        echo "  Input size:  $INPUT_SIZE"
        echo "  Output size: $OUTPUT_SIZE"
    fi
else
    echo -e "${RED}ERROR: Processing failed with exit code $EXIT_CODE${NC}"
    exit $EXIT_CODE
fi
