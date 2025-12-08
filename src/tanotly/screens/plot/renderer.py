"""Plot rendering logic for creating and sizing plot widgets."""

from typing import Union, Tuple
import numpy as np
from textual.widgets import Static

from ...config import ThemeManager
from .widgets import DataPlot1D, DataPlot2D
from .constants import (
    PLOT_1D_MIN_WIDTH,
    PLOT_1D_MAX_WIDTH,
    PLOT_1D_HEIGHT,
    PLOT_2D_MAX_WIDTH,
    PLOT_2D_MAX_HEIGHT,
    PLOT_2D_MIN_WIDTH,
    PLOT_2D_MIN_HEIGHT,
    DEFAULT_PLOT_WIDTH,
    DEFAULT_PLOT_HEIGHT,
    TERMINAL_ASPECT_RATIO,
    DOWNSAMPLE_1D_THRESHOLD,
    DOWNSAMPLE_2D_THRESHOLD,
)
from .utils import downsample_1d, downsample_2d


def calculate_plot_size(data: np.ndarray) -> Tuple[int, int]:
    """Calculate appropriate plot size based on data shape.

    Args:
        data: Input data array

    Returns:
        Tuple of (width, height) in characters
    """
    if data.ndim == 1:
        # 1D: fixed height, width based on data length
        width = min(max(len(data), PLOT_1D_MIN_WIDTH), PLOT_1D_MAX_WIDTH)
        height = PLOT_1D_HEIGHT
    elif data.ndim == 2:
        rows, cols = data.shape
        # Scale to fit while maintaining aspect ratio
        # (terminal characters are ~2x taller than wide)
        width = min(cols, PLOT_2D_MAX_WIDTH)
        height = min(rows // TERMINAL_ASPECT_RATIO + 1, PLOT_2D_MAX_HEIGHT)

        # Ensure minimum size
        width = max(width, PLOT_2D_MIN_WIDTH)
        height = max(height, PLOT_2D_MIN_HEIGHT)
    else:
        # Unsupported dimensionality
        width, height = DEFAULT_PLOT_WIDTH, DEFAULT_PLOT_HEIGHT

    return width, height


def create_plot_widget(
    data: np.ndarray,
    widget_id: str = "plot-widget"
) -> Union[DataPlot1D, DataPlot2D, Static]:
    """Create plot widget sized to data.

    Args:
        data: The data to plot (should be 1D or 2D after slicing)
        widget_id: CSS ID for the widget

    Returns:
        DataPlot1D for 1D data, DataPlot2D for 2D data, or Static for unsupported shapes
    """
    # Ensure we have a proper numpy array copy
    data = np.array(data, copy=True)
    width, height = calculate_plot_size(data)

    # Get theme from ThemeManager
    is_dark = ThemeManager.is_dark()

    if data.ndim == 1:
        # 1D line plot - downsample if needed
        plot_data = downsample_1d(data, DOWNSAMPLE_1D_THRESHOLD)
        return DataPlot1D(
            plot_data,
            is_dark=is_dark,
            width=width,
            height=height,
            id=widget_id
        )

    elif data.ndim == 2:
        # 2D heatmap - downsample if needed
        plot_data = downsample_2d(data, DOWNSAMPLE_2D_THRESHOLD)
        return DataPlot2D(
            plot_data,
            is_dark=is_dark,
            width=width,
            height=height,
            id=widget_id
        )

    elif data.ndim >= 3:
        # For 3D+ data, take a 2D slice
        slice_indices = [0] * (data.ndim - 2) + [slice(None), slice(None)]
        sliced_data = data[tuple(slice_indices)]
        return create_plot_widget(sliced_data, widget_id)

    else:
        # Scalar or empty
        return Static(f"Cannot plot data with shape {data.shape}", id=widget_id)
