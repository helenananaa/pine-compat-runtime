use serde_json::Value;

const FLOAT_ABSOLUTE_TOLERANCE: f64 = 1e-12;
const FLOAT_RELATIVE_TOLERANCE: f64 = 1e-12;

pub(crate) fn assert_json_approximately_equal(actual: &Value, expected: &Value, context: &str) {
    if let Err(message) = json_approximately_equal(actual, expected, "$") {
        panic!("{context}: {message}");
    }
}

fn json_approximately_equal(actual: &Value, expected: &Value, path: &str) -> Result<(), String> {
    match (actual, expected) {
        (Value::Number(actual), Value::Number(expected)) => {
            if actual == expected {
                return Ok(());
            }
            let actual = actual
                .as_f64()
                .ok_or_else(|| format!("{path}: actual number is not representable as f64"))?;
            let expected = expected
                .as_f64()
                .ok_or_else(|| format!("{path}: expected number is not representable as f64"))?;
            let difference = (actual - expected).abs();
            let tolerance = FLOAT_ABSOLUTE_TOLERANCE
                .max(FLOAT_RELATIVE_TOLERANCE * actual.abs().max(expected.abs()));
            if difference <= tolerance {
                Ok(())
            } else {
                Err(format!(
                    "{path}: numeric mismatch: actual={actual}, expected={expected}, difference={difference}, tolerance={tolerance}"
                ))
            }
        }
        (Value::Array(actual), Value::Array(expected)) => {
            if actual.len() != expected.len() {
                return Err(format!(
                    "{path}: array length mismatch: actual={}, expected={}",
                    actual.len(),
                    expected.len()
                ));
            }
            for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
                json_approximately_equal(actual, expected, &format!("{path}[{index}]"))?;
            }
            Ok(())
        }
        (Value::Object(actual), Value::Object(expected)) => {
            if actual.len() != expected.len() {
                return Err(format!(
                    "{path}: object field count mismatch: actual={}, expected={}",
                    actual.len(),
                    expected.len()
                ));
            }
            for (key, expected) in expected {
                let actual = actual
                    .get(key)
                    .ok_or_else(|| format!("{path}: missing field {key:?}"))?;
                json_approximately_equal(actual, expected, &format!("{path}.{key}"))?;
            }
            Ok(())
        }
        _ if actual == expected => Ok(()),
        _ => Err(format!(
            "{path}: value mismatch: actual={actual}, expected={expected}"
        )),
    }
}
