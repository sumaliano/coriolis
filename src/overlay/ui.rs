//! Data viewer overlay - pure rendering layer.

use super::{OverlayState, ViewMode};
use crate::data::LoadedVariable;
use crate::ui::ThemeColors;
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
pub fn draw_overlay(f: &mut Frame<'_>, state: &OverlayState, colors: &ThemeColors) {
    if !state.visible {
        return;
    }

    let area = centered_rect(90, 90, f.area());

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
        // Layout: header, main content, dimension selectors (if 3D+), footer
        let has_selectors = var.ndim() > 2;
        let constraints = if has_selectors {
            vec![
                Constraint::Length(4), // Header (2 lines + border)
                Constraint::Min(5),    // Content
                Constraint::Length(4), // Dimension selectors (2 lines + border)
                Constraint::Length(2), // Footer
            ]
        } else {
            vec![
                Constraint::Length(4), // Header (2 lines + border)
                Constraint::Min(5),    // Content
                Constraint::Length(2), // Footer
            ]
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(inner);

        // Draw header with variable info
        draw_header(f, chunks[0], var, colors);

        // Draw main content based on view mode
        match state.view_mode {
            ViewMode::Table => draw_table_view(f, chunks[1], state, var, colors),
            ViewMode::Plot1D => draw_plot1d_view(f, chunks[1], state, var, colors),
            ViewMode::Heatmap => draw_heatmap_view(f, chunks[1], state, var, colors),
        }

        // Draw dimension selectors for 3D+ data
        if has_selectors {
            draw_dimension_selectors(f, chunks[2], state, var, colors);
            draw_footer(f, chunks[3], state, colors);
        } else {
            draw_footer(f, chunks[2], state, colors);
        }
    }
}

fn draw_header(f: &mut Frame<'_>, area: Rect, var: &LoadedVariable, colors: &ThemeColors) {
    let mut lines = vec![];

    // Variable name and type
    let shape_str = format!("{:?}", var.shape);
    let dims_str = var.dim_names.join(", ");
    lines.push(Line::from(Span::styled(
        format!("{} ({}) | Shape: {} | Dims: [{}]", var.name, var.dtype, shape_str, dims_str),
        Style::default().fg(colors.yellow).add_modifier(Modifier::BOLD),
    )));

    // Statistics
    let mut stats = Vec::new();
    if let Some((min, max)) = var.min_max() {
        stats.push(format!("Min: {:.6}", min));
        stats.push(format!("Max: {:.6}", max));
    }
    if let Some(mean) = var.mean_value() {
        stats.push(format!("Mean: {:.6}", mean));
    }
    if let Some(std) = var.std_value() {
        stats.push(format!("Std: {:.6}", std));
    }
    stats.push(format!("Valid: {}", var.valid_count()));

    if !stats.is_empty() {
        lines.push(Line::from(Span::styled(
            format!("Statistics: {}", stats.join(" | ")),
            Style::default().fg(colors.fg0),
        )));
    }

    let paragraph = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(colors.bg2)),
        );

    f.render_widget(paragraph, area);
}

