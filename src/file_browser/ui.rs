//! File browser UI rendering.

use super::FileBrowserState;
use crate::shared::ThemeColors;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

/// Draw the file browser UI.
pub fn draw_file_browser(
    f: &mut Frame<'_>,
    state: &mut FileBrowserState,
    area: Rect,
    colors: &ThemeColors,
) {
    // Adjust scroll to keep cursor visible (subtract 2 for borders)
    let viewport_height = area.height.saturating_sub(2) as usize;
    state.adjust_scroll(viewport_height);

    let items: Vec<ListItem<'_>> = state
        .entries
        .iter()
        .enumerate()
        .skip(state.scroll)
        .take(viewport_height)
        .map(|(idx, entry)| {
            let icon = if entry.is_dir { "📁" } else { "📄" };
            let symlink_indicator = if entry.is_symlink { " →" } else { "" };
            let text = format!("{} {}{}", icon, entry.name, symlink_indicator);

            let style = if idx == state.cursor {
                Style::default()
                    .fg(colors.bg0)
                    .bg(colors.yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(colors.fg0)
            };

            ListItem::new(Line::from(text)).style(style)
        })
        .collect();

    let title = format!(" File Browser: {} ", state.current_dir.display());

    let list = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors.bg2))
            .style(Style::default().bg(colors.bg0)),
    );

    f.render_widget(list, area);
}
