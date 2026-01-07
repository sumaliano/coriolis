//! Data viewer feature - data visualization and exploration.
//!
//! This module contains all data viewer functionality including state management,
//! business logic for navigation and slicing, and view mode handling (table, plot, heatmap).

pub mod ui;

use crate::data::LoadedVariable;

/// View mode for the data viewer.
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

    /// Get display name.
    pub fn name(self) -> &'static str {
        match self {
            ViewMode::Table => "Table",
            ViewMode::Plot1D => "1D Plot",
            ViewMode::Heatmap => "Heatmap",
        }
    }
}

/// Color palette for heatmap visualization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorPalette {
    /// Viridis colormap (perceptually uniform, colorblind-friendly).
    #[default]
    Viridis,
    /// Plasma colormap (perceptually uniform).
    Plasma,
    /// Rainbow/Spectral colormap (traditional, high contrast).
    Rainbow,
    /// Blue-White-Red diverging colormap.
    BlueRed,
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
}

/// Scroll position for table view.
#[derive(Debug, Clone, Copy, Default)]
pub struct ScrollPosition {
    /// Current row offset.
    pub row: usize,
    /// Current column offset.
    pub col: usize,
}

/// Slicing state for multi-dimensional data.
#[derive(Debug, Clone, Default)]
pub struct SlicingState {
    /// Selected dimension indices for slicing (for 3D+ data).
    pub slice_indices: Vec<usize>,
    /// Which dimensions to display (for 2D views).
    pub display_dims: (usize, usize),
    /// Active dimension selector (for 3D+ data).
    pub active_dim_selector: Option<usize>,
}

impl SlicingState {
    /// Create a new slicing state for a variable.
    pub fn new(ndim: usize, view_mode: ViewMode) -> Self {
        let mut state = Self {
            slice_indices: vec![0; ndim],
            display_dims: if ndim >= 2 {
                (ndim - 2, ndim - 1)
            } else {
                (0, 0)
            },
            active_dim_selector: None,
        };
        state.update_active_selector(ndim, view_mode);
        state
    }

    /// Find the first valid slice dimension.
    fn first_slice_dim(&self, ndim: usize, is_1d: bool) -> Option<usize> {
        (0..ndim).find(|&i| {
            if is_1d {
                i != self.display_dims.0
            } else {
                i != self.display_dims.0 && i != self.display_dims.1
            }
        })
    }

    /// Update active selector to be valid, or set to first available slice dimension.
    pub fn update_active_selector(&mut self, ndim: usize, view_mode: ViewMode) {
        let is_1d = matches!(view_mode, ViewMode::Plot1D);

        // Check if current active selector is still valid
        if let Some(active) = self.active_dim_selector {
            let is_valid = if is_1d {
                active < ndim && active != self.display_dims.0
            } else {
                active < ndim && active != self.display_dims.0 && active != self.display_dims.1
            };

            if is_valid {
                return; // Keep current selector
            }
        }

        // Set to first available slice dimension
        self.active_dim_selector = self.first_slice_dim(ndim, is_1d);
    }
}

/// State for the data viewer.
#[derive(Debug, Clone)]
pub struct DataViewerState {
    /// Currently loaded variable data.
    pub variable: Option<LoadedVariable>,
    /// Current view mode.
    pub view_mode: ViewMode,
    /// Color palette for heatmap.
    pub color_palette: ColorPalette,
    /// Scroll offset for table view.
    pub scroll: ScrollPosition,
    /// Slicing state for multi-dimensional data.
    pub slicing: SlicingState,
    /// Is the data viewer visible.
    pub visible: bool,
    /// Error message if loading failed.
    pub error: Option<String>,
    /// Status message to display inside viewer.
    pub status_message: Option<String>,
    /// 1D plot cursor index (for probe/readout).
    pub plot_cursor: usize,
    /// 2D heatmap crosshair row.
    pub heat_cursor_row: usize,
    /// 2D heatmap crosshair column.
    pub heat_cursor_col: usize,
    /// Whether to apply scale/offset (CF convention). True = scaled, False = raw.
    pub apply_scale_offset: bool,
}

impl Default for DataViewerState {
    fn default() -> Self {
        Self::new()
    }
}

