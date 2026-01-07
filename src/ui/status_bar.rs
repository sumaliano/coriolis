//! Status bar UI component.

use crate::search::SearchState;
use crate::ui::ThemeColors;
use ratatui::{layout::Rect, style::Style, widgets::Paragraph, Frame};

/// Draw the status bar.
pub fn draw_status(
    f: &mut Frame<'_>,
    area: Rect,
    status: &str,
    search: &SearchState,
    colors: &ThemeColors,
) {
    let text = if search.is_active() {
        format!("/{}", search.buffer())
    } else if search.match_count() > 0 {
        format!(
            "Match {}/{} for '{}'",
            search.current_match_index() + 1,
            search.match_count(),
            search.query()
        )
    } else {
        status.to_string()
    };

    let paragraph = Paragraph::new(text).style(Style::default().fg(colors.fg0).bg(colors.bg1));

    f.render_widget(paragraph, area);
}
