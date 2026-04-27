//! Search functionality.

use crate::data::DataNode;

/// Search state.
#[derive(Debug)]
pub struct SearchState {
    is_active: bool,
    buffer: String,
    query: String,
    matches: Vec<String>,
    current_match: usize,
}

impl SearchState {
    /// Create a new search state.
    pub fn new() -> Self {
        Self {
            is_active: false,
            buffer: String::new(),
            query: String::new(),
            matches: Vec::new(),
            current_match: 0,
        }
    }

    /// Check if the search is active.
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// Start a search.
    pub fn start(&mut self) {
        self.is_active = true;
        self.buffer.clear();
    }

    /// Add a character to the search buffer.
    pub fn input(&mut self, c: char) {
        self.buffer.push(c);
    }

    /// Remove the last character from the search buffer.
    pub fn backspace(&mut self) {
        self.buffer.pop();
    }

    /// Submit the search.
    pub fn submit(&mut self) {
        if !self.buffer.is_empty() {
            self.query = self.buffer.clone();
        }
        self.buffer.clear();
        self.is_active = false;
    }

    /// Cancel the search.
    pub fn cancel(&mut self) {
        self.is_active = false;
        self.buffer.clear();
        self.matches.clear();
        self.current_match = 0;
    }

    /// Perform a search on a node tree.
    pub fn perform_search(&mut self, root: &DataNode) {
        self.matches.clear();
        self.current_match = 0;

        if self.query.is_empty() {
            return;
        }

        self.search_node(root);
    }

    fn search_node(&mut self, node: &DataNode) {
        if node.matches_search(&self.query) {
            self.matches.push(node.path.clone());
        }

        for child in &node.children {
            self.search_node(child);
        }
    }

    /// Get the current match path.
    pub fn current_match_path(&self) -> Option<&str> {
        self.matches.get(self.current_match).map(|s| s.as_str())
    }

    /// Move to the next match.
    pub fn next_match(&mut self) {
        if !self.matches.is_empty() {
            self.current_match = (self.current_match + 1) % self.matches.len();
        }
    }

    /// Move to the previous match.
    pub fn prev_match(&mut self) {
        if !self.matches.is_empty() {
            if self.current_match == 0 {
                self.current_match = self.matches.len() - 1;
            } else {
                self.current_match -= 1;
            }
        }
    }

    /// Get the search buffer.
    pub fn buffer(&self) -> &str {
        &self.buffer
    }

    /// Get the search query.
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Get the number of matches.
    pub fn match_count(&self) -> usize {
        self.matches.len()
    }

    /// Get the current match index.
    pub fn current_match_index(&self) -> usize {
        self.current_match
    }
}

impl Default for SearchState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{DataNode, NodeType};

    fn make_tree() -> DataNode {
        let mut root = DataNode::new("root.nc".to_string(), "/".to_string(), NodeType::Root);
        let mut group = DataNode::new("ocean".to_string(), "/ocean".to_string(), NodeType::Group);
        let temp = DataNode::new(
            "temperature".to_string(),
            "/ocean/temperature".to_string(),
            NodeType::Variable,
        );
        let salt = DataNode::new(
            "salinity".to_string(),
            "/ocean/salinity".to_string(),
            NodeType::Variable,
        );
        group.add_child(temp);
        group.add_child(salt);
        root.add_child(group);
        root
    }

    #[test]
    fn search_finds_exact_name_match() {
        let tree = make_tree();
        let mut state = SearchState::new();
        state.start();
        for c in "temperature".chars() {
            state.input(c);
        }
        state.submit();
        state.perform_search(&tree);
        assert_eq!(state.match_count(), 1);
        assert_eq!(state.current_match_path(), Some("/ocean/temperature"));
    }

    #[test]
    fn search_is_case_insensitive() {
        let tree = make_tree();
        let mut state = SearchState::new();
        state.start();
        for c in "SALINITY".chars() {
            state.input(c);
        }
        state.submit();
        state.perform_search(&tree);
        assert_eq!(state.match_count(), 1);
    }

    #[test]
    fn search_returns_no_matches_for_unknown_query() {
        let tree = make_tree();
        let mut state = SearchState::new();
        state.start();
        for c in "pressure".chars() {
            state.input(c);
        }
        state.submit();
        state.perform_search(&tree);
        assert_eq!(state.match_count(), 0);
        assert_eq!(state.current_match_path(), None);
    }

    #[test]
    fn search_navigates_next_and_prev() {
        let tree = make_tree();
        let mut state = SearchState::new();
        state.start();
        for c in "ocean".chars() {
            state.input(c);
        }
        state.submit();
        state.perform_search(&tree);
        // Should match "/ocean" group and both children
        assert!(state.match_count() >= 2);

        let first = state.current_match_path().unwrap().to_string();
        state.next_match();
        let second = state.current_match_path().unwrap().to_string();
        assert_ne!(first, second);

        state.prev_match();
        assert_eq!(state.current_match_path().unwrap(), first);
    }

    #[test]
    fn cancel_clears_matches() {
        let tree = make_tree();
        let mut state = SearchState::new();
        state.start();
        for c in "temp".chars() {
            state.input(c);
        }
        state.submit();
        state.perform_search(&tree);
        assert!(state.match_count() > 0);

        state.cancel();
        assert_eq!(state.match_count(), 0);
        assert!(!state.is_active());
    }

    #[test]
    fn backspace_removes_last_char_from_buffer() {
        let mut state = SearchState::new();
        state.start();
        state.input('a');
        state.input('b');
        state.input('c');
        state.backspace();
        assert_eq!(state.buffer(), "ab");
    }
}
