"""Detail panel rendering for Tanotly.

Renders node information in a Panoply/ncdump-style format with:
- Header with node name, type, and path
- Array info for variables (dimensions, shape, dtype)
- Group contents summary (dimensions, variables, subgroups)
- Attributes section
- Data preview with statistics and sample values
"""

import numpy as np
import xarray as xr

from textual.containers import VerticalScroll
from textual.widgets import Static

from .config import NODE_ICONS, ThemeColors
from .data.models import DataNode, DatasetInfo, NodeType
from .visualization import format_statistics, format_sample_values


def render_details(
    container: VerticalScroll,
    node: DataNode,
    dataset: DatasetInfo | None,
) -> None:
    """Render node details into the container."""
    container.remove_children()

    # Render header
    _render_header(container, node)

    # Render type-specific content
    if node.node_type == NodeType.VARIABLE and node.metadata:
        _render_variable_info(container, node)
    elif node.node_type in (NodeType.GROUP, NodeType.ROOT):
        _render_group_info(container, node)

    # Render attributes (for all node types)
    if node.attributes:
        _render_attributes(container, node)

    # Render data preview for variables
    if node.node_type == NodeType.VARIABLE and dataset:
        _render_data_preview(container, node, dataset)


def _get_node_color(node_type: NodeType) -> str:
    """Get the theme color for a node type."""
    colors = {
        NodeType.ROOT: ThemeColors.root(),
        NodeType.GROUP: ThemeColors.group(),
        NodeType.VARIABLE: ThemeColors.variable(),
        NodeType.DIMENSION: ThemeColors.dimension(),
        NodeType.ATTRIBUTE: ThemeColors.root(),
    }
    return colors.get(node_type, ThemeColors.foreground())


def _render_header(container: VerticalScroll, node: DataNode) -> None:
    """Render the node header with name, type, and path."""
    icon = NODE_ICONS.get(node.node_type, "●")
    color = _get_node_color(node.node_type)
    
    # Build header
    lines = [
        f"{icon} [bold {color}]{node.name}[/bold {color}]",
        "─" * 50,
        f"[{ThemeColors.muted()}]Type:[/{ThemeColors.muted()}] [{color}]{node.node_type.value}[/{color}]",
        f"[{ThemeColors.muted()}]Path:[/{ThemeColors.muted()}] [{ThemeColors.info()}]{node.path}[/{ThemeColors.info()}]",
    ]
    
    container.mount(Static("\n".join(lines)))


def _render_variable_info(container: VerticalScroll, node: DataNode) -> None:
    """Render variable metadata (dimensions, shape, dtype, size)."""
    meta = node.metadata
    shape = meta.get("shape", ())
    dims = meta.get("dims", ())
    dtype = meta.get("dtype", "")
    size = meta.get("size", 0)

    lines = [
        "",
        f"[bold {ThemeColors.group()}]Array Info[/bold {ThemeColors.group()}]",
    ]

    # Dimensions with sizes
    if dims and shape and len(dims) == len(shape):
        dim_parts = [
            f"[{ThemeColors.dimension()}]{d}[/{ThemeColors.dimension()}]={s}"
            for d, s in zip(dims, shape)
        ]
        lines.append(f"  [{ThemeColors.muted()}]Dimensions:[/{ThemeColors.muted()}] {', '.join(dim_parts)}")
    elif shape:
        shape_str = " × ".join(str(s) for s in shape)
        lines.append(f"  [{ThemeColors.muted()}]Shape:[/{ThemeColors.muted()}] {shape_str}")

    # Data type
    if dtype:
        lines.append(f"  [{ThemeColors.muted()}]Type:[/{ThemeColors.muted()}] {dtype}")

    # Size
    if isinstance(size, int) and size > 0:
        lines.append(f"  [{ThemeColors.muted()}]Size:[/{ThemeColors.muted()}] {size:,} elements")

    container.mount(Static("\n".join(lines)))


def _render_group_info(container: VerticalScroll, node: DataNode) -> None:
    """Render group contents summary."""
    # Count children by type
    dims = [c for c in node.children if c.node_type == NodeType.DIMENSION]
    vars = [c for c in node.children if c.node_type == NodeType.VARIABLE]
    groups = [c for c in node.children if c.node_type == NodeType.GROUP]

    # Summary counts
    lines = [
        "",
        f"[bold {ThemeColors.group()}]Contents[/bold {ThemeColors.group()}]",
    ]
    
    if groups:
        lines.append(f"  [{ThemeColors.muted()}]Groups:[/{ThemeColors.muted()}] {len(groups)}")
    if dims:
        lines.append(f"  [{ThemeColors.muted()}]Dimensions:[/{ThemeColors.muted()}] {len(dims)}")
    if vars:
        lines.append(f"  [{ThemeColors.muted()}]Variables:[/{ThemeColors.muted()}] {len(vars)}")
    
    container.mount(Static("\n".join(lines)))

    # Dimensions section (ncdump style)
    if dims:
        _render_dimensions_section(container, dims)

    # Variables section (ncdump style)
    if vars:
        _render_variables_section(container, vars)

    # Subgroups section
    if groups:
        _render_groups_section(container, groups)


