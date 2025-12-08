"""Utility functions for plot data preprocessing."""

from typing import Tuple
import numpy as np


def handle_nan_values_1d(data: np.ndarray) -> Tuple[np.ndarray, np.ndarray]:
    """Handle NaN values in 1D data by creating a mask.

    Args:
        data: 1D numpy array potentially containing NaN values

    Returns:
        Tuple of (cleaned_data, valid_mask)
    """
    if np.issubdtype(data.dtype, np.floating):
        mask = np.isfinite(data)
        if np.any(mask):
            return data.copy(), mask
        # No valid data, return small zero array
        fallback_data = np.zeros(min(10, len(data)))
        fallback_mask = np.ones(len(fallback_data), dtype=bool)
        return fallback_data, fallback_mask

    # Non-floating data, no NaN handling needed
    return data, np.ones(len(data), dtype=bool)


def handle_nan_values_2d(data: np.ndarray) -> np.ndarray:
    """Handle NaN values in 2D data by replacing with mean.

    Args:
        data: 2D numpy array potentially containing NaN values

    Returns:
        Numpy array with NaN values replaced
    """
    data = np.array(data, copy=True)

    if np.issubdtype(data.dtype, np.floating):
        nan_mask = np.isnan(data)
        if np.any(nan_mask):
            valid_mean = np.nanmean(data) if np.any(~nan_mask) else 0.0
            return np.where(nan_mask, valid_mean, data)
    else:
        data = data.astype(float)

    return data


def downsample_1d(data: np.ndarray, max_points: int) -> np.ndarray:
    """Downsample 1D data if it exceeds max_points."""
    if len(data) <= max_points:
        return data
    indices = np.linspace(0, len(data) - 1, max_points, dtype=int)
    return data[indices]


def downsample_2d(data: np.ndarray, max_dim: int) -> np.ndarray:
    """Downsample 2D data if it exceeds max_dim in either dimension."""
    rows, cols = data.shape
    if rows <= max_dim and cols <= max_dim:
        return data

    row_step = max(1, rows // max_dim)
    col_step = max(1, cols // max_dim)
    return data[::row_step, ::col_step]
