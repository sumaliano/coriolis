//! Layout configuration constants for the data viewer overlay.

/// Configuration for table view layout.
#[derive(Debug, Clone)]
pub struct TableLayoutConfig {
    /// Padding to account for borders and headers.
    pub border_padding: usize,
    /// Width of each column in characters.
    pub column_width: u16,
    /// Maximum number of visible columns.
    pub max_visible_columns: usize,
    /// Width of the row header column.
    pub row_header_width: u16,
}

impl Default for TableLayoutConfig {
    fn default() -> Self {
        Self {
            border_padding: 4,
            column_width: 12,
            max_visible_columns: 20,
            row_header_width: 5,
        }
    }
}

/// Configuration for 1D plot view layout.
#[derive(Debug, Clone)]
pub struct PlotLayoutConfig {
    /// Padding factor for Y-axis (0.3 = 30% margin).
    pub y_axis_padding_factor: f64,
}

impl Default for PlotLayoutConfig {
    fn default() -> Self {
        Self {
            y_axis_padding_factor: 0.3, // 30% margin for visual clearance
        }
    }
}

/// Configuration for heatmap view layout.
#[derive(Debug, Clone)]
pub struct HeatmapLayoutConfig {
    /// Terminal characters per pixel horizontally (for aspect ratio correction).
    pub pixel_width: usize,
    /// Height reserved for colorbar.
    pub colorbar_height: u16,
    /// Width of colorbar in characters.
    pub colorbar_width: usize,
}

impl Default for HeatmapLayoutConfig {
    fn default() -> Self {
        Self {
            pixel_width: 2,      // 2:1 aspect ratio correction
            colorbar_height: 1,
            colorbar_width: 50,
        }
    }
}

/// Combined layout configuration for all view modes.
#[derive(Debug, Clone, Default)]
pub struct LayoutConfig {
    /// Configuration for table view.
    pub table: TableLayoutConfig,
    /// Configuration for 1D plot view.
    pub plot: PlotLayoutConfig,
    /// Configuration for heatmap view.
    pub heatmap: HeatmapLayoutConfig,
}
