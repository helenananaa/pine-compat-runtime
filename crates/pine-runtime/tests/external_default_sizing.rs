use pine_runtime::{
    Bar, ChartContext, ExternalAccountFrame, HistoricalRuntime, NoRequestDataProvider,
    RequestEnvironment,
};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;
use std::sync::Arc;

fn program(source: &str) -> pine_ir::HirProgram {
    let analysis = analyze_source(&SourceFile::new("external-sizing.pine", source));
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    analysis.hir.unwrap()
}

fn bar(index: i64, price: f64) -> Bar {
    Bar {
        time: index * 60000,
        open: price,
        high: price + 1.0,
        low: price - 1.0,
        close: price,
        volume: 100.0,
    }
}

fn account(index: i64, equity: f64) -> ExternalAccountFrame {
    ExternalAccountFrame {
        time: index * 60000,
        position_size: 0.0,
        position_avg_price: None,
        equity,
        initial_capital: 1000.0,
        netprofit: equity - 1000.0,
        openprofit: 0.0,
    }
}

#[test]
fn defaults_and_query_use_host_equity_not_declared_initial_capital() {
    for (kind, expected) in [
        ("cash", vec![10.0, 5.0, 20.0]),
        ("percent_of_equity", vec![100.0, 40.0, 320.0]),
    ] {
        let hir = program(&format!(
            "//@version=6\nstrategy(\"Host defaults\",initial_capital=1000000,default_qty_type=strategy.{kind},default_qty_value=100)\nstrategy.entry(\"L\",strategy.long)\nplot(strategy.default_entry_qty(10))"
        ));
        let quantity = pine_runtime::host_requirements(&hir)
            .account
            .unwrap()
            .default_quantity
            .unwrap();
        assert_eq!(quantity.kind, kind);
        assert_eq!(quantity.value, "100");
        let mut runtime = HistoricalRuntime::new(&hir)
            .with_external_accounts(vec![
                account(0, 1000.0),
                account(1, 800.0),
                account(2, 1600.0),
            ])
            .unwrap();
        runtime
            .append_bars(&[bar(0, 10.0), bar(1, 20.0), bar(2, 5.0)])
            .unwrap();
        assert_eq!(
            runtime
                .external_intents()
                .unwrap()
                .iter()
                .map(|row| row.qty.unwrap())
                .collect::<Vec<_>>(),
            expected
        );
        let query = if kind == "cash" {
            vec![10.0, 10.0, 10.0]
        } else {
            vec![100.0, 80.0, 160.0]
        };
        assert_eq!(
            runtime.result().plots[0]
                .values
                .iter()
                .map(|v| v.as_f64().unwrap())
                .collect::<Vec<_>>(),
            query
        );
        assert!(runtime.result().strategy.is_none());
    }
}

#[test]
fn default_and_explicit_quantities_share_chart_grid_and_dust_is_a_noop() {
    for (kind, value) in [
        ("fixed", 1.239),
        ("cash", 12.39),
        ("percent_of_equity", 1.239),
    ] {
        let hir = program(&format!(
            "//@version=6\nstrategy(\"Grid\",default_qty_type=strategy.{kind},default_qty_value={value})\nstrategy.entry(\"D\",strategy.long)\nstrategy.order(\"E\",strategy.long,qty=1.239)\nstrategy.order(\"Dust\",strategy.long,qty=0.001)\nplot(strategy.default_entry_qty(close))"
        ));
        let chart = ChartContext::default().with_quantity_precision(2).unwrap();
        let mut runtime = HistoricalRuntime::with_request_environment(
            &hir,
            RequestEnvironment::new(chart, Arc::new(NoRequestDataProvider)),
        )
        .with_external_accounts(vec![account(0, 1000.0)])
        .unwrap();
        runtime.append_bar(bar(0, 10.0)).unwrap();
        assert_eq!(
            runtime
                .external_intents()
                .unwrap()
                .iter()
                .map(|row| row.qty.unwrap())
                .collect::<Vec<_>>(),
            vec![1.23, 1.23]
        );
        assert_eq!(runtime.result().plots[0].values[0].as_f64(), Some(1.23));
    }
}

#[test]
fn negative_host_equity_fails_before_percent_quantity_grid_rounding() {
    let hir = program(
        "//@version=6\nstrategy(\"Negative\",initial_capital=1000000,default_qty_type=strategy.percent_of_equity,default_qty_value=1)\nstrategy.entry(\"L\",strategy.long)",
    );
    let mut runtime = HistoricalRuntime::new(&hir)
        .with_external_accounts(vec![account(0, -0.001)])
        .unwrap();
    let error = runtime.append_bar(bar(0, 10.0)).unwrap_err();
    assert!(error.message.contains("equity is less than 0"));
}

#[test]
fn zero_default_cash_and_equity_budgets_are_noops() {
    for kind in ["cash", "percent_of_equity"] {
        let hir = program(&format!(
            "//@version=6\nstrategy(\"Zero\",default_qty_type=strategy.{kind},default_qty_value=0)\nstrategy.entry(\"E\",strategy.long)\nstrategy.order(\"O\",strategy.short,qty=na)\nplot(strategy.default_entry_qty(close))"
        ));
        let mut runtime = HistoricalRuntime::new(&hir)
            .with_external_accounts(vec![account(0, 1000.0)])
            .unwrap();
        runtime.append_bar(bar(0, 10.0)).unwrap();
        assert!(runtime.external_intents().unwrap().is_empty());
        assert_eq!(runtime.result().plots[0].values[0].as_f64(), Some(0.0));
    }
}
