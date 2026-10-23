//! Python request streams and host-supplied chart input metadata.
use crate::{chart_metadata, parse_bar, parse_chart_context, parse_request_key};
use pine_runtime::{InMemoryRequestDataProvider, RequestEnvironment};
use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{PyAny, PyDict, PyModule},
};

pub(super) fn parse_request_environment(
    request_bars: Option<&Bound<'_, PyAny>>,
    chart_symbol: Option<&str>,
    chart_timeframe: Option<&str>,
) -> PyResult<RequestEnvironment> {
    let mut chart = parse_chart_context(chart_symbol, chart_timeframe)?;
    let Some(request_bars) = request_bars else {
        return Ok(RequestEnvironment::default().for_chart(chart));
    };
    let dict = request_bars.cast::<PyDict>().map_err(|_| {
        PyValueError::new_err("request_bars must be a dict mapping SYMBOL:TIMEFRAME to bars")
    })?;
    let mut streams = Vec::with_capacity(dict.len());
    let mut calendars = None;
    let mut currency_rates = None;
    let mut symbol_contexts = Vec::new();
    for (key, value) in dict {
        let key: String = key.extract()?;
        if key == "$symbolMetadata" {
            let json = PyModule::import(value.py(), "json")?;
            let raw: String = json.call_method1("dumps", (&value,))?.extract()?;
            symbol_contexts =
                pine_runtime::symbol_contexts_from_json(&raw).map_err(PyValueError::new_err)?;
            continue;
        }
        if key == "$currencyRates" {
            let json = PyModule::import(value.py(), "json")?;
            let raw: String = json.call_method1("dumps", (&value,))?.extract()?;
            currency_rates = Some(
                pine_runtime::currency_rates_from_json(&raw)
                    .map_err(|error| PyValueError::new_err(error.to_string()))?,
            );
            continue;
        }
        if key == "$barCalendars" {
            let json = PyModule::import(value.py(), "json")?;
            let raw: String = json.call_method1("dumps", (&value,))?.extract()?;
            calendars = Some(
                pine_runtime::bar_calendars_from_json(&raw)
                    .map_err(|err| PyValueError::new_err(err.to_string()))?,
            );
            continue;
        }
        if key == "$chart" {
            chart = chart_metadata::parse_chart_metadata(chart, &value)?;
            continue;
        }
        let request_key = parse_request_key(&key)?;
        // Provider validation owns context-specific ordering. Renko bricks
        // preserve their row ordinals even when a source timestamp repeats.
        let mut bars = Vec::new();
        for item in value.try_iter()? {
            bars.push(parse_bar(&item?)?);
        }
        streams.push((request_key, bars));
    }
    let provider = InMemoryRequestDataProvider::from_streams(streams)
        .map_err(|err| PyValueError::new_err(err.to_string()))?;
    let environment = RequestEnvironment::new(chart, std::sync::Arc::new(provider))
        .with_symbol_contexts(symbol_contexts)
        .map_err(PyValueError::new_err)?;
    let environment = match calendars {
        Some(calendars) => environment.with_bar_calendars(std::sync::Arc::new(calendars)),
        None => environment,
    };
    Ok(match currency_rates {
        Some(rates) => environment.with_currency_rates(std::sync::Arc::new(rates)),
        None => environment,
    })
}
