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
    /// Index in the items list.
    pub index: usize,
}

/// Tree navigation state.
#[derive(Debug)]
pub struct TreeState {
    items: Vec<TreeItem>,
    cursor: usize,
}

impl TreeState {
    /// Create a new tree state.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            cursor: 0,
        }
    }

    /// Build tree from dataset.
    pub fn build_from_dataset(&mut self, dataset: &DatasetInfo) {
        self.items.clear();
        self.cursor = 0;

        // Add root node
        self.add_node_recursive(&dataset.root_node, 0, 0);
    }

    fn add_node_recursive(&mut self, node: &DataNode, level: usize, index: usize) -> usize {
        let item_index = self.items.len();

        self.items.push(TreeItem {
            node: node.clone(),
            level,
            expanded: level == 0, // Root is expanded by default
            index: item_index,
        });

        let mut next_index = index + 1;

        // Add children if expanded
        if level == 0 {
            for child in &node.children {
                next_index = self.add_node_recursive(child, level + 1, next_index);
            }
        }

        next_index
    }

    /// Move the cursor up one position.
    pub fn cursor_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    /// Move the cursor down one position.
    pub fn cursor_down(&mut self) {
        if self.cursor + 1 < self.visible_items().len() {
            self.cursor += 1;
        }
    }

    /// Expand the node at the current cursor position.
    pub fn expand_current(&mut self) {
        if self.cursor < self.items.len() {
            let current_item = &self.items[self.cursor];
            if current_item.node.is_group() && !current_item.expanded {
                self.toggle_expand(self.cursor);
            }
        }
    }

    /// Collapse the node at the current cursor position.
    pub fn collapse_current(&mut self) {
        if self.cursor < self.items.len() {
            let current_item = &self.items[self.cursor];
            if current_item.node.is_group() && current_item.expanded {
                self.toggle_expand(self.cursor);
            }
        }
    }

    /// Toggle the expansion state of a node at the given index.
    pub fn toggle_expand(&mut self, index: usize) {
        if index >= self.items.len() {
            return;
        }

        let item = &self.items[index];
        let was_expanded = item.expanded;
        let level = item.level;
        let node = item.node.clone();

        // Toggle expansion
        self.items[index].expanded = !was_expanded;

        if was_expanded {
            // Collapse: remove all children from view
            let mut i = index + 1;
            while i < self.items.len() && self.items[i].level > level {
                i += 1;
            }
            self.items.drain(index + 1..i);
        } else {
            // Expand: add children to view
            let mut insert_pos = index + 1;
            for child in &node.children {
                self.insert_node_recursive(child, level + 1, insert_pos);
                insert_pos = self.find_next_sibling_position(insert_pos, level + 1);
            }
        }
    }

    fn insert_node_recursive(&mut self, node: &DataNode, level: usize, pos: usize) {
        let item = TreeItem {
            node: node.clone(),
            level,
            expanded: false,
            index: pos,
        };

        self.items.insert(pos, item);
    }

    fn find_next_sibling_position(&self, start: usize, target_level: usize) -> usize {
        let mut pos = start;
        while pos < self.items.len() && self.items[pos].level >= target_level {
            pos += 1;
        }
        pos
    }

    /// Go to the first item.
    pub fn goto_first(&mut self) {
        self.cursor = 0;
    }

    /// Go to the last visible item.
    pub fn goto_last(&mut self) {
        let visible = self.visible_items();
        if !visible.is_empty() {
            self.cursor = visible.len() - 1;
        }
    }

    /// Get all currently visible items in the tree.
    pub fn visible_items(&self) -> Vec<&TreeItem> {
        self.items
            .iter()
            .filter(|item| self.is_visible(item))
            .collect()
    }

    fn is_visible(&self, item: &TreeItem) -> bool {
        if item.level == 0 {
            return true;
        }

        // Check if all ancestors are expanded
        let mut current_level = item.level - 1;
        let mut check_index = item.index;

        while current_level > 0 && check_index > 0 {
            check_index -= 1;
            if self.items[check_index].level == current_level {
                if !self.items[check_index].expanded {
                    return false;
                }
                current_level -= 1;
            }
        }

        true
    }

    /// Get the current cursor position.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Check if a node is expanded.
    #[allow(dead_code)]
    pub fn is_expanded(&self, path: &str) -> bool {
        self.items
            .iter()
            .find(|item| item.node.path == path)
            .map(|item| item.expanded)
            .unwrap_or(false)
    }

    /// Get the current node.
    pub fn current_node(&self) -> Option<&DataNode> {
        self.visible_items().get(self.cursor).map(|item| &item.node)
    }

    /// Move the cursor to a node with the given path.
    pub fn goto_node(&mut self, target_path: &str) {
        let visible = self.visible_items();
        for (i, item) in visible.iter().enumerate() {
            if item.node.path == target_path {
                self.cursor = i;
                return;
            }
        }
    }

    /// Expand all nodes in the tree.
    pub fn expand_all(&mut self) {
        for i in 0..self.items.len() {
            if self.items[i].node.is_group() && !self.items[i].expanded {
                self.toggle_expand(i);
            }
        }
    }
}

impl Default for TreeState {
    fn default() -> Self {
        Self::new()
    }
}
