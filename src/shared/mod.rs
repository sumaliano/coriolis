//! Shared UI components.
//!
//! This module contains shared UI components used across different features:
//! - Theme colors and styling
//! - Value formatters
//! - Status bar and keymap bar widgets

mod keymap_bar;
mod status_bar;
mod theme;

pub use keymap_bar::draw_keymap;
pub use status_bar::draw_status;
pub use theme::ThemeColors;
