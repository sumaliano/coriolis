"""Data models for representing dataset structures."""

from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Dict, List, Optional, Tuple


class NodeType(Enum):
    """Type of data node."""

    ROOT = "root"
    GROUP = "group"
    VARIABLE = "variable"
    DIMENSION = "dimension"
    ATTRIBUTE = "attribute"


@dataclass
class DataNode:
    """Represents a node in the data hierarchy."""

    name: str
    node_type: NodeType
    path: str
    parent: Optional["DataNode"] = None
    children: List["DataNode"] = field(default_factory=list)
    attributes: Dict[str, Any] = field(default_factory=dict)
    metadata: Dict[str, Any] = field(default_factory=dict)
    is_fully_loaded: bool = False  # Track if full metadata has been loaded

    def add_child(self, child: "DataNode") -> None:
        """Add a child node."""
        child.parent = self
        self.children.append(child)

    def get_full_path(self) -> str:
        """Get the full path of this node."""
        if self.parent is None or self.parent.node_type == NodeType.ROOT:
            return self.name
        parent_path = self.parent.get_full_path()
        if parent_path == "/":
            return f"/{self.name}"
        return f"{parent_path}/{self.name}"

    def matches_search(self, query: str) -> bool:
        """Check if this node matches a search query (case-insensitive substring)."""
        query_lower = query.lower()

        # Search in node name
        if query_lower in self.name.lower():
            return True

        # Search in attribute names and values
        for attr_name, attr_value in self.attributes.items():
            if query_lower in attr_name.lower():
                return True
            if isinstance(attr_value, str) and query_lower in attr_value.lower():
                return True

        # Search in metadata
        for key, value in self.metadata.items():
            if query_lower in str(key).lower():
                return True
            if isinstance(value, str) and query_lower in value.lower():
                return True

        return False


@dataclass
class DatasetInfo:
    """Information about an opened dataset."""

    file_path: str
    file_type: str
    root_node: DataNode
    dimensions: Dict[str, int] = field(default_factory=dict)
    global_attributes: Dict[str, Any] = field(default_factory=dict)
    variables: List[str] = field(default_factory=list)

    def get_variable_info(self, var_name: str) -> Optional[Dict[str, Any]]:
        """Get detailed information about a variable."""
        for child in self.root_node.children:
            if child.name == var_name and child.node_type == NodeType.VARIABLE:
                return {
                    "name": child.name,
                    "path": child.path,
                    "attributes": child.attributes,
                    "metadata": child.metadata,
                }
        return None
