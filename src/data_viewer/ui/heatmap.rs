//! Heatmap view renderer for the data viewer.

use super::format_axis_label;
use crate::data::LoadedVariable;
use crate::data_viewer::DataViewerState;
use crate::theme::ThemeColors;
use crate::util::formatters::format_stat_value;
use ratatui::{
    layout::{Alignment, Rect},
    style::Style,
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Render the heatmap view.
pub(super) fn draw_heatmap_view(
    f: &mut Frame<'_>,
    area: Rect,
    state: &DataViewerState,
    var: &LoadedVariable,
    colors: &ThemeColors,
) {
    if var.ndim() < 2 {
        f.render_widget(
            Paragraph::new(Line::from("Heatmap requires 2D+ data"))
                .style(Style::default().fg(colors.fg0))
                .alignment(Alignment::Center),
            area,
        );
        return;
    }

    let row_dim = state.slicing.display_dims.0;
    let col_dim = state.slicing.display_dims.1;
    let apply_scale = state.apply_scale_offset;

    let data_2d = var.get_2d_slice(row_dim, col_dim, &state.slicing.slice_indices, apply_scale);

    if data_2d.is_empty() || data_2d[0].is_empty() {
        f.render_widget(
            Paragraph::new(Line::from("No data to display"))
                .style(Style::default().fg(colors.fg0))
                .alignment(Alignment::Center),
            area,
        );
        return;
    }

    let (auto_min, auto_max) = data_2d
        .iter()
        .flatten()
        .filter(|v| v.is_finite())
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), &v| {
            (min.min(v), max.max(v))
        });

    let mut range = auto_max - auto_min;
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

    let cursor_row = state.heat_cursor_row.min(rows.saturating_sub(1));
    let cursor_col = state.heat_cursor_col.min(cols.saturating_sub(1));
    let cursor_val = data_2d
        .get(cursor_row)
        .and_then(|row| row.get(cursor_col))
        .copied()
        .unwrap_or(f64::NAN);
    let row_coord = var.get_coord_label(row_dim, cursor_row);
    let col_coord = var.get_coord_label(col_dim, cursor_col);

    let mut slice_info = String::new();
    if var.ndim() > 2 {
        let slice_parts: Vec<String> = (0..var.ndim())
            .filter(|&i| i != row_dim && i != col_dim)
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

    let colorbar_height = 1;
    let axis_label_height = 1;
    let left_margin = 8u16;
    let right_margin = 8u16;

    let heatmap_area = Rect {
        x: inner.x + left_margin,
        y: inner.y + colorbar_height,
        width: inner.width.saturating_sub(left_margin + right_margin),
        height: inner
            .height
            .saturating_sub(colorbar_height + axis_label_height),
    };

    // Colorbar at top
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

    let min_label = format_axis_label(auto_min);
    let max_label = format_axis_label(auto_max);

    let min_x = colorbar_start.saturating_sub(min_label.len() as u16 + 1);
    for (i, ch) in min_label.chars().enumerate() {
        let x = min_x + i as u16;
        if x < inner.x + inner.width {
            if let Some(cell) = f.buffer_mut().cell_mut((x, inner.y)) {
                cell.set_char(ch).set_fg(colors.green);
            }
        }
    }

    let max_x = colorbar_start + colorbar_width as u16 + 1;
    for (i, ch) in max_label.chars().enumerate() {
        let x = max_x + i as u16;
        if x < inner.x + inner.width {
            if let Some(cell) = f.buffer_mut().cell_mut((x, inner.y)) {
                cell.set_char(ch).set_fg(colors.green);
            }
        }
    }

    if let Some(units) = var.units() {
        let unit_label = format!("[{}]", units);
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

    // Half-block heatmap rendering: each character = 2 vertical pixels (▀)
    let max_w_chars = heatmap_area.width as usize;
    let max_h_pixels = heatmap_area.height as usize * 2;

    if max_w_chars == 0 || max_h_pixels == 0 {
        return;
    }

    let scale = (max_h_pixels as f64 / rows as f64).min(max_w_chars as f64 / cols as f64);
    let disp_rows = ((rows as f64 * scale).floor() as usize).max(1);
    let disp_cols = ((cols as f64 * scale).floor() as usize).max(1);
    let char_rows = disp_rows.div_ceil(2);

    let offset_x_chars = ((max_w_chars - disp_cols) / 2) as u16;
    let offset_y_chars = ((heatmap_area.height as usize - char_rows) / 2) as u16;

    let row_step = (rows as f64) / (disp_rows as f64);
    let col_step = (cols as f64) / (disp_cols as f64);

    let get_color = |row_idx: usize, col_idx: usize| -> ratatui::style::Color {
        let raw_val = data_2d[row_idx][col_idx];
        if raw_val.is_finite() {
            state
                .color_palette
                .color(((raw_val - auto_min) / range).clamp(0.0, 1.0))
        } else {
            colors.gray
        }
    };

    for char_y in 0..char_rows {
        let top_y = char_y * 2;
        let bottom_y = top_y + 1;

        for px in 0..disp_cols {
            let col_idx = ((px as f64 * col_step).floor() as usize).min(cols - 1);

            let top_row_idx = ((top_y as f64 * row_step).floor() as usize).min(rows - 1);
            let top_color = get_color(top_row_idx, col_idx);

            let bottom_color = if bottom_y < disp_rows {
                let bottom_row_idx = ((bottom_y as f64 * row_step).floor() as usize).min(rows - 1);
                get_color(bottom_row_idx, col_idx)
            } else {
                colors.bg0
            };

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

    // Y-axis labels (left side)
    for &y_pos in &[0, disp_rows / 2, disp_rows.saturating_sub(1)] {
        if y_pos >= disp_rows {
            continue;
        }
        let data_row = ((y_pos as f64 * row_step).floor() as usize).min(rows - 1);
        let label = var.get_coord_label(row_dim, data_row);
        let label_short: String = label.chars().take(7).collect();
        let label_len = label_short.len() as u16;

        let char_row = y_pos / 2;
        let screen_y = heatmap_area.y + offset_y_chars + char_row as u16;
        if screen_y < heatmap_area.y + heatmap_area.height {
            let heatmap_start_x = heatmap_area.x + offset_x_chars;
            let label_start_x = if heatmap_start_x > (label_len + 1) {
                heatmap_start_x - label_len - 1
            } else {
                inner.x
            };

            for (i, ch) in label_short.chars().enumerate() {
                let x = label_start_x + i as u16;
                if x < heatmap_start_x {
                    if let Some(cell) = f.buffer_mut().cell_mut((x, screen_y)) {
                        cell.set_char(ch).set_fg(colors.green);
                    }
                }
            }
        }
    }

    // X-axis labels (bottom)
    let x_label_y = heatmap_area.y + offset_y_chars + char_rows as u16;
    if x_label_y < inner.y + inner.height {
        for &x_pos in &[0, disp_cols / 2, disp_cols.saturating_sub(1)] {
            if x_pos >= disp_cols {
                continue;
            }
            let data_col = ((x_pos as f64 * col_step).floor() as usize).min(cols - 1);
            let label = var.get_coord_label(col_dim, data_col);
            let label_short: String = label.chars().take(8).collect();

            let screen_x = heatmap_area.x + offset_x_chars + x_pos as u16;
            for (i, ch) in label_short.chars().enumerate() {
                let x = screen_x + i as u16;
                if x < inner.x + inner.width {
                    if let Some(cell) = f.buffer_mut().cell_mut((x, x_label_y)) {
                        cell.set_char(ch).set_fg(colors.green);
                    }
                }
            }
        }
    }

    // Cursor crosshair
    let cy = ((cursor_row as f64 + 0.5) * (disp_rows as f64) / (rows as f64)).floor() as usize;
    let cx = ((cursor_col as f64 + 0.5) * (disp_cols as f64) / (cols as f64)).floor() as usize;
    let cy = cy.min(disp_rows.saturating_sub(1));
    let cx = cx.min(disp_cols.saturating_sub(1));

    let cursor_char_row = cy / 2;
    let screen_y = heatmap_area.y + offset_y_chars + cursor_char_row as u16;
    let screen_x = heatmap_area.x + offset_x_chars + cx as u16;
    if screen_x < heatmap_area.x + heatmap_area.width
        && screen_y < heatmap_area.y + heatmap_area.height
    {
        if let Some(cell) = f.buffer_mut().cell_mut((screen_x, screen_y)) {
            cell.set_char('┼').set_fg(colors.yellow).set_bg(colors.bg1);
        }
    }
}
