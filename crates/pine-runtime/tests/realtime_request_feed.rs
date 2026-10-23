use pine_runtime::{
    Bar, BarUpdate, BarUpdateKind, ChartContext, InMemoryRequestDataProvider, RealtimeRuntime,
    RequestEnvironment, RequestFeedUpdate, RequestKey, RequestTimeframe,
    public_runtime_changes_json, public_runtime_result_json,
};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;
use std::sync::Arc;

fn program() -> pine_ir::HirProgram {
    let analysis = analyze_source(&SourceFile::new(
        "feed.pine",
        "//@version=6\nindicator(\"feed\")\nvarip int n=0\nn+=1\nplot(n)\na=request.security_lower_tf(\"B\",\"1\",close)\nplot(array.size(a))\nplot(array.sum(a))",
    ));
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    analysis.hir.unwrap()
}

fn bar(time: i64, price: f64) -> Bar {
    Bar {
        time,
        open: price,
        high: price,
        low: price,
        close: price,
        volume: 1.0,
    }
}

fn update(time: i64, price: f64) -> RequestFeedUpdate {
    RequestFeedUpdate {
        symbol: "B".to_owned(),
        timeframe: "1".to_owned(),
        bar: bar(time, price),
        kind: BarUpdateKind::Confirmed,
    }
}

fn seeded(hir: &pine_ir::HirProgram) -> RealtimeRuntime<'_> {
    let tf = RequestTimeframe::parse("1").unwrap();
    let provider = InMemoryRequestDataProvider::from_streams([
        (
            RequestKey::new("B", tf.clone()),
            (0..5).map(|i| bar(i * 60000, 10.0)).collect(),
        ),
        (RequestKey::new("C", tf), vec![bar(0, 10.0)]),
    ])
    .unwrap();
    let env = RequestEnvironment::new(
        ChartContext::new("A", RequestTimeframe::parse("5").unwrap()),
        Arc::new(provider),
    );
    let mut runtime = RealtimeRuntime::with_request_environment(hir, env);
    runtime.seed_historical(&[bar(0, 10.0)]).unwrap();
    runtime
        .apply_request_feed_updates(&[update(300000, 10.0)])
        .unwrap();
    runtime
        .update_with_execution_time(BarUpdate::forming(bar(300000, 10.0)), 359999)
        .unwrap();
    runtime
}

fn last(runtime: &RealtimeRuntime<'_>) -> Vec<f64> {
    runtime
        .result()
        .plots
        .iter()
        .map(|p| p.values.last().unwrap().as_f64().unwrap())
        .collect()
}

#[test]
fn feed_transaction_preserves_output_revision_and_varip_until_one_chart_observation() {
    let hir = program();
    let mut runtime = seeded(&hir);
    assert_eq!(last(&runtime), [2.0, 1.0, 10.0]);
    let before = public_runtime_result_json(&runtime.result());
    let changes = runtime.last_changes().map(public_runtime_changes_json);
    let revision = runtime.revision();
    runtime
        .apply_request_feed_updates(&[update(360000, 20.0), update(420000, 30.0)])
        .unwrap();
    assert_eq!(runtime.revision(), revision);
    assert_eq!(
        runtime.last_changes().map(public_runtime_changes_json),
        changes
    );
    assert_eq!(public_runtime_result_json(&runtime.result()), before);
    runtime
        .update_with_execution_time(BarUpdate::forming(bar(300000, 11.0)), 479999)
        .unwrap();
    assert_eq!(last(&runtime), [3.0, 3.0, 60.0]);
    runtime
        .update_with_execution_time(BarUpdate::confirmed(bar(300000, 11.0)), 599999)
        .unwrap();
    assert_eq!(last(&runtime), [4.0, 3.0, 60.0]);
    assert_eq!(runtime.confirmed_bar_count(), 2);
}

#[test]
fn invalid_later_update_does_not_publish_an_earlier_update_from_the_batch() {
    let hir = program();
    let mut runtime = seeded(&hir);
    let before = public_runtime_result_json(&runtime.result());
    let mut stale = update(0, 999.0);
    stale.symbol = "C".to_owned();
    assert!(
        runtime
            .apply_request_feed_updates(&[update(360000, 20.0), stale])
            .is_err()
    );
    assert_eq!(public_runtime_result_json(&runtime.result()), before);
    runtime
        .apply_request_feed_updates(&[update(360000, 20.0)])
        .unwrap();
    runtime
        .update_with_execution_time(BarUpdate::forming(bar(300000, 11.0)), 419999)
        .unwrap();
    assert_eq!(last(&runtime), [3.0, 2.0, 30.0]);
}

#[test]
fn malformed_ohlcv_keys_and_timeframes_leave_the_feed_retryable() {
    let hir = program();
    let mut runtime = seeded(&hir);
    let base = update(360000, 20.0);
    let mut cases = vec![];
    let mut bad = base.clone();
    bad.bar.high = 19.0;
    cases.push(bad);
    let mut bad = base.clone();
    bad.bar.volume = f64::NAN;
    cases.push(bad);
    let mut bad = base.clone();
    bad.symbol = " B".to_owned();
    cases.push(bad);
    let mut bad = base.clone();
    bad.timeframe = "bad".to_owned();
    cases.push(bad);
    let before = public_runtime_result_json(&runtime.result());
    for bad in cases {
        assert!(runtime.apply_request_feed_updates(&[bad]).is_err());
        assert_eq!(public_runtime_result_json(&runtime.result()), before);
    }
    runtime.apply_request_feed_updates(&[]).unwrap();
    runtime.apply_request_feed_updates(&[base]).unwrap();
    runtime
        .update_with_execution_time(BarUpdate::forming(bar(300000, 11.0)), 419999)
        .unwrap();
    assert_eq!(last(&runtime), [3.0, 2.0, 30.0]);
}

#[test]
fn old_provider_updates_keep_automatic_recalculation_and_close_queries_use_calendar_units() {
    let hir = program();
    let mut runtime = seeded(&hir);
    runtime
        .apply_request_update(
            RequestKey::new("B", RequestTimeframe::parse("1").unwrap()),
            BarUpdate::confirmed(bar(360000, 20.0)),
        )
        .unwrap();
    assert_eq!(last(&runtime), [3.0, 2.0, 30.0]);
    let key = |tf| RequestKey::new("B", RequestTimeframe::parse(tf).unwrap());
    assert_eq!(runtime.request_bar_close(&key("30S"), 0).unwrap(), 30000);
    assert_eq!(runtime.request_bar_close(&key("1"), 60000).unwrap(), 120000);
    assert_eq!(
        runtime
            .request_bar_close(&key("1M"), 1704067200000)
            .unwrap(),
        1706745600000
    );
    assert_eq!(
        runtime
            .request_bar_close(&key("1M"), 1706745600000)
            .unwrap(),
        1709251200000
    );
}
