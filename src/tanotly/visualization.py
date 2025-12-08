"""Data formatting utilities for visualization.

Provides formatting functions for statistics and sample values display.
"""

import numpy as np


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
