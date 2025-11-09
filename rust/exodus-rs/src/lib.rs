//! # exodus-rs
//!
//! A pure Rust implementation of the Exodus II finite element data format.
//!
//! Exodus II is a model developed to store and retrieve data for finite element analyses.
//! It is used for preprocessing (mesh generation), post-processing (visualization, analysis),
//! code-to-code data transfer, and long-term data archival.
//!
//! ## Features
//!
//! - **Pure Rust**: No FFI bindings, native Rust implementation
//! - **Type Safe**: Leverages Rust's type system for compile-time correctness
//! - **Memory Safe**: No unsafe code in public API
//! - **NetCDF Backend**: Built on the mature `netcdf` crate
//! - **Dual API**: Low-level C-compatible and high-level Rust idiomatic interfaces
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use exodus_rs::{ExodusFile, mode};
//!
//! // Create a new Exodus file (to be implemented in Phase 1)
//! let file = ExodusFile::<mode::Write>::create_default("mesh.exo")?;
//!
//! // Open an existing file for reading (to be implemented in Phase 1)
//! let file = ExodusFile::<mode::Read>::open("mesh.exo")?;
//! # Ok::<(), exodus_rs::error::ExodusError>(())
//! ```
//!
//! ## File Modes
//!
//! Exodus files use compile-time type-state to enforce correct usage:
//!
//! - `ExodusFile<mode::Read>` - Read-only access
//! - `ExodusFile<mode::Write>` - Write-only access (creation)
//! - `ExodusFile<mode::Append>` - Read-write access to existing file
//!
//! ## Module Organization
//!
//! - [`error`] - Error types and result aliases
//! - [`types`] - Core type definitions (EntityType, InitParams, etc.)
//! - [`file`] - File handle and mode types
//! - [`init`] - Database initialization
//! - [`coord`] - Coordinate operations
//! - [`block`] - Block (element/edge/face) operations
//! - [`set`] - Set operations (node/edge/face/elem/side sets)
//! - [`variable`] - Variable definitions and I/O
//! - [`time`] - Time step operations
//! - [`metadata`] - QA records, info records, and names
//! - [`map`] - ID maps, order maps, entity naming, and property arrays
//! - [`assembly`] - Assembly (hierarchical grouping) operations
//! - [`blob`] - Blob (arbitrary data) operations
//! - [`attribute`] - Attribute operations

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![warn(rust_2018_idioms)]

// Public modules
pub mod error;
pub mod types;
pub mod coord;

// Internal modules (will be implemented in phases)
mod file;
mod init;
mod block;
mod set;
mod variable;
mod time;
mod metadata;
mod map;
mod assembly;
mod blob;
mod attribute;

// Low-level API
pub mod raw;

// Internal utilities
mod utils;

// Re-exports for convenience
pub use error::{ExodusError, Result};
pub use file::ExodusFile;
pub use types::{
    Assembly, Attribute, AttributeType, Block, Blob, Compression, Connectivity,
    ConnectivityIterator, CreateMode, CreateOptions, EntitySet, EntityType, FileFormat,
    FloatSize, InfoRecord, InitParams, Int64Mode, NodeSet, QaRecord, Set, SideSet, Topology,
    TruthTable,
};
pub use coord::{CoordValue, Coordinates};

// File mode types
/// Type-state pattern for file modes
pub mod mode {
    /// Read-only file mode
    #[derive(Debug)]
    pub struct Read;

    /// Write-only file mode (for file creation)
    #[derive(Debug)]
    pub struct Write;

    /// Read-write file mode (for appending to existing files)
    #[derive(Debug)]
    pub struct Append;
}

/// Trait for file modes (sealed to prevent external implementation)
pub trait FileMode: private::Sealed {}

impl FileMode for mode::Read {}
impl FileMode for mode::Write {}
impl FileMode for mode::Append {}

mod private {
    pub trait Sealed {}
    impl Sealed for super::mode::Read {}
    impl Sealed for super::mode::Write {}
    impl Sealed for super::mode::Append {}
}

/// Type alias for read-only Exodus files
pub type ExodusReader = ExodusFile<mode::Read>;

/// Type alias for write-only Exodus files (creation)
pub type ExodusWriter = ExodusFile<mode::Write>;

/// Type alias for read-write Exodus files (append mode)
pub type ExodusAppender = ExodusFile<mode::Append>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_type_display() {
        assert_eq!(EntityType::ElemBlock.to_string(), "elem_block");
        assert_eq!(EntityType::NodeSet.to_string(), "node_set");
    }

    #[test]
    fn test_init_params_default() {
        let params = InitParams::default();
        assert_eq!(params.num_dim, 3);
        assert_eq!(params.num_nodes, 0);
    }

    #[test]
    fn test_create_options_default() {
        let opts = CreateOptions::default();
        assert_eq!(opts.mode, CreateMode::NoClobber);
        assert_eq!(opts.float_size, FloatSize::Float64);
        assert_eq!(opts.int64_mode, Int64Mode::Int64);
    }
}
