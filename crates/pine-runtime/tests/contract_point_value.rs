use pine_runtime::{Bar, ChartContext, HistoricalRuntime, RequestEnvironment, RuntimeResult};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

fn run(source: &str, point_value: f64, prices: &[f64]) -> RuntimeResult {
    let chart = ChartContext::default()
        .with_point_value(point_value)
        .unwrap()
        .with_quantity_precision(1)
        .unwrap();
    execute(
        source,
        RequestEnvironment::default().for_chart(chart),
        prices,
    )
}

fn execute(source: &str, environment: RequestEnvironment, prices: &[f64]) -> RuntimeResult {
    let analysis = analyze_source(&SourceFile::new("contract-value.pine", source));
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let hir = analysis.hir.unwrap();
    let mut runtime = HistoricalRuntime::with_request_environment(&hir, environment);
    let bars: Vec<_> = prices
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
        .collect();
    runtime.append_bars(&bars).unwrap();
    runtime.result()
}

fn near(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-8, "{actual} != {expected}");
}

#[test]
fn multiplier_values_notional_and_profit_but_cash_fees_keep_their_units() {
    for (point, commission, fee, qty, profit) in [
        (50., "cash_per_contract", 2., 2., 992.),
        (50., "percent", 0.1, 2., 979.),
        (0.01, "cash_per_contract", 0.02, 100., 6.),
        (0.01, "percent", 0.1, 100., 9.79),
        (50., "cash_per_order", 2., 2., 996.),
    ] {
        let source = format!(
            "//@version=6\nstrategy(\"Contract\",initial_capital=10000,margin_long=0,commission_type=strategy.commission.{commission},commission_value={fee})\nif bar_index==0\n    strategy.entry(\"L\",strategy.long,qty={qty})\nif bar_index==2\n    strategy.close_all()\nplot(syminfo.pointvalue)"
        );
        let result = run(&source, point, &[100., 100., 110., 110.]);
        assert_eq!(result.plots[0].values[0].as_f64(), Some(point));
        let strategy = result.strategy.unwrap();
        near(strategy.orders[0].qty, qty);
        near(strategy.trades[0].profit, profit);
        near(strategy.equity.last().unwrap().cash, 10000. + profit);
        near(strategy.equity[1].market_value, qty * 100. * point);
    }
}

#[test]
fn defaults_and_quantity_query_divide_cash_by_point_value_in_both_fill_schedules() {
    for closing in [false, true] {
        for (kind, budget, expected) in [
            ("cash", 10000., 2.),
            ("percent_of_equity", 50., 1.),
            ("fixed", 2., 2.),
        ] {
            let source = format!(
                "//@version=6\nstrategy(\"Contract defaults\",initial_capital=10000,margin_long=0,process_orders_on_close={closing},default_qty_type=strategy.{kind},default_qty_value={budget})\nif bar_index==0\n    strategy.entry(\"L\",strategy.long)\nplot(strategy.default_entry_qty(100))"
            );
            let result = run(&source, 50., &[100.; 3]);
            near(result.plots[0].values[0].as_f64().unwrap(), expected);
            near(result.strategy.unwrap().orders[0].qty, expected);
        }
    }
}

#[test]
fn monetary_risk_uses_contract_values_while_position_caps_use_contract_counts() {
    for limit in ["500,strategy.cash", "5,strategy.percent_of_equity"] {
        let source = format!(
            "//@version=6\nstrategy(\"Contract risk\",initial_capital=10000,margin_long=0)\nstrategy.risk.max_position_size(2)\nstrategy.risk.max_drawdown({limit})\nif bar_index==0\n    strategy.entry(\"L\",strategy.long,qty=10)\nplot(strategy.position_size)"
        );
        let result = run(&source, 50., &[100., 100., 94., 94.]);
        assert_eq!(result.plots[0].values[1].as_f64(), Some(2.));
        assert_eq!(result.plots[0].values[3].as_f64(), Some(0.));
        near(result.strategy.unwrap().equity[3].equity, 9400.);
    }
}

#[test]
fn margin_and_forced_close_use_contract_notional() {
    let source = "//@version=6\nstrategy(\"Contract margin\",initial_capital=5000,margin_long=50)\nif bar_index==0\n    strategy.entry(\"L\",strategy.long,qty=2)\nplot(strategy.position_size)\nplot(strategy.margin_liquidation_price)";
    let result = run(source, 50., &[100., 100., 80., 80.]);
    near(result.plots[1].values[1].as_f64().unwrap(), 100.);
    assert_eq!(result.plots[0].values[3].as_f64(), Some(0.));
    let strategy = result.strategy.unwrap();
    near(
        strategy.trades.iter().map(|trade| trade.profit).sum(),
        -2000.,
    );
    near(strategy.equity[3].equity, 3000.);
}

