//! Variable data reading and manipulation.

use crate::error::{CoriolisError, Result};
use crate::util::formatters::clean_dtype;
use ndarray::{ArrayD, IxDyn};
use netcdf::types::{FloatType, IntType, NcVariableType};
use std::path::Path;

// Previous `VariableData` enum removed: we now load directly into `ArrayD<f64>`.

/// Coordinate variable data for a dimension.
#[derive(Debug, Clone)]
pub struct CoordinateVar {
    /// Coordinate values (1D array).
    pub values: Vec<f64>,
    /// Units attribute (e.g., "degrees_north", "seconds since 1970-01-01").
    pub units: Option<String>,
    /// Long name attribute.
    pub long_name: Option<String>,
}

impl CoordinateVar {
    /// Get formatted value at index with appropriate precision.
    pub fn format_value(&self, index: usize) -> String {
        if let Some(val) = self.values.get(index) {
            if !val.is_finite() {
                return "N/A".to_string();
            }
            // Smart formatting based on value magnitude
            let abs_val = val.abs();
            if abs_val == 0.0 {
                "0".to_string()
            } else if !(0.01..1000.0).contains(&abs_val) {
                format!("{:.2e}", val)
            } else if abs_val >= 10.0 {
                format!("{:.1}", val)
            } else if abs_val >= 1.0 {
                format!("{:.2}", val)
            } else {
                format!("{:.3}", val)
            }
        } else {
            "?".to_string()
        }
    }

    /// Get short label for axis display (value + optional unit suffix).
    pub fn axis_label(&self, index: usize) -> String {
        let val_str = self.format_value(index);
        if let Some(ref units) = self.units {
            // Use abbreviated unit suffixes for common cases
            let short_unit = match units.as_str() {
                u if u.contains("degrees_north") || u.contains("degree_north") => "°N",
                u if u.contains("degrees_south") || u.contains("degree_south") => "°S",
                u if u.contains("degrees_east") || u.contains("degree_east") => "°E",
                u if u.contains("degrees_west") || u.contains("degree_west") => "°W",
                u if u.contains("degrees") || u.contains("degree") => "°",
                u if u.starts_with("seconds since") || u.starts_with("days since") => "",
                _ => "",
            };
            if short_unit.is_empty() {
                val_str
            } else {
                format!("{}{}", val_str, short_unit)
            }
        } else {
            val_str
        }
    }
}

/// Loaded variable with its data and metadata.
#[derive(Debug, Clone)]
pub struct LoadedVariable {
    /// Variable name.
    pub name: String,
    /// Shape of the data (redundant with data.shape(), but kept for convenience).
    pub shape: Vec<usize>,
    /// Dimension names.
    pub dim_names: Vec<String>,
    /// Variable attributes.
    pub attributes: std::collections::HashMap<String, String>,
    /// Variable data type.
    pub dtype: String,
    /// The actual multi-dimensional data as f64 (RAW, unscaled).
    /// This is an N-dimensional array that preserves the structure of the NetCDF variable.
    pub data: ArrayD<f64>,
    /// CF convention scale factor (default 1.0).
    pub scale_factor: f64,
    /// CF convention add offset (default 0.0).
    pub add_offset: f64,
    /// Minimum and maximum values of SCALED data (pre-computed for performance).
    pub min_max: Option<(f64, f64)>,
    /// Mean value of SCALED data (pre-computed for performance).
    pub mean: Option<f64>,
    /// Standard deviation of SCALED data (pre-computed for performance).
    pub std: Option<f64>,
    /// Count of valid (non-NaN) values.
    pub valid_count: usize,
    /// Coordinate variables for each dimension (if found).
    /// Index corresponds to dimension index.
    pub coordinates: Vec<Option<CoordinateVar>>,
}

impl LoadedVariable {
    /// Get the number of dimensions.
    pub fn ndim(&self) -> usize {
        self.data.ndim()
    }

    /// Get total number of elements.
    pub fn total_elements(&self) -> usize {
        self.data.len()
    }

