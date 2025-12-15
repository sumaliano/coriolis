//! Dataset information.

use super::DataNode;
use std::path::PathBuf;

/// Information about a loaded dataset.
#[derive(Debug, Clone)]
pub struct DatasetInfo {
    /// Path to the source file.
    #[allow(dead_code)]
    pub file_path: PathBuf,
    /// Root node of the data tree.
    pub root_node: DataNode,
}

impl DatasetInfo {
    /// Create a new dataset info.
    pub fn new(file_path: PathBuf, root_node: DataNode) -> Self {
        Self {
            file_path,
            root_node,
        }
    }
}
