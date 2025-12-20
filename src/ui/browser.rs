//! Browser UI rendering.

use super::{draw_overlay, ThemeColors};
use crate::app::App;
use crate::data::DataNode;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

/// Draw the browser UI.
pub(super) fn draw_browser(f: &mut Frame<'_>, app: &mut App) {
    let colors = ThemeColors::from_theme(&app.theme);

    // Main layout with status bar and key map bar
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1), Constraint::Length(1)])
        .split(f.area());

    // Content area
    if app.show_preview {
        let content = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[0]);

        draw_tree(f, app, content[0], &colors);
        draw_details(f, app, content[1], &colors);
    } else {
        draw_tree(f, app, chunks[0], &colors);
    }

    // Status bar
    draw_status(f, app, chunks[1], &colors);

    // Key map bar
    draw_keymap(f, app, chunks[2], &colors);

    // Overlays
    draw_overlay(f, &app.overlay, &colors);
}

fn draw_tree(f: &mut Frame<'_>, app: &mut App, area: Rect, colors: &ThemeColors) {
    // Show file browser if in browser mode
    if app.file_browser_mode {
        draw_file_browser(f, app, area, colors);
        return;
    }

    let Some(ref _dataset) = app.dataset else {
        draw_welcome(f, area, colors);
        return;
    };

    let visible = app.tree_cursor.visible_items();
    let cursor = app.tree_cursor.cursor();

    let items: Vec<ListItem<'_>> = visible
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            let indent = "  ".repeat(item.level);
            let expand_icon = if item.node.is_group() {
                if item.expanded {
                    "▼ "
                } else {
                    "▶ "
                }
            } else {
                "  "
            };

            let text = format!("{}{}{}", indent, expand_icon, item.node.display_name());

            let style = if idx == cursor {
                Style::default()
                    .fg(colors.cursor_fg)
                    .bg(colors.cursor_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(colors.text)
            };

            ListItem::new(Line::from(text)).style(style)
        })
        .collect();

    let title = app
        .file_path
        .as_ref()
        .and_then(|p| p.file_name())
        .map(|n| format!(" {} ", n.to_string_lossy()))
        .unwrap_or_else(|| " Coriolis ".to_string());

    let list = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors.border))
            .style(Style::default().bg(colors.bg)),
    );

    f.render_widget(list, area);
}

fn draw_details(f: &mut Frame<'_>, app: &App, area: Rect, colors: &ThemeColors) {
    let lines = if let Some(node) = app.current_node() {
        format_node_details(node, colors)
    } else {
        vec![Line::from(Span::styled(
            "Select a node to view details",
            Style::default().fg(colors.text),
        ))]
    };

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" Details ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors.border))
                .style(Style::default().bg(colors.bg)),
        )
        .style(Style::default().fg(colors.text))
        .wrap(Wrap { trim: true })
        .scroll((app.preview_scroll, 0));

    f.render_widget(paragraph, area);
}

fn draw_status(f: &mut Frame<'_>, app: &App, area: Rect, colors: &ThemeColors) {
    let text = if app.search.is_active() {
        format!("/{}", app.search.buffer())
    } else if app.search.match_count() > 0 {
        format!(
            "Match {}/{} for '{}'",
            app.search.current_match_index() + 1,
            app.search.match_count(),
            app.search.query()
        )
    } else {
        app.status.clone()
    };

    let paragraph =
        Paragraph::new(text).style(Style::default().fg(colors.status_fg).bg(colors.status_bg));

    f.render_widget(paragraph, area);
}

fn draw_keymap(f: &mut Frame<'_>, app: &App, area: Rect, colors: &ThemeColors) {
    let keymap_text = if app.file_browser_mode {
        "jk/↑↓:nav | Enter/l:select | h:parent | q:quit"
    } else if app.overlay.visible {
        "hjkl:pan | Tab:view | +-:slice | []:dim | q/Esc:close"
    } else if app.search.is_active() {
        "Enter:search | Esc:cancel | Type to search"
    } else {
        "q:quit | hjkl:nav | /:search | n/N:next/prev | t:preview | p:plot | c/y:copy | T:theme | ?:help"
    };

    let paragraph = Paragraph::new(keymap_text)
        .style(Style::default().fg(colors.text).bg(colors.bg));

    f.render_widget(paragraph, area);
}

