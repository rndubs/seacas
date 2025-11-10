# SEACAS Rust Docker Build Environment

This Docker container provides a complete build environment for all Rust components in the SEACAS project:

- **exodus-rs**: Pure Rust implementation of the Exodus II file format
- **exodus-py**: Python bindings for exodus-rs (using PyO3)
- **compat-tests**: C/Rust compatibility tests

## Included Components

### System Libraries
- **HDF5 1.10.10**: High-performance data management library
- **NetCDF 4.9.2**: Network Common Data Form library
- **Python 3.11+**: CPython interpreter with development headers

### Rust Toolchain
- **Rust 1.85+**: Rust compiler and cargo build system
- **cargo-watch**: Auto-rebuild on file changes
- **cargo tree**: Dependency tree visualization (built into cargo)

### Python Tools
- **maturin 1.0+**: Build tool for Rust/Python bindings
- **pytest 7.0+**: Python testing framework
- **numpy 1.20+**: Numerical computing library

### Build Tools
- **gcc/g++**: C/C++ compilers for compatibility tests
- **cmake**: Cross-platform build system
- **pkg-config**: Library configuration tool

## Quick Start

### Build the Docker Image

```bash
# From the repository root
cd docker/rust
docker build -t seacas-rust:latest .
```

### Run the Container

```bash
# From the repository root
docker run -it --rm -v $(pwd):/workspace seacas-rust:latest
```

This mounts your local SEACAS repository into the container at `/workspace`.

## Common Tasks

### Building exodus-rs

```bash
# Interactive container shell
docker run -it --rm -v $(pwd):/workspace seacas-rust:latest bash

# Inside container
cd /workspace/rust/exodus-rs
cargo build --features netcdf4
cargo build --release --features netcdf4
```

### Running Rust Tests

```bash
# All tests
cd /workspace/rust/exodus-rs
cargo test --features netcdf4

# With output
cargo test --features netcdf4 -- --nocapture

# Single-threaded (avoid file I/O conflicts)
cargo test --features netcdf4 -- --test-threads=1

# Specific test
cargo test --features netcdf4 test_create_file
```

### Running Examples

```bash
cd /workspace/rust/exodus-rs

# Basic examples
cargo run --example 01_create_file --features netcdf4
cargo run --example 02_initialize --features netcdf4
cargo run --example 03_coordinates --features netcdf4

# Advanced examples
cargo run --example 06_variables --features netcdf4
cargo run --example 09_high_level_builder --features netcdf4
```

### Building Python Bindings

```bash
cd /workspace/rust/exodus-py

# Development build (installs in current Python environment)
maturin develop --features numpy

# Production build (creates wheel)
maturin build --release --features numpy

# The wheel will be in target/wheels/
```

### Running Python Tests

```bash
cd /workspace/rust/exodus-py

# First, install the package in development mode
maturin develop --features numpy

# Run tests
pytest tests/

# With coverage
pytest tests/ --cov=exodus --cov-report=html

# Specific test file
pytest tests/test_file.py -v
```

### Building Compatibility Tests

```bash
# Rust-to-C generator
cd /workspace/rust/compat-tests/rust-to-c
cargo build
cargo run  # Generates test files

# C-to-Rust reader
cd /workspace/rust/compat-tests/c-to-rust
cargo build
cargo run  # Reads C-generated files
```

### Running Benchmarks

```bash
cd /workspace/rust/exodus-rs

# Run all benchmarks
cargo bench --features netcdf4

# Run specific benchmark
cargo bench --features netcdf4 -- file_ops
cargo bench --features netcdf4 -- coordinates
cargo bench --features netcdf4 -- connectivity
cargo bench --features netcdf4 -- variables
```

### Generating Documentation

```bash
cd /workspace/rust/exodus-rs

# Build and open documentation
cargo doc --features netcdf4 --open

# Note: --open won't work in container, but docs will be in target/doc/
# You can view them by opening target/doc/exodus_rs/index.html on your host
```

## Advanced Usage

### One-Shot Commands

Run commands without entering interactive shell:

```bash
# Build exodus-rs
docker run --rm -v $(pwd):/workspace seacas-rust:latest \
  bash -c "cd /workspace/rust/exodus-rs && cargo build --features netcdf4"

# Run tests
docker run --rm -v $(pwd):/workspace seacas-rust:latest \
  bash -c "cd /workspace/rust/exodus-rs && cargo test --features netcdf4"

# Build Python bindings
docker run --rm -v $(pwd):/workspace seacas-rust:latest \
  bash -c "cd /workspace/rust/exodus-py && maturin build --release"
```