impl DataViewerState {
    /// Create a new data viewer state.
    pub fn new() -> Self {
        Self {
            variable: None,
            view_mode: ViewMode::Table,
            color_palette: ColorPalette::default(),
            scroll: ScrollPosition::default(),
            slicing: SlicingState::default(),
            visible: false,
            error: None,
            status_message: None,
            plot_cursor: 0,
            heat_cursor_row: 0,
            heat_cursor_col: 0,
            apply_scale_offset: true,
        }
    }

    /// Load a variable for display.
    pub fn load_variable(&mut self, var: LoadedVariable) {
        let ndim = var.ndim();

        // Initialize slicing state with active selector already set
        self.slicing = SlicingState::new(ndim, self.view_mode);

        // Default to scaled data display
        self.apply_scale_offset = true;

        self.variable = Some(var);
        self.scroll = ScrollPosition::default();
        self.error = None;
        self.visible = true;
        self.status_message = None;
        self.plot_cursor = 0;
        self.heat_cursor_row = 0;
        self.heat_cursor_col = 0;
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

    /// Cycle view mode.
    pub fn cycle_view_mode(&mut self) {
        self.view_mode = self.view_mode.next();
        // Update active selector for the new view mode (preserves slice positions)
        if let Some(ref var) = self.variable {
            self.slicing
                .update_active_selector(var.ndim(), self.view_mode);
        }
    }

    /// Cycle to next color palette.
    pub fn cycle_color_palette(&mut self) {
        self.color_palette = self.color_palette.next();
    }

    /// Check if scale/offset is available for this variable.
    pub fn has_scale_offset(&self) -> bool {
        self.variable
            .as_ref()
            .map(|v| v.has_scale_offset())
            .unwrap_or(false)
    }

    /// Get scale factor from variable.
    pub fn scale_factor(&self) -> f64 {
        self.variable
            .as_ref()
            .map(|v| v.scale_factor)
            .unwrap_or(1.0)
    }

    /// Get add offset from variable.
    pub fn add_offset(&self) -> f64 {
        self.variable.as_ref().map(|v| v.add_offset).unwrap_or(0.0)
    }

    /// Toggle between scaled and raw data display.
    pub fn toggle_scale_offset(&mut self) {
        if self.has_scale_offset() {
            self.apply_scale_offset = !self.apply_scale_offset;
        }
    }

    /// Move 1D plot cursor left.
    pub fn plot_cursor_left(&mut self) {
        if let Some(ref var) = self.variable {
            if var.ndim() == 0 {
                return; // Scalar - no cursor
            }
            let len = if var.ndim() == 1 {
                var.shape[0]
            } else {
                var.shape[self.slicing.display_dims.0]
            };
            if len == 0 {
                return;
            }
            self.plot_cursor = self.plot_cursor.saturating_sub(1);
            if self.plot_cursor >= len {
                self.plot_cursor = len - 1;
            }
        }
    }

    /// Move 1D plot cursor right.
    pub fn plot_cursor_right(&mut self) {
        if let Some(ref var) = self.variable {
            if var.ndim() == 0 {
                return; // Scalar - no cursor
            }
            let len = if var.ndim() == 1 {
                var.shape[0]
            } else {
                var.shape[self.slicing.display_dims.0]
            };
            if len == 0 {
                return;
            }
            self.plot_cursor = (self.plot_cursor + 1).min(len - 1);
        }
    }

    /// Move heatmap crosshair by delta.
    pub fn move_heat_cursor(&mut self, drow: isize, dcol: isize) {
        if let Some(ref var) = self.variable {
            if var.ndim() < 2 {
                return;
            }
            let rows = var.shape[self.slicing.display_dims.0];
            let cols = var.shape[self.slicing.display_dims.1];
            let nr = rows as isize;
            let nc = cols as isize;
            let mut r = self.heat_cursor_row as isize + drow;
            let mut c = self.heat_cursor_col as isize + dcol;
            if r < 0 {
                r = 0;
            }
            if c < 0 {
                c = 0;
            }
            if r >= nr {
                r = nr - 1;
            }
            if c >= nc {
                c = nc - 1;
            }
            self.heat_cursor_row = r as usize;
            self.heat_cursor_col = c as usize;
        }
    }

    /// Copy visible data to clipboard as TSV depending on current view.
    pub fn copy_visible_to_clipboard(&self) {
        if let Some(ref var) = self.variable {
            let mut cb = match arboard::Clipboard::new() {
                Ok(c) => c,
                Err(_) => return,
            };
            let apply_scale = self.apply_scale_offset;
            match self.view_mode {
                ViewMode::Plot1D => {
                    let dim = if var.ndim() <= 1 {
                        0
                    } else {
                        self.slicing.display_dims.0
                    };
                    let data: Vec<f64> = if var.ndim() <= 1 {
                        var.data
                            .iter()
                            .map(|&v| if apply_scale { var.scale_value(v) } else { v })
                            .collect()
                    } else {
                        var.get_1d_slice(dim, &self.slicing.slice_indices, apply_scale)
                    };
                    let mut out = String::with_capacity(data.len() * 12);
                    for (i, v) in data.iter().enumerate() {
                        out.push_str(&format!("{}\t{}\n", i, v));
                    }
                    let _ = cb.set_text(out);
                },
                ViewMode::Heatmap | ViewMode::Table => {
                    let data = var.get_2d_slice(
                        self.slicing.display_dims.0,
                        self.slicing.display_dims.1,
                        &self.slicing.slice_indices,
                        apply_scale,
                    );
                    let mut out = String::new();
                    for row in &data {
                        for (ci, v) in row.iter().enumerate() {
                            if ci > 0 {
                                out.push('\t');
                            }
                            out.push_str(&format!("{}", v));
                        }
                        out.push('\n');
                    }
                    let _ = cb.set_text(out);
                },
            }
        }
    }

    /// Scroll up.
    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll.row = self.scroll.row.saturating_sub(amount);
    }