fn draw_file_browser(f: &mut Frame<'_>, app: &App, area: Rect, colors: &ThemeColors) {
    let items: Vec<ListItem<'_>> = app
        .file_entries
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            let icon = if entry.is_dir { "📁" } else { "📄" };
            let text = format!("{} {}", icon, entry.name);

            let style = if idx == app.file_browser_cursor {
                Style::default()
                    .fg(colors.cursor_fg)
                    .bg(colors.cursor_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(colors.text)
            };

            ListItem::new(Line::from(text)).style(style)
        })
        .collect();

    let title = format!(" File Browser: {} ", app.current_dir.display());

    let list = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors.border))
            .style(Style::default().bg(colors.bg)),
    );

    f.render_widget(list, area);
}

fn draw_welcome(f: &mut Frame<'_>, area: Rect, colors: &ThemeColors) {
    let lines = vec![
        Line::from(Span::styled(
            "Welcome to Coriolis!",
            Style::default()
                .fg(colors.heading)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("Open a NetCDF file to get started"),
        Line::from(""),
        Line::from("Usage: coriolis <file.nc>"),
        Line::from(""),
        Line::from("Keyboard shortcuts:"),
        Line::from("  j/k or ↓/↑  - Navigate"),
        Line::from("  h/l or ←/→  - Collapse/Expand"),
        Line::from("  /           - Search"),
        Line::from("  t           - Toggle preview"),
        Line::from("  T           - Cycle theme"),
        Line::from("  q           - Quit"),
    ];

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" Coriolis ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors.border))
                .style(Style::default().bg(colors.bg)),
        )
        .style(Style::default().fg(colors.text));

    f.render_widget(paragraph, area);
}


fn format_node_details(node: &DataNode, colors: &ThemeColors) -> Vec<Line<'static>> {
    let mut lines = vec![];

    // Header
    lines.push(Line::from(Span::styled(
        node.display_name(),
        Style::default()
            .fg(colors.heading)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    // Path
    lines.push(Line::from(vec![
        Span::styled("Path: ", Style::default().fg(colors.label)),
        Span::styled(node.path.clone(), Style::default().fg(colors.value)),
    ]));

    // Type
    lines.push(Line::from(vec![
        Span::styled("Type: ", Style::default().fg(colors.label)),
        Span::styled(
            format!("{:?}", node.node_type),
            Style::default().fg(colors.value),
        ),
    ]));

    // Shape and dtype for variables
    if node.is_variable() {
        if let Some(ref shape) = node.shape {
            lines.push(Line::from(vec![
                Span::styled("Shape: ", Style::default().fg(colors.label)),
                Span::styled(format!("{:?}", shape), Style::default().fg(colors.value)),
            ]));
        }
        if let Some(ref dtype) = node.dtype {
            lines.push(Line::from(vec![
                Span::styled("DType: ", Style::default().fg(colors.label)),
                Span::styled(dtype.clone(), Style::default().fg(colors.value)),
            ]));
        }
    }

    // Children count for groups
    if node.is_group() {
        lines.push(Line::from(vec![
            Span::styled("Children: ", Style::default().fg(colors.label)),
            Span::styled(
                node.children.len().to_string(),
                Style::default().fg(colors.value),
            ),
        ]));
    }

    // Attributes
    if !node.attributes.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Attributes:",
            Style::default()
                .fg(colors.heading)
                .add_modifier(Modifier::BOLD),
        )));

        for (key, value) in &node.attributes {
            lines.push(Line::from(vec![
                Span::styled(format!("  {}: ", key), Style::default().fg(colors.label)),
                Span::styled(value.clone(), Style::default().fg(colors.value)),
            ]));
        }
    }

    // Metadata
    if !node.metadata.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Metadata:",
            Style::default()
                .fg(colors.heading)
                .add_modifier(Modifier::BOLD),
        )));

        for (key, value) in &node.metadata {
            lines.push(Line::from(vec![
                Span::styled(format!("  {}: ", key), Style::default().fg(colors.label)),
                Span::styled(value.clone(), Style::default().fg(colors.value)),
            ]));
        }
    }

    lines
}

