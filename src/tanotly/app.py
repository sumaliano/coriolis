"""Simplified Tanotly application - more reliable."""

from pathlib import Path
from typing import Optional

from textual.app import App, ComposeResult
from textual.binding import Binding
from textual.containers import Horizontal, Vertical, VerticalScroll
from textual.widgets import Footer, Input, Static, Tree

import numpy as np
import xarray as xr

from .data import DataReader, DatasetInfo, DataNode
from .data.models import NodeType
from .visualization import DataVisualizer, format_statistics, format_sample_values


class TanotlyApp(App[None]):
    """Simplified Tanotly application."""

    # Disable mouse support to avoid terminal issues
    ENABLE_COMMAND_PALETTE = False

    CSS = """
    /* ============================================================================
     * COLOR CUSTOMIZATION GUIDE
     * ============================================================================
     *
     * Textual provides built-in color variables that automatically adapt to
     * light/dark themes. You can customize the app's appearance by changing
     * colors in the CSS rules below.
     *
     * AVAILABLE COLOR VARIABLES (Textual Built-in):
     * ------------------------------------------------
     * $accent           - Cyan (default) - highlights, borders, links
     * $accent-lighten-1 - Lighter cyan variant
     * $accent-lighten-2 - Even lighter cyan
     * $accent-lighten-3 - Lightest cyan
     * $accent-darken-1  - Darker cyan variant
     * $accent-darken-2  - Even darker cyan
     * $accent-darken-3  - Darkest cyan
     *
     * $primary          - Primary brand color
     * $secondary        - Secondary color
     * $background       - Main background color
     * $surface          - Widget background color
     * $panel            - Panel background color (slightly different from surface)
     *
     * $success          - Green - success states, positive actions
     * $warning          - Yellow - warnings, caution states
     * $error            - Red - errors, destructive actions
     *
     * $text             - Default text color (auto: white on dark, black on light)
     * $text-muted       - Dimmed text color
     * $text-disabled    - Disabled text color
     *
     * LITERAL COLOR VALUES (can be used anywhere):
     * ------------------------------------------------
     * Named colors: black, white, red, green, blue, yellow, magenta, cyan
     * Hex colors:   #000000, #ffffff, #ff0000, etc.
     * RGB colors:   rgb(255, 0, 0), rgb(0, 255, 0), etc.
     *
     * RICH MARKUP COLORS (used in text content, not CSS):
     * ------------------------------------------------
     * Used in Static widget content and labels with [color]text[/color] syntax:
     * black, red, green, yellow, blue, magenta, cyan, white
     * bright_black, bright_red, bright_green, bright_yellow,
     * bright_blue, bright_magenta, bright_cyan, bright_white
     *
     * Examples: [cyan]Variable[/cyan], [bold yellow]Title[/bold yellow]
     *
     * WHERE TO CUSTOMIZE:
     * ------------------------------------------------
     * 1. Background colors: Change 'background: black' to any color
     * 2. Border colors: Change 'border-right: solid $accent' to use different color
     * 3. Text colors: Change 'color: $text' to use different color
     * 4. Tree/node colors: See _format_label() method for Rich markup colors
     * 5. Status bar: Change #status-bar background and color below
     *
     * ============================================================================
     */

    /* Main content area - fills space between top and status bar */
    #main {
        height: 1fr;
    }

    /* Status bar - positioned above footer, not docked
     * CUSTOMIZE: Change 'background' to modify status bar background color
     * CUSTOMIZE: Change 'color' to modify status bar text color
     */
    #status-bar {
        height: 1;
        background: $panel;     /* Try: $surface, black, #1a1a1a, etc. */
        color: $text;           /* Try: white, $accent, yellow, etc. */
        content-align: left middle;
        padding-left: 1;
    }

    /* Tree panel - left side (configurable width)
     * CUSTOMIZE: Change 'background' to modify tree panel background
     * CUSTOMIZE: Change 'border-right' color to modify border color
     */
    #tree-container {
        width: 50%;
        border-right: solid $accent;  /* Try: solid green, solid #00ff00, etc. */
        background: black;            /* Try: $background, $surface, #0a0a0a, etc. */
    }

    #tree-container.hidden {
        display: none;
    }

    /* Detail panel - right side (fills remaining space)
     * CUSTOMIZE: Change 'background' to modify detail panel background
     */
    #detail-container {
        width: 50%;
        padding: 1;
        background: black;  /* Try: $background, $surface, #0a0a0a, etc. */
    }

    #detail-container.full-width {
        width: 100%;
    }

    Tree {
        height: 100%;
        scrollbar-gutter: stable;
        background: black;
    }

    /* Wrap tree labels that don't fit */
    Tree > .tree--label {
        text-overflow: ellipsis;
        overflow: hidden;
    }

    /* Hide guide for leaf nodes (no children) */
    Tree > .tree--guides {
        color: $accent-darken-1;
    }

    VerticalScroll {
        height: 100%;
        overflow-y: auto;
    }

    #detail-container {
        scrollbar-gutter: stable;
    }

    /* Plot widgets - ensure they have proper height */
    DataPlot1D, DataPlot2D {
        height: auto;
        min-height: 20;
        width: 100%;
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
        Binding("escape", "cancel_search", "Cancel"),
        Binding("j", "cursor_down", "Down", show=False),
        Binding("k", "cursor_up", "Up", show=False),
        Binding("h", "cursor_left", "Left", show=False),
        Binding("l", "cursor_right", "Right", show=False),
        Binding("p", "toggle_plot", "Plot"),
        Binding("y", "copy_info", "Copy Info", show=False),
        Binding("c", "copy_tree", "Copy Tree"),
        Binding("t", "toggle_preview", "Toggle Preview"),
        Binding("ctrl+d", "scroll_preview_down", "Scroll Down", show=False),
        Binding("ctrl+u", "scroll_preview_up", "Scroll Up", show=False),
        Binding("shift+j", "scroll_preview_down", "Scroll Down", show=False),
        Binding("shift+k", "scroll_preview_up", "Scroll Up", show=False),
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
        self.search_buffer = ""  # What user is typing
        # Plot mode
        self.show_plot = False
        self.current_node: Optional[DataNode] = None
        # Debounce timer for navigation
        self._debounce_timer = None
        # Preview toggle
        self.show_preview = True

    def compose(self) -> ComposeResult:
        # Main content area
        with Horizontal(id="main"):
            with Vertical(id="tree-container"):
                yield Tree("Data", id="tree")
            with VerticalScroll(id="detail-container"):
                yield Static(
                    "[bold yellow]Welcome to Tanotly![/bold yellow]\n\n"
                    "[dim]Navigation: ↑/↓/j/k, ←/→/h/l to expand\n"
                    "Search: / to search, n/N for next/prev\n"
                    "Plot: p to toggle visualization\n"
                    "Toggle preview: t\n"
                    "Quit: q[/dim]",
                    id="welcome"
                )

        # Status bar above footer
        yield Static("Ready", id="status-bar")

        # Footer at bottom
        yield Footer()

    def on_mount(self) -> None:
        if self.file_path:
            self.load_file(self.file_path)
        # Focus tree so arrow keys work immediately
        tree = self.query_one("#tree", Tree)
        tree.focus()

    def load_file(self, path: str) -> None:
        """Load and display file with improved performance."""
        try:
            self._update_status(f"Loading {Path(path).name}...")

            # Load file data
            self.full_dataset_info = DataReader.read_file(path)
            self.dataset = self.full_dataset_info

            # Setup tree
            tree = self.query_one("#tree", Tree)
            tree.clear()
            tree.show_root = True
            tree.show_guides = True
            tree.guide_depth = 4

            # Set root label and data
            tree.root.label = self._format_label(self.dataset.root_node)
            tree.root.data = self.dataset.root_node

            # Populate tree structure
            self._populate_tree(tree.root, self.dataset.root_node)

            # Expand only root (not all children) for faster startup
            tree.root.expand()

            # Focus tree and select root node
            tree.focus()
            tree.select_node(tree.root)

            # Show ready status
            var_count = len(self.dataset.variables) if self.dataset.variables else 0
            self._update_status(
                f"{Path(path).name} loaded | {var_count} variables | Press / to search"
            )
        except Exception as e:
            self._update_status(f"Error loading file: {e}")

    def _populate_tree(self, tree_node, data_node: DataNode) -> None:
        """Recursively populate tree."""
        # First add non-attribute children (groups, variables, dimensions)
        non_attr_children = [c for c in data_node.children if c.node_type != NodeType.ATTRIBUTE]
        for child in non_attr_children:
            label = self._format_label(child)
            # Only allow expansion if node has children
            allow_expand = len(child.children) > 0
            node = tree_node.add(label, data=child, allow_expand=allow_expand)
            if child.children:
                self._populate_tree(node, child)

        # Then add attributes as an "Attributes" group if there are any
        attr_children = [c for c in data_node.children if c.node_type == NodeType.ATTRIBUTE]
        if attr_children:
            # Create an attributes group node
            attrs_label = f"[magenta]🏷️  Attributes ({len(attr_children)})[/magenta]"
            attrs_node = tree_node.add(attrs_label, data=None, allow_expand=True)

            # Add each attribute as a child
            for attr in attr_children:
                attr_label = self._format_label(attr)
                attrs_node.add(attr_label, data=attr, allow_expand=False)

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
        """
        Format node label WITHOUT icons (Tree widget already provides arrows).

        COLOR CUSTOMIZATION:
        -------------------
        Change the colors in the [color]...[/color] tags below to customize
        tree node appearance. Available Rich markup colors:

        - Basic: black, red, green, yellow, blue, magenta, cyan, white
        - Bright: bright_black, bright_red, bright_green, bright_yellow,
                  bright_blue, bright_magenta, bright_cyan, bright_white
        - Modifiers: bold, italic, underline, dim

        Current color scheme:
        - ROOT nodes:      bold magenta
        - GROUP nodes:     yellow
        - VARIABLE nodes:  cyan
        - DIMENSION nodes: blue
        - ATTRIBUTE nodes: magenta
        - Metadata text:   dim (gray)
        """
        if node.node_type == NodeType.ROOT:
            # Change 'magenta' to customize root node color
            return f"[bold magenta]{node.name}[/bold magenta]"

        elif node.node_type == NodeType.GROUP:
            # Count non-attribute children
            child_count = sum(1 for c in node.children if c.node_type != NodeType.ATTRIBUTE)
            # Change 'yellow' to customize group color
            return f"[yellow]{node.name}[/yellow] [dim]({child_count})[/dim]"

        elif node.node_type == NodeType.VARIABLE:
            shape = node.metadata.get("shape", "")
            dtype = node.metadata.get("dtype", "")
            data_type = self._get_data_type_label(node)

            if shape and dtype:
                shape_str = "×".join(str(s) for s in shape)
                # Change 'cyan' to customize variable color
                return f"[cyan]{node.name}[/cyan] [dim]({shape_str}) {data_type} {dtype}[/dim]"
            return f"[cyan]{node.name}[/cyan]"

        elif node.node_type == NodeType.DIMENSION:
            size = node.metadata.get("size", "")
            if size:
                # Change 'blue' to customize dimension color
                return f"[blue]{node.name}[/blue] [dim]({size})[/dim]"
            return f"[blue]{node.name}[/blue]"

        elif node.node_type == NodeType.ATTRIBUTE:
            # Attribute name already includes the value (e.g., "units: meters")
            # Escape any Rich markup in the name
            name_escaped = node.name.replace('[', '\\[').replace(']', '\\]')
            # Truncate if too long
            if len(name_escaped) > 60:
                name_escaped = name_escaped[:57] + "..."
            # Change 'magenta' to customize attribute color
            return f"[magenta]{name_escaped}[/magenta]"

        else:
            return node.name

    def on_tree_node_highlighted(self, event: Tree.NodeHighlighted) -> None:  # type: ignore
        """Show details when node is highlighted."""
        if event.node.data:
            # Use call_later to debounce rapid navigation
            try:
                self._debounce_timer.stop()
            except (AttributeError, RuntimeError):
                pass
            self._debounce_timer = self.set_timer(0.05, lambda: self.show_details(event.node.data))

    def show_details(self, node: DataNode) -> None:
        """Display node details."""
        self.current_node = node  # Track current node for plot/copy

        try:
            detail_container = self.query_one("#detail-container", VerticalScroll)

            # Clear existing content
            detail_container.remove_children()

            # Build header
            icon = self._get_node_icon(node)
            header_content = f"{icon} [bold cyan]{node.name}[/bold cyan]\n"
            header_content += "-" * 60 + "\n\n"

            # Type with color coding
            type_color = self._get_type_color(node.node_type)
            header_content += f"[{type_color}]● Type:[/{type_color}] {node.node_type.value}\n"
            header_content += f"[dim]● Path:[/dim] [cyan]{node.path}[/cyan]\n\n"

            detail_container.mount(Static(header_content))

            # Metadata section
            if node.metadata:
                metadata_content = "[bold yellow]📊 Metadata[/bold yellow]\n"
                metadata_content += "-" * 60 + "\n"
                for key, val in node.metadata.items():
                    if key == "shape":
                        shape_str = " × ".join(str(s) for s in val)
                        metadata_content += f"  [cyan]▸ {key}:[/cyan] {shape_str}\n"
                    elif key == "dims":
                        dims_str = ", ".join(str(d) for d in val)
                        metadata_content += f"  [cyan]▸ {key}:[/cyan] ({dims_str})\n"
                    elif key == "size":
                        size_formatted = f"{val:,}" if isinstance(val, int) else str(val)
                        metadata_content += f"  [cyan]▸ {key}:[/cyan] {size_formatted}\n"
                    else:
                        metadata_content += f"  [cyan]▸ {key}:[/cyan] {val}\n"
                metadata_content += "\n"
                detail_container.mount(Static(metadata_content))

            # Attributes section
            if node.attributes:
                attr_content = "[bold magenta]🏷️  Attributes[/bold magenta]\n"
                attr_content += "-" * 60 + "\n"
                for key, val in node.attributes.items():
                    val_str = str(val)
                    if len(val_str) > 80:
                        val_str = val_str[:77] + "..."
                    # Escape any Rich markup in the value to prevent tag mismatch errors
                    val_str = val_str.replace('[', '\\[').replace(']', '\\]')
                    attr_content += f"  [magenta]:{key}[/magenta] = {val_str}\n"
                attr_content += "\n"
                detail_container.mount(Static(attr_content))

            # Data preview for variables
            if node.node_type == NodeType.VARIABLE and self.dataset:
                try:
                    self._add_data_preview(node, detail_container)
                except Exception as e:
                    error_msg = f"\n[red]Error loading data: {str(e)}[/red]\n"
                    detail_container.mount(Static(error_msg))
        except Exception as e:
            # Fallback if entire method fails
            try:
                detail_container = self.query_one("#detail-container", VerticalScroll)
                detail_container.remove_children()
                fallback_msg = f"[red]Display error[/red]\n\n{node.name}\n{node.path}\n\n{str(e)}"
                detail_container.mount(Static(fallback_msg))
            except:
                pass  # Give up gracefully

    def _add_data_preview(self, node: DataNode, container: VerticalScroll) -> None:
        """Add data preview widgets to the container."""
        # Extract variable path
        var_path = node.path
        if "/variables/" in var_path:
            var_name = var_path.split("/variables/")[1]
        elif "/coordinates/" in var_path:
            var_name = var_path.split("/coordinates/")[1]
        else:
            var_name = node.name

        # Load data
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

        # Try netCDF4 if xarray failed
        if data is None:
            import netCDF4 as nc
            with nc.Dataset(self.dataset.file_path, 'r') as ncds:
                parts = [p for p in var_path.split('/') if p]
                obj = ncds
                for i, part in enumerate(parts[:-1]):
                    if part in obj.groups:
                        obj = obj.groups[part]
                    elif part in obj.variables:
                        obj = obj.variables[part]
                        break

                var_name_final = parts[-1]
                if hasattr(obj, 'variables') and var_name_final in obj.variables:
                    var = obj.variables[var_name_final]
                elif hasattr(obj, '__getitem__'):
                    var = obj[var_name_final]
                else:
                    raise KeyError(f"Cannot find variable at path: {var_path}")

                data = var[:]

        # Add data preview header
        container.mount(Static("\n[bold green]📈 Data Preview[/bold green]\n" + "-" * 60))

        # Add Textual visualization widgets if plot mode is on
        if self.show_plot and np.issubdtype(data.dtype, np.number) and data.size > 0:
            container.mount(Static(" "))  # Spacer (needs non-empty content)
            for widget in DataVisualizer.create_visualization(data, container_width=50):
                container.mount(widget)
            container.mount(Static(" "))  # Spacer (needs non-empty content)

        # Add statistics for numeric data
        if np.issubdtype(data.dtype, np.number):
            stats_content = format_statistics(data)
            if stats_content and stats_content.strip():
                container.mount(Static(stats_content))

        # Add sample values
        container.mount(Static("[cyan]Sample Values:[/cyan]"))
        sample_content = format_sample_values(data, max_lines=8)
        if sample_content and sample_content.strip():
            container.mount(Static(sample_content))
        else:
            container.mount(Static("[dim]No sample data available[/dim]"))

    def _get_node_icon(self, node: DataNode) -> str:
        """Get icon for node type (for detail panel only)."""
        icons = {
            NodeType.ROOT: "🏠 ",
            NodeType.GROUP: "📂 ",
            NodeType.VARIABLE: "🌡️ ",
            NodeType.DIMENSION: "📏 ",
            NodeType.ATTRIBUTE: "🏷️ ",
        }
        return icons.get(node.node_type, "● ")

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


    def _collect_all_tree_nodes(self, tree_node, result_list):
        """Recursively collect all tree nodes."""
        result_list.append(tree_node)
        for child in tree_node.children:
            self._collect_all_tree_nodes(child, result_list)

    def action_start_search(self) -> None:
        """Start search mode - just update status."""
        if not self.full_dataset_info:
            return
        self.search_buffer = ""
        self._update_status("🔍 Search: _ | Type to search, Enter to find, Esc to cancel")

    def on_key(self, event) -> None:
        """Handle key presses for search."""
        # Only handle if we're in search mode (buffer is not empty or we just started)
        if self.search_buffer is not None and len(self.search_buffer) >= 0:
            if event.key == "enter":
                # Execute search
                if self.search_buffer:
                    self.search_query = self.search_buffer
                    self._perform_search()
                self.search_buffer = None
                event.prevent_default()
                event.stop()
            elif event.key == "escape":
                # Cancel search
                self.search_buffer = None
                self.action_cancel_search()
                event.prevent_default()
                event.stop()
            elif event.key == "backspace":
                # Remove last character
                if self.search_buffer:
                    self.search_buffer = self.search_buffer[:-1]
                    self._update_status(f"🔍 Search: {self.search_buffer}_ | Enter to find, Esc to cancel")
                    event.prevent_default()
                    event.stop()
            elif len(event.key) == 1 and event.key.isprintable():
                # Add character to buffer
                self.search_buffer += event.key
                self._update_status(f"🔍 Search: {self.search_buffer}_ | Enter to find, Esc to cancel")
                event.prevent_default()
                event.stop()

    def _perform_search(self):
        """Find all matches for current search query in the entire tree."""
        if not self.search_query:
            return

        tree = self.query_one("#tree", Tree)

        # First, expand ALL nodes to make everything searchable
        self._expand_all_nodes(tree.root)

        # Now collect all nodes (including newly expanded ones)
        all_nodes = []
        self._collect_all_tree_nodes(tree.root, all_nodes)

        self.search_matches = []
        query_lower = self.search_query.lower()

        for tree_node in all_nodes:
            if tree_node.data:
                # Search in node name
                if query_lower in tree_node.data.name.lower():
                    self.search_matches.append(tree_node)
                    continue

                # Search in node path
                if query_lower in tree_node.data.path.lower():
                    self.search_matches.append(tree_node)
                    continue

                # Search in attribute KEYS only (not values)
                if tree_node.data.attributes:
                    for key in tree_node.data.attributes.keys():
                        if query_lower in key.lower():
                            self.search_matches.append(tree_node)
                            break

                # Search in metadata KEYS only (not values)
                if tree_node.data.metadata:
                    for key in tree_node.data.metadata.keys():
                        if query_lower in str(key).lower():
                            self.search_matches.append(tree_node)
                            break

        if self.search_matches:
            self.current_match_idx = 0
            self._jump_to_current_match()
            # Status is updated in _jump_to_current_match
        else:
            self.current_match_idx = -1
            self._update_status(f"🔍 No matches for '{self.search_query}' | Esc to exit search")

    def _expand_all_nodes(self, tree_node):
        """Recursively expand all nodes in the tree."""
        tree_node.expand()
        for child in tree_node.children:
            self._expand_all_nodes(child)

    def _jump_to_current_match(self):
        """Jump to the current match in the search results."""
        if 0 <= self.current_match_idx < len(self.search_matches):
            tree = self.query_one("#tree", Tree)
            match_node = self.search_matches[self.current_match_idx]

            # Expand all parent nodes to make the match visible (they should already be expanded)
            parent = match_node.parent
            while parent is not None:
                parent.expand()
                parent = parent.parent

            tree.select_node(match_node)
            tree.scroll_to_node(match_node)

            # Update the status bar to show current match position
            match_num = self.current_match_idx + 1
            total_matches = len(self.search_matches)
            self._update_status(
                f"🔍 Match {match_num}/{total_matches}: '{self.search_query}' | n=next N=prev Esc=exit"
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

    def action_cancel_search(self) -> None:
        """Cancel/exit search mode."""
        self.search_buffer = None
        self.search_query = ""
        self.search_matches = []
        self.current_match_idx = -1

        # Clear status
        if self.file_path:
            var_count = len(self.dataset.variables) if self.dataset and self.dataset.variables else 0
            self._update_status(f"{Path(self.file_path).name} | {var_count} variables | Press / to search")
        else:
            self._update_status("Ready | Press / to search")

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

    def action_toggle_preview(self) -> None:
        """Toggle preview pane visibility."""
        self.show_preview = not self.show_preview
        detail_container = self.query_one("#detail-container", VerticalScroll)
        tree_container = self.query_one("#tree-container", Vertical)

        if self.show_preview:
            # Show preview pane
            detail_container.remove_class("hidden")
            tree_container.styles.width = "50%"
            detail_container.styles.width = "50%"
            self._update_status("Preview pane: ON")
        else:
            # Hide preview pane, expand tree to full width
            tree_container.styles.width = "100%"
            detail_container.styles.width = "0"
            self._update_status("Preview pane: OFF (tree full width)")

    def action_scroll_preview_down(self) -> None:
        """Scroll detail container down (Ctrl+d or Shift+j)."""
        try:
            detail_container = self.query_one("#detail-container", VerticalScroll)
            detail_container.scroll_relative(y=5, animate=False)
        except Exception:
            pass

    def action_scroll_preview_up(self) -> None:
        """Scroll detail container up (Ctrl+u or Shift+k)."""
        try:
            detail_container = self.query_one("#detail-container", VerticalScroll)
            detail_container.scroll_relative(y=-5, animate=False)
        except Exception:
            pass

    def action_copy_tree(self) -> None:
        """Copy the entire tree structure as text."""
        if not self.dataset:
            self._update_status("No file loaded")
            return

        try:
            tree = self.query_one("#tree", Tree)

            # Build tree text representation
            content = f"Tree Structure: {self.dataset.file_path}\n"
            content += "=" * 80 + "\n\n"

            def format_tree_node(tree_node, prefix="", is_last=True):
                """Recursively format tree nodes as text."""
                result = ""
                if tree_node.data:
                    # Get connector
                    connector = "└── " if is_last else "├── "
                    # Strip markup from label for plain text
                    import re
                    label = tree_node.label
                    if hasattr(label, 'plain'):
                        label_text = label.plain
                    else:
                        # Remove Rich markup
                        label_text = re.sub(r'\[.*?\]', '', str(label))

                    result += prefix + connector + label_text + "\n"

                    # Process children
                    if tree_node.children:
                        extension = "    " if is_last else "│   "
                        for i, child in enumerate(tree_node.children):
                            is_child_last = (i == len(tree_node.children) - 1)
                            result += format_tree_node(child, prefix + extension, is_child_last)
                else:
                    # Root node children
                    for i, child in enumerate(tree_node.children):
                        is_child_last = (i == len(tree_node.children) - 1)
                        result += format_tree_node(child, "", is_child_last)

                return result

            content += format_tree_node(tree.root)

            # Try to copy to clipboard
            import subprocess
            try:
                subprocess.run(['xclip', '-selection', 'clipboard'],
                             input=content.encode(), check=True)
                self._update_status("Tree structure copied to clipboard!")
            except (FileNotFoundError, subprocess.CalledProcessError):
                try:
                    subprocess.run(['pbcopy'], input=content.encode(), check=True)
                    self._update_status("Tree structure copied to clipboard!")
                except (FileNotFoundError, subprocess.CalledProcessError):
                    try:
                        subprocess.run(['clip'], input=content.encode(), check=True)
                        self._update_status("Tree structure copied to clipboard!")
                    except (FileNotFoundError, subprocess.CalledProcessError):
                        # Fallback: save to temp file
                        import tempfile
                        with tempfile.NamedTemporaryFile(mode='w', delete=False, suffix='_tree.txt') as f:
                            f.write(content)
                            self._update_status(f"Tree saved to {f.name}")
        except Exception as e:
            self._update_status(f"Copy tree failed: {e}")

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
        """Update the status bar with a message."""
        self.query_one("#status-bar", Static).update(msg)
