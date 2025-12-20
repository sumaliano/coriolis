//! Data viewer overlay for visualizing variable contents.

use super::ThemeColors;
use crate::data::LoadedVariable;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, Clear, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table, Wrap,
    },
    Frame,
};

/// View mode for the data overlay.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewMode {
    /// Table view showing raw data values.
    #[default]
    Table,
    /// 1D line plot.
    Plot1D,
    /// 2D heatmap visualization.
    Heatmap,
}

impl ViewMode {
    /// Get the next view mode in cycle.
    pub fn next(self) -> Self {
        match self {
            ViewMode::Table => ViewMode::Plot1D,
            ViewMode::Plot1D => ViewMode::Heatmap,
            ViewMode::Heatmap => ViewMode::Table,
        }
    }

    /// Get the previous view mode in cycle.
    #[allow(dead_code)]
    pub fn prev(self) -> Self {
        match self {
            ViewMode::Table => ViewMode::Heatmap,
            ViewMode::Plot1D => ViewMode::Table,
            ViewMode::Heatmap => ViewMode::Plot1D,
        }
    }

    /// Get display name.
    pub fn name(self) -> &'static str {
        match self {
            ViewMode::Table => "Table",
            ViewMode::Plot1D => "1D Plot",
            ViewMode::Heatmap => "Heatmap",
        }
    }
}

/// State for the data overlay.
#[derive(Debug, Clone)]
pub struct OverlayState {
    /// Currently loaded variable data.
    pub variable: Option<LoadedVariable>,
    /// Current view mode.
    pub view_mode: ViewMode,
    /// Scroll offset for table view (row, col).
    pub table_scroll: (usize, usize),
    /// Selected dimension indices for slicing (for 3D+ data).
    pub slice_indices: Vec<usize>,
    /// Which dimensions to display (for 2D views).
    pub display_dims: (usize, usize),
    /// Active dimension selector (for 3D+ data).
    pub active_dim_selector: Option<usize>,
    /// Is the overlay visible.
    pub visible: bool,
    /// Error message if loading failed.
    pub error: Option<String>,
}

impl Default for OverlayState {
    fn default() -> Self {
        Self::new()
    }
}

impl OverlayState {
    /// Create a new overlay state.
    pub fn new() -> Self {
        Self {
            variable: None,
            view_mode: ViewMode::Table,
            table_scroll: (0, 0),
            slice_indices: Vec::new(),
            display_dims: (0, 1),
            active_dim_selector: None,
            visible: false,
            error: None,
        }
    }

    /// Load a variable for display.
    pub fn load_variable(&mut self, var: LoadedVariable) {
        let ndim = var.ndim();
        // Initialize slice indices to 0 for all dimensions
        self.slice_indices = vec![0; ndim];
        // Set display dimensions
        self.display_dims = if ndim >= 2 {
            (ndim - 2, ndim - 1)
        } else {
            (0, 0)
        };
        self.variable = Some(var);
        self.table_scroll = (0, 0);
        self.error = None;
        self.visible = true;
    }

