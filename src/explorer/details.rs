//! Details pane formatting for tree nodes.

use crate::data::DataNode;
use crate::theme::ThemeColors;
use crate::util::formatters::{clean_dtype, format_number, get_dimension_type, parse_dimensions};
use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

/// CF-convention attributes that are shown prominently, not buried in the attribute list.
const CF_KEY_ATTRS: &[&str] = &[
    "long_name",
    "standard_name",
    "units",
    "_FillValue",
    "missing_value",
    "valid_min",
    "valid_max",
    "valid_range",
];

/// Format node details for display in the details pane.
pub fn format_node_details(
    node: &DataNode,
    colors: &ThemeColors,
    width: u16,
) -> Vec<Line<'static>> {
    if node.is_group() {
        return format_group_details(node, colors, width);
    }

    if node.is_variable() {
        return format_variable_details(node, colors, width);
    }

    vec![
        Line::from(Span::styled(
            node.display_name(),
            Style::default()
                .fg(colors.aqua)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled("Path: ", Style::default().fg(colors.fg1)),
            Span::styled(node.path.clone(), Style::default().fg(colors.fg0)),
        ]),
    ]
}

fn format_variable_details(
    node: &DataNode,
    colors: &ThemeColors,
    width: u16,
) -> Vec<Line<'static>> {
    let sep_width = (width as usize).saturating_sub(2).max(1);
    let mut lines = vec![
        Line::from(Span::styled(
            node.name.clone(),
            Style::default()
                .fg(colors.aqua)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "─".repeat(sep_width),
            Style::default().fg(colors.bg2),
        )),
    ];

    // CF key attributes surfaced first for quick orientation
    if let Some(long_name) = node.attributes.get("long_name") {
        lines.push(Line::from(Span::styled(
            long_name.clone(),
            Style::default().fg(colors.fg0),
        )));
    }
    if let Some(standard_name) = node.attributes.get("standard_name") {
        lines.push(Line::from(vec![
            Span::styled("CF: ", Style::default().fg(colors.fg1)),
            Span::styled(standard_name.clone(), Style::default().fg(colors.fg1)),
        ]));
    }
    if let Some(units) = node.attributes.get("units") {
        lines.push(Line::from(vec![
            Span::styled("Units: ", Style::default().fg(colors.fg1)),
            Span::styled(units.clone(), Style::default().fg(colors.green)),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("Type: ", Style::default().fg(colors.fg1)),
        Span::styled("variable", Style::default().fg(colors.aqua)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Path: ", Style::default().fg(colors.fg1)),
        Span::styled(node.path.clone(), Style::default().fg(colors.fg0)),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Array Info:",
        Style::default()
            .fg(colors.yellow)
            .add_modifier(Modifier::BOLD),
    )));

    if let (Some(dim_str), Some(shape)) = (node.metadata.get("dims"), &node.shape) {
        let dim_type = get_dimension_type(dim_str, shape);
        lines.push(Line::from(vec![
            Span::styled("  Dimensions: ", Style::default().fg(colors.fg1)),
            Span::styled(dim_type, Style::default().fg(colors.red)),
        ]));

        let dims = parse_dimensions(dim_str, shape);
        if !dims.is_empty() {
            let mut shape_spans = vec![Span::styled("  Shape: ", Style::default().fg(colors.fg1))];
            for (i, (dim_name, size)) in dims.iter().enumerate() {
                if i > 0 {
                    shape_spans.push(Span::styled(" x ", Style::default().fg(colors.fg1)));
                }
                shape_spans.push(Span::styled(
                    dim_name.to_string(),
                    Style::default().fg(colors.yellow),
                ));
                shape_spans.push(Span::styled("=", Style::default().fg(colors.fg1)));
                shape_spans.push(Span::styled(
                    size.to_string(),
                    Style::default().fg(colors.red),
                ));
            }
            lines.push(Line::from(shape_spans));
        }
    }

    if let Some(dtype) = &node.dtype {
        lines.push(Line::from(vec![
            Span::styled("  Data type: ", Style::default().fg(colors.fg1)),
            Span::styled(clean_dtype(dtype), Style::default().fg(colors.green)),
        ]));
    }

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

    if let Some(sample) = &node.sample {
        let shape = node.shape.as_deref().unwrap_or(&[]);
        let total: usize = shape.iter().product();

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Sample:",
            Style::default()
                .fg(colors.yellow)
                .add_modifier(Modifier::BOLD),
        )));

        if shape.len() == 2 {
            // Grid display: reader gave us up to SAMPLE_ROWS×SAMPLE_COLS values
            const SAMPLE_ROWS: usize = 3;
            const SAMPLE_COLS: usize = 4;
            let rows_shown = shape[0].min(SAMPLE_ROWS);
            let cols_shown = shape[1].min(SAMPLE_COLS);
            let n = (rows_shown * cols_shown).min(sample.len());

            let formatted: Vec<String> = sample[..n]
                .iter()
                .map(|v| format_sample_value(*v))
                .collect();
            let col_w = formatted.iter().map(|s| s.len()).max().unwrap_or(4);

            for r in 0..rows_shown {
                let mut row_spans = vec![Span::styled("  ", Style::default())];
                for c in 0..cols_shown {
                    let idx = r * cols_shown + c;
                    if idx >= n {
                        break;
                    }
                    if c > 0 {
                        row_spans.push(Span::styled("  ", Style::default()));
                    }
                    row_spans.push(Span::styled(
                        format!("{:>width$}", formatted[idx], width = col_w),
                        Style::default().fg(colors.aqua),
                    ));
                }
                lines.push(Line::from(row_spans));
            }

            if rows_shown < shape[0] || cols_shown < shape[1] {
                lines.push(Line::from(Span::styled(
                    format!("  … ({}×{} total)", shape[0], shape[1]),
                    Style::default().fg(colors.fg1),
                )));
            }
        } else {
            // Flat display for 0D, 1D, 3D+
            const MAX_SHOWN: usize = 6;
            let shown = &sample[..sample.len().min(MAX_SHOWN)];
            let formatted: Vec<String> = shown.iter().map(|v| format_sample_value(*v)).collect();

            let mut value_spans: Vec<Span<'static>> = vec![Span::styled("  ", Style::default())];
            value_spans.push(Span::styled(
                formatted.join(",  "),
                Style::default().fg(colors.aqua),
            ));
            if sample.len() > MAX_SHOWN || sample.len() < total {
                let shape_str = if shape.len() >= 3 {
                    shape
                        .iter()
                        .map(|d| d.to_string())
                        .collect::<Vec<_>>()
                        .join("×")
                } else {
                    total.to_string()
                };
                value_spans.push(Span::styled(
                    format!("  … ({} total)", shape_str),
                    Style::default().fg(colors.fg1),
                ));
            }
            lines.push(Line::from(value_spans));
        }
    } else {
        // For larger/higher-dimensional variables: show valid range and fill value if present
        let fill = node
            .attributes
            .get("_FillValue")
            .or_else(|| node.attributes.get("missing_value"));
        let valid_min = node.attributes.get("valid_min");
        let valid_max = node.attributes.get("valid_max");

        if fill.is_some() || valid_min.is_some() || valid_max.is_some() {
            lines.push(Line::from(""));
        }
        if let (Some(vmin), Some(vmax)) = (valid_min, valid_max) {
            lines.push(Line::from(vec![
                Span::styled("  Range: ", Style::default().fg(colors.fg1)),
                Span::styled(vmin.clone(), Style::default().fg(colors.aqua)),
                Span::styled(" → ", Style::default().fg(colors.fg1)),
                Span::styled(vmax.clone(), Style::default().fg(colors.aqua)),
            ]));
        } else if let Some(vmin) = valid_min {
            lines.push(Line::from(vec![
                Span::styled("  Valid min: ", Style::default().fg(colors.fg1)),
                Span::styled(vmin.clone(), Style::default().fg(colors.aqua)),
            ]));
        } else if let Some(vmax) = valid_max {
            lines.push(Line::from(vec![
                Span::styled("  Valid max: ", Style::default().fg(colors.fg1)),
                Span::styled(vmax.clone(), Style::default().fg(colors.aqua)),
            ]));
        }
        if let Some(fv) = fill {
            lines.push(Line::from(vec![
                Span::styled("  Fill value: ", Style::default().fg(colors.fg1)),
                Span::styled(fv.clone(), Style::default().fg(colors.orange)),
            ]));
        }
    }

    // Remaining attributes (skip CF key ones already shown above)
    let other_attrs: Vec<(&String, &String)> = node
        .attributes
        .iter()
        .filter(|(k, _)| !CF_KEY_ATTRS.contains(&k.as_str()))
        .collect();

    if !other_attrs.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Attributes:",
            Style::default()
                .fg(colors.yellow)
                .add_modifier(Modifier::BOLD),
        )));
        let mut sorted: Vec<_> = other_attrs;
        sorted.sort_by_key(|(k, _)| k.as_str());
        for (key, value) in sorted {
            lines.push(Line::from(vec![
                Span::styled(format!("  :{}", key), Style::default().fg(colors.orange)),
                Span::styled(" = ", Style::default().fg(colors.fg1)),
                Span::styled(value.clone(), Style::default().fg(colors.fg0)),
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("Press ", Style::default().fg(colors.fg1)),
        Span::styled(
            "p",
            Style::default()
                .fg(colors.yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" to open data viewer", Style::default().fg(colors.fg1)),
    ]));

    lines
}

fn format_group_details(node: &DataNode, colors: &ThemeColors, width: u16) -> Vec<Line<'static>> {
    let sep_width = (width as usize).saturating_sub(2).max(1);
    let mut lines = vec![
        Line::from(Span::styled(
            node.name.clone(),
            Style::default()
                .fg(colors.blue)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "─".repeat(sep_width),
            Style::default().fg(colors.bg2),
        )),
        Line::from(vec![
            Span::styled("Type: ", Style::default().fg(colors.fg1)),
            Span::styled("group", Style::default().fg(colors.blue)),
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
            "Dimensions:",
            Style::default()
                .fg(colors.yellow)
                .add_modifier(Modifier::BOLD),
        )));

        let mut sorted_dims: Vec<_> = dims;
        sorted_dims.sort_by_key(|(k, _)| k.as_str());
        for (key, value) in sorted_dims {
            let dim_name = key.strip_prefix("dim_").unwrap_or(key);
            lines.push(Line::from(vec![
                Span::styled(format!("  {}", dim_name), Style::default().fg(colors.aqua)),
                Span::styled(" = ", Style::default().fg(colors.fg1)),
                Span::styled(value.clone(), Style::default().fg(colors.red)),
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
            format!("Variables ({}):", variables.len()),
            Style::default()
                .fg(colors.yellow)
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

            // Show long_name inline if present
            if let Some(long_name) = var.attributes.get("long_name") {
                var_spans.push(Span::styled(
                    format!("  {}", long_name),
                    Style::default().fg(colors.fg1),
                ));
            }

            lines.push(Line::from(var_spans));
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
            format!("Subgroups ({}):", groups.len()),
            Style::default()
                .fg(colors.yellow)
                .add_modifier(Modifier::BOLD),
        )));

        for group in groups {
            lines.push(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(group.name.clone(), Style::default().fg(colors.blue)),
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
            format!("Attributes ({}):", node.attributes.len()),
            Style::default()
                .fg(colors.yellow)
                .add_modifier(Modifier::BOLD),
        )));

        let mut sorted: Vec<_> = node.attributes.iter().collect();
        sorted.sort_by_key(|(k, _)| k.as_str());
        for (key, value) in sorted {
            lines.push(Line::from(vec![
                Span::styled(format!("  :{}", key), Style::default().fg(colors.orange)),
                Span::styled(" = ", Style::default().fg(colors.fg1)),
                Span::styled(value.clone(), Style::default().fg(colors.fg0)),
            ]));
        }
    }

    lines
}

/// Format a single sample value with smart precision.
fn format_sample_value(v: f64) -> String {
    if v.is_nan() {
        return "NaN".to_string();
    }
    if v.is_infinite() {
        return if v.is_sign_positive() {
            "+Inf".to_string()
        } else {
            "-Inf".to_string()
        };
    }
    let abs = v.abs();
    if abs == 0.0 {
        "0".to_string()
    } else if !(1e-3..1e6).contains(&abs) {
        format!("{:.3e}", v)
    } else if abs >= 100.0 {
        format!("{:.2}", v)
    } else {
        format!("{:.4}", v)
    }
}
