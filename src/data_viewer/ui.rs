//! Data viewer overlay - pure rendering layer.

use super::{DataViewerState, ViewMode};
use crate::data::LoadedVariable;
use crate::util::formatters::format_stat_value;
use crate::shared::ThemeColors;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{
        Axis, Block, Borders, Cell, Chart, Clear, Dataset, GraphType, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Table, Wrap,
    },
    Frame,
};
/// Draw the data overlay.
pub fn draw_data_viewer(f: &mut Frame<'_>, state: &DataViewerState, colors: &ThemeColors) {
    if !state.visible {
        return;
    }

    let area = centered_rect(98, 98, f.area());

    // Clear the background
    f.render_widget(Clear, area);

    // Draw border
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

    if let Some(ref var) = state.variable {
        // Layout: header, status (if any), [left sidebar (stats + selectors) | main content], footer
        // Show selectors for: 3D+ variables, OR 2D variables in 1D plot mode
        let has_selectors =
            var.ndim() > 2 || (var.ndim() == 2 && matches!(state.view_mode, ViewMode::Plot1D));
        let has_status = state.status_message.is_some();

        let mut constraints = vec![
            Constraint::Length(3), // Header (name, shape)
        ];
        constraints.push(Constraint::Min(5)); // Content area (left sidebar + main view)
        if has_status {
            constraints.push(Constraint::Length(1)); // Status
        }
        constraints.push(Constraint::Length(1)); // Footer

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(inner);

        let mut chunk_idx = 0;

        // Draw header with variable info
        draw_header(f, chunks[chunk_idx], var, state, colors);
        chunk_idx += 1;

        // Split content area: left sidebar on left, main view on right
        let content_area = chunks[chunk_idx];
        let content_split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(25), Constraint::Min(30)])
            .split(content_area);

        // Left sidebar: statistics on top, dimension selectors below (if needed)
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

        // Draw main content based on view mode
        match state.view_mode {
            ViewMode::Table => draw_table_view(f, content_split[1], state, var, colors),
            ViewMode::Plot1D => draw_plot1d_view(f, content_split[1], state, var, colors),
            ViewMode::Heatmap => draw_heatmap_view(f, content_split[1], state, var, colors),
        }
        chunk_idx += 1;

        // Draw status if present
        if has_status {
            draw_status(f, chunks[chunk_idx], state, colors);
            chunk_idx += 1;
        }

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
    let mut lines = vec![];

    // First line: Variable name with optional long_name and units
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

    // Show scale/offset indicator if applicable
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

    lines.push(Line::from(title_parts));

    // Second line: Shape and dimensions (more compact)
    let dims_info: Vec<String> = var
        .dim_names
        .iter()
        .zip(var.shape.iter())
        .map(|(name, size)| format!("{}:{}", name, size))
        .collect();
    lines.push(Line::from(vec![
        Span::styled("Shape: ", Style::default().fg(colors.green)),
        Span::styled(
            format!("[{}]", dims_info.join(", ")),
            Style::default().fg(colors.fg0),
        ),
        Span::styled(
            format!("  ({} total)", var.total_elements()),
            Style::default().fg(colors.gray),
        ),
    ]));

    let paragraph = Paragraph::new(lines).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::BOTTOM)
            .border_style(Style::default().fg(colors.bg2)),
    );

    f.render_widget(paragraph, area);
}

fn draw_status(f: &mut Frame<'_>, area: Rect, state: &DataViewerState, colors: &ThemeColors) {
    if let Some(ref msg) = state.status_message {
        let paragraph = Paragraph::new(msg.as_str())
            .style(Style::default().fg(colors.yellow).bg(colors.bg1))
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area);
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

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" Statistics ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors.bg2))
                .style(Style::default().bg(colors.bg0)),
        )
        .style(Style::default().fg(colors.fg0));

    f.render_widget(paragraph, area);
}

