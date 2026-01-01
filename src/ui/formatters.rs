//! Shared formatting utilities for UI components.

/// Clean a NetCDF data type string for display.
/// Removes "NcVariableType::" prefix and lowercases.
pub fn clean_dtype(dtype: &str) -> String {
    dtype.replace("NcVariableType::", "").to_lowercase()
}

/// Parse dimension string and shape into (name, size) pairs.
/// Dimension string format: "dim1, dim2, dim3"
pub fn parse_dimensions<'a>(dim_str: &'a str, shape: &[usize]) -> Vec<(&'a str, usize)> {
    if dim_str.is_empty() {
        return Vec::new();
    }
    dim_str
        .split(", ")
        .zip(shape.iter().copied())
        .collect()
}

/// Format dimensions as "dim1=size1, dim2=size2".
pub fn format_dimensions(dim_str: &str, shape: &[usize]) -> String {
    parse_dimensions(dim_str, shape)
        .iter()
        .map(|(name, size)| format!("{}={}", name, size))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Format a number with thousand separators.
pub fn format_number(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

/// Format a statistic value with smart precision.
pub fn format_stat_value(val: f64) -> String {
    if !val.is_finite() {
        return if val.is_nan() {
            "NaN".to_string()
        } else if val.is_sign_positive() {
            "+Inf".to_string()
        } else {
            "-Inf".to_string()
        };
    }
    let abs_val = val.abs();
    if abs_val == 0.0 {
        "0".to_string()
    } else if !(1e-3..1e6).contains(&abs_val) {
        format!("{:.3e}", val)
    } else if abs_val >= 100.0 {
        format!("{:.2}", val)
    } else if abs_val >= 1.0 {
        format!("{:.4}", val)
    } else {
        format!("{:.5}", val)
    }
}