    /// Scroll down.
    pub fn scroll_down(&mut self, amount: usize) {
        if let Some(ref var) = self.variable {
            let max_row = self.get_view_rows(var).saturating_sub(1);
            self.scroll.row = (self.scroll.row + amount).min(max_row);
        }
    }

    /// Scroll left.
    pub fn scroll_left(&mut self, amount: usize) {
        self.scroll.col = self.scroll.col.saturating_sub(amount);
    }

    /// Scroll right.
    pub fn scroll_right(&mut self, amount: usize) {
        if let Some(ref var) = self.variable {
            let max_col = self.get_view_cols(var).saturating_sub(1);
            self.scroll.col = (self.scroll.col + amount).min(max_col);
        }
    }

    /// Navigate to next slice index for a dimension.
    pub fn next_slice(&mut self, dim: usize) {
        if let Some(ref var) = self.variable {
            let is_1d = matches!(self.view_mode, ViewMode::Plot1D);

            // Check if this dimension is currently being displayed
            let is_display_dim = if is_1d {
                dim == self.slicing.display_dims.0
            } else {
                dim == self.slicing.display_dims.0 || dim == self.slicing.display_dims.1
            };

            if dim < var.ndim() && !is_display_dim {
                let max = var.shape[dim].saturating_sub(1);
                self.slicing.slice_indices[dim] = (self.slicing.slice_indices[dim] + 1).min(max);
            }
        }
    }

    /// Navigate to previous slice index for a dimension.
    pub fn prev_slice(&mut self, dim: usize) {
        if dim < self.slicing.slice_indices.len() {
            self.slicing.slice_indices[dim] = self.slicing.slice_indices[dim].saturating_sub(1);
        }
    }

    /// Select next dimension selector.
    pub fn next_dim_selector(&mut self) {
        if let Some(ref var) = self.variable {
            let ndim = var.ndim();
            let is_1d = matches!(self.view_mode, ViewMode::Plot1D);

            // For 1D: need at least 2 dims (1 to display, 1+ to slice)
            // For 2D: need at least 3 dims (2 to display, 1+ to slice)
            let min_dims = if is_1d { 2 } else { 3 };

            if ndim >= min_dims {
                // Collect all non-display dimensions
                let slice_dims: Vec<usize> = (0..ndim)
                    .filter(|&i| {
                        if is_1d {
                            i != self.slicing.display_dims.0
                        } else {
                            i != self.slicing.display_dims.0 && i != self.slicing.display_dims.1
                        }
                    })
                    .collect();

                if slice_dims.is_empty() {
                    return;
                }

                match self.slicing.active_dim_selector {
                    None => {
                        // Select first slice dimension
                        self.slicing.active_dim_selector = Some(slice_dims[0]);
                    },
                    Some(current) => {
                        // Find current in list and select next (wrap around)
                        if let Some(pos) = slice_dims.iter().position(|&d| d == current) {
                            let next_pos = (pos + 1) % slice_dims.len();
                            self.slicing.active_dim_selector = Some(slice_dims[next_pos]);
                        } else {
                            // Current not in list (shouldn't happen), select first
                            self.slicing.active_dim_selector = Some(slice_dims[0]);
                        }
                    },
                }
            }
        }
    }

