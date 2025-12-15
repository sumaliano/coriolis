use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    Root,
    Group,
    Variable,
    Dimension,
    Attribute,
}

#[derive(Debug, Clone)]
pub struct DataNode {
    pub name: String,
    pub path: String,
    pub node_type: NodeType,
    pub metadata: HashMap<String, String>,
    pub children: Vec<DataNode>,
    pub attributes: HashMap<String, String>,
    pub shape: Option<Vec<usize>>,
    pub dtype: Option<String>,
}

impl DataNode {
    pub fn new(name: String, path: String, node_type: NodeType) -> Self {
        Self {
            name,
            path,
            node_type,
            metadata: HashMap::new(),
            children: Vec::new(),
            attributes: HashMap::new(),
            shape: None,
            dtype: None,
        }
    }

    pub fn is_variable(&self) -> bool {
        self.node_type == NodeType::Variable
    }

    pub fn is_group(&self) -> bool {
        self.node_type == NodeType::Group || self.node_type == NodeType::Root
    }

    pub fn add_child(&mut self, child: DataNode) {
        self.children.push(child);
    }

    pub fn display_name(&self) -> String {
        let icon = match self.node_type {
            NodeType::Root => "🏠",
            NodeType::Group => "📂",
            NodeType::Variable => "🌡️",
            NodeType::Dimension => "📏",
            NodeType::Attribute => "🏷️",
        };

        let suffix = match self.node_type {
            NodeType::Variable => {
                if let (Some(shape), Some(dtype)) = (&self.shape, &self.dtype) {
                    format!(" {:?} {}", shape, dtype)
                } else {
                    String::new()
                }
            },
            NodeType::Group | NodeType::Root => {
                format!(" ({})", self.children.len())
            },
            _ => String::new(),
        };

        format!("{} {}{}", icon, self.name, suffix)
    }

    pub fn matches_search(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();

        // Check name
        if self.name.to_lowercase().contains(&query_lower) {
            return true;
        }

        // Check path
        if self.path.to_lowercase().contains(&query_lower) {
            return true;
        }

        // Check attributes
        for (key, value) in &self.attributes {
            if key.to_lowercase().contains(&query_lower)
                || value.to_lowercase().contains(&query_lower)
            {
                return true;
            }
        }

        // Check metadata
        for (key, value) in &self.metadata {
            if key.to_lowercase().contains(&query_lower)
                || value.to_lowercase().contains(&query_lower)
            {
                return true;
            }
        }

        false
    }
}

#[derive(Debug, Clone)]
pub struct DatasetInfo {
    pub file_path: PathBuf,
    pub root_node: DataNode,
}

pub struct DataReader;

impl DataReader {
    pub fn read_file(path: &Path) -> Result<DatasetInfo> {
        let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");

        match extension {
            "nc" | "nc4" | "netcdf" => Self::read_netcdf(path),
            _ => Self::read_netcdf(path),
        }
    }

    fn read_netcdf(path: &Path) -> Result<DatasetInfo> {
        let file = netcdf::open(path).context("Failed to open NetCDF file")?;

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

        // Read groups (NetCDF4) - groups() API may vary by version
        // TODO: Verify groups() API for this netcdf version
        // for group in file.groups() {
        //     let group_name = group.name();
        //     let group_node =
        //         Self::read_netcdf_group(&group, group_name, &format!("/{}", group_name))?;
        //     root_node.add_child(group_node);
        // }

        Ok(DatasetInfo {
            file_path: path.to_path_buf(),
            root_node,
        })
    }

    fn read_netcdf_group(group: &netcdf::Group<'_>, name: &str, path: &str) -> Result<DataNode> {
        let mut group_node = DataNode::new(name.to_string(), path.to_string(), NodeType::Group);

        // Read group attributes
        for attr in group.attributes() {
            group_node
                .attributes
                .insert(attr.name().to_string(), Self::attr_value_to_string(&attr));
        }

        // Read variables in group
        for var in group.variables() {
            let var_name = var.name();
            let mut var_node = DataNode::new(
                var_name.to_string(),
                format!("{}/{}", path, var_name),
                NodeType::Variable,
            );

            var_node.shape = Some(
                var.dimensions()
                    .iter()
                    .map(|d: &netcdf::Dimension<'_>| d.len())
                    .collect(),
            );
            var_node.dtype = Some(format!("{:?}", var.vartype()));

            for attr in var.attributes() {
                var_node
                    .attributes
                    .insert(attr.name().to_string(), Self::attr_value_to_string(&attr));
            }

            group_node.add_child(var_node);
        }

        // Recursively read subgroups - groups() API may vary by version
        // TODO: Verify groups() API for this netcdf version
        // for subgroup in group.groups() {
        //     let subgroup_name = subgroup.name();
        //     let subgroup_node = Self::read_netcdf_group(
        //         &subgroup,
        //         subgroup_name,
        //         &format!("{}/{}", path, subgroup_name),
        //     )?;
        //     group_node.add_child(subgroup_node);
        // }

        Ok(group_node)
    }

    fn attr_value_to_string(attr: &netcdf::Attribute<'_>) -> String {
        // Try to convert attribute value to string
        format!("{:?}", attr)
    }
}
