"""Plotting utilities for data visualization in terminal."""

import numpy as np
from typing import Tuple, Optional


class ASCIIPlotter:
    """Creates ASCII plots for terminal display."""

    @staticmethod
    def create_plot(data: np.ndarray, plot_width: int = 60, plot_height: int = 15) -> str:
        """Create an appropriate ASCII plot based on data dimensionality."""
        try:
            # Remove NaN values for plotting
            clean_data = data[~np.isnan(data)] if data.dtype.kind == 'f' else data

            if clean_data.size == 0:
                return "[dim]No valid data to plot[/dim]\n"

            plot_content = "[bold cyan]📊 ASCII Plot[/bold cyan]\n"

            if data.ndim == 1:
                # For 1D data, use line plot or histogram
                if data.size <= 100:
                    plot_content += ASCIIPlotter._create_line_plot(
                        clean_data, height=plot_height, width=plot_width
                    )
                else:
                    plot_content += ASCIIPlotter._create_histogram(
                        clean_data, bins=min(30, plot_width), height=plot_height
                    )
            elif data.ndim == 2:
                # For 2D data, create heatmap with smart sampling
                plot_content += ASCIIPlotter._create_smart_heatmap(
                    data, max_width=plot_width, max_height=plot_height
                )
            else:
                plot_content += "[dim]Plotting only available for 1D and 2D data[/dim]\n"
                plot_content += f"[dim](Data has {data.ndim} dimensions)[/dim]\n"

            return plot_content
        except Exception as e:
            return f"[dim red]Plot error: {e}[/dim red]\n"

    @staticmethod
    def _create_line_plot(data: np.ndarray, height: int = 15, width: int = 60) -> str:
        """Create an ASCII line plot with smart sampling."""
        if data.size == 0:
            return "[dim]No data[/dim]\n"

        # Smart sampling if data is too long
        if len(data) > width:
            # Use extrapolation - sample evenly but preserve trends
            indices = np.linspace(0, len(data) - 1, width).astype(int)
            sampled_data = data[indices]
        else:
            sampled_data = data

        # Normalize data to plot height
        data_min, data_max = np.nanmin(sampled_data), np.nanmax(sampled_data)
        data_range = data_max - data_min

        if data_range == 0:
            data_norm = np.zeros(len(sampled_data), dtype=int)
        else:
            data_norm = ((sampled_data - data_min) / data_range * (height - 1)).astype(int)

        # Create the plot
        plot = ""
        for row in range(height - 1, -1, -1):
            line = ""
            for val in data_norm:
                if val == row:
                    line += "●"
                elif val > row:
                    line += "│"
                else:
                    line += " "

            # Add Y-axis labels
            y_val = data_min + data_range * row / (height - 1)
            plot += f"{y_val:9.3g} │{line}\n"

        # Add X-axis
        plot += " " * 10 + "└" + "─" * len(data_norm) + "\n"

        # Add range info
        plot += f"[dim]Range: [{data_min:.4g}, {data_max:.4g}] | Points: {len(data):,}[/dim]\n"

        return plot

    @staticmethod
    def _create_histogram(data: np.ndarray, bins: int = 30, height: int = 15) -> str:
        """Create an ASCII histogram with better binning."""
        hist, bin_edges = np.histogram(data, bins=bins)
        max_count = hist.max()

        if max_count == 0:
            return "[dim]No data to plot[/dim]\n"

        plot = ""

        # Create histogram bars
        for row in range(height, 0, -1):
            threshold = max_count * row / height
            line = ""
            for count in hist:
                if count >= threshold:
                    line += "█"
                elif count >= threshold * 0.7:
                    line += "▓"
                elif count >= threshold * 0.4:
                    line += "▒"
                elif count >= threshold * 0.1:
                    line += "░"
                else:
                    line += " "

            count_label = int(threshold)
            plot += f"{count_label:7d} │{line}\n"

        # Add axis
        plot += " " * 8 + "└" + "─" * len(hist) + "\n"

        # Add statistics
        plot += f"[dim]Range: [{data.min():.4g}, {data.max():.4g}] | "
        plot += f"Bins: {bins} | Total: {data.size:,}[/dim]\n"

        return plot

    @staticmethod
    def _create_smart_heatmap(
        data: np.ndarray,
        max_width: int = 60,
        max_height: int = 20
    ) -> str:
        """Create an ASCII heatmap with intelligent sampling."""
        rows, cols = data.shape

        # Smart sampling to fit the display
        if rows > max_height or cols > max_width:
            # Calculate sampling rates
            row_step = max(1, rows // max_height)
            col_step = max(1, cols // max_width)

            # Sample the data using striding
            sampled_data = data[::row_step, ::col_step]

            # Also compute min/max for each sampled region to preserve features
            if row_step > 1 or col_step > 1:
                # Use block reduction to preserve important features
                sampled_rows = (rows // row_step)
                sampled_cols = (cols // col_step)
                enhanced_data = np.zeros((sampled_rows, sampled_cols))

                for i in range(sampled_rows):
                    for j in range(sampled_cols):
                        row_start = i * row_step
                        row_end = min(row_start + row_step, rows)
                        col_start = j * col_step
                        col_end = min(col_start + col_step, cols)

                        block = data[row_start:row_end, col_start:col_end]
                        # Use mean for smoother visualization
                        enhanced_data[i, j] = np.nanmean(block)

                sampled_data = enhanced_data

            info = f"[dim](Sampled {sampled_data.shape[0]}×{sampled_data.shape[1]} from {rows}×{cols})[/dim]\n"
        else:
            sampled_data = data
            info = f"[dim](Full resolution: {rows}×{cols})[/dim]\n"

        # Normalize to character range
        data_min, data_max = np.nanmin(sampled_data), np.nanmax(sampled_data)
        data_range = data_max - data_min

        if data_range == 0:
            data_norm = np.zeros_like(sampled_data, dtype=int)
        else:
            data_norm = ((sampled_data - data_min) / data_range * 9).astype(int)

        # Characters from dark to light (more gradations)
        chars = " ·:·=+*#%@"

        # Build the heatmap
        plot = info
        for row in data_norm:
            line = "".join(chars[min(int(val), 9)] for val in row)
            plot += f"{line}\n"

        plot += f"[dim]Range: [{data_min:.4g}, {data_max:.4g}][/dim]\n"

        return plot


def format_array_sample(data: np.ndarray, max_lines: int = 10) -> str:
    """Format array data for display with smart sampling."""
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
        # 2D array: show corner
        rows, cols = data.shape
        show_rows = min(max_lines, rows)
        show_cols = min(12, cols)

        content += f"  [dim]Corner ({show_rows}×{show_cols} of {rows}×{cols}):[/dim]\n"
        for i in range(show_rows):
            row_vals = [f"{data[i, j]:9.3g}" for j in range(show_cols)]
            content += "  " + " ".join(row_vals)
            if cols > show_cols:
                content += " ..."
            content += "\n"

        if rows > show_rows:
            content += "  [dim]...[/dim]\n"

    else:
        # Multi-dimensional: show flattened sample
        content += f"  [dim]First {max_lines} values (flattened from {data.ndim}D):[/dim]\n"
        for i in range(min(max_lines, data.size)):
            content += f"    {data.flat[i]}\n"

        if data.size > max_lines:
            shape_str = " × ".join(f"{s:,}" for s in data.shape)
            content += f"  [dim](Total shape: {shape_str}, {data.size:,} elements)[/dim]\n"

    return content