fn draw_table_view(
    f: &mut Frame<'_>,
    area: Rect,
    state: &DataViewerState,
    var: &LoadedVariable,
    colors: &ThemeColors,
) {
    // Determine visible area
    let visible_rows = (area.height as usize).saturating_sub(4); // Account for border and header
    let col_width = 12;
    let row_header_width = 10; // Wider for coordinate labels
    let visible_cols =
        ((area.width as usize).saturating_sub(row_header_width + 2) / col_width).clamp(1, 20);

    let start_row = state.scroll.row;
    let start_col = state.scroll.col;
    let total_rows = get_view_rows(state, var);
    let total_cols = get_view_cols(state, var);

    // Get coordinate info for row/col dimensions
    let row_dim = state.slicing.display_dims.0;
    let col_dim = state.slicing.display_dims.1;
    let has_row_coords = var.ndim() > 1 && var.get_coordinate(row_dim).is_some();
    let has_col_coords = var.ndim() > 1 && var.get_coordinate(col_dim).is_some();

    // Get data slice efficiently (with scale/offset applied based on state)
    let apply_scale = state.apply_scale_offset;
    let data_slice = if var.ndim() == 0 {
        let raw = var.data.iter().next().copied().unwrap_or(f64::NAN);
        let val = if apply_scale {
            var.scale_value(raw)
        } else {
            raw
        };
        vec![vec![val]]
    } else if var.ndim() == 1 {
        let data: Vec<f64> = var
            .data
            .iter()
            .map(|&v| if apply_scale { var.scale_value(v) } else { v })
            .collect();
        vec![data]
    } else {
        var.get_2d_slice(row_dim, col_dim, &state.slicing.slice_indices, apply_scale)
    };

    // Build table rows from the slice
    let mut rows = Vec::new();
    let end_row = (start_row + visible_rows).min(total_rows);
    let end_col = (start_col + visible_cols).min(total_cols);

    for row_idx in start_row..end_row {
        let mut cells = Vec::new();

        // Row header with coordinate value if available
        let row_label = if has_row_coords {
            var.get_coord_label(row_dim, row_idx)
        } else {
            format!("{}", row_idx)
        };
        cells
            .push(Cell::from(format!("{:>9}", row_label)).style(Style::default().fg(colors.green)));

        for col_idx in start_col..end_col {
            let value = if var.ndim() <= 1 {
                data_slice[0].get(row_idx).copied().unwrap_or(f64::NAN)
            } else {
                data_slice
                    .get(row_idx)
                    .and_then(|row| row.get(col_idx))
                    .copied()
                    .unwrap_or(f64::NAN)
            };

            let formatted = format_value(value);
            cells.push(Cell::from(formatted).style(Style::default().fg(colors.aqua)));
        }

        rows.push(Row::new(cells));
    }

    // Build column header with coordinate values
    let mut header_cells = vec![Cell::from("").style(Style::default().fg(colors.green))];
    for col_idx in start_col..end_col {
        let col_label = if has_col_coords {
            var.get_coord_label(col_dim, col_idx)
        } else {
            format!("{}", col_idx)
        };
        header_cells.push(
            Cell::from(format!("{:>10}", col_label)).style(
                Style::default()
                    .fg(colors.green)
                    .add_modifier(Modifier::BOLD),
            ),
        );
    }

    // Build widths
    let mut widths = vec![Constraint::Length(row_header_width as u16)];
    for _ in 0..visible_cols {
        widths.push(Constraint::Length(col_width as u16));
    }

    // Build title with dimension info
    let row_dim_name = var
        .dim_names
        .get(row_dim)
        .map(|s| s.as_str())
        .unwrap_or("row");
    let col_dim_name = var
        .dim_names
        .get(col_dim)
        .map(|s| s.as_str())
        .unwrap_or("col");
    // Title with dimension info - names stay stable, no cursor info
    let title = format!(
        " {} │ Rows: {} │ Cols: {} ",
        var.name, row_dim_name, col_dim_name
    );

    let table = Table::new(rows, widths)
        .header(Row::new(header_cells).style(Style::default().add_modifier(Modifier::BOLD)))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors.bg2))
                .title(title)
                .title_style(Style::default().fg(colors.yellow)),
        )
        .style(Style::default().fg(colors.fg0));

    f.render_widget(table, area);

    // Draw scrollbar
    if total_rows > visible_rows || total_cols > visible_cols {
        let mut scrollbar_state =
            ScrollbarState::new(total_rows.saturating_sub(visible_rows)).position(start_row);
        f.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("^"))
                .end_symbol(Some("v")),
            area,
            &mut scrollbar_state,
        );
    }
}

