use pine_runtime::{
    Bar, BarUpdate, ChartContext, InMemoryRequestDataProvider, PineValue, RealtimeRuntime,
    RequestEnvironment, RequestTimeframe,
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
fn native_initial_request_origin_extends_across_new_minutes_and_rolls_back_ticks() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.join(
            "docs/evidence/strategy-library-20261010/request-window-live-origin-oracles-44.json",
        ))
        .unwrap(),
    )
    .unwrap();
    let initial = &manifest["records"][0];
    let continued = &manifest["records"][1];
    let source = std::fs::read_to_string(root.join(initial["source"].as_str().unwrap())).unwrap();
    let analysis = analyze_source(&SourceFile::new("live.pine", source));
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let program = analysis.hir.unwrap();
    let environment = RequestEnvironment::new(
        ChartContext::new("BINANCE:BTCUSDT", RequestTimeframe::parse("1").unwrap()),
        Arc::new(InMemoryRequestDataProvider::default()),
    );
    let mut runtime = RealtimeRuntime::with_request_environment(&program, environment);
    let time = initial["native_values"][3].as_i64().unwrap();
    let values = |result: pine_runtime::RuntimeResult| {
        result
            .plots
            .iter()
            .map(|p| p.values.last().and_then(PineValue::as_f64).unwrap())
            .collect::<Vec<_>>()
    };
    let expected = |case: &serde_json::Value| {
        case["native_values"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap())
            .collect::<Vec<_>>()
    };
    assert_eq!(
        values(
            runtime
                .seed_historical(&[bar(time - 60000), bar(time)])
                .unwrap()
        ),
        expected(initial)
    );
    for _ in 0..3 {
        assert_eq!(
            values(
                runtime
                    .update(BarUpdate::forming(bar(time + 60000)))
                    .unwrap()
            ),
            vec![2., 2., 3., (time + 60000) as f64]
        );
    }
    runtime
        .update(BarUpdate::confirmed(bar(time + 60000)))
        .unwrap();
    for _ in 0..3 {
        assert_eq!(
            values(
                runtime
                    .update(BarUpdate::forming(bar(time + 120000)))
                    .unwrap()
            ),
            expected(continued)
        );
    }
    assert_eq!(
        values(
            runtime
                .update(BarUpdate::confirmed(bar(time + 120000)))
                .unwrap()
        ),
        expected(continued)
    );
}