    /// Check if this variable has scale/offset transformation.
    pub fn has_scale_offset(&self) -> bool {
        (self.scale_factor - 1.0).abs() > f64::EPSILON || self.add_offset.abs() > f64::EPSILON
    }

    /// Apply scale/offset to a raw value: scaled = raw * scale_factor + add_offset
    #[inline]
    pub fn scale_value(&self, raw: f64) -> f64 {
        raw * self.scale_factor + self.add_offset
    }

    /// Remove scale/offset from a scaled value: raw = (scaled - add_offset) / scale_factor
    #[inline]
    pub fn unscale_value(&self, scaled: f64) -> f64 {
        (scaled - self.add_offset) / self.scale_factor
    }

    /// Get a value, optionally applying scale/offset.
    #[inline]
    pub fn get_value_transformed(&self, indices: &[usize], apply_scale: bool) -> Option<f64> {
        let raw = self.data.get(IxDyn(indices)).copied()?;
        Some(if apply_scale {
            self.scale_value(raw)
        } else {
            raw
        })
    }

    /// Get minimum and maximum values (of scaled data).
    pub fn min_max(&self) -> Option<(f64, f64)> {
        self.min_max
    }

    /// Get mean value (of scaled data).
    pub fn mean_value(&self) -> Option<f64> {
        self.mean
    }

    /// Get standard deviation (of scaled data).
    pub fn std_value(&self) -> Option<f64> {
        self.std
    }

    /// Get count of valid values.
    pub fn valid_count(&self) -> usize {
        self.valid_count
    }

    /// Get a 1D slice along a dimension, fixing all other dimensions.
    ///
    /// # Arguments
    /// * `dim` - The dimension to extract (will vary)
    /// * `fixed_indices` - Indices for all other dimensions (fixed values)
    /// * `apply_scale` - Whether to apply scale/offset transformation
    ///
    /// # Returns
    /// A 1D vector of values along the specified dimension.
    pub fn get_1d_slice(&self, dim: usize, fixed_indices: &[usize], apply_scale: bool) -> Vec<f64> {
        let mut result = Vec::with_capacity(self.shape[dim]);

        // Reuse a single index vector to avoid allocations
        let mut idx = fixed_indices.to_vec();

        for i in 0..self.shape[dim] {
            idx[dim] = i;
            if let Some(&raw) = self.data.get(IxDyn(&idx)) {
                let val = if apply_scale {
                    self.scale_value(raw)
                } else {
                    raw
                };
                result.push(val);
            }
        }
        result
    }

    /// Get a 2D slice, fixing all dimensions except two.
    ///
    /// # Arguments
    /// * `dim_y` - The dimension for rows (Y-axis, varies in outer loop)
    /// * `dim_x` - The dimension for columns (X-axis, varies in inner loop)
    /// * `fixed_indices` - Indices for all other dimensions
    /// * `apply_scale` - Whether to apply scale/offset transformation
    ///
    /// # Returns
    /// A 2D vector where `result[y][x]` corresponds to data where dim_y=y and dim_x=x.
    /// This ensures correct visual mapping: row index → Y dimension, col index → X dimension.
    pub fn get_2d_slice(
        &self,
        dim_y: usize,
        dim_x: usize,
        fixed_indices: &[usize],
        apply_scale: bool,
    ) -> Vec<Vec<f64>> {
        let mut result = Vec::with_capacity(self.shape[dim_y]);

        // Reuse a single index vector to avoid allocations
        let mut idx = fixed_indices.to_vec();

        for y in 0..self.shape[dim_y] {
            idx[dim_y] = y;
            let mut row = Vec::with_capacity(self.shape[dim_x]);

            for x in 0..self.shape[dim_x] {
                idx[dim_x] = x;

                if let Some(&raw) = self.data.get(IxDyn(&idx)) {
                    let val = if apply_scale {
                        self.scale_value(raw)
                    } else {
                        raw
                    };
                    row.push(val);
                } else {
                    row.push(f64::NAN);
                }
            }
            result.push(row);
        }

        result
    }

