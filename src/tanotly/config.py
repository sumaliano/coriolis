"""Configuration constants for Tanotly."""

from .data.models import NodeType

# Node type display colors (Rich markup)
NODE_COLORS = {
    NodeType.ROOT: "magenta",
    NodeType.GROUP: "yellow",
    NodeType.VARIABLE: "cyan",
    NodeType.DIMENSION: "blue",
    NodeType.ATTRIBUTE: "magenta",
}

# Node type icons for detail panel
NODE_ICONS = {
    NodeType.ROOT: "🏠",
    NodeType.GROUP: "📂",
    NodeType.VARIABLE: "🌡️",
    NodeType.DIMENSION: "📏",
    NodeType.ATTRIBUTE: "🏷️",
}

# Application CSS styles
APP_CSS = """
#main { height: 1fr; }

#status-bar {
    height: 1;
    background: $panel;
    color: $text;
    content-align: left middle;
    padding-left: 1;
}

#tree-container {
    width: 50%;
    border-right: solid $accent;
    background: black;
}

#tree-container.hidden { display: none; }

#detail-container {
    width: 50%;
    padding: 1;
    background: black;
    scrollbar-gutter: stable;
}

#detail-container.full-width { width: 100%; }

Tree {
    height: 100%;
    scrollbar-gutter: stable;
    background: black;
}

Tree > .tree--label {
    text-overflow: ellipsis;
    overflow: hidden;
}

Tree > .tree--guides { color: $accent-darken-1; }

VerticalScroll {
    height: 100%;
    overflow-y: auto;
}

DataPlot1D, DataPlot2D {
    height: auto;
    min-height: 20;
    width: 100%;
}
"""