def _render_dimensions_section(container: VerticalScroll, dims: list[DataNode]) -> None:
    """Render dimensions in ncdump style."""
    lines = [
        "",
        f"[bold {ThemeColors.dimension()}]dimensions:[/bold {ThemeColors.dimension()}]",
    ]
    
    for dim in dims:
        size = dim.metadata.get("size", "?")
        is_unlimited = dim.metadata.get("unlimited", False)
        dim_name = dim.name.split()[0]  # Remove any extra info
        
        if is_unlimited:
            lines.append(
                f"  [{ThemeColors.dimension()}]{dim_name}[/{ThemeColors.dimension()}] = "
                f"[{ThemeColors.warning()}]UNLIMITED[/{ThemeColors.warning()}] ; "
                f"[{ThemeColors.muted()}]// ({size} currently)[/{ThemeColors.muted()}]"
            )
        else:
            lines.append(
                f"  [{ThemeColors.dimension()}]{dim_name}[/{ThemeColors.dimension()}] = {size} ;"
            )
    
    container.mount(Static("\n".join(lines)))


def _render_variables_section(container: VerticalScroll, vars: list[DataNode]) -> None:
    """Render variables in ncdump style with attributes."""
    lines = [
        "",
        f"[bold {ThemeColors.variable()}]variables:[/bold {ThemeColors.variable()}]",
    ]
    
    for var in vars:
        dtype = var.metadata.get("dtype", "")
        dims = var.metadata.get("dims", ())

        # Format: dtype name(dim1, dim2, ...) ;
        if dims:
            dim_str = ", ".join(dims)
            lines.append(
                f"  [{ThemeColors.muted()}]{dtype}[/{ThemeColors.muted()}] "
                f"[bold {ThemeColors.variable()}]{var.name}[/bold {ThemeColors.variable()}]"
                f"({dim_str}) ;"
            )
        else:
            lines.append(
                f"  [{ThemeColors.muted()}]{dtype}[/{ThemeColors.muted()}] "
                f"[bold {ThemeColors.variable()}]{var.name}[/bold {ThemeColors.variable()}] ;"
            )

        # Show variable attributes (ncdump style: :attr = value ;)
        if var.attributes:
            for attr_name, attr_val in var.attributes.items():
                val_str = _format_attribute_value(attr_val, max_len=40)
                lines.append(
                    f"    [{ThemeColors.root()}]:{attr_name}[/{ThemeColors.root()}] = {val_str} ;"
                )
    
    container.mount(Static("\n".join(lines)))


def _render_groups_section(container: VerticalScroll, groups: list[DataNode]) -> None:
    """Render subgroups summary."""
    lines = [
        "",
        f"[bold {ThemeColors.group()}]groups:[/bold {ThemeColors.group()}]",
    ]
    
    for grp in groups:
        sub_vars = sum(1 for c in grp.children if c.node_type == NodeType.VARIABLE)
        sub_dims = sum(1 for c in grp.children if c.node_type == NodeType.DIMENSION)
        sub_grps = sum(1 for c in grp.children if c.node_type == NodeType.GROUP)
        
        info_parts = []
        if sub_vars:
            info_parts.append(f"{sub_vars} vars")
        if sub_dims:
            info_parts.append(f"{sub_dims} dims")
        if sub_grps:
            info_parts.append(f"{sub_grps} groups")
        
        info = ", ".join(info_parts) if info_parts else "empty"
        lines.append(
            f"  [{ThemeColors.group()}]{grp.name}[/{ThemeColors.group()}] "
            f"[{ThemeColors.muted()}]({info})[/{ThemeColors.muted()}]"
        )
    
    container.mount(Static("\n".join(lines)))


def _render_attributes(container: VerticalScroll, node: DataNode) -> None:
    """Render node attributes section."""
    lines = [
        "",
        f"[bold {ThemeColors.root()}]attributes:[/bold {ThemeColors.root()}]",
    ]
    
    for key, val in node.attributes.items():
        val_str = _format_attribute_value(val, max_len=60)
        lines.append(f"  [{ThemeColors.root()}]:{key}[/{ThemeColors.root()}] = {val_str} ;")
    
    container.mount(Static("\n".join(lines)))


