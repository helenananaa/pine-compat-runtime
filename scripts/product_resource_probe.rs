//! Offline acceptance probe, built outside the workspace by the qualification runner.
//! Deliberately calls the public Rust runtime rather than the CLI or a binding.
use pine_runtime::{
    Bar, BarUpdate, ChartContext, HistoricalRuntime, RealtimeRuntime, InMemoryRequestDataProvider, InputOverrides, PineValue,
    RequestEnvironment, RequestKey, RequestTimeframe, ValueKind, chart_source_input_override,
    input_calls, write_public_runtime_result_json,
};
use pine_sema::analyze_source;
use pine_syntax::{Severity, SourceFile};
use serde_json::Value;
use std::sync::Arc;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

fn spool(path: &Path, result: &pine_runtime::RuntimeResult) -> std::io::Result<()> {
    // Coalesce token writes on Linux, including WSL reports on Windows drives.
    #[cfg(target_os = "linux")]
    let mut output = BufWriter::with_capacity(1024 * 1024, std::fs::File::create(path)?);
    #[cfg(not(target_os = "linux"))]
    let mut output = BufWriter::new(std::fs::File::create(path)?);
    write_public_runtime_result_json(result, &mut output)?;
    output.flush()
}

// Compare every byte; fixed-size chunks avoid retaining parsed output forests.
fn same_files(left: &Path, right: &Path) -> std::io::Result<bool> {
    let mut left = BufReader::new(std::fs::File::open(left)?);
    let mut right = BufReader::new(std::fs::File::open(right)?);
    let mut remaining = left.get_ref().metadata()?.len();
    if remaining != right.get_ref().metadata()?.len() { return Ok(false); }
    let mut a = [0u8; 65536];
    let mut b = [0u8; 65536];
    while remaining > 0 {
        let n = remaining.min(a.len() as u64) as usize;
        left.read_exact(&mut a[..n])?;
        right.read_exact(&mut b[..n])?;
        if a[..n] != b[..n] { return Ok(false); }
        remaining -= n as u64;
    }
    Ok(true)
}

struct HistoricalOutput {
    matches: bool,
    batch: PathBuf,
    incremental: PathBuf,
    same_context: PathBuf,
    same_context_matches: bool,
}

type Timings = std::collections::BTreeMap<String, Vec<f64>>;

fn record(timings: &mut Timings, phase: &str, started: std::time::Instant) {
    timings.entry(phase.into()).or_default().push(started.elapsed().as_secs_f64()*1000.0);
}

fn part_reference(report: &Path, part: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let name = report.file_name().and_then(|name| name.to_str()).ok_or("report filename is not UTF-8")?;
    let part_name = part.file_name().and_then(|name| name.to_str()).ok_or("part filename is not UTF-8")?;
    let parts_dir = report.with_file_name(format!("{name}.parts"));
    if name.contains('\\') || part_name.contains('\\') || part.parent() != Some(parts_dir.as_path()) {
        return Err("output part must be in the report's sibling parts directory".into());
    }
    Ok(serde_json::json!({"path": format!("{name}.parts/{part_name}")}))
}

