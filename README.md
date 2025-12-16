# Coriolis 🦀

A fast, terminal-based netCDF/HDF5 data viewer with vim-style navigation.

*Named after the Coriolis effect - fundamental in understanding Earth's atmospheric and oceanic circulation patterns.*

## Quick Start

### 1. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 2. Build

**Using Makefile (easy):**
```bash
cd coriolis
make static          # Build portable binary (recommended)
sudo make install-static  # Install to /usr/local/bin
```

**Or using cargo directly:**
```bash
cargo build --release
./target/release/coriolis your_file.nc
```

### 3. Run

```bash
coriolis your_file.nc
```

## Why "Coriolis"?

The Coriolis effect is crucial in atmospheric and oceanic sciences – the same fields that heavily use NetCDF and HDF5 formats for data storage. It fits for a tool that helps visualize scientific data!

## Building Options

### Static Binary (Recommended for Linux)

Creates a single portable executable with **zero dependencies**:

```bash
make static
```

This produces a ~15MB binary at `target/x86_64-unknown-linux-musl/release/coriolis` that runs on **any Linux system** - no libraries needed!

### Standard Build

```bash
make release        # Optimized build
make build          # Development build (faster, includes debug info)
```

## Makefile Targets

```bash
make static         # Build portable static binary
make release        # Build optimized binary
make test           # Run tests
make clippy         # Run linter
make fmt            # Format code
make clean          # Remove build artifacts
make install        # Install to /usr/local/bin
make uninstall      # Remove from /usr/local/bin
make run FILE=data.nc  # Run with file
make check          # Run all quality checks
make help           # Show all targets
```

## Keyboard Shortcuts

### Browser Navigation
| Key | Action |
|-----|--------|
| `↑/k` | Move up |
| `↓/j` | Move down |
| `→/l` | Expand node |
| `←/h` | Collapse node |
| `gg` | Go to top |
| `G` | Go to bottom |
| `/` | Search |
| `n` | Next match |
| `N` | Previous match |
| `t` | Toggle preview |
| `T` | Change theme |
| `p` | Open data viewer (on variable) |
| `c` | Copy tree |
| `y` | Copy node |
| `?` | Help |
| `q` | Quit |

### Data Viewer (when overlay is open)
| Key | Action |
|-----|--------|
| `Tab` | Cycle view mode (Table → 1D Plot → Heatmap) |
| `hjkl` / Arrows | Pan table / navigate |
| `Ctrl+u` | Page up |
| `Ctrl+d` | Page down |
| `[` / `]` | Select dimension (for 3D+ data) |
| `+` / `-` | Change slice index |
| `Esc` / `q` / `p` | Close viewer |

## Features

- 🚀 Fast NetCDF and HDF5 reading
- 🌲 Tree-based navigation
- 🔍 Powerful search
- ⌨️ Vim-style shortcuts
- 🎨 Gruvbox themes
- 📦 Single portable binary
- 💾 Low memory (~90MB)
- 🔒 Zero runtime dependencies (static build)
- 📊 Interactive data viewer with table, 1D plot, and heatmap views
- 🧊 Multi-dimensional data slicing for 3D+ arrays

## Architecture

Coriolis follows a clean separation between **state management** and **presentation**.

### Module Overview

```
src/
├── main.rs              # Entry point, event loop, keyboard handling
├── app.rs               # Application state and business logic
├── data/                # Data structures and NetCDF reading
│   ├── node.rs          # Tree node types (Root, Group, Variable, Dimension)
│   ├── reader.rs        # NetCDF file parsing
│   ├── variable_data.rs # Variable data loading and slicing
│   └── dataset.rs       # Dataset wrapper
├── navigation/          # Navigation state management
│   ├── tree.rs          # Tree cursor and visibility logic
│   └── search.rs        # Search functionality
├── ui/                  # Pure rendering functions
│   ├── browser.rs       # Main browser UI
│   ├── overlay.rs       # Data viewer overlay (table/plot/heatmap)
│   └── theme.rs         # Color schemes
└── util/                # Utilities (clipboard, etc.)
```

### App vs UI: The Separation

**`app.rs` (State & Logic):**
- Owns all application state (`App` struct)
- Handles business logic and state mutations
- Manages file loading, data reading, navigation state
- **Never renders anything** - just manages data

```rust
pub struct App {
    file_path: Option<PathBuf>,      // What file is open?
    dataset: Option<DatasetInfo>,    // Parsed file structure
    tree_cursor: TreeState,          // Where in the tree are we?
    search: SearchState,             // Search state
    overlay: OverlayState,           // Data viewer state
    theme: Theme,                    // Current theme
    // ... etc
}

impl App {
    pub fn toggle_preview(&mut self) { ... }  // Business logic
    pub fn load_file(&mut self, path: PathBuf) { ... }
    pub fn toggle_plot(&mut self) { ... }
}
```

**`ui/` (Presentation):**
- Pure rendering functions that take state and draw UI
- **Never modifies state** - just reads it
- Each module renders a specific part of the UI