def _render_data_preview(
    container: VerticalScroll,
    node: DataNode,
    dataset: DatasetInfo,
) -> None:
    """Render data preview with statistics and sample values."""
    try:
        data = load_variable_data(node, dataset)
    except Exception as e:
        container.mount(Static(
            f"\n[{ThemeColors.error()}]Error loading data: {e}[/{ThemeColors.error()}]"
        ))
        return

    if data is None:
        container.mount(Static(f"\n[{ThemeColors.muted()}]Could not load data[/{ThemeColors.muted()}]"))
        return

    lines = [
        "",
        f"[bold {ThemeColors.success()}]Data Preview[/bold {ThemeColors.success()}] "
        f"[{ThemeColors.muted()}](press p to plot, d for table)[/{ThemeColors.muted()}]",
    ]
    container.mount(Static("\n".join(lines)))

    # Statistics for numeric data
    if np.issubdtype(data.dtype, np.number):
        stats = _format_statistics_themed(data)
        if stats:
            container.mount(Static(stats))

    # Sample values
    samples = _format_sample_values_themed(data, max_lines=8)
    container.mount(Static(samples))


def _format_attribute_value(val, max_len: int = 60) -> str:
    """Format an attribute value for display."""
    val_str = str(val)
    if len(val_str) > max_len:
        val_str = val_str[:max_len - 3] + "..."
    # Escape Rich markup characters
    val_str = val_str.replace('[', '\\[').replace(']', '\\]')
    return val_str


def _format_statistics_themed(data: np.ndarray) -> str:
    """Format statistics with theme colors."""
    if not np.issubdtype(data.dtype, np.number):
        return ""

    is_float = data.dtype.kind == 'f'
    valid_count = np.count_nonzero(~np.isnan(data)) if is_float else data.size
    nan_count = data.size - valid_count

    lines = [f"[{ThemeColors.info()}]Statistics:[/{ThemeColors.info()}]"]
    
    try:
        lines.append(f"  [{ThemeColors.muted()}]Min:[/{ThemeColors.muted()}]  {np.nanmin(data):.6g}")
        lines.append(f"  [{ThemeColors.muted()}]Max:[/{ThemeColors.muted()}]  {np.nanmax(data):.6g}")
        lines.append(f"  [{ThemeColors.muted()}]Mean:[/{ThemeColors.muted()}] {np.nanmean(data):.6g}")
        
        if data.size > 1:
            lines.append(f"  [{ThemeColors.muted()}]Std:[/{ThemeColors.muted()}]  {np.nanstd(data):.6g}")
        
        if nan_count > 0:
            pct = nan_count / data.size * 100
            lines.append(
                f"  [{ThemeColors.warning()}]NaN:[/{ThemeColors.warning()}]  "
                f"{nan_count:,} ({pct:.1f}%)"
            )
        
        lines.append(f"  [{ThemeColors.muted()}]Valid:[/{ThemeColors.muted()}] {valid_count:,}")
    except Exception:
        lines.append(f"  [{ThemeColors.error()}]Could not compute statistics[/{ThemeColors.error()}]")

    return "\n".join(lines)


def _format_sample_values_themed(data: np.ndarray, max_lines: int = 8) -> str:
    """Format sample values with theme colors."""
    if data.size == 0:
        return f"[{ThemeColors.muted()}](empty array)[/{ThemeColors.muted()}]"

    lines = [f"[{ThemeColors.variable()}]Sample Values:[/{ThemeColors.variable()}]"]

    if data.ndim == 1:
        lines.extend(_format_1d_samples(data, max_lines))
    elif data.ndim == 2:
        lines.extend(_format_2d_samples(data, max_lines))
    else:
        lines.extend(_format_nd_samples(data, max_lines))

    return "\n".join(lines)


def _format_1d_samples(data: np.ndarray, max_lines: int) -> list[str]:
    """Format 1D array samples."""
    lines = []
    
    if data.size <= max_lines:
        for i, val in enumerate(data):
            lines.append(f"  [{ThemeColors.muted()}][{i}][/{ThemeColors.muted()}] {val}")
    else:
        n = max_lines // 2
        for i in range(n):
            lines.append(f"  [{ThemeColors.muted()}][{i}][/{ThemeColors.muted()}] {data[i]}")
        
        lines.append(f"  [{ThemeColors.muted()}]... ({data.size - 2*n} more) ...[/{ThemeColors.muted()}]")
        
        for i in range(data.size - n, data.size):
            lines.append(f"  [{ThemeColors.muted()}][{i}][/{ThemeColors.muted()}] {data[i]}")
    
    return lines


