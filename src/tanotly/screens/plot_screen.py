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
from .plot import create_plot_widget, DataPlot1D, DataPlot2D
from .plot.colormap import VIRIDIS_COLORS
from .plot.constants import DOWNSAMPLE_2D_THRESHOLD
from .plot.utils import downsample_2d

logger = logging.getLogger(__name__)


# Colorbar utilities
def _create_colorbar_text(data: np.ndarray, vmin: float = None, vmax: float = None) -> str:
    """Create colorbar text with min/max values.
    
    Args:
        data: Data array (used for local min/max if vmin/vmax not provided)
        vmin: Global minimum value (optional)
        vmax: Global maximum value (optional)
    """
    if vmin is None:
        data_min = float(np.nanmin(data))
    else:
        data_min = vmin
    
    if vmax is None:
        data_max = float(np.nanmax(data))
    else:
        data_max = vmax

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
def _create_axis_controls(
    data: np.ndarray,
    dim_names: tuple,
    x_axis: int,
    y_axis: int,
    plot_type: str
) -> list:
    """Create axis selection controls for choosing which dimensions to plot.
    
    For Contour (2D): select X and Y axes from available dimensions.
    For Line (1D): select only the axis to plot along.
    """
    widgets = []
    ndim = data.ndim

    if ndim <= 1:
        return widgets

    # Create dimension options for axis selection
    dim_options = [(dim_names[i], i) for i in range(ndim)]

    if plot_type == "contour":
        # For 2D contour plots: X and Y axes
        widgets.append(Static("X axis:", classes="dim-label"))
        widgets.append(
            Select(
                dim_options,
                value=x_axis,
                id="x-axis-select",
                allow_blank=False,
            )
        )
        widgets.append(Static("Y axis:", classes="dim-label"))
        widgets.append(
            Select(
                dim_options,
                value=y_axis,
                id="y-axis-select",
                allow_blank=False,
            )
        )
    else:
        # For 1D line plots: only one axis to plot along
        widgets.append(Static("Axis:", classes="dim-label"))
        widgets.append(
            Select(
                dim_options,
                value=x_axis,
                id="x-axis-select",
                allow_blank=False,
            )
        )

    return widgets


def _create_slice_controls(
    data: np.ndarray,
    dim_names: tuple,
    slice_indices: list,
    x_axis: int,
    y_axis: int,
    plot_type: str
) -> list:
    """Create slice control widgets for dimensions not being plotted.
    
    For Contour (2D) plots: creates controls for all dimensions except X and Y axes.
    For Line (1D) plots: creates controls for all dimensions except the plot axis.
    """
    widgets = []
    ndim = data.ndim

    if ndim <= 1:
        return widgets

    # Find dimensions that are not being plotted
    if plot_type == "contour":
        # For 2D: exclude both X and Y axes
        plot_dims = {x_axis, y_axis}
    else:
        # For 1D: exclude only the plot axis
        plot_dims = {x_axis}
    
    non_plot_dims = [i for i in range(ndim) if i not in plot_dims]
    
    if not non_plot_dims:
        return widgets

    # Create controls for all non-plot dimensions
    for dim_idx in non_plot_dims:
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


def _build_slice_info(
    dim_names: tuple,
    slice_indices: list,
    x_axis: int,
    y_axis: int,
    ndim: int,
    plot_type: str
) -> str:
    """Build slice information text for footer.
    
    Shows which dimensions are plotted and which are sliced.
    Example: "Plot: lat × lon | Slice: time=0, level=5"
    """
    if ndim <= 1:
        return f"Plot: {dim_names[x_axis]}"
    
    # Build plot info based on plot type
    if plot_type == "line":
        plot_info = f"{dim_names[x_axis]} (1D)"
    else:
        plot_info = f"{dim_names[x_axis]} × {dim_names[y_axis]}"
    
    # Build slice info
    plot_dims = {x_axis}
    if plot_type == "contour" and y_axis is not None:
        plot_dims.add(y_axis)
    
    slice_parts = []
    for i in range(ndim):
        if i not in plot_dims:
            slice_parts.append(f"{dim_names[i]}={slice_indices[i]}")
    
    if slice_parts:
        return f"Plot: {plot_info} | Slice: {', '.join(slice_parts)}"
    else:
        return f"Plot: {plot_info}"


