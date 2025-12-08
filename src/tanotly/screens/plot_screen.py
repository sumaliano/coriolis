"""Plot and data table screen for Tanotly.

Provides a modal screen for visualizing variable data with:
- 2D heatmap plots with viridis colormap
- 1D line plots
- Dimension slicing controls for 3D+ data
- Data table view with cell navigation
"""

import logging
import numpy as np

from textual.app import ComposeResult
from textual.binding import Binding
from textual.containers import Container, Horizontal, Vertical, Center
from textual.screen import ModalScreen
from textual.widgets import Static, DataTable, TabbedContent, TabPane, Select
from textual.reactive import reactive

from ..config import Colors
from .plot import create_plot_widget, DataPlot2D
from .plot.colormap import VIRIDIS_COLORS
from .plot.constants import DOWNSAMPLE_2D_THRESHOLD
from .plot.utils import downsample_2d

logger = logging.getLogger(__name__)


# Colorbar utilities
def _create_colorbar_text(data: np.ndarray) -> str:
    """Create colorbar text with min/max values."""
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


# Data table utilities
def _format_table_value(val) -> str:
    """Format a value for display in the table."""
    if isinstance(val, float):
        if np.isnan(val):
            return "NaN"
        if abs(val) >= 1e6 or (abs(val) < 1e-3 and val != 0):
            return f"{val:.3e}"
        return f"{val:.4g}"
    return str(val)


async def _populate_data_table(
    table: DataTable,
    data: np.ndarray,
    dim_names: tuple = (),
    max_rows: int = 500,
    max_cols: int = 50
) -> None:
    """Populate the array data table."""
    table.cursor_type = "cell"
    table.zebra_stripes = True

    if table.row_count > 0:
        table.clear(columns=True)

    if data.ndim == 1:
        table.add_column("Index", width=8)
        table.add_column("Value", width=20)
        for i, val in enumerate(data[:max_rows]):
            table.add_row(str(i), _format_table_value(val))
        if len(data) > max_rows:
            table.add_row("...", f"({len(data) - max_rows} more)")

    elif data.ndim == 2:
        rows, cols = data.shape
        display_cols = min(cols, max_cols)
        display_rows = min(rows, max_rows)

        # Get the row dimension name
        ndim = len(dim_names)
        row_dim_name = dim_names[ndim - 2] if ndim >= 2 else "row"

        table.add_column(row_dim_name, width=8)
        for j in range(display_cols):
            table.add_column(str(j), width=10)
        if cols > max_cols:
            table.add_column("...", width=5)

        for i in range(display_rows):
            row = [str(i)]
            for j in range(display_cols):
                row.append(_format_table_value(data[i, j]))
            if cols > max_cols:
                row.append("...")
            table.add_row(*row)

        if rows > max_rows:
            table.add_row("...", *["..."] * (display_cols + (1 if cols > max_cols else 0)))


# Slice control utilities
def _create_slice_controls(
    data: np.ndarray,
    dim_names: tuple,
    slice_indices: list
) -> list:
    """Create slice control widgets for 3D+ data."""
    widgets = []
    ndim = data.ndim

    if ndim <= 2:
        return widgets

    widgets.append(Static("Slice:", classes="dim-label"))

    for dim_idx in range(ndim - 2):
        dim_name = dim_names[dim_idx]
        dim_size = data.shape[dim_idx]

        widgets.append(Static(f"{dim_name}:", classes="dim-label"))
        # Create options as (prompt, value) tuples with string prompts
        options = [(str(j), j) for j in range(dim_size)]
        widgets.append(
            Select(
                options,
                value=slice_indices[dim_idx],
                id=f"slice-{dim_idx}",
                allow_blank=False,
            )
        )

    return widgets


def _build_slice_info(dim_names: tuple, slice_indices: list, ndim: int) -> str:
    """Build slice information text for footer."""
    if ndim <= 2:
        return ""

    slice_info = ", ".join(
        f"{dim_names[i]}={slice_indices[i]}"
        for i in range(ndim - 2)
    )
    return f"Slice: {slice_info}"