fn draw_table_view(
    f: &mut Frame<'_>,
    area: Rect,
    state: &OverlayState,
    var: &LoadedVariable,
    colors: &ThemeColors,
) {
    // Determine visible area
    let visible_rows = (area.height as usize).saturating_sub(4); // Account for border and header
    let col_width = 12;
    let visible_cols = ((area.width as usize).saturating_sub(6) / col_width).max(1).min(20); // Limit to 20 cols

    let start_row = state.scroll.row;
    let start_col = state.scroll.col;
    let total_rows = get_view_rows(state, var);
    let total_cols = get_view_cols(state, var);

    // Get data slice efficiently - avoid repeated get_value calls
    let data_slice = if var.ndim() == 0 {
        let scalar = var.data.iter().next().copied().unwrap_or(f64::NAN);
        vec![vec![scalar]]
    } else if var.ndim() == 1 {
        let data: Vec<f64> = var.data.iter().copied().collect();
        vec![data]
    } else {
        // Get 2D slice once - much faster than repeated get_value calls
        var.get_2d_slice(state.slicing.display_dims.0, state.slicing.display_dims.1, &state.slicing.slice_indices)
    };

    // Build table rows from the slice
    let mut rows = Vec::new();

    let end_row = (start_row + visible_rows).min(total_rows);
    let end_col = (start_col + visible_cols).min(total_cols);

    for row_idx in start_row..end_row {
        let mut cells = Vec::new();
        // Row header
        cells.push(Cell::from(format!("{:>4}", row_idx)).style(Style::default().fg(colors.green)));

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

    // Build header
    let mut header_cells = vec![Cell::from("").style(Style::default().fg(colors.green))];
    for col_idx in start_col..((start_col + visible_cols).min(total_cols)) {
        header_cells.push(
            Cell::from(format!("{:>10}", col_idx))
                .style(Style::default().fg(colors.green).add_modifier(Modifier::BOLD)),
        );
    }

    // Build widths
    let mut widths = vec![Constraint::Length(5)]; // Row index column
    for _ in 0..visible_cols {
        widths.push(Constraint::Length(col_width as u16));
    }

    let table = Table::new(rows, widths)
        .header(Row::new(header_cells).style(Style::default().add_modifier(Modifier::BOLD)))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors.bg2))
                .title(format!(" {} (Table) ", var.name))
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
    state: &OverlayState,
    var: &LoadedVariable,
    colors: &ThemeColors,
) {
    // For 1D plot, use display_dims.0 as the dimension to plot
    // This allows the user to choose which dimension to visualize
    let slice_dim = if var.ndim() <= 1 {
        0
    } else {
        state.slicing.display_dims.0
    };

    let data = if var.ndim() <= 1 {
        var.data.iter().copied().collect::<Vec<f64>>()
    } else {
        var.get_1d_slice(slice_dim, &state.slicing.slice_indices)
    };

    if data.is_empty() {
        let para = Paragraph::new("No data to display")
            .style(Style::default().fg(colors.fg0))
            .alignment(Alignment::Center);
        f.render_widget(para, area);
        return;
    }

    // Find min/max for scaling
    let (min_val, max_val) = data
        .iter()
        .copied()
        .filter(|v: &f64| v.is_finite())
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), v: f64| {
            (min.min(v), max.max(v))
        });

    // Add padding to avoid edge clipping - 10% margin
    let padding = (max_val - min_val).abs() * 0.3;
    let y_min = min_val - padding;
    let y_max = max_val + padding;

    // Prepare data points for Chart widget
    let chart_data: Vec<(f64, f64)> = data
        .iter()
        .enumerate()
        .filter(|(_, &v): &(usize, &f64)| v.is_finite())
        .map(|(i, &v)| (i as f64, v))
        .collect();

    if chart_data.is_empty() {
        let para = Paragraph::new("No valid data to display")
            .style(Style::default().fg(colors.fg0))
            .alignment(Alignment::Center);
        f.render_widget(para, area);
        return;
    }

    let x_max = (data.len() - 1) as f64;

    // Get dimension name for the slice being displayed
    let dim_name = var
        .dim_names
        .get(slice_dim)
        .map(|s| s.as_str())
        .unwrap_or("index");

    // Build slice info for title (what slices are active)
    let mut slice_info = String::new();
    if var.ndim() > 1 {
        let slice_parts: Vec<String> = (0..var.ndim())
            .filter(|&i| i != slice_dim)
            .map(|i| {
                let dim_name = var.dim_names.get(i).map(|s| s.as_str()).unwrap_or("?");
                let idx = state.slicing.slice_indices.get(i).copied().unwrap_or(0);
                format!("{}={}", dim_name, idx)
            })
            .collect();
        if !slice_parts.is_empty() {
            slice_info = format!(" [{}]", slice_parts.join(", "));
        }
    }

    // Create dataset with thicker line
    let datasets = vec![Dataset::default()
        .name(var.name.as_str())
        .marker(ratatui::symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(state.color_palette.color(0.6)))
        .data(&chart_data)];

    // Create axes with better formatting
    let x_labels = vec![
        "0".to_string(),
        format!("{:.0}", x_max / 2.0),
        format!("{:.0}", x_max),
    ];

    let x_axis = Axis::default()
        .title(format!("Index ({})", dim_name))
        .style(Style::default().fg(colors.fg0))
        .bounds([0.0, x_max])
        .labels(x_labels);

    let y_labels = vec![
        format!("{:.3e}", y_min),
        format!("{:.3e}", (y_min + y_max) / 2.0),
        format!("{:.3e}", y_max),
    ];

    let y_axis = Axis::default()
        .title("Value")
        .style(Style::default().fg(colors.fg0))
        .bounds([y_min, y_max])
        .labels(y_labels);

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors.bg2))
                .title(format!(" {}{} ", var.name, slice_info))
                .title_style(Style::default().fg(colors.yellow)),
        )
        .x_axis(x_axis)
        .y_axis(y_axis);

    f.render_widget(chart, area);
}

