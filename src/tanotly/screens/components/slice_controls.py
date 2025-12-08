"""Slice controls for multi-dimensional data navigation."""

import numpy as np
from rich.text import Text
from textual.widgets import Static, Select
from textual.containers import Horizontal


class SliceControls:
    """Creates dimension slicing controls for 3D+ data."""

    @staticmethod
    def create_controls(
        data: np.ndarray,
        dim_names: tuple,
        slice_indices: list[int]
    ) -> list:
        """Create slice control widgets for 3D+ data.

        Args:
            data: Original data array
            dim_names: Names of dimensions
            slice_indices: Current slice indices for each dimension

        Returns:
            List of widgets to mount in a Horizontal container
        """
        widgets = []
        ndim = data.ndim

        if ndim <= 2:
            return widgets

        widgets.append(Static("Slice:", classes="dim-label"))

        for dim_idx in range(ndim - 2):
            dim_name = dim_names[dim_idx]
            dim_size = data.shape[dim_idx]

            widgets.append(Static(f"{dim_name}:", classes="dim-label"))
            # Create options with Text objects for proper rendering
            options = [(Text(str(j)), j) for j in range(dim_size)]
            widgets.append(
                Select(
                    options,
                    value=slice_indices[dim_idx],
                    id=f"slice-{dim_idx}",
                    allow_blank=False,
                )
            )

        return widgets

    @staticmethod
    def build_slice_info_text(
        dim_names: tuple,
        slice_indices: list[int],
        ndim: int
    ) -> str:
        """Build slice information text for footer.

        Args:
            dim_names: Names of dimensions
            slice_indices: Current slice indices
            ndim: Number of dimensions

        Returns:
            Formatted slice information string
        """
        if ndim <= 2:
            return ""

        slice_info = ", ".join(
            f"{dim_names[i]}={slice_indices[i]}"
            for i in range(ndim - 2)
        )
        return f"Slice: {slice_info}"