class PlotScreen(ModalScreen[None]):
    """Modal screen for displaying plots with dimension slicing controls."""

    BINDINGS = [
        Binding("escape", "close", "Close"),
        Binding("q", "close", "Close"),
        Binding("p", "toggle_view", "Toggle Plot/Array"),
        Binding("d", "toggle_view", "Toggle Plot/Array"),
        Binding("[", "decrement_slice", "Prev Slice", show=False),
        Binding("]", "increment_slice", "Next Slice", show=False),
    ]

    loading: reactive[bool] = reactive(False)

    def __init__(
        self,
        data: np.ndarray,
        var_name: str = "data",
        dim_names: tuple = (),
        initial_tab: str = "plot"
    ):
        super().__init__()
        self._original_data = data
        self._var_name = var_name
        self._dim_names = dim_names or tuple(f"dim{i}" for i in range(data.ndim))
        self._initial_tab = initial_tab
        self._slice_indices = [0] * data.ndim

    @property
    def _title(self) -> str:
        """Build title in tree-style format: name (dims) [nD] dtype"""
        shape = self._original_data.shape
        dtype = str(self._original_data.dtype)
        ndim = self._original_data.ndim

        # Variable name in accent color
        parts = [f"[{Colors.variable()}]{self._var_name}[/]"]

        # Dimension info: (dim1=size1, dim2=size2, ...)
        if self._dim_names and shape:
            dim_str = ", ".join(f"{d}={s}" for d, s in zip(self._dim_names, shape))
            parts.append(f"[{Colors.muted()}]({dim_str})[/]")
        elif shape:
            dim_str = "×".join(str(s) for s in shape)
            parts.append(f"[{Colors.muted()}]({dim_str})[/]")

        # Dimensionality label [nD]
        if ndim > 0:
            parts.append(f"[{Colors.muted()}]\\[{ndim}D][/]")

        # Data type
        parts.append(f"[{Colors.muted()}]{dtype}[/]")

        return " ".join(parts)

    def watch_loading(self, is_loading: bool) -> None:
        """React to loading state changes."""
        self.set_class(is_loading, "loading")

    def _get_display_data(self) -> np.ndarray:
        """Get the 2D or 1D slice to display based on current slice indices.

        For 3D+ arrays, slices along the first N-2 dimensions to get a 2D array.
        """
        data = self._original_data
        ndim = data.ndim

        if ndim <= 2:
            return data

        # For 3D+: take last 2 dimensions fully, slice others at current position
        indices = []
        for dim in range(ndim):
            if dim < ndim - 2:
                # Use the slice index for this dimension
                idx = self._slice_indices[dim]
                # Clamp to valid range
                idx = max(0, min(idx, data.shape[dim] - 1))
                indices.append(idx)
            else:
                # Keep full extent for last 2 dimensions
                indices.append(slice(None))

        # Extract the slice
        sliced = data[tuple(indices)]

        # Ensure it's a proper numpy array (not a view that might cause issues)
        sliced = np.asarray(sliced)

        return sliced

    def compose(self) -> ComposeResult:
        """Compose the plot interface."""
        data = self._get_display_data()
        is_2d = data.ndim >= 2
        ndim = self._original_data.ndim

        with Container(id="plot-container"):
            # Header with title
            yield Static(f" {self._title}", id="plot-title")

            # Slice controls for 3D+ data
            if ndim > 2:
                with Horizontal(id="controls-row"):
                    control_widgets = _create_slice_controls(
                        self._original_data,
                        self._dim_names,
                        self._slice_indices
                    )
                    for widget in control_widgets:
                        yield widget

            # Tabbed content for Plot vs Array view
            initial = "tab-plot" if self._initial_tab == "plot" else "tab-array"
            with TabbedContent(id="view-tabs", initial=initial):
                with TabPane("Plot", id="tab-plot"):
                    with Container(id="plot-view"):
                        # Centered plot container
                        with Center():
                            with Vertical(id="plot-content"):
                                # Plot widget
                                plot_widget = create_plot_widget(data)
                                yield plot_widget

                                # Colorbar legend for 2D data
                                if is_2d:
                                    yield Static(_create_colorbar_text(data), id="colorbar-legend")

                with TabPane("Array", id="tab-array"):
                    with Container(id="table-view"):
                        yield DataTable(id="array-table")

            # Footer
            yield Static(self._build_footer_text(), id="plot-footer")

    def _build_footer_text(self) -> str:
        """Build footer text."""
        ndim = self._original_data.ndim
        if ndim <= 2:
            return " [q/Esc] Close  [Tab] Toggle Plot↔Table "
        else:
            slice_info = _build_slice_info(self._dim_names, self._slice_indices, ndim)
            return f" [q/Esc] Close  [Tab] Toggle | {slice_info} "

    def on_mount(self) -> None:
        """Initialize the screen after mounting."""
        self.run_worker(self._populate_table_async(), exclusive=True, name="populate_table")

    async def _render_plot_async(self) -> None:
        """Render plot asynchronously after slice change."""
        try:
            self.loading = True
            data = self._get_display_data()

            # Try to update existing plot widget if it's a DataPlot2D
            try:
                existing_widget = self.query_one("#plot-widget")
                if isinstance(existing_widget, DataPlot2D) and data.ndim == 2:
                    # Downsample if needed using utility function
                    plot_data = downsample_2d(data, DOWNSAMPLE_2D_THRESHOLD)

                    # Update existing widget
                    existing_widget.update_data(plot_data)

                    # Update colorbar
                    try:
                        colorbar = self.query_one("#colorbar-legend", Static)
                        colorbar.update(_create_colorbar_text(data))
                    except Exception as e:
                        logger.debug(f"Failed to update colorbar: {e}")

                    self.loading = False
                    return
            except Exception as e:
                logger.debug(f"Failed to update existing plot widget: {e}")

            # Fall back to replacing the widget
            plot_widget = create_plot_widget(data)

            # Get the plot content container
            try:
                plot_content = self.query_one("#plot-content", Vertical)
            except Exception as e:
                logger.error(f"Failed to find plot content container: {e}")
                self.loading = False
                return

            # Remove old plot widget
            try:
                old_widget = self.query_one("#plot-widget")
                await old_widget.remove()
            except Exception as e:
                logger.debug(f"No old plot widget to remove: {e}")

            # Remove old colorbar
            try:
                old_colorbar = self.query_one("#colorbar-legend", Static)
                await old_colorbar.remove()
            except Exception as e:
                logger.debug(f"No old colorbar to remove: {e}")

            # Mount new plot widget
            await plot_content.mount(plot_widget)

            # Mount new colorbar for 2D data
            if data.ndim >= 2:
                new_colorbar = Static(_create_colorbar_text(data), id="colorbar-legend")
                await plot_content.mount(new_colorbar)

        except Exception as e:
            logger.exception(f"Failed to render plot: {e}")
        finally:
            self.loading = False

    async def _populate_table_async(self) -> None:
        """Populate the array data table."""
        try:
            table = self.query_one("#array-table", DataTable)
            data = self._get_display_data()
            await _populate_data_table(table, data, self._dim_names)
        except Exception as e:
            logger.exception(f"Failed to populate table: {e}")

    def on_select_changed(self, event: Select.Changed) -> None:
        """Handle slice selection changes."""
        select_id = event.select.id
        if not select_id or not select_id.startswith("slice-"):
            return

        dim_idx = int(select_id.split("-")[1])
        if dim_idx < len(self._slice_indices):
            # Ensure value is an integer
            new_value = event.value
            if new_value is not None:
                self._slice_indices[dim_idx] = int(new_value)
                self.run_worker(self._render_plot_async(), exclusive=True, name="render_plot")
                self.run_worker(self._populate_table_async(), exclusive=True, name="populate_table")
                self._update_footer()

    def _update_footer(self) -> None:
        """Update footer with current slice information."""
        try:
            footer = self.query_one("#plot-footer", Static)
            footer.update(self._build_footer_text())
        except Exception as e:
            logger.debug(f"Failed to update footer: {e}")

    def action_close(self) -> None:
        """Close the modal."""
        self.dismiss()

    def action_toggle_view(self) -> None:
        """Toggle between Plot and Array views."""
        try:
            tabs = self.query_one(TabbedContent)
            if tabs.active == "tab-plot":
                tabs.active = "tab-array"
            else:
                tabs.active = "tab-plot"
        except Exception as e:
            logger.debug(f"Failed to toggle view: {e}")

    def action_decrement_slice(self) -> None:
        """Decrement the value of the focused Select widget (or first one if none focused)."""
        if self._original_data.ndim <= 2:
            return  # No slice controls for 1D/2D data

        # Try to find a focused Select widget
        try:
            focused = self.focused
            if isinstance(focused, Select) and focused.id and focused.id.startswith("slice-"):
                select_widget = focused
            else:
                # If no Select is focused, use the first one
                select_widget = self.query_one("#slice-0", Select)
        except Exception:
            return

        # Get current value and decrement
        current_value = select_widget.value
        if current_value is not None and current_value > 0:
            select_widget.value = current_value - 1

    def action_increment_slice(self) -> None:
        """Increment the value of the focused Select widget (or first one if none focused)."""
        if self._original_data.ndim <= 2:
            return  # No slice controls for 1D/2D data

        # Try to find a focused Select widget
        try:
            focused = self.focused
            if isinstance(focused, Select) and focused.id and focused.id.startswith("slice-"):
                select_widget = focused
            else:
                # If no Select is focused, use the first one
                select_widget = self.query_one("#slice-0", Select)
        except Exception:
            return

        # Get current value and dimension size
        current_value = select_widget.value
        if current_value is not None:
            # Extract dimension index from widget id
            dim_idx = int(select_widget.id.split("-")[1])
            max_value = self._original_data.shape[dim_idx] - 1
            
            if current_value < max_value:
                select_widget.value = current_value + 1