fn draw_plot1d_view(
    f: &mut Frame<'_>,
    area: Rect,
    state: &DataViewerState,
    var: &LoadedVariable,
    colors: &ThemeColors,
) {
    // For 1D plot, use display_dims.0 as the dimension to plot
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
        let para = Paragraph::new("No data to display")
            .style(Style::default().fg(colors.fg0))
            .alignment(Alignment::Center);
        f.render_widget(para, area);
        return;
    }

    // Check for coordinate variable
    let has_coords = var.get_coordinate(slice_dim).is_some();

    // Keep only finite values
    let transformed: Vec<Option<f64>> = data
        .iter()
        .map(|&v| if v.is_finite() { Some(v) } else { None })
        .collect();

    // Find min/max for Y scaling
    let (min_val, max_val) = transformed
        .iter()
        .flatten()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), &v| {
            (min.min(v), max.max(v))
        });

    // Add padding to avoid edge clipping - 15% margin
    let padding = (max_val - min_val).abs() * 0.15;
    let (y_min, y_max) = (min_val - padding, max_val + padding);

    // Prepare data points - use coordinate values for X if available
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
        let para = Paragraph::new("No valid data to display")
            .style(Style::default().fg(colors.fg0))
            .alignment(Alignment::Center);
        f.render_widget(para, area);
        return;
    }

    // Get X bounds
    let (x_min, x_max) = chart_data
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), (x, _)| {
            (min.min(*x), max.max(*x))
        });

    // Get dimension name and units for X axis
    let dim_name = var
        .dim_names
        .get(slice_dim)
        .map(|s| s.as_str())
        .unwrap_or("index");

    let x_units = var
        .get_coordinate(slice_dim)
        .and_then(|c| c.units.as_ref())
        .map(|s| s.as_str());

    // Build slice info for title
    let mut slice_info = String::new();
    if var.ndim() > 1 {
        let slice_parts: Vec<String> = (0..var.ndim())
            .filter(|&i| i != slice_dim)
            .map(|i| {
                let dim_name = var.dim_names.get(i).map(|s| s.as_str()).unwrap_or("?");
                let idx = state.slicing.slice_indices.get(i).copied().unwrap_or(0);
                // Show coordinate value if available
                let val_str = var.get_coord_label(i, idx);
                format!("{}={}", dim_name, val_str)
            })
            .collect();
        if !slice_parts.is_empty() {
            slice_info = format!(" [{}]", slice_parts.join(", "));
        }
    }

    // Downsample to fit width
    let mut series: Vec<(f64, f64)> = chart_data;
    if area.width > 4 {
        let bins = (area.width as usize).saturating_sub(8).max(1);
        if series.len() > bins {
            let step = (series.len() as f64) / (bins as f64);
            let mut simple = Vec::with_capacity(bins);
            let mut pos = 0.0;
            while (pos as usize) < series.len() {
                let idx = (pos as usize).min(series.len() - 1);
                simple.push(series[idx]);
                pos += step;
            }
            series = simple;
        }
    }

    // Create dataset
    let mut datasets = vec![Dataset::default()
        .name(var.name.as_str())
        .marker(ratatui::symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(state.color_palette.color(0.6)))
        .data(&series)];

    // Add cursor as a vertical line
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
    if let Some(ref cursor_line) = cursor_line_opt {
        datasets.push(
            Dataset::default()
                .name("cursor")
                .graph_type(GraphType::Line)
                .style(Style::default().fg(colors.yellow))
                .data(cursor_line),
        );
    }

    // Create X axis with smart labels
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

    // Create Y axis
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

    // Build title with cursor readout and slice info
    // Use fixed width for cursor value to prevent shifting
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

