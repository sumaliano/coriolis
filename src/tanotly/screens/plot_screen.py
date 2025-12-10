"""Modal screen for visualizing variable data.

Features:
- 2D heatmap plots with viridis colormap
- 1D line plots
- Dimension slicing controls for 3D+ data
- Data table view with cell navigation
- Dynamic axis selection and plot type switching
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


# ============================================================================
# COLORBAR UTILITIES
# ============================================================================

def _format_value(value: float) -> str:
    """Format a numeric value for display.
    
    Args:
        value: Numeric value to format
        
    Returns:
        Formatted string (scientific notation for very large/small values)
    """
    if abs(value) >= 1e4 or (abs(value) < 1e-3 and value != 0):
        return f"{value:.2e}"
    return f"{value:.3g}"


def _create_colorbar_text(
    data: np.ndarray,
    vmin: float = None,
    vmax: float = None
) -> str:
    """Create colorbar text with gradient and min/max values.
    
    Args:
        data: Data array (used for local min/max if vmin/vmax not provided)
        vmin: Global minimum value (optional)
        vmax: Global maximum value (optional)
        
    Returns:
        Rich-formatted colorbar string
    """
    # Determine min/max values
    data_min = vmin if vmin is not None else float(np.nanmin(data))
    data_max = vmax if vmax is not None else float(np.nanmax(data))

    # Build colorbar gradient using viridis colors
    n_colors = 16
    color_blocks = []
    for i in range(n_colors):
        color_idx = int(i * (len(VIRIDIS_COLORS) - 1) / (n_colors - 1))
        r, g, b = VIRIDIS_COLORS[color_idx]
        color_blocks.append(f"[rgb({r},{g},{b})]█[/]")

    colorbar = "".join(color_blocks)
    return f"{_format_value(data_min)} {colorbar} {_format_value(data_max)}"


# ============================================================================
# DATA TABLE UTILITIES
# ============================================================================

def _format_table_value(val) -> str:
    """Format a value for display in the data table.
    
    Args:
        val: Value to format (any type)
        
    Returns:
        Formatted string representation
    """
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
    """Populate the data table with array values.
    
    Args:
        table: DataTable widget to populate
        data: 1D or 2D numpy array
        dim_names: Dimension names for column headers
        max_rows: Maximum number of rows to display
        max_cols: Maximum number of columns to display
    """
    table.cursor_type = "cell"
    table.zebra_stripes = True

    # Clear existing data
    if table.row_count > 0:
        table.clear(columns=True)

    if data.ndim == 1:
        # 1D data: Index and Value columns
        table.add_column("Index", width=8)
        table.add_column("Value", width=20)
        for i, val in enumerate(data[:max_rows]):
            table.add_row(str(i), _format_table_value(val))
        if len(data) > max_rows:
            table.add_row("...", f"({len(data) - max_rows} more)")

    elif data.ndim == 2:
        # 2D data: Row index + column values
        rows, cols = data.shape
        display_cols = min(cols, max_cols)
        display_rows = min(rows, max_rows)

        # Get row dimension name
        ndim = len(dim_names)
        row_dim_name = dim_names[ndim - 2] if ndim >= 2 else "row"

        # Add columns
        table.add_column(row_dim_name, width=8)
        for j in range(display_cols):
            table.add_column(str(j), width=10)
        if cols > max_cols:
            table.add_column("...", width=5)

        # Add rows
        for i in range(display_rows):
            row = [str(i)]
            for j in range(display_cols):
                row.append(_format_table_value(data[i, j]))
            if cols > max_cols:
                row.append("...")
            table.add_row(*row)

        # Add truncation indicator
        if rows > max_rows:
            truncation_row = ["..."] * (display_cols + 1 + (1 if cols > max_cols else 0))
            table.add_row(*truncation_row)


# ============================================================================
# CONTROL WIDGET UTILITIES
# ============================================================================

def _create_axis_controls(
    data: np.ndarray,
    dim_names: tuple,
    x_axis: int,
    y_axis: int,
    plot_type: str
) -> list:
    """Create axis selection controls for plot dimensions.
    
    Args:
        data: Data array
        dim_names: Dimension names
        x_axis: Current X axis index
        y_axis: Current Y axis index
        plot_type: "contour" (2D) or "line" (1D)
        
    Returns:
        List of widgets for axis selection
    """
    widgets = []
    ndim = data.ndim

    if ndim <= 1:
        return widgets

    # Create dimension options
    dim_options = [(dim_names[i], i) for i in range(ndim)]

    if plot_type == "contour":
        # 2D contour: X and Y axis selectors
        widgets.extend([
            Static("X axis:", classes="dim-label"),
            Select(dim_options, value=x_axis, id="x-axis-select", allow_blank=False),
            Static("Y axis:", classes="dim-label"),
            Select(dim_options, value=y_axis, id="y-axis-select", allow_blank=False),
        ])
    else:
        # 1D line: only X axis selector
        widgets.extend([
            Static("Axis:", classes="dim-label"),
            Select(dim_options, value=x_axis, id="x-axis-select", allow_blank=False),
        ])

    return widgets


def _create_slice_controls(
    data: np.ndarray,
    dim_names: tuple,
    slice_indices: list,
    x_axis: int,
    y_axis: int,
    plot_type: str
) -> list:
    """Create slice controls for non-plotted dimensions.
    
    Args:
        data: Data array
        dim_names: Dimension names
        slice_indices: Current slice indices
        x_axis: X axis index
        y_axis: Y axis index
        plot_type: "contour" (2D) or "line" (1D)
        
    Returns:
        List of widgets for slice selection
    """
    widgets = []
    ndim = data.ndim

    if ndim <= 1:
        return widgets

    # Determine which dimensions are being plotted
    plot_dims = {x_axis, y_axis} if plot_type == "contour" else {x_axis}
    non_plot_dims = [i for i in range(ndim) if i not in plot_dims]
    
    if not non_plot_dims:
        return widgets

    # Create slice controls for each non-plotted dimension
    for dim_idx in non_plot_dims:
        dim_name = dim_names[dim_idx]
        dim_size = data.shape[dim_idx]
        options = [(str(j), j) for j in range(dim_size)]
        
        widgets.extend([
            Static(f"{dim_name}:", classes="dim-label"),
            Select(
                options,
                value=slice_indices[dim_idx],
                id=f"slice-{dim_idx}",
                allow_blank=False,
            ),
        ])

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
    
    Args:
        dim_names: Dimension names
        slice_indices: Current slice indices
        x_axis: X axis index
        y_axis: Y axis index
        ndim: Number of dimensions
        plot_type: "contour" (2D) or "line" (1D)
        
    Returns:
        Formatted slice info string (e.g., "Plot: lat × lon | Slice: time=0")
    """
    if ndim <= 1:
        return f"Plot: {dim_names[x_axis]}"
    
    # Build plot info
    if plot_type == "line":
        plot_info = f"{dim_names[x_axis]} (1D)"
    else:
        plot_info = f"{dim_names[x_axis]} × {dim_names[y_axis]}"
    
    # Build slice info for non-plotted dimensions
    plot_dims = {x_axis, y_axis} if plot_type == "contour" else {x_axis}
    slice_parts = [
        f"{dim_names[i]}={slice_indices[i]}"
        for i in range(ndim)
        if i not in plot_dims
    ]
    
    if slice_parts:
        return f"Plot: {plot_info} | Slice: {', '.join(slice_parts)}"
    return f"Plot: {plot_info}"


