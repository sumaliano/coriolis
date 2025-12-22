//! User interface rendering - shared UI components.
//!
//! This module contains shared UI components like themes and the browser view.
//! Feature-specific UI (like overlay) lives in their respective feature modules.

mod browser;
mod theme;

use crate::app::App;
use ratatui::Frame;

pub use theme::ThemeColors;

/// Draw the UI.
pub fn draw(f: &mut Frame<'_>, app: &mut App) {
    browser::draw_browser(f, app);
}
