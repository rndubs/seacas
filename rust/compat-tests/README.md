# Exodus II C/Rust Compatibility Tests

Quick start guide for running compatibility tests between the Rust exodus-rs and C libexodus implementations.

## Overview

This test suite verifies bidirectional compatibility:
- **Rust → C**: Rust writes files, C reads and validates
- **C → Rust**: C writes files, Rust reads and validates

## Quick Start

### Prerequisites

```bash
# Install system dependencies (Ubuntu/Debian)
apt-get install -y libhdf5-dev libnetcdf-dev pkg-config gcc

# Or on macOS
brew install hdf5 netcdf
```

### Build Everything

```bash
# From the compat-tests directory
./tools/build_all.sh
```

### Run All Tests

```bash
./tools/run_all_tests.sh
```

### Run Specific Test Direction

```bash
# Only Rust → C tests
./tools/run_all_tests.sh --rust-to-c

# Only C → Rust tests
./tools/run_all_tests.sh --c-to-rust
```

### Run Individual Test

```bash
# Rust writes, C reads
cd rust-to-c
cargo run --features netcdf4 -- basic_mesh
./verify output/basic_mesh.exo

# C writes, Rust reads
cd c-to-rust
./writer basic_mesh
cargo run --manifest-path verify/Cargo.toml -- output/basic_mesh.exo
```

## Test Categories

| Category | Phase | Description | Status |
|----------|-------|-------------|--------|
| Basic I/O | 1-3 | File creation, initialization, coordinates | ⏳ Planned |
| Element Blocks | 4 | Block definitions, connectivity | ⏳ Planned |
| Sets | 5 | Node/side/element sets | ⏳ Planned |
| Variables | 6 | Time-dependent data | ⏳ Planned |
| Maps & Names | 7 | ID mapping, entity naming | ⏳ Future |
| Advanced | 8 | Assemblies, blobs | ⏳ Future |

## Directory Structure

```
compat-tests/
├── README.md              # This file
├── TESTING_PLAN.md        # Detailed testing strategy
├── rust-to-c/            # Rust writes, C verifies
├── c-to-rust/            # C writes, Rust verifies
├── shared/               # Common utilities and test data
└── tools/                # Build and run scripts
```

## Output

Test results go to `output/` directory (gitignored):
```
output/
├── basic_mesh.exo        # Generated test files
├── complex_mesh.exo
├── test_results.txt      # Summary of pass/fail
└── detailed_report.html  # Detailed comparison report
```

## Cleaning Up

```bash
./tools/clean.sh          # Remove all generated files
./tools/clean.sh --all    # Also remove build artifacts
```

## Troubleshooting

**Libraries not found:**
```bash
export LD_LIBRARY_PATH=/path/to/exodus/lib:$LD_LIBRARY_PATH
```

**NetCDF issues:**
```bash
pkg-config --modversion netcdf
export PKG_CONFIG_PATH=/usr/local/lib/pkgconfig:$PKG_CONFIG_PATH
```

**View generated files:**
```bash
ncdump -h output/test.exo  # View file structure
```

## Documentation

See [TESTING_PLAN.md](TESTING_PLAN.md) for:
- Detailed testing strategy
- Test case descriptions
- Adding new tests
- CI/CD integration
- Removal instructions

## Status

🚧 **Under Development** 🚧

Current implementation status tracked in [../RUST.md](../RUST.md).

## Notes

- Tests are completely independent of both library test suites
- Can be removed without affecting either library
- Designed to avoid merge conflicts
- Each test direction is isolated

---

For detailed information, see [TESTING_PLAN.md](TESTING_PLAN.md).
