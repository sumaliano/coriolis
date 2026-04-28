//! Clipboard integration.

use crate::data::DataNode;
use crate::error::Result;
use arboard::Clipboard;

/// Copy text to clipboard, with fallbacks for WSL and headless Linux.
fn copy_to_clipboard(text: &str) -> Result<()> {
    // 1. arboard (works when X11/Wayland display is available)
    if let Ok(mut cb) = Clipboard::new() {
        if cb.set_text(text).is_ok() {
            return Ok(());
        }
    }

    // 2. WSL: clip.exe (Windows clipboard, always available in WSL2)
    if try_pipe_to_cmd("clip.exe", text) {
        return Ok(());
    }

    // 3. Wayland (wl-copy)
    if try_pipe_to_cmd("wl-copy", text) {
        return Ok(());
    }

    // 4. X11 (xclip)
    if try_pipe_to_cmd_args("xclip", &["-selection", "clipboard"], text) {
        return Ok(());
    }

    Err(crate::error::CoriolisError::Clipboard(
        arboard::Error::Unknown {
            description: "No clipboard backend available. On WSL ensure clip.exe is accessible or set DISPLAY.".to_string(),
        },
    ))
}

fn try_pipe_to_cmd(cmd: &str, text: &str) -> bool {
    try_pipe_to_cmd_args(cmd, &[], text)
}

fn try_pipe_to_cmd_args(cmd: &str, args: &[&str], text: &str) -> bool {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut child = match Command::new(cmd).args(args).stdin(Stdio::piped()).spawn() {
        Ok(c) => c,
        Err(_) => return false,
    };

    if let Some(stdin) = child.stdin.take() {
        let mut writer = std::io::BufWriter::new(stdin);
        let _ = writer.write_all(text.as_bytes());
    }

    child.wait().map(|s| s.success()).unwrap_or(false)
}

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

    copy_to_clipboard(&text)
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

    copy_to_clipboard(&text)
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
