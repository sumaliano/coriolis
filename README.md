# Tanotly

A terminal-based netCDF/HDF5 data viewer and explorer - like Panoply, but for the command line.

## Features

- 🌲 **Tree Navigation**: Browse hierarchical data structures intuitively
- 🔍 **Powerful Search**: Find variables, attributes, and values across entire tree
- 📊 **Data Visualization**: Terminal plots for 1D and 2D data
- ⌨️ **Keyboard-Driven**: Efficient navigation with Vim-style shortcuts
- 🎨 **Clean TUI**: Beautiful terminal interface powered by Textual
- 📁 **Multi-Format**: Supports netCDF, HDF5, and related formats
- 🏷️ **Expandable Attributes**: Browse attributes directly in tree view

## Installation

### From Source

```bash
# Clone the repository (or if you're already in the tanotly directory)
cd tanotly

# Install in development mode
pip install -e .

# Or install dependencies directly
pip install textual textual-plotext xarray netcdf4 h5py numpy rich
```

### Requirements

- Python 3.9+
- Dependencies (automatically installed):
  - textual >= 0.47.0
  - textual-plotext >= 1.0.0
  - xarray >= 2023.1.0
  - netcdf4 >= 1.6.0
  - h5py >= 3.8.0
  - numpy >= 1.24.0
  - rich >= 13.0.0

## Quick Start

```bash
# Run Tanotly with your NetCDF file
python -m tanotly your_file.nc

# Or if installed
tanotly your_file.nc
```

## Keyboard Shortcuts

| Key | Action | Description |
|-----|--------|-------------|
| **Navigation** |||
| `↑` `↓` | Navigate | Move up/down in tree |
| `j` `k` | Navigate (Vim) | Move up/down in tree |
| `←` `→` | Expand/Collapse | Toggle tree nodes |
| `h` `l` | Expand/Collapse (Vim) | Toggle tree nodes |
| **Search** |||
| `/` | Start Search | Open search bar at bottom |
| `Enter` | Execute Search | Find all matches in tree |
| `n` | Next Match | Jump to next search result |
| `N` | Previous Match | Jump to previous search result |
| `Esc` | Cancel | Exit search mode |
| **Visualization** |||
| `p` | Toggle Plot | Show/hide data visualization |
| `t` | Toggle Preview | Show/hide preview pane |
| **Actions** |||
| `c` | Copy Tree | Copy entire tree structure |
| `y` | Copy Info | Copy current node information |
| `q` | Quit | Exit application |

## Interface Overview

```
┌────────────────────────────────────────────────────────┐
│ Tanotly                                   [Clock]      │
├────────────────────────────────────────────────────────┤
│ File info / Status                                     │
├──────────────────────────┬─────────────────────────────┤
│                          │                             │
│  🏠 data.nc              │  🏠 data.nc                 │
│   🏷️ Attributes (3)      │                             │
│     Conventions: CF...   │  Type: root                 │
│     history: created...  │  Path: /                    │
│   📂 /data (10)          │                             │
│     🌡️ temperature      │  🏷️ Attributes:             │
│        [1D float32]      │    Conventions: CF-1.6      │
│     🌡️ pressure         │    history: created 2024    │
│        [2D float64]      │    source: satellite        │
│     🏷️ Attributes (2)    │                             │
│   📂 /dims (3)           │  📂 Groups: 2               │
│     📏 time (10)         │  🌡️ Variables: 2           │
│     📏 lat (180)         │                             │
│                          │                             │
├──────────────────────────┴─────────────────────────────┤
│ q: Quit  /: Search  t: Toggle Preview  p: Plot         │
│ Type to search, Enter to find matches, Esc to cancel   │
└────────────────────────────────────────────────────────┘
```

## Tree View Icons

- 🏠 **Root** - The file itself (with global attributes)
- 📂 **Groups** - Organizational containers
- 🌡️ **Variables** - Data arrays with shape and dtype
- 📏 **Dimensions** - Dimension definitions
- 🏷️ **Attributes** - Expandable metadata groups

## Search Functionality

Press `/` to start searching:

1. **Type query** - Search as you type
2. **Press Enter** - Expands entire tree and finds all matches
3. **Navigate** - Use `n`/`N` to jump between matches
4. **Exit** - Press `Esc` to clear search

**Search includes:**
- Node names (variables, groups, dimensions)
- Attribute keys and values
- Node paths
- Metadata fields

**Features:**
- Case-insensitive matching
- Automatic tree expansion
- Match counter in search bar
- Highlights current match

## Data Visualization

Press `p` to toggle visualization for variables:

### 1D Data - Line Plots
- Professional plotext line charts
- Auto-samples to 500 points for performance
- Shows full data range (first to last)
- Displays statistics: min, max, mean, std

### 2D Data - Heatmaps
- Color-coded heatmap visualization
- Intelligent downsampling with block averaging
- Preserves features across entire array
- Maximum 50×100 display grid

## Detail Panel

The right panel shows comprehensive information:

- **Header**: Icon, name, and type
- **Metadata**: Shape, dimensions, dtype, size
- **Attributes**: All metadata with escaped values
- **Data Preview** (for variables):
  - Statistics: min, max, mean, std, NaN count
  - Sample values from top and bottom of array
  - Optional visualization (press `p`)

## Copy Functions

