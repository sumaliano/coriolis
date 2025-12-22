//! Data overlay feature - state and behavior for the data viewer overlay.
//!
//! This module contains all overlay-related functionality including state management,
//! business logic for navigation and slicing, and view mode handling.

pub mod ui;

use crate::data::LoadedVariable;

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

/// State for the data overlay.
#[derive(Debug, Clone)]
pub struct OverlayState {
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
    /// Is the overlay visible.
    pub visible: bool,
    /// Error message if loading failed.
    pub error: Option<String>,
    /// Status message to display inside overlay.
    pub status_message: Option<String>,
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
            scroll: ScrollPosition::default(),
            slicing: SlicingState::default(),
            visible: false,
            error: None,
            status_message: None,
        }
    }

    /// Load a variable for display.
    pub fn load_variable(&mut self, var: LoadedVariable) {
        let ndim = var.ndim();

        // Initialize slicing state
        self.slicing = SlicingState {
            slice_indices: vec![0; ndim],
            display_dims: if ndim >= 2 {
                (ndim - 2, ndim - 1)
            } else {
                (0, 0)
            },
            active_dim_selector: if ndim > 2 {
                (0..ndim).find(|&i| i != (if ndim >= 2 { ndim - 2 } else { 0 })
                                    && i != (if ndim >= 2 { ndim - 1 } else { 0 }))
            } else {
                None
            },
        };

        self.variable = Some(var);
        self.scroll = ScrollPosition::default();
        self.error = None;
        self.visible = true;
        self.status_message = None;
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
    }

    /// Cycle to next color palette.
    pub fn cycle_color_palette(&mut self) {
        self.color_palette = self.color_palette.next();
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
                self.slicing.slice_indices[dim] =
                    (self.slicing.slice_indices[dim] + 1).min(max);
            }
        }
    }

    /// Navigate to previous slice index for a dimension.
    pub fn prev_slice(&mut self, dim: usize) {
        if dim < self.slicing.slice_indices.len() {
            self.slicing.slice_indices[dim] =
                self.slicing.slice_indices[dim].saturating_sub(1);
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
                match self.slicing.active_dim_selector {
                    None => {
                        // Find first non-display dimension
                        for i in 0..ndim {
                            let is_display = if is_1d {
                                i == self.slicing.display_dims.0
                            } else {
                                i == self.slicing.display_dims.0 || i == self.slicing.display_dims.1
                            };

                            if !is_display {
                                self.slicing.active_dim_selector = Some(i);
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
                            } else if found_current {
                                let is_display = if is_1d {
                                    i == self.slicing.display_dims.0
                                } else {
                                    i == self.slicing.display_dims.0 || i == self.slicing.display_dims.1
                                };

                                if !is_display {
                                    next = Some(i);
                                    break;
                                }
                            }
                        }
                        self.slicing.active_dim_selector = next;
                    }
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
                let temp = self.slicing.display_dims.0;
                self.slicing.display_dims.0 = self.slicing.display_dims.1;
                self.slicing.display_dims.1 = temp;
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

            // Update active selector if it's now a display dimension
            if let Some(active) = self.slicing.active_dim_selector {
                if active == next {
                    self.slicing.active_dim_selector = (0..ndim)
                        .find(|&i| {
                            if is_1d {
                                i != self.slicing.display_dims.0
                            } else {
                                i != self.slicing.display_dims.0 && i != self.slicing.display_dims.1
                            }
                        });
                }
            }
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
}
