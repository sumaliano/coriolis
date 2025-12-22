//! Variable data reading and manipulation.

use crate::error::{CoriolisError, Result};
use ndarray::{ArrayD, IxDyn};
use netcdf::types::{FloatType, IntType, NcVariableType};
use std::path::Path;

// Previous `VariableData` enum removed: we now load directly into `ArrayD<f64>`.

/// Loaded variable with its data and metadata.
#[derive(Debug, Clone)]
pub struct LoadedVariable {
    /// Variable name.
    pub name: String,
    /// Variable path.
    #[allow(dead_code)]
    pub path: String,
    /// Shape of the data (redundant with data.shape(), but kept for convenience).
    pub shape: Vec<usize>,
    /// Dimension names.
    pub dim_names: Vec<String>,
    /// Variable attributes.
    #[allow(dead_code)]
    pub attributes: std::collections::HashMap<String, String>,
    /// Variable data type.
    pub dtype: String,
    /// The actual multi-dimensional data as f64.
    /// This is an N-dimensional array that preserves the structure of the NetCDF variable.
    pub data: ArrayD<f64>,
    /// Minimum and maximum values (pre-computed for performance).
    pub min_max: Option<(f64, f64)>,
    /// Mean value (pre-computed for performance).
    pub mean: Option<f64>,
    /// Standard deviation (pre-computed for performance).
    pub std: Option<f64>,
    /// Count of valid (non-NaN) values.
    pub valid_count: usize,
}

#[allow(dead_code)]
impl LoadedVariable {
    /// Get the number of dimensions.
    pub fn ndim(&self) -> usize {
        self.data.ndim()
    }

    /// Get total number of elements.
    pub fn total_elements(&self) -> usize {
        self.data.len()
    }

    /// Get minimum and maximum values.
    pub fn min_max(&self) -> Option<(f64, f64)> {
        self.min_max
    }

    /// Get mean value.
    pub fn mean_value(&self) -> Option<f64> {
        self.mean
    }

