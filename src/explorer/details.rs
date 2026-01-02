//! Details pane formatting for tree nodes.

use crate::data::DataNode;
use crate::ui::formatters::{clean_dtype, format_number, parse_dimensions};
use crate::ui::ThemeColors;
use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

/// Format node details for display in the details pane.
pub fn format_node_details(node: &DataNode, colors: &ThemeColors) -> Vec<Line<'static>> {
    if node.is_group() {
        return format_group_details(node, colors);
    }

    if node.is_variable() {
        return format_variable_details(node, colors);
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
fn format_variable_details(node: &DataNode, colors: &ThemeColors) -> Vec<Line<'static>> {
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
    if let (Some(dim_str), Some(shape)) = (node.metadata.get("dims"), &node.shape) {
        let dims = parse_dimensions(dim_str, shape);
        if !dims.is_empty() {
            let mut dim_spans = vec![Span::styled(
                "  Dimensions: ",
                Style::default().fg(colors.fg1),
            )];
            for (i, (dim_name, size)) in dims.iter().enumerate() {
                if i > 0 {
                    dim_spans.push(Span::styled(" x ", Style::default().fg(colors.fg1)));
                }
                dim_spans.push(Span::styled(
                    dim_name.to_string(),
                    Style::default().fg(colors.yellow),
                ));
                dim_spans.push(Span::styled("=", Style::default().fg(colors.fg1)));
                dim_spans.push(Span::styled(
                    size.to_string(),
                    Style::default().fg(colors.purple),
                ));
            }
            lines.push(Line::from(dim_spans));
        }
    }

    // Data type
    if let Some(dtype) = &node.dtype {
        lines.push(Line::from(vec![
            Span::styled("  Data type: ", Style::default().fg(colors.fg1)),
            Span::styled(clean_dtype(dtype), Style::default().fg(colors.green)),
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
                .map(|d| clean_dtype(d))
                .unwrap_or_else(|| "unknown".to_string());

            let mut var_spans = vec![
                Span::styled("  ", Style::default()),
                Span::styled(dtype, Style::default().fg(colors.fg1)),
                Span::styled(" ", Style::default()),
                Span::styled(var.name.clone(), Style::default().fg(colors.aqua)),
            ];

            if let (Some(dim_str), Some(shape)) = (var.metadata.get("dims"), &var.shape) {
                let dims = parse_dimensions(dim_str, shape);
                if !dims.is_empty() {
                    let dim_info: String = dims
                        .iter()
                        .map(|(name, size)| format!("{}={}", name, size))
                        .collect::<Vec<_>>()
                        .join(", ");
                    var_spans.push(Span::styled(
                        format!(" ({})", dim_info),
                        Style::default().fg(colors.fg1),
                    ));
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
