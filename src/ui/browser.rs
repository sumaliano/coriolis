//! Browser UI rendering.

use super::ThemeColors;
use crate::app::App;
use crate::data::DataNode;
use crate::overlay::ui::draw_overlay;
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
        .constraints([
            Constraint::Min(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
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

    // Adjust scroll to keep cursor visible (subtract 2 for borders)
    let viewport_height = area.height.saturating_sub(2) as usize;
    app.tree_cursor.adjust_scroll(viewport_height);

    let visible = app.tree_cursor.visible_items();
    let cursor = app.tree_cursor.cursor();
    let scroll_offset = app.tree_cursor.scroll_offset();

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
                // For variables: name is bold/green, metadata is grayed out
                let display_str = item.node.display_name();

                // Split at first space or opening parenthesis to separate name from metadata
                let (name_part, meta_part) = if let Some(pos) = display_str.find(['(', ' ']) {
                    let (n, m) = display_str.split_at(pos);
                    (n.to_string(), m.to_string())
                } else {
                    (display_str.clone(), String::new())
                };

                let mut spans = vec![
                    Span::raw(indent),
                    Span::raw(expand_icon),
                    Span::styled(
                        name_part,
                        Style::default()
                            .fg(colors.green)
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
            .border_style(Style::default().fg(colors.bg2))
            .style(Style::default().bg(colors.bg0)),
    );

    f.render_widget(list, area);
}

fn draw_details(f: &mut Frame<'_>, app: &App, area: Rect, colors: &ThemeColors) {
    let lines = if let Some(node) = app.current_node() {
        format_node_details(node, colors)
    } else {
        vec![Line::from(Span::styled(
            "Select a node to view details",
            Style::default().fg(colors.fg0),
        ))]
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

    let paragraph = Paragraph::new(text).style(Style::default().fg(colors.fg0).bg(colors.bg1));

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

    let paragraph =
        Paragraph::new(keymap_text).style(Style::default().fg(colors.fg0).bg(colors.bg0));

    f.render_widget(paragraph, area);
}

fn draw_file_browser(f: &mut Frame<'_>, app: &mut App, area: Rect, colors: &ThemeColors) {
    // Adjust scroll to keep cursor visible (subtract 2 for borders)
    let viewport_height = area.height.saturating_sub(2) as usize;
    app.adjust_browser_scroll(viewport_height);

    let items: Vec<ListItem<'_>> = app
        .file_entries
        .iter()
        .enumerate()
        .skip(app.file_browser_scroll)
        .take(viewport_height)
        .map(|(idx, entry)| {
            let icon = if entry.is_dir { "📁" } else { "📄" };
            let symlink_indicator = if entry.is_symlink { " →" } else { "" };
            let text = format!("{} {}{}", icon, entry.name, symlink_indicator);

            let style = if idx == app.file_browser_cursor {
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

    let title = format!(" File Browser: {} ", app.current_dir.display());

    let list = List::new(items).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors.bg2))
            .style(Style::default().bg(colors.bg0)),
    );

    f.render_widget(list, area);
}

fn draw_welcome(f: &mut Frame<'_>, area: Rect, colors: &ThemeColors) {
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

fn format_node_details(node: &DataNode, colors: &ThemeColors) -> Vec<Line<'static>> {
    let mut lines = vec![];

    // Format groups specially
    if node.is_group() {
        return format_group_details(node, colors);
    }

    // Format variables specially
    if node.is_variable() {
        return format_variable_details(node, colors);
    }

    // Generic node format
    lines.push(Line::from(Span::styled(
        node.display_name(),
        Style::default()
            .fg(colors.yellow)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    lines.push(Line::from(vec![
        Span::styled("Path: ", Style::default().fg(colors.green)),
        Span::styled(node.path.clone(), Style::default().fg(colors.aqua)),
    ]));

    lines
}

fn format_variable_details(node: &DataNode, colors: &ThemeColors) -> Vec<Line<'static>> {
    // Initialize with header and basic info
    let mut lines = vec![
        // Header - variable name in yellow bold
        Line::from(Span::styled(
            node.name.clone(),
            Style::default()
                .fg(colors.yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        // Type and path
        Line::from(vec![
            Span::styled("Type: ", Style::default().fg(colors.blue)),
            Span::styled("variable", Style::default().fg(colors.purple)),
        ]),
        Line::from(vec![
            Span::styled("Path: ", Style::default().fg(colors.blue)),
            Span::styled(node.path.clone(), Style::default().fg(colors.gray)),
        ]),
        Line::from(""),
        // Array Info section header
        Line::from(Span::styled(
            "Array Info",
            Style::default()
                .fg(colors.orange)
                .add_modifier(Modifier::BOLD),
        )),
    ];

    // Dimensions - use aqua for dimension info
    if let Some(dim_str) = node.metadata.get("dims") {
        if !dim_str.is_empty() {
            let dims: Vec<&str> = dim_str.split(", ").collect();
            if let Some(shape) = &node.shape {
                let mut dim_spans = vec![Span::styled(
                    "  Dimensions: ",
                    Style::default().fg(colors.green),
                )];
                for (i, dim_name) in dims.iter().enumerate() {
                    if i > 0 {
                        dim_spans.push(Span::styled(" x ", Style::default().fg(colors.fg1)));
                    }
                    if let Some(&size) = shape.get(i) {
                        dim_spans.push(Span::styled(
                            (*dim_name).to_string(),
                            Style::default().fg(colors.aqua),
                        ));
                        dim_spans.push(Span::styled(
                            format!("={}", size),
                            Style::default().fg(colors.purple),
                        ));
                    }
                }
                lines.push(Line::from(dim_spans));
            }
        }
    }

    // Data type - use blue for label, purple for value
    if let Some(dtype) = &node.dtype {
        let clean_type = dtype.replace("NcVariableType::", "").to_lowercase();
        lines.push(Line::from(vec![
            Span::styled("  Data type: ", Style::default().fg(colors.green)),
            Span::styled(clean_type, Style::default().fg(colors.orange)),
        ]));
    }

    // Size - use green for label, aqua for value
    if let Some(shape) = &node.shape {
        let total: usize = shape.iter().product();
        lines.push(Line::from(vec![
            Span::styled("  Size: ", Style::default().fg(colors.green)),
            Span::styled(
                format!("{} elements", format_number(total)),
                Style::default().fg(colors.aqua),
            ),
        ]));
    }

    lines.push(Line::from(""));

    // Attributes section
    if !node.attributes.is_empty() {
        lines.push(Line::from(Span::styled(
            "Attributes",
            Style::default()
                .fg(colors.orange)
                .add_modifier(Modifier::BOLD),
        )));

        for (key, value) in &node.attributes {
            lines.push(Line::from(vec![
                Span::styled(format!("  :{}", key), Style::default().fg(colors.purple)),
                Span::styled(" = ", Style::default().fg(colors.fg1)),
                Span::styled(format!("{};", value), Style::default().fg(colors.aqua)),
            ]));
        }

        lines.push(Line::from(""));
    }

    // Data preview note
    lines.push(Line::from(Span::styled(
        "Actions",
        Style::default()
            .fg(colors.orange)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(vec![
        Span::styled("  Press ", Style::default().fg(colors.fg0)),
        Span::styled(
            "p",
            Style::default()
                .fg(colors.yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to open data viewer", Style::default().fg(colors.fg0)),
    ]));
    lines.push(Line::from(Span::styled(
        "  Statistics and visualizations available",
        Style::default().fg(colors.gray),
    )));

    lines
}

fn format_number(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

fn format_group_details(node: &DataNode, colors: &ThemeColors) -> Vec<Line<'static>> {
    let mut lines = vec![];

    // Header - group name in yellow bold (same format as variables)
    lines.push(Line::from(Span::styled(
        node.name.clone(),
        Style::default()
            .fg(colors.yellow)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    // Type and path (same format as variables)
    lines.push(Line::from(vec![
        Span::styled("Type: ", Style::default().fg(colors.blue)),
        Span::styled("group", Style::default().fg(colors.purple)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Path: ", Style::default().fg(colors.blue)),
        Span::styled(node.path.clone(), Style::default().fg(colors.gray)),
    ]));
    lines.push(Line::from(""));

    // Dimensions section
    let dims: Vec<_> = node
        .metadata
        .iter()
        .filter(|(k, _)| k.starts_with("dim_"))
        .collect();

    if !dims.is_empty() {
        lines.push(Line::from(Span::styled(
            "Dimensions",
            Style::default()
                .fg(colors.orange)
                .add_modifier(Modifier::BOLD),
        )));

        for (key, value) in dims {
            let dim_name = key.strip_prefix("dim_").unwrap_or(key);
            lines.push(Line::from(vec![
                Span::styled(format!("  {}", dim_name), Style::default().fg(colors.aqua)),
                Span::styled(" = ", Style::default().fg(colors.fg1)),
                Span::styled(format!("{}", value), Style::default().fg(colors.purple)),
            ]));
        }
        lines.push(Line::from(""));
    }

    // Variables section
    let variables: Vec<_> = node
        .children
        .iter()
        .filter(|child| child.is_variable())
        .collect();

    if !variables.is_empty() {
        lines.push(Line::from(Span::styled(
            format!("Variables ({})", variables.len()),
            Style::default()
                .fg(colors.orange)
                .add_modifier(Modifier::BOLD),
        )));

        for var in variables {
            // Build variable line with colorful parts
            let dtype = var
                .dtype
                .as_ref()
                .map(|d| d.replace("NcVariableType::", "").to_lowercase())
                .unwrap_or_else(|| "unknown".to_string());

            let mut var_spans = vec![
                Span::styled("  ", Style::default()),
                Span::styled(dtype, Style::default().fg(colors.blue)),
                Span::styled(" ", Style::default()),
                Span::styled(var.name.clone(), Style::default().fg(colors.green)),
            ];

            // Add dimensions
            if let Some(dim_str) = var.metadata.get("dims") {
                if !dim_str.is_empty() {
                    let dims: Vec<&str> = dim_str.split(", ").collect();
                    if let Some(shape) = &var.shape {
                        let dim_info: Vec<String> = dims
                            .iter()
                            .enumerate()
                            .filter_map(|(i, dim_name)| {
                                shape.get(i).map(|&size| format!("{}={}", dim_name, size))
                            })
                            .collect();
                        if !dim_info.is_empty() {
                            var_spans.push(Span::styled(
                                format!("({})", dim_info.join(", ")),
                                Style::default().fg(colors.purple),
                            ));
                        }
                    }
                }
            }

            lines.push(Line::from(var_spans));

            // Variable attributes (indented, in aqua)
            for (key, value) in &var.attributes {
                lines.push(Line::from(vec![
                    Span::styled(format!("    :{}", key), Style::default().fg(colors.gray)),
                    Span::styled(" = ", Style::default().fg(colors.fg1)),
                    Span::styled(format!("{}", value), Style::default().fg(colors.aqua)),
                ]));
            }
        }
        lines.push(Line::from(""));
    }

    // Child groups section
    let groups: Vec<_> = node
        .children
        .iter()
        .filter(|child| child.is_group())
        .collect();

    if !groups.is_empty() {
        lines.push(Line::from(Span::styled(
            format!("Subgroups ({})", groups.len()),
            Style::default()
                .fg(colors.orange)
                .add_modifier(Modifier::BOLD),
        )));

        for group in groups {
            lines.push(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(group.name.clone(), Style::default().fg(colors.yellow)),
                Span::styled(
                    format!(" ({} items)", group.children.len()),
                    Style::default().fg(colors.gray),
                ),
            ]));
        }
        lines.push(Line::from(""));
    }

    // Global attributes section
    if !node.attributes.is_empty() {
        lines.push(Line::from(Span::styled(
            format!("Attributes ({})", node.attributes.len()),
            Style::default()
                .fg(colors.orange)
                .add_modifier(Modifier::BOLD),
        )));

        for (key, value) in &node.attributes {
            lines.push(Line::from(vec![
                Span::styled(format!("  :{}", key), Style::default().fg(colors.purple)),
                Span::styled(" = ", Style::default().fg(colors.fg1)),
                Span::styled(format!("{}", value), Style::default().fg(colors.aqua)),
            ]));
        }
    }

    lines
}
