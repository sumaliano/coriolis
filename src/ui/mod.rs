//! User interface rendering.

mod browser;
mod theme;

use crate::app::App;
use ratatui::Frame;

pub use theme::ThemeColors;

/// Draw the UI.
pub fn draw(f: &mut Frame<'_>, app: &mut App) {
    browser::draw_browser(f, app);
}
