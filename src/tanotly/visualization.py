"""Data visualization widgets using textual-plotext."""

import numpy as np
import pandas as pd
from textual.widgets import Static
from textual.app import ComposeResult
from textual_plotext import PlotextPlot
from rich.text import Text
from typing import Optional


class DataPlot1D(PlotextPlot):
    """Widget for plotting 1D data."""

    def __init__(self, data: np.ndarray, **kwargs):
        super().__init__(**kwargs)
        # Clean NaN values before storing
        clean_data = data[~np.isnan(data)] if np.issubdtype(data.dtype, np.floating) else data
        self._data = clean_data if len(clean_data) > 0 else data[:10]

    def on_mount(self) -> None:
        """Initialize plot settings when mounted."""
        self.plt.title("1D Data Plot")
        self.plt.xlabel("Index")
        self.plt.ylabel("Value")
        self.replot()

    def replot(self) -> None:
        """Plot the data."""
        self.plt.clear_data()

        # Set plot size (width, height in characters)
        self.plt.plotsize(None, 20)  # Auto width, 20 lines height

        # Create x-axis indices
        x = list(range(len(self._data)))
        y = [float(v) if not np.isnan(v) else 0.0 for v in self._data]

        # Plot the data
        self.plt.plot(x, y, marker="braille")
        self.refresh()


class DataPlot2D(PlotextPlot):
    """Widget for plotting 2D heatmaps."""

    def __init__(self, data: np.ndarray, **kwargs):
        super().__init__(**kwargs)
        # Replace NaN with 0 for plotting
        self._data = np.nan_to_num(data, nan=0.0)

    def on_mount(self) -> None:
        """Initialize plot settings when mounted."""
        self.plt.title("2D Heatmap")
        self.replot()

    def replot(self) -> None:
        """Plot the heatmap."""
        self.plt.clear_data()

        # Set plot size (width, height in characters)
        self.plt.plotsize(None, 25)  # Auto width, 25 lines height

        # Create DataFrame with empty string labels to minimize label space
        rows, cols = self._data.shape
        df = pd.DataFrame(
            self._data,
            columns=[""] * cols,  # Empty labels
            index=[""] * rows     # Empty labels
        )

        # Plot heatmap
        self.plt.heatmap(df)
        self.refresh()


