//! Coriolis - A terminal-based NetCDF data viewer.

use anyhow::Result;
use clap::Parser;
use coriolis::app::App;
use coriolis::data_viewer::ViewMode;
use coriolis::ui;
use coriolis::util;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::path::PathBuf;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[derive(Parser, Debug)]
#[command(name = "coriolis")]
#[command(about = "A terminal-based netCDF data viewer", long_about = None)]
struct Args {
    /// Path to the NetCDF file or directory to open
    file: Option<PathBuf>,

    /// Enable logging to specified file
    #[arg(long)]
    log: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Set up logging if --log option is provided
    if let Some(log_path) = &args.log {
        match std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(log_path)
        {
            Ok(file) => {
                let subscriber = FmtSubscriber::builder()
                    .with_max_level(Level::DEBUG)
                    .with_writer(move || {
                        file.try_clone().unwrap_or_else(|_| {
                            // Fallback to /dev/null if clone fails
                            std::fs::File::create("/dev/null").expect("Cannot open /dev/null")
                        })
                    })
                    .finish();
                if tracing::subscriber::set_global_default(subscriber).is_ok() {
                    tracing::info!("Starting Coriolis");
                }
            },
            Err(e) => {
                eprintln!(
                    "Warning: Failed to open log file '{}': {}",
                    log_path.display(),
                    e
                );
            },
        }
    }

    // Validate path if provided
    if let Some(ref path) = args.file {
        if !path.exists() {
            eprintln!("Error: Path not found: {}", path.display());
            std::process::exit(1);
        }
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run app
    let app = App::new(args.file);
    let res = run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {}", err);
    }

