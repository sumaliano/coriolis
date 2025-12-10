"""Plot widgets for 1D and 2D data visualization."""

from typing import Optional
import numpy as np
from textual_plotext import PlotextPlot

from ...config import ThemeManager
from .colormap import apply_colormap
from .constants import PLOT_1D_DEFAULT_WIDTH, PLOT_1D_HEIGHT, PLOT_2D_DEFAULT_WIDTH, PLOT_2D_DEFAULT_HEIGHT
from .utils import handle_nan_values_1d, handle_nan_values_2d


class DataPlot1D(PlotextPlot):
    """Widget for plotting 1D line data."""

    ALLOW_FOCUS = False

    def __init__(
        self,
        data: np.ndarray,
        is_dark: bool = True,
        width: int = PLOT_1D_DEFAULT_WIDTH,
        height: int = PLOT_1D_HEIGHT,
        **kwargs
    ) -> None:
        """Initialize 1D plot widget.

        Args:
            data: 1D numpy array to plot
            is_dark: Whether to use dark theme
            width: Plot width in characters
            height: Plot height in characters
            **kwargs: Additional arguments passed to PlotextPlot
        """
        super().__init__(**kwargs)
        self._is_dark = is_dark
        self._width = width
        self._height = height

        # Handle NaN values using utility function
        self._data, self._valid_mask = handle_nan_values_1d(data)

        # Set explicit size
        self.styles.width = width
        self.styles.height = height

    def on_mount(self) -> None:
        """Configure and draw the plot."""
        super().on_mount()
        self._draw_plot()

    def _draw_plot(self) -> None:
        """Draw the line plot."""
        colors = ThemeManager.get_plot_colors()
        plot_bg = colors["bg"]
        plot_fg = colors["fg"]
        plot_line = colors.get("line", colors["accent"])

        self.plt.clear_figure()
        self.plt.theme("dark" if self._is_dark else "clear")
        self.plt.canvas_color(plot_bg)
        self.plt.axes_color(plot_bg)
        self.plt.ticks_color(plot_fg)

        # Set plot size
        self.plt.plotsize(self._width, self._height)

        valid_indices = np.where(self._valid_mask)[0]
        x = list(valid_indices)
        
        # Safely convert to floats, handling multi-dimensional elements
        y = []
        for i in valid_indices:
            val = self._data[i]
            # If the value is an array (shouldn't happen for 1D, but be safe)
            if isinstance(val, np.ndarray):
                # Take the first element or mean
                if val.size > 0:
                    y.append(float(val.flat[0]))
                else:
                    y.append(0.0)
            else:
                try:
                    y.append(float(val))
                except (TypeError, ValueError):
                    y.append(0.0)

        if len(x) > 0 and len(y) > 0:
            self.plt.plot(x, y, marker="braille", color=plot_line)
        self.refresh()

    def update_data(self, data: np.ndarray) -> None:
        """Update the plot with new data.

        Args:
            data: New 1D numpy array to display
        """
        # Handle NaN values using utility function
        self._data, self._valid_mask = handle_nan_values_1d(data)
        self._draw_plot()


class DataPlot2D(PlotextPlot):
    """Widget for plotting 2D heatmaps with custom colormap."""

    ALLOW_FOCUS = False

    def __init__(
        self,
        data: np.ndarray,
        is_dark: bool = True,
        width: int = PLOT_2D_DEFAULT_WIDTH,
        height: int = PLOT_2D_DEFAULT_HEIGHT,
        vmin: float = None,
        vmax: float = None,
        **kwargs
    ) -> None:
        """Initialize 2D heatmap plot widget.

        Args:
            data: 2D numpy array to plot
            is_dark: Whether to use dark theme
            width: Plot width in characters
            height: Plot height in characters
            vmin: Minimum value for colormap scaling (optional)
            vmax: Maximum value for colormap scaling (optional)
            **kwargs: Additional arguments passed to PlotextPlot
        """
        super().__init__(**kwargs)
        self._is_dark = is_dark
        self._width = width
        self._height = height
        self._vmin = vmin
        self._vmax = vmax

        # Handle NaN values using utility function
        self._data = handle_nan_values_2d(data)

        # Set explicit size
        self.styles.width = width
        self.styles.height = height

    def on_mount(self) -> None:
        """Configure and draw the heatmap."""
        super().on_mount()
        self._draw_plot()

    def _draw_plot(self) -> None:
        """Draw the heatmap plot."""
        colors = ThemeManager.get_plot_colors()
        plot_bg = colors["bg"]
        plot_fg = colors["fg"]

        self.plt.clear_figure()
        self.plt.theme("dark" if self._is_dark else "clear")
        self.plt.canvas_color(plot_bg)
        self.plt.axes_color(plot_bg)
        self.plt.ticks_color(plot_fg)

        # Set plot size to match data aspect ratio
        self.plt.plotsize(self._width, self._height)

        # Apply colormap with global min/max if provided
        rgb_matrix = apply_colormap(self._data, self._vmin, self._vmax)
        self.plt.matrix_plot(rgb_matrix)
        self.refresh()

    def update_data(self, data: np.ndarray) -> None:
        """Update the plot with new data.

        Args:
            data: New 2D numpy array to display
        """
        # Handle NaN values using utility function
        self._data = handle_nan_values_2d(data)
        self._draw_plot()
