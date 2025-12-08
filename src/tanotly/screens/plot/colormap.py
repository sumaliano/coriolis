"""Colormap utilities for data visualization.

Provides viridis-like perceptually uniform, colorblind-friendly colormap.
"""

import numpy as np


# Viridis-like colormap (perceptually uniform, colorblind-friendly)
VIRIDIS_COLORS = [
    (68, 1, 84), (72, 26, 108), (71, 47, 125), (65, 68, 135), (57, 86, 140),
    (49, 104, 142), (42, 120, 142), (35, 136, 142), (31, 152, 139), (34, 168, 132),
    (53, 183, 121), (83, 198, 105), (122, 209, 81), (165, 219, 54), (210, 226, 27),
    (253, 231, 37),
]


def apply_colormap(data: np.ndarray) -> list[list[tuple[int, int, int]]]:
    """Apply viridis colormap to normalized data, returning RGB tuples.

    Args:
        data: 2D numpy array to apply colormap to

    Returns:
        List of lists of RGB tuples (0-255 range)
    """
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
