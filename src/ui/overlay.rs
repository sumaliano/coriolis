//! Data viewer overlay for visualizing variable contents.

use super::ThemeColors;
use crate::data::LoadedVariable;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Axis, Block, Borders, Cell, Chart, Clear, Dataset, GraphType, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Table, Wrap,
    },
    Frame,
};

/// Color palette for heatmap visualization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorPalette {
    /// Viridis colormap (perceptually uniform, colorblind-friendly).
    Viridis,
    /// Plasma colormap (perceptually uniform).
    Plasma,
    /// Rainbow/Spectral colormap (traditional, high contrast).
    Rainbow,
    /// Blue-White-Red diverging colormap.
    BlueRed,
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self::Viridis
    }
}

impl ColorPalette {
    /// Get the next palette in cycle.
    pub fn next(self) -> Self {
        match self {
            Self::Viridis => Self::Plasma,
            Self::Plasma => Self::Rainbow,
            Self::Rainbow => Self::BlueRed,
            Self::BlueRed => Self::Viridis,
        }
    }

    /// Get palette name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Viridis => "Viridis",
            Self::Plasma => "Plasma",
            Self::Rainbow => "Rainbow",
            Self::BlueRed => "Blue-Red",
        }
    }

    /// Map a normalized value (0.0 to 1.0) to an RGB color.
    pub fn color(self, t: f64) -> Color {
        let t = t.clamp(0.0, 1.0);

        match self {
            Self::Viridis => viridis_color(t),
            Self::Plasma => plasma_color(t),
            Self::Rainbow => rainbow_color(t),
            Self::BlueRed => bluered_color(t),
        }
    }
}

/// Viridis colormap approximation.
fn viridis_color(t: f64) -> Color {
    // Simplified viridis palette using piecewise linear interpolation
    let r = if t < 0.5 {
        68.0 + t * 2.0 * (33.0 - 68.0)
    } else {
        33.0 + (t - 0.5) * 2.0 * (253.0 - 33.0)
    };

    let g = if t < 0.5 {
        1.0 + t * 2.0 * (104.0 - 1.0)
    } else {
        104.0 + (t - 0.5) * 2.0 * (231.0 - 104.0)
    };

    let b = if t < 0.5 {
        84.0 + t * 2.0 * (109.0 - 84.0)
    } else {
        109.0 + (t - 0.5) * 2.0 * (37.0 - 109.0)
    };

    Color::Rgb(r as u8, g as u8, b as u8)
}

/// Plasma colormap approximation.
fn plasma_color(t: f64) -> Color {
    let r = if t < 0.5 {
        13.0 + t * 2.0 * (180.0 - 13.0)
    } else {
        180.0 + (t - 0.5) * 2.0 * (240.0 - 180.0)
    };

    let g = if t < 0.5 {
        8.0 + t * 2.0 * (54.0 - 8.0)
    } else {
        54.0 + (t - 0.5) * 2.0 * (175.0 - 54.0)
    };

    let b = if t < 0.5 {
        135.0 + t * 2.0 * (121.0 - 135.0)
    } else {
        121.0 + (t - 0.5) * 2.0 * (12.0 - 121.0)
    };

    Color::Rgb(r as u8, g as u8, b as u8)
}

