//! Tree panel UI rendering.

use super::ExplorerState;
use crate::data::{DataNode, DatasetInfo};
use crate::ui::formatters::{clean_dtype, parse_dimensions};
use crate::ui::ThemeColors;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use std::path::PathBuf;

/// Get color for a data type.
fn dtype_color(dtype: Option<&String>, colors: &ThemeColors) -> ratatui::style::Color {
    let Some(dtype) = dtype else {
        return colors.green;
    };
    let dtype_lower = dtype.to_lowercase();
    if dtype_lower.contains("float") || dtype_lower.contains("double") {
        colors.aqua
    } else if dtype_lower.contains("int") || dtype_lower.contains("short") || dtype_lower.contains("byte") {
        colors.blue
    } else if dtype_lower.contains("char") || dtype_lower.contains("string") {
        colors.purple
    } else {
        colors.green
    }
}

/// Build styled spans for any node type.
fn build_node_spans(node: &DataNode, colors: &ThemeColors) -> Vec<Span<'static>> {
    let mut spans = Vec::new();

    if node.is_variable() {
        // Variable: name (dims) [ND] dtype
        spans.push(Span::styled(
            node.name.clone(),
            Style::default()
                .fg(dtype_color(node.dtype.as_ref(), colors))
                .add_modifier(Modifier::BOLD),
        ));

        // Dimension info: (dim1=size1, dim2=size2)
        if let (Some(dim_str), Some(shape)) = (node.metadata.get("dims"), &node.shape) {
            let dims = parse_dimensions(dim_str, shape);
            if !dims.is_empty() {
                spans.push(Span::styled(" (", Style::default().fg(colors.fg1)));
                for (i, (dim_name, size)) in dims.iter().enumerate() {
                    if i > 0 {
                        spans.push(Span::styled(", ", Style::default().fg(colors.fg1)));
                    }
                    spans.push(Span::styled(dim_name.to_string(), Style::default().fg(colors.yellow)));
                    spans.push(Span::styled("=", Style::default().fg(colors.fg1)));
                    spans.push(Span::styled(size.to_string(), Style::default().fg(colors.purple)));
                }
                spans.push(Span::styled(")", Style::default().fg(colors.fg1)));
            }

            // Dimensionality: [ND]
            if !shape.is_empty() {
                spans.push(Span::styled(format!(" [{}D]", shape.len()), Style::default().fg(colors.orange)));
            }
        }

        // Data type
        if let Some(dtype) = &node.dtype {
            spans.push(Span::styled(format!(" {}", clean_dtype(dtype)), Style::default().fg(colors.green)));
        }
    } else {
        // Group/Root: icon name (count)
        let icon = if node.node_type == crate::data::NodeType::Root { "🏠 " } else { "📂 " };
        spans.push(Span::styled(icon.to_string(), Style::default().fg(colors.fg0)));
        spans.push(Span::styled(node.name.clone(), Style::default().fg(colors.fg0)));
        spans.push(Span::styled(format!(" ({})", node.children.len()), Style::default().fg(colors.fg1)));
    }

    spans
}

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

            let mut spans = vec![Span::raw(indent), Span::raw(expand_icon)];
            spans.extend(build_node_spans(&item.node, colors));
            let line = if idx == cursor {
                // Cursor highlighting - apply to entire line
                Line::from(spans).style(
                    Style::default()
                        .fg(colors.bg0)
                        .bg(colors.yellow)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Line::from(spans)
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
