use crate::data::{DataNode, DatasetInfo};

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

/// Tree navigation state.
///
/// The tree maintains a flat list of visible items. When nodes are expanded/collapsed,
/// the list is rebuilt to reflect the new visibility state.
#[derive(Debug)]
pub struct TreeState {
    /// All tree items in display order (only visible items).
    items: Vec<TreeItem>,
    /// Cursor position (index into items).
    cursor: usize,
    /// The root node for rebuilding.
    root: Option<DataNode>,
    /// Set of expanded node paths.
    expanded_paths: std::collections::HashSet<String>,
}

impl TreeState {
    /// Create a new tree state.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            cursor: 0,
            root: None,
            expanded_paths: std::collections::HashSet::new(),
        }
    }

    /// Build tree from dataset.
    pub fn build_from_dataset(&mut self, dataset: &DatasetInfo) {
        self.root = Some(dataset.root_node.clone());
        // Root is expanded by default
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

        // Only add children if this node is expanded
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
        }
    }

    /// Move the cursor down one position.
    pub fn cursor_down(&mut self) {
        if self.cursor + 1 < self.items.len() {
            self.cursor += 1;
        }
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

    /// Toggle the expansion state of a node at the given index.
    #[allow(dead_code)]
    pub fn toggle_expand(&mut self, index: usize) {
        if index >= self.items.len() {
            return;
        }

        let item = &self.items[index];
        let path = item.node.path.clone();

        if item.expanded {
            self.expanded_paths.remove(&path);
        } else {
            self.expanded_paths.insert(path);
        }

        self.rebuild_visible_items();
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
    /// Since we rebuild on expand/collapse, all items in the list are visible.
    pub fn visible_items(&self) -> Vec<&TreeItem> {
        self.items.iter().collect()
    }

    /// Get the current cursor position.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Check if a node is expanded.
    #[allow(dead_code)]
    pub fn is_expanded(&self, path: &str) -> bool {
        self.expanded_paths.contains(path)
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

    fn collect_group_paths(node: &DataNode, paths: &mut std::collections::HashSet<String>) {
        if node.is_group() {
            paths.insert(node.path.clone());
        }
        for child in &node.children {
            Self::collect_group_paths(child, paths);
        }
    }
}

impl Default for TreeState {
    fn default() -> Self {
        Self::new()
    }
}
