"""Data visualization widgets using textual-plotext."""

import numpy as np
from textual.widgets import Static
from textual.app import ComposeResult
from textual_plotext import PlotextPlot

from .config import ThemeManager


# Viridis-like colormap (perceptually uniform, colorblind-friendly)
VIRIDIS_COLORS = [
    (68, 1, 84), (72, 26, 108), (71, 47, 125), (65, 68, 135), (57, 86, 140),
    (49, 104, 142), (42, 120, 142), (35, 136, 142), (31, 152, 139), (34, 168, 132),
    (53, 183, 121), (83, 198, 105), (122, 209, 81), (165, 219, 54), (210, 226, 27),
    (253, 231, 37),
]


def _apply_colormap(data: np.ndarray) -> list[list[tuple[int, int, int]]]:
    """Apply viridis colormap to normalized data, returning RGB tuples."""
    data_clean = np.nan_to_num(data, nan=0.0)
    data_min = np.nanmin(data_clean)
    data_max = np.nanmax(data_clean)

    if data_max == data_min:
        normalized = np.full_like(data_clean, 0.5)
    else:
        normalized = (data_clean - data_min) / (data_max - data_min)

    normalized = np.clip(normalized, 0.0, 1.0)

    n_colors = len(VIRIDIS_COLORS)
    result = []
    for row in normalized:
        rgb_row = []
        for val in row:
            idx = min(max(int(val * (n_colors - 1)), 0), n_colors - 1)
            rgb_row.append(VIRIDIS_COLORS[idx])
        result.append(rgb_row)

    return result


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

        rgb_matrix = _apply_colormap(self._data)
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


def format_statistics(data: np.ndarray) -> str:
    """Format statistics for numeric data."""
    if not np.issubdtype(data.dtype, np.number):
        return ""

    is_float = data.dtype.kind == 'f'
    valid_count = np.count_nonzero(~np.isnan(data)) if is_float else data.size
    nan_count = data.size - valid_count

    lines = ["[bold]Statistics:[/bold]"]
    lines.append(f"  ▸ Min:  {np.nanmin(data):.6g}")
    lines.append(f"  ▸ Max:  {np.nanmax(data):.6g}")
    lines.append(f"  ▸ Mean: {np.nanmean(data):.6g}")

    if data.size > 1:
        lines.append(f"  ▸ Std:  {np.nanstd(data):.6g}")

    if nan_count > 0:
        lines.append(f"  ▸ NaN:  {nan_count:,} ({nan_count/data.size*100:.1f}%)")

    lines.append(f"  ▸ Valid: {valid_count:,}")

    return "\n".join(lines) + "\n"


def format_sample_values(data: np.ndarray, max_lines: int = 8) -> str:
    """Format sample values showing first and last elements."""
    if data.size == 0:
        return "(empty array)\n"

    if data.ndim == 1:
        return _format_1d_samples(data, max_lines)
    elif data.ndim == 2:
        return _format_2d_samples(data, max_lines)
    else:
        return _format_nd_samples(data, max_lines)


def _format_1d_samples(data: np.ndarray, max_lines: int) -> str:
    """Format 1D array samples."""
    if data.size <= max_lines:
        return "\n".join(f"  [{i}] {val}" for i, val in enumerate(data)) + "\n"

    n = max_lines // 2
    lines = ["  First values:"]
    lines.extend(f"    [{i}] {data[i]}" for i in range(n))
    lines.append(f"  ... ({data.size - 2*n} more) ...")
    lines.append("  Last values:")
    lines.extend(f"    [{i}] {data[i]}" for i in range(data.size - n, data.size))
    return "\n".join(lines) + "\n"


def _format_2d_samples(data: np.ndarray, max_lines: int) -> str:
    """Format 2D array samples showing corners."""
    rows, cols = data.shape
    show_rows = min(max_lines // 2, 4)
    show_cols = min(8, cols)

    def format_rows(start: int, end: int) -> list[str]:
        result = []
        for i in range(start, min(end, rows)):
            vals = " ".join(f"{data[i, j]:9.3g}" for j in range(min(show_cols, cols)))
            result.append("  " + vals + (" ..." if cols > show_cols else ""))
        return result

    lines = [f"  Top-left ({show_rows}×{show_cols}):"]
    lines.extend(format_rows(0, show_rows))

    if rows > show_rows * 2:
        lines.append(f"  ... {rows - show_rows * 2} rows omitted ...")

    if rows > show_rows:
        lines.append(f"  Bottom-left ({show_rows}×{show_cols}):")
        lines.extend(format_rows(max(0, rows - show_rows), rows))

    return "\n".join(lines) + "\n"


def _format_nd_samples(data: np.ndarray, max_lines: int) -> str:
    """Format multi-dimensional array samples."""
    n = max_lines // 2
    lines = [f"  First {n} values (from {data.ndim}D array):"]
    lines.extend(f"    {data.flat[i]}" for i in range(min(n, data.size)))

    if data.size > max_lines:
        lines.append(f"  ... {data.size - max_lines} values omitted ...")
        lines.append(f"  Last {n} values:")
        lines.extend(f"    {data.flat[i]}" for i in range(max(0, data.size - n), data.size))

    shape_str = " × ".join(f"{s:,}" for s in data.shape)
    lines.append(f"  (Shape: {shape_str}, Total: {data.size:,})")
    return "\n".join(lines) + "\n"