### Development with Auto-Rebuild

Note: `cargo-watch` is not pre-installed in the image to avoid version compatibility issues. If you need it, install it in your container session:

```bash
docker run -it --rm -v $(pwd):/workspace seacas-rust:latest bash

# Install cargo-watch if needed
cargo install cargo-watch

cd /workspace/rust/exodus-rs
cargo watch -x 'test --features netcdf4'
```

### Custom Build with Different Features

```bash
cd /workspace/rust/exodus-rs

# Build with all features
cargo build --all-features

# Build with specific features
cargo build --features "netcdf4,ndarray,parallel"

# Release build with optimizations
cargo build --release --features netcdf4
```

### Multi-Stage Build for CI/CD

```dockerfile
# Example: Build artifacts in container, copy to host
FROM seacas-rust:latest AS builder
COPY . /workspace
WORKDIR /workspace/rust/exodus-rs
RUN cargo build --release --features netcdf4

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libhdf5-103 libnetcdf19
COPY --from=builder /workspace/rust/exodus-rs/target/release/libexodus_rs.so /usr/lib/
```

## Troubleshooting

### "NetCDF library not found"

This error should not occur in the container. If it does, verify:

```bash
pkg-config --modversion netcdf
pkg-config --libs netcdf
```

### "cannot find -lhdf5"

Verify HDF5 installation:

```bash
pkg-config --modversion hdf5
ls -l /usr/lib/x86_64-linux-gnu/libhdf5*
```

### Python Import Errors

If Python can't import the `exodus` module after `maturin develop`:

```bash
# Verify installation
pip3 list | grep exodus

# Reinstall
cd /workspace/rust/exodus-py
maturin develop --features numpy --force
```

### Permission Issues

If you encounter permission errors with the mounted volume:

```bash
# Run container with your user ID
docker run -it --rm -v $(pwd):/workspace -u $(id -u):$(id -g) seacas-rust:latest bash
```

### Disk Space

If the image is too large, you can:

1. Remove unused images:
   ```bash
   docker image prune -a
   ```

2. Use a multi-stage build (see example above)

3. Remove development tools in a production Dockerfile

## Environment Variables

The container sets these environment variables:

- `PKG_CONFIG_PATH=/usr/lib/x86_64-linux-gnu/pkgconfig`
- `LD_LIBRARY_PATH=/usr/lib/x86_64-linux-gnu`

You can override or extend these:

```bash
docker run -it --rm -v $(pwd):/workspace \
  -e PKG_CONFIG_PATH=/custom/path:$PKG_CONFIG_PATH \
  seacas-rust:latest bash
```

## Container Maintenance

### Updating the Image

```bash
# Rebuild from scratch
docker build --no-cache -t seacas-rust:latest docker/rust/

# Tag with version
docker build -t seacas-rust:0.1.0 docker/rust/
docker tag seacas-rust:0.1.0 seacas-rust:latest
```

### Cleaning Up

```bash
# Remove stopped containers
docker container prune

# Remove unused images
docker image prune

# Remove everything (use with caution)
docker system prune -a
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Rust Build

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    container: seacas-rust:latest

    steps:
      - uses: actions/checkout@v3

      - name: Build exodus-rs
        run: |
          cd rust/exodus-rs
          cargo build --features netcdf4

      - name: Run tests
        run: |
          cd rust/exodus-rs
          cargo test --features netcdf4

      - name: Build Python bindings
        run: |
          cd rust/exodus-py
          maturin build --release
```

### GitLab CI Example

```yaml
rust-build:
  image: seacas-rust:latest
  script:
    - cd rust/exodus-rs
    - cargo build --features netcdf4
    - cargo test --features netcdf4
```

## Additional Resources

- [exodus-rs Documentation](../../rust/exodus-rs/README.md)
- [exodus-py Documentation](../../rust/exodus-py/README.md)
- [Compatibility Tests](../../rust/compat-tests/README.md)
- [SEACAS Development Guide](../../CLAUDE.md)
- [Rust Status Report](../../rust/RUST.md)

## Version Information

- **Docker Image Version**: 0.1.0
- **Rust Version**: 1.85+
- **Python Version**: 3.11+
- **HDF5 Version**: 1.10.8
- **NetCDF Version**: 4.9.0

## License

This Dockerfile and related build scripts are part of the SEACAS project and are licensed under the BSD-3-Clause license.

## Support

For issues or questions:
- Check the [SEACAS repository](https://github.com/sandialabs/seacas)
- Review the [Rust development guide](../../CLAUDE.md)
- File an issue in the GitHub repository
