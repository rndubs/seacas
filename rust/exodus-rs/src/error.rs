//! Error types for the Exodus library.

use thiserror::Error;

/// Entity ID type (always 64-bit in Rust, converted to/from file format)
pub type EntityId = i64;

/// Result type alias for Exodus operations
pub type Result<T> = std::result::Result<T, ExodusError>;

/// Errors that can occur when working with Exodus files
#[derive(Error, Debug)]
pub enum ExodusError {
    /// NetCDF library error
    #[cfg(feature = "netcdf4")]
    #[error("NetCDF error: {0}")]
    NetCdf(#[from] netcdf::error::Error),

    /// Invalid file mode operation
    #[error("Invalid file mode: {0}")]
    InvalidMode(String),

    /// Invalid entity type
    #[error("Invalid entity type: {0}")]
    InvalidEntityType(String),

    /// Invalid entity ID
    #[error("Invalid entity ID: {0}")]
    InvalidEntityId(EntityId),

    /// Entity not found
    #[error("Entity not found: {entity_type} with ID {id}")]
    EntityNotFound {
        /// The type of entity that was not found
        entity_type: String,
        /// The ID that was searched for
        id: EntityId,
    },

    /// Invalid dimension
    #[error("Invalid dimension: expected {expected}, got {actual}")]
    InvalidDimension {
        /// Expected dimension value
        expected: String,
        /// Actual dimension value
        actual: usize,
    },

    /// Invalid array length
    #[error("Invalid array length: expected {expected}, got {actual}")]
    InvalidArrayLength {
        /// Expected length
        expected: usize,
        /// Actual length
        actual: usize,
    },

    /// Invalid topology type
    #[error("Invalid topology: {0}")]
    InvalidTopology(String),

    /// String too long for Exodus format
    #[error("String too long: max {max}, got {actual}")]
    StringTooLong {
        /// Maximum allowed length
        max: usize,
        /// Actual string length
        actual: usize,
    },

    /// Invalid time step
    #[error("Invalid time step: {0}")]
    InvalidTimeStep(usize),

    /// NetCDF variable not defined
    #[error("Variable not defined: {0}")]
    VariableNotDefined(String),

    /// Attempted write on read-only file
    #[error("Write operation on read-only file")]
    WriteOnReadOnly,

    /// Attempted read on write-only file
    #[error("Read operation on write-only file")]
    ReadOnWriteOnly,

    /// File not initialized
    #[error("File not initialized")]
    NotInitialized,

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Other error
    #[error("{0}")]
    Other(String),
}
