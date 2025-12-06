"""Detail panel rendering for Tanotly."""

import numpy as np
import xarray as xr

from textual.containers import VerticalScroll
from textual.widgets import Static

from .config import NODE_COLORS, NODE_ICONS
from .data.models import DataNode, DatasetInfo, NodeType
from .visualization import DataVisualizer, format_statistics, format_sample_values


def render_details(
    container: VerticalScroll,
    node: DataNode,
    dataset: DatasetInfo | None,
    show_plot: bool,
) -> None:
    """Render node details into the container."""
    container.remove_children()

    # Header
    icon = NODE_ICONS.get(node.node_type, "●")
    color = NODE_COLORS.get(node.node_type, "white")
    header = f"{icon} [bold cyan]{node.name}[/bold cyan]\n"
    header += "-" * 60 + "\n\n"
    header += f"[{color}]● Type:[/{color}] {node.node_type.value}\n"
    header += f"[dim]● Path:[/dim] [cyan]{node.path}[/cyan]\n\n"
    container.mount(Static(header))

    # Metadata section
    if node.metadata:
        content = "[bold yellow]📊 Metadata[/bold yellow]\n" + "-" * 60 + "\n"
        for key, val in node.metadata.items():
            if key == "shape":
                val = " × ".join(str(s) for s in val)
            elif key == "dims":
                val = "(" + ", ".join(str(d) for d in val) + ")"
            elif key == "size" and isinstance(val, int):
                val = f"{val:,}"
            content += f"  [cyan]▸ {key}:[/cyan] {val}\n"
        container.mount(Static(content + "\n"))

    # Attributes section
    if node.attributes:
        content = "[bold magenta]🏷️  Attributes[/bold magenta]\n" + "-" * 60 + "\n"
        for key, val in node.attributes.items():
            val_str = str(val)[:77] + "..." if len(str(val)) > 80 else str(val)
            val_str = val_str.replace('[', '\\[').replace(']', '\\]')
            content += f"  [magenta]:{key}[/magenta] = {val_str}\n"
        container.mount(Static(content + "\n"))

    # Data preview for variables
    if node.node_type == NodeType.VARIABLE and dataset:
        try:
            _render_data_preview(container, node, dataset, show_plot)
        except Exception as e:
            container.mount(Static(f"\n[red]Error loading data: {e}[/red]\n"))


def _render_data_preview(
    container: VerticalScroll,
    node: DataNode,
    dataset: DatasetInfo,
    show_plot: bool,
) -> None:
    """Render data preview with optional plot."""
    data = _load_variable_data(node, dataset)
    if data is None:
        container.mount(Static("[dim]Could not load data[/dim]"))
        return

    container.mount(Static("\n[bold green]📈 Data Preview[/bold green]\n" + "-" * 60))

    # Plot visualization
    if show_plot and np.issubdtype(data.dtype, np.number) and data.size > 0:
        container.mount(Static(" "))
        for widget in DataVisualizer.create_visualization(data):
            container.mount(widget)
        container.mount(Static(" "))

    # Statistics
    if np.issubdtype(data.dtype, np.number):
        stats = format_statistics(data)
        if stats.strip():
            container.mount(Static(stats))

    # Sample values
    container.mount(Static("[cyan]Sample Values:[/cyan]"))
    samples = format_sample_values(data, max_lines=8)
    container.mount(Static(samples if samples.strip() else "[dim]No sample data[/dim]"))


def _load_variable_data(node: DataNode, dataset: DatasetInfo) -> np.ndarray | None:
    """Load variable data from file."""
    var_path = node.path

    # Extract variable name from path
    if "/variables/" in var_path:
        var_name = var_path.split("/variables/")[1]
    elif "/coordinates/" in var_path:
        var_name = var_path.split("/coordinates/")[1]
    else:
        var_name = node.name

    # Try xarray first
    try:
        with xr.open_dataset(dataset.file_path) as ds:
            if var_name in ds.variables:
                return ds[var_name].values
    except Exception:
        pass

    # Fall back to netCDF4
    try:
        import netCDF4 as nc
        with nc.Dataset(dataset.file_path, 'r') as ncds:
            parts = [p for p in var_path.split('/') if p]
            obj = ncds
            for part in parts[:-1]:
                if part in obj.groups:
                    obj = obj.groups[part]
                elif part in obj.variables:
                    obj = obj.variables[part]
                    break

            var_name_final = parts[-1]
            if hasattr(obj, 'variables') and var_name_final in obj.variables:
                return obj.variables[var_name_final][:]
            elif hasattr(obj, '__getitem__'):
                return obj[var_name_final][:]
    except Exception:
        pass

    return None
