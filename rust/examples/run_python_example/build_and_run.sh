#!/usr/bin/env bash

# Build exodus-py and run a test script to read mesh coordinates
# Usage: ./build_and_run.sh

set -e

# Get script and repo directories
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
EXODUS_PY_DIR="$REPO_ROOT/rust/exodus-py"

echo "=== Building exodus-py ==="
echo "Repository root: $REPO_ROOT"
echo ""

# Run the install_and_test.sh script to build the wheel
cd "$EXODUS_PY_DIR"
./install_and_test.sh

# Find the most recently built wheel
WHEEL_FILE=$(find "$EXODUS_PY_DIR/target/wheels" -name "exodus_py-*.whl" -type f | sort -r | head -n 1)

if [ -z "$WHEEL_FILE" ]; then
    echo "ERROR: Could not find wheel file in $EXODUS_PY_DIR/target/wheels"
    exit 1
fi

echo ""
echo "=== Creating virtualenv in repo root ==="

# Create a new virtualenv in repo root
VENV_DIR="$REPO_ROOT/venv-exodus-test"

# Remove existing venv if present
if [ -d "$VENV_DIR" ]; then
    echo "Removing existing virtualenv..."
    rm -rf "$VENV_DIR"
fi

# Create new virtualenv using uv (already installed by install_and_test.sh)
uv venv "$VENV_DIR"
echo "Created virtualenv at: $VENV_DIR"

# Activate the virtualenv
source "$VENV_DIR/bin/activate"

# Configure dynamic library paths so Python can load NetCDF/HDF5
if [[ "$OSTYPE" == "darwin"* ]]; then
    if command -v brew &> /dev/null; then
        NETCDF_DIR=$(brew --prefix netcdf 2>/dev/null || echo "/usr/local")
        HDF5_DIR=$(brew --prefix hdf5 2>/dev/null || echo "/usr/local")
        BREW_PREFIX=$(brew --prefix 2>/dev/null || echo "/usr/local")
        export NETCDF_DIR HDF5_DIR
        export DYLD_FALLBACK_LIBRARY_PATH="${BREW_PREFIX}/lib:${NETCDF_DIR}/lib:${HDF5_DIR}/lib:${DYLD_FALLBACK_LIBRARY_PATH:-}"
        echo "Configured DYLD_FALLBACK_LIBRARY_PATH for NetCDF/HDF5"
    fi
else
    # Linux: prefer LD_LIBRARY_PATH if NETCDF_DIR/HDF5_DIR are set
    if [ -n "${NETCDF_DIR:-}" ]; then
        export LD_LIBRARY_PATH="${NETCDF_DIR}/lib:${LD_LIBRARY_PATH:-}"
    fi
    if [ -n "${HDF5_DIR:-}" ]; then
        export LD_LIBRARY_PATH="${HDF5_DIR}/lib:${LD_LIBRARY_PATH:-}"
    fi
fi

# Install the wheel
echo ""
echo "=== Installing exodus-py wheel ==="
uv pip install "$WHEEL_FILE"

# Verify installation
python -c "import exodus; print(f'exodus-py version: {exodus.__version__}')"

# Run the python script to read mesh coordinates
echo ""
echo "=== Running read_coordinates.py ==="
python "$SCRIPT_DIR/read_coordinates.py"

# Deactivate virtualenv
deactivate

echo ""
echo "=== Done ==="
echo "Virtualenv available at: $VENV_DIR"
echo "To activate: source $VENV_DIR/bin/activate"
