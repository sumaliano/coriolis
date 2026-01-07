//! Application state and logic.

use std::path::PathBuf;

use crate::data::{read_variable, DataNode, DataReader, DatasetInfo};
use crate::data_viewer::DataViewerState;
use crate::explorer::ExplorerState;
use crate::file_browser::FileBrowserState;
use crate::search::SearchState;

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
    /// Explorer state (tree navigation + details).
    pub explorer: ExplorerState,
    /// Search state.
    pub search: SearchState,
    /// Data viewer state.
    pub data_viewer: DataViewerState,
    /// File browser state.
    pub file_browser: FileBrowserState,
    /// Status message.
    pub status: String,
    /// Current theme.
    pub theme: Theme,
    /// Loading indicator.
    pub loading: bool,
    /// Error message.
    pub error_message: Option<String>,
    /// File browser mode.
    pub file_browser_mode: bool,
}

impl App {
    /// Create a new application instance.
    pub fn new(file_path: Option<PathBuf>) -> Self {
        let mut app = Self {
            file_path: file_path.clone(),
            dataset: None,
            explorer: ExplorerState::new(),
            search: SearchState::new(),
            data_viewer: DataViewerState::new(),
            file_browser: FileBrowserState::new(),
            status: "Ready".to_string(),
            theme: Theme::GruvboxDark,
            loading: false,
            error_message: None,
            file_browser_mode: false,
        };

        // Check if we need to show file browser
        match file_path {
            Some(path) if path.is_dir() => {
                app.file_browser.current_dir = path;
                app.file_browser.load_directory();
                app.file_browser_mode = true;
            },
            Some(path) if path.is_file() => {
                app.load_file(path);
            },
            None => {
                app.file_browser.load_directory();
                app.file_browser_mode = true;
            },
            _ => {
                app.error_message = Some("Invalid path provided".to_string());
            },
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

        let canonical_path = match std::fs::canonicalize(&path) {
            Ok(p) => p,
            Err(e) => {
                self.error_message = Some(format!("Failed to resolve path: {}", e));
                self.status = "Error resolving file path".to_string();
                self.loading = false;
                return;
            },
        };

        match DataReader::read_file(&canonical_path) {
            Ok(dataset) => {
                self.dataset = Some(dataset.clone());
                self.explorer.build_from_dataset(&dataset);
                self.status = format!(
                    "{} loaded",
                    canonical_path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "file".to_string())
                );
                self.error_message = None;
                self.file_path = Some(canonical_path.clone());

                if let Some(parent) = canonical_path.parent() {
                    self.file_browser.current_dir = parent.to_path_buf();
                }

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
        self.explorer.current_node()
    }

    /// Toggle preview panel.
    pub fn toggle_preview(&mut self) {
        self.explorer.toggle_preview();
        self.status = if self.explorer.show_preview {
            "Preview: ON".to_string()
        } else {
            "Preview: OFF".to_string()
        };
    }

    /// Toggle data viewer.
    pub fn toggle_data_viewer(&mut self) {
        if self.data_viewer.visible {
            self.data_viewer.close();
            self.status = "Data viewer closed".to_string();
            return;
        }

        let node = match self.current_node() {
            Some(n) => n.clone(),
            None => {
                self.status = "No node selected".to_string();
                return;
            },
        };

        if !node.is_variable() {
            self.status = "Data viewer only available for variables".to_string();
            return;
        }

        let file_path = match &self.file_path {
            Some(p) => p.clone(),
            None => {
                self.status = "No file loaded".to_string();
                return;
            },
        };

        self.status = format!("Loading {}...", node.name);

        match read_variable(&file_path, &node.path) {
            Ok(loaded_var) => {
                self.data_viewer.load_variable(loaded_var);
                self.status = format!("Loaded {}", node.name);
            },
            Err(e) => {
                self.data_viewer
                    .set_error(format!("Failed to load variable: {}", e));
                self.status = format!("Error loading {}", node.name);
            },
        }
    }

    /// Cycle to the next theme.
    pub fn cycle_theme(&mut self) {
        self.theme = self.theme.next();
        self.status = format!("Theme: {}", self.theme.name());
    }

    /// Scroll preview down.
    pub fn scroll_preview_down(&mut self) {
        self.explorer.scroll_down();
    }

    /// Scroll preview up.
    pub fn scroll_preview_up(&mut self) {
        self.explorer.scroll_up();
    }

    /// Close any open overlays.
    pub fn close_overlay(&mut self) {
        self.data_viewer.close();
        self.search.cancel();
    }

    /// Toggle show hidden files.
    pub fn toggle_hidden(&mut self) {
        self.file_browser.toggle_hidden();
        self.status = format!(
            "Show hidden: {}",
            if self.file_browser.show_hidden {
                "ON"
            } else {
                "OFF"
            }
        );
    }

    /// Navigate to selected file/directory in browser.
    pub fn browser_select(&mut self) {
        if let Some(path) = self.file_browser.select_current() {
            self.file_browser_mode = false;
            self.load_file(path);

            if self.error_message.is_some() {
                self.file_browser_mode = true;
                self.status =
                    "Error loading file (press q to quit, navigate to try another)".to_string();
            }
        }
    }

    /// Navigate to parent directory in file browser.
    pub fn browser_parent(&mut self) {
        self.file_browser.go_to_parent();
        self.status = format!("Browsing: {}", self.file_browser.current_dir.display());
    }

    /// Move cursor up in file browser.
    pub fn browser_up(&mut self) {
        self.file_browser.cursor_up();
    }

    /// Move cursor down in file browser.
    pub fn browser_down(&mut self) {
        self.file_browser.cursor_down();
    }

    /// Open file browser.
    pub fn open_file_browser_at_current(&mut self) {
        let start_dir = self
            .file_path
            .as_ref()
            .and_then(|p| p.parent())
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

        self.file_browser.current_dir = start_dir;
        self.file_browser.load_directory();
        self.file_browser_mode = true;
        self.status = format!("File browser: {}", self.file_browser.current_dir.display());
    }
}
