//! NetCDF file reader.

use super::{DataNode, DatasetInfo, NodeType};
use crate::error::Result;
use std::path::Path;

/// NetCDF data reader.
#[derive(Debug)]
pub struct DataReader;

impl DataReader {
    /// Read a NetCDF file.
    pub fn read_file(path: &Path) -> Result<DatasetInfo> {
        let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");

        match extension {
            "nc" | "nc4" | "netcdf" => Self::read_netcdf(path),
            _ => Self::read_netcdf(path),
        }
    }

    fn read_netcdf(path: &Path) -> Result<DatasetInfo> {
        let file =
            netcdf::open(path).map_err(|e| crate::error::CoriolisError::NetCDF(e.to_string()))?;

        let mut root_node = DataNode::new(
            path.file_name().unwrap().to_string_lossy().to_string(),
            "/".to_string(),
            NodeType::Root,
        );

        // Read global attributes
        for attr in file.attributes() {
            root_node
                .attributes
                .insert(attr.name().to_string(), Self::attr_value_to_string(&attr));
        }

        // Store dimensions as metadata on the root node
        for dim in file.dimensions() {
            let dim_name = dim.name();
            root_node
                .metadata
                .insert(format!("dim_{}", dim_name), dim.len().to_string());
        }

        // Read variables at root level
        for var in file.variables() {
            root_node.add_child(Self::read_variable(&var, ""));
        }

        // Read groups recursively
        if let Ok(groups) = file.groups() {
            for group in groups {
                root_node.add_child(Self::read_group(&group, ""));
            }
        }

        Ok(DatasetInfo::new(path.to_path_buf(), root_node))
    }

    fn read_group(group: &netcdf::Group<'_>, parent_path: &str) -> DataNode {
        let group_name = group.name();
        let group_path = if parent_path.is_empty() {
            format!("/{}", group_name)
        } else {
            format!("{}/{}", parent_path, group_name)
        };

        let mut group_node = DataNode::new(
            group_name.to_string(),
            group_path.clone(),
            NodeType::Group,
        );

        // Read group attributes
        for attr in group.attributes() {
            group_node
                .attributes
                .insert(attr.name().to_string(), Self::attr_value_to_string(&attr));
        }

        // Store dimensions as metadata on the group node
        for dim in group.dimensions() {
            let dim_name = dim.name();
            group_node
                .metadata
                .insert(format!("dim_{}", dim_name), dim.len().to_string());
        }

        // Read variables in this group
        for var in group.variables() {
            group_node.add_child(Self::read_variable(&var, &group_path));
        }

        // Read child groups recursively
        for child_group in group.groups() {
            group_node.add_child(Self::read_group(&child_group, &group_path));
        }

        group_node
    }

    fn read_variable(var: &netcdf::Variable<'_>, parent_path: &str) -> DataNode {
        let var_name = var.name();
        let var_path = if parent_path.is_empty() {
            format!("/{}", var_name)
        } else {
            format!("{}/{}", parent_path, var_name)
        };

        let mut var_node = DataNode::new(
            var_name.to_string(),
            var_path,
            NodeType::Variable,
        );

        // Get shape and type
        var_node.shape = Some(
            var.dimensions()
                .iter()
                .map(|d: &netcdf::Dimension<'_>| d.len())
                .collect(),
        );
        var_node.dtype = Some(format!("{:?}", var.vartype()));

        // Dimension names
        let dim_names: Vec<String> = var
            .dimensions()
            .iter()
            .map(|d: &netcdf::Dimension<'_>| d.name().to_string())
            .collect();
        var_node
            .metadata
            .insert("dims".to_string(), dim_names.join(", "));

        // Read attributes
        for attr in var.attributes() {
            var_node
                .attributes
                .insert(attr.name().to_string(), Self::attr_value_to_string(&attr));
        }

        var_node
    }

    fn attr_value_to_string(attr: &netcdf::Attribute<'_>) -> String {
        use netcdf::AttributeValue;

        match attr.value() {
            Ok(AttributeValue::Uchar(v)) => format!("{}", v),
            Ok(AttributeValue::Schar(v)) => format!("{}", v),
            Ok(AttributeValue::Ushort(v)) => format!("{}", v),
            Ok(AttributeValue::Short(v)) => format!("{}", v),
            Ok(AttributeValue::Uint(v)) => format!("{}", v),
            Ok(AttributeValue::Int(v)) => format!("{}", v),
            Ok(AttributeValue::Ulonglong(v)) => format!("{}", v),
            Ok(AttributeValue::Longlong(v)) => format!("{}", v),
            Ok(AttributeValue::Float(v)) => format!("{}", v),
            Ok(AttributeValue::Double(v)) => format!("{}", v),
            Ok(AttributeValue::Str(v)) => v,
            Ok(AttributeValue::Uchars(v)) => format!("{:?}", v),
            Ok(AttributeValue::Schars(v)) => format!("{:?}", v),
            Ok(AttributeValue::Ushorts(v)) => format!("{:?}", v),
            Ok(AttributeValue::Shorts(v)) => format!("{:?}", v),
            Ok(AttributeValue::Uints(v)) => format!("{:?}", v),
            Ok(AttributeValue::Ints(v)) => format!("{:?}", v),
            Ok(AttributeValue::Ulonglongs(v)) => format!("{:?}", v),
            Ok(AttributeValue::Longlongs(v)) => format!("{:?}", v),
            Ok(AttributeValue::Floats(v)) => format!("{:?}", v),
            Ok(AttributeValue::Doubles(v)) => format!("{:?}", v),
            Ok(AttributeValue::Strs(v)) => v.join(", "),
            Err(_) => format!("{:?}", attr),
        }
    }
}