fn draw_heatmap_view(
    f: &mut Frame<'_>,
    area: Rect,
    state: &OverlayState,
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

    // Get 2D slice
    let data_2d = var.get_2d_slice(
        state.slicing.display_dims.0,
        state.slicing.display_dims.1,
        &state.slicing.slice_indices,
    );

    if data_2d.is_empty() || data_2d[0].is_empty() {
        let para = Paragraph::new("No data to display")
            .style(Style::default().fg(colors.fg0))
            .alignment(Alignment::Center);
        f.render_widget(para, area);
        return;
    }

    // Find min/max
    let (min_val, max_val) = data_2d
        .iter()
        .flatten()
        .filter(|v| v.is_finite())
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), &v| {
            (min.min(v), max.max(v))
        });

    let range = if (max_val - min_val).abs() < 1e-10 {
        1.0
    } else {
        max_val - min_val
    };

    let rows = data_2d.len();
    let cols = data_2d[0].len();

    let dim1_name = var
        .dim_names
        .get(state.slicing.display_dims.0)
        .map(|s| s.as_str())
        .unwrap_or("dim0");
    let dim2_name = var
        .dim_names
        .get(state.slicing.display_dims.1)
        .map(|s| s.as_str())
        .unwrap_or("dim1");

    // Render with direct buffer writes for dense heatmap
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors.bg2))
        .title(format!(" {} | Y: {} | X: {} | {} ", var.name, dim1_name, dim2_name, state.color_palette.name()))
        .title_style(Style::default().fg(colors.yellow));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.width < 4 || inner.height < 4 {
        return;
    }

    // Reserve space for colorbar
    let colorbar_height = 1;
    let heatmap_area = Rect {
        x: inner.x,
        y: inner.y + colorbar_height,
        width: inner.width,
        height: inner.height.saturating_sub(colorbar_height),
    };

    // Render colorbar at top
    let colorbar_width = 50.min(inner.width as usize);
    let colorbar_start = inner.x + ((inner.width as usize - colorbar_width) / 2) as u16;

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

    // Render min/max labels
    let min_label = format!("{:.2e}", min_val);
    let max_label = format!("{:.2e}", max_val);

    for (i, ch) in min_label.chars().enumerate() {
        let x = inner.x + i as u16;
        if x < inner.x + inner.width {
            if let Some(cell) = f.buffer_mut().cell_mut((x, inner.y)) {
                cell.set_char(ch).set_fg(colors.green);
            }
        }
    }

    let max_x_start = inner.x + inner.width - max_label.len() as u16;
    for (i, ch) in max_label.chars().enumerate() {
        let x = max_x_start + i as u16;
        if x < inner.x + inner.width {
            if let Some(cell) = f.buffer_mut().cell_mut((x, inner.y)) {
                cell.set_char(ch).set_fg(colors.green);
            }
        }
    }

    // Render dense heatmap - adjust for terminal character aspect ratio
    // Terminal chars are ~2:1 (height:width), so use 2 chars per pixel horizontally
    let heatmap_height = heatmap_area.height as usize;
    let heatmap_width = heatmap_area.width as usize;

    // Calculate aspect-corrected dimensions
    // Each pixel is 2 terminal chars wide to approximate square aspect ratio
    let pixel_width = 2;
    let display_cols = heatmap_width / pixel_width;

    // Sample the data to fit the display area with square-ish pixels
    let row_step = (rows as f64 / heatmap_height as f64).max(1.0);
    let col_step = (cols as f64 / display_cols as f64).max(1.0);

    for y in 0..heatmap_height {
        let row_idx = ((y as f64) * row_step) as usize;
        if row_idx >= rows {
            break;
        }

        for px in 0..display_cols {
            let col_idx = ((px as f64) * col_step) as usize;
            if col_idx >= cols {
                break;
            }

            let val = data_2d[row_idx][col_idx];
            let color = if val.is_finite() {
                let normalized = ((val - min_val) / range).clamp(0.0, 1.0);
                state.color_palette.color(normalized)
            } else {
                colors.gray
            };

            // Draw pixel_width characters for each pixel (to make it square-ish)
            for i in 0..pixel_width {
                let screen_x = heatmap_area.x + (px * pixel_width + i) as u16;
                let screen_y = heatmap_area.y + y as u16;

                if screen_x >= heatmap_area.x + heatmap_area.width {
                    break;
                }
                if screen_y >= heatmap_area.y + heatmap_area.height {
                    break;
                }

                if let Some(cell) = f.buffer_mut().cell_mut((screen_x, screen_y)) {
                    if val.is_finite() {
                        cell.set_char('█').set_fg(color);
                    } else {
                        cell.set_char('?').set_fg(color);
                    }
                }
            }
        }
    }
}

