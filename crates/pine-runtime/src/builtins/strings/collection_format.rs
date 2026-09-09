use super::{format_number_with_mintick, stringify_array_element};
use crate::PineValue;

pub(super) fn stringify_array_with_mintick(
    values: &[PineValue],
    format: &str,
    mintick: f64,
) -> String {
    let mut result = String::from("[");
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            result.push_str(", ");
        }
        result.push_str(&match value {
            PineValue::Int(v) => format_number_with_mintick(*v as f64, format, mintick),
            PineValue::Float(v) => format_number_with_mintick(*v, format, mintick),
            _ => stringify_array_element(value, format),
        });
    }
    result.push(']');
    result
}

pub(super) fn stringify_matrix_with_mintick(
    values: &[PineValue],
    rows: usize,
    columns: usize,
    format: &str,
    mintick: f64,
) -> String {
    let mut result = String::from("[");
    for row in 0..rows {
        if row > 0 {
            result.push_str(", ");
        }
        let start = row.saturating_mul(columns);
        let end = start.saturating_add(columns);
        let Some(row_values) = values.get(start..end) else {
            return "NaN".to_owned();
        };
        result.push_str(&stringify_array_with_mintick(row_values, format, mintick));
    }
    result.push(']');
    result
}
