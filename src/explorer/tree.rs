//! Tree panel UI rendering.

use super::ExplorerState;
use crate::data::DatasetInfo;
use crate::ui::ThemeColors;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use std::path::PathBuf;

/// Draw the tree panel UI.
pub fn draw_tree(
    f: &mut Frame<'_>,
    explorer: &mut ExplorerState,
    dataset: Option<&DatasetInfo>,
    file_path: Option<&PathBuf>,
    area: Rect,
    colors: &ThemeColors,
) {
    let Some(ref _dataset) = dataset else {
        draw_welcome(f, area, colors);
        return;
    };

    // Adjust scroll to keep cursor visible (subtract 2 for borders)
    let viewport_height = area.height.saturating_sub(2) as usize;
    explorer.adjust_scroll(viewport_height);

    let visible = explorer.visible_items();
    let cursor = explorer.cursor();
    let scroll_offset = explorer.scroll_offset();

    // Only show items within the viewport
    let items: Vec<ListItem<'_>> = visible
        .iter()
        .enumerate()
        .skip(scroll_offset)
        .take(viewport_height)
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

            let line = if idx == cursor {
                // Cursor highlighting - entire line
                let text = format!("{}{}{}", indent, expand_icon, item.node.display_name());
                Line::from(text).style(
                    Style::default()
                        .fg(colors.bg0)
                        .bg(colors.yellow)
                        .add_modifier(Modifier::BOLD),
                )
            } else if item.node.is_variable() {
                // For variables: colorize based on properties
                let display_str = item.node.display_name();

                // Split at first space or opening parenthesis to separate name from metadata
                let (name_part, meta_part) = if let Some(pos) = display_str.find(['(', ' ']) {
                    let (n, m) = display_str.split_at(pos);
                    (n.to_string(), m.to_string())
                } else {
                    (display_str.clone(), String::new())
                };

                // Choose color based on data type and dimensionality
                let var_color = if let Some(dtype) = &item.node.dtype {
                    let dtype_lower = dtype.to_lowercase();
                    if dtype_lower.contains("float") || dtype_lower.contains("double") {
                        colors.aqua
                    } else if dtype_lower.contains("int")
                        || dtype_lower.contains("short")
                        || dtype_lower.contains("byte")
                    {
                        colors.blue
                    } else if dtype_lower.contains("char") || dtype_lower.contains("string") {
                        colors.purple
                    } else {
                        colors.green
                    }
                } else {
                    colors.green
                };

                let mut spans = vec![
                    Span::raw(indent),
                    Span::raw(expand_icon),
                    Span::styled(
                        name_part,
                        Style::default()
                            .fg(var_color)
                            .add_modifier(Modifier::BOLD),
                    ),
                ];

                if !meta_part.is_empty() {
                    spans.push(Span::styled(meta_part, Style::default().fg(colors.gray)));
                }

                Line::from(spans)
            } else {
                // Groups and root - normal styling
                let text = format!("{}{}{}", indent, expand_icon, item.node.display_name());
                Line::from(text).style(Style::default().fg(colors.fg0))
            };

            ListItem::new(line)
        })
        .collect();

    let title = file_path
        .and_then(|p| p.file_name())
        .map(|n| format!(" {} ", n.to_string_lossy()))
        .unwrap_or_else(|| " Coriolis ".to_string());

    let list = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors.bg2))
            .style(Style::default().bg(colors.bg0)),
    );

    f.render_widget(list, area);
}

/// Draw the welcome screen.
pub fn draw_welcome(f: &mut Frame<'_>, area: Rect, colors: &ThemeColors) {
    let lines = vec![
        Line::from(Span::styled(
            "Welcome to Coriolis!",
            Style::default()
                .fg(colors.yellow)
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
                .border_style(Style::default().fg(colors.bg2))
                .style(Style::default().bg(colors.bg0)),
        )
        .style(Style::default().fg(colors.fg0));

    f.render_widget(paragraph, area);
}
