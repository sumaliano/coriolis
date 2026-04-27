//! Table view renderer for the data viewer.

use crate::data::LoadedVariable;
use crate::data_viewer::DataViewerState;
use crate::theme::ThemeColors;
use ratatui::layout::Rect;
use ratatui::{
    layout::Constraint,
    style::{Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table},
    Frame,
};

/// Render the table view.
pub(super) fn draw_table_view(
    f: &mut Frame<'_>,
    area: Rect,
    state: &DataViewerState,
    var: &LoadedVariable,
    colors: &ThemeColors,
) {
    let visible_rows = (area.height as usize).saturating_sub(4);
    let col_width: usize = 12;
    let row_header_width: usize = 10;
    let visible_cols =
        ((area.width as usize).saturating_sub(row_header_width + 2) / col_width).clamp(1, 20);

    let start_row = state.scroll.row;
    let start_col = state.scroll.col;
    let total_rows = state.get_view_rows(var);
    let total_cols = state.get_view_cols(var);

    let row_dim = state.slicing.display_dims.0;
    let col_dim = state.slicing.display_dims.1;
    let has_row_coords = var.ndim() > 1 && var.get_coordinate(row_dim).is_some();
    let has_col_coords = var.ndim() > 1 && var.get_coordinate(col_dim).is_some();

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

    let end_row = (start_row + visible_rows).min(total_rows);
    let end_col = (start_col + visible_cols).min(total_cols);

    let mut rows = Vec::new();
    for row_idx in start_row..end_row {
        let mut cells = Vec::new();

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
            cells
                .push(Cell::from(format_cell_value(value)).style(Style::default().fg(colors.aqua)));
        }

        rows.push(Row::new(cells));
    }

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

    let mut widths = vec![Constraint::Length(row_header_width as u16)];
    for _ in 0..visible_cols {
        widths.push(Constraint::Length(col_width as u16));
    }

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

fn format_cell_value(val: f64) -> String {
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
