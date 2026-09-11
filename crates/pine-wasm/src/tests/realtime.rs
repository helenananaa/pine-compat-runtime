use super::*;
use pine_runtime::{
    Bar, BarUpdate, PUBLIC_RUNTIME_CHANGES_SCHEMA_VERSION, RealtimeRuntime,
    public_runtime_result_json,
};
use pine_sema::{AnalysisInput, analyze_input};
use pine_syntax::SourceFile;
use serde_json::Value;

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

fn bar_json(time: i64, close: f64) -> String {
    format!(
        r#"{{"time":{time},"open":{close},"high":{close},"low":{close},"close":{close},"volume":1}}"#
    )
}

fn bars_csv(bars: &[Bar]) -> String {
    let mut output = String::from("time,open,high,low,close,volume\n");
    for bar in bars {
        output.push_str(&format!(
            "{},{},{},{},{},{}\n",
            bar.time, bar.open, bar.high, bar.low, bar.close, bar.volume
        ));
    }
    output
}

fn hir(source: &str) -> pine_ir::HirProgram {
    let analysis = analyze_input(&AnalysisInput::new(SourceFile::new("<wasm>", source)));
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    analysis.hir.expect("hir")
}

fn plot_values(json: &str) -> Value {
    let parsed: Value = serde_json::from_str(json).expect("result json");
    parsed["plots"][0]["values"].clone()
}

#[test]
fn wasm_realtime_session_matches_rust_event_sequence() {
    let source = "//@version=6\nindicator(\"wasm stream\")\nplot(close)\n";
    let seed = [bar(60_000, 10.0)];
    let forming = bar(120_000, 12.0);
    let replacement = bar(120_000, 13.0);
    let confirmed = bar(120_000, 13.0);

    let mut rust = RealtimeRuntime::from_program(hir(source));
    let rust_seed = public_runtime_result_json(&rust.seed_historical(&seed).expect("rust seed"));
    let rust_forming = public_runtime_result_json(
        &rust
            .update(BarUpdate::forming(forming))
            .expect("rust forming"),
    );
    let mut rust_changes = RealtimeRuntime::from_program(hir(source));
    rust_changes
        .seed_historical(&seed)
        .expect("rust changes seed");
    let rust_apply_forming = rust_changes
        .apply_update(BarUpdate::forming(forming))
        .expect("rust apply forming");
    let rust_apply_replace = rust_changes
        .apply_update(BarUpdate::forming(replacement))
        .expect("rust apply replace");
    let rust_apply_confirm = rust_changes
        .apply_update(BarUpdate::confirmed(confirmed))
        .expect("rust apply confirm");
    let rust_confirmed = public_runtime_result_json(&rust_changes.result());

    let program = compile_script(source).expect("program");
    let mut session = program.realtime_session().expect("session");
    assert_eq!(session.schema_version(), 1);
    assert_eq!(
        runtime_changes_schema_version(),
        PUBLIC_RUNTIME_CHANGES_SCHEMA_VERSION
    );
    let wasm_seed = session.seed(&bars_csv(&seed)).expect("wasm seed");
    assert_eq!(wasm_seed, rust_seed);
    assert_eq!(plot_values(&wasm_seed), serde_json::json!([10]));

    let mut replica = session.replica();
    let forming_changes = session
        .apply_forming(&bar_json(forming.time, forming.close))
        .expect("wasm forming");
    let parsed_forming: Value = serde_json::from_str(&forming_changes).expect("forming json");
    assert_eq!(parsed_forming["visibility"], "preview");
    assert_eq!(
        parsed_forming["schemaVersion"],
        PUBLIC_RUNTIME_CHANGES_SCHEMA_VERSION
    );
    assert_eq!(parsed_forming["series"][0]["op"], "append");
    assert_eq!(
        parsed_forming["series"][0]["values"],
        serde_json::json!([12])
    );
    assert!(
        replica
            .apply_internal(&forming_changes)
            .expect("apply forming")
    );
    assert_eq!(replica.result(), session.result());
    assert!(
        !replica
            .apply_internal(&forming_changes)
            .expect("retransmit forming")
    );

    let rust_forming_via_apply = {
        let mut control = RealtimeRuntime::from_program(hir(source));
        control.seed_historical(&seed).expect("control seed");
        public_runtime_result_json(
            &control
                .update(BarUpdate::forming(forming))
                .expect("control forming"),
        )
    };
    assert_eq!(session.result(), rust_forming_via_apply);
    assert_eq!(session.result(), rust_forming);

    let replace_changes = session
        .apply_forming(&bar_json(replacement.time, replacement.close))
        .expect("wasm replace");
    let parsed_replace: Value = serde_json::from_str(&replace_changes).expect("replace json");
    assert_eq!(parsed_replace["series"][0]["op"], "replaceLast");
    assert_eq!(
        parsed_replace["series"][0]["values"],
        serde_json::json!([13])
    );
    assert!(
        replica
            .apply_internal(&replace_changes)
            .expect("apply replace")
    );
    assert_eq!(replica.result(), session.result());

    let confirm_changes = session
        .apply_confirmed(&bar_json(confirmed.time, confirmed.close))
        .expect("wasm confirm");
    let parsed_confirm: Value = serde_json::from_str(&confirm_changes).expect("confirm json");
    assert_eq!(parsed_confirm["visibility"], "confirmed");
    assert!(
        replica
            .apply_internal(&confirm_changes)
            .expect("apply confirm")
    );
    assert_eq!(replica.result(), session.result());
    assert_eq!(session.result(), session.confirmed_result());
    assert_eq!(session.result(), rust_confirmed);
    assert_eq!(
        pine_runtime::public_runtime_changes_json(&rust_apply_forming),
        forming_changes
    );
    assert_eq!(
        pine_runtime::public_runtime_changes_json(&rust_apply_replace),
        replace_changes
    );
    assert_eq!(
        pine_runtime::public_runtime_changes_json(&rust_apply_confirm),
        confirm_changes
    );
}

