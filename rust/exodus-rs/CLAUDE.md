# Development Guide for exodus-rs

This document provides detailed development instructions for working on the exodus-rs crate.

## Prerequisites

### System Dependencies

The exodus-rs crate requires HDF5 and NetCDF C libraries to be installed on your development system.

#### Ubuntu/Debian

```bash
# Update package list
sudo apt-get update

# Install HDF5 and NetCDF development libraries
sudo apt-get install -y libhdf5-dev libnetcdf-dev

# Optional: Install pkg-config if not already available
sudo apt-get install -y pkg-config

# Verify installation
pkg-config --modversion hdf5
pkg-config --modversion netcdf
```

#### macOS

**Using Homebrew (Recommended)**:

```bash
# Install Homebrew if not already installed
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install HDF5 and NetCDF
brew install hdf5 netcdf

# Verify installation
brew list hdf5
brew list netcdf

# Check pkg-config availability
pkg-config --modversion hdf5
pkg-config --modversion netcdf
```

**Using MacPorts**:

```bash
# Install MacPorts if not already installed
# https://www.macports.org/install.php

# Install HDF5 and NetCDF
sudo port install hdf5 netcdf
sudo port install pkgconfig

# Verify installation
port installed hdf5
port installed netcdf
```

**Troubleshooting macOS**:

If you encounter linking errors on macOS:

```bash
# Set environment variables for homebrew installations
export HDF5_DIR=$(brew --prefix hdf5)
export NETCDF_DIR=$(brew --prefix netcdf)

# Or add to your shell profile (~/.zshrc or ~/.bashrc)
echo 'export HDF5_DIR=$(brew --prefix hdf5)' >> ~/.zshrc
echo 'export NETCDF_DIR=$(brew --prefix netcdf)' >> ~/.zshrc
```

#### Fedora/RHEL/CentOS

```bash
# Fedora 22+ / RHEL 8+ / CentOS 8+
sudo dnf install -y hdf5-devel netcdf-devel

# Older versions
sudo yum install -y hdf5-devel netcdf-devel

# Verify installation
rpm -q hdf5-devel netcdf-devel
```

#### Arch Linux

```bash
# Install HDF5 and NetCDF
sudo pacman -S hdf5 netcdf

# Verify installation
pacman -Q hdf5 netcdf
```

#### Windows

**Option 1: Using vcpkg**:

```bash
# Install vcpkg
git clone https://github.com/Microsoft/vcpkg.git
cd vcpkg
./bootstrap-vcpkg.sh  # or bootstrap-vcpkg.bat on Windows

# Install HDF5 and NetCDF
./vcpkg install hdf5:x64-windows netcdf-c:x64-windows

# Integrate with your system
./vcpkg integrate install
```

**Option 2: Using conda**:

```bash
# Create a development environment
conda create -n exodus-dev python=3.9
conda activate exodus-dev

# Install dependencies
conda install -c conda-forge hdf5 netcdf4

# Set environment variables
set HDF5_DIR=%CONDA_PREFIX%\Library
set NETCDF_DIR=%CONDA_PREFIX%\Library
```

## Building

### Standard Build

```bash
# Navigate to the project directory
cd seacas/rust/exodus-rs

# Build without features (limited functionality)
cargo build

# Build with netcdf4 support (recommended)
cargo build --features netcdf4

# Build in release mode
cargo build --release --features netcdf4
```

### Development Build

```bash
# Build with all features
cargo build --all-features

# Build with specific features
cargo build --features "netcdf4,ndarray,parallel"

# Watch for changes and rebuild (requires cargo-watch)
cargo install cargo-watch
cargo watch -x 'build --features netcdf4'
```

## Testing

### Running Tests

```bash
# Run all tests with netcdf4 support
cargo test --features netcdf4

# Run tests with output
cargo test --features netcdf4 -- --nocapture

# Run specific test
cargo test --features netcdf4 test_coords_2d

# Run tests for a specific module
cargo test --features netcdf4 coord

# Run tests with backtrace
RUST_BACKTRACE=1 cargo test --features netcdf4

# Run tests in parallel (default) or single-threaded
cargo test --features netcdf4 -- --test-threads=1
```

### Running Examples

```bash
# List available examples
ls examples/

# Run an example
cargo run --example 01_create_file --features netcdf4
cargo run --example 02_initialize --features netcdf4
cargo run --example 03_coordinates --features netcdf4

# Run example in release mode
cargo run --release --example 03_coordinates --features netcdf4
```

## Documentation

### Building Documentation

```bash
# Build documentation
cargo doc --features netcdf4

# Build and open documentation in browser
cargo doc --features netcdf4 --open

# Build documentation with all features
cargo doc --all-features --open

# Build private documentation (includes private items)
cargo doc --features netcdf4 --document-private-items --open
```

### Documentation Standards

- All public APIs must have documentation comments (`///`)
- Include examples in documentation where appropriate
- Document errors that functions can return
- Include links to related functions using `[function_name]` syntax

## Code Quality

### Formatting

```bash
# Check formatting
cargo fmt --check

# Apply formatting
cargo fmt

# Apply formatting to all packages
cargo fmt --all
```

### Linting

