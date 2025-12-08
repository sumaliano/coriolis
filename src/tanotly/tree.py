"""Tree population and formatting for Tanotly.

This module handles:
- Populating Textual Tree widgets from DataNode hierarchies
- Formatting tree node labels with Rich markup colors
- Tree traversal utilities (collect nodes, expand all, etc.)

Tree Display Format:
- Root: Bold purple with filename
- Groups: Yellow with child count
- Variables: Aqua with dimension info, shape, and dtype
- Dimensions: Blue (shown in details panel, not tree)
"""

from __future__ import annotations

from typing import TYPE_CHECKING

from textual.widgets import Tree

from .config import ThemeColors
from .data.models import DataNode, NodeType

if TYPE_CHECKING:
    from textual.widgets._tree import TreeNode


# =============================================================================
# Tree Population
# =============================================================================

def populate_tree(tree_node: TreeNode, data_node: DataNode) -> None:
    """Recursively populate a Textual tree from data nodes.
    
    Args:
        tree_node: The Textual TreeNode to populate
        data_node: The DataNode containing the data hierarchy
    
    Note:
        Dimensions and attributes are NOT shown in tree - they appear 
        in the details panel instead for a cleaner tree view.
    """
    for child in data_node.children:
        # Skip dimensions and attributes in tree view
        if child.node_type in (NodeType.ATTRIBUTE, NodeType.DIMENSION):
            continue
        
        label = format_label(child)
        has_expandable_children = _has_expandable_children(child)
        
        node = tree_node.add(label, data=child, allow_expand=has_expandable_children)
        
        if child.children:
            populate_tree(node, child)


def _has_expandable_children(node: DataNode) -> bool:
    """Check if a node has children that should be shown in the tree."""
    return any(
        child.node_type not in (NodeType.ATTRIBUTE, NodeType.DIMENSION)
        for child in node.children
    )


# =============================================================================
# Label Formatting
# =============================================================================

def format_label(node: DataNode) -> str:
    """Format a tree node label with Rich markup colors.
    
    Args:
        node: The DataNode to format
        
    Returns:
        Rich markup string for the tree label
    
    Label formats by type:
        - Root: "filename" (bold purple)
        - Group: "name (count)" (yellow)
        - Variable: "name (dims) [nD] dtype" (aqua)
        - Dimension: "name (size)" (blue)
        - Attribute: "name" (default)
    """
    formatter = _LABEL_FORMATTERS.get(node.node_type, _format_default_label)
    return formatter(node)


def _format_root_label(node: DataNode) -> str:
    """Format root node label."""
    return f"[bold {ThemeColors.root()}]{node.name}[/]"


def _format_group_label(node: DataNode) -> str:
    """Format group node label with child count."""
    # Count non-attribute children
    child_count = sum(
        1 for c in node.children 
        if c.node_type != NodeType.ATTRIBUTE
    )
    return (
        f"[{ThemeColors.group()}]{node.name}[/] "
        f"[{ThemeColors.muted()}]({child_count})[/]"
    )


def _format_variable_label(node: DataNode) -> str:
    """Format variable label with dimension info.
    
    Format: name (dim1=size1, dim2=size2, ...) [nD] dtype
    Example: temperature (time=365, lat=180, lon=360) [3D] float32
    """
    meta = node.metadata or {}
    shape = meta.get("shape", ())
    dims = meta.get("dims", ())
    dtype = meta.get("dtype", "")
    
    parts = [f"[{ThemeColors.variable()}]{node.name}[/]"]
    
    # Dimension info
    dim_str = _format_dimensions(dims, shape)
    if dim_str:
        parts.append(f"[{ThemeColors.muted()}]({dim_str})[/]")
    
    # Dimensionality label (1D, 2D, Geo2D, etc.)
    dim_label = _get_dimensionality_label(dims, shape)
    if dim_label:
        # Escape brackets for Rich markup
        parts.append(f"[{ThemeColors.muted()}]\\[{dim_label}][/]")
    
    # Data type
    if dtype:
        parts.append(f"[{ThemeColors.muted()}]{dtype}[/]")
    
    return " ".join(parts)


def _format_dimension_label(node: DataNode) -> str:
    """Format dimension node label with size."""
    size = node.metadata.get("size", "") if node.metadata else ""
    
    if size:
        return (
            f"[{ThemeColors.dimension()}]{node.name}[/] "
            f"[{ThemeColors.muted()}]({size})[/]"
        )
    return f"[{ThemeColors.dimension()}]{node.name}[/]"


def _format_attribute_label(node: DataNode) -> str:
    """Format attribute node label (truncated if long)."""
    name = _escape_rich_markup(node.name)
    if len(name) > 60:
        name = name[:57] + "..."
    return name


def _format_default_label(node: DataNode) -> str:
    """Format default node label."""
    return _escape_rich_markup(node.name)


# Mapping of node types to their label formatters
_LABEL_FORMATTERS = {
    NodeType.ROOT: _format_root_label,
    NodeType.GROUP: _format_group_label,
    NodeType.VARIABLE: _format_variable_label,
    NodeType.DIMENSION: _format_dimension_label,
    NodeType.ATTRIBUTE: _format_attribute_label,
}


# =============================================================================
# Dimension Formatting Helpers
# =============================================================================

