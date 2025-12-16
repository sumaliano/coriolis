//! Variable data reading and manipulation.

use crate::error::{CoriolisError, Result};
use netcdf::types::{FloatType, IntType, NcVariableType};
use std::path::Path;

/// Numeric data that can be visualized.
#[derive(Debug, Clone)]
pub enum VariableData {
    /// 64-bit floating point data.
    F64(Vec<f64>),
    /// 32-bit floating point data.
    F32(Vec<f32>),
    /// 64-bit integer data.
    I64(Vec<i64>),
    /// 32-bit integer data.
    I32(Vec<i32>),
    /// 16-bit integer data.
    I16(Vec<i16>),
    /// 8-bit integer data.
    I8(Vec<i8>),
    /// Unsigned 64-bit integer data.
    U64(Vec<u64>),
    /// Unsigned 32-bit integer data.
    U32(Vec<u32>),
    /// Unsigned 16-bit integer data.
    U16(Vec<u16>),
    /// Unsigned 8-bit integer data.
    U8(Vec<u8>),
}

#[allow(dead_code)]
impl VariableData {
    /// Convert all data to f64 for visualization.
    pub fn to_f64(&self) -> Vec<f64> {
        match self {
            VariableData::F64(v) => v.clone(),
            VariableData::F32(v) => v.iter().map(|&x| x as f64).collect(),
            VariableData::I64(v) => v.iter().map(|&x| x as f64).collect(),
            VariableData::I32(v) => v.iter().map(|&x| x as f64).collect(),
            VariableData::I16(v) => v.iter().map(|&x| x as f64).collect(),
            VariableData::I8(v) => v.iter().map(|&x| x as f64).collect(),
            VariableData::U64(v) => v.iter().map(|&x| x as f64).collect(),
            VariableData::U32(v) => v.iter().map(|&x| x as f64).collect(),
            VariableData::U16(v) => v.iter().map(|&x| x as f64).collect(),
            VariableData::U8(v) => v.iter().map(|&x| x as f64).collect(),
        }
    }

    /// Get the length of the data.
    pub fn len(&self) -> usize {
        match self {
            VariableData::F64(v) => v.len(),
            VariableData::F32(v) => v.len(),
            VariableData::I64(v) => v.len(),
            VariableData::I32(v) => v.len(),
            VariableData::I16(v) => v.len(),
            VariableData::I8(v) => v.len(),
            VariableData::U64(v) => v.len(),
            VariableData::U32(v) => v.len(),
            VariableData::U16(v) => v.len(),
            VariableData::U8(v) => v.len(),
        }
    }

    /// Check if data is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get min and max values.
    pub fn min_max(&self) -> Option<(f64, f64)> {
        let data = self.to_f64();
        if data.is_empty() {
            return None;
        }

        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;

        for &v in &data {
            if v.is_finite() {
                if v < min {
                    min = v;
                }
                if v > max {
                    max = v;
                }
            }
        }

        if min.is_finite() && max.is_finite() {
            Some((min, max))
        } else {
            None
        }
    }
}

/// Loaded variable with its data and metadata.
#[derive(Debug, Clone)]
pub struct LoadedVariable {
    /// Variable name.
    pub name: String,
    /// Variable path.
    #[allow(dead_code)]
    pub path: String,
    /// Shape of the data.
    pub shape: Vec<usize>,
    /// Dimension names.
    pub dim_names: Vec<String>,
    /// The actual data (flattened).
    pub data: VariableData,
}

#[allow(dead_code)]
impl LoadedVariable {
    /// Get the number of dimensions.
    pub fn ndim(&self) -> usize {
        self.shape.len()
    }

    /// Get total number of elements.
    pub fn total_elements(&self) -> usize {
        self.shape.iter().product()
    }

    /// Convert linear index to multi-dimensional indices.
    pub fn linear_to_indices(&self, linear: usize) -> Vec<usize> {
        let mut indices = vec![0; self.shape.len()];
        let mut remaining = linear;

        for i in (0..self.shape.len()).rev() {
            indices[i] = remaining % self.shape[i];
            remaining /= self.shape[i];
        }

        indices
    }

    /// Convert multi-dimensional indices to linear index.
    pub fn indices_to_linear(&self, indices: &[usize]) -> usize {
        let mut linear = 0;
        let mut stride = 1;

        for i in (0..self.shape.len()).rev() {
            linear += indices[i] * stride;
            stride *= self.shape[i];
        }

        linear
    }

    /// Get a 1D slice along a dimension, fixing all other dimensions.
    pub fn get_1d_slice(&self, dim: usize, fixed_indices: &[usize]) -> Vec<f64> {
        let data = self.data.to_f64();
        let mut result = Vec::with_capacity(self.shape[dim]);

        for i in 0..self.shape[dim] {
            let mut indices = fixed_indices.to_vec();
            indices[dim] = i;
            let linear = self.indices_to_linear(&indices);
            if linear < data.len() {
                result.push(data[linear]);
            }
        }

        result
    }

