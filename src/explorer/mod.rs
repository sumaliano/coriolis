//! Explorer feature - NetCDF structure exploration.
//!
//! This module provides functionality for exploring NetCDF file structure,
//! including tree navigation and details display.

pub mod details;
pub mod tree;
pub mod ui;

use crate::data::DataNode;
use std::collections::HashSet;

/// Explorer state - combines tree navigation and details display.
#[derive(Debug)]
pub struct ExplorerState {
    /// All tree items in display order (only visible items).
    items: Vec<TreeItem>,
    /// Cursor position (index into items).
    cursor: usize,
    /// The root node for rebuilding.
    root: Option<DataNode>,
    /// Set of expanded node paths.
    expanded_paths: HashSet<String>,
    /// Scroll offset for the tree view.
    scroll_offset: usize,
    /// Show preview/details panel.
    pub show_preview: bool,
    /// Preview scroll offset.
    pub preview_scroll: u16,
}

/// A single item in the tree view.
#[derive(Debug, Clone)]
pub struct TreeItem {
    /// The data node.
    pub node: DataNode,
    /// Nesting level.
    pub level: usize,
    /// Whether this node is expanded.
    pub expanded: bool,
}

impl ExplorerState {
    /// Create a new explorer state.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            cursor: 0,
            root: None,
            expanded_paths: HashSet::new(),
            scroll_offset: 0,
            show_preview: true,
            preview_scroll: 0,
        }
    }

    /// Build tree from dataset.
    pub fn build_from_dataset(&mut self, dataset: &crate::data::DatasetInfo) {
        self.root = Some(dataset.root_node.clone());
        self.expanded_paths.clear();
        self.expanded_paths.insert(dataset.root_node.path.clone());
        self.rebuild_visible_items();
        self.cursor = 0;
    }

    /// Rebuild the visible items list based on expanded state.
    fn rebuild_visible_items(&mut self) {
        self.items.clear();
        if let Some(root) = self.root.clone() {
            self.add_visible_recursive(&root, 0);
        }
    }

    fn add_visible_recursive(&mut self, node: &DataNode, level: usize) {
        let is_expanded = self.expanded_paths.contains(&node.path);

        self.items.push(TreeItem {
            node: node.clone(),
            level,
            expanded: is_expanded,
        });

        if is_expanded {
            for child in &node.children {
                self.add_visible_recursive(child, level + 1);
            }
        }
    }

    /// Move the cursor up one position.
    pub fn cursor_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.preview_scroll = 0;
        }
    }

    /// Move the cursor down one position.
    pub fn cursor_down(&mut self) {
        if self.cursor + 1 < self.items.len() {
            self.cursor += 1;
            self.preview_scroll = 0;
        }
    }

    /// Adjust scroll to keep cursor visible.
    pub fn adjust_scroll(&mut self, viewport_height: usize) {
        if viewport_height == 0 {
            return;
        }

        if self.cursor < self.scroll_offset {
            self.scroll_offset = self.cursor;
        }

        if self.cursor >= self.scroll_offset + viewport_height {
            self.scroll_offset = self.cursor.saturating_sub(viewport_height - 1);
        }
    }

    /// Get the current scroll offset.
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Expand the node at the current cursor position.
    pub fn expand_current(&mut self) {
        if self.cursor < self.items.len() {
            let item = &self.items[self.cursor];
            if item.node.is_group() && !item.expanded {
                let path = item.node.path.clone();
                self.expanded_paths.insert(path);
                self.rebuild_visible_items();
            }
        }
    }

    /// Collapse the node at the current cursor position.
    pub fn collapse_current(&mut self) {
        if self.cursor < self.items.len() {
            let item = &self.items[self.cursor];
            if item.node.is_group() && item.expanded {
                let path = item.node.path.clone();
                self.expanded_paths.remove(&path);
                self.rebuild_visible_items();
            }
        }
    }

    /// Go to the first item.
    pub fn goto_first(&mut self) {
        self.cursor = 0;
    }

    /// Go to the last visible item.
    pub fn goto_last(&mut self) {
        if !self.items.is_empty() {
            self.cursor = self.items.len() - 1;
        }
    }

    /// Get all currently visible items in the tree.
    pub fn visible_items(&self) -> Vec<&TreeItem> {
        self.items.iter().collect()
    }

    /// Get the current cursor position.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Get the current node.
    pub fn current_node(&self) -> Option<&DataNode> {
        self.items.get(self.cursor).map(|item| &item.node)
    }

    /// Move the cursor to a node with the given path.
    pub fn goto_node(&mut self, target_path: &str) {
        for (i, item) in self.items.iter().enumerate() {
            if item.node.path == target_path {
                self.cursor = i;
                return;
            }
        }
    }

    /// Expand all nodes in the tree.
    pub fn expand_all(&mut self) {
        if let Some(root) = self.root.clone() {
            Self::collect_group_paths(&root, &mut self.expanded_paths);
        }
        self.rebuild_visible_items();
    }

    fn collect_group_paths(node: &DataNode, paths: &mut HashSet<String>) {
        if node.is_group() {
            paths.insert(node.path.clone());
        }
        for child in &node.children {
            Self::collect_group_paths(child, paths);
        }
    }

    /// Toggle preview panel.
    pub fn toggle_preview(&mut self) {
        self.show_preview = !self.show_preview;
    }

    /// Scroll preview down.
    pub fn scroll_down(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_add(5);
    }

    /// Scroll preview up.
    pub fn scroll_up(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_sub(5);
    }
}

impl Default for ExplorerState {
    fn default() -> Self {
        Self::new()
    }
}
