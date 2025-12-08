"""Plot rendering components for data visualization."""

from .colormap import VIRIDIS_COLORS, apply_colormap
from .widgets import DataPlot1D, DataPlot2D
from .renderer import PlotRenderer

__all__ = ["VIRIDIS_COLORS", "apply_colormap", "DataPlot1D", "DataPlot2D", "PlotRenderer"]