    /// Set error state.
    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
        self.variable = None;
        self.visible = true;
    }

    /// Close the overlay.
    pub fn close(&mut self) {
        self.visible = false;
        self.variable = None;
        self.error = None;
    }

    /// Toggle view mode.
    pub fn cycle_view_mode(&mut self) {
        self.view_mode = self.view_mode.next();
    }

    /// Scroll table up.
    pub fn scroll_up(&mut self, amount: usize) {
        self.table_scroll.0 = self.table_scroll.0.saturating_sub(amount);
    }

    /// Scroll table down.
    pub fn scroll_down(&mut self, amount: usize) {
        if let Some(ref var) = self.variable {
            let max_row = self.get_view_rows(var).saturating_sub(1);
            self.table_scroll.0 = (self.table_scroll.0 + amount).min(max_row);
        }
    }

    /// Scroll table left.
    pub fn scroll_left(&mut self, amount: usize) {
        self.table_scroll.1 = self.table_scroll.1.saturating_sub(amount);
    }

    /// Scroll table right.
    pub fn scroll_right(&mut self, amount: usize) {
        if let Some(ref var) = self.variable {
            let max_col = self.get_view_cols(var).saturating_sub(1);
            self.table_scroll.1 = (self.table_scroll.1 + amount).min(max_col);
        }
    }

    /// Get number of rows for current view.
    fn get_view_rows(&self, var: &LoadedVariable) -> usize {
        if var.ndim() == 0 {
            1
        } else if var.ndim() == 1 {
            var.shape[0]
        } else {
            var.shape[self.display_dims.0]
        }
    }

    /// Get number of columns for current view.
    fn get_view_cols(&self, var: &LoadedVariable) -> usize {
        if var.ndim() <= 1 {
            1
        } else {
            var.shape[self.display_dims.1]
        }
    }

    /// Navigate to next slice index for a dimension.
    pub fn next_slice(&mut self, dim: usize) {
        if let Some(ref var) = self.variable {
            if dim < var.ndim() && dim != self.display_dims.0 && dim != self.display_dims.1 {
                let max = var.shape[dim].saturating_sub(1);
                self.slice_indices[dim] = (self.slice_indices[dim] + 1).min(max);
            }
        }
    }

    /// Navigate to previous slice index for a dimension.
    pub fn prev_slice(&mut self, dim: usize) {
        if dim < self.slice_indices.len() {
            self.slice_indices[dim] = self.slice_indices[dim].saturating_sub(1);
        }
    }

    /// Select next dimension selector.
    pub fn next_dim_selector(&mut self) {
        if let Some(ref var) = self.variable {
            let ndim = var.ndim();
            if ndim > 2 {
                match self.active_dim_selector {
                    None => {
                        // Find first non-display dimension
                        for i in 0..ndim {
                            if i != self.display_dims.0 && i != self.display_dims.1 {
                                self.active_dim_selector = Some(i);
                                break;
                            }
                        }
                    }
                    Some(current) => {
                        // Find next non-display dimension
                        let mut found_current = false;
                        let mut next = None;
                        for i in 0..ndim {
                            if i == current {
                                found_current = true;
                            } else if found_current
                                && i != self.display_dims.0
                                && i != self.display_dims.1
                            {
                                next = Some(i);
                                break;
                            }
                        }
                        self.active_dim_selector = next;
                    }
                }
            }
        }
    }

    /// Increment value for active dimension selector.
    pub fn increment_active_slice(&mut self) {
        if let Some(dim) = self.active_dim_selector {
            self.next_slice(dim);
        }
    }

    /// Decrement value for active dimension selector.
    pub fn decrement_active_slice(&mut self) {
        if let Some(dim) = self.active_dim_selector {
            self.prev_slice(dim);
        }
    }
}

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
                Constraint::Length(3), // Dimension selectors
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
            draw_footer(f, chunks[3], colors);
        } else {
            draw_footer(f, chunks[2], colors);
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
    if let Some((min, max)) = var.data.min_max() {
        stats.push(format!("Min: {:.6}", min));
        stats.push(format!("Max: {:.6}", max));
    }
    if let Some(mean) = var.data.mean() {
        stats.push(format!("Mean: {:.6}", mean));
    }
    if let Some(std) = var.data.std() {
        stats.push(format!("Std: {:.6}", std));
    }
    stats.push(format!("Valid: {}", var.data.valid_count()));

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
    let data = var.data.to_f64();

    // Determine visible area
    let visible_rows = (area.height as usize).saturating_sub(2);
    let col_width = 12;
    let visible_cols = (area.width as usize / col_width).max(1);

    let (start_row, start_col) = state.table_scroll;
    let total_rows = state.get_view_rows(var);
    let total_cols = state.get_view_cols(var);

    // Build table rows
    let mut rows = Vec::new();

    for row_idx in start_row..((start_row + visible_rows).min(total_rows)) {
        let mut cells = Vec::new();
        // Row header
        cells.push(Cell::from(format!("{:>4}", row_idx)).style(Style::default().fg(colors.green)));

        for col_idx in start_col..((start_col + visible_cols).min(total_cols)) {
            let value = if var.ndim() == 0 {
                data.first().copied().unwrap_or(f64::NAN)
            } else if var.ndim() == 1 {
                data.get(row_idx).copied().unwrap_or(f64::NAN)
            } else {
                // Build indices
                let mut indices = state.slice_indices.clone();
                indices[state.display_dims.0] = row_idx;
                indices[state.display_dims.1] = col_idx;
                var.get_value(&indices).unwrap_or(f64::NAN)
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
        .block(Block::default())
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
    // Get 1D slice of data
    let slice_dim = if var.ndim() > 1 {
        state.display_dims.1
    } else {
        0
    };

    let data = if var.ndim() <= 1 {
        var.data.to_f64()
    } else {
        var.get_1d_slice(slice_dim, &state.slice_indices)
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
        .filter(|v| v.is_finite())
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(min, max), &v| {
            (min.min(v), max.max(v))
        });

    let range = if (max_val - min_val).abs() < 1e-10 {
        1.0
    } else {
        max_val - min_val
    };

    // Build ASCII plot
    let plot_height = area.height.saturating_sub(2) as usize;
    let plot_width = area.width.saturating_sub(2) as usize;

    if plot_height == 0 || plot_width == 0 {
        return;
    }

    // Sample data to fit width
    let step = (data.len() as f64 / plot_width as f64).max(1.0);

    let mut lines = Vec::new();

    // Y-axis label
    lines.push(Line::from(format!("{:.2e}", max_val)));

    // Plot area
    for y in 0..plot_height {
        let threshold = max_val - (y as f64 / plot_height as f64) * range;
        let mut chars = String::new();

        for x in 0..plot_width {
            let idx = ((x as f64) * step) as usize;
            if idx < data.len() {
                let val = data[idx];
                if val.is_finite() && val >= threshold {
                    chars.push('\u{2588}'); // Full block
                } else if val.is_finite() && val >= threshold - range / (2.0 * plot_height as f64) {
                    chars.push('\u{2584}'); // Lower half block
                } else {
                    chars.push(' ');
                }
            } else {
                chars.push(' ');
            }
        }

        lines.push(Line::from(Span::styled(chars, Style::default().fg(colors.aqua))));
    }

    lines.push(Line::from(format!("{:.2e}", min_val)));
    lines.push(Line::from(format!(
        "0{:>width$}{}",
        "",
        data.len() - 1,
        width = plot_width.saturating_sub(10)
    )));

    let paragraph = Paragraph::new(lines)
        .style(Style::default().fg(colors.fg0))
        .block(Block::default());

    f.render_widget(paragraph, area);
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
    let data_2d = var.get_2d_slice(state.display_dims.0, state.display_dims.1, &state.slice_indices);

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

    let heatmap_height = area.height.saturating_sub(3) as usize;
    let heatmap_width = area.width.saturating_sub(2) as usize;

    if heatmap_height == 0 || heatmap_width == 0 {
        return;
    }

    let rows = data_2d.len();
    let cols = data_2d[0].len();

    // Unicode block characters for heatmap: ░▒▓█
    let heat_chars = [' ', '\u{2591}', '\u{2592}', '\u{2593}', '\u{2588}'];

    let mut lines = Vec::new();

    // Add colorbar legend
    lines.push(Line::from(vec![
        Span::raw("Low "),
        Span::styled("\u{2591}\u{2592}\u{2593}\u{2588}", Style::default().fg(colors.aqua)),
        Span::raw(" High | "),
        Span::styled(
            format!("[{:.2e}, {:.2e}]", min_val, max_val),
            Style::default().fg(colors.green),
        ),
    ]));

    // Sample and render
    let row_step = (rows as f64 / heatmap_height as f64).max(1.0);
    let col_step = (cols as f64 / heatmap_width as f64).max(1.0);

    for y in 0..heatmap_height {
        let row_idx = ((y as f64) * row_step) as usize;
        if row_idx >= rows {
            break;
        }

        let mut chars = String::new();
        for x in 0..heatmap_width {
            let col_idx = ((x as f64) * col_step) as usize;
            if col_idx >= cols {
                break;
            }

            let val = data_2d[row_idx][col_idx];
            if val.is_finite() {
                let normalized = ((val - min_val) / range).clamp(0.0, 1.0);
                let idx = (normalized * 4.0).floor() as usize;
                chars.push(heat_chars[idx.min(4)]);
            } else {
                chars.push('?');
            }
        }

        lines.push(Line::from(Span::styled(chars, Style::default().fg(colors.aqua))));
    }

    // Add axis labels
    let dim1_name = var.dim_names.get(state.display_dims.0).map(|s| s.as_str()).unwrap_or("dim0");
    let dim2_name = var.dim_names.get(state.display_dims.1).map(|s| s.as_str()).unwrap_or("dim1");
    lines.push(Line::from(format!("Y: {} | X: {}", dim1_name, dim2_name)));

    let paragraph = Paragraph::new(lines)
        .style(Style::default().fg(colors.fg0))
        .block(Block::default());

    f.render_widget(paragraph, area);
}

fn draw_dimension_selectors(
    f: &mut Frame<'_>,
    area: Rect,
    state: &OverlayState,
    var: &LoadedVariable,
    colors: &ThemeColors,
) {
    let mut spans = Vec::new();
    spans.push(Span::styled("Slices: ", Style::default().fg(colors.green)));

    for (i, (dim_name, &size)) in var.dim_names.iter().zip(var.shape.iter()).enumerate() {
        // Skip display dimensions
        if i == state.display_dims.0 || i == state.display_dims.1 {
            continue;
        }

        let is_active = state.active_dim_selector == Some(i);
        let idx = state.slice_indices.get(i).copied().unwrap_or(0);

        let style = if is_active {
            Style::default()
                .fg(colors.bg0)
                .bg(colors.yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(colors.aqua)
        };

        spans.push(Span::styled(
            format!(" {}[{}/{}] ", dim_name, idx, size - 1),
            style,
        ));
    }

    let paragraph = Paragraph::new(Line::from(spans))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(colors.bg2)),
        );

    f.render_widget(paragraph, area);
}

fn draw_footer(f: &mut Frame<'_>, area: Rect, colors: &ThemeColors) {
    let help = "Tab: View | hjkl/Arrows: Pan | [/]: Dim | +/-: Slice | Esc: Close";
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