#[test]
fn wasm_replica_gap_recovery_matches_session_snapshot() {
    let source = "//@version=6\nindicator(\"cursor\")\nplot(close)\n";
    let program = compile_script(source).expect("program");
    let mut session = program.realtime_session().expect("session");
    session.seed(&bars_csv(&[bar(0, 10.0)])).expect("seed");
    let mut replica = session.replica();
    let first = session
        .apply_forming(&bar_json(60_000, 20.0))
        .expect("first");
    let second = session
        .apply_forming(&bar_json(60_000, 30.0))
        .expect("second");
    let before = replica.result();
    let gap = replica.apply_internal(&second);
    assert!(gap.is_err(), "missing first revision must fail");
    assert_eq!(replica.result(), before);
    assert!(replica.apply_internal(&first).expect("apply first"));
    assert!(!replica.apply_internal(&first).expect("retransmit first"));
    assert!(replica.apply_internal(&second).expect("apply second"));

    session
        .apply_forming(&bar_json(60_000, 40.0))
        .expect("third");
    let missed = session
        .apply_forming(&bar_json(60_000, 50.0))
        .expect("fourth");
    assert!(replica.apply_internal(&missed).is_err());
    let snapshot: Value = serde_json::from_str(&session.stream_snapshot()).expect("snapshot json");
    replica
        .reset_internal(
            &snapshot["result"].to_string(),
            snapshot["revision"].as_u64().expect("revision"),
            snapshot["retainedFrom"].as_u64().expect("retainedFrom") as u32,
        )
        .expect("reset");
    let confirmed = session
        .apply_confirmed(&bar_json(60_000, 60.0))
        .expect("confirm");
    assert!(
        replica
            .apply_internal(&confirmed)
            .expect("apply after reset")
    );
    assert_eq!(replica.result(), session.result());
}

#[test]
fn wasm_request_forming_does_not_leak_into_confirmed_chart() {
    let source = r#"//@version=6
indicator("request stream")
plot(request.security("NYSE:IBM", timeframe.period, close))
"#;
    let request_bars =
        r#"{"NYSE:IBM:1":[{"time":0,"open":20,"high":20,"low":20,"close":20,"volume":1}]}"#;
    let program = compile_script(source).expect("program");
    let mut session = program
        .realtime_session_with_request_bars(request_bars)
        .expect("session");
    session.seed(&bars_csv(&[bar(0, 5.0)])).expect("seed");
    let forming = session
        .apply_request_forming("NYSE:IBM", "1", &bar_json(60_000, 99.0))
        .expect("request forming");
    assert_eq!(forming, "null");
    session
        .apply_forming(&bar_json(60_000, 6.0))
        .expect("chart forming");
    assert_eq!(plot_values(&session.result()), serde_json::json!([20, 99]));
    assert_eq!(
        plot_values(&session.confirmed_result()),
        serde_json::json!([20])
    );
    session
        .apply_request_confirmed("NYSE:IBM", "1", &bar_json(60_000, 99.0))
        .expect("request confirm");
    session
        .apply_confirmed(&bar_json(60_000, 6.0))
        .expect("chart confirm");
    assert_eq!(plot_values(&session.result()), serde_json::json!([20, 99]));
    session
        .apply_request_forming("NYSE:IBM", "1", &bar_json(120_000, 50.0))
        .expect("next request forming");
    session
        .apply_forming(&bar_json(120_000, 7.0))
        .expect("next chart forming");
    assert_eq!(
        plot_values(&session.result()),
        serde_json::json!([20, 99, 50])
    );
    session
        .apply_confirmed(&bar_json(120_000, 7.0))
        .expect("chart confirm ignores unconfirmed request bar");
    assert_eq!(
        plot_values(&session.result()),
        serde_json::json!([20, 99, 99])
    );
}

