"""Data file readers for netCDF, HDF5, and other formats.

This module provides lazy loading for scientific data files:
- Fast structure-only reading (groups, variables, dimensions)
- On-demand detail loading (attributes, metadata)
- Support for netCDF4 and HDF5 formats
"""

from pathlib import Path
from typing import Union

from .models import DataNode, DatasetInfo, NodeType

# Defer heavy imports until needed
_h5py = None
_netcdf4 = None


def _get_h5py():
    """Lazy import h5py."""
    global _h5py
    if _h5py is None:
        import h5py
        _h5py = h5py
    return _h5py


def _get_netcdf4():
    """Lazy import netCDF4."""
    global _netcdf4
    if _netcdf4 is None:
        import netCDF4
        _netcdf4 = netCDF4
    return _netcdf4


class DataReader:
    """Reader for various scientific data formats.
    
    Supports lazy loading for fast initial file opening:
    1. read_file_structure_only() - Fast structure reading
    2. load_node_details() - On-demand detail loading
    """

    SUPPORTED_EXTENSIONS = {".nc", ".nc4", ".netcdf", ".hdf5", ".h5", ".he5"}

    @classmethod
    def can_read(cls, file_path: Union[str, Path]) -> bool:
        """Check if the file can be read by this reader.
        
        Args:
            file_path: Path to the file to check
            
        Returns:
            True if file extension is supported
        """
        path = Path(file_path)
        return path.suffix.lower() in cls.SUPPORTED_EXTENSIONS

    @classmethod
    def read_file_structure_only(cls, file_path: Union[str, Path]) -> DatasetInfo:
        """Read only the basic file structure (fast initial load).
        
        This loads just the hierarchy (groups, variable names, dimensions)
        without loading full metadata or attributes. Use this for initial
        tree population, then call load_node_details() for full info.
        
        Args:
            file_path: Path to the file to read
            
        Returns:
            DatasetInfo with basic structure (nodes have is_fully_loaded=False)
            
        Raises:
            FileNotFoundError: If file doesn't exist
            ValueError: If file type is unsupported or reading fails
        """
        path = Path(file_path)

        if not path.exists():
            raise FileNotFoundError(f"File not found: {file_path}")

        if not cls.can_read(path):
            raise ValueError(f"Unsupported file type: {path.suffix}")

        # Try netCDF4 first (handles both netCDF4 and HDF5 with groups)
        try:
            result = cls._read_structure_netcdf4(path)
            if result.root_node.children:
                return result
        except Exception as e:
            # Log but continue to fallback
            pass

        # Fall back to h5py for pure HDF5 files
        try:
            return cls._read_structure_h5py(path)
        except Exception as e:
            raise ValueError(f"Failed to read file {file_path}: {e}")

    @classmethod
    def load_node_details(cls, file_path: Union[str, Path], node: DataNode) -> None:
        """Load full details for a specific node (attributes, metadata).
        
        This is called on-demand when a node is selected to populate
        its full information without blocking the initial load.
        
        Args:
            file_path: Path to the data file
            node: DataNode to load details for
            
        Side effects:
            Updates node.attributes and node.is_fully_loaded
        """
        if node.is_fully_loaded:
            return
            
        path = Path(file_path)
        
        # Try netCDF4 first
        try:
            nc = _get_netcdf4()
            with nc.Dataset(path, 'r') as ds:
                cls._load_node_details_nc4(ds, node)
                node.is_fully_loaded = True
                return
        except Exception:
            pass
            
        # Fall back to h5py
        try:
            h5py = _get_h5py()
            with h5py.File(path, "r") as f:
                cls._load_node_details_h5(f, node)
                node.is_fully_loaded = True
        except Exception:
            # If both fail, mark as loaded anyway to avoid repeated attempts
            node.is_fully_loaded = True

    # =========================================================================
    # Fast Structure-Only Loading (for initial tree population)
    # =========================================================================

    @classmethod
    def _read_structure_netcdf4(cls, file_path: Path) -> DatasetInfo:
        """Read only structure using netCDF4 (no attributes).
        
        Args:
            file_path: Path to netCDF file
            
        Returns:
            DatasetInfo with structure only
        """
        nc = _get_netcdf4()

        root = DataNode(
            name=file_path.name,
            node_type=NodeType.ROOT,
            path="/",
            is_fully_loaded=False,
        )

        with nc.Dataset(file_path, 'r') as ds:
            cls._read_nc4_group_structure(ds, root, "/")

        return DatasetInfo(
            file_path=str(file_path),
            file_type="NetCDF4/HDF5",
            root_node=root,
        )

    @classmethod
    def _read_nc4_group_structure(cls, group, parent_node: DataNode, path: str) -> None:
        """Read netCDF4 group structure without attributes (fast).
        
        Recursively traverses groups, adding dimensions, variables, and subgroups
        without loading their attributes.
        
        Args:
            group: netCDF4 Group or Dataset object
            parent_node: Parent DataNode to attach children to
            path: Current path in the hierarchy
        """
        # Add dimensions
        if hasattr(group, 'dimensions') and group.dimensions:
            for dim_name, dim_obj in group.dimensions.items():
                dim_node = DataNode(
                    name=dim_name,
                    node_type=NodeType.DIMENSION,
                    path=f"{path}{dim_name}",
                    metadata={
                        "size": len(dim_obj),
                        "unlimited": dim_obj.isunlimited(),
                    },
                    parent=parent_node,
                    is_fully_loaded=True,  # Dimensions are simple
                )
                parent_node.add_child(dim_node)

        # Add variables (minimal info)
        if hasattr(group, 'variables') and group.variables:
            for var_name, var_obj in group.variables.items():
                var_node = DataNode(
                    name=var_name,
                    node_type=NodeType.VARIABLE,
                    path=f"{path}{var_name}",
                    metadata={
                        "dtype": str(var_obj.dtype),
                        "shape": var_obj.shape,
                        "dims": var_obj.dimensions,
                        "size": var_obj.size,
                    },
                    parent=parent_node,
                    is_fully_loaded=False,  # Attributes not loaded yet
                )
                parent_node.add_child(var_node)

        # Add subgroups
        if hasattr(group, 'groups') and group.groups:
            for grp_name, grp_obj in group.groups.items():
                grp_node = DataNode(
                    name=f"/{grp_name}",
                    node_type=NodeType.GROUP,
                    path=f"{path}{grp_name}/",
                    parent=parent_node,
                    is_fully_loaded=False,
                )
                parent_node.add_child(grp_node)
                cls._read_nc4_group_structure(grp_obj, grp_node, f"{path}{grp_name}/")

    @classmethod
    def _read_structure_h5py(cls, file_path: Path) -> DatasetInfo:
        """Read only structure using h5py (no attributes).
        
        Args:
            file_path: Path to HDF5 file
            
        Returns:
            DatasetInfo with structure only
        """
        h5py = _get_h5py()
        root = DataNode(
            name=file_path.name,
            node_type=NodeType.ROOT,
            path="/",
            is_fully_loaded=False,
        )

        with h5py.File(file_path, "r") as f:
            cls._read_h5_group_structure(f, root, "/")

        return DatasetInfo(
            file_path=str(file_path),
            file_type="HDF5",
            root_node=root,
        )

    @classmethod
    def _read_h5_group_structure(cls, h5_obj, parent_node: DataNode, path: str) -> None:
        """Read HDF5 group structure without attributes (fast).
        
        Recursively traverses HDF5 groups and datasets without loading attributes.
        
        Args:
            h5_obj: h5py Group or File object
            parent_node: Parent DataNode to attach children to
            path: Current path in the hierarchy
        """
        h5py = _get_h5py()
        
        if not hasattr(h5_obj, 'keys'):
            return
            
        for key in h5_obj.keys():
            try:
                item = h5_obj[key]
                item_path = f"{path}{key}" if path.endswith("/") else f"{path}/{key}"

                if isinstance(item, h5py.Group):
                    # It's a group
                    group_node = DataNode(
                        name=key,
                        node_type=NodeType.GROUP,
                        path=item_path,
                        is_fully_loaded=False,
                    )
                    parent_node.add_child(group_node)
                    cls._read_h5_group_structure(item, group_node, item_path)

                elif isinstance(item, h5py.Dataset):
                    # It's a dataset (variable)
                    var_node = DataNode(
                        name=key,
                        node_type=NodeType.VARIABLE,
                        path=item_path,
                        metadata={
                            "dtype": str(item.dtype),
                            "shape": item.shape,
                            "size": item.size,
                        },
                        is_fully_loaded=False,
                    )
                    parent_node.add_child(var_node)
            except Exception:
                # Skip items that can't be read
                continue

    # =========================================================================
    # On-Demand Detail Loading
    # =========================================================================

    @classmethod
    def _load_node_details_nc4(cls, dataset, node: DataNode) -> None:
        """Load full details for a node from netCDF4 dataset.
        
        Args:
            dataset: Open netCDF4 Dataset
            node: DataNode to populate with details
        """
        # Navigate to the node's location in the file
        obj = cls._navigate_to_nc4_path(dataset, node.path)
        if obj is None:
            return

        # Load attributes
        try:
            if hasattr(obj, 'ncattrs'):
                node.attributes = {attr: getattr(obj, attr) for attr in obj.ncattrs()}
        except Exception:
            pass

        # For variables, ensure metadata is complete
        if node.node_type == NodeType.VARIABLE and hasattr(obj, 'dimensions'):
            try:
                node.metadata.update({
                    "dtype": str(obj.dtype),
                    "shape": obj.shape,
                    "dims": obj.dimensions,
                    "size": obj.size,
                })
            except Exception:
                pass

    @classmethod
    def _navigate_to_nc4_path(cls, dataset, path: str):
        """Navigate to a specific path in netCDF4 dataset.
        
        Args:
            dataset: netCDF4 Dataset object
            path: Path to navigate to (e.g., "/group1/var1")
            
        Returns:
            The object at the path, or None if not found
        """
        if path == "/" or not path:
            return dataset

        # Remove leading/trailing slashes and split
        parts = path.strip("/").split("/")
        obj = dataset

        for part in parts:
            if not part:
                continue
                
            # Try as group first
            if hasattr(obj, 'groups') and part in obj.groups:
                obj = obj.groups[part]
            # Then try as variable
            elif hasattr(obj, 'variables') and part in obj.variables:
                return obj.variables[part]
            # Then try as dimension
            elif hasattr(obj, 'dimensions') and part in obj.dimensions:
                return obj.dimensions[part]
            else:
                return None

        return obj

    @classmethod
    def _load_node_details_h5(cls, h5file, node: DataNode) -> None:
        """Load full details for a node from HDF5 file.
        
        Args:
            h5file: Open h5py File object
            node: DataNode to populate with details
        """
        try:
            obj = h5file[node.path]
            
            # Load attributes
            if hasattr(obj, 'attrs'):
                node.attributes = dict(obj.attrs)
            
            # For datasets, ensure metadata is complete
            h5py = _get_h5py()
            if isinstance(obj, h5py.Dataset):
                try:
                    node.metadata.update({
                        "dtype": str(obj.dtype),
                        "shape": obj.shape,
                        "size": obj.size,
                    })
                except Exception:
                    pass
        except Exception:
            pass
