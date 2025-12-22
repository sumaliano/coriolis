# Coriolis 🦀

A fast, terminal-based NetCDF data explorer and viewer with vim-style navigation.

(Named after the Coriolis effect — fundamental in understanding Earth's atmospheric and oceanic circulation.)

Note: This application reads NetCDF files (classic and NetCDF‑4/HDF5‑backed). It does not provide generic HDF5 browsing beyond the NetCDF common data model.

## Overview

Coriolis lets you quickly explore NetCDF datasets from the terminal:
- Browse dataset structure (groups, dimensions, variables, attributes)
- View variable metadata and previews
- Open an interactive data viewer overlay with multiple modes: table, 1D plot, and heatmap
- Slice multi-dimensional arrays and navigate with familiar vim keys

## Stack

- Language: Rust (edition 2021, Rust ≥ 1.70)
- TUI framework: `ratatui` + `crossterm`
- Data access: `netcdf` crate (with optional static linking)
- Arrays: `ndarray`
- CLI: `clap`
- Logging: `tracing` + `tracing-subscriber`

Package manager and build tool: Cargo

Binary entry point: `src/main.rs` (bin name `coriolis`)

## Requirements

Base (all platforms):
- Rust toolchain (rustup) — Rust 1.70 or newer

For Linux static build (recommended for portability):
- musl tools: `musl-gcc` (package `musl-tools` on Debian/Ubuntu)
- Rust MUSL target: `x86_64-unknown-linux-musl`

Optional tooling:
- `cargo-watch` for `make watch`

## Installation and Setup

Install Rust using rustup:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

Clone and build:

```bash
git clone <this-repo>
cd coriolis
make release    # or: cargo build --release
```

Install system-wide (optional, requires sudo):

```bash
sudo make install           # install target/release/coriolis
sudo make install-static    # after `make static`, installs the static binary
```

## Build and Run

Using Makefile (recommended shortcuts):

```bash
make build          # dev build
make release        # optimized build
make static         # fully static Linux binary (portable)
```

Using Cargo directly:

```bash
cargo build --release
./target/release/coriolis path/to/file.nc
```

Run with a file or directory (directory opens the file browser):

```bash
coriolis path/to/file_or_directory
```

CLI options:

```text
USAGE: coriolis [FILE_OR_DIR] [--log PATH]

ARGS:
  FILE_OR_DIR         Optional path to a NetCDF file or directory to start in

OPTIONS:
  --log <PATH>        Enable logging to the given file
```

## Makefile Targets

```bash
make build            # Development build (fast)
make release          # Production build (optimized)
make static           # Build static Linux binary via MUSL
make test             # Run tests
make test-verbose     # Run tests with output
make fmt              # Format code
make fmt-check        # Check formatting
make clippy           # Lints with warnings as errors
make doc              # Build docs
make clean            # Remove build artifacts
make install          # Install release binary to /usr/local/bin
make install-static   # Install static binary to /usr/local/bin
make uninstall        # Remove installed binary
make run FILE=data.nc # Run with a file
make run-dev FILE=... # Run dev build with a file
make check            # fmt-check + clippy + test + build
make watch            # Rebuild on changes (needs cargo-watch)
make help             # Show this summary
```

Helper script:
- `build.sh`: cross‑platform convenience script for producing portable binaries. On Linux it builds a fully static MUSL binary; on macOS it builds a standard release.

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

- 🚀 Fast NetCDF reading (classic + NetCDF‑4/HDF5‑backed)
- 🌲 Tree-based navigation of groups, variables, dimensions, attributes
- 🔍 Search within the tree
- ⌨️ Vim-style shortcuts
- 🎨 Gruvbox light/dark themes
- 📦 Single portable static binary on Linux (no runtime deps)
- 💾 Low memory usage
- 📊 Interactive data viewer: table, 1D plot, heatmap
- 🧊 Multi-dimensional slicing for 3D+ arrays

## Project Structure

```
src/
├── main.rs            # Entry point, terminal event loop & key handling
├── lib.rs             # Module exports
├── app.rs             # Application state & business logic
├── data/              # NetCDF reading and dataset representation
│   ├── dataset.rs     # Dataset metadata wrapper
│   ├── node.rs        # Tree node types (root/group/var/dim/attr)
│   ├── reader.rs      # File reading utilities
│   └── variable_data.rs # Variable data loading & slicing
├── navigation/        # Navigation and search state
│   ├── tree.rs        # Tree cursor & visibility
│   └── search.rs      # Search logic
├── overlay/           # Data viewer overlay (state + UI helpers)
│   └── ui.rs          # Overlay rendering primitives
├── ui/                # Common UI components & theming
│   ├── browser.rs     # Main browser view
│   └── theme.rs       # Themes and colors
└── util/              # Utilities (clipboard, colormaps, layout, etc.)
```

## Environment Variables

No required environment variables.

Optional:
- `PKG_CONFIG_ALL_STATIC=1` — used by static builds to prefer static libraries (already set in Makefile/build.sh for MUSL builds).

## Tests

Run all tests:

```bash
cargo test
# or
make test
```

Show test output:

```bash
make test-verbose
```

## Development

- Lint: `make clippy`
- Format: `make fmt` / `make fmt-check`
- Docs: `make doc`
- Auto-rebuild on changes: `make watch` (requires `cargo install cargo-watch`)

## Supported Platforms

- Linux: first-class (including fully static portable binary via MUSL)
- macOS: standard Cargo builds work
- Windows: standard Cargo builds should work in a proper terminal; for static cross‑compile, use the guidance in `build.sh` (requires mingw toolchain)

## License

Licensed under the MIT License (see `Cargo.toml`).

TODO: Add a `LICENSE` file to the repository root if missing.

## Why "Coriolis"?

The Coriolis effect is crucial in atmospheric and oceanic sciences — fields that commonly use NetCDF for data storage — fitting for a tool that helps visualize scientific data.

## Notes and TODOs

- TODO: Document any additional CLI flags if introduced in the future.
- TODO: Attach prebuilt release binaries (Linux/macOS/Windows) to GitHub Releases.
- TODO: Expand documentation for very large datasets and performance tips.
