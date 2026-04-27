//! Data viewer overlay - pure rendering layer.

mod heatmap;
mod plot;
mod table;

use super::{DataViewerState, ViewMode};
use crate::data::LoadedVariable;
use crate::theme::ThemeColors;
use crate::util::formatters::format_stat_value;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

/// Draw the data overlay.
pub fn draw_data_viewer(f: &mut Frame<'_>, state: &DataViewerState, colors: &ThemeColors) {
    if !state.visible {
        return;
    }

    let area = centered_rect(98, 98, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(format!(" Data Viewer - {} ", state.view_mode.name()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.bg2))
        .style(Style::default().bg(colors.bg0));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if let Some(ref error) = state.error {
        draw_error(f, inner, error, colors);
        return;
    }

    if state.variable.is_none() && state.error.is_none() {
        f.render_widget(
            Paragraph::new("Loading variable\u{2026}")
                .style(Style::default().fg(colors.fg1))
                .alignment(Alignment::Center),
            inner,
        );
        return;
    }

    if let Some(ref var) = state.variable {
        let has_selectors =
            var.ndim() > 2 || (var.ndim() == 2 && matches!(state.view_mode, ViewMode::Plot1D));
        let has_status = state.status_message.is_some();

        let mut constraints = vec![Constraint::Length(3)];
        if has_status {
            constraints.push(Constraint::Length(1));
        }
        constraints.push(Constraint::Min(5));
        constraints.push(Constraint::Length(1));

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(inner);

        let mut chunk_idx = 0;

        draw_header(f, chunks[chunk_idx], var, state, colors);
        chunk_idx += 1;

        if has_status {
            draw_status(f, chunks[chunk_idx], state, colors);
            chunk_idx += 1;
        }

        let content_area = chunks[chunk_idx];
        let content_split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(25), Constraint::Min(30)])
            .split(content_area);

        let sidebar_area = content_split[0];
        if has_selectors {
            let sidebar_split = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(6), Constraint::Min(6)])
                .split(sidebar_area);
            draw_stats_sidebar(f, sidebar_split[0], var, colors);
            draw_dimension_selectors(f, sidebar_split[1], state, var, colors);
        } else {
            draw_stats_sidebar(f, sidebar_area, var, colors);
        }

        match state.view_mode {
            ViewMode::Table => table::draw_table_view(f, content_split[1], state, var, colors),
            ViewMode::Plot1D => plot::draw_plot1d_view(f, content_split[1], state, var, colors),
            ViewMode::Heatmap => {
                heatmap::draw_heatmap_view(f, content_split[1], state, var, colors)
            },
        }
        chunk_idx += 1;

        draw_footer(f, chunks[chunk_idx], state, colors);
    }
}

fn draw_header(
    f: &mut Frame<'_>,
    area: Rect,
    var: &LoadedVariable,
    state: &DataViewerState,
    colors: &ThemeColors,
) {
    let mut title_parts = vec![Span::styled(
        var.name.clone(),
        Style::default()
            .fg(colors.yellow)
            .add_modifier(Modifier::BOLD),
    )];

    if let Some(long_name) = var.long_name() {
        title_parts.push(Span::styled(
            format!(" - {}", long_name),
            Style::default().fg(colors.fg0),
        ));
    }

    if let Some(units) = var.units() {
        title_parts.push(Span::styled(
            format!(" [{}]", units),
            Style::default().fg(colors.aqua),
        ));
    }

    if state.has_scale_offset() {
        let mode = if state.apply_scale_offset {
            "Scaled"
        } else {
            "Raw"
        };
        title_parts.push(Span::styled(
            format!(" ({})", mode),
            Style::default().fg(if state.apply_scale_offset {
                colors.green
            } else {
                colors.orange
            }),
        ));
    }

    let dims_info: Vec<String> = var
        .dim_names
        .iter()
        .zip(var.shape.iter())
        .map(|(name, size)| format!("{}:{}", name, size))
        .collect();

    let lines = vec![
        Line::from(title_parts),
        Line::from(vec![
            Span::styled("Shape: ", Style::default().fg(colors.green)),
            Span::styled(
                format!("[{}]", dims_info.join(", ")),
                Style::default().fg(colors.fg0),
            ),
            Span::styled(
                format!("  ({} total)", var.total_elements()),
                Style::default().fg(colors.gray),
            ),
        ]),
    ];

    f.render_widget(
        Paragraph::new(lines).alignment(Alignment::Center).block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(colors.bg2)),
        ),
        area,
    );
}

