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
//! # Architecture
//!
//! Coriolis follows a clean feature-based architecture with 3 core features:
//!
//! ## Core Features
//!
//! - **`file_browser`**: File system navigation for opening NetCDF files
//! - **`explorer`**: NetCDF structure exploration (tree + details)
//! - **`data_viewer`**: Data visualization (table, plot, heatmap views)
//!
//! ## Supporting Modules
//!
//! - **`search`**: Search functionality (cross-cutting capability)
//! - **`data`**: Data layer (NetCDF reading and processing)
//! - **`ui`**: Shared UI components (themes, formatters, widgets)
//! - **`util`**: Utilities and helpers (colormaps, clipboard, etc.)
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
/// Data viewer feature (visualization).
pub mod data_viewer;
/// Error types.
pub mod error;
/// Explorer feature (tree + details).
pub mod explorer;
/// File browser feature.
pub mod file_browser;
/// Search functionality.
pub mod search;
/// User interface - shared components.
pub mod ui;
/// Utility functions.
pub mod util;

pub use error::{CoriolisError, Result};