/// Rainbow/Spectral colormap.
fn rainbow_color(t: f64) -> Color {
    // HSV to RGB conversion with H varying from 240° (blue) to 0° (red)
    let h = (1.0 - t) * 240.0;
    let s = 1.0;
    let v = 1.0;

    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    Color::Rgb(
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}

/// Blue-White-Red diverging colormap.
fn bluered_color(t: f64) -> Color {
    if t < 0.5 {
        // Blue to white
        let t2 = t * 2.0;
        let r = (t2 * 255.0) as u8;
        let g = (t2 * 255.0) as u8;
        let b = 255;
        Color::Rgb(r, g, b)
    } else {
        // White to red
        let t2 = (t - 0.5) * 2.0;
        let r = 255;
        let g = ((1.0 - t2) * 255.0) as u8;
        let b = ((1.0 - t2) * 255.0) as u8;
        Color::Rgb(r, g, b)
    }
}

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
    /// Color palette for heatmap.
    pub color_palette: ColorPalette,
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
            color_palette: ColorPalette::default(),
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

        // For 3D+ data, automatically select the first non-display dimension
        self.active_dim_selector = if ndim > 2 {
            // Find first dimension that's not being displayed
            (0..ndim)
                .find(|&i| i != self.display_dims.0 && i != self.display_dims.1)
        } else {
            None
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

    /// Cycle to next color palette.
    pub fn cycle_color_palette(&mut self) {
        self.color_palette = self.color_palette.next();
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

    /// Rotate display dimensions forward (swap Y and X axes).
    pub fn rotate_display_dims(&mut self) {
        if let Some(ref var) = self.variable {
            let ndim = var.ndim();
            if ndim >= 2 {
                // Swap the two display dimensions
                let temp = self.display_dims.0;
                self.display_dims.0 = self.display_dims.1;
                self.display_dims.1 = temp;
            }
        }
    }

    /// Cycle through available dimensions for display.
    pub fn cycle_display_dim(&mut self, which: usize) {
        if let Some(ref var) = self.variable {
            let ndim = var.ndim();
            if ndim < 2 {
                return;
            }

            // Get current dimension
            let current = if which == 0 {
                self.display_dims.0
            } else {
                self.display_dims.1
            };

            // Find next available dimension (wrap around)
            let mut next = (current + 1) % ndim;

            // Skip if it would duplicate the other display dimension
            let other = if which == 0 {
                self.display_dims.1
            } else {
                self.display_dims.0
            };

            // If next equals other, skip to the one after
            if next == other {
                next = (next + 1) % ndim;
            }

            // Update display dimension
            if which == 0 {
                self.display_dims.0 = next;
            } else {
                self.display_dims.1 = next;
            }

            // Update active selector if it's now a display dimension
            if let Some(active) = self.active_dim_selector {
                if active == next {
                    // Find a new active dimension
                    self.active_dim_selector = (0..ndim)
                        .find(|&i| i != self.display_dims.0 && i != self.display_dims.1);
                }
            }
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
    // Determine visible area
    let visible_rows = (area.height as usize).saturating_sub(4); // Account for border and header
    let col_width = 12;
    let visible_cols = ((area.width as usize).saturating_sub(6) / col_width).max(1).min(20); // Limit to 20 cols

    let (start_row, start_col) = state.table_scroll;
    let total_rows = state.get_view_rows(var);
    let total_cols = state.get_view_cols(var);

    // Get data slice efficiently - avoid repeated get_value calls
    let data_slice = if var.ndim() == 0 {
        vec![vec![var.data.to_f64().first().copied().unwrap_or(f64::NAN)]]
    } else if var.ndim() == 1 {
        let data = var.data.to_f64();
        vec![data]
    } else {
        // Get 2D slice once - much faster than repeated get_value calls
        var.get_2d_slice(state.display_dims.0, state.display_dims.1, &state.slice_indices)
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
    // For 1D plot, always use the last dimension regardless of display_dims
    // This makes sense: X = index, Y = value
    let slice_dim = var.ndim().saturating_sub(1);

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

    // Add padding to avoid edge clipping - 10% margin
    let padding = (max_val - min_val).abs() * 0.3;
    let y_min = min_val - padding;
    let y_max = max_val + padding;

    // Prepare data points for Chart widget
    let chart_data: Vec<(f64, f64)> = data
        .iter()
        .enumerate()
        .filter(|(_, &v)| v.is_finite())
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
                let idx = state.slice_indices.get(i).copied().unwrap_or(0);
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
        state.display_dims.0,
        state.display_dims.1,
        &state.slice_indices,
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
        .get(state.display_dims.0)
        .map(|s| s.as_str())
        .unwrap_or("dim0");
    let dim2_name = var
        .dim_names
        .get(state.display_dims.1)
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

    // First line: Display dimensions
    let mut display_spans = vec![
        Span::styled("Display: ", Style::default().fg(colors.green)),
    ];

    let dim1_name = var.dim_names.get(state.display_dims.0).map(|s| s.as_str()).unwrap_or("?");
    let dim2_name = var.dim_names.get(state.display_dims.1).map(|s| s.as_str()).unwrap_or("?");
    let dim1_size = var.shape.get(state.display_dims.0).copied().unwrap_or(0);
    let dim2_size = var.shape.get(state.display_dims.1).copied().unwrap_or(0);

    display_spans.push(Span::styled(
        format!("Y: {}[{}] ", dim1_name, dim1_size),
        Style::default().fg(colors.aqua),
    ));
    display_spans.push(Span::styled(
        format!("X: {}[{}]", dim2_name, dim2_size),
        Style::default().fg(colors.aqua),
    ));

    lines.push(Line::from(display_spans));

    // Second line: Slice selectors (for non-display dimensions)
    let mut slice_spans = vec![
        Span::styled("Slices: ", Style::default().fg(colors.green)),
    ];

    let has_slices = var.dim_names.iter().zip(var.shape.iter()).enumerate()
        .any(|(i, _)| i != state.display_dims.0 && i != state.display_dims.1);

    if has_slices {
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

fn draw_footer(f: &mut Frame<'_>, area: Rect, colors: &ThemeColors) {
    let help = "Tab: View | C: Palette | R: Rotate | Y/X: Change Dims | Shift+Tab: Slice Dim | PgUp/PgDn: Slice | Esc: Close";
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
