//! Application state and logic.

use std::path::PathBuf;

use crate::data::{DataNode, DataReader, DatasetInfo};
use crate::navigation::{SearchState, TreeState};

/// Application theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    /// Gruvbox dark theme.
    GruvboxDark,
    /// Gruvbox light theme.
    GruvboxLight,
}

impl Theme {
    /// Get the next theme in the cycle.
    pub fn next(self) -> Self {
        match self {
            Theme::GruvboxDark => Theme::GruvboxLight,
            Theme::GruvboxLight => Theme::GruvboxDark,
        }
    }

    /// Get the theme name.
    pub fn name(self) -> &'static str {
        match self {
            Theme::GruvboxDark => "Gruvbox Dark",
            Theme::GruvboxLight => "Gruvbox Light",
        }
    }
}

/// Application state.
#[derive(Debug)]
pub struct App {
    /// Current file path.
    pub file_path: Option<PathBuf>,
    /// Loaded dataset.
    pub dataset: Option<DatasetInfo>,
    /// Tree navigation state.
    pub tree_cursor: TreeState,
    /// Search state.
    pub search: SearchState,
    /// Show preview panel.
    pub show_preview: bool,
    /// Show plot overlay.
    pub show_plot: bool,
    /// Preview scroll offset.
    pub preview_scroll: u16,
    /// Status message.
    pub status: String,
    /// Current theme.
    pub theme: Theme,
    /// Loading indicator.
    pub loading: bool,
    /// Error message.
    pub error_message: Option<String>,
}

impl App {
    /// Create a new application instance.
    pub fn new(file_path: Option<PathBuf>) -> Self {
        let mut app = Self {
            file_path: file_path.clone(),
            dataset: None,
            tree_cursor: TreeState::new(),
            search: SearchState::new(),
            show_preview: true,
            show_plot: false,
            preview_scroll: 0,
            status: "Ready".to_string(),
            theme: Theme::GruvboxDark,
            loading: false,
            error_message: None,
        };

        // Load file if provided
        if let Some(path) = file_path {
            app.load_file(path);
        }

        app
    }

    /// Load a file.
    pub fn load_file(&mut self, path: PathBuf) {
        self.loading = true;
        self.status = format!(
            "Loading {}...",
            path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "file".to_string())
        );

        match DataReader::read_file(&path) {
            Ok(dataset) => {
                self.dataset = Some(dataset.clone());
                self.tree_cursor.build_from_dataset(&dataset);
                self.status = format!(
                    "{} loaded",
                    path.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "file".to_string())
                );
                self.error_message = None;
                tracing::info!("File loaded successfully");
            },
            Err(e) => {
                self.error_message = Some(format!("Error loading file: {}", e));
                self.status = "Error loading file".to_string();
                tracing::error!("Error loading file: {}", e);
            },
        }
        self.loading = false;
    }

    /// Get the current node.
    pub fn current_node(&self) -> Option<&DataNode> {
        self.tree_cursor.current_node()
    }

    /// Toggle preview panel.
    pub fn toggle_preview(&mut self) {
        self.show_preview = !self.show_preview;
        self.status = if self.show_preview {
            "Preview: ON".to_string()
        } else {
            "Preview: OFF".to_string()
        };
    }

    /// Toggle plot overlay.
    pub fn toggle_plot(&mut self) {
        if let Some(node) = self.current_node() {
            if node.is_variable() {
                self.show_plot = !self.show_plot;
                self.status = if self.show_plot {
                    "Plot: ON".to_string()
                } else {
                    "Plot: OFF".to_string()
                };
            } else {
                self.status = "Plot only available for variables".to_string();
            }
        }
    }

    /// Cycle to the next theme.
    pub fn cycle_theme(&mut self) {
        self.theme = self.theme.next();
        self.status = format!("Theme: {}", self.theme.name());
    }

    /// Scroll preview down.
    pub fn scroll_preview_down(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_add(5);
    }

    /// Scroll preview up.
    pub fn scroll_preview_up(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_sub(5);
    }

    /// Close any open overlay.
    pub fn close_overlay(&mut self) {
        self.show_plot = false;
        self.search.cancel();
    }
}
