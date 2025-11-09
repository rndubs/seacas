# Exodus II C/Rust Compatibility Testing Plan

## Overview

This document outlines the comprehensive testing strategy for ensuring compatibility between the Rust exodus-rs library and the C libexodus library. The goal is to verify that both implementations can correctly read files written by the other, ensuring full interoperability.

## Goals

1. **Bidirectional Compatibility**: Ensure Rust can read C-generated files and C can read Rust-generated files
2. **Data Integrity**: Verify that all data round-trips correctly between implementations
3. **API Coverage**: Test all major Exodus II features supported by both libraries
4. **Regression Prevention**: Catch compatibility breaks early in development
5. **Easy Maintenance**: Keep tests separate from both libraries' test suites to avoid merge conflicts
6. **Easy Removal**: Structure allows complete removal without affecting either library

## Directory Structure

```
rust/compat-tests/
├── TESTING_PLAN.md           # This document
├── README.md                 # Quick start guide
├── rust-to-c/                # Tests where Rust writes, C reads
│   ├── Cargo.toml           # Rust test binary dependencies
│   ├── src/
│   │   ├── main.rs          # Rust writer entry point
│   │   ├── basic_mesh.rs    # Basic mesh generation tests
│   │   ├── element_blocks.rs # Element block tests
│   │   ├── sets.rs          # Node/side/element set tests
│   │   └── variables.rs     # Variable and time step tests
│   └── verify.c             # C verification program
├── c-to-rust/                # Tests where C writes, Rust reads
│   ├── writer.c             # C file writer
│   ├── CMakeLists.txt       # C build configuration
│   ├── Cargo.toml           # Rust verifier dependencies
│   └── src/
│       ├── main.rs          # Rust verification entry point
│       ├── verify_basic.rs  # Basic mesh verification
│       ├── verify_blocks.rs # Element block verification
│       ├── verify_sets.rs   # Set verification
│       └── verify_vars.rs   # Variable verification
├── shared/                   # Shared test utilities and data
│   ├── test_data/           # Reference data files
│   │   ├── simple_mesh.json # Simple mesh definition
│   │   ├── complex_mesh.json # Complex mesh with all features
│   │   └── expected_values/ # Expected output values
│   ├── schemas/             # Data validation schemas
│   └── compare_utils/       # Cross-language comparison tools
│       ├── compare.py       # Python script to compare files
│       └── diff_exodus.sh   # Shell script using ncdump
└── tools/                    # Build and test automation
    ├── run_all_tests.sh     # Master test runner
    ├── build_c.sh           # C compilation script
    ├── build_rust.sh        # Rust compilation script
    └── clean.sh             # Cleanup script
```

## Testing Strategy

### Phase 1: Basic File Operations (Phases 0-3)

Test the fundamental file operations implemented in exodus-rs Phases 0-3:

**Features to Test:**
- File creation with various options (clobber/noclobber)
- File opening for read/write
- Initialization parameters (title, dimensions, counts)
- QA and info records
- Nodal coordinates (f32 and f64)
- Coordinate names

**Test Cases:**
1. **Empty File**: Create empty exodus file, verify header
2. **Simple 2D Mesh**: Single quad element with 4 nodes
3. **Simple 3D Mesh**: Single hex element with 8 nodes
4. **Float Precision**: Test both f32 and f64 coordinate storage
5. **Large Coordinate Arrays**: Test performance with 10K+ nodes

### Phase 2: Element Blocks (Phase 4)

Test element block functionality:

**Features to Test:**
- Element block definitions
- Connectivity arrays
- Various element topologies (quad, hex, tet, wedge, pyramid, etc.)
- Multiple element blocks
- Attribute storage

**Test Cases:**
1. **Single Block**: One element type, simple connectivity
2. **Multiple Blocks**: Different element types in same file
3. **All Topologies**: Test each supported topology
4. **Large Connectivity**: 100K+ elements
5. **Attributes**: Element attributes storage and retrieval

### Phase 3: Sets (Phase 5)

Test node sets, side sets, and element sets:

**Features to Test:**
- Node set definitions and data
- Side set definitions and data
- Element set definitions
- Distribution factors
- Set names and properties

**Test Cases:**
1. **Node Sets**: Simple boundary node sets
2. **Side Sets**: Surface definitions with distribution factors
3. **Element Sets**: Material region definitions
4. **Multiple Sets**: All set types in single file
5. **Empty Sets**: Edge case handling

### Phase 4: Variables and Time Steps (Phase 6)

Test variable definitions and time-dependent data:

**Features to Test:**
- Global variables
- Nodal variables
- Element variables
- Node set variables
- Side set variables
- Time step management
- Truth tables
- Partial I/O (read/write specific time steps)