    /// Get value at given multi-dimensional indices.
    pub fn get_value(&self, indices: &[usize]) -> Option<f64> {
        self.data.get(IxDyn(indices)).copied()
    }

    /// Get coordinate variable for a dimension, if available.
    pub fn get_coordinate(&self, dim: usize) -> Option<&CoordinateVar> {
        self.coordinates.get(dim).and_then(|c| c.as_ref())
    }

    /// Get coordinate value for a dimension at given index.
    pub fn get_coord_value(&self, dim: usize, index: usize) -> Option<f64> {
        self.get_coordinate(dim)
            .and_then(|c| c.values.get(index).copied())
    }

    /// Get formatted coordinate label for a dimension at given index.
    pub fn get_coord_label(&self, dim: usize, index: usize) -> String {
        if let Some(coord) = self.get_coordinate(dim) {
            coord.axis_label(index)
        } else {
            format!("{}", index)
        }
    }

    /// Get the units for this variable.
    pub fn units(&self) -> Option<&str> {
        self.attributes.get("units").map(|s| s.as_str())
    }

    /// Get the long name for this variable.
    pub fn long_name(&self) -> Option<&str> {
        self.attributes.get("long_name").map(|s| s.as_str())
    }
}

/// Read variable data from a NetCDF file.
pub fn read_variable(file_path: &Path, var_path: &str) -> Result<LoadedVariable> {
    let file = netcdf::open(file_path)
        .map_err(|e| CoriolisError::NetCDF(format!("Failed to open file: {}", e)))?;

    // Extract variable name from path
    let var_name = var_path
        .rsplit('/')
        .next()
        .ok_or_else(|| CoriolisError::NetCDF("Invalid variable path".to_string()))?;

    // For variables in groups, use the full path (without leading /)
    // e.g., "/data/View_000/latitude" -> "data/View_000/latitude"
    let netcdf_path = var_path.trim_start_matches('/');

    let var = file.variable(netcdf_path).ok_or_else(|| {
        CoriolisError::NetCDF(format!(
            "Variable '{}' not found at path '{}'",
            var_name, netcdf_path
        ))
    })?;

    let shape: Vec<usize> = var.dimensions().iter().map(|d| d.len()).collect();
    let dim_names: Vec<String> = var
        .dimensions()
        .iter()
        .map(|d| d.name().to_string())
        .collect();

    // Read attributes
    let mut attributes = std::collections::HashMap::new();
    for attr in var.attributes() {
        attributes.insert(
            attr.name().to_string(),
            crate::data::reader::DataReader::attr_value_to_string(&attr),
        );
    }

    // Extract scale_factor and add_offset (CF convention)
    let scale_factor = attributes
        .get("scale_factor")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(1.0);

    let add_offset = attributes
        .get("add_offset")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);

    // Get data type
    let dtype = clean_dtype(&format!("{:?}", var.vartype()));

    // Read the RAW data into f64 array (don't apply scale/offset here)
    let data = read_variable_array(&var, &shape)?;

    // Compute statistics on SCALED data using Welford's algorithm (single pass)
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    let mut count = 0usize;
    let mut mean_accum = 0.0f64;
    let mut m2 = 0.0f64; // For variance calculation

    for &raw in data.iter() {
        let v = raw * scale_factor + add_offset; // Apply scale for statistics
        if v.is_finite() {
            count += 1;

            // Update min/max
            if v < min {
                min = v;
            }
            if v > max {
                max = v;
            }

            // Welford's online algorithm for mean and variance
            let delta = v - mean_accum;
            mean_accum += delta / count as f64;
            let delta2 = v - mean_accum;
            m2 += delta * delta2;
        }
    }

    let min_max = if count > 0 { Some((min, max)) } else { None };
    let mean = if count > 0 { Some(mean_accum) } else { None };
    let std = if count > 1 {
        Some((m2 / (count - 1) as f64).sqrt())
    } else {
        None
    };
    let valid_count = count;

    // Try to load coordinate variables for each dimension
    // CF convention: coordinate variables have the same name as the dimension
    let coordinates = load_coordinate_variables(&file, &dim_names, var_path);

    Ok(LoadedVariable {
        name: var_name.to_string(),
        shape,
        dim_names,
        attributes,
        dtype,
        data,
        scale_factor,
        add_offset,
        min_max,
        mean,
        std,
        valid_count,
        coordinates,
    })
}

