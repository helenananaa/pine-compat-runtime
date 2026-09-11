use std::path::PathBuf;

use pine_runtime::{
    AlertEvent, Bar, BarUpdate, PineValue, RealtimeRuntime, RuntimeChanges, RuntimeReplica,
    SeriesChangeOp, StreamingVisibility,
};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

fn apply_runtime_changes(
    base: &pine_runtime::RuntimeResult,
    changes: &RuntimeChanges,
) -> pine_runtime::RuntimeResult {
    let mut replica = RuntimeReplica::new(base.clone(), changes.base_revision);
    replica.apply(changes).unwrap();
    replica.result().clone()
}

fn workspace_fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn runtime_for_source(source: &str) -> RealtimeRuntime<'static> {
    let hir = analyze_source(&SourceFile::new("streaming.pine", source))
        .hir
        .unwrap();
    RealtimeRuntime::from_program(hir)
}

fn runtime_for_fixture(path: &str) -> RealtimeRuntime<'static> {
    let path = workspace_fixture(path);
    let source = std::fs::read_to_string(&path).expect("fixture");
    let hir = analyze_source(&SourceFile::new(path.display().to_string(), source))
        .hir
        .unwrap();
    RealtimeRuntime::from_program(hir)
}

fn bar(time: i64, close: f64) -> Bar {
    Bar {
        time,
        open: close,
        high: close,
        low: close,
        close,
        volume: 1.0,
    }
}

fn assert_visible_matches_applied(
    visible: &pine_runtime::RuntimeResult,
    changes: &RuntimeChanges,
    runtime: &RealtimeRuntime<'_>,
) -> pine_runtime::RuntimeResult {
    let applied = apply_runtime_changes(visible, changes);
    let snapshot = runtime.result();
    assert_eq!(applied, snapshot);
    let mut replica = RuntimeReplica::new(visible.clone(), changes.base_revision);
    replica.apply(changes).unwrap();
    assert!(!replica.apply(changes).unwrap());
    assert_eq!(replica.result(), &snapshot);
    applied
}

fn assert_series_is_delta(changes: &RuntimeChanges) {
    for change in &changes.series {
        let len = change
            .fields
            .values
            .len()
            .max(change.fields.closes.len())
            .max(change.fields.colors.len());
        assert!(
            len <= 1,
            "streaming series change for {:?} id {} reconstructed {} values",
            change.family,
            change.id,
            len
        );
        assert!(matches!(
            change.op,
            SeriesChangeOp::Append | SeriesChangeOp::ReplaceLast
        ));
    }
}

#[test]
fn streaming_plot_forming_and_confirm_match_snapshots() {
    let mut runtime = runtime_for_source(
        "//@version=6\nindicator(\"stream plots\")\nplot(close, color=close>10 ? color.red : color.blue)\n",
    );
    let mut visible = runtime
        .seed_historical(&[bar(0, 10.0)])
        .expect("seed should run");
    assert_eq!(visible.plots[0].values.len(), 1);
    assert_eq!(runtime.revision(), 1);

    let first = runtime
        .apply_update(BarUpdate::forming(bar(60_000, 20.0)))
        .expect("forming update");
    assert_eq!(first.visibility, StreamingVisibility::Preview);
    assert_eq!(first.revision, 2);
    assert_series_is_delta(&first);
    assert_eq!(first.series[0].op, SeriesChangeOp::Append);
    assert_eq!(first.series[0].fields.values, vec![PineValue::Float(20.0)]);
    visible = assert_visible_matches_applied(&visible, &first, &runtime);
    assert_eq!(runtime.confirmed_result().plots[0].values.len(), 1);

    let replacement = runtime
        .apply_update(BarUpdate::forming(bar(60_000, 5.0)))
        .expect("forming replacement");
    assert_eq!(replacement.visibility, StreamingVisibility::Preview);
    assert_eq!(replacement.revision, 3);
    assert_series_is_delta(&replacement);
    assert_eq!(replacement.series[0].op, SeriesChangeOp::ReplaceLast);
    assert_eq!(
        replacement.series[0].fields.values,
        vec![PineValue::Float(5.0)]
    );
    visible = assert_visible_matches_applied(&visible, &replacement, &runtime);

    let confirmed = runtime
        .apply_update(BarUpdate::confirmed(bar(60_000, 5.0)))
        .expect("confirmed update");
    assert_eq!(confirmed.visibility, StreamingVisibility::Confirmed);
    assert_series_is_delta(&confirmed);
    visible = assert_visible_matches_applied(&visible, &confirmed, &runtime);
    assert_eq!(visible, runtime.confirmed_result());
    assert_eq!(runtime.last_changes(), Some(&confirmed));
}

