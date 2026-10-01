//! Offline acceptance probe, built outside the workspace by the qualification runner.
//! Deliberately calls the public Rust runtime rather than the CLI or a binding.
use pine_runtime::{
    Bar, BarUpdate, ChartContext, HistoricalRuntime, RealtimeRuntime, InMemoryRequestDataProvider, InputOverrides, PineValue,
    RequestEnvironment, RequestKey, RequestTimeframe, ValueKind, chart_source_input_override,
    input_calls, public_runtime_result_json,
};
use pine_sema::analyze_source;
use pine_syntax::{Severity, SourceFile};
use serde_json::Value;
use std::sync::Arc;

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
    let args: Vec<String> = std::env::args().collect();
    let p: Value = serde_json::from_reader(std::fs::File::open(&args[1])?)?;
    let count: usize = args[2].parse()?;
    let compile_started = std::time::Instant::now();
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
        chart.clone(),
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
    let mut metrics: std::collections::BTreeMap<String, Vec<f64>> = std::collections::BTreeMap::new();
    metrics.entry("compile".into()).or_default().push(compile_started.elapsed().as_secs_f64()*1000.0);
    let mut sessions = Vec::new();
    let mut replicas = Vec::new();
    let offset = |mut b: Bar, i: usize| { let d = i as f64 * 0.125; b.open += d; b.high += d; b.low += d; b.close += d; b };
    for i in 0..count {
        let mut streams = Vec::new();
        for (key, value) in p["request"].as_object().unwrap() {
            if key == "$chart" { continue; }
            let (symbol, tf) = key.rsplit_once(':').unwrap();
            streams.push((RequestKey::new(symbol, RequestTimeframe::parse(tf).unwrap()), bars(value).into_iter().map(|b|offset(b,i)).collect()));
        }
        let env_i = RequestEnvironment::new(chart.clone(), Arc::new(InMemoryRequestDataProvider::from_streams(streams).unwrap()));
        let mut session = RealtimeRuntime::with_request_environment_and_input_overrides(&hir, env_i, overrides.clone());
        let input: Vec<Bar> = bars(&p["bars"]).into_iter().map(|b|offset(b,i)).collect();
        let started = std::time::Instant::now();
        session.seed_historical(&input).map_err(|e|e.message)?;
        metrics.entry("seed".into()).or_default().push(started.elapsed().as_secs_f64()*1000.0);
        replicas.push(session.replica()); sessions.push(session);
    }
    for (index, event) in p["events"].as_array().unwrap().iter().enumerate() {
        for (i, (session, replica)) in sessions.iter_mut().zip(replicas.iter_mut()).enumerate() {
            let b = offset(bars(&serde_json::json!([event["bar"].clone()]))[0],i);
            let before = session.confirmed_bar_count();
            let kind = event["kind"].as_str().unwrap();
            let started = std::time::Instant::now();
            let changes = if kind.starts_with("request") {
                session.apply_request_update(RequestKey::new("OFFLINE:RESOURCE",RequestTimeframe::parse("1D").unwrap()),
                  if kind == "request_forming" { BarUpdate::forming(b) } else { BarUpdate::confirmed(b) }).map_err(|e|e.message)?
            } else {
                Some(session.apply_update(if kind == "forming" { BarUpdate::forming(b) } else { BarUpdate::confirmed(b) }).map_err(|e|e.message)?)
            };
            metrics.entry(event["phase"].as_str().unwrap().into()).or_default().push(started.elapsed().as_secs_f64()*1000.0);
            if let Some(changes) = changes {
                let started = std::time::Instant::now(); replica.apply(&changes).map_err(|e|e.message)?;
                metrics.entry("replica".into()).or_default().push(started.elapsed().as_secs_f64()*1000.0);
            }
            assert_eq!(session.confirmed_bar_count(),before+usize::from(kind == "confirmed"));
        }
        if index % 1024 == 0 { eprintln!("events {index}"); }
    }
    let mut results = Vec::new();
    for (session, replica) in sessions.iter().zip(replicas.iter()) {
        let started = std::time::Instant::now();let result = session.result();
        metrics.entry("snapshot".into()).or_default().push(started.elapsed().as_secs_f64()*1000.0);
        let started = std::time::Instant::now();let text = public_runtime_result_json(&result);
        metrics.entry("serialization".into()).or_default().push(started.elapsed().as_secs_f64()*1000.0);
        assert_eq!(text,public_runtime_result_json(replica.result()));
        assert!(result.diagnostics.is_empty());
        assert_eq!(session.confirmed_bar_count(),p["bars"].as_array().unwrap().len()+p["tail"].as_array().unwrap().len());
        results.push(serde_json::from_str::<Value>(&text)?);
    }
    // Historical append and batch have identical immutable provider inputs.
    let expected_count = p["bars"].as_array().unwrap().len()+p["tail"].as_array().unwrap().len();
    for i in 0..count {
      let mut historical = HistoricalRuntime::with_request_environment_and_input_overrides(&hir,env.clone(),overrides.clone());
      let seed: Vec<Bar> = bars(&p["bars"]).into_iter().map(|b|offset(b,i)).collect(); historical.append_bars(&seed).map_err(|e|e.message)?;
      let tail: Vec<Bar> = bars(&p["tail"]).into_iter().map(|b|offset(b,i)).collect();
      for b in &tail {
        let started = std::time::Instant::now();historical.append_bar(*b).map_err(|e|e.message)?;
        metrics.entry("append".into()).or_default().push(started.elapsed().as_secs_f64()*1000.0);
    }
      let mut batch = HistoricalRuntime::with_request_environment_and_input_overrides(&hir,env.clone(),overrides.clone());
    let all: Vec<Bar> = seed.into_iter().chain(tail).collect();batch.append_bars(&all).map_err(|e|e.message)?;
    assert_eq!(public_runtime_result_json(&batch.result()),public_runtime_result_json(&historical.result()));
    }
    std::fs::write(&args[3],serde_json::to_string(&serde_json::json!({"metrics":metrics,"results":results,"historicalAppendMatches":true,"confirmedBars":expected_count}))?)?;
    Ok(())
}
