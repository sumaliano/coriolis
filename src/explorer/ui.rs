//! Explorer UI - main application view rendering.

use super::{details, tree};
use crate::app::App;
use crate::data_viewer::ui::draw_data_viewer;
use crate::explorer::search::SearchState;
use crate::file_browser::ui::draw_file_browser;
use crate::theme::ThemeColors;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::Line,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

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
            app.loading,
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
            app.loading,
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
        app.pending_g,
        app.explorer.show_preview,
        &colors,
    );

    // Overlays
    draw_data_viewer(f, &app.data_viewer, &colors);
}

/// Draw the details pane.
fn draw_details(f: &mut Frame<'_>, app: &App, area: Rect, colors: &ThemeColors) {
    let lines = if let Some(node) = app.current_node() {
        details::format_node_details(node, colors, area.width)
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
        .wrap(Wrap { trim: false })
        .scroll((app.explorer.preview_scroll, 0));

    f.render_widget(paragraph, area);
}

/// Draw the status bar.
fn draw_status(
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

/// Draw the keymap help bar.
fn draw_keymap(
    f: &mut Frame<'_>,
    area: Rect,
    file_browser_mode: bool,
    data_viewer_visible: bool,
    search_active: bool,
    pending_g: bool,
    show_preview: bool,
    colors: &ThemeColors,
) {
    let keymap_text = if pending_g {
        "gg: Jump to first | Any other key: cancel"
    } else if file_browser_mode {
        "Navigate: jk/↑↓ | Select: Enter/l | Parent: h | Hidden: . | Quit: q"
    } else if data_viewer_visible {
        "Pan: hjkl | View: Tab | Slice: s +-[] | Dims: yx | Rotate: r | Copy: c | Quit: q/Esc"
    } else if search_active {
        "Search: Enter | Cancel: Esc | Type to search..."
    } else if show_preview {
        "Nav: hjkl/↑↓ | Scroll details: ^D/^U | Search: / n N | Plot: p | File: f | Copy: c y | t=hide | Quit: q"
    } else {
        "Nav: hjkl/↑↓ | Search: / n N | Details: t | Plot: p | File: f | Copy: c y | Theme: T | Help: ? | Quit: q"
    };

    let paragraph =
        Paragraph::new(keymap_text).style(Style::default().fg(colors.fg0).bg(colors.bg0));
    f.render_widget(paragraph, area);
}
