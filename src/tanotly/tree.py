"""Tree population and node formatting for Tanotly."""

from textual.widgets import Tree

from .config import NODE_COLORS
from .data.models import DataNode, NodeType


def populate_tree(tree_node, data_node: DataNode) -> None:
    """Recursively populate a Textual tree from data nodes."""
    # Add non-attribute children first (groups, variables, dimensions)
    for child in data_node.children:
        if child.node_type == NodeType.ATTRIBUTE:
            continue
        label = format_label(child)
        has_children = len(child.children) > 0
        node = tree_node.add(label, data=child, allow_expand=has_children)
        if child.children:
            populate_tree(node, child)

    # Add attributes as a grouped node
    attr_children = [c for c in data_node.children if c.node_type == NodeType.ATTRIBUTE]
    if attr_children:
        attrs_label = f"[magenta]🏷️  Attributes ({len(attr_children)})[/magenta]"
        attrs_node = tree_node.add(attrs_label, data=None, allow_expand=True)
        for attr in attr_children:
            attrs_node.add(format_label(attr), data=attr, allow_expand=False)


def format_label(node: DataNode) -> str:
    """Format a tree node label with Rich markup colors."""
    color = NODE_COLORS.get(node.node_type, "white")

    if node.node_type == NodeType.ROOT:
        return f"[bold {color}]{node.name}[/bold {color}]"

    if node.node_type == NodeType.GROUP:
        child_count = sum(1 for c in node.children if c.node_type != NodeType.ATTRIBUTE)
        return f"[{color}]{node.name}[/{color}] [dim]({child_count})[/dim]"

    if node.node_type == NodeType.VARIABLE:
        shape = node.metadata.get("shape", "")
        dtype = node.metadata.get("dtype", "")
        if shape and dtype:
            shape_str = "×".join(str(s) for s in shape)
            dim_label = get_dimension_label(node)
            return f"[{color}]{node.name}[/{color}] [dim]({shape_str}) {dim_label} {dtype}[/dim]"
        return f"[{color}]{node.name}[/{color}]"

    if node.node_type == NodeType.DIMENSION:
        size = node.metadata.get("size", "")
        if size:
            return f"[{color}]{node.name}[/{color}] [dim]({size})[/dim]"
        return f"[{color}]{node.name}[/{color}]"

    if node.node_type == NodeType.ATTRIBUTE:
        name = node.name.replace('[', '\\[').replace(']', '\\]')
        if len(name) > 60:
            name = name[:57] + "..."
        return f"[{color}]{name}[/{color}]"

    return node.name


def get_dimension_label(node: DataNode) -> str:
    """Get dimension label (1D, 2D, Geo2D, etc.)."""
    shape = node.metadata.get("shape", ())
    dims = node.metadata.get("dims", ())

    if not shape:
        return ""

    ndim = len(shape)
    if ndim == 0:
        return "scalar"

    # Check for geographic data
    geo_terms = ('lat', 'lon', 'latitude', 'longitude')
    is_geo = dims and any(term in str(dims).lower() for term in geo_terms)
    prefix = "Geo" if is_geo and ndim in (2, 3) else ""

    return f"{prefix}{ndim}D"


def collect_all_nodes(tree_node, result: list) -> None:
    """Recursively collect all tree nodes into a list."""
    result.append(tree_node)
    for child in tree_node.children:
        collect_all_nodes(child, result)


def expand_all(tree_node) -> None:
    """Recursively expand all tree nodes."""
    tree_node.expand()
    for child in tree_node.children:
        expand_all(child)
