//! Tree panel UI rendering.

use super::ExplorerState;
use crate::data::{DataNode, DatasetInfo};
use crate::ui::ThemeColors;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};
use std::path::PathBuf;

/// Build styled spans for a variable node.
fn build_variable_spans(node: &DataNode, colors: &ThemeColors) -> Vec<Span<'static>> {
    let mut spans = Vec::new();

    // Icon
    spans.push(Span::raw("- "));

    // Variable name - bold and colored by data type
    let var_color = if let Some(dtype) = &node.dtype {
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

    spans.push(Span::styled(
        node.name.clone(),
        Style::default()
            .fg(var_color)
            .add_modifier(Modifier::BOLD),
    ));

    // Dimension info: (dim1=size1, dim2=size2)
    if let Some(dim_str) = node.metadata.get("dims") {
        if !dim_str.is_empty() {
            let dims: Vec<&str> = dim_str.split(", ").collect();
            if let Some(shape) = &node.shape {
                spans.push(Span::styled(" (", Style::default().fg(colors.fg1)));

                for (i, dim_name) in dims.iter().enumerate() {
                    if i > 0 {
                        spans.push(Span::styled(", ", Style::default().fg(colors.fg1)));
                    }
                    if let Some(&size) = shape.get(i) {
                        // Dimension name in yellow
                        spans.push(Span::styled(
                            dim_name.to_string(),
                            Style::default().fg(colors.yellow),
                        ));
                        spans.push(Span::styled("=", Style::default().fg(colors.fg1)));
                        // Size in purple
                        spans.push(Span::styled(
                            size.to_string(),
                            Style::default().fg(colors.purple),
                        ));
                    }
                }

                spans.push(Span::styled(")", Style::default().fg(colors.fg1)));
            }
        }
    }

    // Dimensionality: [ND]
    if let Some(shape) = &node.shape {
        let ndim = shape.len();
        if ndim > 0 {
            spans.push(Span::styled(" [", Style::default().fg(colors.fg1)));
            spans.push(Span::styled(
                format!("{}", ndim),
                Style::default().fg(colors.orange),
            ));
            spans.push(Span::styled("D]", Style::default().fg(colors.fg1)));
        }
    }

    // Data type - in a complementary color
    if let Some(dtype) = &node.dtype {
        let clean_type = dtype.replace("NcVariableType::", "").to_lowercase();
        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            clean_type,
            Style::default().fg(colors.green),
        ));
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
                // Build styled spans for variable
                let mut spans = vec![Span::raw(indent), Span::raw(expand_icon)];
                spans.extend(build_variable_spans(&item.node, colors));
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
