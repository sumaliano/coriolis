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

        // Read dimensions
        let mut dims_node = DataNode::new(
            "Dimensions".to_string(),
            "/dimensions".to_string(),
            NodeType::Group,
        );
        for dim in file.dimensions() {
            let dim_name = dim.name();
            let mut dim_node = DataNode::new(
                format!("{} ({})", dim_name, dim.len()),
                format!("/dimensions/{}", dim_name),
                NodeType::Dimension,
            );
            dim_node
                .metadata
                .insert("length".to_string(), dim.len().to_string());
            dims_node.add_child(dim_node);
        }
        if !dims_node.children.is_empty() {
            root_node.add_child(dims_node);
        }

        // Read variables
        for var in file.variables() {
            let var_name = var.name();
            let mut var_node = DataNode::new(
                var_name.to_string(),
                format!("/variables/{}", var_name),
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

            root_node.add_child(var_node);
        }

        // Note: NetCDF4 groups reading requires API verification
        // Groups iteration may work differently in this netcdf version

        Ok(DatasetInfo::new(path.to_path_buf(), root_node))
    }

    fn attr_value_to_string(attr: &netcdf::Attribute<'_>) -> String {
        format!("{:?}", attr)
    }
}