def _format_2d_samples(data: np.ndarray, max_lines: int) -> list[str]:
    """Format 2D array samples showing corners."""
    rows, cols = data.shape
    show_rows = min(max_lines // 2, 4)
    show_cols = min(8, cols)
    
    lines = []

    def format_row(i: int) -> str:
        vals = " ".join(f"{data[i, j]:9.3g}" for j in range(min(show_cols, cols)))
        suffix = " ..." if cols > show_cols else ""
        return f"  [{ThemeColors.muted()}][{i}][/{ThemeColors.muted()}] {vals}{suffix}"

    # Top rows
    for i in range(min(show_rows, rows)):
        lines.append(format_row(i))

    # Middle indicator
    if rows > show_rows * 2:
        lines.append(f"  [{ThemeColors.muted()}]... {rows - show_rows * 2} rows omitted ...[/{ThemeColors.muted()}]")

    # Bottom rows
    if rows > show_rows:
        for i in range(max(show_rows, rows - show_rows), rows):
            lines.append(format_row(i))

    return lines


def _format_nd_samples(data: np.ndarray, max_lines: int) -> list[str]:
    """Format multi-dimensional array samples."""
    n = max_lines // 2
    lines = []
    
    # First values
    for i in range(min(n, data.size)):
        lines.append(f"  [{ThemeColors.muted()}][{i}][/{ThemeColors.muted()}] {data.flat[i]}")

    # Middle indicator
    if data.size > max_lines:
        lines.append(f"  [{ThemeColors.muted()}]... {data.size - max_lines} values omitted ...[/{ThemeColors.muted()}]")
        
        # Last values
        for i in range(max(0, data.size - n), data.size):
            lines.append(f"  [{ThemeColors.muted()}][{i}][/{ThemeColors.muted()}] {data.flat[i]}")

    # Shape info
    shape_str = " × ".join(f"{s:,}" for s in data.shape)
    lines.append(f"  [{ThemeColors.muted()}](Shape: {shape_str}, Total: {data.size:,})[/{ThemeColors.muted()}]")
    
    return lines


# =============================================================================
# Data Loading
# =============================================================================

def load_variable_data(node: DataNode, dataset: DatasetInfo) -> np.ndarray | None:
    """Load variable data from file.
    
    Tries xarray first, then falls back to netCDF4 for nested groups.
    """
    var_path = node.path

    # Extract variable name from path
    if "/variables/" in var_path:
        var_name = var_path.split("/variables/")[1]
    elif "/coordinates/" in var_path:
        var_name = var_path.split("/coordinates/")[1]
    else:
        var_name = node.name

    # Try xarray first (handles most cases)
    data = _load_with_xarray(dataset.file_path, var_name)
    if data is not None:
        return data

    # Fall back to netCDF4 for nested groups
    data = _load_with_netcdf4(dataset.file_path, var_path)
    if data is not None:
        return data

    return None


def _load_with_xarray(file_path: str, var_name: str) -> np.ndarray | None:
    """Load variable data using xarray."""
    try:
        with xr.open_dataset(file_path) as ds:
            if var_name in ds.variables:
                data = ds[var_name].values
                return _sanitize_array(data)
    except Exception:
        pass
    return None


def _load_with_netcdf4(file_path: str, var_path: str) -> np.ndarray | None:
    """Load variable data using netCDF4 (for nested groups)."""
    try:
        import netCDF4 as nc
        
        with nc.Dataset(file_path, 'r') as ncds:
            parts = [p for p in var_path.split('/') if p]
            obj = ncds
            
            # Navigate to the variable
            for part in parts[:-1]:
                if part in obj.groups:
                    obj = obj.groups[part]
                elif part in obj.variables:
                    obj = obj.variables[part]
                    break

            var_name = parts[-1]
            
            if hasattr(obj, 'variables') and var_name in obj.variables:
                data = obj.variables[var_name][:]
                return _sanitize_array(data)
            elif hasattr(obj, '__getitem__'):
                data = obj[var_name][:]
                return _sanitize_array(data)
    except Exception:
        pass
    
    return None


def _sanitize_array(data: np.ndarray) -> np.ndarray:
    """Sanitize array data: handle masked arrays and ensure writability."""
    # Handle masked arrays from netCDF4
    if hasattr(data, 'filled'):
        if np.issubdtype(data.dtype, np.floating):
            data = data.filled(np.nan)
        else:
            # Convert integer types to float to use NaN
            data = data.astype(float)
            if hasattr(data, 'filled'):
                data = data.filled(np.nan)

    # Ensure array is writable (copy if read-only)
    if not data.flags.writeable:
        data = np.array(data, copy=True)

    return data
