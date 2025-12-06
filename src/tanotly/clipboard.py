"""Clipboard utilities for Tanotly."""

import re
import subprocess
import tempfile

from .data.models import DataNode


def copy_to_clipboard(content: str) -> tuple[bool, str]:
    """Copy text to clipboard. Returns (success, message)."""
    commands = [
        ['xclip', '-selection', 'clipboard'],  # Linux
        ['pbcopy'],                              # macOS
        ['clip'],                                # Windows
    ]

    for cmd in commands:
        try:
            subprocess.run(cmd, input=content.encode(), check=True, capture_output=True)
            return True, "Copied to clipboard"
        except (FileNotFoundError, subprocess.CalledProcessError):
            continue

    # Fallback: save to temp file
    with tempfile.NamedTemporaryFile(mode='w', delete=False, suffix='.txt') as f:
        f.write(content)
        return False, f"Saved to {f.name}"


def format_tree_text(tree_node, prefix: str = "", is_last: bool = True) -> str:
    """Recursively format tree nodes as plain text."""
    result = ""
    if tree_node.data:
        connector = "└── " if is_last else "├── "
        label = tree_node.label
        label_text = label.plain if hasattr(label, 'plain') else re.sub(r'\[.*?\]', '', str(label))
        result += prefix + connector + label_text + "\n"

        if tree_node.children:
            extension = "    " if is_last else "│   "
            for i, child in enumerate(tree_node.children):
                result += format_tree_text(child, prefix + extension, i == len(tree_node.children) - 1)
    else:
        for i, child in enumerate(tree_node.children):
            result += format_tree_text(child, "", i == len(tree_node.children) - 1)
    return result


def format_node_info(node: DataNode) -> str:
    """Format node information as plain text."""
    lines = [
        f"Node: {node.name}",
        f"Type: {node.node_type.value}",
        f"Path: {node.path}",
    ]

    if node.metadata:
        lines.append("\nMetadata:")
        for key, val in node.metadata.items():
            lines.append(f"  {key}: {val}")

    if node.attributes:
        lines.append("\nAttributes:")
        for key, val in node.attributes.items():
            lines.append(f"  {key}: {val}")

    return "\n".join(lines) + "\n"
