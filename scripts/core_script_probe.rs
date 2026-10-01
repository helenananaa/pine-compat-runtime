//! Offline acceptance probe, built outside the workspace by the qualification runner.
//! Deliberately calls the public Rust runtime rather than the CLI or a binding.
use pine_runtime::{
    Bar, ChartContext, HistoricalRuntime, InMemoryRequestDataProvider, InputOverrides, PineValue,
    RequestEnvironment, RequestKey, RequestTimeframe, ValueKind, chart_source_input_override,
    input_calls, public_runtime_result_json,
};
use pine_sema::analyze_source;
use pine_syntax::{Severity, SourceFile};
use serde_json::Value;
use std::{io, sync::Arc};

fn bars(value: &Value) -> Vec<Bar> {
    value
        .as_array()
        .unwrap()
        .iter()
        .map(|b| Bar {
            time: b["time"].as_i64().unwrap(),
            open: b["open"].as_f64().unwrap(),
            high: b["high"].as_f64().unwrap(),
            low: b["low"].as_f64().unwrap(),
            close: b["close"].as_f64().unwrap(),
            volume: b["volume"].as_f64().unwrap(),
        })
        .collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let p: Value = serde_json::from_reader(io::stdin())?;
    let source = SourceFile::new("acceptance.pine", p["source"].as_str().unwrap());
    let analysis = analyze_source(&source);
    if analysis
        .diagnostics
        .iter()
        .any(|d| d.severity == Severity::Error)
    {
        return Err(format!("{:?}", analysis.diagnostics).into());
    }
    let hir = analysis.hir.ok_or("missing HIR")?;
    let c = &p["request"]["$chart"];
    let mut chart = ChartContext::new(
        c["symbol"].as_str().unwrap(),
        RequestTimeframe::parse(c["timeframe"].as_str().unwrap()).map_err(|e| e.to_string())?,
    )
    .with_price_grid(
        c["minMove"].as_u64().unwrap() as u32,
        c["priceScale"].as_u64().unwrap() as u32,
    )?;
    if let Some(q) = c["quantityPrecision"].as_u64() {
        chart = chart.with_quantity_precision(q as u32)?;
    }
    if let Some(v) = c["pointValue"].as_f64() {
        chart = chart.with_point_value(v)?;
    }
    let mut streams = Vec::new();
    for (key, value) in p["request"].as_object().unwrap() {
        if key == "$chart" {
            continue;
        }
        let (symbol, tf) = key.rsplit_once(':').ok_or("request key")?;
        streams.push((
            RequestKey::new(
                symbol,
                RequestTimeframe::parse(tf).map_err(|e| e.to_string())?,
            ),
            bars(value),
        ));
    }
    let env = RequestEnvironment::new(
        chart,
        Arc::new(InMemoryRequestDataProvider::from_streams(streams).map_err(|e| e.to_string())?),
    );
    let calls = input_calls(&hir);
    let mut overrides = InputOverrides::new();
    for (key, v) in p["overrides"].as_object().unwrap() {
        let id: u32 = key.parse()?;
        let input = calls
            .iter()
            .find(|c| c.call_site_id == id)
            .ok_or("unknown input")?;
        let value = if input.is_source {
            chart_source_input_override(v.as_str().unwrap())?
        } else {
            match input.value_kind {
                ValueKind::Int => PineValue::Int(v.as_i64().unwrap()),
                ValueKind::Float => PineValue::Float(v.as_f64().unwrap()),
                ValueKind::Bool => PineValue::Bool(v.as_bool().unwrap()),
                ValueKind::String => PineValue::String(v.as_str().unwrap().to_owned()),
                _ => return Err("probe input type outside frozen scope".into()),
            }
        };
        overrides.insert(id, value);
    }
    let mut runtime =
        HistoricalRuntime::with_request_environment_and_input_overrides(&hir, env, overrides);
    runtime
        .append_bars(&bars(&p["bars"]))
        .map_err(|e| e.message)?;
    println!("{}", public_runtime_result_json(&runtime.result()));
    Ok(())
}