    /// Increment value for active dimension selector.
    pub fn increment_active_slice(&mut self) {
        if let Some(dim) = self.slicing.active_dim_selector {
            self.next_slice(dim);
        }
    }

    /// Decrement value for active dimension selector.
    pub fn decrement_active_slice(&mut self) {
        if let Some(dim) = self.slicing.active_dim_selector {
            self.prev_slice(dim);
        }
    }

    /// Rotate display dimensions forward (swap Y and X axes).
    pub fn rotate_display_dims(&mut self) {
        if let Some(ref var) = self.variable {
            let ndim = var.ndim();
            if ndim >= 2 {
                std::mem::swap(
                    &mut self.slicing.display_dims.0,
                    &mut self.slicing.display_dims.1,
                );
                // Also swap cursor position to maintain the same logical position
                std::mem::swap(&mut self.heat_cursor_row, &mut self.heat_cursor_col);
                // Clamp cursor to new dimensions
                self.clamp_heat_cursor();
                // Update active selector after rotation
                self.slicing.update_active_selector(ndim, self.view_mode);
            }
        }
    }

    /// Cycle through available dimensions for display.
    pub fn cycle_display_dim(&mut self, which: usize) {
        if let Some(ref var) = self.variable {
            let ndim = var.ndim();
            if ndim == 0 {
                return;
            }

            let is_1d = matches!(self.view_mode, ViewMode::Plot1D);

            let current = if which == 0 {
                self.slicing.display_dims.0
            } else {
                self.slicing.display_dims.1
            };

            let mut next = (current + 1) % ndim;

            // For 2D views, make sure we don't overlap with the other dimension
            if !is_1d {
                let other = if which == 0 {
                    self.slicing.display_dims.1
                } else {
                    self.slicing.display_dims.0
                };

                if next == other {
                    next = (next + 1) % ndim;
                }
            }

            if which == 0 {
                self.slicing.display_dims.0 = next;
            } else {
                self.slicing.display_dims.1 = next;
            }

            // Clamp cursor to new dimensions
            self.clamp_heat_cursor();

            // Update active selector after changing display dimension
            self.slicing.update_active_selector(ndim, self.view_mode);
        }
    }

    /// Set status message (displayed inside overlay).
    pub fn set_status(&mut self, message: String) {
        self.status_message = Some(message);
    }

    /// Clear status message.
    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    /// Get number of rows for current view (helper).
    fn get_view_rows(&self, var: &LoadedVariable) -> usize {
        if var.ndim() == 0 {
            1
        } else if var.ndim() == 1 {
            var.shape[0]
        } else {
            var.shape[self.slicing.display_dims.0]
        }
    }

    /// Get number of columns for current view (helper).
    fn get_view_cols(&self, var: &LoadedVariable) -> usize {
        if var.ndim() <= 1 {
            1
        } else {
            var.shape[self.slicing.display_dims.1]
        }
    }

    /// Clamp heatmap cursor to current dimensions.
    fn clamp_heat_cursor(&mut self) {
        if let Some(ref var) = self.variable {
            if var.ndim() >= 2 {
                let rows = var.shape[self.slicing.display_dims.0];
                let cols = var.shape[self.slicing.display_dims.1];

                if rows > 0 {
                    self.heat_cursor_row = self.heat_cursor_row.min(rows - 1);
                } else {
                    self.heat_cursor_row = 0;
                }

                if cols > 0 {
                    self.heat_cursor_col = self.heat_cursor_col.min(cols - 1);
                } else {
                    self.heat_cursor_col = 0;
                }
            }
        }
    }
}
