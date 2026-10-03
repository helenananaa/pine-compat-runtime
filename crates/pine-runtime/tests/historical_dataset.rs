use pine_runtime::{Bar, HistoricalRuntime, public_runtime_result_json};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

fn bars() -> Vec<Bar> {
    (0..5)
        .map(|index| Bar {
            time: index * 60_000,
            open: 1.0,
            high: 3.0,
            low: 0.5,
            close: 2.0,
            volume: 10.0,
        })
        .collect()
}

#[test]
fn fixed_dataset_steps_preserve_last_bar_and_calc_window_semantics() {
    for count in [0, 3] {
        let source = SourceFile::new(
            "dataset.pine",
            format!(
                "//@version=6\nindicator(\"dataset\", calc_bars_count={count})\nplot(last_bar_index)\nplot(last_bar_time)\nplot(barstate.islastconfirmedhistory ? 1 : 0)\nif barstate.islastconfirmedhistory\n    label.new(bar_index, close, \"last\")\n"
            ),
        );
        let hir = analyze_source(&source).hir.unwrap();
        let bars = bars();
        let mut batch = HistoricalRuntime::new(&hir);
        batch.append_bars(&bars).unwrap();
        let mut stepped = HistoricalRuntime::new(&hir);
        for step in stepped.historical_dataset(&bars).unwrap() {
            step.unwrap();
        }
        assert_eq!(
            public_runtime_result_json(&batch.result()),
            public_runtime_result_json(&stepped.result())
        );
        assert_eq!(stepped.result().labels.len(), 1);
    }
}

#[test]
fn dropping_partial_iterator_releases_endpoint_for_newly_discovered_bar() {
    let source = SourceFile::new(
        "drop.pine",
        "//@version=6\nindicator(\"drop\")\nplot(barstate.islastconfirmedhistory ? 1 : 0)\nplot(last_bar_index)\n",
    );
    let hir = analyze_source(&source).hir.unwrap();
    let bars = bars();
    let mut runtime = HistoricalRuntime::new(&hir);
    {
        let mut dataset = runtime.historical_dataset(&bars).unwrap();
        dataset.next().unwrap().unwrap();
    }
    runtime.append_bar(bars[1]).unwrap();
    let text = public_runtime_result_json(&runtime.result());
    assert!(text.contains("\"values\":[0,1]"), "{text}");
    assert!(text.contains("\"values\":[4,1]"), "{text}");
}

#[test]
fn execution_error_stops_iterator_and_restores_streaming_context() {
    let source = SourceFile::new(
        "error.pine",
        "//@version=6\nindicator(\"error\")\nif bar_index == 1 and last_bar_index == 4\n    runtime.error(\"stop\")\nplot(close)\n",
    );
    let hir = analyze_source(&source).hir.unwrap();
    let bars = bars();
    let mut runtime = HistoricalRuntime::new(&hir);
    let mut dataset = runtime.historical_dataset(&bars).unwrap();
    dataset.next().unwrap().unwrap();
    assert!(dataset.next().unwrap().is_err());
    assert!(dataset.next().is_none());
    assert!(dataset.next().is_none());
    drop(dataset);
    runtime.append_bar(bars[1]).unwrap();
}