    /// Get a 2D slice, fixing all dimensions except two.
    pub fn get_2d_slice(&self, dim1: usize, dim2: usize, fixed_indices: &[usize]) -> Vec<Vec<f64>> {
        let data = self.data.to_f64();
        let mut result = Vec::with_capacity(self.shape[dim1]);

        for i in 0..self.shape[dim1] {
            let mut row = Vec::with_capacity(self.shape[dim2]);
            for j in 0..self.shape[dim2] {
                let mut indices = fixed_indices.to_vec();
                indices[dim1] = i;
                indices[dim2] = j;
                let linear = self.indices_to_linear(&indices);
                if linear < data.len() {
                    row.push(data[linear]);
                }
            }
            result.push(row);
        }

        result
    }

    /// Get value at given indices.
    pub fn get_value(&self, indices: &[usize]) -> Option<f64> {
        let linear = self.indices_to_linear(indices);
        let data = self.data.to_f64();
        data.get(linear).copied()
    }
}

/// Read variable data from a NetCDF file.
pub fn read_variable(file_path: &Path, var_path: &str) -> Result<LoadedVariable> {
    let file = netcdf::open(file_path)
        .map_err(|e| CoriolisError::NetCDF(format!("Failed to open file: {}", e)))?;

    // Extract variable name from path (e.g., "/variables/temperature" -> "temperature")
    let var_name = var_path
        .rsplit('/')
        .next()
        .ok_or_else(|| CoriolisError::NetCDF("Invalid variable path".to_string()))?;

    let var = file
        .variable(var_name)
        .ok_or_else(|| CoriolisError::NetCDF(format!("Variable '{}' not found", var_name)))?;

    let shape: Vec<usize> = var.dimensions().iter().map(|d| d.len()).collect();
    let dim_names: Vec<String> = var.dimensions().iter().map(|d| d.name().to_string()).collect();

    // Read the data based on type
    let data = read_variable_data(&var)?;

    Ok(LoadedVariable {
        name: var_name.to_string(),
        path: var_path.to_string(),
        shape,
        dim_names,
        data,
    })
}

fn read_variable_data(var: &netcdf::Variable<'_>) -> Result<VariableData> {
    let vartype = var.vartype();

    match vartype {
        NcVariableType::Float(FloatType::F64) => {
            let values: Vec<f64> = var
                .get_values(..)
                .map_err(|e| CoriolisError::NetCDF(format!("Failed to read f64 data: {}", e)))?;
            Ok(VariableData::F64(values))
        }
        NcVariableType::Float(FloatType::F32) => {
            let values: Vec<f32> = var
                .get_values(..)
                .map_err(|e| CoriolisError::NetCDF(format!("Failed to read f32 data: {}", e)))?;
            Ok(VariableData::F32(values))
        }
        NcVariableType::Int(IntType::I64) => {
            let values: Vec<i64> = var
                .get_values(..)
                .map_err(|e| CoriolisError::NetCDF(format!("Failed to read i64 data: {}", e)))?;
            Ok(VariableData::I64(values))
        }
        NcVariableType::Int(IntType::I32) => {
            let values: Vec<i32> = var
                .get_values(..)
                .map_err(|e| CoriolisError::NetCDF(format!("Failed to read i32 data: {}", e)))?;
            Ok(VariableData::I32(values))
        }
        NcVariableType::Int(IntType::I16) => {
            let values: Vec<i16> = var
                .get_values(..)
                .map_err(|e| CoriolisError::NetCDF(format!("Failed to read i16 data: {}", e)))?;
            Ok(VariableData::I16(values))
        }
        NcVariableType::Int(IntType::I8) => {
            let values: Vec<i8> = var
                .get_values(..)
                .map_err(|e| CoriolisError::NetCDF(format!("Failed to read i8 data: {}", e)))?;
            Ok(VariableData::I8(values))
        }
        NcVariableType::Int(IntType::U64) => {
            let values: Vec<u64> = var
                .get_values(..)
                .map_err(|e| CoriolisError::NetCDF(format!("Failed to read u64 data: {}", e)))?;
            Ok(VariableData::U64(values))
        }
        NcVariableType::Int(IntType::U32) => {
            let values: Vec<u32> = var
                .get_values(..)
                .map_err(|e| CoriolisError::NetCDF(format!("Failed to read u32 data: {}", e)))?;
            Ok(VariableData::U32(values))
        }
        NcVariableType::Int(IntType::U16) => {
            let values: Vec<u16> = var
                .get_values(..)
                .map_err(|e| CoriolisError::NetCDF(format!("Failed to read u16 data: {}", e)))?;
            Ok(VariableData::U16(values))
        }
        NcVariableType::Int(IntType::U8) => {
            let values: Vec<u8> = var
                .get_values(..)
                .map_err(|e| CoriolisError::NetCDF(format!("Failed to read u8 data: {}", e)))?;
            Ok(VariableData::U8(values))
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
