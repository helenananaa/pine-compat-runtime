use pine_runtime::{
    Bar, BarUpdate, HistoricalRuntime, RealtimeRuntime, public_runtime_result_json,
};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

fn program(recalculate: bool) -> pine_ir::HirProgram {
    let source = SourceFile::new(
        "checkpoint.pine",
        format!(
            r#"//@version=6
strategy("Checkpoint state", calc_on_order_fills={recalculate}, calc_on_every_tick=true)
var total = 0.0
total += close
plot(total)
plot(ta.sma(close, 20))
plot(ta.ema(close, 9))
plot(ta.rsi(close, 14))
plot(close[3])
"#
        ),
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    analysis.hir.unwrap()
}

fn bars(count: usize) -> Vec<Bar> {
    (0..count)
        .map(|i| {
            let close = 100.0 + (i % 11) as f64;
            Bar {
                time: i as i64 * 60000,
                open: close,
                high: close + 1.0,
                low: close - 1.0,
                close,
                volume: 10.0,
            }
        })
        .collect()
}

#[test]
fn single_pass_state_matches_checkpoint_path_and_keeps_rolling_storage_bounded() {
    let with = program(true);
    let without = program(false);
    let mut reference = HistoricalRuntime::new(&with);
    let mut candidate = HistoricalRuntime::new(&without);
    for bar in bars(2048) {
        reference.append_bar(bar).unwrap();
        candidate.append_bar(bar).unwrap();
    }
    assert_eq!(
        public_runtime_result_json(&candidate.result()),
        public_runtime_result_json(&reference.result())
    );
    let profile = candidate.profile();
    assert_eq!(profile.strategy_recalculation_passes, 0);
    assert_eq!(
        profile.rolling_window_values,
        reference.profile().rolling_window_values
    );
    assert!(profile.rolling_window_values <= 64);
    assert!(profile.rolling_window_value_capacity <= 128);
}

#[test]
fn repeated_forming_updates_restore_var_and_stateful_calls_with_or_without_fill_checkpoint() {
    let with = program(true);
    let without = program(false);
    let input = bars(65);
    let mut reference = RealtimeRuntime::new(&with);
    let mut candidate = RealtimeRuntime::new(&without);
    reference.seed_historical(&input[..64]).unwrap();
    candidate.seed_historical(&input[..64]).unwrap();
    for i in 0..100 {
        let mut bar = input[64];
        bar.close = if i % 2 == 0 { bar.high } else { bar.low };
        let a = reference.update(BarUpdate::forming(bar)).unwrap();
        let b = candidate.update(BarUpdate::forming(bar)).unwrap();
        assert_eq!(
            public_runtime_result_json(&a),
            public_runtime_result_json(&b)
        );
    }
    let a = reference.update(BarUpdate::confirmed(input[64])).unwrap();
    let b = candidate.update(BarUpdate::confirmed(input[64])).unwrap();
    assert_eq!(
        public_runtime_result_json(&a),
        public_runtime_result_json(&b)
    );
    assert_eq!(candidate.confirmed_profile().bars, 65);
}
