//! File browser feature - file system navigation for opening NetCDF files.
//!
//! This module contains state management and business logic for browsing
//! the file system to select NetCDF files to open.

pub mod ui;

use std::fs;
use std::path::PathBuf;

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

/// File browser state.
#[derive(Debug)]
pub struct FileBrowserState {
    /// Current directory being browsed.
    pub current_dir: PathBuf,
    /// File entries in current directory.
    pub entries: Vec<FileEntry>,
    /// Cursor position.
    pub cursor: usize,
    /// Scroll offset.
    pub scroll: usize,
    /// Show hidden dot-prefixed entries.
    pub show_hidden: bool,
}

impl FileBrowserState {
    /// Create a new file browser state.
    pub fn new() -> Self {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self {
            current_dir,
            entries: Vec::new(),
            cursor: 0,
            scroll: 0,
            show_hidden: false,
        }
    }

    /// Load directory contents.
    pub fn load_directory(&mut self) {
        self.entries.clear();

        // Add parent directory entry if not at root
        if let Some(parent) = self.current_dir.parent() {
            self.entries.push(FileEntry {
                path: parent.to_path_buf(),
                name: "..".to_string(),
                is_dir: true,
                is_symlink: parent.is_symlink(),
            });
        }

        // Read directory contents
        let Ok(dir_entries) = fs::read_dir(&self.current_dir) else {
            return;
        };

        for entry in dir_entries.flatten() {
            let path = entry.path();
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            // Skip hidden files if not showing them
            if !self.show_hidden && name.starts_with('.') {
                continue;
            }

            let is_symlink = path.is_symlink();
            let is_dir = if is_symlink {
                // For symlinks, check the final target
                path.metadata().map(|m| m.is_dir()).unwrap_or(false)
            } else {
                path.is_dir()
            };

            self.entries.push(FileEntry {
                path,
                name,
                is_dir,
                is_symlink,
            });
        }

        // Sort: directories first, then files, both alphabetically
        self.entries.sort_by(|a, b| {
            if a.name == ".." {
                std::cmp::Ordering::Less
            } else if b.name == ".." {
                std::cmp::Ordering::Greater
            } else {
                match (a.is_dir, b.is_dir) {
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                }
            }
        });

        // Reset cursor
        self.cursor = 0;
        self.scroll = 0;
    }

    /// Move cursor up.
    pub fn cursor_up(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    /// Move cursor down.
    pub fn cursor_down(&mut self) {
        if self.cursor + 1 < self.entries.len() {
            self.cursor += 1;
        }
    }

    /// Get the currently selected entry.
    pub fn current_entry(&self) -> Option<&FileEntry> {
        self.entries.get(self.cursor)
    }

    /// Navigate into the selected directory or return the selected file.
    pub fn select_current(&mut self) -> Option<PathBuf> {
        let entry = self.current_entry()?.clone();

        if entry.is_dir {
            // Navigate into directory
            self.current_dir = entry.path;
            self.load_directory();
            None
        } else {
            // Return selected file
            Some(entry.path)
        }
    }

    /// Navigate to parent directory.
    pub fn go_to_parent(&mut self) {
        if let Some(parent) = self.current_dir.parent() {
            self.current_dir = parent.to_path_buf();
            self.load_directory();
        }
    }

    /// Toggle show hidden files.
    pub fn toggle_hidden(&mut self) {
        self.show_hidden = !self.show_hidden;
        self.load_directory();
    }

    /// Adjust scroll to keep cursor visible.
    pub fn adjust_scroll(&mut self, viewport_height: usize) {
        if viewport_height == 0 {
            return;
        }

        // If cursor is above the visible area, scroll up
        if self.cursor < self.scroll {
            self.scroll = self.cursor;
        }

        // If cursor is below the visible area, scroll down
        if self.cursor >= self.scroll + viewport_height {
            self.scroll = self.cursor.saturating_sub(viewport_height - 1);
        }
    }
}

impl Default for FileBrowserState {
    fn default() -> Self {
        Self::new()
    }
}
