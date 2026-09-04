use std::{collections::BTreeMap, sync::Arc};

use pine_runtime::{
    Bar, ChartContext, InMemoryRequestDataProvider, MagnifierInput, RequestEnvironment, RequestKey,
    RequestTimeframe, magnifier_input_from_json,
};
use serde_json::Value;

pub(crate) struct RequestHostParse {
    pub environment: RequestEnvironment,
    pub execution_times: Option<Vec<i64>>,
    pub magnifier: Option<MagnifierInput>,
}

#[cfg(test)]
pub(crate) fn request_environment_from_json(
    request_bars_json: &str,
) -> Result<RequestEnvironment, String> {
    request_environment_and_execution_times_from_json(request_bars_json)
        .map(|parsed| parsed.environment)
}

pub(crate) fn request_environment_and_execution_times_from_json(
    request_bars_json: &str,
) -> Result<RequestHostParse, String> {
    let value: Value = serde_json::from_str(request_bars_json).map_err(|err| {
        format!("request bars must be a JSON object mapping SYMBOL:TIMEFRAME to bar arrays: {err}")
    })?;
    let object = value.as_object().ok_or_else(|| {
        "request bars must be a JSON object mapping SYMBOL:TIMEFRAME to bar arrays".to_owned()
    })?;
    let chart = parse_chart_context(object.get("$chart"))?;
    let execution_times = parse_execution_times(object.get("$executionTimes"))?;
    let magnifier = parse_magnifier(object.get("$magnifier"))?;

    let mut streams = Vec::with_capacity(object.len());
    for (key, bars) in deterministic_entries(object) {
        if matches!(key.as_str(), "$chart" | "$executionTimes" | "$magnifier") {
            continue;
        }
        let request_key = parse_request_key(key)?;
        streams.push((request_key, parse_bars(key, bars)?));
    }
    let environment = if streams.is_empty() {
        RequestEnvironment::default().for_chart(chart)
    } else {
        let provider =
            InMemoryRequestDataProvider::from_streams(streams).map_err(|err| err.to_string())?;
        RequestEnvironment::new(chart, Arc::new(provider))
    };
    Ok(RequestHostParse {
        environment,
        execution_times,
        magnifier,
    })
}

fn parse_magnifier(value: Option<&Value>) -> Result<Option<MagnifierInput>, String> {
    let Some(value) = value else {
        return Ok(None);
    };
    let json = serde_json::to_string(value)
        .map_err(|err| format!("request host input `$magnifier` is invalid: {err}"))?;
    magnifier_input_from_json(&json).map(Some)
}

