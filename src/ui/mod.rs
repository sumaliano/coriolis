//! User interface - shared UI components.
//!
//! This module contains shared UI components like themes, formatters, and common widgets.
//! Feature-specific UI (like data_viewer, explorer) lives in their respective feature modules.

pub mod formatters;
mod keymap_bar;
mod status_bar;
mod theme;

use crate::app::App;
use crate::data_viewer::ui::draw_data_viewer;
use crate::explorer::{details, tree};
use crate::file_browser::ui::draw_file_browser;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub use keymap_bar::draw_keymap;
pub use status_bar::draw_status;
pub use theme::ThemeColors;

/// Draw the main UI.
pub fn draw(f: &mut Frame<'_>, app: &mut App) {
    let colors = ThemeColors::from_theme(&app.theme);

    // Main layout with status bar and key map bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(f.area());

    // Content area
    if app.file_browser_mode {
        draw_file_browser(f, &mut app.file_browser, chunks[0], &colors);
    } else if app.explorer.show_preview && app.dataset.is_some() {
        let content = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[0]);

        tree::draw_tree(
            f,
            &mut app.explorer,
            app.dataset.as_ref(),
            app.file_path.as_ref(),
            content[0],
            &colors,
        );
        draw_details(f, app, content[1], &colors);
    } else {
        tree::draw_tree(
            f,
            &mut app.explorer,
            app.dataset.as_ref(),
            app.file_path.as_ref(),
            chunks[0],
            &colors,
        );
    }

    // Status bar
    draw_status(f, chunks[1], &app.status, &app.search, &colors);

    // Key map bar
    draw_keymap(
        f,
        chunks[2],
        app.file_browser_mode,
        app.data_viewer.visible,
        app.search.is_active(),
        &colors,
    );

    // Overlays
    draw_data_viewer(f, &app.data_viewer, &colors);
}

/// Draw the details pane.
fn draw_details(f: &mut Frame<'_>, app: &App, area: Rect, colors: &ThemeColors) {
    let lines = if let Some(node) = app.current_node() {
        details::format_node_details(node, colors)
    } else {
        vec![Line::from("Select a node to view details")]
    };

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" Details ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors.bg2))
                .style(Style::default().bg(colors.bg0)),
        )
        .style(Style::default().fg(colors.fg0))
        .wrap(Wrap { trim: true })
        .scroll((app.explorer.preview_scroll, 0));

    f.render_widget(paragraph, area);
}
