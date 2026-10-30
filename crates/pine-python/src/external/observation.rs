//! Allocation-light decoding of the ordinary, already materialized tick shape.
//!
//! This is deliberately a subset of the JSON contract. Anything outside exact
//! built-in containers and losslessly representable scalars uses the existing
//! JSON/serde decoder, preserving its admission and error behavior.
use pine_runtime::{
    Bar, ExternalAccountFrame, ExternalBarState, ExternalEvaluationTrigger, ExternalPass,
    ExternalPassContext, ExternalScheduledPass,
};
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyFloat, PyInt, PyList, PyString};

fn dictionary<'py>(value: &Bound<'py, PyAny>, fields: &[&str]) -> Option<Bound<'py, PyDict>> {
    if !value.is_exact_instance_of::<PyDict>() {
        return None;
    }
    let dict = value.cast::<PyDict>().ok()?;
    if dict.len() > fields.len() {
        return None;
    }
    for (key, _) in dict.iter() {
        if !key.is_exact_instance_of::<PyString>()
            || !fields.contains(&key.cast::<PyString>().ok()?.to_str().ok()?)
        {
            return None;
        }
    }
    Some(dict.clone())
}

fn item<'py>(dict: &Bound<'py, PyDict>, key: &str) -> Option<Bound<'py, PyAny>> {
    dict.get_item(key).ok()?
}

fn integer(value: &Bound<'_, PyAny>) -> Option<i64> {
    value.is_exact_instance_of::<PyInt>().then_some(())?;
    value.extract().ok()
}

fn number(value: &Bound<'_, PyAny>) -> Option<f64> {
    if value.is_exact_instance_of::<PyFloat>() {
        let value: f64 = value.extract().ok()?;
        return value.is_finite().then_some(value);
    }
    let value = integer(value)?;
    // Larger integers retain the established serde decimal-to-float behavior.
    (value.unsigned_abs() <= (1_u64 << 53)).then_some(value as f64)
}

fn boolean(value: &Bound<'_, PyAny>) -> Option<bool> {
    value.is_exact_instance_of::<PyBool>().then_some(())?;
    value.extract().ok()
}

fn empty_updates<T>(dict: &Bound<'_, PyDict>, key: &str) -> Option<Option<Vec<T>>> {
    let Some(value) = dict.get_item(key).ok()? else {
        return Some(None);
    };
    if value.is_none() {
        return Some(None);
    }
    if value.is_exact_instance_of::<PyList>() && value.cast::<PyList>().ok()?.is_empty() {
        return Some(Some(Vec::new()));
    }
    None
}

pub(super) fn decode(value: &Bound<'_, PyAny>) -> Option<ExternalScheduledPass> {
    let root = dictionary(value, &["evaluation", "context"])?;
    let evaluation = dictionary(
        &item(&root, "evaluation")?,
        &[
            "account",
            "bar",
            "event_time_ms",
            "confirmed",
            "request_data",
            "request_updates",
        ],
    )?;
    let context = dictionary(
        &item(&root, "context")?,
        &["bar_state", "trigger", "is_new"],
    )?;
    let bar = dictionary(
        &item(&evaluation, "bar")?,
        &["time", "open", "high", "low", "close", "volume"],
    )?;
    let account = dictionary(
        &item(&evaluation, "account")?,
        &[
            "time",
            "position_size",
            "position_avg_price",
            "equity",
            "initial_capital",
            "netprofit",
            "openprofit",
        ],
    )?;
    let state = item(&context, "bar_state")?;
    let trigger = item(&context, "trigger")?;
    if !state.is_exact_instance_of::<PyString>() || !trigger.is_exact_instance_of::<PyString>() {
        return None;
    }
    let bar_state = match state.cast::<PyString>().ok()?.to_str().ok()? {
        "historical" => ExternalBarState::Historical,
        "realtime" => ExternalBarState::Realtime,
        _ => return None,
    };
    let trigger = match trigger.cast::<PyString>().ok()?.to_str().ok()? {
        "bar_close" => ExternalEvaluationTrigger::BarClose,
        "order_fill" => ExternalEvaluationTrigger::OrderFill,
        "closing_fill" => ExternalEvaluationTrigger::ClosingFill,
        "realtime_tick" => ExternalEvaluationTrigger::RealtimeTick,
        "historical_tick" => ExternalEvaluationTrigger::HistoricalTick,
        _ => return None,
    };
    let average = item(&account, "position_avg_price")?;
    Some(ExternalScheduledPass {
        evaluation: ExternalPass {
            account: ExternalAccountFrame {
                time: integer(&item(&account, "time")?)?,
                position_size: number(&item(&account, "position_size")?)?,
                position_avg_price: if average.is_none() {
                    None
                } else {
                    Some(number(&average)?)
                },
                equity: number(&item(&account, "equity")?)?,
                initial_capital: number(&item(&account, "initial_capital")?)?,
                netprofit: number(&item(&account, "netprofit")?)?,
                openprofit: number(&item(&account, "openprofit")?)?,
            },
            bar: Bar {
                time: integer(&item(&bar, "time")?)?,
                open: number(&item(&bar, "open")?)?,
                high: number(&item(&bar, "high")?)?,
                low: number(&item(&bar, "low")?)?,
                close: number(&item(&bar, "close")?)?,
                volume: number(&item(&bar, "volume")?)?,
            },
            event_time_ms: integer(&item(&evaluation, "event_time_ms")?)?,
            confirmed: boolean(&item(&evaluation, "confirmed")?)?,
            request_data: empty_updates(&evaluation, "request_data")?,
            request_updates: empty_updates(&evaluation, "request_updates")?,
        },
        context: ExternalPassContext {
            bar_state,
            trigger,
            is_new: boolean(&item(&context, "is_new")?)?,
        },
    })
}
