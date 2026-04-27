//! 1D plot view renderer for the data viewer.

use super::format_axis_label;
use crate::data::LoadedVariable;
use crate::data_viewer::DataViewerState;
use crate::theme::ThemeColors;
use crate::util::formatters::format_stat_value;
use ratatui::{
    layout::{Alignment, Rect},
    style::Style,
    text::Line,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph},
    Frame,
};

/// Render the 1D plot view.
pub(super) fn draw_plot1d_view(
    f: &mut Frame<'_>,
    area: Rect,
    state: &DataViewerState,
    var: &LoadedVariable,
    colors: &ThemeColors,
) {
    let slice_dim = if var.ndim() <= 1 {
        0
    } else {
        state.slicing.display_dims.0
    };

    let apply_scale = state.apply_scale_offset;
    let data = if var.ndim() <= 1 {
        var.data
            .iter()
            .map(|&v| if apply_scale { var.scale_value(v) } else { v })
            .collect::<Vec<f64>>()
    } else {
        var.get_1d_slice(slice_dim, &state.slicing.slice_indices, apply_scale)
    };

    if data.is_empty() {
        f.render_widget(
            Paragraph::new(Line::from("No data to display"))
                .style(Style::default().fg(colors.fg0))
                .alignment(Alignment::Center),
            area,
        );
        return;
    }

    let has_coords = var.get_coordinate(slice_dim).is_some();

    let transformed: Vec<Option<f64>> = data
        .iter()
        .map(|&v| if v.is_finite() { Some(v) } else { None })
        .collect();

    let (min_val, max_val) = transformed
        .iter()
        .flatten()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), &v| {
            (min.min(v), max.max(v))
        });

    let padding = (max_val - min_val).abs() * 0.15;
    let (y_min, y_max) = (min_val - padding, max_val + padding);

    let chart_data: Vec<(f64, f64)> = if has_coords {
        transformed
            .iter()
            .enumerate()
            .filter_map(|(i, v)| {
                let x = var.get_coord_value(slice_dim, i)?;
                let y = (*v)?;
                Some((x, y))
            })
            .collect()
    } else {
        transformed
            .iter()
            .enumerate()
            .filter_map(|(i, v)| v.map(|y| (i as f64, y)))
            .collect()
    };

    if chart_data.is_empty() {
        f.render_widget(
            Paragraph::new(Line::from("No valid data to display"))
                .style(Style::default().fg(colors.fg0))
                .alignment(Alignment::Center),
            area,
        );
        return;
    }

    let (x_min, x_max) = chart_data
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), (x, _)| {
            (min.min(*x), max.max(*x))
        });

    let dim_name = var
        .dim_names
        .get(slice_dim)
        .map(|s| s.as_str())
        .unwrap_or("index");
    let x_units = var
        .get_coordinate(slice_dim)
        .and_then(|c| c.units.as_ref())
        .map(|s| s.as_str());

    let mut slice_info = String::new();
    if var.ndim() > 1 {
        let slice_parts: Vec<String> = (0..var.ndim())
            .filter(|&i| i != slice_dim)
            .map(|i| {
                let name = var.dim_names.get(i).map(|s| s.as_str()).unwrap_or("?");
                let idx = state.slicing.slice_indices.get(i).copied().unwrap_or(0);
                format!("{}={}", name, var.get_coord_label(i, idx))
            })
            .collect();
        if !slice_parts.is_empty() {
            slice_info = format!(" [{}]", slice_parts.join(", "));
        }
    }

    let mut series: Vec<(f64, f64)> = chart_data;
    if area.width > 4 {
        let bins = (area.width as usize).saturating_sub(8).max(1);
        if series.len() > bins {
            let step = (series.len() as f64) / (bins as f64);
            let mut simple = Vec::with_capacity(bins);
            let mut pos = 0.0f64;
            while (pos as usize) < series.len() {
                simple.push(series[(pos as usize).min(series.len() - 1)]);
                pos += step;
            }
            series = simple;
        }
    }

    let cursor_idx = state.plot_cursor;
    let cursor_x = if has_coords {
        var.get_coord_value(slice_dim, cursor_idx)
            .unwrap_or(cursor_idx as f64)
    } else {
        cursor_idx as f64
    };
    let mut cursor_line_opt: Option<Vec<(f64, f64)>> = None;
    if cursor_x >= x_min && cursor_x <= x_max {
        cursor_line_opt = Some(vec![(cursor_x, y_min), (cursor_x, y_max)]);
    }

    let mut datasets = vec![Dataset::default()
        .name(var.name.as_str())
        .marker(ratatui::symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(state.color_palette.color(0.6)))
        .data(&series)];

    if let Some(ref cursor_line) = cursor_line_opt {
        datasets.push(
            Dataset::default()
                .name("cursor")
                .graph_type(GraphType::Line)
                .style(Style::default().fg(colors.yellow))
                .data(cursor_line),
        );
    }

    let x_labels = vec![
        format_axis_label(x_min),
        format_axis_label((x_min + x_max) / 2.0),
        format_axis_label(x_max),
    ];
    let x_axis_title = match x_units {
        Some(u) if !u.is_empty() => format!("{} [{}]", dim_name, u),
        _ => dim_name.to_string(),
    };
    let x_axis = Axis::default()
        .title(x_axis_title)
        .style(Style::default().fg(colors.fg0))
        .bounds([x_min, x_max])
        .labels(x_labels);

    let y_labels = vec![
        format_axis_label(y_min),
        format_axis_label((y_min + y_max) / 2.0),
        format_axis_label(y_max),
    ];
    let y_units = var.units().unwrap_or("");
    let y_axis_title = if y_units.is_empty() {
        "Value".to_string()
    } else {
        format!("[{}]", y_units)
    };
    let y_axis = Axis::default()
        .title(y_axis_title)
        .style(Style::default().fg(colors.fg0))
        .bounds([y_min, y_max])
        .labels(y_labels);

    let cursor_val = data.get(cursor_idx).copied().unwrap_or(f64::NAN);
    let cursor_coord = var.get_coord_label(slice_dim, cursor_idx);
    let title = format!(
        " {} @ {}={}: {:<12}{} ",
        var.name,
        dim_name,
        cursor_coord,
        format_stat_value(cursor_val),
        slice_info
    );

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors.bg2))
                .title(title)
                .title_style(Style::default().fg(colors.yellow)),
        )
        .x_axis(x_axis)
        .y_axis(y_axis);

    f.render_widget(chart, area);
}
