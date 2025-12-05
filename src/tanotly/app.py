"""Simplified Tanotly application - more reliable."""

from pathlib import Path
from typing import Optional

from textual import on, work
from textual.app import App, ComposeResult
from textual.binding import Binding
from textual.containers import Horizontal, Vertical, VerticalScroll
from textual.widgets import Footer, Header, Input, Static, Tree

import numpy as np
import xarray as xr

from tanotly.data import DataReader, DatasetInfo, DataNode
from tanotly.data.models import NodeType
from tanotly.utils.search import search_nodes


class TanotlyApp(App[None]):
    """Simplified Tanotly application."""

    # Disable mouse support to avoid terminal issues
    ENABLE_COMMAND_PALETTE = False

    CSS = """
    #top-bar {
        dock: top;
        height: 1;
        background: $accent;
        content-align: center middle;
    }

    #main {
        height: 1fr;
    }

    #tree-container {
        width: 40%;
        border-right: solid $accent;
    }

    #detail-container {
        width: 50%;
        padding: 1;
    }

    Tree {
        height: 100%;
    }

    VerticalScroll {
        height: 100%;
    }
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
            if child.node_type == NodeType.ATTRIBUTE:
                continue
            label = self._format_label(child)
            node = tree_node.add(label, data=child)
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
        """Format node label."""
        if node.node_type == NodeType.GROUP:
            # Count non-attribute children
            child_count = sum(1 for c in node.children if c.node_type != NodeType.ATTRIBUTE)
            return f"[yellow]{node.name}[/yellow] ({child_count})"
        elif node.node_type == NodeType.VARIABLE:
            shape = node.metadata.get("shape", "")
            dtype = node.metadata.get("dtype", "")
            data_type = self._get_data_type_label(node)
            if shape and dtype:
                shape_str = "×".join(str(s) for s in shape)
                return f"[cyan]{node.name}[/cyan] [{shape_str}] {data_type} {dtype}"
            return f"[cyan]{node.name}[/cyan]"
        elif node.node_type == NodeType.DIMENSION:
            return f"[blue]{node.name}[/blue]"
        else:
            return node.name

    def on_tree_node_highlighted(self, event: Tree.NodeHighlighted) -> None:  # type: ignore
        """Show details when node is highlighted."""
        if event.node.data:
            self.show_details(event.node.data)

    def show_details(self, node: DataNode) -> None:
        """Display node details."""
        detail = self.query_one("#detail", Static)

        # Build content
        content = f"[bold cyan]{node.name}[/bold cyan]\n\n"
        content += f"[dim]Type:[/dim] {node.node_type.value}\n"
        content += f"[dim]Path:[/dim] {node.path}\n\n"

        # Metadata
        if node.metadata:
            content += "[yellow]Metadata:[/yellow]\n"
            for key, val in node.metadata.items():
                if key == "shape":
                    content += f"  {key}: {' × '.join(str(s) for s in val)}\n"
                elif key == "dims":
                    content += f"  {key}: ({', '.join(str(d) for d in val)})\n"
                else:
                    content += f"  {key}: {val}\n"
            content += "\n"

        # Attributes (Panoply-style with : prefix)
        if node.attributes:
            content += "[magenta]Attributes:[/magenta]\n"
            for key, val in node.attributes.items():
                val_str = str(val)[:100]
                content += f"  :{key} = {val_str}\n"
            content += "\n"

        # Data preview for variables
        if node.node_type == NodeType.VARIABLE and self.dataset:
            content += self._get_data_preview(node)

        detail.update(content)

    def _get_data_preview(self, node: DataNode) -> str:
        """Get data preview for a variable."""
        try:
            # Extract variable name
            var_path = node.path
            if "/variables/" in var_path:
                var_name = var_path.split("/variables/")[1]
            elif "/coordinates/" in var_path:
                var_name = var_path.split("/coordinates/")[1]
            else:
                var_name = node.name

            # Load data
            ds = xr.open_dataset(self.dataset.file_path)
            if var_name not in ds.variables:
                ds.close()
                return ""

            var = ds[var_name]
            data = var.values

            content = "[green]Data Preview:[/green]\n"

            # Statistics for numeric data
            if np.issubdtype(data.dtype, np.number):
                content += f"  Min:  {np.nanmin(data):.6g}\n"
                content += f"  Max:  {np.nanmax(data):.6g}\n"
                content += f"  Mean: {np.nanmean(data):.6g}\n"
                if data.size > 1:
                    content += f"  Std:  {np.nanstd(data):.6g}\n"
                content += "\n"

            # Sample values
            content += "[dim]Sample values:[/dim]\n"
            if data.size <= 20:
                content += f"  {data}\n"
            elif data.ndim == 1:
                content += f"  {data[:20]} ...\n"
                content += f"  ({data.size} total elements)\n"
            elif data.ndim == 2:
                content += f"  {data[:5, :5]}\n"
                content += f"  ({data.shape[0]} × {data.shape[1]} total)\n"
            else:
                content += f"  First slice: {data.flat[:20]}\n"
                shape_str = " × ".join(str(s) for s in data.shape)
                content += f"  (shape: {shape_str})\n"

            ds.close()
            return content

        except Exception as e:
            return f"[dim red]Could not load data: {e}[/dim red]\n"

    def _collect_all_tree_nodes(self, tree_node, result_list):
        """Recursively collect all tree nodes."""
        result_list.append(tree_node)
        for child in tree_node.children:
            self._collect_all_tree_nodes(child, result_list)

    def action_start_search(self) -> None:
        """Start a search using the footer."""
        if not self.full_dataset_info:
            return

        # Prompt for search query
        self._update_status("Search: ")

        def handle_input(result: str) -> None:
            if result:
                self.search_query = result
                self._perform_search()
            else:
                self._update_status(f"{Path(self.file_path).name} | Use / to search")

        # Use textual's built-in prompt
        from textual.widgets import Label
        from textual.screen import ModalScreen
        from textual.containers import Container

        class SearchPrompt(ModalScreen):
            def compose(self):
                with Container():
                    yield Input(placeholder="Search...", id="search-input")

            def on_mount(self):
                self.query_one("#search-input", Input).focus()

            def on_input_submitted(self, event):
                self.dismiss(event.value)

        self.push_screen(SearchPrompt(), handle_input)

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

    def _update_status(self, msg: str) -> None:
        self.query_one("#top-bar", Static).update(msg)
