# Exodus II C/Rust Compatibility Tests

Quick start guide for running compatibility tests between the Rust exodus-rs and C libexodus implementations.

## Overview

This test suite verifies bidirectional compatibility:
- **Rust → C**: Rust writes files, C reads and validates
- **C → Rust**: C writes files, Rust reads and validates

## Quick Start

### Prerequisites

Install basic development tools:

```bash
# Ubuntu/Debian
apt-get install -y gcc g++ gfortran cmake make pkg-config git curl

# macOS
xcode-select --install
brew install cmake
```

**Note:** You do NOT need to install HDF5/NetCDF separately. The setup script builds compatible versions from source.

### One-Time Setup

Build the TPL libraries and C Exodus library (takes ~10 minutes):

```bash
# From the compat-tests directory
./setup-environment.sh

# For faster builds (use more CPU cores)
./setup-environment.sh --jobs 8

# To rebuild from scratch
./setup-environment.sh --clean
```

This script:
- Builds HDF5 1.14.6 and NetCDF 4.9.2 from source
- Compiles the SEACAS C Exodus library
- Creates the C verification tool
- Sets up environment configuration

### Run Compatibility Tests

After setup, run the full test suite:

```bash
# Source the environment (required after setup and in new shells)
source ./env-compat.sh

# Run all tests
./run-compat-tests.sh

# Verbose output
./run-compat-tests.sh --verbose

# Keep failed files for debugging
./run-compat-tests.sh --keep-failures
```

### Manual Testing

You can also run individual tests manually:

```bash
# Generate a single file
cd rust-to-c
cargo run --features netcdf4 -- basic_mesh_2d

# Verify with C library
./verify output/basic_mesh_2d.exo
```

## Test Status

### Rust → C (Completed ✅)

All 11 test files verified with C Exodus library - **80/80 tests passed (100%)**

| File | Tests | Features |
|------|-------|----------|
| basic_mesh_2d.exo | 6/6 ✅ | 2D QUAD4 mesh |
| basic_mesh_3d.exo | 6/6 ✅ | 3D HEX8 mesh |
| multiple_blocks.exo | 6/6 ✅ | Multiple element blocks |
| node_sets.exo | 7/7 ✅ | Node sets with distribution factors |
| side_sets.exo | 7/7 ✅ | Side sets with element-side pairs |
| element_sets.exo | 6/6 ✅ | Element sets |
| all_sets.exo | 8/8 ✅ | All set types combined |
| global_variables.exo | 8/8 ✅ | Global variables with time steps |
| nodal_variables.exo | 8/8 ✅ | Nodal variables with time steps |
| element_variables.exo | 8/8 ✅ | Element variables with time steps |
| all_variables.exo | 10/10 ✅ | All variable types combined |

**Result:** 100% C compatibility verified for Phases 1-6

### C → Rust (Future Work)

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

✅ **Rust → C Compatibility: 100% Verified**

- All 80 C verification tests passed
- Covers Exodus II Phases 1-6 (basic I/O through variables)
- Production-ready for writing Exodus files compatible with C tools

Current implementation status tracked in [../RUST.md](../RUST.md).

See [TEST_STATUS.md](TEST_STATUS.md) for detailed verification results.

## Notes

- Tests are completely independent of both library test suites
- Can be removed without affecting either library
- Designed to avoid merge conflicts
- Each test direction is isolated

---

For detailed information, see [TESTING_PLAN.md](TESTING_PLAN.md).
