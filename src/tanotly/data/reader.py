"""Data file readers for netCDF, HDF5, and other formats."""

import os
from pathlib import Path
from typing import Optional, Union

import h5py
import numpy as np
import xarray as xr

from .models import DataNode, DatasetInfo, NodeType


class DataReader:
    """Reader for various scientific data formats."""

    SUPPORTED_EXTENSIONS = {".nc", ".nc4", ".netcdf", ".hdf5", ".h5", ".he5"}

    @classmethod
    def can_read(cls, file_path: Union[str, Path]) -> bool:
        """Check if the file can be read by this reader."""
        path = Path(file_path)
        return path.suffix.lower() in cls.SUPPORTED_EXTENSIONS

    @classmethod
    def read_file(cls, file_path: Union[str, Path]) -> DatasetInfo:
        """Read a data file and return its structure."""
        path = Path(file_path)

        if not path.exists():
            raise FileNotFoundError(f"File not found: {file_path}")

        if not cls.can_read(path):
            raise ValueError(f"Unsupported file type: {path.suffix}")

        # Try xarray first (handles simple netCDF files well)
        try:
            result = cls._read_with_xarray(path)
            # Check if xarray actually found data
            if result.variables or result.dimensions:
                return result
        except Exception:
            pass

        # Try netCDF4 for files with groups (handles both netCDF4 and HDF5)
        try:
            import netCDF4 as nc
            result = cls._read_with_netcdf4(path)
            if result.root_node.children:
                return result
        except Exception:
            pass

        # Fall back to h5py for pure HDF5 files
        try:
            return cls._read_with_h5py(path)
        except Exception as e:
            raise ValueError(f"Failed to read file {file_path}: {e}")

    @classmethod
    def _read_with_xarray(cls, file_path: Path) -> DatasetInfo:
        """Read file using xarray."""
        ds = xr.open_dataset(file_path)

        # Create root node
        root = DataNode(
            name=file_path.name,
            node_type=NodeType.ROOT,
            path="/",
            attributes=dict(ds.attrs),
        )

        # Add dimensions
        dimensions_node = DataNode(
            name="Dimensions",
            node_type=NodeType.GROUP,
            path="/dimensions",
        )
        root.add_child(dimensions_node)

        dimensions = {}
        for dim_name, dim_size in ds.sizes.items():
            dimensions[dim_name] = dim_size
            dim_node = DataNode(
                name=f"{dim_name} ({dim_size})",
                node_type=NodeType.DIMENSION,
                path=f"/dimensions/{dim_name}",
                metadata={"size": dim_size},
            )
            dimensions_node.add_child(dim_node)

        # Add variables
        variables_node = DataNode(
            name="Variables",
            node_type=NodeType.GROUP,
            path="/variables",
        )
        root.add_child(variables_node)

        variable_names = []
        for var_name, var_data in ds.data_vars.items():
            variable_names.append(var_name)
            var_node = DataNode(
                name=var_name,
                node_type=NodeType.VARIABLE,
                path=f"/variables/{var_name}",
                attributes=dict(var_data.attrs),
                metadata={
                    "dtype": str(var_data.dtype),
                    "shape": var_data.shape,
                    "dims": var_data.dims,
                    "size": var_data.size,
                },
            )
            variables_node.add_child(var_node)

            # Add attributes as child nodes
            for attr_name, attr_value in var_data.attrs.items():
                attr_node = DataNode(
                    name=f"{attr_name}: {attr_value}",
                    node_type=NodeType.ATTRIBUTE,
                    path=f"/variables/{var_name}/{attr_name}",
                    metadata={"value": attr_value},
                )
                var_node.add_child(attr_node)

        # Add coordinates
        if ds.coords:
            coords_node = DataNode(
                name="Coordinates",
                node_type=NodeType.GROUP,
                path="/coordinates",
            )
            root.add_child(coords_node)

            for coord_name, coord_data in ds.coords.items():
                coord_node = DataNode(
                    name=coord_name,
                    node_type=NodeType.VARIABLE,
                    path=f"/coordinates/{coord_name}",
                    attributes=dict(coord_data.attrs),
                    metadata={
                        "dtype": str(coord_data.dtype),
                        "shape": coord_data.shape,
                        "size": coord_data.size,
                    },
                )
                coords_node.add_child(coord_node)

        ds.close()

        return DatasetInfo(
            file_path=str(file_path),
            file_type="netCDF/HDF5",
            root_node=root,
            dimensions=dimensions,
            global_attributes=dict(root.attributes),
            variables=variable_names,
        )

    @classmethod
    def _read_with_netcdf4(cls, file_path: Path) -> DatasetInfo:
        """Read file using netCDF4 (handles groups and dimensions better)."""
        import netCDF4 as nc

        root = DataNode(
            name=file_path.name,
            node_type=NodeType.ROOT,
            path="/",
        )

        with nc.Dataset(file_path, 'r') as ds:
            # Get global attributes
            root.attributes = {attr: getattr(ds, attr) for attr in ds.ncattrs()}

            # Recursively read structure
            cls._read_nc4_group(ds, root, "/")

        return DatasetInfo(
            file_path=str(file_path),
            file_type="NetCDF4/HDF5",
            root_node=root,
            global_attributes=dict(root.attributes),
        )

    @classmethod
    def _read_nc4_group(cls, group, parent_node: DataNode, path: str) -> None:
        """Recursively read netCDF4 group structure."""
        # Add dimensions directly (not in a sub-group)
        if group.dimensions:
            for dim_name, dim_obj in group.dimensions.items():
                dim_node = DataNode(
                    name=dim_name,
                    node_type=NodeType.DIMENSION,
                    path=f"{path}{dim_name}",
                    metadata={"size": len(dim_obj)},
                    parent=parent_node
                )
                parent_node.add_child(dim_node)

        # Add variables
        for var_name, var_obj in group.variables.items():
            var_node = DataNode(
                name=var_name,
                node_type=NodeType.VARIABLE,
                path=f"{path}{var_name}",
                attributes={attr: getattr(var_obj, attr) for attr in var_obj.ncattrs()},
                metadata={
                    "dtype": str(var_obj.dtype),
                    "shape": var_obj.shape,
                    "dims": var_obj.dimensions,
                    "size": var_obj.size,
                },
                parent=parent_node
            )
            parent_node.add_child(var_node)

        # Add subgroups
        for grp_name, grp_obj in group.groups.items():
            grp_node = DataNode(
                name=f"/{grp_name}",
                node_type=NodeType.GROUP,
                path=f"{path}{grp_name}/",
                attributes={attr: getattr(grp_obj, attr) for attr in grp_obj.ncattrs()},
                parent=parent_node
            )
            parent_node.add_child(grp_node)
            cls._read_nc4_group(grp_obj, grp_node, f"{path}{grp_name}/")

    @classmethod
    def _read_with_h5py(cls, file_path: Path) -> DatasetInfo:
        """Read HDF5 file using h5py."""
        root = DataNode(
            name=file_path.name,
            node_type=NodeType.ROOT,
            path="/",
        )

        with h5py.File(file_path, "r") as f:
            # Get global attributes
            global_attrs = dict(f.attrs)
            root.attributes = global_attrs

            # Recursively read the HDF5 structure
            cls._read_h5_group(f, root, "/")

        return DatasetInfo(
            file_path=str(file_path),
            file_type="HDF5",
            root_node=root,
            global_attributes=global_attrs,
        )

    @classmethod
    def _read_h5_group(cls, h5_obj: Union[h5py.File, h5py.Group], parent_node: DataNode, path: str) -> None:
        """Recursively read HDF5 group structure."""
        for key in h5_obj.keys():
            item = h5_obj[key]
            item_path = f"{path}{key}" if path.endswith("/") else f"{path}/{key}"

            if isinstance(item, h5py.Group):
                # It's a group
                group_node = DataNode(
                    name=key,
                    node_type=NodeType.GROUP,
                    path=item_path,
                    attributes=dict(item.attrs),
                )
                parent_node.add_child(group_node)
                cls._read_h5_group(item, group_node, item_path)

            elif isinstance(item, h5py.Dataset):
                # It's a dataset (variable)
                var_node = DataNode(
                    name=key,
                    node_type=NodeType.VARIABLE,
                    path=item_path,
                    attributes=dict(item.attrs),
                    metadata={
                        "dtype": str(item.dtype),
                        "shape": item.shape,
                        "size": item.size,
                    },
                )
                parent_node.add_child(var_node)

                # Add attributes as child nodes
                for attr_name, attr_value in item.attrs.items():
                    attr_node = DataNode(
                        name=f"{attr_name}: {attr_value}",
                        node_type=NodeType.ATTRIBUTE,
                        path=f"{item_path}/{attr_name}",
                        metadata={"value": attr_value},
                    )
                    var_node.add_child(attr_node)
