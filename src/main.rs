//! Coriolis - A terminal-based NetCDF data viewer.

mod app;
mod data;
mod error;
mod navigation;
mod ui;
mod util;

use anyhow::Result;
use app::App;
use clap::Parser;
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
        let log_path = log_path.clone();
        let subscriber = FmtSubscriber::builder()
            .with_max_level(Level::DEBUG)
            .with_writer(move || {
                std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .append(false)
                    .open(&log_path)
                    .expect("Failed to open log file")
            })
            .finish();
        tracing::subscriber::set_global_default(subscriber)?;
        tracing::info!("Starting Coriolis");
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

    if args.log.is_some() {
        tracing::info!("Coriolis exited");
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<()> {
    let mut pending_g = false; // For 'gg' vim binding

    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Overlay mode - handle separately
                if app.overlay.visible {
                    match (key.modifiers, key.code) {
                        // Close overlay
                        (KeyModifiers::NONE, KeyCode::Esc)
                        | (KeyModifiers::NONE, KeyCode::Char('q'))
                        | (KeyModifiers::NONE, KeyCode::Char('p')) => {
                            app.overlay.close();
                            app.status = "Data viewer closed".to_string();
                        }
                        // Cycle view mode with Tab
                        (KeyModifiers::NONE, KeyCode::Tab) => {
                            app.overlay.cycle_view_mode();
                            app.status = format!("View: {}", app.overlay.view_mode.name());
                        }
                        // Cycle color palette with C
                        (KeyModifiers::NONE, KeyCode::Char('c'))
                        | (KeyModifiers::NONE, KeyCode::Char('C')) => {
                            app.overlay.cycle_color_palette();
                            app.status = format!("Palette: {}", app.overlay.color_palette.name());
                        }
                        // Pan with hjkl or arrows
                        (KeyModifiers::NONE, KeyCode::Up)
                        | (KeyModifiers::NONE, KeyCode::Char('k')) => {
                            app.overlay.scroll_up(1);
                        }
                        (KeyModifiers::NONE, KeyCode::Down)
                        | (KeyModifiers::NONE, KeyCode::Char('j')) => {
                            app.overlay.scroll_down(1);
                        }
                        (KeyModifiers::NONE, KeyCode::Left)
                        | (KeyModifiers::NONE, KeyCode::Char('h')) => {
                            app.overlay.scroll_left(1);
                        }
                        (KeyModifiers::NONE, KeyCode::Right)
                        | (KeyModifiers::NONE, KeyCode::Char('l')) => {
                            app.overlay.scroll_right(1);
                        }
                        // Page up/down for large scrolling
                        (KeyModifiers::CONTROL, KeyCode::Char('u')) => {
                            app.overlay.scroll_up(10);
                        }
                        (KeyModifiers::CONTROL, KeyCode::Char('d')) => {
                            app.overlay.scroll_down(10);
                        }
                        // Dimension selector navigation (Tab through dimensions for 3D+ data)
                        | (KeyModifiers::NONE, KeyCode::Char('s')) => {
                            app.overlay.next_dim_selector();
                            if let Some(dim) = app.overlay.active_dim_selector {
                                app.status = format!("Selected dimension {} for slicing", dim);
                            }
                        }
                        // Slice navigation with PageUp/PageDown
                        (KeyModifiers::NONE, KeyCode::PageUp) => {
                            app.overlay.increment_active_slice();
                        }
                        (KeyModifiers::NONE, KeyCode::PageDown) => {
                            app.overlay.decrement_active_slice();
                        }
                        // Also keep +/- for slice navigation
                        (KeyModifiers::NONE, KeyCode::Char(']'))
                        | (KeyModifiers::NONE, KeyCode::Char('+'))
                        | (KeyModifiers::NONE, KeyCode::Char('=')) => {
                            app.overlay.increment_active_slice();
                        }
                        (KeyModifiers::NONE, KeyCode::Char('['))
                        | (KeyModifiers::NONE, KeyCode::Char('-'))
                        | (KeyModifiers::NONE, KeyCode::Char('_')) => {
                            app.overlay.decrement_active_slice();
                        }
                        // Change which dimensions are displayed
                        (KeyModifiers::NONE, KeyCode::Char('r'))
                        | (KeyModifiers::NONE, KeyCode::Char('R')) => {
                            app.overlay.rotate_display_dims();
                            app.status = "Rotated display dimensions".to_string();
                        }
                        (KeyModifiers::NONE, KeyCode::Char('y'))
                        | (KeyModifiers::NONE, KeyCode::Char('Y')) => {
                            app.overlay.cycle_display_dim(0);
                            app.status = "Cycled Y dimension".to_string();
                        }
                        (KeyModifiers::NONE, KeyCode::Char('x'))
                        | (KeyModifiers::NONE, KeyCode::Char('X')) => {
                            app.overlay.cycle_display_dim(1);
                            app.status = "Cycled X dimension".to_string();
                        }
                        _ => {}
                    }
                    continue;
                }

                // Search mode - handle separately
                if app.search.is_active() {
                    match key.code {
                        KeyCode::Enter => {
                            app.search.submit();
                            if let Some(ref dataset) = app.dataset {
                                app.tree_cursor.expand_all();
                                app.search.perform_search(&dataset.root_node);

                                if let Some(path) = app.search.current_match_path() {
                                    app.tree_cursor.goto_node(path);
                                }
                            }
                        }
                        KeyCode::Esc => app.search.cancel(),
                        KeyCode::Backspace => app.search.backspace(),
                        KeyCode::Char(c) => app.search.input(c),
                        _ => {}
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
                        }
                        (KeyModifiers::NONE, KeyCode::Down)
                        | (KeyModifiers::NONE, KeyCode::Char('j')) => {
                            app.browser_down();
                        }

                        // Select/Open
                        (KeyModifiers::NONE, KeyCode::Enter)
                        | (KeyModifiers::NONE, KeyCode::Char('l'))
                        | (KeyModifiers::NONE, KeyCode::Right) => {
                            app.browser_select();
                        }

                        // Go to parent directory
                        (KeyModifiers::NONE, KeyCode::Char('h'))
                        | (KeyModifiers::NONE, KeyCode::Left) => {
                            if let Some(parent) = app.current_dir.parent() {
                                app.current_dir = parent.to_path_buf();
                                app.load_directory();
                            }
                        }

                        _ => {}
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
                        app.tree_cursor.cursor_up();
                        app.preview_scroll = 0;
                    },
                    (KeyModifiers::NONE, KeyCode::Down)
                    | (KeyModifiers::NONE, KeyCode::Char('j')) => {
                        app.tree_cursor.cursor_down();
                        app.preview_scroll = 0;
                    },
                    (KeyModifiers::NONE, KeyCode::Left)
                    | (KeyModifiers::NONE, KeyCode::Char('h')) => {
                        app.tree_cursor.collapse_current();
                    },
                    (KeyModifiers::NONE, KeyCode::Right)
                    | (KeyModifiers::NONE, KeyCode::Char('l')) => {
                        app.tree_cursor.expand_current();
                    },

                    // Vim navigation
                    (KeyModifiers::NONE, KeyCode::Char('g')) => {
                        if pending_g {
                            app.tree_cursor.goto_first();
                            app.preview_scroll = 0;
                            pending_g = false;
                        } else {
                            pending_g = true;
                        }
                    },
                    (KeyModifiers::SHIFT, KeyCode::Char('G')) => {
                        app.tree_cursor.goto_last();
                        app.preview_scroll = 0;
                    },
                    (KeyModifiers::CONTROL, KeyCode::Char('f')) => {
                        for _ in 0..15 {
                            app.tree_cursor.cursor_down();
                        }
                        app.preview_scroll = 0;
                    },
                    (KeyModifiers::CONTROL, KeyCode::Char('b')) => {
                        for _ in 0..15 {
                            app.tree_cursor.cursor_up();
                        }
                        app.preview_scroll = 0;
                    },

                    // Search
                    (KeyModifiers::NONE, KeyCode::Char('/')) => {
                        app.search.start();
                    },
                    (KeyModifiers::NONE, KeyCode::Char('n')) => {
                        app.search.next_match();
                        if let Some(path) = app.search.current_match_path() {
                            app.tree_cursor.goto_node(path);
                        }
                    },
                    (KeyModifiers::SHIFT, KeyCode::Char('N')) => {
                        app.search.prev_match();
                        if let Some(path) = app.search.current_match_path() {
                            app.tree_cursor.goto_node(path);
                        }
                    },

                    // Features
                    (KeyModifiers::NONE, KeyCode::Char('p')) => {
                        app.toggle_data_viewer();
                    },
                    (KeyModifiers::NONE, KeyCode::Char('t')) => {
                        app.toggle_preview();
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
                            match util::copy_tree_structure(
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
                            match util::copy_node_info(node) {
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
