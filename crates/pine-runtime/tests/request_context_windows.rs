use pine_runtime::{
    Bar, ChartContext, HistoricalRuntime, InMemoryRequestDataProvider, PineValue, PreparedProgram,
    RequestEnvironment, RequestKey, RequestTimeframe, RuntimeError,
};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;
use std::sync::Arc;

fn bar(time: i64) -> Bar {
    Bar {
        time,
        open: 10.,
        high: 10.,
        low: 10.,
        close: 10.,
        volume: 1.,
    }
}

#[test]
fn native_whole_sources_select_shared_windows_before_public_iteration() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(
            root.join("docs/evidence/strategy-library-20261010/request-window-oracles-44.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let day = 86_400_000i64;
    let bars: Vec<_> = (0..3350).map(|i| bar(1502928000000 + i * day)).collect();
    let weekly: Vec<_> = (0..479).map(|i| bar(1502668800000 + i * day * 7)).collect();
    let tf = |value| RequestTimeframe::parse(value).unwrap();
    let provider = Arc::new(
        InMemoryRequestDataProvider::from_streams(vec![
            (RequestKey::new("BINANCE:BTCUSDT", tf("W")), weekly),
            (RequestKey::new("NASDAQ:AAPL", tf("D")), bars.clone()),
        ])
        .unwrap(),
    );
    let environment =
        RequestEnvironment::new(ChartContext::new("BINANCE:BTCUSDT", tf("D")), provider);
    for case in manifest["records"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let source = std::fs::read_to_string(root.join(case["source"].as_str().unwrap())).unwrap();
        let analysis = analyze_source(&SourceFile::new(name, source));
        if case["native_status"] == "analysis_rejected" {
            assert!(
                !analysis.diagnostics.is_empty(),
                "{name}: Native rejects this source"
            );
            assert!(
                analysis
                    .diagnostics
                    .iter()
                    .all(|d| d.code != "E_UNKNOWN_SYMBOL"
                        && !d.code.starts_with(concat!("E_", "PARSE"))),
                "{name}: parser must reach qualifier validation: {:?}",
                analysis.diagnostics
            );
            continue;
        }
        assert!(
            analysis.diagnostics.is_empty(),
            "{name}: {:?}",
            analysis.diagnostics
        );
        let program = analysis.hir.unwrap();
        let prepared = PreparedProgram::from_shared_hir(Arc::new(program.clone()));
        for mode in 0..4 {
            let mut runtime = if mode == 0 {
                HistoricalRuntime::with_request_environment(&program, environment.clone())
            } else {
                HistoricalRuntime::from_prepared_with_request_environment_and_input_overrides(
                    &prepared,
                    environment.clone(),
                    Default::default(),
                )
            };
            let outcome = if mode < 2 {
                runtime.append_bars(&bars)
            } else if mode == 3 {
                let skip = runtime.initialize_historical_dataset(&bars, None).unwrap();
                bars[skip..]
                    .iter()
                    .try_for_each(|bar| runtime.append_bar(*bar))
            } else {
                runtime
                    .historical_dataset(&bars)
                    .unwrap()
                    .collect::<Result<Vec<_>, RuntimeError>>()
                    .map(|_| ())
            };
            if case["native_status"] == "runtime_error" {
                assert_eq!(
                    outcome.unwrap_err().message,
                    case["runtime_error_message"].as_str().unwrap(),
                    "{name}/{mode}"
                );
                assert_eq!(
                    runtime.profile().bars,
                    case["runtime_error_bar"].as_u64().unwrap() as usize,
                    "{name}/{mode}"
                );
            } else {
                outcome.unwrap_or_else(|error| panic!("{name}/{mode}: {error:?}"));
                let values: Vec<_> = runtime
                    .result()
                    .plots
                    .iter()
                    .map(|p| p.values.last().and_then(PineValue::as_f64))
                    .collect();
                let expected: Vec<_> = case["native_values"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_f64())
                    .collect();
                assert_eq!(values, expected, "{name}/{mode}");
                assert_eq!(
                    runtime.profile().bars as f64,
                    expected.last().copied().flatten().unwrap(),
                    "{name}/{mode} must never expose discarded provisional bars"
                );
            }
        }
    }
}