#[test]
fn wasm_replay_matches_fresh_seed_and_requires_replica_reset() {
    let source = "//@version=6\nindicator(\"wasm replay\")\nplot(close)\n";
    let program = compile_script(source).expect("program");
    let mut session = program.realtime_session().expect("session");
    session
        .seed(&bars_csv(&[bar(0, 10.0), bar(60_000, 20.0)]))
        .expect("seed");
    let mut replica = session.replica();
    let forming = session
        .apply_forming(&bar_json(120_000, 30.0))
        .expect("forming");
    replica.apply_internal(&forming).expect("apply forming");
    let replayed = session
        .replay_internal(&bars_csv(&[bar(0, 11.0), bar(60_000, 21.0)]), None)
        .expect("replay");
    assert_eq!(plot_values(&replayed), serde_json::json!([11, 21]));
    assert_eq!(session.forming_time(), None);
    assert_eq!(session.confirmed_bars(), 2);
    assert_ne!(replica.revision(), session.revision());
    assert!(
        !replica
            .apply_internal(&forming)
            .expect("old forming is a retransmission")
    );
    let next = session
        .apply_forming(&bar_json(120_000, 31.0))
        .expect("next forming");
    assert!(replica.apply_internal(&next).is_err());
    let snapshot: Value = serde_json::from_str(&session.stream_snapshot()).expect("snapshot");
    replica
        .reset_internal(
            &snapshot["result"].to_string(),
            snapshot["revision"].as_u64().expect("revision"),
            snapshot["retainedFrom"].as_u64().expect("retainedFrom") as u32,
        )
        .expect("reset");
    assert_eq!(replica.result(), session.result());

    let mut rust = RealtimeRuntime::from_program(hir(source));
    rust.seed_historical(&[bar(0, 10.0), bar(60_000, 20.0)])
        .expect("rust seed");
    rust.apply_update(BarUpdate::forming(bar(120_000, 30.0)))
        .expect("rust forming");
    let rust_replayed = rust
        .replay_historical(&[bar(0, 11.0), bar(60_000, 21.0)])
        .expect("rust replay");
    assert_eq!(replayed, public_runtime_result_json(&rust_replayed));
}

#[test]
fn wasm_correct_from_keeps_prefix_and_matches_rust() {
    let source = "//@version=6\nindicator(\"wasm correct\")\nplot(close)\n";
    let program = compile_script(source).expect("program");
    let mut session = program.realtime_session().expect("session");
    session
        .seed(&bars_csv(&[
            bar(0, 10.0),
            bar(60_000, 20.0),
            bar(120_000, 30.0),
        ]))
        .expect("seed");
    session
        .apply_forming(&bar_json(180_000, 40.0))
        .expect("forming");
    let corrected = session
        .correct_internal(
            60_000,
            &bars_csv(&[bar(60_000, 21.0), bar(120_000, 31.0)]),
            None,
        )
        .expect("correct");
    assert_eq!(plot_values(&corrected), serde_json::json!([10, 21, 31]));
    assert_eq!(session.forming_time(), None);
    assert_eq!(session.confirmed_bars(), 3);

    let mut rust = RealtimeRuntime::from_program(hir(source));
    rust.seed_historical(&[bar(0, 10.0), bar(60_000, 20.0), bar(120_000, 30.0)])
        .expect("rust seed");
    let rust_corrected = rust
        .correct_historical(60_000, &[bar(60_000, 21.0), bar(120_000, 31.0)])
        .expect("rust correct");
    assert_eq!(corrected, public_runtime_result_json(&rust_corrected));
}