#[test]
fn streaming_label_create_mutate_and_delete_match_snapshots() {
    let mut runtime = runtime_for_fixture("tests/fixtures/realtime/label_rollback.pine");
    let mut visible = runtime
        .update(BarUpdate::historical(bar(0, 1.0)))
        .expect("historical seed");
    assert_eq!(visible.labels.len(), 1);

    let forming = runtime
        .apply_update(BarUpdate::forming(bar(60_000, 2.0)))
        .expect("forming labels");
    assert!(
        forming
            .drawings
            .iter()
            .any(|change| change.action.as_str() == "add")
    );
    assert!(
        forming
            .drawings
            .iter()
            .any(|change| change.action.as_str() == "setTail")
    );
    visible = assert_visible_matches_applied(&visible, &forming, &runtime);
    assert_eq!(visible.labels.len(), 2);

    let replacement = runtime
        .apply_update(BarUpdate::forming(bar(60_000, 3.0)))
        .expect("replacement labels");
    visible = assert_visible_matches_applied(&visible, &replacement, &runtime);
    assert_eq!(visible.labels.len(), 2);
    assert!(!visible.labels[0].snapshots[1].exists);

    let confirmed = runtime
        .apply_update(BarUpdate::confirmed(bar(60_000, 4.0)))
        .expect("confirmed labels");
    assert!(
        confirmed
            .drawings
            .iter()
            .any(|change| change.action.as_str() == "delete")
    );
    visible = assert_visible_matches_applied(&visible, &confirmed, &runtime);
    assert_eq!(visible.labels.len(), 1);
    assert_eq!(visible, runtime.confirmed_result());
}

#[test]
fn streaming_alerts_and_strategy_fills_do_not_duplicate_on_reread() {
    let mut alerts = runtime_for_source(
        r#"//@version=6
indicator("stream alerts")
if close > 2
    alert("high")
plot(close)
"#,
    );
    let mut visible = alerts.seed_historical(&[bar(0, 1.0)]).expect("alert seed");
    let mut alert_replica = alerts.replica();
    let forming = alerts
        .apply_update(BarUpdate::forming(bar(60_000, 3.0)))
        .expect("alert forming");
    assert!(
        forming
            .alerts
            .iter()
            .any(|change| change.action.as_str() == "add")
    );
    visible = assert_visible_matches_applied(&visible, &forming, &alerts);
    assert_eq!(visible.alerts.len(), 1);
    let reread = alerts.last_changes().cloned().expect("last alert changes");
    alert_replica.apply(&forming).unwrap();
    assert!(!alert_replica.apply(&reread).unwrap());
    assert_eq!(alert_replica.result(), &visible);
    assert_eq!(visible.alerts.len(), 1);

    let quiet = alerts
        .apply_update(BarUpdate::forming(bar(60_000, 1.0)))
        .expect("alert rollback");
    assert!(
        quiet
            .alerts
            .iter()
            .any(|change| change.action.as_str() == "remove")
    );
    visible = assert_visible_matches_applied(&visible, &quiet, &alerts);
    assert!(visible.alerts.is_empty());

    let mut strategy = runtime_for_source(
        r#"//@version=6
strategy("stream fills")
if bar_index == 1
    strategy.entry("L", strategy.long, qty=1)
plot(close)
"#,
    );
    let mut strategy_visible = strategy
        .seed_historical(&[bar(0, 1.0)])
        .expect("strategy seed");
    let forming = strategy
        .apply_update(BarUpdate::forming(bar(60_000, 2.0)))
        .expect("strategy forming");
    strategy_visible = assert_visible_matches_applied(&strategy_visible, &forming, &strategy);
    let mut repeated = RuntimeReplica::new(strategy_visible.clone(), strategy.revision());
    let confirmed = strategy
        .apply_update(BarUpdate::confirmed(bar(60_000, 2.0)))
        .expect("strategy confirm");
    strategy_visible = assert_visible_matches_applied(&strategy_visible, &confirmed, &strategy);
    let orders = strategy_visible
        .strategy
        .as_ref()
        .map(|item| item.orders.len())
        .unwrap_or(0);
    let fills = strategy_visible
        .strategy
        .as_ref()
        .map(|item| item.alerts.len())
        .unwrap_or(0);
    repeated.apply(&confirmed).unwrap();
    assert!(!repeated.apply(&confirmed).unwrap());
    let replayed = repeated.result().clone();
    assert_eq!(
        replayed.strategy.as_ref().map(|item| item.orders.len()),
        Some(orders)
    );
    assert_eq!(
        replayed.strategy.as_ref().map(|item| item.alerts.len()),
        Some(fills)
    );
    assert_eq!(replayed, strategy.result());
}