# ============================================================================
# PLOT SCREEN
# ============================================================================

class PlotScreen(ModalScreen[None]):
    """Modal screen for displaying plots with dimension slicing controls.
    
    This screen provides an interactive interface for visualizing multi-dimensional
    data with support for:
    - 1D line plots and 2D heatmaps
    - Dynamic axis selection
    - Dimension slicing for 3D+ data
    - Tabbed view switching between plot and data table
    - Consistent colormap scaling across slices
    """

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
        """Initialize the plot screen.
        
        Args:
            data: Multi-dimensional numpy array to visualize
            var_name: Variable name for display
            dim_names: Tuple of dimension names
            initial_tab: Initial tab to display ("plot" or "array")
        """
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
        
        # Initialize plot configuration
        self._plot_type = "contour" if data.ndim >= 2 else "line"
        self._initialize_axes()
    
    def _initialize_axes(self) -> None:
        """Initialize X and Y axis indices based on data dimensionality."""
        if self._original_data.ndim >= 2:
            # Default to last two dimensions (e.g., lat, lon)
            self._x_axis = self._original_data.ndim - 1
            self._y_axis = self._original_data.ndim - 2
        else:
            self._x_axis = 0
            self._y_axis = None
    
    @property
    def _title(self) -> str:
        """Build formatted title with variable info.
        
        Returns:
            Rich-formatted title string: name (dims) [nD] dtype
        """
        shape = self._original_data.shape
        dtype = str(self._original_data.dtype)
        ndim = self._original_data.ndim

        parts = [f"[{Colors.variable()}]{self._var_name}[/]"]

        # Dimension info
        if self._dim_names and shape:
            dim_str = ", ".join(f"{d}={s}" for d, s in zip(self._dim_names, shape))
            parts.append(f"[{Colors.muted()}]({dim_str})[/]")
        elif shape:
            dim_str = "×".join(str(s) for s in shape)
            parts.append(f"[{Colors.muted()}]({dim_str})[/]")

        # Dimensionality label
        if ndim > 0:
            parts.append(f"[{Colors.muted()}]\\[{ndim}D][/]")

        # Data type
        parts.append(f"[{Colors.muted()}]{dtype}[/]")

        return " ".join(parts)

    def watch_loading(self, is_loading: bool) -> None:
        """React to loading state changes."""
        self.set_class(is_loading, "loading")

    def _get_display_data(self) -> np.ndarray:
        """Extract the data slice to display based on current configuration.
        
        For 1D plots: extracts data along the X axis.
        For 2D plots: extracts the X and Y dimensions.
        For 3D+ arrays: slices other dimensions at their current index.
        
        Returns:
            1D or 2D numpy array ready for plotting
        """
        data = self._original_data
        ndim = data.ndim

        if ndim == 1:
            return data
        
        # Build indexing tuple
        indices = []
        
        if self._plot_type == "line":
            # 1D line plot: extract slice along X axis
            for dim in range(ndim):
                if dim == self._x_axis:
                    indices.append(slice(None))  # Keep full extent
                else:
                    idx = max(0, min(self._slice_indices[dim], data.shape[dim] - 1))
                    indices.append(idx)
            
            return np.asarray(data[tuple(indices)])
        
        # 2D contour plot
        if ndim == 2:
            # Transpose if needed to match axis selection
            if self._x_axis == 0 and self._y_axis == 1:
                return data
            else:
                return data.T
        
        # 3D+ data: extract X and Y dimensions
        for dim in range(ndim):
            if dim == self._x_axis or dim == self._y_axis:
                indices.append(slice(None))  # Keep full extent
            else:
                idx = max(0, min(self._slice_indices[dim], data.shape[dim] - 1))
                indices.append(idx)

        sliced = np.asarray(data[tuple(indices)])
        
        # Ensure Y is rows and X is columns
        if sliced.ndim == 2:
            remaining_dims = [d for d in range(ndim) if d in {self._x_axis, self._y_axis}]
            if remaining_dims[0] == self._x_axis:
                sliced = sliced.T
        
        return sliced

    def compose(self) -> ComposeResult:
        """Compose the plot interface layout."""
        data = self._get_display_data()
        is_2d = data.ndim >= 2
        ndim = self._original_data.ndim

        with Container(id="plot-container"):
            # Header
            yield Static(f" {self._title}", id="plot-title")

            # Controls (for 2D+ data)
            if ndim >= 2:
                with Horizontal(id="controls-row"):
                    # Plot type selector
                    yield Static("Plot type:", classes="dim-label")
                    yield Select(
                        [("Contour (2D)", "contour"), ("Line (1D)", "line")],
                        value=self._plot_type,
                        id="plot-type-select",
                        allow_blank=False,
                    )
                    
                    # Axis selection controls
                    for widget in _create_axis_controls(
                        self._original_data,
                        self._dim_names,
                        self._x_axis,
                        self._y_axis,
                        self._plot_type
                    ):
                        yield widget
                    
                    # Slice controls
                    for widget in _create_slice_controls(
                        self._original_data,
                        self._dim_names,
                        self._slice_indices,
                        self._x_axis,
                        self._y_axis,
                        self._plot_type
                    ):
                        yield widget

            # Tabbed content
            initial = "tab-plot" if self._initial_tab == "plot" else "tab-array"
            with TabbedContent(id="view-tabs", initial=initial):
                # Plot view
                with TabPane("Plot", id="tab-plot"):
                    with Container(id="plot-view"):
                        with Center():
                            with Vertical(id="plot-content"):
                                # Plot widget
                                plot_widget = create_plot_widget(
                                    data,
                                    vmin=self._global_vmin,
                                    vmax=self._global_vmax
                                )
                                yield plot_widget

                                # Colorbar (for 2D plots)
                                if is_2d:
                                    yield Static(
                                        _create_colorbar_text(
                                            data,
                                            self._global_vmin,
                                            self._global_vmax
                                        ),
                                        id="colorbar-legend"
                                    )

                # Array view
                with TabPane("Array", id="tab-array"):
                    with Container(id="table-view"):
                        yield DataTable(id="array-table")

            # Footer
            yield Static(self._build_footer_text(), id="plot-footer")

    def _build_footer_text(self) -> str:
        """Build footer text with controls and slice info."""
        ndim = self._original_data.ndim
        if ndim <= 1:
            return " [q/Esc] Close  [Tab] Toggle Plot↔Table "
        
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
        self.run_worker(
            self._populate_table_async(),
            exclusive=True,
            name="populate_table"
        )

    # ========================================================================
    # ASYNC UPDATE METHODS
    # ========================================================================

    def _can_update_plot_in_place(self, existing_widget, new_data: np.ndarray) -> bool:
        """Check if we can update the existing plot widget without replacing it.
        
        This is an optimization to avoid expensive widget removal/mounting operations.
        We can update in place when:
        - The widget type matches the data dimensionality (1D→1D or 2D→2D)
        - The widget has an update_data() method
        
        Args:
            existing_widget: The current plot widget
            new_data: The new data to display
            
        Returns:
            True if we can update in place, False if we need to replace the widget
        """
        # Check if widget type matches data dimensionality
        if isinstance(existing_widget, DataPlot2D) and new_data.ndim == 2:
            return True
        elif isinstance(existing_widget, DataPlot1D) and new_data.ndim == 1:
            return True
        
        # Widget type doesn't match data (e.g., switching from 2D to 1D plot)
        return False

    async def _update_plot_in_place(self, existing_widget, data: np.ndarray) -> None:
        """Update existing plot widget with new data (optimization path).
        
        Args:
            existing_widget: The plot widget to update
            data: New data to display
        """
        if isinstance(existing_widget, DataPlot2D):
            # Update 2D plot with downsampling if needed
            plot_data = downsample_2d(data, DOWNSAMPLE_2D_THRESHOLD)
            existing_widget.update_data(plot_data)
            
            # Update colorbar
            await self._update_colorbar(data)
            
        elif isinstance(existing_widget, DataPlot1D):
            # Update 1D plot
            existing_widget.update_data(data)

    async def _update_colorbar(self, data: np.ndarray) -> None:
        """Update the colorbar legend with new data range.
        
        Args:
            data: Data array to compute colorbar range from
        """
        try:
            colorbar = self.query_one("#colorbar-legend", Static)
            colorbar.update(_create_colorbar_text(
                data,
                self._global_vmin,
                self._global_vmax
            ))
        except Exception as e:
            logger.debug(f"Colorbar not found or failed to update: {e}")

    async def _replace_plot_widget(self, plot_content: Vertical, data: np.ndarray) -> None:
        """Replace the plot widget entirely (when in-place update isn't possible).
        
        This is needed when:
        - Switching between 1D and 2D plot types
        - Initial plot creation
        - Widget type mismatch
        
        Args:
            plot_content: Container to mount the new widget in
            data: Data to display
        """
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

        # Create and mount new plot widget
        plot_widget = create_plot_widget(
            data,
            vmin=self._global_vmin,
            vmax=self._global_vmax
        )
        await plot_content.mount(plot_widget)

        # Mount colorbar for 2D data
        if data.ndim >= 2:
            new_colorbar = Static(
                _create_colorbar_text(
                    data,
                    self._global_vmin,
                    self._global_vmax
                ),
                id="colorbar-legend"
            )
            await plot_content.mount(new_colorbar)

    async def _render_plot_async(self) -> None:
        """Render plot asynchronously after configuration changes.
        
        This method uses two strategies:
        1. In-place update: Fast path for slice changes (same plot type)
        2. Widget replacement: Needed when switching plot types (1D ↔ 2D)
        """
        try:
            self.loading = True
            data = self._get_display_data()

            # Get plot content container
            try:
                plot_content = self.query_one("#plot-content", Vertical)
            except Exception as e:
                logger.error(f"Failed to find plot content container: {e}")
                return

            # Try in-place update first (optimization for slice changes)
            try:
                existing_widget = self.query_one("#plot-widget")
                
                if self._can_update_plot_in_place(existing_widget, data):
                    # Fast path: update existing widget
                    logger.debug("Updating plot in place")
                    await self._update_plot_in_place(existing_widget, data)
                    return
                else:
                    # Widget type mismatch: need to replace
                    logger.debug("Plot type changed, replacing widget")
                    
            except Exception as e:
                # No existing widget found: initial render
                logger.debug(f"No existing plot widget, creating new one: {e}")

            # Replace widget (for plot type changes or initial render)
            await self._replace_plot_widget(plot_content, data)

        except Exception as e:
            logger.exception(f"Failed to render plot: {e}")
        finally:
            self.loading = False

    async def _populate_table_async(self) -> None:
        """Populate the array data table asynchronously."""
        try:
            table = self.query_one("#array-table", DataTable)
            data = self._get_display_data()
            await _populate_data_table(table, data, self._dim_names)
        except Exception as e:
            logger.exception(f"Failed to populate table: {e}")

    async def _refresh_controls_async(self) -> None:
        """Refresh axis and slice controls after plot type or axis changes."""
        if self._original_data.ndim < 2:
            return
        
        try:
            controls_row = self.query_one("#controls-row", Horizontal)
            
            logger.debug(f"Refreshing controls. Plot type: {self._plot_type}")
            
            # Remove all controls except plot type selector (first 2 widgets)
            children_before = list(controls_row.children)
            for widget in children_before[2:]:
                try:
                    await widget.remove()
                except Exception as e:
                    logger.debug(f"Failed to remove widget: {e}")
            
            # Mount new axis controls
            for widget in _create_axis_controls(
                self._original_data,
                self._dim_names,
                self._x_axis,
                self._y_axis,
                self._plot_type
            ):
                await controls_row.mount(widget)
            
            # Mount new slice controls
            for widget in _create_slice_controls(
                self._original_data,
                self._dim_names,
                self._slice_indices,
                self._x_axis,
                self._y_axis,
                self._plot_type
            ):
                await controls_row.mount(widget)
            
        except Exception as e:
            logger.exception(f"Failed to refresh controls: {e}")

    async def _refresh_all_async(self) -> None:
        """Refresh controls, plot, and table."""
        await self._refresh_controls_async()
        await self._render_plot_async()
        await self._populate_table_async()
        self._update_footer()
    
    async def _update_plot_and_table_async(self) -> None:
        """Update plot and table without refreshing controls."""
        await self._render_plot_async()
        await self._populate_table_async()
        self._update_footer()
    
    # ========================================================================
    # EVENT HANDLERS
    # ========================================================================

    def on_select_changed(self, event: Select.Changed) -> None:
        """Handle select control changes."""
        select_id = event.select.id
        if not select_id:
            return
        
        # Plot type selection
        if select_id == "plot-type-select":
            new_value = event.value
            if new_value is not None and new_value != self._plot_type:
                old_plot_type = self._plot_type
                self._plot_type = new_value
                logger.info(f"Plot type changed: {old_plot_type} → {new_value}")
                self.run_worker(
                    self._refresh_all_async(),
                    exclusive=True,
                    name="refresh_all"
                )
            return
        
        # X axis selection
        if select_id == "x-axis-select":
            new_value = event.value
            if new_value is not None:
                old_x_axis = self._x_axis
                self._x_axis = int(new_value)
                if old_x_axis != self._x_axis:
                    self.run_worker(
                        self._refresh_all_async(),
                        exclusive=True,
                        name="refresh_all"
                    )
            return
        
        # Y axis selection
        if select_id == "y-axis-select":
            new_value = event.value
            if new_value is not None:
                old_y_axis = self._y_axis
                self._y_axis = int(new_value)
                if old_y_axis != self._y_axis:
                    self.run_worker(
                        self._refresh_all_async(),
                        exclusive=True,
                        name="refresh_all"
                    )
            return
        
        # Slice selection
        if select_id.startswith("slice-"):
            dim_idx = int(select_id.split("-")[1])
            if dim_idx < len(self._slice_indices):
                new_value = event.value
                if new_value is not None:
                    self._slice_indices[dim_idx] = int(new_value)
                    self.run_worker(
                        self._update_plot_and_table_async(),
                        exclusive=True,
                        name="update_plot"
                    )

    def _update_footer(self) -> None:
        """Update footer with current slice information."""
        try:
            footer = self.query_one("#plot-footer", Static)
            footer.update(self._build_footer_text())
        except Exception as e:
            logger.debug(f"Failed to update footer: {e}")

    # ========================================================================
    # ACTIONS
    # ========================================================================

    def action_close(self) -> None:
        """Close the modal."""
        self.dismiss()

    def action_toggle_view(self) -> None:
        """Toggle between Plot and Array views."""
        try:
            tabs = self.query_one(TabbedContent)
            tabs.active = "tab-array" if tabs.active == "tab-plot" else "tab-plot"
        except Exception as e:
            logger.debug(f"Failed to toggle view: {e}")

    def action_decrement_slice(self) -> None:
        """Decrement the focused slice control value."""
        if self._original_data.ndim <= 2:
            return

        try:
            # Get focused Select widget or use first slice control
            focused = self.focused
            if isinstance(focused, Select) and focused.id and focused.id.startswith("slice-"):
                select_widget = focused
            else:
                select_widget = self.query_one("#slice-0", Select)
        except Exception:
            return

        # Decrement value
        current_value = select_widget.value
        if current_value is not None and current_value > 0:
            select_widget.value = current_value - 1

    def action_increment_slice(self) -> None:
        """Increment the focused slice control value."""
        if self._original_data.ndim <= 2:
            return

        try:
            # Get focused Select widget or use first slice control
            focused = self.focused
            if isinstance(focused, Select) and focused.id and focused.id.startswith("slice-"):
                select_widget = focused
            else:
                select_widget = self.query_one("#slice-0", Select)
        except Exception:
            return

        # Increment value
        current_value = select_widget.value
        if current_value is not None:
            dim_idx = int(select_widget.id.split("-")[1])
            max_value = self._original_data.shape[dim_idx] - 1
            
            if current_value < max_value:
                select_widget.value = current_value + 1

    def action_transpose(self) -> None:
        """Swap X and Y axes (contour plots only)."""
        if self._original_data.ndim < 2 or self._plot_type != "contour":
            return
        
        # Swap axes
        self._x_axis, self._y_axis = self._y_axis, self._x_axis
        
        # Update Select widgets
        try:
            x_select = self.query_one("#x-axis-select", Select)
            y_select = self.query_one("#y-axis-select", Select)
            x_select.value = self._x_axis
            y_select.value = self._y_axis
        except Exception as e:
            logger.debug(f"Failed to update axis select widgets: {e}")
        
        # Re-render
        self.run_worker( self._render_plot_async(), exclusive=True, name="render_plot" )
        self.run_worker(
            self._populate_table_async(),
            exclusive=True,
            name="populate_table"
        )
        self._update_footer()
