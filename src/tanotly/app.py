"""Simplified Tanotly application - more reliable."""

from pathlib import Path
from typing import Optional

from textual.app import App, ComposeResult
from textual.binding import Binding
from textual.containers import Horizontal, Vertical, VerticalScroll
from textual.widgets import Footer, Header, Input, Static, Tree

import numpy as np
import xarray as xr

from .data import DataReader, DatasetInfo, DataNode
from .data.models import NodeType


class TanotlyApp(App[None]):
    """Simplified Tanotly application."""

    # Disable mouse support to avoid terminal issues
    ENABLE_COMMAND_PALETTE = False

    CSS = """
    /* Top status bar - accent color (cyan by default) */
    #top-bar {
        dock: top;
        height: 1;
        background: $accent;
        content-align: center middle;
    }

    /* Main content area */
    #main {
        height: 1fr;
    }

    /* Tree panel - left side (40%) */
    #tree-container {
        width: 40%;
        border-right: solid $accent;
    }

    /* Detail panel - right side (60%) */
    #detail-container {
        width: 60%;
        padding: 1;
    }

    Tree {
        height: 100%;
    }

    /* Hide guide for leaf nodes (no children) */
    Tree > .tree--guides {
        color: $accent-darken-1;
    }

    VerticalScroll {
        height: 100%;
    }

    /* Search input - docked at bottom, hidden by default */
    #search-input {
        dock: bottom;
        display: none;
        border: tall $accent;
        background: $surface;
        color: $text;
    }

    #search-input:focus {
        border: tall $success;
    }

    /* Color scheme reference:
     * $accent - cyan (default) - used for highlights, borders
     * $success - green - used for success states
     * $warning - yellow - used for warnings
     * $error - red - used for errors
     * $surface - background color for widgets
     * $text - default text color
     * $primary - primary color
     * $secondary - secondary color
     */
    """

    BINDINGS = [
        Binding("q", "quit", "Quit"),
        Binding("/", "start_search", "Search"),
        Binding("n", "next_match", "Next", show=False),
        Binding("N", "prev_match", "Prev", show=False),
        Binding("escape", "clear_search", "Clear"),
        Binding("j", "cursor_down", "Down", show=False),
        Binding("k", "cursor_up", "Up", show=False),
        Binding("h", "cursor_left", "Left", show=False),
        Binding("l", "cursor_right", "Right", show=False),
        Binding("p", "toggle_plot", "Plot"),
        Binding("y", "copy_info", "Copy", show=False),
    ]

    def __init__(self, file_path: Optional[str] = None):
        super().__init__()
        self.file_path = file_path
        self.dataset: Optional[DatasetInfo] = None
        self.full_dataset_info: Optional[DatasetInfo] = None
        # Disable mouse completely
        self.mouse_enabled = False
        # Search state
        self.search_query = ""
        self.search_matches = []
        self.current_match_idx = -1
        # Plot mode
        self.show_plot = False
        self.current_node: Optional[DataNode] = None

    def compose(self) -> ComposeResult:
        yield Header(show_clock=True)
        yield Static("", id="top-bar")

        with Horizontal(id="main"):
            with Vertical(id="tree-container"):
                yield Tree("Data", id="tree")
            with VerticalScroll(id="detail-container"):
                yield Static(
                    "[bold yellow]Welcome to Tanotly![/bold yellow]\n\n"
                    "[dim]Navigation: ↑/↓/j/k, ←/→/h/l to expand\n"
                    "Search: / then type, n/N for next/prev\n"
                    "Quit: q or Esc[/dim]",
                    id="detail"
                )

        # Search input (hidden by default)
        yield Input(placeholder="Search...", id="search-input")
        yield Footer()

    def on_mount(self) -> None:
        if self.file_path:
            self.load_file(self.file_path)
        # Focus tree so arrow keys work immediately
        tree = self.query_one("#tree", Tree)
        tree.focus()

    def load_file(self, path: str) -> None:
        try:
            self._update_status(f"Loading {Path(path).name}...")
            self.full_dataset_info = DataReader.read_file(path)
            self.dataset = self.full_dataset_info

            tree = self.query_one("#tree", Tree)
            tree.clear()
            tree.show_root = False
            tree.show_guides = True
            tree.guide_depth = 4
            self._populate_tree(tree.root, self.dataset.root_node)

            # Expand first level
            for child in tree.root.children:
                child.expand()

            # Focus tree and select first node
            tree.focus()
            if tree.root.children:
                tree.select_node(tree.root.children[0])

            self._update_status(
                f"{Path(path).name} | {len(self.dataset.variables)} variables | Use ↑↓ arrows"
            )
        except Exception as e:
            self._update_status(f"Error: {e}")

    def _populate_tree(self, tree_node, data_node: DataNode) -> None:
        """Recursively populate tree."""
        for child in data_node.children:
            # Skip attributes - they'll be shown in detail view
            # if child.node_type == NodeType.ATTRIBUTE:
            #     continue
            label = self._format_label(child)
            # Only allow expansion if node has children
            allow_expand = len(child.children) > 0
            node = tree_node.add(label, data=child, allow_expand=allow_expand)
            if child.children:
                self._populate_tree(node, child)

    def _get_data_type_label(self, node: DataNode) -> str:
        """Get Panoply-style data type label (2D, 3D, Geo2D, etc.)."""
        shape = node.metadata.get("shape", ())
        dims = node.metadata.get("dims", ())

        if not shape:
            return ""

        ndim = len(shape)
        if ndim == 0:
            return "scalar"
        elif ndim == 1:
            return "1D"
        elif ndim == 2:
            # Check if it looks like geographic data
            if dims and any(d in str(dims).lower() for d in ['lat', 'lon', 'latitude', 'longitude']):
                return "Geo2D"
            return "2D"
        elif ndim == 3:
            # Check for geo-temporal data
            if dims and any(d in str(dims).lower() for d in ['lat', 'lon', 'latitude', 'longitude']):
                return "Geo3D"
            return "3D"
        elif ndim == 4:
            return "4D"
        else:
            return f"{ndim}D"

    def _format_label(self, node: DataNode) -> str:
        """Format node label with icons and styling."""
        # Icon mapping for different node types
        if node.node_type == NodeType.ROOT:
            icon = "🏠"
            name_color = "bold magenta"
            return f"{icon} [{name_color}]{node.name}[/{name_color}]"

        elif node.node_type == NodeType.GROUP:
            icon = "📂"
            name_color = "yellow"
            # Count non-attribute children
            child_count = sum(1 for c in node.children if c.node_type != NodeType.ATTRIBUTE)
            return f"{icon} [{name_color}]{node.name}[/{name_color}] [dim]({child_count})[/dim]"

        elif node.node_type == NodeType.VARIABLE:
            icon = "🌡️"
            name_color = "cyan"
            shape = node.metadata.get("shape", "")
            dtype = node.metadata.get("dtype", "")
            data_type = self._get_data_type_label(node)

            if shape and dtype:
                shape_str = "×".join(str(s) for s in shape)
                return f"{icon} [{name_color}]{node.name}[/{name_color}] [dim]\\[{shape_str}][/dim] [green]{data_type}[/green] [dim]{dtype}[/dim]"
            return f"{icon} [{name_color}]{node.name}[/{name_color}]"

        elif node.node_type == NodeType.DIMENSION:
            icon = "📏"
            name_color = "blue"
            size = node.metadata.get("size", "")
            if size:
                return f"{icon} [{name_color}]{node.name}[/{name_color}] [dim]({size})[/dim]"
            return f"{icon} [{name_color}]{node.name}[/{name_color}]"

        elif node.node_type == NodeType.ATTRIBUTE:
            icon = "🏷️"
            name_color = "magenta"
            return f"{icon} [{name_color}]{node.name}[/{name_color}]"

        else:
            return node.name

    def on_tree_node_highlighted(self, event: Tree.NodeHighlighted) -> None:  # type: ignore
        """Show details when node is highlighted."""
        if event.node.data:
            self.show_details(event.node.data)

    def show_details(self, node: DataNode) -> None:
        """Display node details."""
        self.current_node = node  # Track current node for plot/copy
        detail = self.query_one("#detail", Static)

        # Build content with icons and better formatting
        icon = self._get_node_icon(node)
        content = f"{icon} [bold cyan]{node.name}[/bold cyan]\n"
        content += "─" * 60 + "\n\n"

        # Type with color coding
        type_color = self._get_type_color(node.node_type)
        content += f"[{type_color}]● Type:[/{type_color}] {node.node_type.value}\n"
        content += f"[dim]● Path:[/dim] {node.path}\n\n"

        # Metadata in a structured format
        if node.metadata:
            content += "[bold yellow]📊 Metadata[/bold yellow]\n"
            content += "─" * 60 + "\n"
            for key, val in node.metadata.items():
                if key == "shape":
                    shape_str = " × ".join(str(s) for s in val)
                    content += f"  [cyan]▸ {key}:[/cyan] {shape_str}\n"
                elif key == "dims":
                    dims_str = ", ".join(str(d) for d in val)
                    content += f"  [cyan]▸ {key}:[/cyan] ({dims_str})\n"
                elif key == "size":
                    # Format large numbers with commas
                    size_formatted = f"{val:,}" if isinstance(val, int) else str(val)
                    content += f"  [cyan]▸ {key}:[/cyan] {size_formatted}\n"
                else:
                    content += f"  [cyan]▸ {key}:[/cyan] {val}\n"
            content += "\n"

        # Attributes (Panoply-style with : prefix)
        if node.attributes:
            content += "[bold magenta]🏷️  Attributes[/bold magenta]\n"
            content += "─" * 60 + "\n"
            for key, val in node.attributes.items():
                val_str = str(val)
                if len(val_str) > 80:
                    val_str = val_str[:77] + "..."
                content += f"  [magenta]:{key}[/magenta] = {val_str}\n"
            content += "\n"

        # Data preview for variables
        if node.node_type == NodeType.VARIABLE and self.dataset:
            content += self._get_data_preview(node)

        detail.update(content)

    def _create_ascii_plot(self, data: np.ndarray) -> str:
        """Create an ASCII plot of the data."""
        try:
            # Remove NaN values
            clean_data = data[~np.isnan(data)] if data.dtype.kind == 'f' else data

            if clean_data.size == 0:
                return "[dim]No valid data to plot[/dim]\n"

            plot_content = "[bold cyan]📊 ASCII Plot[/bold cyan]\n"

            if data.ndim == 1 and data.size <= 100:
                # Line plot for 1D data
                plot_content += self._create_line_plot(clean_data)
            elif data.ndim == 1:
                # Histogram for larger 1D data
                plot_content += self._create_histogram(clean_data)
            elif data.ndim == 2:
                # Heatmap for 2D data (sample if too large)
                if data.shape[0] > 20 or data.shape[1] > 40:
                    sample_data = data[:20, :40]
                    plot_content += self._create_heatmap(sample_data)
                    plot_content += f"[dim](Showing {min(20, data.shape[0])}×{min(40, data.shape[1])} sample)[/dim]\n"
                else:
                    plot_content += self._create_heatmap(data)
            else:
                plot_content += "[dim]Plotting only available for 1D and 2D data[/dim]\n"

            return plot_content
        except Exception as e:
            return f"[dim red]Plot error: {e}[/dim red]\n"

    def _create_line_plot(self, data: np.ndarray, height: int = 10, width: int = 50) -> str:
        """Create a simple ASCII line plot."""
        if data.size == 0:
            return ""

        # Normalize data to plot height
        data_min, data_max = np.nanmin(data), np.nanmax(data)
        if data_max == data_min:
            data_norm = np.zeros_like(data, dtype=int)
        else:
            data_norm = ((data - data_min) / (data_max - data_min) * (height - 1)).astype(int)

        # Sample data if too wide
        if len(data) > width:
            indices = np.linspace(0, len(data) - 1, width).astype(int)
            data_norm = data_norm[indices]

        plot = ""
        for row in range(height - 1, -1, -1):
            line = ""
            for val in data_norm:
                if val == row:
                    line += "●"
                elif val > row:
                    line += "│"
                else:
                    line += " "
            # Add axis labels
            y_val = data_min + (data_max - data_min) * row / (height - 1)
            plot += f"{y_val:8.2g} │{line}\n"

        plot += " " * 9 + "└" + "─" * len(data_norm) + "\n"
        return plot

    def _create_histogram(self, data: np.ndarray, bins: int = 20, height: int = 10) -> str:
        """Create an ASCII histogram."""
        hist, bin_edges = np.histogram(data, bins=bins)
        max_count = hist.max()

        if max_count == 0:
            return "[dim]No data to plot[/dim]\n"

        plot = ""
        for row in range(height, 0, -1):
            threshold = max_count * row / height
            line = ""
            for count in hist:
                if count >= threshold:
                    line += "█"
                elif count >= threshold * 0.5:
                    line += "▄"
                else:
                    line += " "
            plot += f"{int(threshold):6d} │{line}\n"

        plot += " " * 7 + "└" + "─" * len(hist) + "\n"
        plot += f"       Range: [{data.min():.3g}, {data.max():.3g}]\n"
        return plot

    def _create_heatmap(self, data: np.ndarray) -> str:
        """Create an ASCII heatmap for 2D data."""
        # Normalize to 0-9 range for characters
        data_min, data_max = np.nanmin(data), np.nanmax(data)
        if data_max == data_min:
            data_norm = np.zeros_like(data, dtype=int)
        else:
            data_norm = ((data - data_min) / (data_max - data_min) * 9).astype(int)

        # Characters from dark to light
        chars = " .:-=+*#%@"

        plot = ""
        for row in data_norm:
            line = "".join(chars[min(val, 9)] for val in row)
            plot += f"{line}\n"

        plot += f"Range: [{data_min:.3g}, {data_max:.3g}]\n"
        return plot

    def _get_node_icon(self, node: DataNode) -> str:
        """Get icon for node type."""
        icons = {
            NodeType.ROOT: "🏠",
            NodeType.GROUP: "📂",
            NodeType.VARIABLE: "🌡️",
            NodeType.DIMENSION: "📏",
            NodeType.ATTRIBUTE: "🏷️",
        }
        return icons.get(node.node_type, "●")

    def _get_type_color(self, node_type: NodeType) -> str:
        """Get color for node type."""
        colors = {
            NodeType.ROOT: "magenta",
            NodeType.GROUP: "yellow",
            NodeType.VARIABLE: "cyan",
            NodeType.DIMENSION: "blue",
            NodeType.ATTRIBUTE: "magenta",
        }
        return colors.get(node_type, "white")

    def _get_data_preview(self, node: DataNode) -> str:
        """Get data preview for a variable."""
        try:
            # Extract variable path - handle grouped NetCDF4/HDF5 files
            var_path = node.path

            # For xarray-style paths
            if "/variables/" in var_path:
                var_name = var_path.split("/variables/")[1]
            elif "/coordinates/" in var_path:
                var_name = var_path.split("/coordinates/")[1]
            else:
                var_name = node.name

            # Try xarray first - it works for simple files
            data = None
            ds = None
            try:
                ds = xr.open_dataset(self.dataset.file_path)
                if var_name in ds.variables:
                    var = ds[var_name]
                    data = var.values
                    ds.close()
            except Exception:
                if ds:
                    ds.close()

            # If xarray didn't work, try netCDF4 with full path
            if data is None:
                import netCDF4 as nc
                with nc.Dataset(self.dataset.file_path, 'r') as ncds:
                    # Navigate through groups to find the variable
                    # Parse the path like /data/navigation/mws_lat
                    parts = [p for p in var_path.split('/') if p]

                    # Navigate to the right group
                    obj = ncds
                    for i, part in enumerate(parts[:-1]):  # All but the last part are groups
                        if part in obj.groups:
                            obj = obj.groups[part]
                        elif part in obj.variables:
                            # It's a variable, not a group - use it directly
                            obj = obj.variables[part]
                            break

                    # Get the variable (last part of path)
                    var_name_final = parts[-1]
                    if hasattr(obj, 'variables') and var_name_final in obj.variables:
                        var = obj.variables[var_name_final]
                    elif hasattr(obj, '__getitem__'):
                        var = obj[var_name_final]
                    else:
                        raise KeyError(f"Cannot find variable at path: {var_path}")

                    data = var[:]

            content = "[bold green]📈 Data Preview[/bold green]\n"
            content += "─" * 60 + "\n"

            # Add ASCII plot if enabled and data is numeric
            if self.show_plot and np.issubdtype(data.dtype, np.number) and data.size > 0:
                content += self._create_ascii_plot(data)
                content += "\n"

            # Statistics for numeric data
            if np.issubdtype(data.dtype, np.number):
                # Count valid values
                valid_count = np.count_nonzero(~np.isnan(data)) if data.dtype.kind == 'f' else data.size
                nan_count = data.size - valid_count

                content += "[cyan]Statistics:[/cyan]\n"
                content += f"  [dim]▸ Min:[/dim]  {np.nanmin(data):.6g}\n"
                content += f"  [dim]▸ Max:[/dim]  {np.nanmax(data):.6g}\n"
                content += f"  [dim]▸ Mean:[/dim] {np.nanmean(data):.6g}\n"
                if data.size > 1:
                    content += f"  [dim]▸ Std:[/dim]  {np.nanstd(data):.6g}\n"
                if nan_count > 0:
                    content += f"  [dim]▸ NaN:[/dim]  {nan_count:,} ({nan_count/data.size*100:.1f}%)\n"
                content += f"  [dim]▸ Valid:[/dim] {valid_count:,}\n"
                content += "\n"

            # Sample values with better formatting
            content += "[cyan]Sample Values:[/cyan]\n"
            if data.size == 0:
                content += "  [dim](empty array)[/dim]\n"
            elif data.size <= 10:
                # Show all values for small arrays
                if data.ndim == 1:
                    for i, val in enumerate(data):
                        content += f"  [{i}] {val}\n"
                else:
                    content += f"  {data}\n"
            elif data.ndim == 1:
                # Show first and last 5 for 1D arrays
                content += "  [dim]First 5:[/dim]\n"
                for i in range(min(5, data.size)):
                    content += f"    [{i}] {data[i]}\n"
                if data.size > 10:
                    content += f"  [dim]  ... ({data.size - 10} more) ...[/dim]\n"
                    content += "  [dim]Last 5:[/dim]\n"
                    for i in range(data.size - 5, data.size):
                        content += f"    [{i}] {data[i]}\n"
                else:
                    content += "  [dim]Last 5:[/dim]\n"
                    for i in range(5, data.size):
                        content += f"    [{i}] {data[i]}\n"
            elif data.ndim == 2:
                # Show corner of 2D arrays
                rows, cols = data.shape
                show_rows = min(5, rows)
                show_cols = min(10, cols)
                content += f"  [dim]First {show_rows}×{show_cols} corner:[/dim]\n"
                for i in range(show_rows):
                    row_str = "  " + " ".join(f"{data[i, j]:8.3g}" for j in range(show_cols))
                    if cols > show_cols:
                        row_str += " ..."
                    content += row_str + "\n"
                if rows > show_rows:
                    content += "  [dim]...[/dim]\n"
                content += f"  [dim](Full shape: {rows:,} × {cols:,})[/dim]\n"
            else:
                # Multi-dimensional arrays
                content += f"  [dim]First 10 values (flattened):[/dim]\n"
                for i in range(min(10, data.size)):
                    content += f"    {data.flat[i]}\n"
                shape_str = " × ".join(f"{s:,}" for s in data.shape)
                content += f"  [dim](Shape: {shape_str}, Total: {data.size:,})[/dim]\n"

            return content

        except Exception as e:
            return f"[dim red]Could not load data: {e}[/dim red]\n"

    def _collect_all_tree_nodes(self, tree_node, result_list):
        """Recursively collect all tree nodes."""
        result_list.append(tree_node)
        for child in tree_node.children:
            self._collect_all_tree_nodes(child, result_list)

    def action_start_search(self) -> None:
        """Start a search using the bottom bar."""
        if not self.full_dataset_info:
            return

        # Show and focus the search input
        search_input = self.query_one("#search-input", Input)
        search_input.styles.display = "block"
        search_input.value = ""
        search_input.focus()
        self._update_status("Type to search, Enter to find, Esc to cancel")

    def on_input_submitted(self, event: Input.Submitted) -> None:
        """Handle search query submission."""
        if event.input.id == "search-input":
            self.search_query = event.value
            if self.search_query:
                self._perform_search()
            # Keep input visible if there are matches
            if not self.search_matches:
                event.input.styles.display = "none"
                tree = self.query_one("#tree", Tree)
                tree.focus()

    def _perform_search(self):
        """Find all matches for current search query."""
        if not self.search_query:
            return

        tree = self.query_one("#tree", Tree)
        all_nodes = []
        self._collect_all_tree_nodes(tree.root, all_nodes)

        self.search_matches = []
        query_lower = self.search_query.lower()

        for tree_node in all_nodes:
            if tree_node.data:
                # Search in node name and attributes
                if query_lower in tree_node.data.name.lower():
                    self.search_matches.append(tree_node)
                elif tree_node.data.attributes:
                    for key, val in tree_node.data.attributes.items():
                        if query_lower in key.lower() or query_lower in str(val).lower():
                            self.search_matches.append(tree_node)
                            break

        if self.search_matches:
            self.current_match_idx = 0
            self._jump_to_current_match()
            self._update_status(f"Search: '{self.search_query}' | {len(self.search_matches)} matches | n/N to navigate")
        else:
            self.current_match_idx = -1
            self._update_status(f"No matches for '{self.search_query}' | Press ESC to clear")

    def _jump_to_current_match(self):
        """Jump to the current match in the search results."""
        if 0 <= self.current_match_idx < len(self.search_matches):
            tree = self.query_one("#tree", Tree)
            match_node = self.search_matches[self.current_match_idx]

            # Expand all parent nodes to make the match visible
            parent = match_node.parent
            while parent is not None:
                parent.expand()
                parent = parent.parent

            tree.select_node(match_node)
            tree.scroll_to_node(match_node)
            self._update_status(
                f"Match {self.current_match_idx + 1}/{len(self.search_matches)}: '{self.search_query}' | n/N to navigate | ESC to clear"
            )

    def action_next_match(self) -> None:
        """Jump to next search match."""
        if not self.search_matches:
            return
        self.current_match_idx = (self.current_match_idx + 1) % len(self.search_matches)
        self._jump_to_current_match()

    def action_prev_match(self) -> None:
        """Jump to previous search match."""
        if not self.search_matches:
            return
        self.current_match_idx = (self.current_match_idx - 1) % len(self.search_matches)
        self._jump_to_current_match()

    def action_clear_search(self) -> None:
        """Clear search results."""
        self.search_query = ""
        self.search_matches = []
        self.current_match_idx = -1

        # Hide search input
        search_input = self.query_one("#search-input", Input)
        search_input.styles.display = "none"

        # Return focus to tree
        tree = self.query_one("#tree", Tree)
        tree.focus()

        if self.file_path:
            self._update_status(f"{Path(self.file_path).name} | Use / to search")
        else:
            self._update_status("")

    # Vim-style navigation
    def action_cursor_up(self) -> None:
        """Move cursor up (vim k)."""
        tree = self.query_one("#tree", Tree)
        tree.action_cursor_up()

    def action_cursor_down(self) -> None:
        """Move cursor down (vim j)."""
        tree = self.query_one("#tree", Tree)
        tree.action_cursor_down()

    def action_cursor_left(self) -> None:
        """Collapse node (vim h)."""
        tree = self.query_one("#tree", Tree)
        if tree.cursor_node:
            tree.cursor_node.collapse()

    def action_cursor_right(self) -> None:
        """Expand node (vim l)."""
        tree = self.query_one("#tree", Tree)
        if tree.cursor_node:
            tree.cursor_node.expand()

    def action_toggle_plot(self) -> None:
        """Toggle graphical plot view for current variable."""
        if not self.current_node or self.current_node.node_type != NodeType.VARIABLE:
            self._update_status("Plot view only available for variables")
            return

        self.show_plot = not self.show_plot
        if self.show_plot:
            self._update_status("Plot view: ON (displaying ASCII plot)")
            # Refresh the details to show plot
            self.show_details(self.current_node)
        else:
            self._update_status("Plot view: OFF")
            # Refresh to show normal view
            self.show_details(self.current_node)

    def action_copy_info(self) -> None:
        """Copy current node information to clipboard."""
        if not self.current_node:
            self._update_status("No node selected")
            return

        try:
            # Build text content to copy
            content = f"Node: {self.current_node.name}\n"
            content += f"Type: {self.current_node.node_type.value}\n"
            content += f"Path: {self.current_node.path}\n"

            if self.current_node.metadata:
                content += "\nMetadata:\n"
                for key, val in self.current_node.metadata.items():
                    content += f"  {key}: {val}\n"

            if self.current_node.attributes:
                content += "\nAttributes:\n"
                for key, val in self.current_node.attributes.items():
                    content += f"  {key}: {val}\n"

            # Try to copy to clipboard using common methods
            import subprocess
            # Try xclip (Linux), pbcopy (Mac), or clip (Windows)
            try:
                subprocess.run(['xclip', '-selection', 'clipboard'],
                             input=content.encode(), check=True)
                self._update_status(f"Copied {self.current_node.name} info to clipboard")
            except (FileNotFoundError, subprocess.CalledProcessError):
                try:
                    subprocess.run(['pbcopy'], input=content.encode(), check=True)
                    self._update_status(f"Copied {self.current_node.name} info to clipboard")
                except (FileNotFoundError, subprocess.CalledProcessError):
                    try:
                        subprocess.run(['clip'], input=content.encode(), check=True)
                        self._update_status(f"Copied {self.current_node.name} info to clipboard")
                    except (FileNotFoundError, subprocess.CalledProcessError):
                        # Fallback: save to temp file
                        import tempfile
                        with tempfile.NamedTemporaryFile(mode='w', delete=False, suffix='.txt') as f:
                            f.write(content)
                            self._update_status(f"Info saved to {f.name}")
        except Exception as e:
            self._update_status(f"Copy failed: {e}")

    def _update_status(self, msg: str) -> None:
        self.query_one("#top-bar", Static).update(msg)