#[test]
fn streaming_preserves_var_varip_failure_isolation_and_caller_independence() {
    let mut runtime = runtime_for_source(
        r#"//@version=6
indicator("stream var")
var float regular = 0.0
varip float intrabar = 0.0
regular += 1.0
intrabar += 1.0
plot(regular)
plot(intrabar)
"#,
    );
    let mut visible = runtime.seed_historical(&[bar(0, 1.0)]).expect("var seed");
    let first = runtime
        .apply_update(BarUpdate::forming(bar(60_000, 2.0)))
        .expect("var forming");
    visible = assert_visible_matches_applied(&visible, &first, &runtime);
    assert_eq!(
        visible.plots[0].values,
        vec![PineValue::Float(1.0), PineValue::Float(2.0)]
    );
    assert_eq!(
        visible.plots[1].values,
        vec![PineValue::Float(1.0), PineValue::Float(2.0)]
    );

    let second = runtime
        .apply_update(BarUpdate::forming(bar(60_000, 3.0)))
        .expect("varip persist");
    visible = assert_visible_matches_applied(&visible, &second, &runtime);
    assert_eq!(visible.plots[0].values.last(), Some(&PineValue::Float(2.0)));
    assert_eq!(visible.plots[1].values.last(), Some(&PineValue::Float(3.0)));
    assert_eq!(
        runtime.confirmed_result().plots[0].values,
        vec![PineValue::Float(1.0)]
    );

    let handed = runtime.result();
    let handed_json = pine_runtime::public_runtime_result_json(&handed);
    let mut caller = apply_runtime_changes(&visible, &second);
    caller.plots[0].values[0] = PineValue::Float(999.0);
    caller.alerts.push(AlertEvent {
        id: 99,
        bar_index: 0,
        time: 0,
        message: "caller".to_owned(),
        source: "caller".to_owned(),
    });
    assert_eq!(
        pine_runtime::public_runtime_result_json(&runtime.result()),
        handed_json
    );

    let mut failing = runtime_for_source(
        r#"//@version=6
indicator("last-bar resource limit")
if bar_index >= 2
    while true
        x = close
plot(close)
"#,
    );
    let seed = failing
        .seed_historical(&[bar(60_000, 1.0), bar(120_000, 2.0)])
        .expect("prefix");
    let seed_json = pine_runtime::public_runtime_result_json(&seed);
    let before = pine_runtime::public_runtime_result_json(&failing.result());
    let revision = failing.revision();
    let error = failing
        .apply_update(BarUpdate::forming(bar(180_000, 3.0)))
        .expect_err("iteration ceiling");
    assert!(error.message.contains("exceeded maximum iteration"));
    assert_eq!(failing.revision(), revision);
    assert_eq!(
        pine_runtime::public_runtime_result_json(&failing.result()),
        before
    );
    assert_eq!(
        pine_runtime::public_runtime_result_json(&failing.confirmed_result()),
        seed_json
    );
    assert_eq!(pine_runtime::public_runtime_result_json(&seed), seed_json);
}

#[test]
fn compatibility_update_still_returns_complete_snapshot() {
    let mut runtime = runtime_for_source("//@version=6\nindicator(\"compat\")\nplot(close)\n");
    let seed = runtime.seed_historical(&[bar(0, 10.0)]).expect("seed");
    assert_eq!(seed.plots[0].values.len(), 1);
    let forming = runtime
        .update(BarUpdate::forming(bar(60_000, 20.0)))
        .expect("compat forming");
    assert_eq!(forming.plots[0].values.len(), 2);
    assert!(runtime.last_changes().is_none());
    let streaming = runtime
        .apply_update(BarUpdate::forming(bar(60_000, 21.0)))
        .expect("stream after compat");
    assert_series_is_delta(&streaming);
    assert_eq!(streaming.series[0].op, SeriesChangeOp::ReplaceLast);
    assert_eq!(
        apply_runtime_changes(&forming, &streaming),
        runtime.result()
    );
}
