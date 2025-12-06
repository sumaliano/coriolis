"""Data visualization widgets using textual-plotext."""

import numpy as np
from textual.widgets import Static
from textual.app import ComposeResult
from textual_plotext import PlotextPlot


def _apply_colormap(data: np.ndarray) -> list[list[tuple[int, int, int]]]:
    """Apply a blue-to-red colormap to normalized data, returning RGB tuples."""
    # Normalize data to 0-1 range
    data_min = np.nanmin(data)
    data_max = np.nanmax(data)

    if data_max == data_min:
        # Constant data - use middle color
        normalized = np.full_like(data, 0.5)
    else:
        normalized = (data - data_min) / (data_max - data_min)

    # Replace NaN with 0
    normalized = np.nan_to_num(normalized, nan=0.0)

    # Create RGB colormap: blue (cold) -> cyan -> green -> yellow -> red (hot)
    # This is a perceptually better colormap than simple grayscale
    result = []
    for row in normalized:
        rgb_row = []
        for val in row:
            if val < 0.25:
                # Blue to cyan
                t = val / 0.25
                r, g, b = 0, int(255 * t), 255
            elif val < 0.5:
                # Cyan to green
                t = (val - 0.25) / 0.25
                r, g, b = 0, 255, int(255 * (1 - t))
            elif val < 0.75:
                # Green to yellow
                t = (val - 0.5) / 0.25
                r, g, b = int(255 * t), 255, 0
            else:
                # Yellow to red
                t = (val - 0.75) / 0.25
                r, g, b = 255, int(255 * (1 - t)), 0
            rgb_row.append((r, g, b))
        result.append(rgb_row)

    return result


class DataPlot1D(PlotextPlot):
    """Widget for plotting 1D line data."""

    ALLOW_FOCUS = False  # Disable mouse to prevent hover color changes

    def __init__(self, data: np.ndarray, **kwargs):
        super().__init__(**kwargs)
        if np.issubdtype(data.dtype, np.floating):
            clean = data[~np.isnan(data)]
            self._data = clean if len(clean) > 0 else data[:10]
        else:
            self._data = data

    def on_mount(self) -> None:
        """Configure and draw the plot."""
        self.plt.theme("dark")
        self.plt.canvas_color((0, 0, 0))  # Pure black background
        self.plt.axes_color((0, 0, 0))
        self.plt.ticks_color((200, 200, 200))  # Light gray ticks
        self.plt.title("1D Data")
        self.plt.xlabel("Index")
        self.plt.ylabel("Value")
        self._draw()

    def _draw(self) -> None:
        """Draw the line plot."""
        self.plt.clear_data()
        self.plt.plotsize(None, 18)

        x = list(range(len(self._data)))
        y = [float(v) if np.isfinite(v) else 0.0 for v in self._data]

        self.plt.plot(x, y, marker="braille", color=(0, 200, 255))  # Cyan
        self.refresh()


class DataPlot2D(PlotextPlot):
    """Widget for plotting 2D heatmaps with custom colormap."""

    ALLOW_FOCUS = False  # Disable mouse to prevent hover color changes

    def __init__(self, data: np.ndarray, **kwargs):
        super().__init__(**kwargs)
        self._data = np.nan_to_num(data, nan=0.0)

    def on_mount(self) -> None:
        """Configure and draw the heatmap."""
        self.plt.theme("dark")
        self.plt.canvas_color((0, 0, 0))  # Pure black background
        self.plt.axes_color((0, 0, 0))
        self.plt.ticks_color((200, 200, 200))  # Light gray ticks
        self.plt.title("2D Heatmap")
        self._draw()

    def _draw(self) -> None:
        """Draw the heatmap with custom colormap."""
        self.plt.clear_data()
        self.plt.plotsize(None, 18)

        # Apply colormap to get RGB matrix
        rgb_matrix = _apply_colormap(self._data)

        # Plot with RGB colors
        self.plt.matrix_plot(rgb_matrix)

        # Add axis labels with data range
        rows, cols = self._data.shape
        data_min = np.nanmin(self._data)
        data_max = np.nanmax(self._data)
        self.plt.xlabel(f"Col 0-{cols-1} | Range: [{data_min:.3g}, {data_max:.3g}]")
        self.plt.ylabel(f"Row 0-{rows-1}")

        self.refresh()


