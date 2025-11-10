# Dockerfile for SEACAS Rust Components
# This container provides a complete build environment for:
# - exodus-rs (Pure Rust Exodus II library)
# - exodus-py (Python bindings via PyO3)
# - compat-tests (C/Rust compatibility tests)

FROM rust:1.85-bookworm

LABEL maintainer="SEACAS Development Team"
LABEL description="Build environment for SEACAS Rust components"
LABEL version="0.1.0"

# Install system dependencies
# - HDF5 and NetCDF libraries for Exodus II file format support
# - Python3 and development headers for PyO3 bindings
# - Build tools for C compatibility tests
# - pkg-config for library detection
RUN apt-get update && apt-get install -y \
    # HDF5 and NetCDF libraries (required for exodus-rs)
    libhdf5-dev \
    libnetcdf-dev \
    # Python and development headers (required for exodus-py)
    python3 \
    python3-dev \
    python3-pip \
    python3-venv \
    # C/C++ build tools (required for compat-tests)
    build-essential \
    gcc \
    g++ \
    make \
    cmake \
    # Library configuration tools
    pkg-config \
    # Version control (useful for development)
    git \
    # Cleanup
    && rm -rf /var/lib/apt/lists/*

# Verify HDF5 and NetCDF installations
RUN pkg-config --modversion hdf5 && \
    pkg-config --modversion netcdf && \
    echo "HDF5 version: $(pkg-config --modversion hdf5)" && \
    echo "NetCDF version: $(pkg-config --modversion netcdf)"

# Install Python tools for building and testing Python bindings
# - maturin: Build tool for PyO3 Rust/Python bindings
# - pytest: Testing framework for Python bindings
# - numpy: Required for array operations in exodus-py
RUN pip3 install --break-system-packages --no-cache-dir \
    "maturin>=1.0,<2.0" \
    "pytest>=7.0" \
    "pytest-cov>=4.0" \
    "numpy>=1.20"

# Install Rust tooling for development
# Note: cargo-tree is built into cargo since 1.44
# Note: cargo-watch can be installed later if needed: cargo install cargo-watch
# Skipping cargo-watch installation during image build to avoid compatibility issues

# Set up working directory
WORKDIR /workspace

# Set environment variables for library paths
ENV PKG_CONFIG_PATH=/usr/lib/x86_64-linux-gnu/pkgconfig:$PKG_CONFIG_PATH
ENV LD_LIBRARY_PATH=/usr/lib/x86_64-linux-gnu:$LD_LIBRARY_PATH

# Default command: show help
CMD ["/bin/bash", "-c", "echo 'SEACAS Rust Build Environment' && \
    echo '' && \
    echo 'Installed components:' && \
    echo '  - Rust $(rustc --version)' && \
    echo '  - Python $(python3 --version 2>&1)' && \
    echo '  - HDF5 $(pkg-config --modversion hdf5)' && \
    echo '  - NetCDF $(pkg-config --modversion netcdf)' && \
    echo '  - Maturin $(maturin --version)' && \
    echo '' && \
    echo 'Usage:' && \
    echo '  Build exodus-rs:' && \
    echo '    cd /workspace/rust/exodus-rs && cargo build --features netcdf4' && \
    echo '' && \
    echo '  Run tests:' && \
    echo '    cd /workspace/rust/exodus-rs && cargo test --features netcdf4' && \
    echo '' && \
    echo '  Build Python bindings:' && \
    echo '    cd /workspace/rust/exodus-py && maturin develop --features numpy' && \
    echo '' && \
    echo '  Run Python tests:' && \
    echo '    cd /workspace/rust/exodus-py && pytest tests/' && \
    echo '' && \
    echo '  Build compatibility tests:' && \
    echo '    cd /workspace/rust/compat-tests/rust-to-c && cargo build' && \
    echo '    cd /workspace/rust/compat-tests/c-to-rust && cargo build' && \
    echo '' && \
    echo 'Mount the seacas repository at /workspace to get started.' && \
    /bin/bash"]
