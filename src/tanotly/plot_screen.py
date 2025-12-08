"""Plot and data table screen for Tanotly.

Provides a modal screen for visualizing variable data with:
- 2D heatmap plots with viridis colormap
- 1D line plots
- Dimension slicing controls for 3D+ data
- Data table view with cell navigation
"""

import numpy as np
from rich.text import Text

from textual.app import ComposeResult
from textual.binding import Binding
from textual.containers import Container, Horizontal, Vertical, Center
from textual.screen import ModalScreen
from textual.widgets import Static, DataTable, Select, TabbedContent, TabPane
from textual.reactive import reactive

from .visualization import DataPlot1D, DataPlot2D, VIRIDIS_COLORS
from .config import Colors, ThemeManager


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

    def _calculate_plot_size(self, data: np.ndarray) -> tuple[int, int]:
        """Calculate appropriate plot size based on data shape.
        
        Returns (width, height) in characters.
        For 2D data, we want 1:1 aspect ratio where possible.
        """
        if data.ndim == 1:
            # 1D: fixed height, width based on data length
            width = min(max(len(data), 40), 80)
            height = 15
        elif data.ndim == 2:
            rows, cols = data.shape
            # Target: each data point = ~1 character width, ~0.5 character height
            # (terminal characters are ~2x taller than wide)
            max_width = 80
            max_height = 30
            
            # Scale to fit while maintaining aspect ratio
            width = min(cols, max_width)
            height = min(rows // 2 + 1, max_height)  # Divide by 2 for terminal aspect
            
            # Ensure minimum size
            width = max(width, 20)
            height = max(height, 10)
        else:
            width, height = 60, 20
            
        return width, height

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
                    yield Static("Slice:", classes="dim-label")
                    for dim_idx in range(ndim - 2):
                        dim_name = self._dim_names[dim_idx]
                        dim_size = self._original_data.shape[dim_idx]
                        
                        yield Static(f"{dim_name}:", classes="dim-label")
                        # Create options with Text objects for proper rendering
                        options = [(Text(str(j)), j) for j in range(dim_size)]
                        yield Select(
                            options,
                            value=self._slice_indices[dim_idx],
                            id=f"slice-{dim_idx}",
                            allow_blank=False,
                        )

            # Tabbed content for Plot vs Array view
            initial = "tab-plot" if self._initial_tab == "plot" else "tab-array"
            with TabbedContent(id="view-tabs", initial=initial):
                with TabPane("Plot", id="tab-plot"):
                    with Container(id="plot-view"):
                        # Centered plot container
                        with Center():
                            with Vertical(id="plot-content"):
                                # Plot widget
                                plot_widget = self._create_plot_widget(data)
                                yield plot_widget
                                
                                # Colorbar legend for 2D data
                                if is_2d:
                                    yield self._create_colorbar_legend(data)

                with TabPane("Array", id="tab-array"):
                    with Container(id="table-view"):
                        yield DataTable(id="array-table")

            # Footer
            yield Static(self._build_footer_text(), id="plot-footer")

    def _create_colorbar_text(self, data: np.ndarray) -> str:
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

    def _create_colorbar_legend(self, data: np.ndarray) -> Static:
        """Create a horizontal colorbar legend widget with min/max values."""
        return Static(self._create_colorbar_text(data), id="colorbar-legend")

    def _build_footer_text(self) -> str:
        """Build footer text."""
        ndim = self._original_data.ndim
        if ndim <= 2:
            return " [q/Esc] Close  [Tab] Toggle Plot↔Table "
        else:
            slice_info = ", ".join(
                f"{self._dim_names[i]}={self._slice_indices[i]}"
                for i in range(ndim - 2)
            )
            return f" [q/Esc] Close  [Tab] Toggle | Slice: {slice_info} "

    def _create_plot_widget(self, data: np.ndarray):
        """Create plot widget sized to data.
        
        Args:
            data: The data to plot (should be 1D or 2D after slicing)
            
        Returns:
            DataPlot1D, DataPlot2D, or Static widget
        """
        # Ensure we have a numpy array and make a copy to avoid issues with views
        data = np.array(data, copy=True)
        width, height = self._calculate_plot_size(data)
        
        # Get theme from ThemeManager
        is_dark = ThemeManager.is_dark()
        
        if data.ndim == 1:
            # 1D line plot
            plot_data = data
            if len(plot_data) > 500:
                indices = np.linspace(0, len(plot_data) - 1, 500, dtype=int)
                plot_data = plot_data[indices]
            return DataPlot1D(plot_data, is_dark=is_dark, width=width, height=height, id="plot-widget")

        elif data.ndim == 2:
            # 2D heatmap
            rows, cols = data.shape
            max_dim = 100
            if rows > max_dim or cols > max_dim:
                row_step = max(1, rows // max_dim)
                col_step = max(1, cols // max_dim)
                plot_data = data[::row_step, ::col_step]
            else:
                plot_data = data
            return DataPlot2D(plot_data, is_dark=is_dark, width=width, height=height, id="plot-widget")
        
        elif data.ndim >= 3:
            # For 3D+ data, take a 2D slice (first slice of each extra dimension)
            # This shouldn't normally happen as _get_display_data should return 2D
            slice_indices = [0] * (data.ndim - 2) + [slice(None), slice(None)]
            sliced_data = data[tuple(slice_indices)]
            return self._create_plot_widget(sliced_data)
        
        else:
            # Scalar or empty
            return Static(f"Cannot plot data with shape {data.shape}", id="plot-widget")

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
                        colorbar.update(self._create_colorbar_text(data))
                    except Exception:
                        pass
                    
                    self.loading = False
                    return
            except Exception:
                pass

            # Fall back to replacing the widget
            plot_widget = self._create_plot_widget(data)

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
                new_colorbar = self._create_colorbar_legend(data)
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
            table.cursor_type = "cell"
            table.zebra_stripes = True

            if table.row_count > 0:
                table.clear(columns=True)

            data = self._get_display_data()
            max_rows, max_cols = 500, 50

            if data.ndim == 1:
                table.add_column("Index", width=8)
                table.add_column("Value", width=20)
                for i, val in enumerate(data[:max_rows]):
                    table.add_row(str(i), self._format_value(val))
                if len(data) > max_rows:
                    table.add_row("...", f"({len(data) - max_rows} more)")

            elif data.ndim == 2:
                rows, cols = data.shape
                display_cols = min(cols, max_cols)
                display_rows = min(rows, max_rows)

                ndim = self._original_data.ndim
                row_dim_name = self._dim_names[ndim - 2] if ndim >= 2 else "row"

                table.add_column(row_dim_name, width=8)
                for j in range(display_cols):
                    table.add_column(str(j), width=10)
                if cols > max_cols:
                    table.add_column("...", width=5)

                for i in range(display_rows):
                    row = [str(i)]
                    for j in range(display_cols):
                        row.append(self._format_value(data[i, j]))
                    if cols > max_cols:
                        row.append("...")
                    table.add_row(*row)

                if rows > max_rows:
                    table.add_row("...", *["..."] * (display_cols + (1 if cols > max_cols else 0)))
        except Exception:
            pass

    def _format_value(self, val) -> str:
        """Format a value for display."""
        if isinstance(val, float):
            if np.isnan(val):
                return "NaN"
            if abs(val) >= 1e6 or (abs(val) < 1e-3 and val != 0):
                return f"{val:.3e}"
            return f"{val:.4g}"
        return str(val)

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
