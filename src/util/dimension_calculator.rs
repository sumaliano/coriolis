//! Dimension calculation logic for different view modes.

use crate::data_viewer::ViewMode;

/// Calculator for determining which dimensions to display/slice based on view mode.
#[derive(Debug, Clone)]
pub struct DimensionCalculator {
    ndim: usize,
    view_mode: ViewMode,
    display_dims: (usize, usize),
}

impl DimensionCalculator {
    /// Create a new dimension calculator.
    pub fn new(ndim: usize, view_mode: ViewMode, display_dims: (usize, usize)) -> Self {
        Self {
            ndim,
            view_mode,
            display_dims,
        }
    }

    /// Get the dimension being plotted in 1D view.
    /// For 1D plot: uses display_dims.1 (X dimension from 2D view).
    /// This way 1D has one more slice dimension than 2D.
    pub fn get_plot_dimension(&self) -> usize {
        if self.ndim <= 1 {
            0
        } else {
            self.display_dims.1 // X-axis from 2D view
        }
    }

    /// Get dimensions that should be sliced (not displayed).
    pub fn get_slice_dimensions(&self) -> Vec<usize> {
        match self.view_mode {
            ViewMode::Plot1D => {
                // For 1D: slice through all dimensions except the plot dimension
                let plot_dim = self.get_plot_dimension();
                (0..self.ndim).filter(|&i| i != plot_dim).collect()
            },
            ViewMode::Table | ViewMode::Heatmap => {
                // For 2D: slice through all dimensions except the two being displayed
                (0..self.ndim)
                    .filter(|&i| i != self.display_dims.0 && i != self.display_dims.1)
                    .collect()
            },
        }
    }

    /// Get dimensions that are being displayed (not sliced).
    pub fn get_display_dimensions(&self) -> Vec<usize> {
        match self.view_mode {
            ViewMode::Plot1D => vec![self.get_plot_dimension()],
            ViewMode::Table | ViewMode::Heatmap => {
                if self.ndim >= 2 {
                    vec![self.display_dims.0, self.display_dims.1]
                } else if self.ndim == 1 {
                    vec![0]
                } else {
                    vec![]
                }
            },
        }
    }

    /// Check if a dimension is currently being displayed.
    pub fn is_display_dimension(&self, dim: usize) -> bool {
        self.get_display_dimensions().contains(&dim)
    }

    /// Check if a dimension is being sliced.
    pub fn is_slice_dimension(&self, dim: usize) -> bool {
        !self.is_display_dimension(dim)
    }
}