fn draw_status(f: &mut Frame<'_>, area: Rect, state: &DataViewerState, colors: &ThemeColors) {
    if let Some(ref msg) = state.status_message {
        f.render_widget(
            Paragraph::new(msg.as_str())
                .style(Style::default().fg(colors.yellow).bg(colors.bg1))
                .alignment(Alignment::Center),
            area,
        );
    }
}

fn draw_stats_sidebar(f: &mut Frame<'_>, area: Rect, var: &LoadedVariable, colors: &ThemeColors) {
    let mut lines = vec![];

    if let Some((min, max)) = var.min_max() {
        lines.push(Line::from(vec![
            Span::styled("Min:  ", Style::default().fg(colors.fg1)),
            Span::styled(format_stat_value(min), Style::default().fg(colors.aqua)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Max:  ", Style::default().fg(colors.fg1)),
            Span::styled(format_stat_value(max), Style::default().fg(colors.aqua)),
        ]));
    }

    if let Some(mean) = var.mean_value() {
        lines.push(Line::from(vec![
            Span::styled("Mean: ", Style::default().fg(colors.fg1)),
            Span::styled(format_stat_value(mean), Style::default().fg(colors.aqua)),
        ]));
    }

    if let Some(std) = var.std_value() {
        lines.push(Line::from(vec![
            Span::styled("Std:  ", Style::default().fg(colors.fg1)),
            Span::styled(format_stat_value(std), Style::default().fg(colors.aqua)),
        ]));
    }

    let valid = var.valid_count();
    let total = var.total_elements();
    if valid < total {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Valid: ", Style::default().fg(colors.fg1)),
            Span::styled(
                format!("{:.1}%", (valid as f64 / total as f64) * 100.0),
                Style::default().fg(colors.orange),
            ),
        ]));
    }

    f.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .title(" Statistics ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(colors.bg2))
                    .style(Style::default().bg(colors.bg0)),
            )
            .style(Style::default().fg(colors.fg0)),
        area,
    );
}

