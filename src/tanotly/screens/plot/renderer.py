"""Plot rendering logic for creating and sizing plot widgets."""

import numpy as np
from textual.widgets import Static

from ...config import ThemeManager
from .widgets import DataPlot1D, DataPlot2D


class PlotRenderer:
    """Handles creation and sizing of plot widgets."""

    @staticmethod
    def calculate_plot_size(data: np.ndarray) -> tuple[int, int]:
        """Calculate appropriate plot size based on data shape.

        Args:
            data: Input data array

        Returns:
            Tuple of (width, height) in characters
        """
        if data.ndim == 1:
            # 1D: fixed height, width based on data length
            width = min(max(len(data), 40), 80)
            height = 15
        elif data.ndim == 2:
            rows, cols = data.shape
            # Target: each data point = ~1 character width, ~0.5 character height
            # (terminal characters are ~2x taller than wide)
            max_width = 80
            max_height = 30

            # Scale to fit while maintaining aspect ratio
            width = min(cols, max_width)
            height = min(rows // 2 + 1, max_height)  # Divide by 2 for terminal aspect

            # Ensure minimum size
            width = max(width, 20)
            height = max(height, 10)
        else:
            width, height = 60, 20

        return width, height

    @staticmethod
    def create_plot_widget(data: np.ndarray, widget_id: str = "plot-widget"):
        """Create plot widget sized to data.

        Args:
            data: The data to plot (should be 1D or 2D after slicing)
            widget_id: CSS ID for the widget

        Returns:
            DataPlot1D, DataPlot2D, or Static widget
        """
        # Ensure we have a numpy array and make a copy to avoid issues with views
        data = np.array(data, copy=True)
        width, height = PlotRenderer.calculate_plot_size(data)

        # Get theme from ThemeManager
        is_dark = ThemeManager.is_dark()

        if data.ndim == 1:
            # 1D line plot
            plot_data = data
            if len(plot_data) > 500:
                indices = np.linspace(0, len(plot_data) - 1, 500, dtype=int)
                plot_data = plot_data[indices]
            return DataPlot1D(plot_data, is_dark=is_dark, width=width, height=height, id=widget_id)

        elif data.ndim == 2:
            # 2D heatmap
            rows, cols = data.shape
            max_dim = 100
            if rows > max_dim or cols > max_dim:
                row_step = max(1, rows // max_dim)
                col_step = max(1, cols // max_dim)
                plot_data = data[::row_step, ::col_step]
            else:
                plot_data = data
            return DataPlot2D(plot_data, is_dark=is_dark, width=width, height=height, id=widget_id)

        elif data.ndim >= 3:
            # For 3D+ data, take a 2D slice (first slice of each extra dimension)
            # This shouldn't normally happen as _get_display_data should return 2D
            slice_indices = [0] * (data.ndim - 2) + [slice(None), slice(None)]
            sliced_data = data[tuple(slice_indices)]
            return PlotRenderer.create_plot_widget(sliced_data, widget_id)

        else:
            # Scalar or empty
            return Static(f"Cannot plot data with shape {data.shape}", id=widget_id)
