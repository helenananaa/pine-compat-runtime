use crate::{PineValue, RuntimeError};

/// Pine uses single-character strings; drawing output retains the established
/// host-neutral renderer vocabulary independently of those language values.
pub(super) fn normalize_extend(value: PineValue, callee: &str) -> Result<PineValue, RuntimeError> {
    let name = match &value {
        PineValue::String(value) => match value.as_str() {
            "n" => "extend.none",
            "r" => "extend.right",
            "l" => "extend.left",
            "b" => "extend.both",
            _ => {
                return Err(RuntimeError {
                    message: format!("{callee} invalid extend value {value:?}"),
                });
            }
        },
        _ => {
            return Err(RuntimeError {
                message: format!("{callee} invalid extend value {value:?}"),
            });
        }
    };
    Ok(PineValue::String(name.to_owned()))
}
