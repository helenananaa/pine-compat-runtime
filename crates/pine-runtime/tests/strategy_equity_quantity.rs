use pine_runtime::{Bar, PineValue, RuntimeError, RuntimeResult, run_historical};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

fn run(source: &str) -> Result<RuntimeResult, RuntimeError> {
    let analysis = analyze_source(&SourceFile::new("equity-quantity.pine", source));
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let bars: Vec<_> = [100., 100., 300., 300.]
        .into_iter()
        .enumerate()
        .map(|(i, price)| Bar {
            time: 1_700_000_000_000 + i as i64 * 60_000,
            open: price,
            high: price,
            low: price,
            close: price,
            volume: 1.,
        })
        .collect();
    run_historical(&analysis.hir.unwrap(), &bars)
}

fn negative_equity_source(version: u16, percent: u16, body: &str) -> String {
    format!(
        "//@version={version}\nstrategy(\"equity\",initial_capital=100,default_qty_type=strategy.percent_of_equity,default_qty_value={percent},margin_long=0,margin_short=0)\nif bar_index==0\n    strategy.entry(\"S\",strategy.short,qty=1)\n{body}\n"
    )
}

#[test]
fn zero_equity_queries_and_default_orders_are_zero_without_trades() {
    for version in [5, 6] {
        let result = run(&format!(
            "//@version={version}\nstrategy(\"zero\",initial_capital=0,default_qty_type=strategy.percent_of_equity,default_qty_value=100)\nstrategy.entry(\"E\",strategy.long)\nstrategy.order(\"O\",strategy.long)\nplot(strategy.default_entry_qty(100))\nplot(strategy.position_size)\n"
        ))
        .unwrap();
        for plot in result.plots {
            assert_eq!(plot.values, vec![PineValue::Float(0.); 4]);
        }
        let strategy = result.strategy.unwrap();
        assert!(strategy.orders.is_empty());
        assert!(strategy.trades.is_empty());
    }
}

#[test]
fn negative_equity_default_entry_order_and_query_report_the_actual_cause() {
    for version in [5, 6] {
        for body in [
            "if bar_index==2\n    strategy.entry(\"L\",strategy.long)",
            "if bar_index==2\n    strategy.order(\"L\",strategy.long)",
            "plot(strategy.default_entry_qty(100))",
        ] {
            let error = run(&negative_equity_source(version, 100, body)).unwrap_err();
            assert!(error.message.contains("equity is less than 0"));
            assert!(error.message.contains("bar 2"));
            assert!(!error.message.contains("NaN"));
        }
    }
}

#[test]
fn zero_percent_remains_zero_after_equity_becomes_negative() {
    let result = run(&negative_equity_source(
        5,
        0,
        "strategy.entry(\"L\",strategy.long)\nstrategy.order(\"O\",strategy.long)\nplot(strategy.default_entry_qty(100))\nplot(strategy.equity)",
    ))
    .unwrap();
    assert_eq!(result.plots[0].values, vec![PineValue::Float(0.); 4]);
    assert_eq!(result.plots[1].values[2], PineValue::Float(-100.));
    assert_eq!(result.strategy.unwrap().orders.len(), 1);
}

#[test]
fn missing_zero_and_negative_fill_prices_keep_native_query_precedence() {
    let result = run(&negative_equity_source(
        5,
        100,
        "plot(strategy.default_entry_qty(na))\nplot(strategy.default_entry_qty(0))\nplot(strategy.default_entry_qty(-100))",
    ))
    .unwrap();
    assert_eq!(result.plots[0].values, vec![PineValue::Float(0.); 4]);
    assert_eq!(result.plots[1].values, vec![PineValue::Float(0.); 4]);
    assert_eq!(
        result.plots[2].values,
        [-1., -1., 1., 1.].map(PineValue::Float)
    );
}

#[test]
fn explicit_positive_quantities_can_reverse_a_negative_equity_position() {
    for command in ["entry", "order"] {
        let result = run(&negative_equity_source(
            5,
            100,
            &format!(
                "if bar_index==2\n    strategy.{command}(\"L\",strategy.long,qty=2)\nplot(strategy.position_size)"
            ),
        ))
        .unwrap();
        assert_eq!(
            result.plots[0].values,
            [0., -1., -1., if command == "entry" { 2. } else { 1. }].map(PineValue::Float)
        );
    }
}

#[test]
fn fixed_and_cash_helpers_keep_their_price_and_equity_independence() {
    for (kind, expected) in [
        ("fixed", [25., 25., 25., 25.]),
        ("cash", [0., 0., -0.25, 0.25]),
    ] {
        let result = run(&format!(
            "//@version=5\nstrategy(\"prices\",default_qty_type=strategy.{kind},default_qty_value=25)\nplot(strategy.default_entry_qty(na))\nplot(strategy.default_entry_qty(0))\nplot(strategy.default_entry_qty(-100))\nplot(strategy.default_entry_qty(100))"
        ))
        .unwrap();
        for (plot, value) in result.plots.iter().zip(expected) {
            assert_eq!(plot.values, vec![PineValue::Float(value); 4]);
        }
    }
}