def _format_dimensions(dims: tuple, shape: tuple) -> str:
    """Format dimension names with sizes.
    
    Args:
        dims: Tuple of dimension names
        shape: Tuple of dimension sizes
        
    Returns:
        Formatted string like "time=365, lat=180, lon=360"
        or "365×180×360" if no dimension names
    """
    if dims and shape and len(dims) == len(shape):
        return ", ".join(f"{d}={s}" for d, s in zip(dims, shape))
    elif shape:
        return "×".join(str(s) for s in shape)
    return ""


def _get_dimensionality_label(dims: tuple, shape: tuple) -> str:
    """Get dimensionality label (1D, 2D, Geo2D, etc.).
    
    Args:
        dims: Tuple of dimension names
        shape: Tuple of dimension sizes
        
    Returns:
        Label like "1D", "2D", "3D", "Geo2D", "Geo3D", or "scalar"
    """
    if not shape:
        return ""
    
    ndim = len(shape)
    if ndim == 0:
        return "scalar"
    
    # Check for geographic data based on dimension names
    is_geo = _is_geographic_data(dims)
    
    # Only add "Geo" prefix for 2D and 3D data
    prefix = "Geo" if is_geo and ndim in (2, 3) else ""
    
    return f"{prefix}{ndim}D"


def _is_geographic_data(dims: tuple) -> bool:
    """Check if dimensions suggest geographic/spatial data.
    
    Looks for common geographic dimension names like:
    lat, lon, latitude, longitude, x, y, row, col, etc.
    """
    if not dims:
        return False
    
    geo_terms = {
        'lat', 'lon', 'latitude', 'longitude',
        'x', 'y', 'row', 'col', 'rows', 'cols',
        'northing', 'easting', 'across', 'along',
    }
    
    dim_names = " ".join(str(d).lower() for d in dims)
    return any(term in dim_names for term in geo_terms)


# =============================================================================
# Utility Functions
# =============================================================================

def _escape_rich_markup(text: str) -> str:
    """Escape Rich markup characters in text."""
    return text.replace('[', '\\[').replace(']', '\\]')


def collect_all_nodes(tree_node: TreeNode, result: list[TreeNode]) -> None:
    """Recursively collect all tree nodes into a list.
    
    Args:
        tree_node: Starting tree node
        result: List to append nodes to (modified in place)
    """
    result.append(tree_node)
    for child in tree_node.children:
        collect_all_nodes(child, result)


def get_all_nodes(tree_node: TreeNode) -> list[TreeNode]:
    """Get all tree nodes as a list.
    
    Args:
        tree_node: Starting tree node
        
    Returns:
        List of all tree nodes (depth-first order)
    """
    result: list[TreeNode] = []
    collect_all_nodes(tree_node, result)
    return result


def expand_all(tree_node: TreeNode) -> None:
    """Recursively expand all tree nodes.
    
    Args:
        tree_node: Starting tree node to expand
    """
    tree_node.expand()
    for child in tree_node.children:
        expand_all(child)


def collapse_all(tree_node: TreeNode) -> None:
    """Recursively collapse all tree nodes.
    
    Args:
        tree_node: Starting tree node to collapse
    """
    for child in tree_node.children:
        collapse_all(child)
    tree_node.collapse()


def expand_to_depth(tree_node: TreeNode, depth: int) -> None:
    """Expand tree nodes up to a certain depth.
    
    Args:
        tree_node: Starting tree node
        depth: Maximum depth to expand (0 = just root)
    """
    if depth < 0:
        return
    
    tree_node.expand()
    
    if depth > 0:
        for child in tree_node.children:
            expand_to_depth(child, depth - 1)


def find_node_by_path(tree_node: TreeNode, path: str) -> TreeNode | None:
    """Find a tree node by its data path.
    
    Args:
        tree_node: Starting tree node to search from
        path: Path string to match against node.data.path
        
    Returns:
        The matching TreeNode, or None if not found
    """
    if tree_node.data and hasattr(tree_node.data, 'path'):
        if tree_node.data.path == path:
            return tree_node
    
    for child in tree_node.children:
        result = find_node_by_path(child, path)
        if result:
            return result
    
    return None


def find_nodes_by_name(tree_node: TreeNode, name: str, case_sensitive: bool = False) -> list[TreeNode]:
    """Find all tree nodes matching a name pattern.
    
    Args:
        tree_node: Starting tree node to search from
        name: Name to search for (substring match)
        case_sensitive: Whether to match case
        
    Returns:
        List of matching TreeNodes
    """
    results: list[TreeNode] = []
    search_name = name if case_sensitive else name.lower()
    
    def search(node: TreeNode) -> None:
        if node.data and hasattr(node.data, 'name'):
            node_name = node.data.name if case_sensitive else node.data.name.lower()
            if search_name in node_name:
                results.append(node)
        
        for child in node.children:
            search(child)
    
    search(tree_node)
    return results


def count_nodes(tree_node: TreeNode) -> dict[str, int]:
    """Count nodes by type in the tree.
    
    Args:
        tree_node: Starting tree node
        
    Returns:
        Dictionary with counts: {'groups': n, 'variables': n, 'total': n}
    """
    counts = {'groups': 0, 'variables': 0, 'total': 0}
    
    def count(node: TreeNode) -> None:
        counts['total'] += 1
        
        if node.data and hasattr(node.data, 'node_type'):
            if node.data.node_type == NodeType.GROUP:
                counts['groups'] += 1
            elif node.data.node_type == NodeType.VARIABLE:
                counts['variables'] += 1
        
        for child in node.children:
            count(child)
    
    count(tree_node)
    return counts
