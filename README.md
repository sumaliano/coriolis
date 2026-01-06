# Coriolis

A fast, terminal-based NetCDF data explorer and viewer with vim-style navigation.

*Named after the Coriolis effect — fundamental in understanding Earth's atmospheric and oceanic circulation.*

![Coriolis Demo](https://via.placeholder.com/800x400?text=Coriolis+TUI+Screenshot)

## Features

- **Fast NetCDF reading** — supports both classic and NetCDF-4/HDF5-backed files
- **Tree-based navigation** — browse groups, variables, dimensions, and attributes
- **Interactive data viewer** — table view, 1D plots, and heatmap visualizations
- **Multi-dimensional slicing** — navigate through 3D+ arrays with intuitive controls
- **Vim-style shortcuts** — feel at home with familiar keybindings
- **Dual themes** — Gruvbox light and dark themes
- **Portable binary** — single static binary on Linux (no runtime dependencies)
- **Low memory footprint** — efficient handling of large datasets
- **Clipboard support** — copy data and tree structures

## Quick Start

### Installation

#### From Source (Recommended)

```bash
# Clone the repository
git clone https://github.com/jsilva/coriolis.git
cd coriolis

# Build and install
cargo build --release
sudo cp target/release/coriolis /usr/local/bin/
```

#### Using Cargo

```bash
cargo install coriolis
```

### Usage

```bash
# Open a NetCDF file
coriolis path/to/data.nc

# Open a directory (file browser mode)
coriolis path/to/directory/

# Enable debug logging
coriolis data.nc --log debug.log
```

## Keyboard Shortcuts

### Browser Navigation

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `l` / `→` | Expand node |
| `h` / `←` | Collapse node |
| `gg` | Go to top |
| `G` | Go to bottom |
| `Ctrl+f` | Page down |
| `Ctrl+b` | Page up |
| `/` | Search |
| `n` / `N` | Next / Previous match |
| `p` | Open data viewer |
| `t` | Toggle preview panel |
| `T` | Cycle theme |
| `c` | Copy tree structure |
| `y` | Copy current node |
| `f` | Open file browser |
| `q` | Quit |

### Data Viewer

| Key | Action |
|-----|--------|
| `Tab` | Cycle view mode (Table → Plot → Heatmap) |
| `h/j/k/l` | Navigate / Pan |
| `Ctrl+u/d` | Page up / down |
| `s` | Select slice dimension |
| `+/-` or `PgUp/PgDn` | Change slice index |
| `y/x` | Cycle Y/X display dimension |
| `r` | Rotate (swap) Y/X dimensions |
| `o` | Toggle scale/offset (raw vs scaled data) |
| `c` | Cycle color palette (heatmap) |
| `Ctrl+c` | Copy visible data to clipboard |
| `Esc` / `q` | Close viewer |

## Building

### Requirements

- Rust 1.70 or newer
- For static Linux builds: `musl-tools` package

### Build Commands

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Static Linux binary (portable, no dependencies)
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl

# Run tests
cargo test

# Run lints
cargo clippy
```

### Using the Makefile

```bash
make build          # Development build
make release        # Optimized build
make static         # Static Linux binary
make test           # Run tests
make clippy         # Lint checks
make fmt            # Format code
make install        # Install to /usr/local/bin
make help           # Show all targets
```

## Project Structure

```
src/
├── main.rs              # Entry point and event loop
├── lib.rs               # Library exports
├── app.rs               # Application state
├── error.rs             # Error types
├── data/                # NetCDF data handling
│   ├── dataset.rs       # Dataset wrapper
│   ├── node.rs          # Tree node types
│   ├── reader.rs        # File reading
│   └── variable_data.rs # Variable loading and slicing
├── navigation/          # Navigation logic
│   ├── tree.rs          # Tree cursor and state
│   └── search.rs        # Search functionality
├── overlay/             # Data viewer overlay
│   ├── mod.rs           # Overlay state
│   └── ui.rs            # Overlay rendering
├── ui/                  # UI components
│   ├── mod.rs           # UI entry point
│   ├── browser.rs       # Main browser view
│   └── theme.rs         # Color themes
└── util/                # Utilities
    ├── clipboard.rs     # Clipboard support
    ├── colormaps.rs     # Heatmap color palettes
    └── ...
```

## Supported Platforms

| Platform | Status | Notes |
|----------|--------|-------|
| Linux | Fully supported | Static binary available |
| macOS | Supported | Standard Cargo build |
| Windows | Experimental | Requires proper terminal emulator |

## Configuration

Coriolis currently requires no configuration files. All settings are controlled via command-line arguments and runtime keyboard shortcuts.

### Command-Line Options

```
USAGE:
    coriolis [OPTIONS] [FILE_OR_DIR]

ARGS:
    <FILE_OR_DIR>    Path to NetCDF file or directory

OPTIONS:
    --log <PATH>     Enable logging to file
    -h, --help       Print help information
    -V, --version    Print version information
```

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

```bash
# Clone and build
git clone https://github.com/jsilva/coriolis.git
cd coriolis
cargo build

# Run with a test file
cargo run -- path/to/test.nc

# Run tests
cargo test

# Check formatting and lints
cargo fmt --check
cargo clippy
```

## License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Ratatui](https://github.com/ratatui-org/ratatui) for the terminal UI
- Uses the [netcdf](https://crates.io/crates/netcdf) crate for data access
- Color schemes inspired by [Gruvbox](https://github.com/morhetz/gruvbox)

## Why "Coriolis"?

The Coriolis effect is a fundamental concept in atmospheric and oceanic sciences — fields that heavily rely on NetCDF for data storage and exchange. This tool aims to make exploring that scientific data as intuitive as the physical phenomena it represents ;).