```bash
# Run clippy (Rust linter)
cargo clippy --features netcdf4

# Run clippy with all features
cargo clippy --all-features

# Run clippy with warnings as errors
cargo clippy --features netcdf4 -- -D warnings

# Apply clippy suggestions automatically (use with caution)
cargo clippy --features netcdf4 --fix
```

### Type Checking

```bash
# Check without building
cargo check --features netcdf4

# Check all targets (including tests, benches, examples)
cargo check --all-targets --features netcdf4
```

## Troubleshooting

### Common Issues

#### "NetCDF library not found"

```bash
# Ensure pkg-config can find the libraries
export PKG_CONFIG_PATH=/usr/local/lib/pkgconfig:$PKG_CONFIG_PATH

# Or set library paths directly
export LD_LIBRARY_PATH=/usr/local/lib:$LD_LIBRARY_PATH  # Linux
export DYLD_LIBRARY_PATH=/usr/local/lib:$DYLD_LIBRARY_PATH  # macOS
```

#### "Cannot find -lhdf5"

On some systems, you may need to specify library paths:

```bash
# Ubuntu/Debian
export HDF5_DIR=/usr/lib/x86_64-linux-gnu/hdf5/serial

# macOS with Homebrew
export HDF5_DIR=$(brew --prefix hdf5)
export NETCDF_DIR=$(brew --prefix netcdf)
```

#### "AlreadyExists" errors in tests

This can occur when:
- Previous test runs left temporary files
- Tests are run in parallel and conflict

Solutions:
```bash
# Clean up temporary files
rm -rf /tmp/*.exo

# Run tests single-threaded
cargo test --features netcdf4 -- --test-threads=1
```

#### "HDF5: infinite loop closing library"

This is a known issue with HDF5 when running tests. It's typically harmless and occurs during cleanup. To minimize:

```bash
# Run tests with less parallelism
cargo test --features netcdf4 -- --test-threads=4
```

### NetCDF Version Compatibility

The crate uses netcdf-rs 0.11.x which requires:
- NetCDF-C 4.1.2 or later
- HDF5 1.8.0 or later (for NetCDF-4 support)

Check your versions:

```bash
# Check NetCDF version
nc-config --version  # or nf-config --version

# Check HDF5 version
h5cc -showconfig | grep "Version"
```

## Development Workflow

### Phase Implementation

Each phase of development follows this pattern:

1. **Read the spec**: Check `RUST.md` for phase requirements
2. **Implement core functionality**: Add types and methods
3. **Write tests**: Add comprehensive unit tests
4. **Write examples**: Create runnable examples
5. **Document**: Add rustdoc comments
6. **Verify**: Run tests and check documentation
7. **Commit**: Create descriptive commit messages

### Code Style

- Follow Rust naming conventions (snake_case for functions/variables, CamelCase for types)
- Use meaningful variable names
- Keep functions focused and concise
- Prefer explicit error handling over panics
- Use the `?` operator for error propagation
- Add `#[cfg(feature = "netcdf4")]` guards where appropriate

### Git Workflow

```bash
# Create a feature branch
git checkout -b feature/phase-4-blocks

# Make changes and commit
git add src/block.rs
git commit -m "Phase 4: Implement block operations"

# Run tests before pushing
cargo test --features netcdf4

# Push to remote
git push origin feature/phase-4-blocks
```

## Useful Commands

```bash
# Show dependency tree
cargo tree --features netcdf4

# Check for outdated dependencies
cargo outdated

# Update dependencies
cargo update

# Clean build artifacts
cargo clean

# Benchmark (if benchmarks are added)
cargo bench --features netcdf4

# Generate code coverage (requires cargo-tarpaulin)
cargo install cargo-tarpaulin
cargo tarpaulin --features netcdf4 --out Html

# Check for security vulnerabilities
cargo install cargo-audit
cargo audit
```

## Performance Profiling

```bash
# Build with debug symbols in release mode
RUSTFLAGS="-C debuginfo=2" cargo build --release --features netcdf4

# Profile with perf (Linux)
perf record --call-graph dwarf ./target/release/examples/03_coordinates
perf report

# Profile with Instruments (macOS)
cargo instruments --release --features netcdf4 --example 03_coordinates
```

## IDE Setup

### VS Code

Recommended extensions:
- rust-analyzer
- CodeLLDB (for debugging)
- Better TOML
- Error Lens

Settings (`.vscode/settings.json`):
```json
{
  "rust-analyzer.cargo.features": ["netcdf4"],
  "rust-analyzer.checkOnSave.command": "clippy"
}
```

### IntelliJ IDEA / CLion

- Install the Rust plugin
- Configure Cargo features in Run Configurations
- Enable Clippy integration

## Resources

- [Exodus II Format Specification](https://sandialabs.github.io/seacas-docs/)
- [NetCDF-C Documentation](https://www.unidata.ucar.edu/software/netcdf/docs/)
- [HDF5 Documentation](https://portal.hdfgroup.org/display/HDF5/HDF5)
- [netcdf-rs Crate Documentation](https://docs.rs/netcdf/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

## Contact

For questions or issues related to development:
- Open an issue on GitHub
- Check existing documentation in `RUST.md`
- Consult the SEACAS project documentation
