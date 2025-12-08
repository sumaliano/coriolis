"""Colorbar legend component for 2D plots."""

import numpy as np
from textual.widgets import Static

from ..plot.colormap import VIRIDIS_COLORS


class ColorbarLegend:
    """Creates colorbar legends for heatmap plots."""

    @staticmethod
    def create_colorbar_text(data: np.ndarray) -> str:
        """Create colorbar text with min/max values.

        Args:
            data: Data array to create colorbar for

        Returns:
            Rich-formatted string with colorbar gradient
        """
        data_min = float(np.nanmin(data))
        data_max = float(np.nanmax(data))

        # Format values
        def fmt(v):
            if abs(v) >= 1e4 or (abs(v) < 1e-3 and v != 0):
                return f"{v:.2e}"
            return f"{v:.3g}"

        # Build colorbar with gradient
        n_colors = 16
        color_blocks = []
        for i in range(n_colors):
            r, g, b = VIRIDIS_COLORS[int(i * (len(VIRIDIS_COLORS) - 1) / (n_colors - 1))]
            color_blocks.append(f"[rgb({r},{g},{b})]█[/]")

        colorbar = "".join(color_blocks)
        return f"{fmt(data_min)} {colorbar} {fmt(data_max)}"

    @staticmethod
    def create_widget(data: np.ndarray, widget_id: str = "colorbar-legend") -> Static:
        """Create a colorbar legend widget.

        Args:
            data: Data array to create colorbar for
            widget_id: CSS ID for the widget

        Returns:
            Static widget with colorbar text
        """
        return Static(ColorbarLegend.create_colorbar_text(data), id=widget_id)