fn parse_execution_times(value: Option<&Value>) -> Result<Option<Vec<i64>>, String> {
    let Some(value) = value else {
        return Ok(None);
    };
    let values = value
        .as_array()
        .ok_or_else(|| "request host input `$executionTimes` must be an array".to_owned())?;
    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            value.as_i64().ok_or_else(|| {
                format!(
                    "request host input `$executionTimes` value at index {index} must be an integer millisecond timestamp"
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}

fn parse_chart_context(value: Option<&Value>) -> Result<ChartContext, String> {
    let Some(value) = value else {
        return Ok(ChartContext::default());
    };
    let object = value
        .as_object()
        .ok_or_else(|| "request bars `$chart` must be an object".to_owned())?;
    if let Some(field) = object
        .keys()
        .find(|field| !matches!(field.as_str(), "symbol" | "timeframe"))
    {
        return Err(format!("request bars `$chart` has unknown field `{field}`"));
    }
    let mut chart = ChartContext::default();
    if let Some(symbol) = object.get("symbol") {
        let symbol = symbol
            .as_str()
            .ok_or_else(|| "request bars `$chart.symbol` must be a string".to_owned())?;
        if symbol.trim().is_empty() {
            return Err("request bars `$chart.symbol` must not be empty".to_owned());
        }
        chart = chart.with_symbol(symbol.trim());
    }
    if let Some(timeframe) = object.get("timeframe") {
        let timeframe = timeframe
            .as_str()
            .ok_or_else(|| "request bars `$chart.timeframe` must be a string".to_owned())?;
        let timeframe = RequestTimeframe::parse(timeframe).map_err(|err| err.to_string())?;
        chart = chart.with_timeframe(timeframe);
    }
    Ok(chart)
}

fn deterministic_entries(object: &serde_json::Map<String, Value>) -> BTreeMap<&String, &Value> {
    object.iter().collect()
}

fn parse_request_key(key: &str) -> Result<RequestKey, String> {
    let Some((symbol, timeframe)) = key.rsplit_once(':') else {
        return Err("request bars key must use SYMBOL:TIMEFRAME".to_owned());
    };
    if symbol.trim().is_empty() {
        return Err("request bars symbol must not be empty".to_owned());
    }
    let timeframe = RequestTimeframe::parse(timeframe).map_err(|err| err.to_string())?;
    Ok(RequestKey::new(symbol.trim(), timeframe))
}

fn parse_bars(key: &str, value: &Value) -> Result<Vec<Bar>, String> {
    let bars = value
        .as_array()
        .ok_or_else(|| format!("request bars for key `{key}` must be an array of bar objects"))?;
    bars.iter()
        .enumerate()
        .map(|(index, bar)| parse_bar(key, index, bar))
        .collect()
}

fn parse_bar(key: &str, index: usize, value: &Value) -> Result<Bar, String> {
    let object = value
        .as_object()
        .ok_or_else(|| format!("request bar for key `{key}` at index {index} must be an object"))?;
    Ok(Bar {
        time: bar_i64(object, key, index, "time")?,
        open: bar_f64(object, key, index, "open")?,
        high: bar_f64(object, key, index, "high")?,
        low: bar_f64(object, key, index, "low")?,
        close: bar_f64(object, key, index, "close")?,
        volume: bar_f64(object, key, index, "volume")?,
    })
}

fn bar_i64(
    object: &serde_json::Map<String, Value>,
    key: &str,
    index: usize,
    field: &str,
) -> Result<i64, String> {
    let value = object.get(field).ok_or_else(|| {
        format!("request bar for key `{key}` at index {index} is missing `{field}`")
    })?;
    value.as_i64().ok_or_else(|| {
        format!("request bar field `{field}` for key `{key}` at index {index} must be an integer")
    })
}

fn bar_f64(
    object: &serde_json::Map<String, Value>,
    key: &str,
    index: usize,
    field: &str,
) -> Result<f64, String> {
    let value = object.get(field).ok_or_else(|| {
        format!("request bar for key `{key}` at index {index} is missing `{field}`")
    })?;
    let parsed = value.as_f64().ok_or_else(|| {
        format!("request bar field `{field}` for key `{key}` at index {index} must be a number")
    })?;
    if !parsed.is_finite() {
        return Err(format!(
            "request bar field `{field}` for key `{key}` at index {index} must be finite"
        ));
    }
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request_bars_json(key: &str, bars: &str) -> String {
        format!("{{\"{key}\":{bars}}}")
    }

    fn request_bars_error(json: &str) -> String {
        match request_environment_from_json(json) {
            Ok(_) => panic!("request bars JSON should fail"),
            Err(message) => message,
        }
    }

    #[test]
    fn request_bars_parses_exchange_prefixed_symbol() {
        let environment = request_environment_from_json(&request_bars_json(
            "NYSE:IBM:1",
            r#"[{"time":0,"open":10,"high":11,"low":9,"close":30,"volume":100}]"#,
        ))
        .expect("request environment");

        let bars = environment
            .provider()
            .bars(&RequestKey::new("NYSE:IBM", RequestTimeframe::default()))
            .expect("request bars");

        assert_eq!(bars.len(), 1);
        assert_eq!(bars[0].close, 30.0);
    }

    #[test]
    fn request_bars_empty_object_uses_no_request_provider() {
        let environment = request_environment_from_json("{}").expect("request environment");
        let message = environment
            .provider()
            .bars(&RequestKey::new("NYSE:IBM", RequestTimeframe::default()))
            .expect_err("empty object should keep no-provider behavior")
            .to_string();

        assert_eq!(
            message,
            "missing request data for symbol `NYSE:IBM` timeframe `1`"
        );
    }

    #[test]
    fn request_bars_chart_metadata_configures_runtime_context() {
        let environment =
            request_environment_from_json(r#"{"$chart":{"symbol":" TEST ","timeframe":"5"}}"#)
                .expect("chart metadata");

        assert_eq!(environment.chart().symbol(), "TEST");
        assert_eq!(environment.chart().timeframe().value(), "5");
    }

    #[test]
    fn request_host_input_parses_execution_times_without_creating_a_request_stream() {
        let parsed = request_environment_and_execution_times_from_json(
            r#"{"$executionTimes":[101,205,333]}"#,
        )
        .expect("execution time host input");

        assert_eq!(parsed.execution_times, Some(vec![101, 205, 333]));
        assert!(parsed.magnifier.is_none());
        let message = parsed
            .environment
            .provider()
            .bars(&RequestKey::new("NYSE:IBM", RequestTimeframe::default()))
            .expect_err("execution times do not create provider data")
            .to_string();
        assert!(message.contains("missing request data"));
    }

    #[test]
    fn request_host_input_rejects_invalid_execution_times() {
        assert_eq!(
            request_bars_error(r#"{"$executionTimes":101}"#),
            "request host input `$executionTimes` must be an array"
        );
        assert_eq!(
            request_bars_error(r#"{"$executionTimes":[101,true]}"#),
            "request host input `$executionTimes` value at index 1 must be an integer millisecond timestamp"
        );
    }

    #[test]
    fn request_bars_rejects_invalid_chart_metadata() {
        assert_eq!(
            request_bars_error(r#"{"$chart":[]}"#),
            "request bars `$chart` must be an object"
        );
        assert_eq!(
            request_bars_error(r#"{"$chart":{"symbol":" "}}"#),
            "request bars `$chart.symbol` must not be empty"
        );
        assert_eq!(
            request_bars_error(r#"{"$chart":{"timeframe":"bad"}}"#),
            "unsupported request timeframe `bad`"
        );
        assert_eq!(
            request_bars_error(r#"{"$chart":{"timeFrame":"5"}}"#),
            "request bars `$chart` has unknown field `timeFrame`"
        );
    }

    #[test]
    fn request_bars_rejects_non_object_json() {
        let message = request_bars_error("[]");

        assert_eq!(
            message,
            "request bars must be a JSON object mapping SYMBOL:TIMEFRAME to bar arrays"
        );
    }

    #[test]
    fn request_bars_rejects_invalid_key_without_timeframe() {
        let message = request_bars_error(&request_bars_json(
            "NYSE_IBM",
            r#"[{"time":0,"open":10,"high":11,"low":9,"close":30,"volume":100}]"#,
        ));

        assert_eq!(message, "request bars key must use SYMBOL:TIMEFRAME");
    }

    #[test]
    fn request_bars_rejects_empty_symbol() {
        let message = request_bars_error(&request_bars_json(
            ":1",
            r#"[{"time":0,"open":10,"high":11,"low":9,"close":30,"volume":100}]"#,
        ));

        assert_eq!(message, "request bars symbol must not be empty");
    }

    #[test]
    fn request_bars_rejects_invalid_timeframe() {
        let message = request_bars_error(&request_bars_json(
            "NYSE:IBM:not-a-timeframe",
            r#"[{"time":0,"open":10,"high":11,"low":9,"close":30,"volume":100}]"#,
        ));

        assert_eq!(message, "unsupported request timeframe `not-a-timeframe`");
    }

    #[test]
    fn request_bars_rejects_missing_bar_field() {
        let message = request_bars_error(&request_bars_json(
            "NYSE:IBM:1",
            r#"[{"time":0,"open":10,"high":11,"low":9,"volume":100}]"#,
        ));

        assert_eq!(
            message,
            "request bar for key `NYSE:IBM:1` at index 0 is missing `close`"
        );
    }

    #[test]
    fn request_bars_rejects_non_finite_bar_fields() {
        let message = request_bars_error(&request_bars_json(
            "NYSE:IBM:1",
            r#"[{"time":0,"open":1e309,"high":11,"low":9,"close":30,"volume":100}]"#,
        ));

        assert!(
            message.contains("number out of range"),
            "unexpected error: {message}"
        );
    }

    #[test]
    fn request_bars_documents_duplicate_json_key_collapse() {
        let environment = request_environment_from_json(
            r#"{"NYSE:IBM:1":[{"time":0,"open":1,"high":1,"low":1,"close":1,"volume":1}],"NYSE:IBM:1":[{"time":0,"open":2,"high":2,"low":2,"close":2,"volume":2}]}"#,
        )
        .expect("serde_json map parsing collapses duplicate object keys before provider validation");

        let bars = environment
            .provider()
            .bars(&RequestKey::new("NYSE:IBM", RequestTimeframe::default()))
            .expect("request bars");

        assert_eq!(bars.len(), 1);
        assert_eq!(bars[0].close, 2.0);
    }

    #[test]
    fn request_bars_rejects_unsorted_requested_bars() {
        let message = request_bars_error(&request_bars_json(
            "NYSE:IBM:1",
            r#"[{"time":60000,"open":10,"high":11,"low":9,"close":30,"volume":100},{"time":0,"open":11,"high":12,"low":10,"close":32,"volume":100}]"#,
        ));

        assert_eq!(
            message,
            "requested bars are not sorted: `0` follows `60000`"
        );
    }

    #[test]
    fn request_bars_rejects_duplicate_requested_bar_times() {
        let message = request_bars_error(&request_bars_json(
            "NYSE:IBM:1",
            r#"[{"time":0,"open":10,"high":11,"low":9,"close":30,"volume":100},{"time":0,"open":11,"high":12,"low":10,"close":32,"volume":100}]"#,
        ));

        assert_eq!(message, "duplicate requested bar time `0`");
    }
}
