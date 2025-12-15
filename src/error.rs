//! Error types for Coriolis.
//!
//! This module provides a unified error handling approach using `thiserror`.

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for Coriolis operations.
pub type Result<T> = std::result::Result<T, CoriolisError>;

/// Errors that can occur in Coriolis.
#[derive(Debug, Error)]
pub enum CoriolisError {
    /// Failed to open a file.
    #[error("Failed to open file: {path}")]
    FileOpen {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Unsupported file format.
    #[error("Unsupported file format: {extension}")]
    UnsupportedFormat { extension: String },

    /// Failed to read NetCDF file.
    #[error("NetCDF error: {0}")]
    NetCDF(String),

    /// Failed to access clipboard.
    #[error("Clipboard error: {0}")]
    Clipboard(#[from] arboard::Error),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Terminal error.
    #[error("Terminal error: {0}")]
    Terminal(String),

    /// Node not found in tree.
    #[error("Node not found: {path}")]
    NodeNotFound { path: String },
}

impl CoriolisError {
    /// Create a FileOpen error.
    pub fn file_open(path: PathBuf, source: std::io::Error) -> Self {
        Self::FileOpen { path, source }
    }

    /// Create an UnsupportedFormat error.
    pub fn unsupported_format(extension: impl Into<String>) -> Self {
        Self::UnsupportedFormat {
            extension: extension.into(),
        }
    }

    /// Create a NodeNotFound error.
    pub fn node_not_found(path: impl Into<String>) -> Self {
        Self::NodeNotFound { path: path.into() }
    }
}

impl From<netcdf::Error> for CoriolisError {
    fn from(err: netcdf::Error) -> Self {
        Self::NetCDF(err.to_string())
    }
}
