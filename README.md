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

The Coriolis effect is crucial in atmospheric and oceanic sciences - the same fields that heavily use NetCDF and HDF5 formats for data storage. It's fitting for a tool that helps visualize scientific data!

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
| `t` | Toggle preview |
| `T` | Change theme |
| `c` | Copy tree |
| `y` | Copy node |
| `?` | Help |
| `q` | Quit |

## Features

- 🚀 Fast NetCDF and HDF5 reading
- 🌲 Tree-based navigation
- 🔍 Powerful search
- ⌨️ Vim-style shortcuts
- 🎨 Gruvbox themes
- 📦 Single portable binary
- 💾 Low memory (~90MB)
- 🔒 Zero runtime dependencies (static build)

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