#[test]
fn multiplier_covers_shorts_partial_allocations_and_reversal_transaction_fees() {
    let short = "//@version=6\nstrategy(\"Contract short\",initial_capital=10000,margin_short=0,commission_type=strategy.commission.cash_per_contract,commission_value=2)\nif bar_index==0\n    strategy.entry(\"S\",strategy.short,qty=2)\nif bar_index==2\n    strategy.close_all()";
    let result = run(short, 50., &[100., 100., 90., 90.]);
    let strategy = result.strategy.unwrap();
    near(strategy.trades[0].profit, 992.);
    near(strategy.equity[1].cash, 19996.);
    near(strategy.equity[1].market_value, -10000.);

    let partial = "//@version=6\nstrategy(\"Contract partial\",initial_capital=10000,margin_long=0,commission_type=strategy.commission.cash_per_contract,commission_value=2)\nif bar_index==0\n    strategy.entry(\"L\",strategy.long,qty=3)\nif bar_index==2\n    strategy.close(\"L\",qty=1)\nif bar_index==4\n    strategy.close_all()";
    let result = run(partial, 50., &[100., 100., 105., 105., 110., 110.]);
    let strategy = result.strategy.unwrap();
    near(strategy.trades[0].profit, 246.);
    near(strategy.trades[1].profit, 992.);
    near(strategy.equity[5].equity, 11238.);

    let reversal = "//@version=6\nstrategy(\"Contract reversal\",initial_capital=10000,margin_long=0,margin_short=0,commission_type=strategy.commission.cash_per_order,commission_value=2)\nif bar_index==0\n    strategy.entry(\"L\",strategy.long,qty=2)\nif bar_index==2\n    strategy.entry(\"S\",strategy.short,qty=3)\nif bar_index==4\n    strategy.close_all()";
    let result = run(reversal, 50., &[100., 100., 110., 110., 100., 100.]);
    let strategy = result.strategy.unwrap();
    near(strategy.trades[0].profit, 997.2);
    near(strategy.trades[1].profit, 1496.8);
    near(strategy.equity[5].equity, 12494.);
    assert_eq!(
        strategy
            .alerts
            .iter()
            .map(|alert| alert.qty)
            .collect::<Vec<_>>(),
        [2., 5., 3.]
    );
}

#[test]
fn fx_rebasing_cannot_create_profit_when_contract_price_is_unchanged() {
    let chart = ChartContext::default().with_point_value(50.).unwrap();
    let rates = pine_runtime::currency_rates_from_json(
        r#"{"schemaVersion":1,"pairs":[{"from":"USD","to":"EUR","rates":[{"effectiveTime":0,"rate":1.25},{"effectiveTime":120000,"rate":1.5}]}]}"#,
    ).unwrap();
    let environment = RequestEnvironment::default()
        .for_chart(chart)
        .with_currency_rates(std::sync::Arc::new(rates));
    let source = "//@version=6\nstrategy(\"Contract FX\",currency=currency.EUR,initial_capital=10000,margin_long=0,commission_type=strategy.commission.cash_per_contract,commission_value=2)\nif bar_index==0\n    strategy.entry(\"L\",strategy.long,qty=2)\nif bar_index==3\n    strategy.close_all()\nplot(strategy.convert_to_account(100))\nplot(strategy.convert_to_symbol(150))";
    let result = execute(source, environment, &[100., 100., 100., 110., 110.]);
    let strategy = result.strategy.unwrap();
    near(strategy.equity[1].equity, 9996.);
    near(strategy.equity[2].equity, 9996.);
    near(strategy.trades[0].profit, 1492.);
    near(strategy.equity[4].equity, 11492.);
    near(result.plots[0].values[1].as_f64().unwrap(), 125.);
    near(result.plots[0].values[2].as_f64().unwrap(), 150.);
    near(result.plots[1].values[2].as_f64().unwrap(), 100.);
}

#[test]
fn trade_extrema_and_percentage_denominators_include_contract_value() {
    let source = "//@version=6\nstrategy(\"Contract trade metrics\",initial_capital=10000,margin_long=0,commission_type=strategy.commission.cash_per_contract,commission_value=2)\nif bar_index==0\n    strategy.entry(\"L\",strategy.long,qty=2)\nif bar_index==2\n    strategy.close_all()\nplot(strategy.opentrades.profit(0))\nplot(strategy.opentrades.profit_percent(0))\nplot(strategy.opentrades.max_runup(0))\nplot(strategy.closedtrades.profit_percent(0))\nplot(strategy.closedtrades.max_runup(0))\nplot(strategy.closedtrades.commission(0))";
    let result = run(source, 50., &[100., 100., 110., 110.]);
    near(result.plots[0].values[2].as_f64().unwrap(), 992.);
    near(
        result.plots[1].values[2].as_f64().unwrap(),
        992. / 10004. * 100.,
    );
    near(result.plots[2].values[2].as_f64().unwrap(), 996.);
    near(
        result.plots[3].values[3].as_f64().unwrap(),
        992. / 10004. * 100.,
    );
    near(result.plots[4].values[3].as_f64().unwrap(), 996.);
    near(result.plots[5].values[3].as_f64().unwrap(), 8.);
}