    /// Get standard deviation.
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
    ///
    /// # Returns
    /// A 1D vector of values along the specified dimension.
    pub fn get_1d_slice(&self, dim: usize, fixed_indices: &[usize]) -> Vec<f64> {
        // Use direct IxDyn indexing to avoid borrow/shape juggling.
        let mut result = Vec::with_capacity(self.shape[dim]);
        for i in 0..self.shape[dim] {
            let mut idx = fixed_indices.to_vec();
            idx[dim] = i;
            if let Some(&val) = self.data.get(IxDyn(&idx)) {
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
    ///
    /// # Returns
    /// A 2D vector where `result[y][x]` corresponds to data where dim_y=y and dim_x=x.
    /// This ensures correct visual mapping: row index → Y dimension, col index → X dimension.
    pub fn get_2d_slice(&self, dim_y: usize, dim_x: usize, fixed_indices: &[usize]) -> Vec<Vec<f64>> {
        let mut result = Vec::with_capacity(self.shape[dim_y]);

        for y in 0..self.shape[dim_y] {
            let mut row = Vec::with_capacity(self.shape[dim_x]);
            for x in 0..self.shape[dim_x] {
                let mut idx = fixed_indices.to_vec();
                idx[dim_y] = y;
                idx[dim_x] = x;

                // Use ndarray's indexing - it handles all the complexity!
                if let Some(&val) = self.data.get(IxDyn(&idx)) {
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

    let var = file
        .variable(netcdf_path)
        .ok_or_else(|| CoriolisError::NetCDF(format!("Variable '{}' not found at path '{}'", var_name, netcdf_path)))?;

    let shape: Vec<usize> = var.dimensions().iter().map(|d| d.len()).collect();
    let dim_names: Vec<String> = var.dimensions().iter().map(|d| d.name().to_string()).collect();

    // Read attributes
    let mut attributes = std::collections::HashMap::new();
    for attr in var.attributes() {
        attributes.insert(
            attr.name().to_string(),
            crate::data::reader::DataReader::attr_value_to_string(&attr),
        );
    }

    // Extract scale_factor and add_offset (CF convention)
    let scale_factor = attributes.get("scale_factor")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(1.0);

    let add_offset = attributes.get("add_offset")
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);

    // Get data type
    let dtype = format!("{:?}", var.vartype()).replace("NcVariableType::", "").to_lowercase();

    // Read the data into f64 array
    let mut data = read_variable_array(&var, &shape)?;

    // Apply CF scale/offset if present
    if (scale_factor - 1.0).abs() > 0.0 || add_offset != 0.0 {
        data.mapv_inplace(|v| v * scale_factor + add_offset);
    }

    // Compute statistics
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    let mut sum = 0.0f64;
    let mut count = 0usize;
    for &v in data.iter() {
        if v.is_finite() {
            if v < min { min = v; }
            if v > max { max = v; }
            sum += v;
            count += 1;
        }
    }
    let min_max = if count > 0 { Some((min, max)) } else { None };
    let mean = if count > 0 { Some(sum / count as f64) } else { None };
    let std = if count > 1 {
        let mean_val = mean.unwrap();
        let mut ssd = 0.0;
        for &v in data.iter() {
            if v.is_finite() {
                let d = v - mean_val;
                ssd += d * d;
            }
        }
        Some((ssd / (count - 1) as f64).sqrt())
    } else { None };
    let valid_count = count;

    Ok(LoadedVariable {
        name: var_name.to_string(),
        path: var_path.to_string(),
        shape,
        dim_names,
        attributes,
        dtype,
        data,
        min_max,
        mean,
        std,
        valid_count,
    })
}

fn read_variable_array(var: &netcdf::Variable<'_>, shape: &Vec<usize>) -> Result<ArrayD<f64>> {
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
        }
        NcVariableType::Float(FloatType::F32) => {
            let values: Vec<f32> = var
                .get_values(..)
                .map_err(|e| CoriolisError::NetCDF(format!("Failed to read f32 data: {}", e)))?;
            from_vec(values.into_iter().map(|x| x as f64).collect())
        }
        NcVariableType::Int(IntType::I64) => {
            let values: Vec<i64> = var
                .get_values(..)
                .map_err(|e| CoriolisError::NetCDF(format!("Failed to read i64 data: {}", e)))?;
            from_vec(values.into_iter().map(|x| x as f64).collect())
        }
        NcVariableType::Int(IntType::I32) => {
            let values: Vec<i32> = var
                .get_values(..)
                .map_err(|e| CoriolisError::NetCDF(format!("Failed to read i32 data: {}", e)))?;
            from_vec(values.into_iter().map(|x| x as f64).collect())
        }
        NcVariableType::Int(IntType::I16) => {
            let values: Vec<i16> = var
                .get_values(..)
                .map_err(|e| CoriolisError::NetCDF(format!("Failed to read i16 data: {}", e)))?;
            from_vec(values.into_iter().map(|x| x as f64).collect())
        }
        NcVariableType::Int(IntType::I8) => {
            let values: Vec<i8> = var
                .get_values(..)
                .map_err(|e| CoriolisError::NetCDF(format!("Failed to read i8 data: {}", e)))?;
            from_vec(values.into_iter().map(|x| x as f64).collect())
        }
        NcVariableType::Int(IntType::U64) => {
            let values: Vec<u64> = var
                .get_values(..)
                .map_err(|e| CoriolisError::NetCDF(format!("Failed to read u64 data: {}", e)))?;
            from_vec(values.into_iter().map(|x| x as f64).collect())
        }
        NcVariableType::Int(IntType::U32) => {
            let values: Vec<u32> = var
                .get_values(..)
                .map_err(|e| CoriolisError::NetCDF(format!("Failed to read u32 data: {}", e)))?;
            from_vec(values.into_iter().map(|x| x as f64).collect())
        }
        NcVariableType::Int(IntType::U16) => {
            let values: Vec<u16> = var
                .get_values(..)
                .map_err(|e| CoriolisError::NetCDF(format!("Failed to read u16 data: {}", e)))?;
            from_vec(values.into_iter().map(|x| x as f64).collect())
        }
        NcVariableType::Int(IntType::U8) => {
            let values: Vec<u8> = var
                .get_values(..)
                .map_err(|e| CoriolisError::NetCDF(format!("Failed to read u8 data: {}", e)))?;
            from_vec(values.into_iter().map(|x| x as f64).collect())
        }
        NcVariableType::Char | NcVariableType::String => {
            Err(CoriolisError::NetCDF(
                "Character/string data cannot be visualized".to_string(),
            ))
        }
        _ => Err(CoriolisError::NetCDF(format!(
            "Unsupported variable type: {:?}",
            vartype
        ))),
    }
}
