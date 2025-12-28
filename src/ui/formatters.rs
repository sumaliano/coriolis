//! Shared formatting utilities for UI components.

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