class PlotScreen(ModalScreen[None]):
    """Modal screen for displaying plots with dimension slicing controls."""

    BINDINGS = [
        Binding("escape", "close", "Close"),
        Binding("q", "close", "Close"),
        Binding("p", "toggle_view", "Toggle Plot/Array"),
        Binding("d", "toggle_view", "Toggle Plot/Array"),
        Binding("[", "decrement_slice", "Prev Slice", show=False),
        Binding("]", "increment_slice", "Next Slice", show=False),
        Binding("x", "transpose", "Transpose", show=False),
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
        
        # Calculate global min/max for consistent colormap scaling
        if np.issubdtype(data.dtype, np.number):
            self._global_vmin = float(np.nanmin(data))
            self._global_vmax = float(np.nanmax(data))
        else:
            self._global_vmin = None
            self._global_vmax = None
        
        # Plot type: "contour" (2D) or "line" (1D)
        self._plot_type = "contour" if data.ndim >= 2 else "line"
        
        # Track which dimensions are used for X and Y axes
        # For 1D data: only X axis is used
        # For 2D+ data: X and Y axes default to last two dimensions
        if data.ndim >= 2:
            self._x_axis = data.ndim - 1  # Last dimension (e.g., lon)
            self._y_axis = data.ndim - 2  # Second-to-last dimension (e.g., lat)
        else:
            self._x_axis = 0
            self._y_axis = None
    
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
        """Get the 2D or 1D slice to display based on current plot type and axes.

        For Line (1D) plots: extracts data along the X axis.
        For Contour (2D) plots: extracts the X and Y dimensions.
        For 3D+ arrays: slices other dimensions at their current index.
        """
        data = self._original_data
        ndim = data.ndim

        if ndim == 1:
            return data
        
        # For Line (1D) plots: extract 1D slice along X axis
        if self._plot_type == "line":
            indices = []
            for dim in range(ndim):
                if dim == self._x_axis:
                    # Keep full extent for X axis
                    indices.append(slice(None))
                else:
                    # Use the slice index for all other dimensions
                    idx = self._slice_indices[dim]
                    idx = max(0, min(idx, data.shape[dim] - 1))
                    indices.append(idx)
            
            sliced = data[tuple(indices)]
            return np.asarray(sliced)
        
        # For Contour (2D) plots
        if ndim == 2:
            # For 2D, transpose if needed to match X/Y axis selection
            if self._x_axis == 0 and self._y_axis == 1:
                return data  # Y (rows) × X (cols) - standard orientation
            else:
                return data.T  # Transpose to match axis selection
        
        # For 3D+: build indices to extract the X and Y dimensions
        indices = []
        for dim in range(ndim):
            if dim == self._x_axis or dim == self._y_axis:
                # Keep full extent for plot dimensions
                indices.append(slice(None))
            else:
                # Use the slice index for non-plot dimensions
                idx = self._slice_indices[dim]
                idx = max(0, min(idx, data.shape[dim] - 1))
                indices.append(idx)

        # Extract the slice
        sliced = data[tuple(indices)]
        sliced = np.asarray(sliced)
        
        # Transpose if needed so Y is rows and X is columns
        if sliced.ndim == 2:
            remaining_dims = [d for d in range(ndim) if d == self._x_axis or d == self._y_axis]
            if remaining_dims[0] == self._x_axis:
                sliced = sliced.T
        
        return sliced

    def compose(self) -> ComposeResult:
        """Compose the plot interface."""
        data = self._get_display_data()
        is_2d = data.ndim >= 2
        ndim = self._original_data.ndim

        with Container(id="plot-container"):
            # Header with title
            yield Static(f" {self._title}", id="plot-title")

            # Controls for 2D+ data
            if ndim >= 2:
                with Horizontal(id="controls-row"):
                    # Plot type selector (Contour/Line)
                    yield Static("Plot type:", classes="dim-label")
                    yield Select(
                        [("Contour (2D)", "contour"), ("Line (1D)", "line")],
                        value=self._plot_type,
                        id="plot-type-select",
                        allow_blank=False,
                    )
                    
                    # Axis selection controls
                    axis_widgets = _create_axis_controls(
                        self._original_data,
                        self._dim_names,
                        self._x_axis,
                        self._y_axis,
                        self._plot_type
                    )
                    for widget in axis_widgets:
                        yield widget
                    
                    # Slice controls (dimensions not being plotted)
                    slice_widgets = _create_slice_controls(
                        self._original_data,
                        self._dim_names,
                        self._slice_indices,
                        self._x_axis,
                        self._y_axis,
                        self._plot_type
                    )
                    for widget in slice_widgets:
                        yield widget

            # Tabbed content for Plot vs Array view
            initial = "tab-plot" if self._initial_tab == "plot" else "tab-array"
            with TabbedContent(id="view-tabs", initial=initial):
                with TabPane("Plot", id="tab-plot"):
                    with Container(id="plot-view"):
                        # Centered plot container
                        with Center():
                            with Vertical(id="plot-content"):
                                # Plot widget with global min/max for consistent scaling
                                plot_widget = create_plot_widget(
                                    data,
                                    vmin=self._global_vmin,
                                    vmax=self._global_vmax
                                )
                                yield plot_widget

                                # Colorbar legend for 2D data
                                if is_2d:
                                    yield Static(
                                        _create_colorbar_text(data, self._global_vmin, self._global_vmax),
                                        id="colorbar-legend"
                                    )

                with TabPane("Array", id="tab-array"):
                    with Container(id="table-view"):
                        yield DataTable(id="array-table")

            # Footer
            yield Static(self._build_footer_text(), id="plot-footer")

    def _build_footer_text(self) -> str:
        """Build footer text."""
        ndim = self._original_data.ndim
        if ndim <= 1:
            return " [q/Esc] Close  [Tab] Toggle Plot↔Table "
        else:
            slice_info = _build_slice_info(
                self._dim_names,
                self._slice_indices,
                self._x_axis,
                self._y_axis,
                ndim,
                self._plot_type
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

            # Get the plot content container
            try:
                plot_content = self.query_one("#plot-content", Vertical)
            except Exception as e:
                logger.error(f"Failed to find plot content container: {e}")
                self.loading = False
                return

            # Check if we can update existing widget (optimization for both 1D and 2D plots)
            should_replace = True
            try:
                existing_widget = self.query_one("#plot-widget")
                
                # Update 2D plot in place
                if isinstance(existing_widget, DataPlot2D) and data.ndim == 2:
                    # Downsample if needed
                    plot_data = downsample_2d(data, DOWNSAMPLE_2D_THRESHOLD)
                    
                    # Update existing widget
                    existing_widget.update_data(plot_data)
                    
                    # Update colorbar with global min/max
                    try:
                        colorbar = self.query_one("#colorbar-legend", Static)
                        colorbar.update(_create_colorbar_text(data, self._global_vmin, self._global_vmax))
                    except Exception as e:
                        logger.debug(f"Failed to update colorbar: {e}")
                    
                    should_replace = False
                
                # Update 1D plot in place
                elif isinstance(existing_widget, DataPlot1D) and data.ndim == 1:
                    # Update existing widget
                    existing_widget.update_data(data)
                    should_replace = False
                    
            except Exception as e:
                logger.debug(f"Could not update existing widget, will replace: {e}")

            # Replace widget if needed (for 1D plots or when update fails)
            if should_replace:
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

                # Create and mount new plot widget with global min/max
                plot_widget = create_plot_widget(
                    data,
                    vmin=self._global_vmin,
                    vmax=self._global_vmax
                )
                await plot_content.mount(plot_widget)

                # Mount new colorbar for 2D data with global min/max
                if data.ndim >= 2:
                    new_colorbar = Static(
                        _create_colorbar_text(data, self._global_vmin, self._global_vmax),
                        id="colorbar-legend"
                    )
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

    async def _refresh_controls_async(self) -> None:
        """Refresh the axis and slice controls when plot type or axes change."""
        if self._original_data.ndim < 2:
            return
        
        try:
            controls_row = self.query_one("#controls-row", Horizontal)
            
            logger.debug(f"Refreshing controls. Current plot type: {self._plot_type}")
            
            # Remove all controls except plot type (first 2 widgets)
            all_children = list(controls_row.children)
            for widget in all_children[2:]:
                try:
                    widget.remove()
                except Exception as e:
                    logger.debug(f"Failed to remove widget: {e}")
            
            # Create and mount new axis controls
            axis_control_widgets = _create_axis_controls(
                self._original_data,
                self._dim_names,
                self._x_axis,
                self._y_axis,
                self._plot_type
            )
            
            for widget in axis_control_widgets:
                controls_row.mount(widget)
            
            # Create and mount new slice controls
            slice_control_widgets = _create_slice_controls(
                self._original_data,
                self._dim_names,
                self._slice_indices,
                self._x_axis,
                self._y_axis,
                self._plot_type
            )
            
            for widget in slice_control_widgets:
                controls_row.mount(widget)
            
            logger.debug(f"Mounted {len(axis_control_widgets)} axis controls and {len(slice_control_widgets)} slice controls")
            
        except Exception as e:
            logger.exception(f"Failed to refresh controls: {e}")

    def on_select_changed(self, event: Select.Changed) -> None:
        """Handle all select control changes."""
        select_id = event.select.id
        if not select_id:
            return
        
        # Handle plot type selection changes
        if select_id == "plot-type-select":
            new_value = event.value
            if new_value is not None and new_value != self._plot_type:
                old_plot_type = self._plot_type
                self._plot_type = new_value
                
                logger.info(f"Plot type changed from {old_plot_type} to {new_value}")
                
                # Refresh controls, plot, and table sequentially
                async def refresh_all():
                    await self._refresh_controls_async()
                    await self._render_plot_async()
                    await self._populate_table_async()
                    self._update_footer()
                
                self.run_worker(refresh_all(), exclusive=True, name="refresh_all")
            return
        
        # Handle axis selection changes
        if select_id == "x-axis-select":
            new_value = event.value
            if new_value is not None:
                old_x_axis = self._x_axis
                self._x_axis = int(new_value)
                
                if old_x_axis != self._x_axis:
                    # Refresh plot and table
                    self.run_worker(self._render_plot_async(), exclusive=True, name="render_plot")
                    self.run_worker(self._populate_table_async(), exclusive=True, name="populate_table")
                    
                    # Refresh slice controls if needed
                    self.run_worker(self._refresh_controls_async(), exclusive=False, name="refresh_controls")
                    
                    self._update_footer()
            return
        
        if select_id == "y-axis-select":
            new_value = event.value
            if new_value is not None:
                old_y_axis = self._y_axis
                self._y_axis = int(new_value)
                
                if old_y_axis != self._y_axis:
                    # Refresh plot and table
                    self.run_worker(self._render_plot_async(), exclusive=True, name="render_plot")
                    self.run_worker(self._populate_table_async(), exclusive=True, name="populate_table")
                    
                    # Refresh slice controls if needed
                    self.run_worker(self._refresh_controls_async(), exclusive=False, name="refresh_controls")
                    
                    self._update_footer()
            return
        
        # Handle slice selection changes
        if select_id.startswith("slice-"):
            dim_idx = int(select_id.split("-")[1])
            if dim_idx < len(self._slice_indices):
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

    def action_transpose(self) -> None:
        """Swap X and Y axes (only for Contour plots)."""
        if self._original_data.ndim < 2 or self._plot_type != "contour":
            return  # Transpose only works for contour plots
        
        # Swap the axes
        self._x_axis, self._y_axis = self._y_axis, self._x_axis
        
        # Update the Select widgets to reflect the swap
        try:
            x_select = self.query_one("#x-axis-select", Select)
            y_select = self.query_one("#y-axis-select", Select)
            x_select.value = self._x_axis
            y_select.value = self._y_axis
        except Exception as e:
            logger.debug(f"Failed to update axis select widgets: {e}")
        
        # Re-render the plot and table
        self.run_worker(self._render_plot_async(), exclusive=True, name="render_plot")
        self.run_worker(self._populate_table_async(), exclusive=True, name="populate_table")
        self._update_footer()