/// Load coordinate variables for the given dimension names.
/// CF convention: coordinate variables have the same name as their dimension.
fn load_coordinate_variables(
    file: &netcdf::File,
    dim_names: &[String],
    var_path: &str,
) -> Vec<Option<CoordinateVar>> {
    // Determine the group path for the variable
    let group_path = var_path
        .trim_start_matches('/')
        .rsplit_once('/')
        .map(|(p, _)| p)
        .unwrap_or("");

    dim_names
        .iter()
        .map(|dim_name| {
            // Try to find coordinate variable with same name as dimension
            // First try in the same group as the variable
            let coord_path = if group_path.is_empty() {
                dim_name.clone()
            } else {
                format!("{}/{}", group_path, dim_name)
            };

            // Try to load from same group first, then from root
            try_load_coordinate(file, &coord_path).or_else(|| try_load_coordinate(file, dim_name))
        })
        .collect()
}

/// Try to load a single coordinate variable.
fn try_load_coordinate(file: &netcdf::File, path: &str) -> Option<CoordinateVar> {
    let var = file.variable(path)?;

    // Coordinate variables must be 1D
    let dims = var.dimensions();
    if dims.len() != 1 {
        return None;
    }

    // Read the values
    let values: Vec<f64> = match var.vartype() {
        NcVariableType::Float(FloatType::F64) => var.get_values(..).ok()?,
        NcVariableType::Float(FloatType::F32) => {
            let vals: Vec<f32> = var.get_values(..).ok()?;
            vals.into_iter().map(|x| x as f64).collect()
        },
        NcVariableType::Int(IntType::I64) => {
            let vals: Vec<i64> = var.get_values(..).ok()?;
            vals.into_iter().map(|x| x as f64).collect()
        },
        NcVariableType::Int(IntType::I32) => {
            let vals: Vec<i32> = var.get_values(..).ok()?;
            vals.into_iter().map(|x| x as f64).collect()
        },
        NcVariableType::Int(IntType::I16) => {
            let vals: Vec<i16> = var.get_values(..).ok()?;
            vals.into_iter().map(|x| x as f64).collect()
        },
        NcVariableType::Int(IntType::U32) => {
            let vals: Vec<u32> = var.get_values(..).ok()?;
            vals.into_iter().map(|x| x as f64).collect()
        },
        NcVariableType::Int(IntType::U16) => {
            let vals: Vec<u16> = var.get_values(..).ok()?;
            vals.into_iter().map(|x| x as f64).collect()
        },
        _ => return None,
    };

    // Read units and long_name attributes
    let units = var.attribute("units").and_then(|a| {
        use netcdf::AttributeValue;
        match a.value().ok()? {
            AttributeValue::Str(s) => Some(s),
            _ => None,
        }
    });

    let long_name = var.attribute("long_name").and_then(|a| {
        use netcdf::AttributeValue;
        match a.value().ok()? {
            AttributeValue::Str(s) => Some(s),
            _ => None,
        }
    });

    Some(CoordinateVar {
        values,
        units,
        long_name,
    })
}

