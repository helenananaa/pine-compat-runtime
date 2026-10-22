use pine_runtime::{Bar, ChartContext, HistoricalRuntime, RequestEnvironment, RuntimeResult};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

fn program(source: &str) -> pine_ir::HirProgram {
    let analysis = analyze_source(&SourceFile::new("contract-quantity.pine", source));
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    analysis.hir.unwrap()
}

fn bars(prices: &[f64]) -> Vec<Bar> {
    prices
        .iter()
        .enumerate()
        .map(|(i, &price)| Bar {
            time: i as i64 * 60_000,
            open: price,
            high: price,
            low: price,
            close: price,
            volume: 1000.,
        })
        .collect()
}

fn run(chart: ChartContext, source: &str, prices: &[f64]) -> RuntimeResult {
    let hir = program(source);
    let mut runtime = HistoricalRuntime::with_request_environment(
        &hir,
        RequestEnvironment::default().for_chart(chart),
    );
    runtime.append_bars(&bars(prices)).unwrap();
    runtime.result()
}

fn chart() -> ChartContext {
    ChartContext::default().with_quantity_step("0.25").unwrap()
}

#[test]
fn decimal_step_applies_to_explicit_entry_generic_and_micro_contracts() {
    for (step, request, expected) in [
        ("0.25", 1.249999999999, 1.),
        ("0.25", 1.25, 1.25),
        ("0.000001", 0.129515, 0.129515),
        ("2e-12", 7e-12, 6e-12),
    ] {
        for command in ["entry", "order"] {
            let source = format!(
                "//@version=6\nstrategy(\"Grid\",margin_long=0)\nif bar_index==0\n    strategy.{command}(\"L\",strategy.long,qty={request})\nplot(syminfo.mincontract)"
            );
            let result = run(
                ChartContext::default().with_quantity_step(step).unwrap(),
                &source,
                &[100.; 3],
            );
            assert_eq!(
                result.strategy.unwrap().orders[0].qty,
                expected,
                "{step} {command}"
            );
        }
    }
}

#[test]
fn minima_and_legacy_precision_are_separate_consistent_metadata() {
    for blocked in [true, false] {
        let request = if blocked { 1.99 } else { 3.49 };
        let source = format!(
            "//@version=6\nstrategy(\"Minimum\",margin_long=0)\nif bar_index==0\n    strategy.entry(\"L\",strategy.long,qty={request})\nplot(syminfo.mincontract)"
        );
        let result = run(chart().with_min_contract("2").unwrap(), &source, &[100.; 3]);
        assert_eq!(result.plots[0].values[0].as_f64(), Some(2.));
        let strategy = result.strategy.unwrap();
        if blocked {
            assert!(strategy.orders.is_empty());
        } else {
            assert_eq!(strategy.orders[0].qty, 3.25);
        }
    }
    assert!(chart().with_quantity_precision(2).is_ok());
    assert!(chart().with_quantity_precision(1).is_err());
    assert!(
        ChartContext::default()
            .with_quantity_precision(2)
            .unwrap()
            .with_quantity_step("0.25")
            .is_ok()
    );
    assert!(
        ChartContext::default()
            .with_quantity_precision(1)
            .unwrap()
            .with_quantity_step("0.25")
            .is_err()
    );
}

#[test]
fn default_sizing_and_queries_use_exact_grid_in_both_execution_schedules() {
    for closing in [false, true] {
        for (kind, budget) in [("cash", 131.), ("percent_of_equity", 1.31), ("fixed", 1.49)] {
            let source = format!(
                "//@version=6\nstrategy(\"Grid defaults\",initial_capital=10000,margin_long=0,process_orders_on_close={closing},default_qty_type=strategy.{kind},default_qty_value={budget})\nif bar_index==0\n    strategy.entry(\"L\",strategy.long)\nplot(strategy.default_entry_qty(100))"
            );
            let result = run(chart(), &source, &[100.; 3]);
            assert_eq!(result.plots[0].values[0].as_f64(), Some(1.25));
            assert_eq!(result.strategy.unwrap().orders[0].qty, 1.25);
            let blocked = run(chart().with_min_contract("2").unwrap(), &source, &[100.; 3]);
            assert_eq!(blocked.plots[0].values[0].as_f64(), Some(0.));
            assert!(blocked.strategy.unwrap().orders.is_empty());
        }
    }
}

#[test]
fn position_cap_cannot_create_off_grid_or_below_minimum_entries() {
    let source = "//@version=6\nstrategy(\"Grid cap\",margin_long=0)\nstrategy.risk.max_position_size(1.2)\nif bar_index==0\n    strategy.entry(\"L\",strategy.long,qty=2)";
    let result = run(chart(), source, &[100.; 3]);
    assert_eq!(result.strategy.unwrap().orders[0].qty, 1.);
    assert!(
        run(chart().with_min_contract("2").unwrap(), source, &[100.; 3])
            .strategy
            .unwrap()
            .orders
            .is_empty()
    );
}

