use std::cmp::Ordering;

use crate::{PineValue, RuntimeError};

/// Keep integral counters exact, while allowing fractional starts and bounds.
#[derive(Clone, Copy)]
pub(super) enum LoopNumber {
    Int(i64),
    Float(f64),
}

impl LoopNumber {
    pub(super) fn from_value(value: PineValue) -> Option<Self> {
        match value {
            PineValue::Int(value) => Some(Self::Int(value)),
            PineValue::Float(value) if value.is_finite() => Some(Self::Float(value)),
            _ => None,
        }
    }

    pub(super) fn compare(self, other: Self) -> Ordering {
        match (self, other) {
            (Self::Int(a), Self::Int(b)) => a.cmp(&b),
            (Self::Float(a), Self::Float(b)) => {
                if pine_ir::pine_numeric_comparison(pine_ir::HirBinaryOp::Eq, a, b) == Some(true) {
                    Ordering::Equal
                } else {
                    a.partial_cmp(&b).expect("finite loop values")
                }
            }
            (Self::Int(a), Self::Float(b)) => compare_int_float(a, b),
            (Self::Float(a), Self::Int(b)) => compare_int_float(b, a).reverse(),
        }
    }

    pub(super) fn step(self, step: Option<Self>, ascending: bool) -> Result<Self, RuntimeError> {
        let invalid = |message: &str| RuntimeError {
            message: message.to_owned(),
        };
        match (self, step.unwrap_or(Self::Int(1))) {
            (_, Self::Int(0) | Self::Float(0.0)) => Err(invalid("for loop step cannot be zero")),
            (_, Self::Int(value)) if value < 0 => {
                Err(invalid("for loop step must be greater than zero"))
            }
            (_, Self::Float(value)) if value < 0.0 => {
                Err(invalid("for loop step must be greater than zero"))
            }
            (Self::Int(_), Self::Int(value)) => {
                Ok(Self::Int(if ascending { value } else { -value }))
            }
            (Self::Int(_), Self::Float(_)) => {
                Err(invalid("for loop with an int counter requires an int step"))
            }
            (Self::Float(_), value) => {
                let magnitude = match value {
                    Self::Int(value) => value as f64,
                    Self::Float(value) => value,
                };
                Ok(Self::Float(if ascending { magnitude } else { -magnitude }))
            }
        }
    }

    pub(super) fn advance(self, step: Self) -> Option<Self> {
        match (self, step) {
            (Self::Int(value), Self::Int(step)) => value.checked_add(step).map(Self::Int),
            (Self::Float(value), Self::Float(step)) => {
                let next = value + step;
                next.is_finite().then_some(Self::Float(next))
            }
            _ => unreachable!("counter and step types agree"),
        }
    }

    pub(super) fn into_value(self) -> PineValue {
        match self {
            Self::Int(value) => PineValue::Int(value),
            Self::Float(value) => PineValue::Float(value),
        }
    }
}

fn compare_int_float(integer: i64, float: f64) -> Ordering {
    // Converting an i64 counter to f64 would lose its low bits near 2^63.
    if float >= 9_223_372_036_854_775_808.0 {
        return Ordering::Less;
    }
    if float < -9_223_372_036_854_775_808.0 {
        return Ordering::Greater;
    }
    if integer.checked_sub(float as i64).is_some_and(|difference| {
        pine_ir::pine_numeric_comparison(pine_ir::HirBinaryOp::Eq, difference as f64, float.fract())
            == Some(true)
    }) {
        return Ordering::Equal;
    }
    match integer.cmp(&(float as i64)) {
        Ordering::Equal => 0.0_f64.partial_cmp(&float.fract()).expect("finite bound"),
        ordering => ordering,
    }
}
