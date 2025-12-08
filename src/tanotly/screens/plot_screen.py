"""Plot and data table screen for Tanotly.

Provides a modal screen for visualizing variable data with:
- 2D heatmap plots with viridis colormap
- 1D line plots
- Dimension slicing controls for 3D+ data
- Data table view with cell navigation
"""

import numpy as np

from textual.app import ComposeResult
from textual.binding import Binding
from textual.containers import Container, Horizontal, Vertical, Center
from textual.screen import ModalScreen
from textual.widgets import Static, DataTable, TabbedContent, TabPane, Select
from textual.reactive import reactive

from ..config import Colors
from .plot import PlotRenderer, DataPlot2D
from .components import ColorbarLegend, ArrayDataTable, SliceControls


class PlotScreen(ModalScreen[None]):
    """Modal screen for displaying plots with dimension slicing controls."""

    BINDINGS = [
        Binding("escape", "close", "Close"),
        Binding("q", "close", "Close"),
        Binding("p", "toggle_view", "Toggle Plot/Array"),
        Binding("d", "toggle_view", "Toggle Plot/Array"),
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
                    control_widgets = SliceControls.create_controls(
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
                                plot_widget = PlotRenderer.create_plot_widget(data)
                                yield plot_widget

                                # Colorbar legend for 2D data
                                if is_2d:
                                    yield ColorbarLegend.create_widget(data)

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
            slice_info = SliceControls.build_slice_info_text(
                self._dim_names,
                self._slice_indices,
                ndim
            )
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
                    # Downsample if needed
                    rows, cols = data.shape
                    max_dim = 100
                    if rows > max_dim or cols > max_dim:
                        row_step = max(1, rows // max_dim)
                        col_step = max(1, cols // max_dim)
                        plot_data = data[::row_step, ::col_step]
                    else:
                        plot_data = data

                    # Update existing widget
                    existing_widget.update_data(plot_data)

                    # Update colorbar
                    try:
                        colorbar = self.query_one("#colorbar-legend", Static)
                        colorbar.update(ColorbarLegend.create_colorbar_text(data))
                    except Exception:
                        pass

                    self.loading = False
                    return
            except Exception:
                pass

            # Fall back to replacing the widget
            plot_widget = PlotRenderer.create_plot_widget(data)

            # Get the plot content container
            try:
                plot_content = self.query_one("#plot-content", Vertical)
            except Exception:
                self.loading = False
                return

            # Remove old plot widget
            try:
                old_widget = self.query_one("#plot-widget")
                await old_widget.remove()
            except Exception:
                pass

            # Remove old colorbar
            try:
                old_colorbar = self.query_one("#colorbar-legend", Static)
                await old_colorbar.remove()
            except Exception:
                pass

            # Mount new plot widget
            await plot_content.mount(plot_widget)

            # Mount new colorbar for 2D data
            if data.ndim >= 2:
                new_colorbar = ColorbarLegend.create_widget(data)
                await plot_content.mount(new_colorbar)

        except Exception as e:
            # Could add logging here if needed
            import traceback
            traceback.print_exc()
        finally:
            self.loading = False

    async def _populate_table_async(self) -> None:
        """Populate the array data table."""
        try:
            table = self.query_one("#array-table", DataTable)
            data = self._get_display_data()
            await ArrayDataTable.populate_table(
                table,
                data,
                self._dim_names
            )
        except Exception:
            pass

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
        except Exception:
            pass

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
        except Exception:
            pass
