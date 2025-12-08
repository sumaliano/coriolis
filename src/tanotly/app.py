"""Tanotly - Terminal-based netCDF/HDF5 data viewer.

A TUI application for exploring scientific data files with:
- Tree navigation of file structure
- Variable preview with statistics
- Interactive plotting and data tables
- Vim-style keybindings
- Gruvbox dark/light themes
"""

from __future__ import annotations

import asyncio
from pathlib import Path
from typing import TYPE_CHECKING

from textual.app import App, ComposeResult
from textual.binding import Binding
from textual.containers import Horizontal, Vertical, VerticalScroll
from textual.widgets import Footer, Static, Tree
from textual.timer import Timer

from .clipboard import copy_to_clipboard, format_tree_text, format_node_info
from .config import CSS_PATH as APP_CSS_PATH, ThemeColors
from .data import DataReader, DatasetInfo, DataNode
from .data.models import NodeType
from .details import render_details, load_variable_data
from .search import SearchState, perform_search, jump_to_match, format_search_status
from .tree import populate_tree, format_label
from .plot_screen import PlotScreen

if TYPE_CHECKING:
    from textual.widgets._tree import TreeNode
    import numpy as np


# =============================================================================
# Constants
# =============================================================================

# Debounce delay for tree navigation (seconds)
NAV_DEBOUNCE_DELAY = 0.05

# Timeout for vim key sequences like 'gg' (seconds)
KEY_SEQUENCE_TIMEOUT = 0.5

# Number of lines to scroll for page up/down
PAGE_SCROLL_LINES = 15

# Number of lines to scroll for preview panel
PREVIEW_SCROLL_LINES = 5


# =============================================================================
# Main Application
# =============================================================================