/// Format axis label with smart precision.
fn format_axis_label(val: f64) -> String {
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

fn draw_heatmap_view(
    f: &mut Frame<'_>,
    area: Rect,
    state: &DataViewerState,
    var: &LoadedVariable,
    colors: &ThemeColors,
) {
    if var.ndim() < 2 {
        let para = Paragraph::new("Heatmap requires 2D+ data")
            .style(Style::default().fg(colors.fg0))
            .alignment(Alignment::Center);
        f.render_widget(para, area);
        return;
    }

    let row_dim = state.slicing.display_dims.0;
    let col_dim = state.slicing.display_dims.1;
    let apply_scale = state.apply_scale_offset;

    // Get full 2D slice (with scale/offset applied based on state)
    let data_2d = var.get_2d_slice(row_dim, col_dim, &state.slicing.slice_indices, apply_scale);

    if data_2d.is_empty() || data_2d[0].is_empty() {
        let para = Paragraph::new("No data to display")
            .style(Style::default().fg(colors.fg0))
            .alignment(Alignment::Center);
        f.render_widget(para, area);
        return;
    }

    // Find min/max (data is already transformed)
    let (auto_min, auto_max) = data_2d
        .iter()
        .flatten()
        .filter(|v| v.is_finite())
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), &v| {
            (min.min(v), max.max(v))
        });
    let (min_val, max_val) = (auto_min, auto_max);

    let mut range = max_val - min_val;
    if range.abs() < 1e-10 {
        range = 1.0;
    }

    let rows = data_2d.len();
    let cols = data_2d[0].len();

    let dim1_name = var
        .dim_names
        .get(row_dim)
        .map(|s| s.as_str())
        .unwrap_or("Y");
    let dim2_name = var
        .dim_names
        .get(col_dim)
        .map(|s| s.as_str())
        .unwrap_or("X");

    // Build cursor readout for title (data is already scaled/unscaled)
    let cursor_row = state.heat_cursor_row.min(rows.saturating_sub(1));
    let cursor_col = state.heat_cursor_col.min(cols.saturating_sub(1));
    let cursor_val = data_2d
        .get(cursor_row)
        .and_then(|row| row.get(cursor_col))
        .copied()
        .unwrap_or(f64::NAN);

    // Get coordinate labels for cursor position
    let row_coord = var.get_coord_label(row_dim, cursor_row);
    let col_coord = var.get_coord_label(col_dim, cursor_col);

    // Build slice info for title (show other sliced dimensions)
    let mut slice_info = String::new();
    if var.ndim() > 2 {
        let slice_parts: Vec<String> = (0..var.ndim())
            .filter(|&i| i != row_dim && i != col_dim)
            .map(|i| {
                let dim_name = var.dim_names.get(i).map(|s| s.as_str()).unwrap_or("?");
                let idx = state.slicing.slice_indices.get(i).copied().unwrap_or(0);
                let val_str = var.get_coord_label(i, idx);
                format!("{}={}", dim_name, val_str)
            })
            .collect();
        if !slice_parts.is_empty() {
            slice_info = format!(" [{}]", slice_parts.join(", "));
        }
    }

    // Build title with cursor readout using fixed width for value to prevent shifting
    let title = format!(
        " {} @ {}={}, {}={}: {:<12}{} │ Colormap: {} ",
        var.name,
        dim1_name,
        row_coord,
        dim2_name,
        col_coord,
        format_stat_value(cursor_val),
        slice_info,
        state.color_palette.name()
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.bg2))
        .title(title)
        .title_style(Style::default().fg(colors.yellow));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.width < 4 || inner.height < 4 {
        return;
    }

    // Reserve space for colorbar and axis labels
    let colorbar_height = 1;
    let axis_label_height = 1;
    let left_margin = 8; // For Y-axis labels
    let right_margin = 8; // Balance the left margin for symmetric centering

    let heatmap_area = Rect {
        x: inner.x + left_margin,
        y: inner.y + colorbar_height,
        width: inner.width.saturating_sub(left_margin + right_margin),
        height: inner
            .height
            .saturating_sub(colorbar_height + axis_label_height),
    };

    // Render colorbar at top with units
    let colorbar_width = 40.min((inner.width as usize).saturating_sub(20));
    let colorbar_start = inner.x
        + left_margin
        + ((heatmap_area.width as usize).saturating_sub(colorbar_width)) as u16 / 2;

    for i in 0..colorbar_width {
        let t = i as f64 / colorbar_width as f64;
        let color = state.color_palette.color(t);
        let x = colorbar_start + i as u16;
        if x < inner.x + inner.width {
            if let Some(cell) = f.buffer_mut().cell_mut((x, inner.y)) {
                cell.set_char('█').set_fg(color);
            }
        }
    }

    // Colorbar labels with smart formatting
    let min_label = format_axis_label(min_val);
    let max_label = format_axis_label(max_val);

    // Min label (left of colorbar)
    let min_x = colorbar_start.saturating_sub(min_label.len() as u16 + 1);
    for (i, ch) in min_label.chars().enumerate() {
        let x = min_x + i as u16;
        if x < inner.x + inner.width {
            if let Some(cell) = f.buffer_mut().cell_mut((x, inner.y)) {
                cell.set_char(ch).set_fg(colors.green);
            }
        }
    }

    // Max label (right of colorbar)
    let max_x = colorbar_start + colorbar_width as u16 + 1;
    for (i, ch) in max_label.chars().enumerate() {
        let x = max_x + i as u16;
        if x < inner.x + inner.width {
            if let Some(cell) = f.buffer_mut().cell_mut((x, inner.y)) {
                cell.set_char(ch).set_fg(colors.green);
            }
        }
    }

    // Units label if available
    if let Some(units) = var.units() {
        let unit_label = format!("[{}]", units);
        // let unit_x = inner.x + inner.width / 2 - unit_label.len() as u16 / 2;
        let unit_x = max_x + max_label.len() as u16 + 1;
        for (i, ch) in unit_label.chars().enumerate() {
            let x = unit_x + i as u16;
            if x < inner.x + inner.width {
                if let Some(cell) = f.buffer_mut().cell_mut((x, inner.y)) {
                    cell.set_char(ch).set_fg(colors.aqua);
                }
            }
        }
    }

    // Render dense heatmap using half-block technique (▀)
    // Each character shows 2 vertical pixels: foreground=top, background=bottom
    // With terminal char aspect ~2:1, each half-pixel is roughly square

    // Available space
    // - Horizontal: heatmap_area.width characters (1 char = 1 data column)
    // - Vertical: heatmap_area.height * 2 pixels (half-blocks give 2 pixels per char)
    let max_w_chars = heatmap_area.width as usize;
    let max_h_pixels = heatmap_area.height as usize * 2;

    if max_w_chars == 0 || max_h_pixels == 0 {
        return;
    }

    // Simple uniform scaling to fit data while preserving aspect ratio
    let scale = (max_h_pixels as f64 / rows as f64).min(max_w_chars as f64 / cols as f64);

    // Calculate display dimensions
    let disp_rows = ((rows as f64 * scale).floor() as usize).max(1); // Vertical pixels
    let disp_cols = ((cols as f64 * scale).floor() as usize).max(1); // Horizontal chars

    // Convert vertical pixels to character rows for rendering
    let char_rows = (disp_rows + 1) / 2; // Round up for odd rows

    // Center the heatmap in available space (within heatmap_area)
    let offset_x_chars = ((max_w_chars - disp_cols) / 2) as u16;
    let offset_y_chars = ((heatmap_area.height as usize - char_rows) / 2) as u16;

    // Use uniform step size for consistent sampling
    let row_step = (rows as f64) / (disp_rows as f64);
    let col_step = (cols as f64) / (disp_cols as f64);

    // Helper function to get color for a data value
    let get_color = |row_idx: usize, col_idx: usize| -> ratatui::style::Color {
        let raw_val = data_2d[row_idx][col_idx];
        if raw_val.is_finite() {
            state
                .color_palette
                .color(((raw_val - min_val) / range).clamp(0.0, 1.0))
        } else {
            colors.gray
        }
    };

    // Render using half-block characters (▀)
    // Process rows in pairs: foreground = top pixel, background = bottom pixel
    for char_y in 0..char_rows {
        let top_y = char_y * 2;
        let bottom_y = top_y + 1;

        for px in 0..disp_cols {
            let col_idx = (px as f64 * col_step).floor() as usize;
            let col_idx = col_idx.min(cols - 1);

            // Get top pixel color
            let top_row_idx = (top_y as f64 * row_step).floor() as usize;
            let top_row_idx = top_row_idx.min(rows - 1);
            let top_color = get_color(top_row_idx, col_idx);

            // Get bottom pixel color (or use background color if no bottom pixel)
            let bottom_color = if bottom_y < disp_rows {
                let bottom_row_idx = (bottom_y as f64 * row_step).floor() as usize;
                let bottom_row_idx = bottom_row_idx.min(rows - 1);
                get_color(bottom_row_idx, col_idx)
            } else {
                colors.bg0 // Use background color for unpaired bottom pixel
            };

            // Render one half-block character per pixel (maximum resolution)
            let screen_x = heatmap_area.x + offset_x_chars + px as u16;
            let screen_y = heatmap_area.y + offset_y_chars + char_y as u16;

            if screen_x < heatmap_area.x + heatmap_area.width
                && screen_y < heatmap_area.y + heatmap_area.height
            {
                if let Some(cell) = f.buffer_mut().cell_mut((screen_x, screen_y)) {
                    cell.set_char('▀').set_fg(top_color).set_bg(bottom_color);
                }
            }
        }
    }

    // Draw Y-axis labels (left side), aligned to the actual heatmap start so they stay
    // visually adjacent when the heatmap is horizontally centered.
    // Note: With half-block rendering, we work with pixel positions but display on char rows
    let y_label_positions = [0, disp_rows / 2, disp_rows.saturating_sub(1)];
    for &y_pos in &y_label_positions {
        if y_pos >= disp_rows {
            continue;
        }
        let data_row = (y_pos as f64 * row_step).floor() as usize;
        let data_row = data_row.min(rows - 1);
        let label = var.get_coord_label(row_dim, data_row);
        let label_short: String = label.chars().take(7).collect();
        let label_len = label_short.len() as u16;

        // Convert pixel row to character row for half-block rendering
        let char_row = y_pos / 2;
        let screen_y = heatmap_area.y + offset_y_chars + char_row as u16;
        if screen_y < heatmap_area.y + heatmap_area.height {
            // Right-align label immediately to the left of the heatmap drawing region
            let heatmap_start_x = heatmap_area.x + offset_x_chars;
            // Add a 1-char gap between label and heatmap, clamp to the inner left edge
            let label_start_x = if heatmap_start_x > (label_len + 1) {
                heatmap_start_x - label_len - 1
            } else {
                inner.x
            };

            for (i, ch) in label_short.chars().enumerate() {
                let x = label_start_x + i as u16;
                if x < heatmap_start_x {
                    // ensure we don't overwrite heatmap
                    if let Some(cell) = f.buffer_mut().cell_mut((x, screen_y)) {
                        cell.set_char(ch).set_fg(colors.green);
                    }
                }
            }
        }
    }

    // Draw X-axis labels (bottom) - position immediately after the actual heatmap pixels
    // With half-block rendering, use char_rows instead of disp_rows
    let x_label_y = heatmap_area.y + offset_y_chars + char_rows as u16;
    if x_label_y < inner.y + inner.height {
        let x_label_positions = [0, disp_cols / 2, disp_cols.saturating_sub(1)];
        for &x_pos in &x_label_positions {
            if x_pos >= disp_cols {
                continue;
            }
            let data_col = (x_pos as f64 * col_step).floor() as usize;
            let data_col = data_col.min(cols - 1);
            let label = var.get_coord_label(col_dim, data_col);
            let label_short: String = label.chars().take(8).collect();

            let screen_x = heatmap_area.x + offset_x_chars + x_pos as u16;
            for (i, ch) in label_short.chars().enumerate() {
                let x = screen_x + i as u16;
                // Allow labels to extend into full inner width (not just heatmap_area)
                if x < inner.x + inner.width {
                    if let Some(cell) = f.buffer_mut().cell_mut((x, x_label_y)) {
                        cell.set_char(ch).set_fg(colors.green);
                    }
                }
            }
        }
    }

    // Draw cursor as a simple cross at the data position
    // Use inverse mapping: find which display pixel corresponds to the cursor data position
    // This ensures the cursor is drawn at the exact pixel that represents the data cell
    let cy = ((cursor_row as f64 + 0.5) * (disp_rows as f64) / (rows as f64)).floor() as usize;
    let cx = ((cursor_col as f64 + 0.5) * (disp_cols as f64) / (cols as f64)).floor() as usize;
    let cy = cy.min(disp_rows.saturating_sub(1));
    let cx = cx.min(disp_cols.saturating_sub(1));

    // Convert pixel row to character row for half-block rendering
    let cursor_char_row = cy / 2;
    let screen_y = heatmap_area.y + offset_y_chars + cursor_char_row as u16;
    let screen_x = heatmap_area.x + offset_x_chars + cx as u16;
    if screen_x < heatmap_area.x + heatmap_area.width
        && screen_y < heatmap_area.y + heatmap_area.height
    {
        if let Some(cell) = f.buffer_mut().cell_mut((screen_x, screen_y)) {
            // Use '┼' for cursor with high-visibility colors
            cell.set_char('┼').set_fg(colors.yellow).set_bg(colors.bg1);
        }
    }
}

