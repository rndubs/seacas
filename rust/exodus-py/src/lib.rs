//! Python bindings for exodus-rs
//!
//! This module provides Python bindings for the exodus-rs library,
//! exposing the high-level builder API and core file operations.

// Suppress non-local definitions warning from PyO3 0.20.3 macros
// This is a known issue with PyO3 < 0.21 and newer Rust compilers
// Upgrading to PyO3 0.21+ requires significant API changes (bound API)
#![allow(non_local_definitions)]

use pyo3::prelude::*;

// Module declarations
mod types;
mod error;
mod file;
mod builder;
mod coord;
mod block;
mod set;
mod metadata;
mod map;
mod assembly;
mod variable;
mod attribute;
mod performance;

// Re-exports
use types::*;
use file::*;
use builder::*;

/// Python module for exodus-py
#[pymodule]
fn exodus(_py: Python, m: &PyModule) -> PyResult<()> {
    // Add version info
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("__doc__", "Python bindings for exodus-rs - Pure Rust Exodus II implementation")?;

    // Note: Error handling uses PyRuntimeError - see error.rs

    // Register enum types
    m.add_class::<EntityType>()?;
    m.add_class::<CreateMode>()?;
    m.add_class::<FloatSize>()?;
    m.add_class::<Int64Mode>()?;
    m.add_class::<AttributeType>()?;

    // Register performance types
    m.add_class::<performance::PyNodeType>()?;
    m.add_class::<performance::PyCacheConfig>()?;
    m.add_class::<performance::PyChunkConfig>()?;
    m.add_class::<performance::PyPerformanceConfig>()?;

    // Register data types
    m.add_class::<InitParams>()?;
    m.add_class::<CreateOptions>()?;
    m.add_class::<Block>()?;
    m.add_class::<NodeSet>()?;
    m.add_class::<SideSet>()?;
    m.add_class::<EntitySet>()?;
    m.add_class::<Assembly>()?;
    m.add_class::<Blob>()?;
    m.add_class::<QaRecord>()?;
    m.add_class::<TruthTable>()?;
    m.add_class::<attribute::AttributeData>()?;

    // Register file classes
    m.add_class::<ExodusReader>()?;
    m.add_class::<ExodusWriter>()?;
    m.add_class::<ExodusAppender>()?;

    // Register builder API
    m.add_class::<MeshBuilder>()?;
    m.add_class::<BlockBuilder>()?;

    Ok(())
}
