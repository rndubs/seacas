#!/bin/bash
# Build script for exodus-py documentation
#
# This script builds the Sphinx documentation for exodus-py.
# It handles dependency installation and runs the Sphinx build process.

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get script directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
DOCS_DIR="$SCRIPT_DIR/docs"
BUILD_DIR="$DOCS_DIR/_build"
SOURCE_DIR="$DOCS_DIR"

echo -e "${GREEN}=== exodus-py Documentation Build ===${NC}\n"

# Show usage if --help is passed
if [ "$1" == "--help" ] || [ "$1" == "-h" ]; then
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --test, --test-docs    Run documentation code example tests before building"
    echo "  --help, -h             Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                     Build documentation only"
    echo "  $0 --test             Test code examples, then build documentation"
    echo ""
    exit 0
fi


# Check if we're in the right directory
if [ ! -f "$DOCS_DIR/conf.py" ]; then
    echo -e "${RED}Error: conf.py not found in $DOCS_DIR${NC}"
    echo "Please run this script from the exodus-py directory"
    exit 1
fi

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check for Python
if ! command_exists python3; then
    echo -e "${RED}Error: python3 not found${NC}"
    echo "Please install Python 3.8 or later"
    exit 1
fi

PYTHON=$(command -v python3)
echo -e "${GREEN}Using Python:${NC} $PYTHON"
$PYTHON --version

# Check for pip
if ! $PYTHON -m pip --version >/dev/null 2>&1; then
    echo -e "${RED}Error: pip not found${NC}"
    echo "Please install pip"
    exit 1
fi

# Install documentation dependencies
echo -e "\n${YELLOW}Installing documentation dependencies...${NC}"
$PYTHON -m pip install -q -r "$DOCS_DIR/requirements.txt"

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Documentation dependencies installed${NC}"
else
    echo -e "${RED}✗ Failed to install documentation dependencies${NC}"
    exit 1
fi

# Build the exodus module (required for autodoc)
# Note: This requires maturin to be installed
echo -e "\n${YELLOW}Checking for exodus module...${NC}"
if ! $PYTHON -c "import exodus" 2>/dev/null; then
    echo -e "${YELLOW}exodus module not found. Attempting to build...${NC}"

    if command_exists maturin; then
        echo -e "${YELLOW}Building exodus-py with maturin...${NC}"
        cd "$SCRIPT_DIR"
        maturin develop --quiet
        if [ $? -eq 0 ]; then
            echo -e "${GREEN}✓ exodus-py built successfully${NC}"
        else
            echo -e "${YELLOW}Warning: Failed to build exodus-py${NC}"
            echo -e "${YELLOW}Continuing with documentation build (some API docs may be incomplete)${NC}"
        fi
        cd - > /dev/null
    else
        echo -e "${YELLOW}Warning: maturin not found${NC}"
        echo -e "${YELLOW}To build the module, install maturin: pip install maturin${NC}"
        echo -e "${YELLOW}Continuing with documentation build (some API docs may be incomplete)${NC}"
    fi
else
    echo -e "${GREEN}✓ exodus module found${NC}"
fi

# Clean previous build
echo -e "\n${YELLOW}Cleaning previous build...${NC}"
if [ -d "$BUILD_DIR" ]; then
    rm -rf "$BUILD_DIR"
    echo -e "${GREEN}✓ Cleaned previous build${NC}"
fi

# Test documentation code examples (optional)
if [ "$1" == "--test" ] || [ "$1" == "--test-docs" ]; then
    echo -e "\n${YELLOW}Testing documentation code examples...${NC}"
    cd "$DOCS_DIR"
    $PYTHON -m pytest -v --tb=short
    TEST_STATUS=$?
    cd - > /dev/null

    if [ $TEST_STATUS -ne 0 ]; then
        echo -e "\n${RED}════════════════════════════════════════${NC}"
        echo -e "${RED}✗ Documentation tests failed${NC}"
        echo -e "${RED}════════════════════════════════════════${NC}"
        exit 1
    else
        echo -e "${GREEN}✓ All documentation tests passed${NC}"
    fi
fi

# Build documentation
echo -e "\n${YELLOW}Building documentation...${NC}"
$PYTHON -m sphinx -b html -W --keep-going -j auto "$SOURCE_DIR" "$BUILD_DIR/html"

BUILD_STATUS=$?

if [ $BUILD_STATUS -eq 0 ]; then
    echo -e "\n${GREEN}════════════════════════════════════════${NC}"
    echo -e "${GREEN}✓ Documentation built successfully!${NC}"
    echo -e "${GREEN}════════════════════════════════════════${NC}"
    echo -e "\n${GREEN}Output directory:${NC} $BUILD_DIR/html"
    echo -e "${GREEN}Open:${NC} $BUILD_DIR/html/index.html"

    # If running in a display environment, offer to open the docs
    if [ -n "$DISPLAY" ] && command_exists xdg-open; then
        echo -e "\n${YELLOW}Open documentation in browser? [y/N]${NC}"
        read -r response
        if [[ "$response" =~ ^[Yy]$ ]]; then
            xdg-open "$BUILD_DIR/html/index.html"
        fi
    fi
else
    echo -e "\n${RED}════════════════════════════════════════${NC}"
    echo -e "${RED}✗ Documentation build failed${NC}"
    echo -e "${RED}════════════════════════════════════════${NC}"
    echo -e "\n${YELLOW}Check the output above for errors${NC}"
    exit 1
fi

echo ""
