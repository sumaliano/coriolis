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
        "jk/↑↓:nav | Enter/l:select | h:parent | q:quit"
    } else if data_viewer_visible {
        "hjkl:pan | Tab:view | +-:slice | []:dim | q/Esc:close"
    } else if search_active {
        "Enter:search | Esc:cancel | Type to search"
    } else {
        "q:quit | hjkl:nav | /:search | n/N:next/prev | t:preview | p:plot | c/y:copy | T:theme | ?:help"
    };

    let paragraph =
        Paragraph::new(keymap_text).style(Style::default().fg(colors.fg0).bg(colors.bg0));

    f.render_widget(paragraph, area);
}
