use super::*;

fn program(text: &str) -> pine_ir::HirProgram {
    let analysis = analyze_source(&modern_source_file("external", text));
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    analysis.hir.unwrap()
}

fn account(time: i64, qty: f64) -> ExternalAccountFrame {
    ExternalAccountFrame {
        time,
        position_size: qty,
        position_avg_price: (qty != 0.0).then_some(10.0),
        equity: 1000.0,
        initial_capital: 1000.0,
        netprofit: 0.0,
        openprofit: 0.0,
    }
}

#[test]
fn external_generic_orders_keep_fixed_quantity_and_oca_without_native_execution() {
    let hir = program(
        "strategy(\"Orders\",pyramiding=1)\nstrategy.order(\"sell\",strategy.short,qty=3,limit=11,stop=9,oca_name=\"g\",oca_type=strategy.oca.reduce)\nstrategy.entry(\"buy\",strategy.long,qty=2,oca_name=\"g\",oca_type=strategy.oca.cancel)\nplot(strategy.position_size)",
    );
    let mut runtime = HistoricalRuntime::new(&hir)
        .with_external_accounts(vec![account(0, 4.0)])
        .unwrap();
    runtime.append_bar(bar(10.0)).unwrap();
    let intents = runtime.external_intents().unwrap();
    assert_eq!(intents[0].action, "order");
    assert_eq!(intents[0].qty, Some(3.0));
    assert_eq!(intents[0].oca_name.as_deref(), Some("g"));
    assert_eq!(intents[0].oca_type.as_deref(), Some("reduce"));
    assert_eq!(intents[1].oca_type.as_deref(), Some("cancel"));
    assert_eq!(
        runtime.result().plots[0].values,
        vec![PineValue::Float(4.0)]
    );
    assert!(runtime.result().strategy.is_none());
}

#[test]
fn external_append_clock_matches_batch_and_preflight_keeps_feedback_unmodified() {
    let hir = program(
        "strategy(\"Clock\")\nplot(timenow)\nplot(strategy.position_size)\nif timenow > 100000\n    strategy.entry(\"L\", strategy.long)",
    );
    let bars = [
        bar(10.0),
        Bar {
            time: 60000,
            ..bar(10.0)
        },
    ];
    let clocks = [59999, 119999];
    let mut batch = HistoricalRuntime::new(&hir)
        .with_external_accounts(vec![account(0, 0.0), account(60000, 2.0)])
        .unwrap();
    batch
        .append_bars_with_execution_times(&bars, &clocks)
        .unwrap();
    let mut session = HistoricalRuntime::new(&hir)
        .with_external_accounts(vec![])
        .unwrap();
    assert!(
        session
            .append_external_bars_with_execution_times(&bars[..1], vec![account(0, 0.0)], None, &[])
            .is_err()
    );
    for (index, time) in [0, 60000].into_iter().enumerate() {
        session
            .append_external_bars_with_execution_times(
                &bars[index..index + 1],
                vec![account(time, index as f64 * 2.0)],
                None,
                &clocks[index..index + 1],
            )
            .unwrap();
    }
    assert_eq!(session.result().plots, batch.result().plots);
    assert_eq!(
        serde_json::to_string(session.external_intents().unwrap()).unwrap(),
        serde_json::to_string(batch.external_intents().unwrap()).unwrap()
    );
    assert_eq!(
        session.result().plots[0].values,
        vec![PineValue::Int(59999), PineValue::Int(119999)]
    );
    assert!(session.result().strategy.is_none());
}

#[test]
fn external_mode_never_exposes_a_native_ledger_on_any_result_surface() {
    let hir = program(
        "strategy(\"External\")\nstrategy.entry(\"L\", strategy.long)\nplot(strategy.position_size[1])",
    );
    let bars = [
        bar(10.0),
        Bar {
            time: 60000,
            ..bar(10.0)
        },
    ];
    let mut runtime = HistoricalRuntime::new(&hir)
        .with_external_accounts(vec![account(0, 2.0), account(60000, 0.0)])
        .unwrap();
    runtime.append_bars(&bars).unwrap();
    assert!(runtime.result().strategy.is_none());
    assert!(runtime.result_view().strategy.is_none());
    assert_eq!(
        runtime.result().plots[0].values,
        vec![PineValue::Na, PineValue::Float(2.0)]
    );
    assert_eq!(runtime.external_intents().unwrap().len(), 2);
    assert!(!public_runtime_result_json(&runtime.result()).contains("\"strategy\""));
}

#[test]
fn unsupported_account_read_is_an_execution_error_and_disables_reuse() {
    let hir = program("strategy(\"Unsupported\")\nplot(strategy.wintrades)");
    let mut runtime = HistoricalRuntime::new(&hir)
        .with_external_accounts(vec![account(0, 0.0)])
        .unwrap();
    assert!(
        runtime
            .append_bar(bar(10.0))
            .unwrap_err()
            .message
            .contains("E_EXTERNAL_UNSUPPORTED")
    );
    assert!(
        runtime
            .append_bar(bar(10.0))
            .unwrap_err()
            .message
            .contains("E_RUNTIME_POISONED")
    );
}

#[test]
fn historical_horizon_can_only_be_set_before_execution() {
    let hir = program("indicator(\"Horizon\")\nplot(last_bar_index)");
    let mut runtime = HistoricalRuntime::new(&hir);
    runtime.set_historical_horizon(2, 60000).unwrap();
    runtime.append_bar(bar(10.0)).unwrap();
    let prior = runtime.result();
    assert!(runtime.set_historical_horizon(3, 120000).is_err());
    assert_eq!(runtime.result(), prior);
    runtime
        .append_bar(Bar {
            time: 60000,
            ..bar(10.0)
        })
        .unwrap();
    assert_eq!(
        runtime.result().plots[0].values,
        vec![PineValue::Int(1), PineValue::Int(1)]
    );
}
