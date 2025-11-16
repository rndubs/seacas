# Installation Guide for exodus-py

## Prerequisites

### System Dependencies

exodus-py requires NetCDF-4 libraries to be installed on your system.

#### Ubuntu/Debian
```bash
sudo apt-get update
sudo apt-get install libnetcdf-dev libhdf5-dev
```

#### macOS (using Homebrew)
```bash
brew install netcdf hdf5
```

#### Fedora/RHEL/CentOS
```bash
sudo dnf install netcdf-devel hdf5-devel
```

### Python Requirements

- Python 3.8 or later
- pip or conda package manager

## Installation Methods

### Method 1: From PyPI (when published)

```bash
pip install exodus-py
```

### Method 2: From Source (Development)

This is the recommended method for development and testing.

1. **Clone the repository** (if not already done):
```bash
git clone https://github.com/rndubs/seacas
cd seacas/rust/exodus-py
```

2. **Install maturin** (build tool for PyO3 projects):
```bash
pip install maturin
```

3. **Build and install in development mode**:
```bash
# This builds the Rust extension and installs it in your Python environment
maturin develop

# For release mode (faster, but longer compile time):
maturin develop --release
```

4. **Verify the installation**:
```bash
python -c "import exodus; print(exodus.__version__)"
```

### Method 3: Building Wheel for Distribution

To create a distributable wheel file:

```bash
# Install build dependencies
pip install maturin

# Build wheel
maturin build --release

# The wheel will be in target/wheels/
# Install it with:
pip install target/wheels/exodus_py-*.whl
```

## Troubleshooting

### NetCDF Library Not Found

If you get errors about NetCDF not being found:

1. Make sure NetCDF is installed (see Prerequisites)
2. Set the `NETCDF_DIR` environment variable:
```bash
export NETCDF_DIR=/usr/local  # or wherever NetCDF is installed
maturin develop
```

3. On macOS with Homebrew:
```bash
export NETCDF_DIR=$(brew --prefix netcdf)
maturin develop
```

### HDF5 Library Issues

If you encounter HDF5-related errors:

```bash
# On Ubuntu/Debian
export HDF5_DIR=/usr/lib/x86_64-linux-gnu/hdf5/serial

# On macOS with Homebrew
export HDF5_DIR=$(brew --prefix hdf5)
```

### Rust Compiler Not Found

maturin requires Rust. Install it from [rustup.rs](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Permission Errors

If you get permission errors during installation:

```bash
# Use virtual environment (recommended)
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate
maturin develop

# Or use --user flag
pip install --user maturin
```

## Testing the Installation

Run the included example:

```bash
python examples/simple_mesh.py
```

Or run the test suite:

```bash
# Install pytest
pip install pytest

# Run tests
pytest tests/
```

## Virtual Environment (Recommended)

It's recommended to use a virtual environment:

```bash
# Create virtual environment
python -m venv exodus-env

# Activate it
source exodus-env/bin/activate  # On Windows: exodus-env\Scripts\activate

# Install exodus-py
maturin develop --release

# When done
deactivate
```

## Updating

To update an existing installation:

```bash
cd seacas/rust/exodus-py
git pull
maturin develop --release
```

## Uninstallation

```bash
pip uninstall exodus-py
```

## Getting Help

If you encounter issues:

1. Check the [SEACAS documentation](https://rndubs.github.io/seacas-docs/)
2. Review the [examples](examples/) directory
3. File an issue on [GitHub](https://github.com/rndubs/seacas/issues)
