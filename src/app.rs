//! Application state and logic.

use std::path::PathBuf;
use std::fs;

use crate::data::{read_variable, DataNode, DataReader, DatasetInfo};
use crate::navigation::{SearchState, TreeState};
use crate::overlay::OverlayState;

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

/// File browser entry.
#[derive(Debug, Clone)]
pub struct FileEntry {
    /// Full path to the file/directory.
    pub path: PathBuf,
    /// Display name.
    pub name: String,
    /// Is this entry a directory?
    pub is_dir: bool,
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
    /// Data overlay state.
    pub overlay: OverlayState,
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
    /// File browser mode.
    pub file_browser_mode: bool,
    /// Current directory being browsed.
    pub current_dir: PathBuf,
    /// File entries in current directory.
    pub file_entries: Vec<FileEntry>,
    /// File browser cursor position.
    pub file_browser_cursor: usize,
}

impl App {
    /// Create a new application instance.
    pub fn new(file_path: Option<PathBuf>) -> Self {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        let mut app = Self {
            file_path: file_path.clone(),
            dataset: None,
            tree_cursor: TreeState::new(),
            search: SearchState::new(),
            show_preview: true,
            overlay: OverlayState::new(),
            preview_scroll: 0,
            status: "Ready".to_string(),
            theme: Theme::GruvboxDark,
            loading: false,
            error_message: None,
            file_browser_mode: false,
            current_dir: current_dir.clone(),
            file_entries: Vec::new(),
            file_browser_cursor: 0,
        };

        // Check if we need to show file browser
        match file_path {
            Some(path) if path.is_dir() => {
                // Directory provided, show browser
                app.current_dir = path;
                app.load_directory();
                app.file_browser_mode = true;
            }
            Some(path) if path.is_file() => {
                // File provided, load it
                app.load_file(path);
            }
            None => {
                // No path provided, show browser
                app.load_directory();
                app.file_browser_mode = true;
            }
            _ => {
                app.error_message = Some("Invalid path provided".to_string());
            }
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

    /// Toggle data overlay for viewing variable content.
    pub fn toggle_data_viewer(&mut self) {
        // If overlay is already visible, close it
        if self.overlay.visible {
            self.overlay.close();
            self.status = "Data viewer closed".to_string();
            return;
        }

        // Check if we have a variable selected
        let node = match self.current_node() {
            Some(n) => n.clone(),
            None => {
                self.status = "No node selected".to_string();
                return;
            }
        };

        if !node.is_variable() {
            self.status = "Data viewer only available for variables".to_string();
            return;
        }

        // Try to load the variable data
        let file_path = match &self.file_path {
            Some(p) => p.clone(),
            None => {
                self.status = "No file loaded".to_string();
                return;
            }
        };

        self.status = format!("Loading {}...", node.name);

        match read_variable(&file_path, &node.path) {
            Ok(loaded_var) => {
                self.overlay.load_variable(loaded_var);
                self.status = format!("Loaded {}", node.name);
            }
            Err(e) => {
                self.overlay.set_error(format!("Failed to load variable: {}", e));
                self.status = format!("Error loading {}", node.name);
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
        self.overlay.close();
        self.search.cancel();
    }

    /// Load directory contents for file browser.
    pub fn load_directory(&mut self) {
        self.file_entries.clear();

        // Add parent directory entry if not at root
        if self.current_dir.parent().is_some() {
            self.file_entries.push(FileEntry {
                path: self.current_dir.parent().unwrap().to_path_buf(),
                name: "..".to_string(),
                is_dir: true,
            });
        }

        match fs::read_dir(&self.current_dir) {
            Ok(entries) => {
                let mut dirs = Vec::new();
                let mut files = Vec::new();

                for entry in entries.flatten() {
                    if let Ok(metadata) = entry.metadata() {
                        let path = entry.path();
                        let name = entry.file_name().to_string_lossy().to_string();

                        // Skip hidden files
                        if name.starts_with('.') {
                            continue;
                        }

                        let file_entry = FileEntry {
                            path: path.clone(),
                            name,
                            is_dir: metadata.is_dir(),
                        };

                        if metadata.is_dir() {
                            dirs.push(file_entry);
                        } else {
                            // Only show netcdf files
                            if let Some(ext) = path.extension() {
                                let ext_str = ext.to_string_lossy();
                                if ext_str == "nc" || ext_str == "nc4" || ext_str == "netcdf" {
                                    files.push(file_entry);
                                }
                            }
                        }
                    }
                }

                // Sort directories and files alphabetically
                dirs.sort_by(|a, b| a.name.cmp(&b.name));
                files.sort_by(|a, b| a.name.cmp(&b.name));

                // Add directories first, then files
                self.file_entries.extend(dirs);
                self.file_entries.extend(files);

                self.status = format!("Browsing: {}", self.current_dir.display());
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to read directory: {}", e));
                self.status = "Error reading directory".to_string();
            }
        }

        // Reset cursor
        self.file_browser_cursor = 0;
    }

    /// Navigate to selected file/directory in browser.
    pub fn browser_select(&mut self) {
        if self.file_entries.is_empty() {
            return;
        }

        let entry = &self.file_entries[self.file_browser_cursor];

        if entry.is_dir {
            // Navigate to directory
            self.current_dir = entry.path.clone();
            self.load_directory();
        } else {
            // Load file and exit browser mode
            self.file_browser_mode = false;
            self.load_file(entry.path.clone());
        }
    }

    /// Move cursor up in file browser.
    pub fn browser_up(&mut self) {
        if self.file_browser_cursor > 0 {
            self.file_browser_cursor -= 1;
        }
    }

    /// Move cursor down in file browser.
    pub fn browser_down(&mut self) {
        if self.file_browser_cursor + 1 < self.file_entries.len() {
            self.file_browser_cursor += 1;
        }
    }
}