fn draw_dimension_selectors(
    f: &mut Frame<'_>,
    area: Rect,
    state: &OverlayState,
    var: &LoadedVariable,
    colors: &ThemeColors,
) {
    let mut lines = Vec::new();

    // First line: Display dimensions (different for 1D vs 2D/Heatmap)
    let mut display_spans = vec![
        Span::styled("Display: ", Style::default().fg(colors.green)),
    ];

    match state.view_mode {
        ViewMode::Plot1D => {
            // 1D plot only shows one dimension
            let dim_name = var.dim_names.get(state.slicing.display_dims.0).map(|s| s.as_str()).unwrap_or("?");
            let dim_size = var.shape.get(state.slicing.display_dims.0).copied().unwrap_or(0);
            display_spans.push(Span::styled(
                format!("{}[{}]", dim_name, dim_size),
                Style::default().fg(colors.aqua),
            ));
        }
        ViewMode::Table | ViewMode::Heatmap => {
            // 2D views show both dimensions
            let dim1_name = var.dim_names.get(state.slicing.display_dims.0).map(|s| s.as_str()).unwrap_or("?");
            let dim2_name = var.dim_names.get(state.slicing.display_dims.1).map(|s| s.as_str()).unwrap_or("?");
            let dim1_size = var.shape.get(state.slicing.display_dims.0).copied().unwrap_or(0);
            let dim2_size = var.shape.get(state.slicing.display_dims.1).copied().unwrap_or(0);

            display_spans.push(Span::styled(
                format!("Y: {}[{}] ", dim1_name, dim1_size),
                Style::default().fg(colors.aqua),
            ));
            display_spans.push(Span::styled(
                format!("X: {}[{}]", dim2_name, dim2_size),
                Style::default().fg(colors.aqua),
            ));
        }
    }

    lines.push(Line::from(display_spans));

    // Second line: Slice selectors (for non-display dimensions)
    let mut slice_spans = vec![
        Span::styled("Slices: ", Style::default().fg(colors.green)),
    ];

    // For 1D plots, only display_dims.0 is used, so we can slice display_dims.1 too
    let is_1d = matches!(state.view_mode, ViewMode::Plot1D);

    let has_slices = var.dim_names.iter().zip(var.shape.iter()).enumerate()
        .any(|(i, _)| {
            if is_1d {
                i != state.slicing.display_dims.0
            } else {
                i != state.slicing.display_dims.0 && i != state.slicing.display_dims.1
            }
        });

    if has_slices {
        for (i, (dim_name, &size)) in var.dim_names.iter().zip(var.shape.iter()).enumerate() {
            // Skip display dimensions
            // For 1D: only skip display_dims.0
            // For 2D/Heatmap: skip both display_dims.0 and display_dims.1
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

            let style = if is_active {
                Style::default()
                    .fg(colors.bg0)
                    .bg(colors.yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(colors.fg0)
            };

            slice_spans.push(Span::styled(
                format!(" {}={}/{} ", dim_name, idx, size - 1),
                style,
            ));
        }
        lines.push(Line::from(slice_spans));
    } else {
        slice_spans.push(Span::styled(
            "(none - 2D data)",
            Style::default().fg(colors.gray),
        ));
        lines.push(Line::from(slice_spans));
    }

    let paragraph = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(colors.bg2)),
        );

    f.render_widget(paragraph, area);
}

fn draw_footer(f: &mut Frame<'_>, area: Rect, state: &OverlayState, colors: &ThemeColors) {
    let help = match state.view_mode {
        ViewMode::Plot1D => "Tab: View | C: Palette | Y: Change Dim | S: Slice Dim | PgUp/PgDn: Slice | Esc: Close",
        ViewMode::Table | ViewMode::Heatmap => "Tab: View | C: Palette | R: Rotate | Y/X: Change Dims | S: Slice Dim | PgUp/PgDn: Slice | Esc: Close",
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
            Style::default().fg(colors.yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(error, Style::default().fg(colors.fg0))),
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
fn get_view_rows(state: &OverlayState, var: &LoadedVariable) -> usize {
    if var.ndim() == 0 {
        1
    } else if var.ndim() == 1 {
        var.shape[0]
    } else {
        var.shape[state.slicing.display_dims.0]
    }
}

/// Get number of columns for current view (helper for rendering).
fn get_view_cols(state: &OverlayState, var: &LoadedVariable) -> usize {
    if var.ndim() <= 1 {
        1
    } else {
        var.shape[state.slicing.display_dims.1]
    }
}
