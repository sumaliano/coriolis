"""Colormap utilities for data visualization.

Provides viridis-like perceptually uniform, colorblind-friendly colormap.
"""

from typing import List, Tuple
import numpy as np


# Viridis-like colormap (perceptually uniform, colorblind-friendly)
VIRIDIS_COLORS = [
    (68, 1, 84), (72, 26, 108), (71, 47, 125), (65, 68, 135), (57, 86, 140),
    (49, 104, 142), (42, 120, 142), (35, 136, 142), (31, 152, 139), (34, 168, 132),
    (53, 183, 121), (83, 198, 105), (122, 209, 81), (165, 219, 54), (210, 226, 27),
    (253, 231, 37),
]


def apply_colormap(
    data: np.ndarray,
    vmin: float = None,
    vmax: float = None
) -> List[List[Tuple[int, int, int]]]:
    """Apply viridis colormap to normalized data, returning RGB tuples.

    Args:
        data: 2D numpy array to apply colormap to
        vmin: Minimum value for colormap scaling (optional, uses data min if not provided)
        vmax: Maximum value for colormap scaling (optional, uses data max if not provided)

    Returns:
        List of lists of RGB tuples (0-255 range)
    """
    # Replace NaN values with zero
    data_clean = np.nan_to_num(data, nan=0.0)

    # Calculate data range
    if vmin is None:
        data_min = np.nanmin(data_clean)
    else:
        data_min = vmin
    
    if vmax is None:
        data_max = np.nanmax(data_clean)
    else:
        data_max = vmax

    # Normalize to [0, 1] range
    if data_max == data_min:
        # Constant data - use middle of colormap
        normalized = np.full_like(data_clean, 0.5)
    else:
        normalized = (data_clean - data_min) / (data_max - data_min)

    normalized = np.clip(normalized, 0.0, 1.0)

    # Map normalized values to colormap indices
    n_colors = len(VIRIDIS_COLORS)
    result: List[List[Tuple[int, int, int]]] = []

    for row in normalized:
        rgb_row: List[Tuple[int, int, int]] = []
        for val in row:
            # Map [0, 1] value to colormap index
            color_idx = int(val * (n_colors - 1))
            color_idx = max(0, min(color_idx, n_colors - 1))  # Clamp to valid range
            rgb_row.append(VIRIDIS_COLORS[color_idx])
        result.append(rgb_row)

    return result