```rust
// ui/browser.rs
pub fn draw_browser(f: &mut Frame, app: &App) {
    // Read app state, render UI
    let colors = ThemeColors::from_theme(&app.theme);
    draw_tree(f, app, area, &colors);
    draw_details(f, app, area, &colors);
    // ...
}

// ui/overlay.rs
pub fn draw_overlay(f: &mut Frame, state: &OverlayState, colors: &ThemeColors) {
    // Read overlay state, render data viewer
    match state.view_mode {
        ViewMode::Table => draw_table_view(...),
        ViewMode::Plot1D => draw_plot1d_view(...),
        ViewMode::Heatmap => draw_heatmap_view(...),
    }
}
```

### Data Flow

```
User Input (main.rs)
    │
    ├─→ Keyboard Event
    │       │
    │       ├─→ Modify App State (app.rs methods)
    │       │       │
    │       │       ├─→ Update TreeState (navigation/tree.rs)
    │       │       ├─→ Update SearchState (navigation/search.rs)
    │       │       └─→ Update OverlayState (ui/overlay.rs)
    │       │
    │       └─→ Trigger Redraw
    │
    └─→ Render Loop (60 FPS)
            │
            └─→ ui::draw(frame, &app)
                    │
                    ├─→ ui/browser.rs reads app state
                    └─→ ui/overlay.rs reads app.overlay state
```

### Key Design Principles

1. **Unidirectional Data Flow**: Input → State Update → Render
2. **Pure Rendering**: UI functions never mutate state
3. **State Encapsulation**: Each module owns its state (TreeState, SearchState, OverlayState)
4. **No Business Logic in UI**: UI code only knows how to draw, not what to do

### Example: Opening the Data Viewer

```rust
// 1. User presses 'p' (main.rs)
KeyCode::Char('p') => {
    app.toggle_plot();  // State mutation
}

// 2. App modifies state (app.rs)
pub fn toggle_plot(&mut self) {
    let node = self.current_node()?;
    let data = read_variable(&self.file_path, &node.path)?;
    self.overlay.load_variable(data);  // Update overlay state
}

// 3. Next render cycle (ui/overlay.rs)
pub fn draw_overlay(f: &mut Frame, state: &OverlayState, ...) {
    if !state.visible { return; }
    // Read state.variable and render table/plot/heatmap
}
```

### State Structures

**TreeState** (navigation/tree.rs):
- Flat list of visible tree items
- Cursor position (index into visible items)
- Set of expanded node paths
- Rebuilds on expand/collapse for correctness

**SearchState** (navigation/search.rs):
- Search buffer and submitted query
- List of match paths
- Current match index

**OverlayState** (ui/overlay.rs):
- Loaded variable data (LoadedVariable)
- View mode (Table/Plot1D/Heatmap)
- Scroll position
- Dimension slice indices (for 3D+ data)

## Musl vs Glibc

When building with `make static`, we use **musl libc** instead of **glibc**:

### Why Musl?

| Feature | glibc | musl |
|---------|-------|------|
| **Portability** | Requires specific glibc version | Fully static, runs anywhere |
| **Binary size** | Larger | Smaller |
| **Dependencies** | Many shared libraries | Zero (fully static) |
| **Startup time** | Slower | Faster |
| **Memory usage** | Higher | Lower |
| **Security** | Complex codebase | Minimal, auditable |

### Practical Benefits

**With glibc (standard build):**
```bash
$ ldd target/release/coriolis
    linux-vdso.so.1
    libnetcdf.so.19 => /usr/lib/libnetcdf.so.19
    libhdf5.so.103 => /usr/lib/libhdf5.so.103
    libc.so.6 => /lib/x86_64-linux-gnu/libc.so.6
    ... (15+ more libraries)
```

**With musl (static build):**
```bash
$ ldd target/x86_64-unknown-linux-musl/release/coriolis
    not a dynamic executable
```

✨ **Single file, runs everywhere!**

### When to Use Each

**Use musl (`make static`):**
- ✅ Deploying to multiple systems
- ✅ Don't want to deal with dependencies
- ✅ Creating portable binaries
- ✅ Container/Docker deployments
- ✅ Older Linux systems

**Use glibc (`make release`):**
- ✅ Quick local development
- ✅ Your system already has dependencies
- ✅ Faster compilation (5 min vs 15 min)

## Supported Formats

- NetCDF (.nc, .nc4, .netcdf)
- HDF5 (.h5, .hdf5, .he5)

## Command Line Options

```bash
coriolis data.nc                    # Basic usage
coriolis --log-level debug data.nc  # With debug logging
coriolis --help                     # Show help
```

## Installation

### From Source (Recommended)

```bash
# Build static binary
make static

# Install (requires sudo)
sudo make install-static

# Now run from anywhere
coriolis data.nc
```

### Manual Installation

```bash
# Build
cargo build --release --target x86_64-unknown-linux-musl

# Copy to PATH
sudo cp target/x86_64-unknown-linux-musl/release/coriolis /usr/local/bin/

# Done!
coriolis data.nc
```

## Troubleshooting

**"musl-tools not found"**
```bash
sudo apt-get install musl-tools
```

**"cargo: command not found"**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Build is slow**
- First build takes 10-15 minutes (compiling C libraries)
- Subsequent builds are much faster (~30 seconds)
- Use `make build` for quick dev builds

## Development

```bash
make build          # Quick development build
make test           # Run tests
make clippy         # Lint code
make fmt            # Format code
make check          # Run all checks
make doc            # Generate documentation
```

## License

MIT
