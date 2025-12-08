"""Plot widgets for 1D and 2D data visualization."""

import numpy as np
from textual_plotext import PlotextPlot

from ...config import ThemeManager
from .colormap import apply_colormap


class DataPlot1D(PlotextPlot):
    """Widget for plotting 1D line data."""

    ALLOW_FOCUS = False

    def __init__(self, data: np.ndarray, is_dark: bool = True, width: int = 60, height: int = 15, **kwargs):
        super().__init__(**kwargs)
        self._is_dark = is_dark
        self._width = width
        self._height = height

        # Handle NaN values
        if np.issubdtype(data.dtype, np.floating):
            mask = np.isfinite(data)
            if np.any(mask):
                self._data = data.copy()
                self._valid_mask = mask
            else:
                self._data = np.zeros(min(10, len(data)))
                self._valid_mask = np.ones(len(self._data), dtype=bool)
        else:
            self._data = data
            self._valid_mask = np.ones(len(data), dtype=bool)

        # Set explicit size
        self.styles.width = width
        self.styles.height = height

    def on_mount(self) -> None:
        """Configure and draw the plot."""
        super().on_mount()
        colors = ThemeManager.get_plot_colors()
        plot_bg = colors["bg"]
        plot_fg = colors["fg"]
        plot_line = colors.get("line", colors["accent"])

        self.plt.theme("dark" if self._is_dark else "clear")
        self.plt.canvas_color(plot_bg)
        self.plt.axes_color(plot_bg)
        self.plt.ticks_color(plot_fg)

        # Set plot size
        self.plt.plotsize(self._width, self._height)

        valid_indices = np.where(self._valid_mask)[0]
        x = list(valid_indices)
        y = [float(self._data[i]) for i in valid_indices]

        if len(x) > 0:
            self.plt.plot(x, y, marker="braille", color=plot_line)
        self.refresh()


class DataPlot2D(PlotextPlot):
    """Widget for plotting 2D heatmaps with custom colormap."""

    ALLOW_FOCUS = False

    def __init__(self, data: np.ndarray, is_dark: bool = True, width: int = 60, height: int = 20, **kwargs):
        super().__init__(**kwargs)
        self._is_dark = is_dark
        self._width = width
        self._height = height

        # Handle NaN values and ensure we have a proper copy
        data = np.array(data, copy=True)
        if np.issubdtype(data.dtype, np.floating):
            nan_mask = np.isnan(data)
            if np.any(nan_mask):
                valid_mean = np.nanmean(data) if np.any(~nan_mask) else 0.0
                self._data = np.where(nan_mask, valid_mean, data)
            else:
                self._data = data
        else:
            self._data = data.astype(float)

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

        rgb_matrix = apply_colormap(self._data)
        self.plt.matrix_plot(rgb_matrix)
        self.refresh()

    def update_data(self, data: np.ndarray) -> None:
        """Update the plot with new data."""
        # Handle NaN values and ensure we have a proper copy
        data = np.array(data, copy=True)
        if np.issubdtype(data.dtype, np.floating):
            nan_mask = np.isnan(data)
            if np.any(nan_mask):
                valid_mean = np.nanmean(data) if np.any(~nan_mask) else 0.0
                self._data = np.where(nan_mask, valid_mean, data)
            else:
                self._data = data
        else:
            self._data = data.astype(float)

        self._draw_plot()