    // Log exit if logging was successfully set up
    tracing::info!("Coriolis exited");

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<()> {
    let mut pending_g = false; // For 'gg' vim binding

    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Overlay mode - handle separately
                if app.data_viewer.visible {
                    match (key.modifiers, key.code) {
                        // Close overlay
                        (KeyModifiers::NONE, KeyCode::Esc)
                        | (KeyModifiers::NONE, KeyCode::Char('q'))
                        | (KeyModifiers::NONE, KeyCode::Char('p')) => {
                            app.data_viewer.close();
                            app.status = "Data viewer closed".to_string();
                        },
                        // Cycle view mode with Tab
                        (KeyModifiers::NONE, KeyCode::Tab) => {
                            app.data_viewer.cycle_view_mode();
                            let view_name = app.data_viewer.view_mode.name();
                            app.data_viewer.set_status(format!("View: {}", view_name));
                        },
                        // Cycle color palette with C
                        (KeyModifiers::NONE, KeyCode::Char('c'))
                        | (KeyModifiers::NONE, KeyCode::Char('C')) => {
                            app.data_viewer.cycle_color_palette();
                            let palette_name = app.data_viewer.color_palette.name();
                            app.data_viewer.set_status(format!("Palette: {}", palette_name));
                        },
                        // Contextual arrows/hjkl
                        // Table: pan; Plot1D: move cursor; Heatmap: move crosshair
                        (KeyModifiers::NONE, KeyCode::Up)
                        | (KeyModifiers::NONE, KeyCode::Char('k')) => {
                            match app.data_viewer.view_mode {
                                ViewMode::Table => app.data_viewer.scroll_up(1),
                                ViewMode::Heatmap => app.data_viewer.move_heat_cursor(-1, 0),
                                ViewMode::Plot1D => { /* reserved for future y-zoom */ },
                            }
                        },
                        (KeyModifiers::NONE, KeyCode::Down)
                        | (KeyModifiers::NONE, KeyCode::Char('j')) => {
                            match app.data_viewer.view_mode {
                                ViewMode::Table => app.data_viewer.scroll_down(1),
                                ViewMode::Heatmap => app.data_viewer.move_heat_cursor(1, 0),
                                ViewMode::Plot1D => { /* reserved for future y-zoom */ },
                            }
                        },
                        (KeyModifiers::NONE, KeyCode::Left)
                        | (KeyModifiers::NONE, KeyCode::Char('h')) => match app.data_viewer.view_mode {
                            ViewMode::Table => app.data_viewer.scroll_left(1),
                            ViewMode::Heatmap => app.data_viewer.move_heat_cursor(0, -1),
                            ViewMode::Plot1D => app.data_viewer.plot_cursor_left(),
                        },
                        (KeyModifiers::NONE, KeyCode::Right)
                        | (KeyModifiers::NONE, KeyCode::Char('l')) => match app.data_viewer.view_mode {
                            ViewMode::Table => app.data_viewer.scroll_right(1),
                            ViewMode::Heatmap => app.data_viewer.move_heat_cursor(0, 1),
                            ViewMode::Plot1D => app.data_viewer.plot_cursor_right(),
                        },
                        // Page up/down for large scrolling
                        (KeyModifiers::CONTROL, KeyCode::Char('u')) => {
                            app.data_viewer.scroll_up(10);
                        },
                        (KeyModifiers::CONTROL, KeyCode::Char('d')) => {
                            app.data_viewer.scroll_down(10);
                        },
                        // Dimension selector navigation (Tab through dimensions for 3D+ data)
                        (KeyModifiers::NONE, KeyCode::Char('s')) => {
                            app.data_viewer.next_dim_selector();
                            let status_msg = if let Some(dim) =
                                app.data_viewer.slicing.active_dim_selector
                            {
                                if let Some(ref var) = app.data_viewer.variable {
                                    let dim_name =
                                        var.dim_names.get(dim).map(|s| s.as_str()).unwrap_or("?");
                                    Some(format!("Slicing dimension: {}", dim_name))
                                } else {
                                    None
                                }
                            } else {
                                None
                            };
                            if let Some(msg) = status_msg {
                                app.data_viewer.set_status(msg);
                            }
                        },
                        // Slice navigation with PageUp/PageDown
                        (KeyModifiers::NONE, KeyCode::PageUp) => {
                            app.data_viewer.increment_active_slice();
                        },
                        (KeyModifiers::NONE, KeyCode::PageDown) => {
                            app.data_viewer.decrement_active_slice();
                        },
                        // Also keep +/- for slice navigation
                        (KeyModifiers::NONE, KeyCode::Char(']'))
                        | (KeyModifiers::NONE, KeyCode::Char('+'))
                        | (KeyModifiers::NONE, KeyCode::Char('=')) => {
                            app.data_viewer.increment_active_slice();
                        },
                        (KeyModifiers::NONE, KeyCode::Char('['))
                        | (KeyModifiers::NONE, KeyCode::Char('-'))
                        | (KeyModifiers::NONE, KeyCode::Char('_')) => {
                            app.data_viewer.decrement_active_slice();
                        },
                        // Change which dimensions are displayed
                        (KeyModifiers::NONE, KeyCode::Char('r'))
                        | (KeyModifiers::NONE, KeyCode::Char('R')) => {
                            app.data_viewer.rotate_display_dims();
                            app.data_viewer
                                .set_status("Rotated Y ↔ X dimensions".to_string());
                        },
                        // Simplified UI: removed 1D options (auto/log/agg) and heatmap range/zoom/pan toggles
                        // Clipboard export remains below
                        (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                            app.data_viewer.copy_visible_to_clipboard();
                            app.data_viewer
                                .set_status("Copied visible data to clipboard (TSV)".to_string());
                        },
                        (KeyModifiers::NONE, KeyCode::Char('y'))
                        | (KeyModifiers::NONE, KeyCode::Char('Y')) => {
                            app.data_viewer.cycle_display_dim(0);
                            // Show which dimension was selected
                            let status_msg = if let Some(ref var) = app.data_viewer.variable {
                                let dim_idx = app.data_viewer.slicing.display_dims.0;
                                let dim_name = var
                                    .dim_names
                                    .get(dim_idx)
                                    .map(|s| s.as_str())
                                    .unwrap_or("?");
                                format!("Y dimension: {}", dim_name)
                            } else {
                                "Cycled Y dimension".to_string()
                            };
                            app.data_viewer.set_status(status_msg);
                        },
                        (KeyModifiers::NONE, KeyCode::Char('x'))
                        | (KeyModifiers::NONE, KeyCode::Char('X')) => {
                            app.data_viewer.cycle_display_dim(1);
                            // Show which dimension was selected
                            let status_msg = if let Some(ref var) = app.data_viewer.variable {
                                let dim_idx = app.data_viewer.slicing.display_dims.1;
                                let dim_name = var
                                    .dim_names
                                    .get(dim_idx)
                                    .map(|s| s.as_str())
                                    .unwrap_or("?");
                                format!("X dimension: {}", dim_name)
                            } else {
                                "Cycled X dimension".to_string()
                            };
                            app.data_viewer.set_status(status_msg);
                        },
                        // Toggle scale/offset
                        (KeyModifiers::NONE, KeyCode::Char('o'))
                        | (KeyModifiers::NONE, KeyCode::Char('O')) => {
                            if app.data_viewer.has_scale_offset() {
                                app.data_viewer.toggle_scale_offset();
                                let mode = if app.data_viewer.apply_scale_offset {
                                    "Scaled"
                                } else {
                                    "Raw"
                                };
                                app.data_viewer.set_status(format!(
                                    "Data: {} (scale={}, offset={})",
                                    mode,
                                    app.data_viewer.scale_factor(),
                                    app.data_viewer.add_offset()
                                ));
                            } else {
                                app.data_viewer
                                    .set_status("No scale/offset for this variable".to_string());
                            }
                        },
                        _ => {},
                    }
                    continue;
                }

                // Search mode - handle separately
                if app.search.is_active() {
                    match key.code {
                        KeyCode::Enter => {
                            app.search.submit();
                            if let Some(ref dataset) = app.dataset {
                                app.explorer.expand_all();
                                app.search.perform_search(&dataset.root_node);

                                if let Some(path) = app.search.current_match_path() {
                                    app.explorer.goto_node(path);
                                }
                            }
                        },
                        KeyCode::Esc => app.search.cancel(),
                        KeyCode::Backspace => app.search.backspace(),
                        KeyCode::Char(c) => app.search.input(c),
                        _ => {},
                    }
                    continue;
                }

                // File browser mode
                if app.file_browser_mode {
                    match (key.modifiers, key.code) {
                        // Quit
                        (KeyModifiers::NONE, KeyCode::Char('q')) => return Ok(()),

                        // Navigation
                        (KeyModifiers::NONE, KeyCode::Up)
                        | (KeyModifiers::NONE, KeyCode::Char('k')) => {
                            app.browser_up();
                        },
                        (KeyModifiers::NONE, KeyCode::Down)
                        | (KeyModifiers::NONE, KeyCode::Char('j')) => {
                            app.browser_down();
                        },

                        // Select/Open
                        (KeyModifiers::NONE, KeyCode::Enter)
                        | (KeyModifiers::NONE, KeyCode::Char('l'))
                        | (KeyModifiers::NONE, KeyCode::Right) => {
                            app.browser_select();
                        },

                        // Go to parent directory
                        (KeyModifiers::NONE, KeyCode::Char('h'))
                        | (KeyModifiers::NONE, KeyCode::Left) => {
                            if let Some(parent) = app.file_browser.current_dir.parent() {
                                app.file_browser.current_dir = parent.to_path_buf();
                                app.file_browser.load_directory();
                            }
                        },

                        // Toggle show hidden
                        (KeyModifiers::NONE, KeyCode::Char('.')) => {
                            app.toggle_hidden();
                        },

                        _ => {},
                    }
                    continue;
                }

                // Normal mode
                match (key.modifiers, key.code) {
                    // Quit
                    (KeyModifiers::NONE, KeyCode::Char('q')) => return Ok(()),

                    // Navigation
                    (KeyModifiers::NONE, KeyCode::Up)
                    | (KeyModifiers::NONE, KeyCode::Char('k')) => {
                        app.explorer.cursor_up();
                        app.explorer.preview_scroll = 0;
                    },
                    (KeyModifiers::NONE, KeyCode::Down)
                    | (KeyModifiers::NONE, KeyCode::Char('j')) => {
                        app.explorer.cursor_down();
                        app.explorer.preview_scroll = 0;
                    },
                    (KeyModifiers::NONE, KeyCode::Left)
                    | (KeyModifiers::NONE, KeyCode::Char('h')) => {
                        app.explorer.collapse_current();
                    },
                    (KeyModifiers::NONE, KeyCode::Right)
                    | (KeyModifiers::NONE, KeyCode::Char('l')) => {
                        app.explorer.expand_current();
                    },

                    // Vim navigation
                    (KeyModifiers::NONE, KeyCode::Char('g')) => {
                        if pending_g {
                            app.explorer.goto_first();
                            app.explorer.preview_scroll = 0;
                            pending_g = false;
                        } else {
                            pending_g = true;
                        }
                    },
                    (KeyModifiers::SHIFT, KeyCode::Char('G')) => {
                        app.explorer.goto_last();
                        app.explorer.preview_scroll = 0;
                    },
                    (KeyModifiers::CONTROL, KeyCode::Char('f')) => {
                        for _ in 0..15 {
                            app.explorer.cursor_down();
                        }
                        app.explorer.preview_scroll = 0;
                    },
                    (KeyModifiers::CONTROL, KeyCode::Char('b')) => {
                        for _ in 0..15 {
                            app.explorer.cursor_up();
                        }
                        app.explorer.preview_scroll = 0;
                    },

                    // Search
                    (KeyModifiers::NONE, KeyCode::Char('/')) => {
                        app.search.start();
                    },
                    (KeyModifiers::NONE, KeyCode::Char('n')) => {
                        app.search.next_match();
                        if let Some(path) = app.search.current_match_path() {
                            app.explorer.goto_node(path);
                        }
                    },
                    (KeyModifiers::SHIFT, KeyCode::Char('N')) => {
                        app.search.prev_match();
                        if let Some(path) = app.search.current_match_path() {
                            app.explorer.goto_node(path);
                        }
                    },

                    // Features
                    (KeyModifiers::NONE, KeyCode::Char('p')) => {
                        app.toggle_data_viewer();
                    },
                    (KeyModifiers::NONE, KeyCode::Char('t')) => {
                        app.toggle_preview();
                    },
                    (KeyModifiers::NONE, KeyCode::Char('f')) => {
                        app.open_file_browser_at_current();
                    },
                    (KeyModifiers::SHIFT, KeyCode::Char('T')) => {
                        app.cycle_theme();
                    },
                    (KeyModifiers::SHIFT, KeyCode::Char('?')) => {
                        app.status = "Help: q=quit, j/k=nav, /=search, t=toggle preview, T=theme, c=copy tree, y=copy node".to_string();
                    },

                    // Clipboard
                    (KeyModifiers::NONE, KeyCode::Char('c')) => {
                        if let Some(ref dataset) = app.dataset {
                            let file_name = app
                                .file_path
                                .as_ref()
                                .and_then(|p| p.file_name())
                                .map(|n| n.to_string_lossy().to_string());
                            match util::clipboard::copy_tree_structure(
                                &dataset.root_node,
                                file_name.as_deref(),
                            ) {
                                Ok(_) => app.status = "Tree copied!".to_string(),
                                Err(e) => app.status = format!("Copy failed: {}", e),
                            }
                        } else {
                            app.status = "No file loaded".to_string();
                        }
                    },
                    (KeyModifiers::NONE, KeyCode::Char('y')) => {
                        if let Some(node) = app.current_node() {
                            match util::clipboard::copy_node_info(node) {
                                Ok(_) => app.status = format!("Copied {}!", node.name),
                                Err(e) => app.status = format!("Copy failed: {}", e),
                            }
                        } else {
                            app.status = "No node selected".to_string();
                        }
                    },

                    // Preview scrolling
                    (KeyModifiers::CONTROL, KeyCode::Char('d'))
                    | (KeyModifiers::SHIFT, KeyCode::Char('J')) => {
                        app.scroll_preview_down();
                    },
                    (KeyModifiers::CONTROL, KeyCode::Char('u'))
                    | (KeyModifiers::SHIFT, KeyCode::Char('K')) => {
                        app.scroll_preview_up();
                    },

                    // Escape - close overlays
                    (KeyModifiers::NONE, KeyCode::Esc) => {
                        app.close_overlay();
                    },

                    _ => {
                        pending_g = false;
                    },
                }
            }
        }
    }
}