class DataVisualizer:
    """Creates visualization widgets for numpy arrays."""

    MAX_PLOT_POINTS_1D = 500
    MAX_HEATMAP_ROWS = 40
    MAX_HEATMAP_COLS = 60

    @staticmethod
    def create_visualization(data: np.ndarray, container_width: int = 60) -> ComposeResult:
        """Create visualization widgets based on data dimensions."""
        if data.ndim == 1:
            yield from DataVisualizer._create_line_plot(data)
        elif data.ndim == 2:
            yield from DataVisualizer._create_heatmap(data)
        else:
            yield Static(f"[dim]Visualization not available for {data.ndim}D data[/dim]")

    @staticmethod
    def _create_line_plot(data: np.ndarray) -> ComposeResult:
        """Create a line plot for 1D data with intelligent downsampling."""
        # Remove NaN values
        if data.dtype.kind == 'f':
            clean_data = data[~np.isnan(data)]
        else:
            clean_data = data

        if clean_data.size == 0:
            yield Static("[dim]No valid data to plot[/dim]")
            return

        # Downsample if needed
        max_points = DataVisualizer.MAX_PLOT_POINTS_1D
        if len(clean_data) > max_points:
            indices = np.linspace(0, len(clean_data) - 1, max_points, dtype=int)
            plot_data = clean_data[indices]
            info = f"[dim]({max_points} of {len(clean_data):,} points)[/dim]"
        else:
            plot_data = clean_data
            info = f"[dim]({len(clean_data):,} points)[/dim]"

        yield Static(f"[bold cyan]📊 Line Plot[/bold cyan] {info}")
        yield DataPlot1D(plot_data)

        # Statistics
        stats = f"Range: [{clean_data.min():.4g}, {clean_data.max():.4g}] | "
        stats += f"Mean: {clean_data.mean():.4g} | Std: {clean_data.std():.4g}"
        yield Static(f"[dim]{stats}[/dim]")

    @staticmethod
    def _create_heatmap(data: np.ndarray) -> ComposeResult:
        """Create a heatmap for 2D data with efficient downsampling."""
        rows, cols = data.shape
        max_rows = DataVisualizer.MAX_HEATMAP_ROWS
        max_cols = DataVisualizer.MAX_HEATMAP_COLS

        if rows > max_rows or cols > max_cols:
            # Efficient downsampling using stride and reshape
            row_step = max(1, rows // max_rows)
            col_step = max(1, cols // max_cols)

            # Simple strided sampling (fast)
            sampled = data[::row_step, ::col_step]

            # Ensure we don't exceed max dimensions
            sampled = sampled[:max_rows, :max_cols]
            info = f"[dim]({sampled.shape[0]}×{sampled.shape[1]} of {rows}×{cols})[/dim]"
        else:
            sampled = data
            info = f"[dim]({rows}×{cols})[/dim]"

        yield Static(f"[bold cyan]📊 2D Heatmap[/bold cyan] {info}")
        yield DataPlot2D(sampled)

        # Range info
        data_min = np.nanmin(sampled)
        data_max = np.nanmax(sampled)
        yield Static(f"[dim]Range: [{data_min:.4g}, {data_max:.4g}][/dim]")


def format_statistics(data: np.ndarray) -> str:
    """Format statistics for numeric data."""
    if not np.issubdtype(data.dtype, np.number):
        return ""

    is_float = data.dtype.kind == 'f'
    valid_count = np.count_nonzero(~np.isnan(data)) if is_float else data.size
    nan_count = data.size - valid_count

    lines = ["[cyan]Statistics:[/cyan]"]
    lines.append(f"  [dim]▸ Min:[/dim]  {np.nanmin(data):.6g}")
    lines.append(f"  [dim]▸ Max:[/dim]  {np.nanmax(data):.6g}")
    lines.append(f"  [dim]▸ Mean:[/dim] {np.nanmean(data):.6g}")

    if data.size > 1:
        lines.append(f"  [dim]▸ Std:[/dim]  {np.nanstd(data):.6g}")

    if nan_count > 0:
        lines.append(f"  [dim]▸ NaN:[/dim]  {nan_count:,} ({nan_count/data.size*100:.1f}%)")

    lines.append(f"  [dim]▸ Valid:[/dim] {valid_count:,}")

    return "\n".join(lines) + "\n"


def format_sample_values(data: np.ndarray, max_lines: int = 8) -> str:
    """Format sample values showing first and last elements."""
    if data.size == 0:
        return "[dim](empty array)[/dim]\n"

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
    lines = ["  [dim]First values:[/dim]"]
    lines.extend(f"    [{i}] {data[i]}" for i in range(n))
    lines.append(f"  [dim]... ({data.size - 2*n} more) ...[/dim]")
    lines.append("  [dim]Last values:[/dim]")
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

    lines = [f"  [dim]Top-left ({show_rows}×{show_cols}):[/dim]"]
    lines.extend(format_rows(0, show_rows))

    if rows > show_rows * 2:
        lines.append(f"  [dim]... {rows - show_rows * 2} rows omitted ...[/dim]")

    if rows > show_rows:
        lines.append(f"  [dim]Bottom-left ({show_rows}×{show_cols}):[/dim]")
        lines.extend(format_rows(max(0, rows - show_rows), rows))

    return "\n".join(lines) + "\n"


def _format_nd_samples(data: np.ndarray, max_lines: int) -> str:
    """Format multi-dimensional array samples."""
    n = max_lines // 2
    lines = [f"  [dim]First {n} values (from {data.ndim}D array):[/dim]"]
    lines.extend(f"    {data.flat[i]}" for i in range(min(n, data.size)))

    if data.size > max_lines:
        lines.append(f"  [dim]... {data.size - max_lines} values omitted ...[/dim]")
        lines.append(f"  [dim]Last {n} values:[/dim]")
        lines.extend(f"    {data.flat[i]}" for i in range(max(0, data.size - n), data.size))

    shape_str = " × ".join(f"{s:,}" for s in data.shape)
    lines.append(f"  [dim](Shape: {shape_str}, Total: {data.size:,})[/dim]")
    return "\n".join(lines) + "\n"
