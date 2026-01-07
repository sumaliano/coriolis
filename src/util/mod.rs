//! Utility functions and helpers.
//!
//! This module contains pure utility functions with no side effects:
//! - Color mapping functions
//! - Dimension calculation logic
//! - Layout configuration constants
//! - Clipboard operations

pub mod clipboard;
pub mod colormaps;
pub mod dimension_calculator;
pub mod formatters;

pub use dimension_calculator::DimensionCalculator;