fn write_report(path: &str, metrics: &std::collections::BTreeMap<String, Vec<f64>>,
    overhead: &mut Timings,
    results: &[PathBuf], controls: Option<&[HistoricalOutput]>, expected_count: usize,
    seed_count: usize) -> Result<(), Box<dyn std::error::Error>> {
    let started = std::time::Instant::now();
    let report_path = Path::new(path);
    let name = report_path.file_name().and_then(|name| name.to_str()).ok_or("report filename is not UTF-8")?;
    if name.contains('\\') { return Err("report filename must not contain a backslash".into()); }
    let result_refs = results.iter().map(|part| part_reference(report_path, part)).collect::<Result<Vec<_>, _>>()?;
    // Full outputs remain in their spool files; the manifest never reads or
    // rewrites those bytes and stays proportional to the session count.
    let mut report = serde_json::json!({
        "resourceReportVersion": 2,
        "metadata": {"path": format!("{name}.metadata.json")},
        "results": result_refs,
        "confirmedBars": expected_count
    });
    if let Some(controls) = controls {
        let mut contexts = Vec::with_capacity(controls.len());
        for (i, context) in controls.iter().enumerate() {
            contexts.push(serde_json::json!({
                "stream": i,
                "matches": context.matches,
                "batchDatasetEnd": expected_count-1,
                "initialSeedDatasetEnd": seed_count-1,
                "batch": part_reference(report_path, &context.batch)?,
                "incremental": part_reference(report_path, &context.incremental)?,
                "sameContextMatches": context.same_context_matches,
                "sameContextIncremental": part_reference(report_path, &context.same_context)?
            }));
        }
        report["historicalAppendMatches"] = serde_json::json!(controls.iter().all(|c| c.matches));
        report["historicalSameContextMatches"] = serde_json::json!(controls.iter().all(|c| c.same_context_matches));
        report["historicalContexts"] = serde_json::json!(contexts);
    }
    let mut output = BufWriter::new(std::fs::File::create(path)?);
    serde_json::to_writer(&mut output, &report)?;
    output.flush()?;
    drop(output);
    record(overhead, "reportWrite", started);
    // Timing/control metadata is independent of the retained complete outputs.
    // The controller need not parse all historical output copies into RAM.
    let metadata = serde_json::json!({
        "resourceMetadataVersion": 1,
        "metrics": metrics,
        "overheadMetrics": overhead,
        "timingNotes": "Wall-clock milliseconds. Metrics retain their original boundaries. Overhead phases exclude measured metric intervals and do not overlap each other; reportWrite includes manifest assembly/write/flush but excludes this final metadata write. These phases are attribution samples, not an exhaustive process wall-time total.",
        "resultCount": results.len(),
        "confirmedBars": expected_count,
        "historicalAppendMatches": controls.map(|items| items.iter().all(|item| item.matches)),
        "historicalSameContextMatches": controls.map(|items| items.iter().all(|item| item.same_context_matches))
    });
    let mut sidecar = BufWriter::new(std::fs::File::create(format!("{path}.metadata.json"))?);
    serde_json::to_writer(&mut sidecar, &metadata)?;
    sidecar.flush()?;
    Ok(())
}

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
    let mut overhead = Timings::new();
    let input_started = std::time::Instant::now();
    let p: Value = serde_json::from_reader(std::io::BufReader::new(std::fs::File::open(&args[1])?))?;
    record(&mut overhead, "inputRead", input_started);
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
    let offset = |mut b: Bar, i: usize| { let d = i as f64 * 0.125 * ((b.time/60000)%17-8) as f64; b.open += d; b.high += d; b.low += d; b.close += d; b };
    for i in 0..count {
        let setup_started = std::time::Instant::now();
        let mut streams = Vec::new();
        for (key, value) in p["request"].as_object().unwrap() {
            if key == "$chart" { continue; }
            let (symbol, tf) = key.rsplit_once(':').unwrap();
            streams.push((RequestKey::new(symbol, RequestTimeframe::parse(tf).unwrap()), bars(value).into_iter().map(|b|offset(b,i)).collect()));
        }
        let env_i = RequestEnvironment::new(chart.clone(), Arc::new(InMemoryRequestDataProvider::from_streams(streams).unwrap()));
        let mut session = RealtimeRuntime::with_request_environment_and_input_overrides(&hir, env_i, overrides.clone());
        let input: Vec<Bar> = bars(&p["bars"]).into_iter().map(|b|offset(b,i)).collect();
        record(&mut overhead, "sessionSetup", setup_started);
        let started = std::time::Instant::now();
        session.seed_historical(&input).map_err(|e|e.message)?;
        metrics.entry("seed".into()).or_default().push(started.elapsed().as_secs_f64()*1000.0);
        let replica_started = std::time::Instant::now();
        replicas.push(session.replica()); sessions.push(session);
        record(&mut overhead, "replicaSnapshot", replica_started);
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
    // attribution: tailComplete
    let spool_dir = PathBuf::from(format!("{}.parts", &args[3]));
    std::fs::create_dir_all(&spool_dir)?;
    let expected_count = p["bars"].as_array().unwrap().len()+p["tail"].as_array().unwrap().len();
    for (i, replica) in replicas.into_iter().enumerate() {
        let started = std::time::Instant::now();
        spool(&spool_dir.join(format!("replica-{i}.json")), replica.result())?;
        record(&mut overhead, "replicaSerialization", started);
        let started = std::time::Instant::now();
        drop(replica);
        record(&mut overhead, "replicaRelease", started);
    }
    // attribution: replicasReleased
    let mut results = Vec::new();
    for (i, session) in sessions.iter().enumerate() {
        let started = std::time::Instant::now();let result = session.result();
        metrics.entry("snapshot".into()).or_default().push(started.elapsed().as_secs_f64()*1000.0);
        let result_path = spool_dir.join(format!("live-{i}.json"));
        let replica_path = spool_dir.join(format!("replica-{i}.json"));
        let started = std::time::Instant::now();spool(&result_path, &result)?;
        metrics.entry("serialization".into()).or_default().push(started.elapsed().as_secs_f64()*1000.0);
        let started = std::time::Instant::now();
        assert!(same_files(&result_path, &replica_path)?);
        record(&mut overhead, "comparison", started);
        assert!(result.diagnostics.is_empty());
        assert_eq!(session.confirmed_bar_count(),p["bars"].as_array().unwrap().len()+p["tail"].as_array().unwrap().len());
        results.push(result_path);
        let started = std::time::Instant::now();
        drop(result);
        record(&mut overhead, "snapshotRelease", started);
    }
    let started = std::time::Instant::now();
    if count > 1 { for r in &results[1..] { assert!(!same_files(&results[0],r)?); } }
    record(&mut overhead, "comparison", started);
    // attribution: publicResultsCompared
    let started = std::time::Instant::now();
    drop(sessions);
    record(&mut overhead, "sessionsRelease", started);
    // attribution: sessionsReleased
    if args.get(4).is_some_and(|s| s=="--no-controls") {
        write_report(&args[3], &metrics, &mut overhead, &results, None, expected_count, p["bars"].as_array().unwrap().len())?;
        return Ok(());
    }
    // Historical append and batch have identical immutable provider inputs.
    let mut historical_contexts = Vec::new();
    for i in 0..count {
      let started = std::time::Instant::now();
      let mut historical = HistoricalRuntime::with_request_environment_and_input_overrides(&hir,env.clone(),overrides.clone());
      let seed: Vec<Bar> = bars(&p["bars"]).into_iter().map(|b|offset(b,i)).collect(); historical.append_bars(&seed).map_err(|e|e.message)?;
      let tail: Vec<Bar> = bars(&p["tail"]).into_iter().map(|b|offset(b,i)).collect();
      record(&mut overhead, "historicalSeed", started);
      for b in &tail {
        let started = std::time::Instant::now();historical.append_bar(*b).map_err(|e|e.message)?;
        metrics.entry("append".into()).or_default().push(started.elapsed().as_secs_f64()*1000.0);
    }
      let started = std::time::Instant::now();
      let mut batch = HistoricalRuntime::with_request_environment_and_input_overrides(&hir,env.clone(),overrides.clone());
    let all: Vec<Bar> = seed.into_iter().chain(tail).collect();batch.append_bars(&all).map_err(|e|e.message)?;
    record(&mut overhead, "historicalBatch", started);
    // attribution: historicalControlsExecuted
    let batch_path = spool_dir.join(format!("batch-{i}.json"));
    let incremental_path = spool_dir.join(format!("incremental-{i}.json"));
    let started = std::time::Instant::now();
    let result = batch.result();
    record(&mut overhead, "historicalSnapshot", started);
    let started = std::time::Instant::now();
    spool(&batch_path, &result)?;
    record(&mut overhead, "historicalSerialization", started);
    let started = std::time::Instant::now();
    drop(result);
    drop(batch);
    record(&mut overhead, "historicalRelease", started);
    let started = std::time::Instant::now();
    let result = historical.result();
    record(&mut overhead, "historicalSnapshot", started);
    let started = std::time::Instant::now();
    spool(&incremental_path, &result)?;
    record(&mut overhead, "historicalSerialization", started);
    let started = std::time::Instant::now();
    drop(result);
    drop(historical);
    record(&mut overhead, "historicalRelease", started);
    let started = std::time::Instant::now();
    let matches = same_files(&batch_path, &incremental_path)?;
    record(&mut overhead, "comparison", started);
    let started = std::time::Instant::now();
    let mut known = HistoricalRuntime::with_request_environment_and_input_overrides(&hir,env.clone(),overrides.clone());
    {
        let mut dataset = known.historical_dataset(&all).map_err(|e|e.message)?;
        for _ in 0..p["bars"].as_array().unwrap().len() { dataset.next().ok_or("missing seed step")?.map_err(|e|e.message)?; }
        record(&mut overhead, "historicalKnownPrefix", started);
        for _ in 0..p["tail"].as_array().unwrap().len() {
            let started = std::time::Instant::now();dataset.next().ok_or("missing tail step")?.map_err(|e|e.message)?;
            metrics.entry("sameContextAppend".into()).or_default().push(started.elapsed().as_secs_f64()*1000.0);
        }
    }
    let same_context = spool_dir.join(format!("same-context-{i}.json"));
    let started = std::time::Instant::now();
    let result = known.result();
    record(&mut overhead, "historicalSnapshot", started);
    let started = std::time::Instant::now();
    spool(&same_context, &result)?;
    record(&mut overhead, "historicalSerialization", started);
    let started = std::time::Instant::now();
    drop(result);
    drop(known);
    record(&mut overhead, "historicalRelease", started);
    let started = std::time::Instant::now();
    let same_context_matches = same_files(&batch_path, &same_context)?;
    record(&mut overhead, "comparison", started);
    assert!(same_context_matches, "known dataset differs from batch in stream {i}");
    historical_contexts.push(HistoricalOutput { matches, batch: batch_path, incremental: incremental_path, same_context, same_context_matches });
    }
    // attribution: reportAssembly
    write_report(&args[3], &metrics, &mut overhead, &results, Some(&historical_contexts), expected_count, p["bars"].as_array().unwrap().len())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let suffix = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!("resource-probe-{}-{}-{suffix}",
                std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.0); }
    }

    #[test]
    fn compact_manifest_preserves_complete_spools_and_control_results() {
        let temp = TempDir::new();
        let report = temp.0.join("result.json");
        let parts = temp.0.join("result.json.parts");
        std::fs::create_dir(&parts).unwrap();
        let retained = serde_json::to_vec(&serde_json::json!({"plots": ["complete-output".repeat(100_000)]})).unwrap();
        let live = parts.join("live-0.json");
        let controls = [HistoricalOutput {
            matches: false,
            batch: parts.join("batch-0.json"),
            incremental: parts.join("incremental-0.json"),
            same_context: parts.join("same-context-0.json"),
            same_context_matches: true,
        }];
        for path in [&live, &controls[0].batch, &controls[0].incremental, &controls[0].same_context] {
            std::fs::write(path, &retained).unwrap();
        }
        let metrics = Timings::from([("seed".into(), vec![1.25])]);
        let mut overhead = Timings::from([("comparison".into(), vec![0.5])]);
        write_report(report.to_str().unwrap(), &metrics, &mut overhead, &[live], Some(&controls), 12, 10).unwrap();
        let bytes = std::fs::read(&report).unwrap();
        assert!(bytes.len() < 2048, "manifest must not grow with full output size");
        let value: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(value["resourceReportVersion"], 2);
        assert_eq!(value["metadata"]["path"], "result.json.metadata.json");
        assert_eq!(value["confirmedBars"], 12);
        assert_eq!(value["historicalAppendMatches"], false);
        assert_eq!(value["historicalSameContextMatches"], true);
        assert_eq!(value["historicalContexts"][0]["batchDatasetEnd"], 11);
        assert_eq!(value["historicalContexts"][0]["initialSeedDatasetEnd"], 9);
        assert!(value.get("metrics").is_none());
        for reference in [&value["results"][0], &value["historicalContexts"][0]["batch"],
            &value["historicalContexts"][0]["incremental"], &value["historicalContexts"][0]["sameContextIncremental"]] {
            let relative = reference["path"].as_str().unwrap();
            assert!(!relative.contains('\\'));
            assert!(!Path::new(relative).is_absolute());
            assert_eq!(std::fs::read(temp.0.join(relative)).unwrap(), retained);
        }
        let metadata: Value = serde_json::from_slice(&std::fs::read(temp.0.join("result.json.metadata.json")).unwrap()).unwrap();
        assert_eq!(metadata["resourceMetadataVersion"], 1);
        assert_eq!(metadata["metrics"], serde_json::json!(metrics));
        assert_eq!(metadata["resultCount"], 1);
        assert_eq!(metadata["confirmedBars"], 12);
        assert_eq!(metadata["historicalAppendMatches"], false);
        assert_eq!(metadata["historicalSameContextMatches"], true);
        assert_eq!(metadata["overheadMetrics"]["comparison"], serde_json::json!([0.5]));
        let writes = metadata["overheadMetrics"]["reportWrite"].as_array().unwrap();
        assert_eq!(writes.len(), 1);
        assert!(writes[0].as_f64().unwrap().is_finite());
        assert!(writes[0].as_f64().unwrap() >= 0.0);
    }

    #[test]
    fn compact_manifest_without_controls_does_not_claim_control_matches() {
        let temp = TempDir::new();
        let report = temp.0.join("result.json");
        write_report(report.to_str().unwrap(), &Timings::new(), &mut Timings::new(), &[], None, 12, 10).unwrap();
        let value: Value = serde_json::from_slice(&std::fs::read(&report).unwrap()).unwrap();
        assert!(value.get("historicalContexts").is_none());
        assert!(value.get("historicalAppendMatches").is_none());
        assert!(value.get("historicalSameContextMatches").is_none());
        let metadata: Value = serde_json::from_slice(&std::fs::read(temp.0.join("result.json.metadata.json")).unwrap()).unwrap();
        assert!(metadata["historicalAppendMatches"].is_null());
        assert!(metadata["historicalSameContextMatches"].is_null());
    }

    #[test]
    fn references_are_restricted_to_the_report_sibling_parts_directory() {
        let report = Path::new("nested/result.json");
        assert_eq!(part_reference(report, Path::new("nested/result.json.parts/live-0.json")).unwrap(),
            serde_json::json!({"path": "result.json.parts/live-0.json"}));
        assert!(part_reference(report, Path::new("nested/other.parts/live-0.json")).is_err());
        assert!(part_reference(report, Path::new("nested/result.json.parts/../outside.json")).is_err());
    }
}
