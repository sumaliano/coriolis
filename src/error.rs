//! Error types for Coriolis.

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
        /// Path to the file that could not be opened.
        path: PathBuf,
        /// Underlying IO error.
        #[source]
        source: std::io::Error,
    },

    /// Unsupported file format.
    #[error("Unsupported file format: {extension}")]
    UnsupportedFormat {
        /// File extension that is not supported.
        extension: String,
    },

    /// Failed to read NetCDF file.
    #[error("NetCDF error: {0}")]
    NetCDF(String),

    /// Failed to access clipboard.
    #[error("Clipboard error: {0}")]
    Clipboard(#[from] arboard::Error),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