class DataVisualizer:
    """Creates appropriate visualization widgets for different data types."""

    @staticmethod
    def create_visualization(data: np.ndarray, container_width: int = 60) -> ComposeResult:
        """
        Create appropriate visualization widgets based on data dimensionality.

        Args:
            data: The numpy array to visualize
            container_width: Available width for visualization

        Yields:
            Textual widgets for visualization
        """
        if data.ndim == 1:
            # Use PlotextPlot for 1D data
            yield from DataVisualizer._create_line_plot(data)
        elif data.ndim == 2:
            # Use PlotextPlot heatmap for 2D data
            yield from DataVisualizer._create_heatmap(data)
        else:
            # Fallback message for higher dimensions
            yield Static(f"[dim]Visualization not available for {data.ndim}D data[/dim]")

    @staticmethod
    def _create_line_plot(data: np.ndarray) -> ComposeResult:
        """Create a line plot widget for 1D data using plotext."""
        # Clean data - remove NaN values
        clean_data = data[~np.isnan(data)] if data.dtype.kind == 'f' else data

        if clean_data.size == 0:
            yield Static("[dim]No valid data to plot[/dim]")
            return

        # Downsample intelligently to show full range
        max_points = 500
        if len(clean_data) > max_points:
            # Use linear spacing across the entire range to show full data
            indices = np.linspace(0, len(clean_data) - 1, max_points).astype(int)
            sampled_data = clean_data[indices]
            info_text = f"[dim](Showing {max_points} sampled points from {len(clean_data):,} total)[/dim]"
        else:
            sampled_data = clean_data
            info_text = f"[dim]({len(clean_data):,} points)[/dim]"

        yield Static(f"[bold cyan]📊 Data Plot[/bold cyan] {info_text}")
        yield DataPlot1D(sampled_data)

        # Add statistics
        stats_text = f"Range: [{clean_data.min():.4g}, {clean_data.max():.4g}] | "
        stats_text += f"Mean: {clean_data.mean():.4g} | Std: {clean_data.std():.4g}"
        yield Static(f"[dim]{stats_text}[/dim]")

    @staticmethod
    def _create_heatmap(data: np.ndarray) -> ComposeResult:
        """Create a heatmap widget for 2D data using plotext with downsampling."""
        rows, cols = data.shape

        # Downsample BEFORE creating widget to save resources
        max_display_rows = 30  # Smaller for performance
        max_display_cols = 50

        if rows > max_display_rows or cols > max_display_cols:
            # Calculate steps to cover full range
            row_step = max(1, rows // max_display_rows)
            col_step = max(1, cols // max_display_cols)

            # Use block averaging to preserve features across entire array
            sampled_rows = min(max_display_rows, (rows + row_step - 1) // row_step)
            sampled_cols = min(max_display_cols, (cols + col_step - 1) // col_step)
            sampled_data = np.zeros((sampled_rows, sampled_cols))

            for i in range(sampled_rows):
                for j in range(sampled_cols):
                    row_start = i * row_step
                    row_end = min(row_start + row_step, rows)
                    col_start = j * col_step
                    col_end = min(col_start + col_step, cols)

                    block = data[row_start:row_end, col_start:col_end]
                    sampled_data[i, j] = np.nanmean(block)

            info_text = f"[dim](Downsampled {sampled_rows}×{sampled_cols} from {rows}×{cols})[/dim]"
        else:
            sampled_data = data
            info_text = f"[dim]({rows}×{cols})[/dim]"

        yield Static(f"[bold cyan]📊 2D Heatmap[/bold cyan] {info_text}")
        # Pass downsampled data to widget
        yield DataPlot2D(sampled_data)

        # Add range info
        data_min, data_max = np.nanmin(sampled_data), np.nanmax(sampled_data)
        range_text = f"Range: [{data_min:.4g}, {data_max:.4g}]"
        yield Static(f"[dim]{range_text}[/dim]")


def format_statistics(data: np.ndarray) -> str:
    """Format statistics for numeric data."""
    if not np.issubdtype(data.dtype, np.number):
        return ""

    # Count valid values
    valid_count = np.count_nonzero(~np.isnan(data)) if data.dtype.kind == 'f' else data.size
    nan_count = data.size - valid_count

    content = "[cyan]Statistics:[/cyan]\n"
    content += f"  [dim]▸ Min:[/dim]  {np.nanmin(data):.6g}\n"
    content += f"  [dim]▸ Max:[/dim]  {np.nanmax(data):.6g}\n"
    content += f"  [dim]▸ Mean:[/dim] {np.nanmean(data):.6g}\n"

    if data.size > 1:
        content += f"  [dim]▸ Std:[/dim]  {np.nanstd(data):.6g}\n"

    if nan_count > 0:
        content += f"  [dim]▸ NaN:[/dim]  {nan_count:,} ({nan_count/data.size*100:.1f}%)\n"

    content += f"  [dim]▸ Valid:[/dim] {valid_count:,}\n"

    return content


def format_sample_values(data: np.ndarray, max_lines: int = 8) -> str:
    """Format sample values showing full range (first and last) instead of just corner."""
    if data.size == 0:
        return "[dim](empty array)[/dim]\n"

    content = ""

    if data.ndim == 1:
        # 1D array: show first and last values
        if data.size <= max_lines:
            for i, val in enumerate(data):
                content += f"  [{i}] {val}\n"
        else:
            show_count = max_lines // 2
            content += "  [dim]First values:[/dim]\n"
            for i in range(show_count):
                content += f"    [{i}] {data[i]}\n"

            content += f"  [dim]... ({data.size - 2*show_count} more) ...[/dim]\n"

            content += "  [dim]Last values:[/dim]\n"
            for i in range(data.size - show_count, data.size):
                content += f"    [{i}] {data[i]}\n"

    elif data.ndim == 2:
        # 2D array: show corners from all four quadrants to represent full range
        rows, cols = data.shape
        show_rows = min(max_lines // 2, 4)
        show_cols = min(8, cols)

        # Top-left corner
        content += f"  [dim]Top-left ({show_rows}×{show_cols}):[/dim]\n"
        for i in range(min(show_rows, rows)):
            row_vals = [f"{data[i, j]:9.3g}" for j in range(min(show_cols, cols))]
            content += "  " + " ".join(row_vals)
            if cols > show_cols:
                content += " ..."
            content += "\n"

        if rows > show_rows * 2:
            content += f"  [dim]... {rows - show_rows * 2} rows omitted ...[/dim]\n"

        # Bottom-left corner (to show full vertical range)
        if rows > show_rows:
            content += f"  [dim]Bottom-left ({show_rows}×{show_cols}):[/dim]\n"
            for i in range(max(0, rows - show_rows), rows):
                row_vals = [f"{data[i, j]:9.3g}" for j in range(min(show_cols, cols))]
                content += "  " + " ".join(row_vals)
                if cols > show_cols:
                    content += " ..."
                content += "\n"

    else:
        # Multi-dimensional: show first and last values to represent full range
        content += f"  [dim]First {max_lines//2} values (from {data.ndim}D array):[/dim]\n"
        for i in range(min(max_lines//2, data.size)):
            content += f"    {data.flat[i]}\n"

        if data.size > max_lines:
            content += f"  [dim]... {data.size - max_lines} values omitted ...[/dim]\n"
            content += f"  [dim]Last {max_lines//2} values:[/dim]\n"
            for i in range(max(0, data.size - max_lines//2), data.size):
                content += f"    {data.flat[i]}\n"

        shape_str = " × ".join(f"{s:,}" for s in data.shape)
        content += f"  [dim](Shape: {shape_str}, Total: {data.size:,})[/dim]\n"

    return content
