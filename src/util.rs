//! Utility functions for Coriolis.

use crate::clipboard;
use crate::data::DataNode;
use crate::error::Result;

/// Copy tree structure to clipboard.
pub fn copy_tree_structure(node: &DataNode, file_name: Option<&str>) -> Result<()> {
    let mut text = String::new();

    if let Some(name) = file_name {
        text.push_str(&format!("Tree Structure: {}\n", name));
    } else {
        text.push_str("Tree Structure\n");
    }

    text.push_str(&"=".repeat(80));
    text.push_str("\n\n");

    text.push_str(&format_tree_recursive(node, "", true));

    clipboard::copy_to_clipboard(&text)
}

/// Copy node information to clipboard.
pub fn copy_node_info(node: &DataNode) -> Result<()> {
    let mut text = format!("Node: {}\n", node.name);
    text.push_str(&format!("Path: {}\n", node.path));
    text.push_str(&format!("Type: {:?}\n", node.node_type));

    if let Some(ref shape) = node.shape {
        text.push_str(&format!("Shape: {:?}\n", shape));
    }

    if let Some(ref dtype) = node.dtype {
        text.push_str(&format!("DType: {}\n", dtype));
    }

    if !node.attributes.is_empty() {
        text.push_str("\nAttributes:\n");
        for (key, value) in &node.attributes {
            text.push_str(&format!("  {}: {}\n", key, value));
        }
    }

    if !node.metadata.is_empty() {
        text.push_str("\nMetadata:\n");
        for (key, value) in &node.metadata {
            text.push_str(&format!("  {}: {}\n", key, value));
        }
    }

    clipboard::copy_to_clipboard(&text)
}

fn format_tree_recursive(node: &DataNode, prefix: &str, is_last: bool) -> String {
    let mut result = String::new();

    let connector = if is_last { "└── " } else { "├── " };
    result.push_str(&format!("{}{}{}\n", prefix, connector, node.display_name()));

    let new_prefix = format!("{}{}   ", prefix, if is_last { " " } else { "│" });

    for (i, child) in node.children.iter().enumerate() {
        let is_last_child = i == node.children.len() - 1;
        result.push_str(&format_tree_recursive(child, &new_prefix, is_last_child));
    }

    result
}