fn draw_dimension_selectors(
    f: &mut Frame<'_>,
    area: Rect,
    state: &DataViewerState,
    var: &LoadedVariable,
    colors: &ThemeColors,
) {
    let mut lines = Vec::new();

    // Display dimensions - show each on its own line for vertical layout
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
        lines.push(Line::from("")); // Empty line separator
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

            // Compact format: dimname=idx/max
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

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" Dimensions ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors.bg2))
                .style(Style::default().bg(colors.bg0)),
        )
        .style(Style::default().fg(colors.fg0));

    f.render_widget(paragraph, area);
}

fn draw_footer(f: &mut Frame<'_>, area: Rect, state: &DataViewerState, colors: &ThemeColors) {
    // Build help string - add scale/offset hint if applicable
    let scale_hint = if state.has_scale_offset() {
        " | Scale: O"
    } else {
        ""
    };

    // Show the next view mode that Tab will switch to
    let next_mode = state.view_mode.next().name();

    let help = match state.view_mode {
        ViewMode::Plot1D => format!(
            "{}: Tab | Colors: C | Y-Axis: Y | Slice: S +/-[] | Navigate: ←→ | Copy: Ctrl+C{} | Close: q/Esc",
            next_mode, scale_hint
        ),
        ViewMode::Table => format!(
            "{}: Tab | Colors: C | Rotate: R | Axes: YX | Slice: S +/-[] | Pan: hjkl Ctrl+U/D | Copy: Ctrl+C{} | Close: q/Esc",
            next_mode, scale_hint
        ),
        ViewMode::Heatmap => format!(
            "{}: Tab | Colors: C | Rotate: R | Axes: YX | Slice: S +/-[] | Move: hjkl | Copy: Ctrl+C{} | Close: q/Esc",
            next_mode, scale_hint
        ),
    };
    let paragraph = Paragraph::new(help)
        .style(Style::default().fg(colors.green))
        .alignment(Alignment::Center);
    f.render_widget(paragraph, area);
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

    let paragraph = Paragraph::new(lines)
        .style(Style::default().fg(colors.fg0))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn format_value(val: f64) -> String {
    if val.is_nan() {
        "NaN".to_string()
    } else if val.is_infinite() {
        if val.is_sign_positive() {
            "+Inf".to_string()
        } else {
            "-Inf".to_string()
        }
    } else if val.abs() < 0.001 || val.abs() >= 10000.0 {
        format!("{:>10.3e}", val)
    } else {
        format!("{:>10.4}", val)
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

/// Get number of rows for current view (helper for rendering).
fn get_view_rows(state: &DataViewerState, var: &LoadedVariable) -> usize {
    if var.ndim() == 0 {
        1
    } else if var.ndim() == 1 {
        var.shape[0]
    } else {
        var.shape[state.slicing.display_dims.0]
    }
}

/// Get number of columns for current view (helper for rendering).
fn get_view_cols(state: &DataViewerState, var: &LoadedVariable) -> usize {
    if var.ndim() <= 1 {
        1
    } else {
        var.shape[state.slicing.display_dims.1]
    }
}