### Copy Tree (`c`)
Copies the entire file structure as plain text:
```
Tree Structure: /path/to/file.nc
================================================================================

├── 🏠 data.nc
│   ├── 🏷️ Attributes (3)
│   │   ├── Conventions: CF-1.6
│   │   └── history: created 2024
│   └── 📂 /data (10)
│       └── 🌡️ temperature (100×180×360) 3D float32
```

### Copy Info (`y`)
Copies current node details with all metadata and attributes.

## Supported File Formats

- ✅ NetCDF (.nc, .nc4, .netcdf)
- ✅ HDF5 (.h5, .hdf5, .he5)
- ✅ NetCDF4 with groups
- ✅ Complex nested structures

## Data Type Classification

Tanotly automatically classifies variables:

| Type | Description | Example |
|------|-------------|---------|
| **Scalar** | Single value | `()` |
| **1D** | One dimension | `(100,)` |
| **2D** | Two dimensions | `(80, 95)` |
| **Geo2D** | Geographic 2D | lat/lon dimensions |
| **3D** | Three dimensions | `(10, 80, 95)` |
| **Geo3D** | Geographic 3D | time/lat/lon |
| **4D+** | Four or more | `(5, 10, 80, 95)` |

## Tips & Tricks

### Navigation
- **Fast browsing**: Use `hjkl` (Vim keys) for speed
- **Full width tree**: Press `t` to hide preview pane
- **Scroll preview**: Preview pane is fully scrollable

### Search
- **Find anything**: Search looks in names, paths, attributes, and metadata
- **See all matches**: Search expands entire tree automatically
- **Quick navigation**: Use `n`/`N` to jump through results

### Visualization
- **Compare data**: Toggle between stats and plots with `p`
- **Large files**: Plots auto-sample intelligently
- **Full range**: Downsampling covers entire array, not just corners

### Performance
- **Lazy loading**: Data loads on-demand, not upfront
- **Smart sampling**: Visualizations preserve features while being fast
- **Debounced navigation**: Arrow keys respond smoothly without lag

## Workflow Examples

### Explore File Structure
```
1. Open: tanotly data.nc
2. Browse: Use ↑↓ arrows
3. Expand: Use → on groups
4. Attributes: Expand 🏷️ Attributes nodes
```

### Find Specific Variable
```
1. Search: Press /
2. Type: "temperature"
3. Enter: Find all matches
4. Navigate: Use n/N
5. Visualize: Press p
```

### Export Documentation
```
1. Navigate to section of interest
2. Press c for full tree
3. Or press y for current node
4. Paste into documentation
```

## Recent Updates (2025-12-05)

### Major Improvements

1. **Enhanced Search**
   - Uses bottom bar for search input
   - Press Enter to initiate search
   - Expands entire tree automatically
   - Shows match count and navigation in search bar

2. **Root & Attributes Visible**
   - Root node now shown in tree
   - Global attributes accessible immediately
   - Attributes expandable in tree view

3. **Better Visualization**
   - Uses textual-plotext for professional plots
   - Full range downsampling (not just corners)
   - Proper heatmap colors for 2D data

4. **UI Enhancements**
   - Scrollable preview pane
   - Better contrast in top bar
   - Toggle preview with `t` key
   - Text wrapping in tree labels
   - 50ms debouncing for smooth navigation

### Bug Fixes
- Fixed PlotextPlot initialization error
- Fixed markup escaping errors
- Removed invalid 'fast' argument from heatmap
- Fixed arrow navigation lag

## Troubleshooting

### App doesn't start
```bash
# Check Python version (3.9+)
python --version

# Install/upgrade dependencies
pip install --upgrade textual textual-plotext xarray netcdf4 h5py numpy
```

### Visualization errors
- Make sure `textual-plotext` is installed
- Only numeric data can be plotted
- Press `p` to toggle visualization on/off

### Copy doesn't work
Requires clipboard tool:
- **Linux**: `xclip` (`sudo apt install xclip`)
- **macOS**: `pbcopy` (built-in)
- **Windows**: `clip` (built-in)
- Falls back to file save if unavailable

### Search not finding results
- Ensure you press Enter after typing
- Search is case-insensitive
- Searches entire tree including attributes

## Development

### Project Structure

```
tanotly/
├── pyproject.toml          # Project configuration
├── README.md               # This file
└── src/
    └── tanotly/
        ├── __init__.py
        ├── __main__.py     # Entry point
        ├── app.py          # Main Textual application
        ├── visualization.py # Plotting widgets
        └── data/           # Data reading and models
            ├── models.py   # Data structure models
            └── reader.py   # File readers
```

### Running Tests

```bash
# Install dev dependencies
pip install -e ".[dev]"

# Run tests (when available)
pytest

# Code formatting
black src/

# Linting
ruff check src/
```

## Contributing

Contributions welcome! Enhancement ideas:

- GRIB file support (cfgrib integration)
- 3D visualization support
- Data export functionality
- Advanced search filters (regex, wildcards)
- Bookmarking favorite nodes
- Configuration file support
- Custom color themes

## License

MIT License - See LICENSE file for details

## Acknowledgments

- Inspired by [Panoply](https://www.giss.nasa.gov/tools/panoply/) from NASA GISS
- Built with [Textual](https://github.com/Textualize/textual)
- Plotting powered by [textual-plotext](https://github.com/Textualize/textual-plotext)
- Data reading powered by [xarray](https://xarray.dev/) and [h5py](https://www.h5py.org/)

---

**Enjoy exploring your scientific data files!** 🚀
