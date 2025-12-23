//! Application state and logic.

use std::fs;
use std::path::PathBuf;

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
    /// Full path to the file/directory (or symlink itself).
    pub path: PathBuf,
    /// Display name (basename of path).
    pub name: String,
    /// Is this entry a directory (final target if symlink resolves)?
    pub is_dir: bool,
    /// Is this entry a symlink?
    pub is_symlink: bool,
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
    /// File browser scroll offset.
    pub file_browser_scroll: usize,
    /// Show hidden dot-prefixed entries in file browser.
    pub show_hidden: bool,
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
            file_browser_scroll: 0,
            show_hidden: false,
        };

        // Check if we need to show file browser
        match file_path {
            Some(path) if path.is_dir() => {
                // Directory provided, show browser
                app.current_dir = path;
                app.load_directory();
                app.file_browser_mode = true;
            },
            Some(path) if path.is_file() => {
                // File provided, load it
                app.load_file(path);
            },
            None => {
                // No path provided, show browser
                app.load_directory();
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

        // Canonicalize the path to get absolute path (handles relative paths correctly)
        let canonical_path = match fs::canonicalize(&path) {
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
                self.tree_cursor.build_from_dataset(&dataset);
                self.status = format!(
                    "{} loaded",
                    canonical_path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "file".to_string())
                );
                self.error_message = None;

                // Update file_path with canonical (absolute) path so preview and other features work
                self.file_path = Some(canonical_path.clone());

                // Update current_dir to the file's directory for consistent file browser behavior
                if let Some(parent) = canonical_path.parent() {
                    self.current_dir = parent.to_path_buf();
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
            },
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
            },
        };

        self.status = format!("Loading {}...", node.name);

        match read_variable(&file_path, &node.path) {
            Ok(loaded_var) => {
                self.overlay.load_variable(loaded_var);
                self.status = format!("Loaded {}", node.name);
            },
            Err(e) => {
                self.overlay
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

    /// Toggle show hidden files.
    pub fn toggle_hidden(&mut self) {
        self.show_hidden = !self.show_hidden;
        self.status = if self.file_browser_mode {
            "Show hidden: ON".to_string()
        } else {
            "Show hidden: OFF".to_string()
        };
    }

    /// Load directory contents for file browser.
    pub fn load_directory(&mut self) {
        self.file_entries.clear();

        // Add parent directory entry if not at root
        if let Some(parent) = self.current_dir.parent() {
            self.file_entries.push(FileEntry {
                path: parent.to_path_buf(),
                name: "..".to_string(),
                is_dir: true,
                is_symlink: parent.is_symlink(),
            });
        }

        match fs::read_dir(&self.current_dir) {
            Ok(entries) => {
                let mut dirs = Vec::new();
                let mut files = Vec::new();

                for entry in entries.flatten() {
                    let path = entry.path();
                    let name = entry.file_name().to_string_lossy().to_string();

                    // Skip hidden files
                    if name.starts_with('.') && !self.show_hidden {
                        continue;
                    }

                    // Get symlink status using file_type (doesn't follow symlinks)
                    let is_symlink = entry.file_type().map(|t| t.is_symlink()).unwrap_or(false);

                    // Try to get metadata (follows symlinks to check target type)
                    // If this fails (broken symlink, permissions, etc.), try symlink_metadata as fallback
                    let metadata = entry.metadata().or_else(|_| fs::symlink_metadata(&path));

                    match metadata {
                        Ok(meta) => {
                            let file_entry = FileEntry {
                                path: path.clone(),
                                name,
                                is_dir: meta.is_dir(),
                                is_symlink,
                            };

                            if meta.is_dir() {
                                dirs.push(file_entry);
                            } else {
                                // Only show netcdf files (case-insensitive)
                                if let Some(ext) = path.extension() {
                                    let ext_str = ext.to_string_lossy().to_lowercase();
                                    if ext_str == "nc" || ext_str == "nc4" || ext_str == "netcdf" {
                                        files.push(file_entry);
                                    }
                                }
                            }
                        },
                        Err(_) => {
                            // If we can't get any metadata, skip this entry
                            // (this is rare and usually indicates serious permission issues)
                            continue;
                        },
                    }
                }

                // Sort directories and files alphabetically
                dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
                files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

                // Add directories first, then files
                self.file_entries.extend(dirs);
                self.file_entries.extend(files);

                self.status = format!("Browsing: {}", self.current_dir.display());
            },
            Err(e) => {
                self.error_message = Some(format!("Failed to read directory: {}", e));
                self.status = "Error reading directory".to_string();
            },
        }

        // Reset cursor and scroll
        self.file_browser_cursor = 0;
        self.file_browser_scroll = 0;
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
            // Try to load file
            self.file_browser_mode = false;
            self.load_file(entry.path.clone());

            // If loading failed, return to browser mode
            if self.error_message.is_some() {
                self.file_browser_mode = true;
                self.status =
                    "Error loading file (press q to quit, navigate to try another)".to_string();
            }
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

    /// Adjust scroll offset to keep cursor visible in file browser.
    pub fn adjust_browser_scroll(&mut self, viewport_height: usize) {
        if viewport_height == 0 {
            return;
        }

        // Scroll down if cursor is below viewport
        if self.file_browser_cursor >= self.file_browser_scroll + viewport_height {
            self.file_browser_scroll = self.file_browser_cursor - viewport_height + 1;
        }

        // Scroll up if cursor is above viewport
        if self.file_browser_cursor < self.file_browser_scroll {
            self.file_browser_scroll = self.file_browser_cursor;
        }
    }

    /// Open file browser starting at the current file's directory.
    pub fn open_file_browser_at_current(&mut self) {
        // Get the directory of the current file, or use current working directory
        let start_dir = self
            .file_path
            .as_ref()
            .and_then(|p| p.parent())
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

        self.current_dir = start_dir;
        self.load_directory();
        self.file_browser_mode = true;
        self.status = format!("File browser: {}", self.current_dir.display());
    }
}
