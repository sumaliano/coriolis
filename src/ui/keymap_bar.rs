//! Keymap help bar UI component.

use crate::ui::ThemeColors;
use ratatui::{
    layout::Rect,
    style::Style,
    widgets::Paragraph,
    Frame,
};

/// Draw the keymap help bar.
pub fn draw_keymap(
    f: &mut Frame<'_>,
    area: Rect,
    file_browser_mode: bool,
    data_viewer_visible: bool,
    search_active: bool,
    colors: &ThemeColors,
) {
    let keymap_text = if file_browser_mode {
        "Navigate: jk/↑↓ | Select: Enter/l | Parent: h | Quit: q"
    } else if data_viewer_visible {
        "Pan: hjkl | View: Tab | Slice: +-[] | Dims: xy | Quit: q/Esc"
    } else if search_active {
        "Search: Enter | Cancel: Esc | Type to search..."
    } else {
        "Nav: hjkl/↑↓ | Search: / n N | Details: t | Plot: p | Theme: T | Help: ? | Quit: q"
    };

    let paragraph =
        Paragraph::new(keymap_text).style(Style::default().fg(colors.fg0).bg(colors.bg0));

    f.render_widget(paragraph, area);
}
