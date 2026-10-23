use std::sync::Arc;

use pine_runtime::{Bar, ExternalAccountFrame, ExternalPass, HistoricalRuntime, PreparedProgram};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

fn program(source: &str) -> PreparedProgram {
    let analysis = analyze_source(&SourceFile::new("external-session", source));
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    PreparedProgram::from_shared_hir(Arc::new(analysis.hir.unwrap()))
}

fn bar(index: i64) -> Bar {
    Bar {
        time: index * 60_000,
        open: 10.0,
        high: 11.0,
        low: 9.0,
        close: 10.0,
        volume: 1.0,
    }
}

fn account(index: i64) -> ExternalAccountFrame {
    ExternalAccountFrame {
        time: bar(index).time,
        position_size: (index % 2) as f64,
        position_avg_price: (index % 2 != 0).then_some(10.0),
        equity: 1_000.0 + index as f64,
        initial_capital: 1_000.0,
        netprofit: index as f64,
        openprofit: 0.0,
    }
}

#[test]
fn incremental_external_state_matches_prefix_stable_batch_execution() {
    let program = program(
        "//@version=6\nstrategy(\"External\")\nvar float total = 0\ntotal += strategy.position_size\nplot(total)\nplot(ta.sma(close, 2))\nplot(strategy.position_size[1])\nstrategy.entry(\"L\", strategy.long)",
    );
    let mut incremental = HistoricalRuntime::from_prepared(&program)
        .with_external_accounts(vec![])
        .unwrap();
    for index in 0..4 {
        incremental
            .append_external_bars(&[bar(index)], vec![account(index)], None)
            .unwrap();
        let mut batch = HistoricalRuntime::from_prepared(&program)
            .with_external_accounts((0..=index).map(account).collect())
            .unwrap();
        batch
            .append_bars(&(0..=index).map(bar).collect::<Vec<_>>())
            .unwrap();
        assert_eq!(incremental.result(), batch.result());
        assert_eq!(
            serde_json::to_value(incremental.external_intents().unwrap()).unwrap(),
            serde_json::to_value(batch.external_intents().unwrap()).unwrap(),
        );
    }
}

#[test]
fn empty_append_is_a_noop_for_fill_pass_sessions() {
    let program = program(
        "//@version=6\nstrategy(\"Passes\", calc_on_order_fills=true)\nplot(strategy.equity)",
    );
    let mut runtime = HistoricalRuntime::from_prepared(&program)
        .with_external_accounts_and_passes(vec![], Some(vec![]))
        .unwrap();
    let empty = runtime.result();
    runtime.append_external_bars(&[], vec![], None).unwrap();
    assert_eq!(runtime.result(), empty);
    runtime
        .append_external_bars(
            &[bar(0)],
            vec![account(0)],
            Some(vec![vec![ExternalPass {
                account: account(0),
                bar: bar(0),
                event_time_ms: 1,
                confirmed: false,
                request_data: None,
            }]]),
        )
        .unwrap();
    let forming = runtime.result();
    runtime.append_external_bars(&[], vec![], None).unwrap();
    assert_eq!(runtime.result(), forming);
    assert!(
        runtime
            .append_external_bars(&[bar(1)], vec![account(1)], Some(vec![vec![]]))
            .is_err()
    );
}

#[test]
fn pending_feedback_cannot_be_silently_consumed_by_an_append() {
    let program = program("//@version=6\nstrategy(\"Pending\")\nplot(strategy.equity)");
    let mut runtime = HistoricalRuntime::from_prepared(&program)
        .with_external_accounts(vec![account(0)])
        .unwrap();
    let mut replacement = account(0);
    replacement.equity = 2_000.0;
    assert!(
        runtime
            .append_external_bars(&[bar(0)], vec![replacement], None)
            .unwrap_err()
            .message
            .contains("pending feedback")
    );
    runtime.append_bars(&[bar(0)]).unwrap();
    runtime
        .append_external_bars(&[bar(1)], vec![account(1)], None)
        .unwrap();
    assert_eq!(runtime.result().plots[0].values.len(), 2);
}

#[test]
fn execution_failure_after_binding_feedback_disables_reuse() {
    let program = program(
        "//@version=6\nstrategy(\"Missing FX\", currency=currency.EUR)\nplot(strategy.equity)",
    );
    let mut runtime = HistoricalRuntime::from_prepared(&program)
        .with_external_accounts(vec![])
        .unwrap();
    assert!(
        runtime
            .append_external_bars(&[bar(0)], vec![account(0)], None)
            .is_err()
    );
    assert!(
        runtime
            .append_external_bars(&[bar(1)], vec![account(1)], None)
            .unwrap_err()
            .message
            .contains("E_RUNTIME_POISONED")
    );
}
