//! NetCDF file reader.

use super::{DataNode, DatasetInfo, NodeType};
use crate::error::Result;
use netcdf::types::{FloatType, IntType, NcVariableType};
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

        let mut group_node =
            DataNode::new(group_name.to_string(), group_path.clone(), NodeType::Group);

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

        let mut var_node = DataNode::new(var_name.to_string(), var_path, NodeType::Variable);

        // Get shape and type
        let shape: Vec<usize> = var
            .dimensions()
            .iter()
            .map(|d: &netcdf::Dimension<'_>| d.len())
            .collect();

        var_node.sample = Self::try_read_sample(var, &shape);
        var_node.shape = Some(shape);
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

    /// Read a small preview block from a variable for the details pane.
    ///
    /// - 0D (scalar): full read via RangeFull.
    /// - 1D: up to 6 elements from the start.
    /// - 2D: up to 3×4 = 12 elements (a grid block), stored row-major.
    /// - 3D+: up to 6 elements along the last dimension at index-zero for
    ///   all other dims (a single row from the innermost slice).
    ///
    /// All reads are true partial reads — no full-array load.
    fn try_read_sample(var: &netcdf::Variable<'_>, shape: &[usize]) -> Option<Vec<f64>> {
        let total: usize = shape.iter().product();
        if total == 0 {
            return None;
        }

        let ndim = shape.len();

        if ndim == 0 {
            return Self::read_full(var);
        }

        let starts = vec![0usize; ndim];
        let mut counts = vec![1usize; ndim];

        if ndim == 2 {
            counts[0] = shape[0].min(3);
            counts[1] = shape[1].min(4);
        } else {
            counts[ndim - 1] = shape[ndim - 1].min(6);
        }

        Self::read_partial(var, &starts, &counts)
    }

    fn read_full(var: &netcdf::Variable<'_>) -> Option<Vec<f64>> {
        match var.vartype() {
            NcVariableType::Float(FloatType::F64) => var.get_values::<f64, _>(..).ok(),
            NcVariableType::Float(FloatType::F32) => {
                let v: Vec<f32> = var.get_values(..).ok()?;
                Some(v.into_iter().map(|x| x as f64).collect())
            },
            NcVariableType::Int(IntType::I64) => {
                let v: Vec<i64> = var.get_values(..).ok()?;
                Some(v.into_iter().map(|x| x as f64).collect())
            },
            NcVariableType::Int(IntType::U64) => {
                let v: Vec<u64> = var.get_values(..).ok()?;
                Some(v.into_iter().map(|x| x as f64).collect())
            },
            NcVariableType::Int(IntType::I32) => {
                let v: Vec<i32> = var.get_values(..).ok()?;
                Some(v.into_iter().map(|x| x as f64).collect())
            },
            NcVariableType::Int(IntType::U32) => {
                let v: Vec<u32> = var.get_values(..).ok()?;
                Some(v.into_iter().map(|x| x as f64).collect())
            },
            NcVariableType::Int(IntType::I16) => {
                let v: Vec<i16> = var.get_values(..).ok()?;
                Some(v.into_iter().map(|x| x as f64).collect())
            },
            NcVariableType::Int(IntType::U16) => {
                let v: Vec<u16> = var.get_values(..).ok()?;
                Some(v.into_iter().map(|x| x as f64).collect())
            },
            NcVariableType::Int(IntType::I8) => {
                let v: Vec<i8> = var.get_values(..).ok()?;
                Some(v.into_iter().map(|x| x as f64).collect())
            },
            NcVariableType::Int(IntType::U8) => {
                let v: Vec<u8> = var.get_values(..).ok()?;
                Some(v.into_iter().map(|x| x as f64).collect())
            },
            other => {
                tracing::debug!("no sample: unhandled type {:?}", other);
                None
            },
        }
    }

    fn read_partial(
        var: &netcdf::Variable<'_>,
        starts: &[usize],
        counts: &[usize],
    ) -> Option<Vec<f64>> {
        match var.vartype() {
            NcVariableType::Float(FloatType::F64) => {
                var.get_values::<f64, _>((starts, counts)).ok()
            },
            NcVariableType::Float(FloatType::F32) => {
                let v: Vec<f32> = var.get_values((starts, counts)).ok()?;
                Some(v.into_iter().map(|x| x as f64).collect())
            },
            NcVariableType::Int(IntType::I64) => {
                let v: Vec<i64> = var.get_values((starts, counts)).ok()?;
                Some(v.into_iter().map(|x| x as f64).collect())
            },
            NcVariableType::Int(IntType::U64) => {
                let v: Vec<u64> = var.get_values((starts, counts)).ok()?;
                Some(v.into_iter().map(|x| x as f64).collect())
            },
            NcVariableType::Int(IntType::I32) => {
                let v: Vec<i32> = var.get_values((starts, counts)).ok()?;
                Some(v.into_iter().map(|x| x as f64).collect())
            },
            NcVariableType::Int(IntType::U32) => {
                let v: Vec<u32> = var.get_values((starts, counts)).ok()?;
                Some(v.into_iter().map(|x| x as f64).collect())
            },
            NcVariableType::Int(IntType::I16) => {
                let v: Vec<i16> = var.get_values((starts, counts)).ok()?;
                Some(v.into_iter().map(|x| x as f64).collect())
            },
            NcVariableType::Int(IntType::U16) => {
                let v: Vec<u16> = var.get_values((starts, counts)).ok()?;
                Some(v.into_iter().map(|x| x as f64).collect())
            },
            NcVariableType::Int(IntType::I8) => {
                let v: Vec<i8> = var.get_values((starts, counts)).ok()?;
                Some(v.into_iter().map(|x| x as f64).collect())
            },
            NcVariableType::Int(IntType::U8) => {
                let v: Vec<u8> = var.get_values((starts, counts)).ok()?;
                Some(v.into_iter().map(|x| x as f64).collect())
            },
            other => {
                tracing::debug!("no sample: unhandled type {:?}", other);
                None
            },
        }
    }

    /// Convert a NetCDF attribute value to a string representation.
    pub fn attr_value_to_string(attr: &netcdf::Attribute<'_>) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn write_test_nc(path: &std::path::Path, shape: &[usize], values: &[f32]) {
        let mut file = netcdf::create(path).unwrap();
        let dim_names: Vec<String> = (0..shape.len()).map(|i| format!("d{}", i)).collect();
        for (name, &len) in dim_names.iter().zip(shape.iter()) {
            file.add_dimension(name, len).unwrap();
        }
        let names: Vec<&str> = dim_names.iter().map(|s| s.as_str()).collect();
        let mut var = file.add_variable::<f32>("v", &names).unwrap();
        var.put_values(values, ..).unwrap();
    }

    #[test]
    fn sample_scalar() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.nc");
        {
            let mut file = netcdf::create(&path).unwrap();
            let mut var = file.add_variable::<f32>("v", &[]).unwrap();
            var.put_value(42.0_f32, ..).unwrap();
        }
        let ds = DataReader::read_file(&path).unwrap();
        let v = ds
            .root_node
            .children
            .iter()
            .find(|n| n.name == "v")
            .unwrap();
        assert_eq!(v.sample.as_deref(), Some([42.0].as_slice()));
    }

    #[test]
    fn sample_1d_partial() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.nc");
        let data: Vec<f32> = (0..20).map(|i| i as f32).collect();
        write_test_nc(&path, &[20], &data);
        let ds = DataReader::read_file(&path).unwrap();
        let v = ds
            .root_node
            .children
            .iter()
            .find(|n| n.name == "v")
            .unwrap();
        let sample = v.sample.as_ref().expect("1D should have sample");
        assert_eq!(sample.len(), 6);
        assert_eq!(sample[0], 0.0);
        assert_eq!(sample[5], 5.0);
    }

    #[test]
    fn sample_2d_partial() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.nc");
        let data: Vec<f32> = (0..50).map(|i| i as f32).collect();
        write_test_nc(&path, &[5, 10], &data);
        let ds = DataReader::read_file(&path).unwrap();
        let v = ds
            .root_node
            .children
            .iter()
            .find(|n| n.name == "v")
            .unwrap();
        let sample = v.sample.as_ref().expect("2D should have sample");
        // shape=[5,10]: reads 3 rows × 4 cols = 12 values, row-major
        // row 0: 0,1,2,3  row 1: 10,11,12,13  row 2: 20,21,22,23
        assert_eq!(sample.len(), 12);
        assert_eq!(sample[0], 0.0); // [0,0]
        assert_eq!(sample[3], 3.0); // [0,3]
        assert_eq!(sample[4], 10.0); // [1,0]
        assert_eq!(sample[11], 23.0); // [2,3]
    }

    #[test]
    fn sample_3d_partial() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.nc");
        let data: Vec<f32> = (0..24).map(|i| i as f32).collect();
        write_test_nc(&path, &[2, 3, 4], &data);
        let ds = DataReader::read_file(&path).unwrap();
        let v = ds
            .root_node
            .children
            .iter()
            .find(|n| n.name == "v")
            .unwrap();
        let sample = v.sample.as_ref().expect("3D should have sample");
        // shape[2]=4 < 6, so get all 4 values from first element of other dims
        assert_eq!(sample.len(), 4);
        assert_eq!(sample[0], 0.0);
        assert_eq!(sample[3], 3.0);
    }
}
