# Tanotly

A terminal-based netCDF/HDF5 data viewer and explorer - like Panoply, but for the command line.

## Features

- 🌲 **Tree Navigation**: Browse hierarchical data structures intuitively
- 🔍 **Powerful Search**: Case-insensitive substring search across nodes and attributes
- 📊 **Data Inspection**: View detailed metadata, attributes, and dimensions
- ⌨️ **Keyboard-Driven**: Efficient navigation with keyboard shortcuts
- 🎨 **Clean TUI**: Beautiful terminal interface powered by Textual
- 📁 **Multi-Format**: Supports netCDF, HDF5, and related formats

## Installation

### From Source

```bash
# Clone the repository (or if you're already in the tanotly directory)
cd tanotly

# Install in development mode
pip install -e .

# Or install dependencies directly
pip install textual xarray netcdf4 h5py numpy rich
```

### Requirements

- Python 3.9+
- Dependencies (automatically installed):
  - textual >= 0.47.0
  - xarray >= 2023.1.0
  - netcdf4 >= 1.6.0
  - h5py >= 3.8.0
  - numpy >= 1.24.0
  - rich >= 13.0.0

## Usage

### Open a file directly

```bash
tanotly /path/to/your/data.nc
```

Or with python -m:

```bash
python -m tanotly /path/to/your/data.nc
```

### Open the application and load a file interactively

```bash
tanotly
# Then press 'o' to open a file
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `/` or `Ctrl+F` | Focus search bar (live filtering) |
| `Esc` | Clear search |
| `↑/↓` | Navigate tree |
| `←/→` | Collapse/expand tree nodes |
| `q` or `Ctrl+C` | Quit application |

## Interface Overview

```
┌────────────────────────────────────────────────────────┐
│ Header                                    [Clock]      │
├────────────────────────────────────────────────────────┤
│ File info / Status                                     │
├────────────────────────────────────────────────────────┤
│ Type to search (case-insensitive)...                  │
├──────────────────────────┬─────────────────────────────┤
│                          │                             │
│  [Variables] (3)         │  ┌─ temperature ─────────┐ │
│    temperature [...]     │  │                        │ │
│    precipitation [...]   │  │ Type: variable         │ │
│    pressure [...]        │  │ Shape: 10 × 180 × 360  │ │
│                          │  │                        │ │
│  [Dimensions] (3)        │  │ Attributes:            │ │
│    time (10)             │  │   units: celsius       │ │
│    lat (180)             │  │                        │ │
│    lon (360)             │  │ Data Preview:          │ │
│                          │  │   Min: -12.3456        │ │
│  [Coordinates] (3)       │  │   Max: 45.6789         │ │
│    time [10]             │  │   Mean: 15.2341        │ │
│    lat [180]             │  │   Sample: [...]        │ │
│    lon [360]             │  └────────────────────────┘ │
│                          │                             │
├──────────────────────────┴─────────────────────────────┤
│ q: Quit  /: Search  ↑↓: Navigate                      │
└────────────────────────────────────────────────────────┘
```

## Features in Detail

### Tree Navigation

The left panel displays your data in a hierarchical tree structure:

- 📁 **Groups**: Organizational containers
- 📊 **Variables**: Data arrays with shape and dtype information
- 📏 **Dimensions**: Dimension definitions
- 🏷️ **Attributes**: Metadata attributes

Use arrow keys to navigate, and the detail panel updates automatically.

### Search Functionality

Press `/` to activate the search bar. Search features:

- **Live filtering**: Results appear as you type
- **Case-insensitive**: `temp` matches `Temperature`, `TEMP`, etc.
- **Substring matching**: `lat` matches `latitude`, `lat_bnds`, etc.
- **Multi-field search**: Searches node names, attribute names, and attribute values
- **Clear display**: Filtered results show full paths for easy identification

### Detail View

The right panel shows comprehensive information about the selected node:

- Name and type
- Full path in the data hierarchy
- Metadata (dtype, shape, dimensions, size)
- All attributes with values
- **Data preview** (for variables):
  - Statistics: min, max, mean, std
  - Sample values from the actual dataset
  - Formatted display for arrays of any dimension
- Child count

## Supported File Formats

Tanotly supports various scientific data formats:

- **NetCDF**: `.nc`, `.nc4`, `.netcdf`
- **HDF5**: `.hdf5`, `.h5`, `.he5`

The tool automatically detects the format and uses the appropriate reader.

## Example Workflow

1. **Open a file**: `tanotly climate_data.nc`
2. **Browse structure**: Use arrow keys to explore dimensions and variables
3. **View data**: Select a variable to see statistics and actual data values
4. **Search**: Press `/` and start typing `temp` - results filter in real-time
5. **Explore matches**: Use arrow keys to navigate through filtered results
6. **Clear search**: Press `Esc` to return to full tree view

## Python Best Practices

Tanotly follows Python best practices:

- ✅ Type hints throughout
- ✅ Modular architecture with clear separation of concerns
- ✅ Comprehensive docstrings
- ✅ Clean code structure
- ✅ Configuration via pyproject.toml
- ✅ Follows PEP 8 style guidelines

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
        ├── data/           # Data reading and models
        │   ├── models.py   # Data structure models
        │   └── reader.py   # File readers
        ├── ui/             # UI components
        │   ├── data_tree.py
        │   └── detail_view.py
        └── utils/          # Utilities
            └── search.py   # Search functionality
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

Contributions are welcome! Areas for enhancement:

- GRIB file support (cfgrib integration)
- Data visualization (plots in terminal)
- Data export functionality
- More advanced search filters
- Bookmarking favorite nodes
- Configuration file support

## License

MIT License - See LICENSE file for details

## Acknowledgments

- Inspired by [Panoply](https://www.giss.nasa.gov/tools/panoply/) from NASA GISS
- Built with [Textual](https://github.com/Textualize/textual)
- Data reading powered by [xarray](https://xarray.dev/) and [h5py](https://www.h5py.org/)
