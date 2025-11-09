# exodus-rs

A pure Rust implementation of the Exodus II finite element data format.

## Overview

Exodus II is a widely-used model for storing and retrieving finite element analysis (FEA) data. It is used for:

- **Pre-processing**: Mesh generation
- **Post-processing**: Visualization and analysis
- **Code-to-code data transfer**: Interoperability between FEA codes
- **Long-term data archival**: Persistent storage of simulation results

This crate provides a complete, native Rust implementation with no FFI bindings to the C library.

## Features

- ✅ **Pure Rust**: No unsafe code or C dependencies (beyond NetCDF)
- ✅ **Type Safe**: Compile-time enforcement of correct usage via type-state pattern
- ✅ **Memory Safe**: Leverages Rust's ownership system
- ✅ **NetCDF Backend**: Built on the mature `netcdf` crate
- ✅ **Dual API**: Both low-level C-compatible and high-level Rust idiomatic interfaces
- ✅ **Full Format Support**: Read and write all Exodus II file versions
- ✅ **Multiple NetCDF Formats**: Supports NetCDF-3, NetCDF-4, and CDF-5

## Status

🚧 **Under Active Development** 🚧

This library is currently in the initial development phase. The following phases are planned:

- ✅ Phase 0: Project Setup and Infrastructure
- ✅ Phase 1: File Lifecycle (Create, Open, Close)
- ✅ Phase 2: Initialization and Basic Metadata (CURRENT)
- ⏳ Phase 3: Coordinate Operations
- ⏳ Phase 4: Block Operations
- ⏳ Phase 5: Set Operations
- ⏳ Phase 6: Variable Operations
- ⏳ Phase 7: Time Step Operations
- ⏳ Phase 8: Assembly Operations
- ⏳ Phase 9: Blob Operations
- ⏳ Phase 10: Attribute Operations

### Phase 1 Complete

Phase 1 provides basic file lifecycle operations:
- ✅ Create new Exodus files with customizable options
- ✅ Open existing files for reading
- ✅ Append to existing files (read-write mode)
- ✅ Query file properties (version, format, path)
- ✅ Automatic resource cleanup (RAII)
- ✅ Type-safe file modes at compile time
- ✅ Support for different NetCDF formats
- ✅ Comprehensive unit tests

### Phase 2 Complete

Phase 2 provides database initialization and metadata operations:
- ✅ Initialize database with comprehensive parameters
- ✅ Builder pattern for fluent initialization API
- ✅ Read back initialization parameters
- ✅ QA records for software provenance tracking
- ✅ Info records for arbitrary text metadata
- ✅ Coordinate axis naming
- ✅ Complete validation and error handling
- ✅ Comprehensive unit tests

## Quick Start

```rust
use exodus_rs::{ExodusFile, mode, InitParams};

// Create a new Exodus file
let mut file = ExodusFile::<mode::Write>::create_default("mesh.exo")?;

// Initialize the database
let params = InitParams {
    title: "Example Mesh".into(),
    num_dim: 3,
    num_nodes: 8,
    num_elems: 1,
    num_elem_blocks: 1,
    ..Default::default()
};
file.init(&params)?;

// Open an existing file for reading
let file = ExodusFile::<mode::Read>::open("mesh.exo")?;
let params = file.init_params()?;
println!("Title: {}", params.title);
```

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
exodus-rs = "0.1"
```

### System Requirements

- Rust 1.70 or later
- NetCDF C library (version 4.1.2+)
- HDF5 library (for NetCDF-4 support)

## Architecture

### Type-State Pattern

The library uses Rust's type system to prevent incorrect usage at compile time:

```rust
use exodus_rs::{ExodusFile, mode};

// Read-only file - can only perform read operations
let reader: ExodusFile<mode::Read> = ExodusFile::open("mesh.exo")?;

// Write-only file - can only perform write operations
let writer: ExodusFile<mode::Write> = ExodusFile::create_default("output.exo")?;

// Append mode - can read and write
let appender: ExodusFile<mode::Append> = ExodusFile::append("existing.exo")?;
```

### Module Organization

- `error` - Error types and result aliases
- `types` - Core type definitions
- `file` - File handle and mode types
- `init` - Database initialization
- `coord` - Coordinate operations
- `block` - Block (element/edge/face) operations
- `set` - Set operations
- `variable` - Variable definitions and I/O
- `time` - Time step operations
- `metadata` - QA records, info records, and names
- `assembly` - Assembly (hierarchical grouping) operations
- `blob` - Blob (arbitrary data) operations
- `attribute` - Attribute operations
- `raw` - Low-level C-compatible API

## Optional Features

```toml
[dependencies]
exodus-rs = { version = "0.1", features = ["ndarray", "parallel", "serde"] }
```

- `netcdf4` (default) - NetCDF-4 format support
- `ndarray` - Integration with ndarray for multi-dimensional arrays
- `parallel` - Parallel I/O support via rayon
- `serde` - Serialization support for data structures

## Documentation

Full API documentation is available at [docs.rs/exodus-rs](https://docs.rs/exodus-rs).

For more details on the Exodus II format, see the [official documentation](https://sandialabs.github.io/seacas-docs/).

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

This library is part of the [SEACAS](https://github.com/sandialabs/seacas) project.

## License

This project is licensed under the BSD 3-Clause License - see the LICENSE file for details.

## Acknowledgments

- Built on the [netcdf](https://crates.io/crates/netcdf) crate by the GeoRust community
- Based on the Exodus II format developed at Sandia National Laboratories
- Part of the SEACAS suite of computational tools
