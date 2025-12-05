"""Search utilities for finding nodes in the data structure."""

from typing import List

from tanotly.data.models import DataNode


def search_nodes(root: DataNode, query: str) -> List[DataNode]:
    """
    Search for nodes matching the query string (case-insensitive substring match).

    Args:
        root: The root node to start searching from
        query: The search query string

    Returns:
        List of matching DataNode objects
    """
    if not query:
        return []

    matches: List[DataNode] = []
    _search_recursive(root, query, matches)
    return matches


def _search_recursive(node: DataNode, query: str, matches: List[DataNode]) -> None:
    """Recursively search through the node tree."""
    if node.matches_search(query):
        matches.append(node)

    for child in node.children:
        _search_recursive(child, query, matches)