fn draw_dimension_selectors(
    f: &mut Frame<'_>,
    area: Rect,
    state: &DataViewerState,
    var: &LoadedVariable,
    colors: &ThemeColors,
) {
    let mut lines = Vec::new();

    lines.push(Line::from(Span::styled(
        "Display:",
        Style::default().fg(colors.green),
    )));

    match state.view_mode {
        ViewMode::Plot1D => {
            let dim_idx = state.slicing.display_dims.0;
            let dim_name = var
                .dim_names
                .get(dim_idx)
                .map(|s| s.as_str())
                .unwrap_or("?");
            let dim_size = var.shape.get(dim_idx).copied().unwrap_or(0);
            lines.push(Line::from(Span::styled(
                format!(" {}[{}]", dim_name, dim_size),
                Style::default().fg(colors.aqua),
            )));
        },
        ViewMode::Table | ViewMode::Heatmap => {
            for (label, dim_idx) in [
                ("Y", state.slicing.display_dims.0),
                ("X", state.slicing.display_dims.1),
            ] {
                let dim_name = var
                    .dim_names
                    .get(dim_idx)
                    .map(|s| s.as_str())
                    .unwrap_or("?");
                let dim_size = var.shape.get(dim_idx).copied().unwrap_or(0);
                lines.push(Line::from(vec![
                    Span::styled(format!(" {}: ", label), Style::default().fg(colors.fg1)),
                    Span::styled(
                        format!("{}[{}]", dim_name, dim_size),
                        Style::default().fg(colors.aqua),
                    ),
                ]));
            }
        },
    }

    let is_1d = matches!(state.view_mode, ViewMode::Plot1D);
    let has_slices = var.dim_names.iter().enumerate().any(|(i, _)| {
        if is_1d {
            i != state.slicing.display_dims.0
        } else {
            i != state.slicing.display_dims.0 && i != state.slicing.display_dims.1
        }
    });

    if has_slices {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Slices:",
            Style::default().fg(colors.green),
        )));

        for (i, (dim_name, &size)) in var.dim_names.iter().zip(var.shape.iter()).enumerate() {
            let should_skip = if is_1d {
                i == state.slicing.display_dims.0
            } else {
                i == state.slicing.display_dims.0 || i == state.slicing.display_dims.1
            };

            if should_skip {
                continue;
            }

            let is_active = state.slicing.active_dim_selector == Some(i);
            let idx = state.slicing.slice_indices.get(i).copied().unwrap_or(0);
            let label = format!(" {}={}/{}", dim_name, idx, size - 1);

            let style = if is_active {
                Style::default()
                    .fg(colors.bg0)
                    .bg(colors.yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(colors.fg0)
            };

            lines.push(Line::from(Span::styled(label, style)));
        }
    }

    f.render_widget(
        Paragraph::new(lines)
            .block(
                Block::default()
                    .title(" Dimensions ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(colors.bg2))
                    .style(Style::default().bg(colors.bg0)),
            )
            .style(Style::default().fg(colors.fg0)),
        area,
    );
}

fn draw_footer(f: &mut Frame<'_>, area: Rect, state: &DataViewerState, colors: &ThemeColors) {
    let scale_hint = if state.has_scale_offset() {
        " | Scale: O"
    } else {
        ""
    };
    let next_mode = state.view_mode.next().name();

    let help = match state.view_mode {
        ViewMode::Plot1D => format!(
            "{}: Tab | Y-Axis: y | Slice: s +/-[] | Navigate: ←→ | Copy: c{} | Close: q/Esc",
            next_mode, scale_hint
        ),
        ViewMode::Table => format!(
            "{}: Tab | Rotate: r | Axes: y/x | Slice: s +/-[] | Pan: hjkl Ctrl+U/D | Copy: c{} | Close: q/Esc",
            next_mode, scale_hint
        ),
        ViewMode::Heatmap => format!(
            "{}: Tab | Palette: ⇧C | Rotate: r | Axes: y/x | Slice: s +/-[] | Move: hjkl | Copy: c{} | Close: q/Esc",
            next_mode, scale_hint
        ),
    };

    f.render_widget(
        Paragraph::new(help)
            .style(Style::default().fg(colors.green))
            .alignment(Alignment::Center),
        area,
    );
}

fn draw_error(f: &mut Frame<'_>, area: Rect, error: &str, colors: &ThemeColors) {
    let lines = vec![
        Line::from(Span::styled(
            "Error Loading Variable",
            Style::default()
                .fg(colors.yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(error, Style::default().fg(colors.fg1))),
        Line::from(""),
        Line::from("Press Esc to close"),
    ];

    f.render_widget(
        Paragraph::new(lines)
            .style(Style::default().fg(colors.fg0))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true }),
        area,
    );
}

/// Format a value for axis labels with smart precision.
pub(super) fn format_axis_label(val: f64) -> String {
    if !val.is_finite() {
        return "?".to_string();
    }
    let abs_val = val.abs();
    if abs_val == 0.0 {
        "0".to_string()
    } else if !(1e-2..1e5).contains(&abs_val) {
        format!("{:.1e}", val)
    } else if abs_val >= 100.0 {
        format!("{:.0}", val)
    } else if abs_val >= 1.0 {
        format!("{:.1}", val)
    } else {
        format!("{:.2}", val)
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
