//! Plot functionality (placeholder).
//!
//! This module provides basic plot state management.
//! Full plotting would require a terminal plotting library.

/// Plot state.
#[derive(Debug)]
pub struct PlotState;

impl PlotState {
    /// Create a new plot state.
    pub fn new() -> Self {
        Self
    }
}

impl Default for PlotState {
    fn default() -> Self {
        Self::new()
    }
}
