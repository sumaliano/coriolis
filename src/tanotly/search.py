"""Search functionality for Tanotly."""

from dataclasses import dataclass, field

from textual.widgets import Tree

from .tree import collect_all_nodes, expand_all


@dataclass
class SearchState:
    """Holds the current search state."""
    buffer: str | None = None  # None = not searching, str = typing query
    query: str = ""
    matches: list = field(default_factory=list)
    index: int = -1

    @property
    def is_active(self) -> bool:
        """True if currently in search mode."""
        return self.buffer is not None

    def start(self) -> None:
        """Enter search mode."""
        self.buffer = ""

    def cancel(self) -> None:
        """Exit search mode and clear results."""
        self.buffer = None
        self.query = ""
        self.matches = []
        self.index = -1

    def next_match(self) -> None:
        """Move to next match."""
        if self.matches:
            self.index = (self.index + 1) % len(self.matches)

    def prev_match(self) -> None:
        """Move to previous match."""
        if self.matches:
            self.index = (self.index - 1) % len(self.matches)


def perform_search(tree: Tree, state: SearchState) -> None:
    """Execute search and populate matches."""
    if not state.query:
        return

    # Expand all nodes to make everything searchable
    expand_all(tree.root)

    # Collect all nodes
    all_nodes = []
    collect_all_nodes(tree.root, all_nodes)

    # Find matches
    state.matches = []
    query_lower = state.query.lower()

    for tree_node in all_nodes:
        if not tree_node.data:
            continue

        # Search in name and path
        if query_lower in tree_node.data.name.lower():
            state.matches.append(tree_node)
            continue

        if query_lower in tree_node.data.path.lower():
            state.matches.append(tree_node)
            continue

        # Search in attribute keys
        if tree_node.data.attributes:
            if any(query_lower in k.lower() for k in tree_node.data.attributes):
                state.matches.append(tree_node)
                continue

        # Search in metadata keys
        if tree_node.data.metadata:
            if any(query_lower in str(k).lower() for k in tree_node.data.metadata):
                state.matches.append(tree_node)

    state.index = 0 if state.matches else -1


def jump_to_match(tree: Tree, state: SearchState) -> bool:
    """Jump to current match. Returns True if successful."""
    if not (0 <= state.index < len(state.matches)):
        return False

    match_node = state.matches[state.index]

    # Expand parents
    parent = match_node.parent
    while parent:
        parent.expand()
        parent = parent.parent

    tree.select_node(match_node)
    tree.scroll_to_node(match_node)
    return True


def format_search_status(state: SearchState) -> str:
    """Format status message for current search state."""
    if state.buffer is not None:
        return f"🔍 Search: {state.buffer}_ | Enter to find, Esc to cancel"

    if state.matches:
        pos = state.index + 1
        total = len(state.matches)
        return f"🔍 Match {pos}/{total}: '{state.query}' | n=next N=prev Esc=exit"

    if state.query:
        return f"🔍 No matches for '{state.query}' | Esc to exit search"

    return ""