**Test Cases:**
1. **Global Variables**: Scalars changing over time
2. **Nodal Variables**: Temperature, pressure fields
3. **Element Variables**: Stress, strain tensors
4. **Multiple Time Steps**: 100+ time steps
5. **Truth Tables**: Sparse variable storage
6. **Partial I/O**: Read/write subset of variables/steps

### Phase 5: Advanced Features (Phases 7-8)

Test advanced Exodus II features:

**Features to Test:**
- ID maps (node, element, face, edge)
- Entity naming
- Properties
- Assemblies
- Blobs
- Attributes

**Test Cases:**
1. **ID Mapping**: Custom node/element numbering
2. **Named Entities**: Names for blocks, sets, variables
3. **Assemblies**: Hierarchical grouping
4. **Blobs**: Arbitrary data storage

## Implementation Approach

### Test Structure

Each test follows this pattern:

```
1. GENERATE: One library writes an exodus file
2. VERIFY: Other library reads and validates
3. COMPARE: Compare against expected values
4. REPORT: Generate detailed pass/fail report
```

### Data Validation

Three levels of validation:

1. **Structural**: File format, dimensions, entity counts
2. **Metadata**: Names, properties, QA records
3. **Numerical**: Coordinate, connectivity, and variable data
   - Exact integer comparison
   - Floating-point tolerance for coordinates
   - Relative tolerance for computed values

### Test Execution Flow

```bash
# For rust-to-c tests:
1. cargo run --manifest-path rust-to-c/Cargo.toml -- test_case_name
   → Generates: output/test_case_name.exo

2. gcc -o verify rust-to-c/verify.c -lexodus
   → Compiles C verifier

3. ./verify output/test_case_name.exo
   → Reads file, validates, returns 0 (pass) or 1 (fail)

# For c-to-rust tests:
1. gcc -o writer c-to-rust/writer.c -lexodus
   → Compiles C writer

2. ./writer test_case_name
   → Generates: output/test_case_name.exo

3. cargo run --manifest-path c-to-rust/Cargo.toml -- test_case_name
   → Reads file, validates, returns exit code
```

### Comparison Tools

Multiple comparison strategies:

1. **Direct API Comparison**: Read with both libraries, compare in-memory
2. **NetCDF Comparison**: Use `ncdump` to compare raw NetCDF structure
3. **Exodiff Tool**: Use existing SEACAS `exodiff` utility
4. **Custom Python**: Fine-grained comparison with detailed reports

## Running the Tests

### Prerequisites

```bash
# Ensure both libraries are built
cd /home/user/seacas

# Build C library (if not already built)
# ... (depends on your cmake setup)

# Build Rust library
cd rust/exodus-rs
cargo build --features netcdf4
```

### Quick Start

```bash
cd rust/compat-tests

# Run all compatibility tests
./tools/run_all_tests.sh

# Run only rust-to-c tests
./tools/run_all_tests.sh --rust-to-c

# Run only c-to-rust tests
./tools/run_all_tests.sh --c-to-rust

# Run specific test
./tools/run_all_tests.sh --test basic_mesh
```

### Individual Test Execution

```bash
# Rust writes, C reads
cd rust-to-c
cargo run --features netcdf4 -- --test basic_mesh
gcc -o verify verify.c -I../../packages/seacas/libraries/exodus/include \
    -L../../packages/seacas/libraries/exodus -lexodus
./verify output/basic_mesh.exo

# C writes, Rust reads
cd c-to-rust
gcc -o writer writer.c -I../../packages/seacas/libraries/exodus/include \
    -L../../packages/seacas/libraries/exodus -lexodus
./writer basic_mesh
cd verify && cargo run --features netcdf4 -- ../output/basic_mesh.exo
```

### Continuous Integration

The test suite should be integrated into CI/CD:

```yaml
# Example GitHub Actions workflow
name: Compatibility Tests

on: [push, pull_request]

jobs:
  compat-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Install dependencies
        run: |
          sudo apt-get install -y libhdf5-dev libnetcdf-dev
      - name: Build C library
        run: |
          # Build C exodus library
      - name: Run compatibility tests
        run: |
          cd rust/compat-tests
          ./tools/run_all_tests.sh
```

## Test Coverage Matrix

Track which features have compatibility tests:

| Feature | Phase | Rust→C | C→Rust | Status |
|---------|-------|--------|--------|--------|
| File Create/Open | 1 | ⏳ | ⏳ | Planned |
| Initialization | 2 | ⏳ | ⏳ | Planned |
| Coordinates | 3 | ⏳ | ⏳ | Planned |
| Element Blocks | 4 | ⏳ | ⏳ | Planned |
| Node Sets | 5 | ⏳ | ⏳ | Planned |
| Side Sets | 5 | ⏳ | ⏳ | Planned |
| Element Sets | 5 | ⏳ | ⏳ | Planned |
| Global Variables | 6 | ⏳ | ⏳ | Planned |
| Nodal Variables | 6 | ⏳ | ⏳ | Planned |
| Element Variables | 6 | ⏳ | ⏳ | Planned |
| Time Steps | 6 | ⏳ | ⏳ | Planned |
| Truth Tables | 6 | ⏳ | ⏳ | Planned |
| ID Maps | 7 | ⏳ | ⏳ | Future |
| Names | 7 | ⏳ | ⏳ | Future |
| Assemblies | 8 | ⏳ | ⏳ | Future |
| Blobs | 8 | ⏳ | ⏳ | Future |

**Legend:** ✅ Implemented | 🔄 In Progress | ⏳ Planned | ❌ Not Applicable

## Success Criteria

A test passes when:

1. **File Generation**: Writer completes without errors
2. **File Reading**: Reader opens file successfully
3. **Data Integrity**: All values match within tolerances
   - Integers: Exact match
   - Floats: Relative error < 1e-6 for f32, < 1e-12 for f64
4. **Completeness**: All expected entities present
5. **Structure**: NetCDF structure matches expected schema

## Troubleshooting

### Common Issues

**Issue**: C program can't find libexodus.so
```bash
export LD_LIBRARY_PATH=/path/to/seacas/lib:$LD_LIBRARY_PATH
```

**Issue**: Rust can't find NetCDF libraries
```bash
export PKG_CONFIG_PATH=/usr/local/lib/pkgconfig:$PKG_CONFIG_PATH
```

**Issue**: Test files not cleaned up
```bash
cd rust/compat-tests
./tools/clean.sh
```

**Issue**: Floating-point comparison failures
- Check tolerance settings in comparison utilities
- Verify both libraries using same precision mode
- Use `ncdump -p 15` for high-precision inspection

### Debugging Workflow

1. **Generate file with verbose output**:
   ```bash
   RUST_LOG=debug cargo run -- test_name
   ```

2. **Inspect with ncdump**:
   ```bash
   ncdump -h output/test.exo  # Header only
   ncdump output/test.exo      # Full dump
   ```

3. **Compare two files**:
   ```bash
   ./shared/compare_utils/diff_exodus.sh file1.exo file2.exo
   ```

4. **Use exodiff** (SEACAS tool):
   ```bash
   exodiff -dump file1.exo file2.exo
   ```

## Maintenance

### Adding New Tests

1. Create test data definition in `shared/test_data/`
2. Add Rust writer function in `rust-to-c/src/`
3. Add C verification in `rust-to-c/verify.c`
4. Add reverse test in `c-to-rust/`
5. Update test coverage matrix above
6. Add test to `tools/run_all_tests.sh`

### Updating for New Features

When adding new Exodus II features to exodus-rs:

1. Review this TESTING_PLAN.md
2. Add test cases for new feature
3. Implement in both directions (Rust→C and C→Rust)
4. Update coverage matrix
5. Document any special considerations

## Removal Strategy

If compatibility testing needs to be removed:

1. Delete entire `rust/compat-tests/` directory
2. No changes needed to `rust/exodus-rs/`
3. No changes needed to C library
4. Remove any CI/CD references
5. Remove references in documentation

The isolation ensures clean removal with zero merge conflicts.

## Future Enhancements

### Short Term
- [ ] Implement basic test framework (Phases 1-3)
- [ ] Add element block tests (Phase 4)
- [ ] Add set tests (Phase 5)
- [ ] Add variable tests (Phase 6)

### Medium Term
- [ ] Automated test result reporting (HTML reports)
- [ ] Performance benchmarking (read/write speed comparison)
- [ ] Memory usage profiling
- [ ] Fuzzing for edge cases

### Long Term
- [ ] Integration with SEACAS regression test suite
- [ ] Cross-platform testing (Linux, macOS, Windows)
- [ ] Parallel I/O compatibility tests
- [ ] Large file testing (multi-GB files)
- [ ] Compression format compatibility

## References

- [Exodus II Manual](https://sandialabs.github.io/seacas-docs/)
- [exodus-rs Implementation Plan](../RUST.md)
- [C Library Location](../../packages/seacas/libraries/exodus/)
- [NetCDF Documentation](https://www.unidata.ucar.edu/software/netcdf/docs/)

## Contact

For questions or issues with compatibility testing:
- Check existing issues in the repository
- Review the exodus-rs implementation status in `rust/RUST.md`
- Consult the C library documentation

---

**Last Updated**: 2025-11-09
**Status**: Initial Planning
**Next Review**: After Phase 6 completion