#[test]
fn close_and_exit_fixed_or_percentage_quantities_reserve_on_the_grid() {
    for (command, expected) in [
        ("strategy.close(\"L\",qty=0.6)", 0.5),
        ("strategy.close(\"L\",qty_percent=33)", 0.25),
        ("strategy.exit(\"X\",\"L\",qty=0.6,limit=110)", 0.5),
        ("strategy.exit(\"X\",\"L\",qty_percent=33,limit=110)", 0.25),
    ] {
        let source = format!(
            "//@version=6\nstrategy(\"Grid exits\",margin_long=0)\nif bar_index==0\n    strategy.entry(\"L\",strategy.long,qty=1.25)\nif bar_index==1\n    {command}\nif bar_index==3\n    strategy.close_all()"
        );
        let result = run(chart(), &source, &[100., 100., 110., 110., 110.]);
        let strategy = result.strategy.unwrap();
        assert_eq!(strategy.trades[0].qty, expected, "{command}");
        assert_eq!(strategy.trades[1].qty, 1.25 - expected, "{command}");
        assert_eq!(strategy.position.last().unwrap().size, 0.);
    }
}

#[test]
fn external_defaults_use_host_equity_and_contract_rules_without_native_fills() {
    let source = "//@version=6\nstrategy(\"External grid\",default_qty_type=strategy.percent_of_equity,default_qty_value=33)\nif bar_index==0\n    strategy.entry(\"L\",strategy.long)\nplot(strategy.default_entry_qty(100))";
    let hir = program(source);
    let chart = chart().with_point_value(50.).unwrap();
    let accounts = (0..3)
        .map(|i| pine_runtime::ExternalAccountFrame {
            time: i * 60_000,
            position_size: 0.,
            position_avg_price: None,
            equity: 20000.,
            initial_capital: 20000.,
            netprofit: 0.,
            openprofit: 0.,
        })
        .collect();
    let mut runtime = HistoricalRuntime::with_request_environment(
        &hir,
        RequestEnvironment::default().for_chart(chart),
    )
    .with_external_accounts(accounts)
    .unwrap();
    runtime.append_bars(&bars(&[100.; 3])).unwrap();
    assert_eq!(runtime.external_intents().unwrap()[0].qty, Some(1.25));
    assert_eq!(runtime.result().plots[0].values[0].as_f64(), Some(1.25));
    assert!(runtime.result().strategy.is_none());
}

#[test]
fn requested_contract_metadata_does_not_replace_chart_sizing_or_valuation() {
    use pine_runtime::{
        InMemoryRequestDataProvider, RequestKey, RequestTimeframe, symbol_contexts_from_json,
    };
    use std::sync::Arc;
    let timeframe = RequestTimeframe::parse("1").unwrap();
    let chart = ChartContext::new("A", timeframe.clone())
        .with_point_value(50.)
        .unwrap()
        .with_quantity_step("0.25")
        .unwrap();
    let provider = InMemoryRequestDataProvider::from_streams(vec![(
        RequestKey::new("B", timeframe),
        bars(&[100.; 3]),
    )])
    .unwrap();
    let environment=RequestEnvironment::new(chart,Arc::new(provider)).with_symbol_contexts(
        symbol_contexts_from_json(r#"{"B":{"minMove":1,"priceScale":100,"pointValue":0.01,"quantityStep":"0.5","minContract":"1.5"}}"#).unwrap()
    ).unwrap();
    let hir = program(
        "//@version=6\nstrategy(\"Foreign contract\",initial_capital=10000,margin_long=0)\nif bar_index==0\n    strategy.entry(\"L\",strategy.long,qty=1.49)\nplot(request.security(\"B\",\"1\",syminfo.pointvalue))\nplot(request.security(\"B\",\"1\",syminfo.mincontract))\nplot(syminfo.pointvalue)",
    );
    let mut runtime = HistoricalRuntime::with_request_environment(&hir, environment);
    runtime.append_bars(&bars(&[100.; 3])).unwrap();
    let result = runtime.result();
    assert_eq!(result.plots[0].values[2].as_f64(), Some(0.01));
    assert_eq!(result.plots[1].values[2].as_f64(), Some(1.5));
    assert_eq!(result.plots[2].values[2].as_f64(), Some(50.));
    let strategy = result.strategy.unwrap();
    assert_eq!(strategy.orders[0].qty, 1.25);
    assert_eq!(strategy.equity[1].market_value, 6250.);
    assert_eq!(strategy.equity[1].cash, 3750.);
}