class TanotlyApp(App[None]):
    """Terminal-based netCDF/HDF5 data viewer with tree navigation.
    
    Attributes:
        file_path: Path to the currently loaded file
        dataset: Loaded dataset information
        current_node: Currently selected tree node
        search: Search state manager
        show_preview: Whether the preview panel is visible
    """

    ENABLE_COMMAND_PALETTE = False
    CSS_PATH = APP_CSS_PATH
    
    # Default to dark theme
    dark: bool = True

    BINDINGS = [
        # Core actions
        Binding("q", "quit", "Quit"),
        Binding("escape", "cancel_search", "Cancel"),
        
        # Search
        Binding("/", "start_search", "Search"),
        Binding("n", "next_match", "Next", show=False),
        Binding("N", "prev_match", "Prev", show=False),
        
        # Vim-style navigation
        Binding("j", "cursor_down", "Down", show=False),
        Binding("k", "cursor_up", "Up", show=False),
        Binding("h", "cursor_left", "Collapse", show=False),
        Binding("l", "cursor_right", "Expand", show=False),
        Binding("g", "goto_top", "Top", show=False),
        Binding("G", "goto_bottom", "Bottom", show=False),
        Binding("ctrl+f", "page_down", "Page Down", show=False),
        Binding("ctrl+b", "page_up", "Page Up", show=False),
        Binding("z", "center_cursor", "Center", show=False),
        
        # Features
        Binding("p", "toggle_plot", "Plot"),
        Binding("d", "show_table", "Table"),
        Binding("t", "toggle_preview", "Preview"),
        Binding("T", "cycle_theme", "Theme"),
        
        # Clipboard
        Binding("y", "copy_info", "Copy", show=False),
        Binding("c", "copy_tree", "Copy Tree"),
        
        # Preview scrolling
        Binding("ctrl+d", "scroll_preview_down", "Scroll Down", show=False),
        Binding("ctrl+u", "scroll_preview_up", "Scroll Up", show=False),
        Binding("shift+j", "scroll_preview_down", "Scroll Down", show=False),
        Binding("shift+k", "scroll_preview_up", "Scroll Up", show=False),
    ]

    def __init__(self, file_path: str | None = None) -> None:
        """Initialize the application.
        
        Args:
            file_path: Optional path to a file to load on startup
        """
        super().__init__()
        self.file_path = file_path
        self.dataset: DatasetInfo | None = None
        self.current_node: DataNode | None = None
        self.search = SearchState()
        self.show_preview = True
        self._nav_timer: Timer | None = None
        self._pending_key: str | None = None

    # =========================================================================
    # Composition
    # =========================================================================

    def compose(self) -> ComposeResult:
        """Create the application layout."""
        with Horizontal(id="main"):
            with Vertical(id="tree-container"):
                yield Tree("Data", id="tree")
            with VerticalScroll(id="detail-container"):
                yield Static(self._welcome_message(), id="welcome")
        yield Static("Ready", id="status-bar")
        yield Footer()

    def _welcome_message(self) -> str:
        """Generate the welcome message."""
        return (
            "[bold yellow]Welcome to Tanotly![/bold yellow]\n\n"
            "[dim]Navigation: ↑/↓/j/k, ←/→/h/l to expand\n"
            "Search: / to search, n/N for next/prev\n"
            "Plot: p to visualize, d for data table\n"
            "Toggle preview: t, Theme: T\n"
            "Quit: q[/dim]"
        )

    # =========================================================================
    # Lifecycle Events
    # =========================================================================

    def on_mount(self) -> None:
        """Handle application mount - load file if provided."""
        self._get_tree().focus()
        if self.file_path:
            self.run_worker(self._load_file_async(self.file_path), name="file_loader")

    # =========================================================================
    # File Loading
    # =========================================================================

    async def _load_file_async(self, path: str) -> None:
        """Load and display a data file asynchronously.
        
        Args:
            path: Path to the file to load
        """
        filename = Path(path).name
        
        try:
            self._status(f"Loading {filename}...")
            await self._show_loading_message(filename)
            
            # Load file in background thread
            loop = asyncio.get_event_loop()
            self.dataset = await loop.run_in_executor(
                None, DataReader.read_file, path
            )
            
            # Populate tree
            self._populate_tree_from_dataset()
            
            # Show success message
            await self._show_loaded_message(filename)
            self._status(f"{filename} loaded")
            
        except Exception as e:
            await self._show_error_message(str(e))
            self._status(f"Error: {e}")

    def _populate_tree_from_dataset(self) -> None:
        """Populate the tree widget from the loaded dataset."""
        if not self.dataset:
            return
            
        tree = self._get_tree()
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

    async def _show_loading_message(self, filename: str) -> None:
        """Show loading indicator in detail panel."""
        detail = self._get_detail_container()
        detail.remove_children()
        await detail.mount(Static(
            f"[bold yellow]Loading {filename}...[/bold yellow]\n\n"
            "[dim]Please wait...[/dim]"
        ))

    async def _show_loaded_message(self, filename: str) -> None:
        """Show success message after loading."""
        detail = self._get_detail_container()
        detail.remove_children()
        await detail.mount(Static(
            f"[bold yellow]{filename} loaded![/bold yellow]\n\n"
            "[dim]Navigate the tree to explore data[/dim]"
        ))

    async def _show_error_message(self, error: str) -> None:
        """Show error message in detail panel."""
        detail = self._get_detail_container()
        detail.remove_children()
        await detail.mount(Static(
            f"[{ThemeColors.error()}]Error loading file:[/{ThemeColors.error()}]\n\n{error}"
        ))

    # =========================================================================
    # Event Handlers
    # =========================================================================

    def on_tree_node_highlighted(self, event: Tree.NodeHighlighted) -> None:
        """Handle tree node highlight - show details (debounced)."""
        if not event.node.data:
            return
        
        # Cancel previous timer
        if self._nav_timer:
            try:
                self._nav_timer.stop()
            except RuntimeError:
                pass
        
        # Set new debounced timer
        self._nav_timer = self.set_timer(
            NAV_DEBOUNCE_DELAY,
            lambda: self._show_details(event.node.data)
        )

    def on_key(self, event) -> None:
        """Handle keyboard input for search mode."""
        if not self.search.is_active:
            return

        key = event.key
        
        if key == "enter":
            self._handle_search_submit()
        elif key == "escape":
            self._handle_search_cancel()
        elif key == "backspace":
            self._handle_search_backspace()
        elif key == "underscore" or key == "_":
            # Handle underscore explicitly (Textual may report it as "underscore")
            self._handle_search_input("_")
        elif len(key) == 1 and key.isprintable():
            self._handle_search_input(key)
        else:
            return  # Don't consume unhandled keys
        
        event.prevent_default()
        event.stop()

    def _handle_search_submit(self) -> None:
        """Handle search submission (Enter key)."""
        if self.search.buffer:
            self.search.query = self.search.buffer
            perform_search(self._get_tree(), self.search)
            if self.search.matches:
                jump_to_match(self._get_tree(), self.search)
        self.search.buffer = None
        self._status(format_search_status(self.search) or self._default_status())

    def _handle_search_cancel(self) -> None:
        """Handle search cancellation (Escape key)."""
        self.search.cancel()
        self._status(self._default_status())

    def _handle_search_backspace(self) -> None:
        """Handle backspace in search mode."""
        if self.search.buffer:
            self.search.buffer = self.search.buffer[:-1]
        self._status(format_search_status(self.search))

    def _handle_search_input(self, char: str) -> None:
        """Handle character input in search mode."""
        self.search.buffer += char
        self._status(format_search_status(self.search))

    # =========================================================================
    # Search Actions
    # =========================================================================

    def action_start_search(self) -> None:
        """Start search mode."""
        if not self.dataset:
            return
        self.search.start()
        self._status(format_search_status(self.search))

    def action_next_match(self) -> None:
        """Jump to next search match."""
        if self.search.matches:
            self.search.next_match()
            jump_to_match(self._get_tree(), self.search)
            self._status(format_search_status(self.search))

    def action_prev_match(self) -> None:
        """Jump to previous search match."""
        if self.search.matches:
            self.search.prev_match()
            jump_to_match(self._get_tree(), self.search)
            self._status(format_search_status(self.search))

    def action_cancel_search(self) -> None:
        """Cancel search mode."""
        self.search.cancel()
        self._status(self._default_status())

    # =========================================================================
    # Navigation Actions
    # =========================================================================

    def action_cursor_up(self) -> None:
        """Move cursor up in tree."""
        self._get_tree().action_cursor_up()

    def action_cursor_down(self) -> None:
        """Move cursor down in tree."""
        self._get_tree().action_cursor_down()

    def action_cursor_left(self) -> None:
        """Collapse current tree node."""
        tree = self._get_tree()
        if tree.cursor_node:
            tree.cursor_node.collapse()

    def action_cursor_right(self) -> None:
        """Expand current tree node."""
        tree = self._get_tree()
        if tree.cursor_node:
            tree.cursor_node.expand()

    def action_goto_top(self) -> None:
        """Go to first node (vim gg - requires double press)."""
        if self._pending_key == "g":
            # Second 'g' pressed - go to top
            tree = self._get_tree()
            tree.select_node(tree.root)
            tree.scroll_to_node(tree.root)
            self._pending_key = None
        else:
            # First 'g' - wait for second
            self._pending_key = "g"
            self.set_timer(KEY_SEQUENCE_TIMEOUT, self._clear_pending_key)

    def action_goto_bottom(self) -> None:
        """Go to last visible node (vim G)."""
        self._pending_key = None
        tree = self._get_tree()
        last_node = self._find_last_visible_node(tree.root)
        tree.select_node(last_node)
        tree.scroll_to_node(last_node)

    def _find_last_visible_node(self, node: TreeNode) -> TreeNode:
        """Find the last visible node in the tree."""
        last = node
        children = list(node.children)
        
        while children:
            last = children[-1]
            if last.is_expanded and last.children:
                children = list(last.children)
            else:
                break
        
        return last

    def action_page_down(self) -> None:
        """Page down in tree (vim Ctrl+f)."""
        tree = self._get_tree()
        for _ in range(PAGE_SCROLL_LINES):
            tree.action_cursor_down()

    def action_page_up(self) -> None:
        """Page up in tree (vim Ctrl+b)."""
        tree = self._get_tree()
        for _ in range(PAGE_SCROLL_LINES):
            tree.action_cursor_up()

    def action_center_cursor(self) -> None:
        """Center current node in view (vim zz)."""
        tree = self._get_tree()
        if tree.cursor_node:
            tree.scroll_to_node(tree.cursor_node, animate=False)

    def _clear_pending_key(self) -> None:
        """Clear pending key sequence after timeout."""
        self._pending_key = None

    # =========================================================================
    # Theme Actions
    # =========================================================================

    def action_cycle_theme(self) -> None:
        """Toggle between Gruvbox dark and light themes."""
        if self.has_class("light-theme"):
            self.remove_class("light-theme")
            self.dark = True
            ThemeColors.set_dark_mode(True)
            theme_name = "Gruvbox Dark"
        else:
            self.add_class("light-theme")
            self.dark = False
            ThemeColors.set_dark_mode(False)
            theme_name = "Gruvbox Light"
        
        # Refresh tree labels with new theme colors
        if self.dataset:
            self._refresh_tree_labels()
        
        self._status(f"Theme: {theme_name}")

    def _refresh_tree_labels(self) -> None:
        """Refresh all tree labels with current theme colors."""
        tree = self._get_tree()
        tree.root.label = format_label(self.dataset.root_node)
        self._refresh_node_labels(tree.root)

    def _refresh_node_labels(self, node: TreeNode) -> None:
        """Recursively refresh tree node labels."""
        if node.data:
            node.label = format_label(node.data)
        for child in node.children:
            self._refresh_node_labels(child)

    # =========================================================================
    # Plot/Table Actions
    # =========================================================================

    def action_toggle_plot(self) -> None:
        """Show plot for current variable."""
        if not self._validate_variable_action("Plot"):
            return
        self.run_worker(self._open_data_screen("plot"), name="plot_loader")

    def action_show_table(self) -> None:
        """Show data table for current variable."""
        if not self._validate_variable_action("Table"):
            return
        self.run_worker(self._open_data_screen("table"), name="table_loader")

    def _validate_variable_action(self, action_name: str) -> bool:
        """Validate that a variable is selected for plot/table actions."""
        if not self.current_node or self.current_node.node_type != NodeType.VARIABLE:
            self._status(f"{action_name} only available for variables")
            return False
        if not self.dataset:
            return False
        return True

    async def _open_data_screen(self, initial_tab: str) -> None:
        """Load data and open plot/table screen.
        
        Args:
            initial_tab: Which tab to show initially ("plot" or "table")
        """
        node = self.current_node
        if not node:
            return
            
        try:
            self._status(f"Loading {node.name}...")
            
            # Load data in background thread
            loop = asyncio.get_event_loop()
            data: np.ndarray | None = await loop.run_in_executor(
                None, load_variable_data, node, self.dataset
            )

            if data is None:
                self._status("Could not load data")
                return

            # Get dimension names from metadata
            dim_names = tuple(node.metadata.get("dims", ()))

            self.push_screen(PlotScreen(
                data, 
                var_name=node.name, 
                dim_names=dim_names, 
                initial_tab=initial_tab
            ))
            self._status(f"{initial_tab.capitalize()} ready")
            
        except Exception as e:
            self._status(f"Error: {e}")

    # =========================================================================
    # Preview Panel Actions
    # =========================================================================

    def action_toggle_preview(self) -> None:
        """Toggle the preview panel visibility."""
        self.show_preview = not self.show_preview
        
        detail = self._get_detail_container()
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
        """Scroll preview panel down."""
        try:
            self._get_detail_container().scroll_relative(
                y=PREVIEW_SCROLL_LINES, animate=False
            )
        except Exception:
            pass

    def action_scroll_preview_up(self) -> None:
        """Scroll preview panel up."""
        try:
            self._get_detail_container().scroll_relative(
                y=-PREVIEW_SCROLL_LINES, animate=False
            )
        except Exception:
            pass

    # =========================================================================
    # Clipboard Actions
    # =========================================================================

    def action_copy_tree(self) -> None:
        """Copy tree structure to clipboard."""
        if not self.dataset:
            self._status("No file loaded")
            return
        
        try:
            content = (
                f"Tree Structure: {self.dataset.file_path}\n"
                f"{'=' * 80}\n\n"
                f"{format_tree_text(self._get_tree().root)}"
            )
            success, msg = copy_to_clipboard(content)
            self._status("Tree copied!" if success else msg)
        except Exception as e:
            self._status(f"Copy failed: {e}")

    def action_copy_info(self) -> None:
        """Copy current node info to clipboard."""
        if not self.current_node:
            self._status("No node selected")
            return
        
        content = format_node_info(self.current_node)
        success, msg = copy_to_clipboard(content)
        self._status(f"Copied {self.current_node.name}!" if success else msg)

    # =========================================================================
    # Helper Methods
    # =========================================================================

    def _get_tree(self) -> Tree:
        """Get the tree widget."""
        return self.query_one("#tree", Tree)

    def _get_detail_container(self) -> VerticalScroll:
        """Get the detail container widget."""
        return self.query_one("#detail-container", VerticalScroll)

    def _show_details(self, node: DataNode) -> None:
        """Display node details in the preview panel."""
        self.current_node = node
        
        try:
            container = self._get_detail_container()
            render_details(container, node, self.dataset)
        except Exception as e:
            self._show_details_error(e)

    def _show_details_error(self, error: Exception) -> None:
        """Show error in details panel."""
        try:
            container = self._get_detail_container()
            container.remove_children()
            container.mount(Static(
                f"[{ThemeColors.error()}]Error: {error}[/{ThemeColors.error()}]"
            ))
        except Exception:
            pass

    def _status(self, msg: str) -> None:
        """Update the status bar message."""
        self.query_one("#status-bar", Static).update(msg)

    def _default_status(self) -> str:
        """Get the default status bar message."""
        if self.file_path:
            return Path(self.file_path).name
        return "Ready"