fn read_variable_array(var: &netcdf::Variable<'_>, shape: &[usize]) -> Result<ArrayD<f64>> {
    let vartype = var.vartype();

    // Helper to build ArrayD<f64> from a Vec<f64> and the known shape
    let from_vec = |v: Vec<f64>| -> Result<ArrayD<f64>> {
        ndarray::ArrayD::from_shape_vec(IxDyn(shape), v)
            .map_err(|e| CoriolisError::NetCDF(format!("Invalid shape/data size: {}", e)))
    };

    match vartype {
        NcVariableType::Float(FloatType::F64) => {
            let values: Vec<f64> = var
                .get_values(..)
                .map_err(|e| CoriolisError::NetCDF(format!("Failed to read f64 data: {}", e)))?;
            from_vec(values)
        },
        NcVariableType::Float(FloatType::F32) => {
            let values: Vec<f32> = var
                .get_values(..)
                .map_err(|e| CoriolisError::NetCDF(format!("Failed to read f32 data: {}", e)))?;
            from_vec(values.into_iter().map(|x| x as f64).collect())
        },
        NcVariableType::Int(IntType::I64) => {
            let values: Vec<i64> = var
                .get_values(..)
                .map_err(|e| CoriolisError::NetCDF(format!("Failed to read i64 data: {}", e)))?;
            from_vec(values.into_iter().map(|x| x as f64).collect())
        },
        NcVariableType::Int(IntType::I32) => {
            let values: Vec<i32> = var
                .get_values(..)
                .map_err(|e| CoriolisError::NetCDF(format!("Failed to read i32 data: {}", e)))?;
            from_vec(values.into_iter().map(|x| x as f64).collect())
        },
        NcVariableType::Int(IntType::I16) => {
            let values: Vec<i16> = var
                .get_values(..)
                .map_err(|e| CoriolisError::NetCDF(format!("Failed to read i16 data: {}", e)))?;
            from_vec(values.into_iter().map(|x| x as f64).collect())
        },
        NcVariableType::Int(IntType::I8) => {
            let values: Vec<i8> = var
                .get_values(..)
                .map_err(|e| CoriolisError::NetCDF(format!("Failed to read i8 data: {}", e)))?;
            from_vec(values.into_iter().map(|x| x as f64).collect())
        },
        NcVariableType::Int(IntType::U64) => {
            let values: Vec<u64> = var
                .get_values(..)
                .map_err(|e| CoriolisError::NetCDF(format!("Failed to read u64 data: {}", e)))?;
            from_vec(values.into_iter().map(|x| x as f64).collect())
        },
        NcVariableType::Int(IntType::U32) => {
            let values: Vec<u32> = var
                .get_values(..)
                .map_err(|e| CoriolisError::NetCDF(format!("Failed to read u32 data: {}", e)))?;
            from_vec(values.into_iter().map(|x| x as f64).collect())
        },
        NcVariableType::Int(IntType::U16) => {
            let values: Vec<u16> = var
                .get_values(..)
                .map_err(|e| CoriolisError::NetCDF(format!("Failed to read u16 data: {}", e)))?;
            from_vec(values.into_iter().map(|x| x as f64).collect())
        },
        NcVariableType::Int(IntType::U8) => {
            let values: Vec<u8> = var
                .get_values(..)
                .map_err(|e| CoriolisError::NetCDF(format!("Failed to read u8 data: {}", e)))?;
            from_vec(values.into_iter().map(|x| x as f64).collect())
        },
        NcVariableType::Char | NcVariableType::String => Err(CoriolisError::NetCDF(
            "Character/string data cannot be visualized".to_string(),
        )),
        _ => Err(CoriolisError::NetCDF(format!(
            "Unsupported variable type: {:?}",
            vartype
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::ArrayD;

    fn make_var(data: Vec<f64>, shape: Vec<usize>, dim_names: Vec<&str>) -> LoadedVariable {
        let arr = ArrayD::from_shape_vec(ndarray::IxDyn(&shape), data).unwrap();
        let ndim = shape.len();
        LoadedVariable {
            name: "test".to_string(),
            shape,
            dim_names: dim_names.into_iter().map(|s| s.to_string()).collect(),
            attributes: Default::default(),
            dtype: "float64".to_string(),
            data: arr,
            scale_factor: 1.0,
            add_offset: 0.0,
            min_max: None,
            mean: None,
            std: None,
            valid_count: 0,
            coordinates: vec![None; ndim],
        }
    }

    #[test]
    fn scalar_variable_has_one_element() {
        let var = make_var(vec![42.0], vec![1], vec!["dim"]);
        assert_eq!(var.total_elements(), 1);
    }

    #[test]
    fn get_1d_slice_extracts_correct_values() {
        // 2D variable shape [3, 4]: values 0..12
        let data: Vec<f64> = (0..12).map(|x| x as f64).collect();
        let var = make_var(data, vec![3, 4], vec!["row", "col"]);

        // Slice along dim 1 (columns), fixing row=1 → values [4,5,6,7]
        let fixed = vec![1, 0]; // row index 1, col will vary
        let slice = var.get_1d_slice(1, &fixed, false);
        assert_eq!(slice, vec![4.0, 5.0, 6.0, 7.0]);
    }

    #[test]
    fn get_2d_slice_shape_matches_display_dims() {
        let data: Vec<f64> = (0..24).map(|x| x as f64).collect();
        let var = make_var(data, vec![2, 3, 4], vec!["t", "y", "x"]);

        // Display dims: y=1, x=2; fix t=0
        let fixed = vec![0, 0, 0];
        let slice = var.get_2d_slice(1, 2, &fixed, false);
        assert_eq!(slice.len(), 3); // 3 rows (y dimension)
        assert_eq!(slice[0].len(), 4); // 4 cols (x dimension)
        assert_eq!(slice[0][0], 0.0);
        assert_eq!(slice[2][3], 11.0);
    }

    #[test]
    fn get_2d_slice_second_time_slice() {
        let data: Vec<f64> = (0..24).map(|x| x as f64).collect();
        let var = make_var(data, vec![2, 3, 4], vec!["t", "y", "x"]);

        // t=1: values start at index 12
        let fixed = vec![1, 0, 0];
        let slice = var.get_2d_slice(1, 2, &fixed, false);
        assert_eq!(slice[0][0], 12.0);
        assert_eq!(slice[2][3], 23.0);
    }

    #[test]
    fn scale_value_applies_cf_convention() {
        let var = LoadedVariable {
            scale_factor: 0.1,
            add_offset: 5.0,
            ..make_var(vec![100.0], vec![1], vec!["x"])
        };
        // scaled = raw * scale + offset = 100 * 0.1 + 5 = 15
        assert!((var.scale_value(100.0) - 15.0).abs() < 1e-10);
    }

    #[test]
    fn scale_identity_when_no_scale_offset() {
        let var = make_var(vec![3.14], vec![1], vec!["x"]);
        assert!(!var.has_scale_offset());
        assert!((var.scale_value(3.14) - 3.14).abs() < 1e-10);
    }

    #[test]
    fn get_1d_slice_applies_scale() {
        let data = vec![10.0, 20.0, 30.0];
        let var = LoadedVariable {
            scale_factor: 2.0,
            add_offset: 1.0,
            ..make_var(data, vec![3], vec!["x"])
        };
        let fixed = vec![0];
        let scaled = var.get_1d_slice(0, &fixed, true);
        assert_eq!(scaled, vec![21.0, 41.0, 61.0]);
    }

    #[test]
    fn ndim_reflects_shape_rank() {
        let var1d = make_var(vec![1.0, 2.0], vec![2], vec!["x"]);
        assert_eq!(var1d.ndim(), 1);

        let var3d = make_var(vec![0.0; 24], vec![2, 3, 4], vec!["t", "y", "x"]);
        assert_eq!(var3d.ndim(), 3);
    }

    #[test]
    fn coord_label_falls_back_to_index_when_no_coord() {
        let var = make_var(vec![1.0, 2.0, 3.0], vec![3], vec!["x"]);
        assert_eq!(var.get_coord_label(0, 2), "2");
    }
}
