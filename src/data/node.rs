//! Data node types and structures.

use std::collections::HashMap;

/// Type of node in the NetCDF hierarchy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeType {
    /// Root node (file level).
    Root,
    /// Group node.
    Group,
    /// Variable node.
    Variable,
    /// Dimension node.
    Dimension,
}

/// A node in the NetCDF data tree.
#[derive(Debug, Clone)]
pub struct DataNode {
    /// Node name.
    pub name: String,
    /// Full path to this node.
    pub path: String,
    /// Type of node.
    pub node_type: NodeType,
    /// Metadata key-value pairs.
    pub metadata: HashMap<String, String>,
    /// Child nodes.
    pub children: Vec<DataNode>,
    /// NetCDF attributes.
    pub attributes: HashMap<String, String>,
    /// Shape for variable nodes.
    pub shape: Option<Vec<usize>>,
    /// Data type for variable nodes.
    pub dtype: Option<String>,
}

impl DataNode {
    /// Create a new data node.
    pub fn new(name: String, path: String, node_type: NodeType) -> Self {
        Self {
            name,
            path,
            node_type,
            metadata: HashMap::new(),
            children: Vec::new(),
            attributes: HashMap::new(),
            shape: None,
            dtype: None,
        }
    }

    /// Check if this node is a variable.
    pub fn is_variable(&self) -> bool {
        self.node_type == NodeType::Variable
    }

    /// Check if this node is a group (or root).
    pub fn is_group(&self) -> bool {
        matches!(self.node_type, NodeType::Group | NodeType::Root)
    }

    /// Add a child node.
    pub fn add_child(&mut self, child: DataNode) {
        self.children.push(child);
    }

    /// Get display name with icon and metadata.
    pub fn display_name(&self) -> String {
        let icon = match self.node_type {
            NodeType::Root => "🏠",
            NodeType::Group => "📂",
            NodeType::Variable => "🌡️",
            NodeType::Dimension => "📏",
        };

        let suffix = match self.node_type {
            NodeType::Variable => {
                // Format: name(dim1=size1, dim2=size2); [ND] type
                let mut parts = Vec::new();

                // Dimension info
                if let Some(dim_str) = self.metadata.get("dims") {
                    if !dim_str.is_empty() {
                        let dims: Vec<&str> = dim_str.split(", ").collect();
                        let mut dim_info = Vec::new();

                        if let Some(shape) = &self.shape {
                            for (i, dim_name) in dims.iter().enumerate() {
                                if let Some(&size) = shape.get(i) {
                                    dim_info.push(format!("{}={}", dim_name, size));
                                }
                            }
                        }

                        if !dim_info.is_empty() {
                            parts.push(format!("({})", dim_info.join(", ")));
                        }
                    }
                }

                // Dimensionality
                if let Some(shape) = &self.shape {
                    let ndim = shape.len();
                    if ndim > 0 {
                        parts.push(format!("[{}D]", ndim));
                    }
                }

                // Data type
                if let Some(dtype) = &self.dtype {
                    // Clean up the type name (remove "NcVariableType::" prefix if present)
                    let clean_type = dtype.replace("NcVariableType::", "").to_lowercase();
                    parts.push(clean_type);
                }

                if !parts.is_empty() {
                    format!(" {}", parts.join(" "))
                } else {
                    String::new()
                }
            },
            NodeType::Group | NodeType::Root => {
                format!(" ({})", self.children.len())
            },
            NodeType::Dimension => String::new(),
        };

        format!("{} {}{}", icon, self.name, suffix)
    }

    /// Check if this node matches a search query.
    pub fn matches_search(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();

        // Check name
        if self.name.to_lowercase().contains(&query_lower) {
            return true;
        }

        // Check path
        if self.path.to_lowercase().contains(&query_lower) {
            return true;
        }

        // Check attributes
        for (key, value) in &self.attributes {
            if key.to_lowercase().contains(&query_lower)
                || value.to_lowercase().contains(&query_lower)
            {
                return true;
            }
        }

        // Check metadata
        for (key, value) in &self.metadata {
            if key.to_lowercase().contains(&query_lower)
                || value.to_lowercase().contains(&query_lower)
            {
                return true;
            }
        }

        false
    }
}
