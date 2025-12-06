"""Tanotly - Terminal-based netCDF/HDF5 data viewer."""

from pathlib import Path

from textual.app import App, ComposeResult
from textual.binding import Binding
from textual.containers import Horizontal, Vertical, VerticalScroll
from textual.widgets import Footer, Static, Tree

from .clipboard import copy_to_clipboard, format_tree_text, format_node_info
from .config import APP_CSS
from .data import DataReader, DatasetInfo, DataNode
from .data.models import NodeType
from .details import render_details
from .search import SearchState, perform_search, jump_to_match, format_search_status
from .tree import populate_tree, format_label


class TanotlyApp(App[None]):
    """Terminal-based netCDF/HDF5 data viewer with tree navigation."""

    ENABLE_COMMAND_PALETTE = False
    CSS = APP_CSS

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

    def __init__(self, file_path: str | None = None):
        super().__init__()
        self.file_path = file_path
        self.dataset: DatasetInfo | None = None
        self.current_node: DataNode | None = None
        self.search = SearchState()
        self.show_plot = False
        self.show_preview = True
        self._nav_timer = None

    def compose(self) -> ComposeResult:
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
        yield Static("Ready", id="status-bar")
        yield Footer()

    def on_mount(self) -> None:
        if self.file_path:
            self._load_file(self.file_path)
        self.query_one("#tree", Tree).focus()

    def _load_file(self, path: str) -> None:
        """Load and display a data file."""
        try:
            self._status(f"Loading {Path(path).name}...")
            self.dataset = DataReader.read_file(path)

            tree = self.query_one("#tree", Tree)
            tree.clear()
            tree.show_root = True
            tree.show_guides = True
            tree.guide_depth = 4

            tree.root.label = format_label(self.dataset.root_node)
            tree.root.data = self.dataset.root_node
            populate_tree(tree.root, self.dataset.root_node)
            tree.root.expand()

            tree.focus()
            tree.select_node(tree.root)
            self._status(f"{Path(path).name} loaded")
        except Exception as e:
            self._status(f"Error: {e}")

    # Event handlers

    def on_tree_node_highlighted(self, event: Tree.NodeHighlighted) -> None:
        """Show details when node is highlighted (debounced)."""
        if not event.node.data:
            return
        if self._nav_timer:
            try:
                self._nav_timer.stop()
            except RuntimeError:
                pass
        self._nav_timer = self.set_timer(0.05, lambda: self._show_details(event.node.data))

    def on_key(self, event) -> None:
        """Handle search input keys."""
        if not self.search.is_active:
            return

        if event.key == "enter":
            if self.search.buffer:
                self.search.query = self.search.buffer
                perform_search(self.query_one("#tree", Tree), self.search)
                if self.search.matches:
                    jump_to_match(self.query_one("#tree", Tree), self.search)
            self.search.buffer = None
            self._status(format_search_status(self.search) or self._default_status())
            event.prevent_default()
            event.stop()
        elif event.key == "escape":
            self.search.cancel()
            self._status(self._default_status())
            event.prevent_default()
            event.stop()
        elif event.key == "backspace":
            if self.search.buffer:
                self.search.buffer = self.search.buffer[:-1]
            self._status(format_search_status(self.search))
            event.prevent_default()
            event.stop()
        elif len(event.key) == 1 and event.key.isprintable():
            self.search.buffer += event.key
            self._status(format_search_status(self.search))
            event.prevent_default()
            event.stop()

    # Actions

    def action_start_search(self) -> None:
        if not self.dataset:
            return
        self.search.start()
        self._status(format_search_status(self.search))

    def action_next_match(self) -> None:
        if self.search.matches:
            self.search.next_match()
            jump_to_match(self.query_one("#tree", Tree), self.search)
            self._status(format_search_status(self.search))

    def action_prev_match(self) -> None:
        if self.search.matches:
            self.search.prev_match()
            jump_to_match(self.query_one("#tree", Tree), self.search)
            self._status(format_search_status(self.search))

    def action_cancel_search(self) -> None:
        self.search.cancel()
        self._status(self._default_status())

    def action_cursor_up(self) -> None:
        self.query_one("#tree", Tree).action_cursor_up()

    def action_cursor_down(self) -> None:
        self.query_one("#tree", Tree).action_cursor_down()

    def action_cursor_left(self) -> None:
        tree = self.query_one("#tree", Tree)
        if tree.cursor_node:
            tree.cursor_node.collapse()

    def action_cursor_right(self) -> None:
        tree = self.query_one("#tree", Tree)
        if tree.cursor_node:
            tree.cursor_node.expand()

    def action_toggle_plot(self) -> None:
        if not self.current_node or self.current_node.node_type != NodeType.VARIABLE:
            self._status("Plot only available for variables")
            return
        self.show_plot = not self.show_plot
        self._status(f"Plot: {'ON' if self.show_plot else 'OFF'}")
        self._show_details(self.current_node)

    def action_toggle_preview(self) -> None:
        self.show_preview = not self.show_preview
        detail = self.query_one("#detail-container", VerticalScroll)
        tree_cont = self.query_one("#tree-container", Vertical)

        if self.show_preview:
            detail.remove_class("hidden")
            tree_cont.styles.width = "50%"
            detail.styles.width = "50%"
            self._status("Preview: ON")
        else:
            tree_cont.styles.width = "100%"
            detail.styles.width = "0"
            self._status("Preview: OFF")

    def action_scroll_preview_down(self) -> None:
        try:
            self.query_one("#detail-container", VerticalScroll).scroll_relative(y=5, animate=False)
        except Exception:
            pass

    def action_scroll_preview_up(self) -> None:
        try:
            self.query_one("#detail-container", VerticalScroll).scroll_relative(y=-5, animate=False)
        except Exception:
            pass

    def action_copy_tree(self) -> None:
        if not self.dataset:
            self._status("No file loaded")
            return
        try:
            content = f"Tree Structure: {self.dataset.file_path}\n{'=' * 80}\n\n"
            content += format_tree_text(self.query_one("#tree", Tree).root)
            success, msg = copy_to_clipboard(content)
            self._status("Tree copied!" if success else msg)
        except Exception as e:
            self._status(f"Copy failed: {e}")

    def action_copy_info(self) -> None:
        if not self.current_node:
            self._status("No node selected")
            return
        content = format_node_info(self.current_node)
        success, msg = copy_to_clipboard(content)
        self._status(f"Copied {self.current_node.name}!" if success else msg)

    # Helpers

    def _show_details(self, node: DataNode) -> None:
        """Display node details."""
        self.current_node = node
        try:
            container = self.query_one("#detail-container", VerticalScroll)
            render_details(container, node, self.dataset, self.show_plot)
        except Exception as e:
            try:
                container = self.query_one("#detail-container", VerticalScroll)
                container.remove_children()
                container.mount(Static(f"[red]Error: {e}[/red]"))
            except:
                pass

    def _status(self, msg: str) -> None:
        """Update status bar."""
        self.query_one("#status-bar", Static).update(msg)

    def _default_status(self) -> str:
        """Get default status message."""
        return Path(self.file_path).name if self.file_path else "Ready"
