# Development Guide for exodus-rs

This document provides detailed development instructions for working on rust and python development for the Exodus library.

The rust crate can be found in ./rust/exodus-rs/.

The python bindings use PyO3 to expose rust features to a python package, which is all located in ./rust/exodus-py/.

Development status is tracked in the ./rust/RUST.md file.
Ensure that completed tasks are marked off from the progress tracker in that file after they have been completed.

DO NOT WRITE MARKDOWN FILES WITH A SESSION SUMMARY UNLESS PROMPTED TO DO SO.

# Requirements

- new rust features should be formatted with cargo, linted with clippy, compiled, and tested
- new python features should include documentation (when appropirate), new tests, and the test and build step should be done with `./rust/exodus-py/install_and_test.sh`

## Prerequisites

### System Dependencies

The exodus-rs crate requires HDF5 and NetCDF C libraries to be installed on your development system.

See the ./rust/CONTRIBUTING.md file if you are having issues with missing dependencies.


## Development Workflow

### Phase Implementation

Each phase of development follows this pattern:

1. **Read the spec**: Check `RUST.md` for phase requirements
2. **Implement core functionality**: Add types and methods
3. **Write tests**: Add comprehensive unit tests
4. **Write examples**: Create runnable examples
5. **Document**: Add rustdoc comments
6. **Verify**: Run tests and check documentation
7. **Format**: Run `cargo fmt` and clippy
8. **Commit**: Create descriptive commit messages

### Code Style

- Follow Rust naming conventions (snake_case for functions/variables, CamelCase for types)
- Use meaningful variable names
- Keep functions focused and concise
- Prefer explicit error handling over panics
- Use the `?` operator for error propagation
- Add `#[cfg(feature = "netcdf4")]` guards where appropriate

## Resources

- [Exodus II Format Specification](https://sandialabs.github.io/seacas-docs/)
- [NetCDF-C Documentation](https://www.unidata.ucar.edu/software/netcdf/docs/)
- [HDF5 Documentation](https://portal.hdfgroup.org/display/HDF5/HDF5)
- [netcdf-rs Crate Documentation](https://docs.rs/netcdf/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
