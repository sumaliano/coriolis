//! Coriolis - A fast, terminal-based netCDF data viewer.
//!
//! Coriolis provides an interactive terminal interface for exploring scientific data files
//! with vim-style keyboard navigation and a clean, hierarchical view of data structures.
//!
//! # Features
//!
//! - Fast NetCDF file reading
//! - Tree-based navigation with expand/collapse
//! - Powerful search functionality
//! - Vim-style keyboard shortcuts
//! - Gruvbox color themes
//! - Clipboard integration
//!
//! # Example
//!
//! ```ignore
//! use coriolis::data::DataReader;
//! use std::path::Path;
//!
//! // Open a dataset
//! let dataset = DataReader::read_file(Path::new("data.nc"))?;
//!
//! // Access the root node
//! let root = &dataset.root_node;
//! println!("Loaded {} with {} children", root.name, root.children.len());
//! ```

#![warn(
    missing_docs,
    missing_debug_implementations,
    rust_2018_idioms,
    unreachable_pub
)]
#![deny(unsafe_code)]

/// Application state and logic.
pub mod app;
/// Data reading and representation.
pub mod data;
/// Error types.
pub mod error;
/// Navigation and search.
pub mod navigation;
/// User interface.
pub mod ui;
/// Utility functions.
pub mod util;

pub use error::{CoriolisError, Result};
