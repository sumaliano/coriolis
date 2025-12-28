//! Details pane formatting for tree nodes.

use crate::data::{read_variable, DataNode};
use crate::ui::formatters::{format_number, format_stat_value};
use crate::ui::ThemeColors;
use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};
use std::path::PathBuf;

/// Format node details for display in the details pane.
pub fn format_node_details(
    node: &DataNode,
    colors: &ThemeColors,
    file_path: Option<&PathBuf>,
) -> Vec<Line<'static>> {
    if node.is_group() {
        return format_group_details(node, colors);
    }

    if node.is_variable() {
        return format_variable_details(node, colors, file_path);
    }

    // Generic node format
    vec![
        Line::from(Span::styled(
            node.display_name(),
            Style::default()
                .fg(colors.aqua)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Path: ", Style::default().fg(colors.fg1)),
            Span::styled(node.path.clone(), Style::default().fg(colors.fg0)),
        ]),
    ]
}

/// Format variable node details.
fn format_variable_details(
    node: &DataNode,
    colors: &ThemeColors,
    file_path: Option<&PathBuf>,
) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::from(Span::styled(
            node.name.clone(),
            Style::default()
                .fg(colors.aqua)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "─".repeat(50),
            Style::default().fg(colors.bg2),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Type: ", Style::default().fg(colors.fg1)),
            Span::styled("variable", Style::default().fg(colors.aqua)),
        ]),
        Line::from(vec![
            Span::styled("Path: ", Style::default().fg(colors.fg1)),
            Span::styled(node.path.clone(), Style::default().fg(colors.fg0)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Array Info",
            Style::default()
                .fg(colors.aqua)
                .add_modifier(Modifier::BOLD),
        )),
    ];

    // Dimensions
    if let Some(dim_str) = node.metadata.get("dims") {
        if !dim_str.is_empty() {
            let dims: Vec<&str> = dim_str.split(", ").collect();
            if let Some(shape) = &node.shape {
                let mut dim_spans = vec![Span::styled(
                    "  Dimensions: ",
                    Style::default().fg(colors.fg1),
                )];
                for (i, dim_name) in dims.iter().enumerate() {
                    if i > 0 {
                        dim_spans.push(Span::styled(" x ", Style::default().fg(colors.fg1)));
                    }
                    if let Some(&size) = shape.get(i) {
                        dim_spans.push(Span::styled(
                            format!("{}={}", dim_name, size),
                            Style::default().fg(colors.aqua),
                        ));
                    }
                }
                lines.push(Line::from(dim_spans));
            }
        }
    }

    // Data type
    if let Some(dtype) = &node.dtype {
        let clean_type = dtype.replace("NcVariableType::", "").to_lowercase();
        lines.push(Line::from(vec![
            Span::styled("  Data type: ", Style::default().fg(colors.fg1)),
            Span::styled(clean_type, Style::default().fg(colors.fg0)),
        ]));
    }

    // Size
    if let Some(shape) = &node.shape {
        let total: usize = shape.iter().product();
        lines.push(Line::from(vec![
            Span::styled("  Size: ", Style::default().fg(colors.fg1)),
            Span::styled(
                format!("{} elements", format_number(total)),
                Style::default().fg(colors.fg0),
            ),
        ]));
    }

    lines.push(Line::from(""));

    // Statistics section - load variable if small enough
    if let Some(path) = file_path {
        if let Some(shape) = &node.shape {
            let total_elements: usize = shape.iter().product();
            if total_elements > 0 && total_elements < 10_000_000 {
                if let Ok(var) = read_variable(path, &node.path) {
                    lines.push(Line::from(Span::styled(
                        "Statistics",
                        Style::default()
                            .fg(colors.green)
                            .add_modifier(Modifier::BOLD),
                    )));

                    if let Some((min_val, max_val)) = var.min_max() {
                        lines.push(Line::from(vec![
                            Span::styled("  Min: ", Style::default().fg(colors.fg1)),
                            Span::styled(
                                format_stat_value(min_val),
                                Style::default().fg(colors.green),
                            ),
                        ]));
                        lines.push(Line::from(vec![
                            Span::styled("  Max: ", Style::default().fg(colors.fg1)),
                            Span::styled(
                                format_stat_value(max_val),
                                Style::default().fg(colors.green),
                            ),
                        ]));
                    }

                    if let Some(mean_val) = var.mean_value() {
                        lines.push(Line::from(vec![
                            Span::styled("  Mean: ", Style::default().fg(colors.fg1)),
                            Span::styled(
                                format_stat_value(mean_val),
                                Style::default().fg(colors.green),
                            ),
                        ]));
                    }

                    if let Some(std_val) = var.std_value() {
                        lines.push(Line::from(vec![
                            Span::styled("  Std Dev: ", Style::default().fg(colors.fg1)),
                            Span::styled(
                                format_stat_value(std_val),
                                Style::default().fg(colors.green),
                            ),
                        ]));
                    }

                    let total = var.total_elements();
                    let valid = var.valid_count();
                    if valid < total {
                        lines.push(Line::from(vec![
                            Span::styled("  Valid: ", Style::default().fg(colors.fg1)),
                            Span::styled(
                                format!("{} / {} ({:.1}%)", valid, total, (valid as f64 / total as f64) * 100.0),
                                Style::default().fg(colors.green),
                            ),
                        ]));
                    }

                    lines.push(Line::from(""));
                }
            } else if total_elements >= 10_000_000 {
                lines.push(Line::from(Span::styled(
                    "Statistics",
                    Style::default()
                        .fg(colors.green)
                        .add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(Span::styled(
                    "  (Variable too large - use data viewer)",
                    Style::default().fg(colors.fg1),
                )));
                lines.push(Line::from(""));
            }
        }
    }

    // Attributes
    if !node.attributes.is_empty() {
        lines.push(Line::from(Span::styled(
            "Attributes",
            Style::default()
                .fg(colors.orange)
                .add_modifier(Modifier::BOLD),
        )));

        for (key, value) in &node.attributes {
            lines.push(Line::from(vec![
                Span::styled(format!("  :{}", key), Style::default().fg(colors.orange)),
                Span::styled(" = ", Style::default().fg(colors.fg1)),
                Span::styled(format!("{}", value), Style::default().fg(colors.fg0)),
            ]));
        }

        lines.push(Line::from(""));
    }

    // Actions
    lines.push(Line::from(Span::styled(
        "Actions",
        Style::default()
            .fg(colors.yellow)
            .add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(vec![
        Span::styled("  Press ", Style::default().fg(colors.fg1)),
        Span::styled(
            "p",
            Style::default()
                .fg(colors.yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to open data viewer", Style::default().fg(colors.fg1)),
    ]));
    lines.push(Line::from(Span::styled(
        "  Statistics and visualizations available",
        Style::default().fg(colors.fg1),
    )));

    lines
}

/// Format group node details.
fn format_group_details(node: &DataNode, colors: &ThemeColors) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::from(Span::styled(
            node.name.clone(),
            Style::default()
                .fg(colors.aqua)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "─".repeat(50),
            Style::default().fg(colors.bg2),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Type: ", Style::default().fg(colors.fg1)),
            Span::styled("group", Style::default().fg(colors.aqua)),
        ]),
        Line::from(vec![
            Span::styled("Path: ", Style::default().fg(colors.fg1)),
            Span::styled(node.path.clone(), Style::default().fg(colors.fg0)),
        ]),
        Line::from(""),
    ];

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
                .fg(colors.aqua)
                .add_modifier(Modifier::BOLD),
        )));

        for (key, value) in dims {
            let dim_name = key.strip_prefix("dim_").unwrap_or(key);
            lines.push(Line::from(vec![
                Span::styled(format!("  {}", dim_name), Style::default().fg(colors.aqua)),
                Span::styled(" = ", Style::default().fg(colors.fg1)),
                Span::styled(format!("{}", value), Style::default().fg(colors.aqua)),
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
                .fg(colors.aqua)
                .add_modifier(Modifier::BOLD),
        )));

        for var in variables {
            let dtype = var
                .dtype
                .as_ref()
                .map(|d| d.replace("NcVariableType::", "").to_lowercase())
                .unwrap_or_else(|| "unknown".to_string());

            let mut var_spans = vec![
                Span::styled("  ", Style::default()),
                Span::styled(dtype, Style::default().fg(colors.fg1)),
                Span::styled(" ", Style::default()),
                Span::styled(var.name.clone(), Style::default().fg(colors.aqua)),
            ];

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
                                format!(" ({})", dim_info.join(", ")),
                                Style::default().fg(colors.fg1),
                            ));
                        }
                    }
                }
            }

            lines.push(Line::from(var_spans));

            for (key, value) in &var.attributes {
                lines.push(Line::from(vec![
                    Span::styled(format!("    :{}", key), Style::default().fg(colors.orange)),
                    Span::styled(" = ", Style::default().fg(colors.fg1)),
                    Span::styled(format!("{}", value), Style::default().fg(colors.fg0)),
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
                .fg(colors.aqua)
                .add_modifier(Modifier::BOLD),
        )));

        for group in groups {
            lines.push(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(group.name.clone(), Style::default().fg(colors.aqua)),
                Span::styled(
                    format!(" ({} items)", group.children.len()),
                    Style::default().fg(colors.fg1),
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
                Span::styled(format!("  :{}", key), Style::default().fg(colors.orange)),
                Span::styled(" = ", Style::default().fg(colors.fg1)),
                Span::styled(format!("{}", value), Style::default().fg(colors.fg0)),
            ]));
        }
    }

    lines
}
