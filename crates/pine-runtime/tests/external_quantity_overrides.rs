use pine_runtime::{Bar, ExternalAccountFrame, HistoricalRuntime};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

fn compile(source: &str) -> pine_ir::HirProgram {
    let analysis = analyze_source(&SourceFile::new("external-quantity.pine", source));
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    analysis.hir.expect("HIR")
}

fn bar(time: i64) -> Bar {
    Bar {
        time,
        open: 10.0,
        high: 11.0,
        low: 9.0,
        close: 10.0,
        volume: 100.0,
    }
}

fn account(time: i64) -> ExternalAccountFrame {
    ExternalAccountFrame {
        time,
        position_size: 0.0,
        position_avg_price: None,
        equity: 1000.0,
        initial_capital: 1000.0,
        netprofit: 0.0,
        openprofit: 0.0,
    }
}

#[test]
fn explicit_quantity_overrides_cash_and_equity_defaults_in_batch_and_append() {
    for version in [5, 6] {
        for kind in ["cash", "percent_of_equity"] {
            for command in ["entry", "order"] {
                let hir = compile(&format!(
                    "//@version={version}\nstrategy(\"Override\",default_qty_type=strategy.{kind},default_qty_value=25)\nstrategy.{command}(\"L\",strategy.long,qty=2)\nplot(strategy.equity)"
                ));
                let mut batch = HistoricalRuntime::new(&hir)
                    .with_external_accounts(vec![account(0), account(60000)])
                    .expect("explicit quantities must override defaults");
                batch.append_bars(&[bar(0), bar(60000)]).unwrap();
                let mut append = HistoricalRuntime::new(&hir)
                    .with_external_accounts(vec![])
                    .unwrap();
                for time in [0, 60000] {
                    append
                        .append_external_bars(&[bar(time)], vec![account(time)], None)
                        .unwrap();
                }
                assert_eq!(
                    serde_json::to_value(batch.external_intents().unwrap()).unwrap(),
                    serde_json::to_value(append.external_intents().unwrap()).unwrap()
                );
                assert_eq!(batch.external_intents().unwrap().len(), 2);
                assert!(
                    batch
                        .external_intents()
                        .unwrap()
                        .iter()
                        .all(|intent| intent.qty == Some(2.0))
                );
                assert!(batch.result().strategy.is_none());
                assert!(append.result().strategy.is_none());
            }
        }
    }
}

#[test]
fn zero_explicit_and_default_quantities_emit_no_order_or_cancellation() {
    for version in [5, 6] {
        for kind in ["fixed", "cash", "percent_of_equity"] {
            let hir = compile(&format!(
                "//@version={version}\nstrategy(\"Zero\",default_qty_type=strategy.{kind},default_qty_value=0)\nstrategy.entry(\"L\",strategy.long,qty=2)\nstrategy.entry(\"L\",strategy.short,qty=0)\nstrategy.order(\"Z\",strategy.long,qty=-0.0)"
            ));
            let mut runtime = HistoricalRuntime::new(&hir)
                .with_external_accounts(vec![account(0)])
                .unwrap();
            runtime.append_bar(bar(0)).unwrap();
            let intents = runtime.external_intents().unwrap();
            assert_eq!(intents.len(), 1);
            assert_eq!(intents[0].id, "L");
            assert_eq!(intents[0].direction.as_deref(), Some("long"));
            assert_eq!(intents[0].qty, Some(2.0));
        }
        let hir = compile(&format!(
            "//@version={version}\nstrategy(\"Zero fixed\",default_qty_value=0)\nstrategy.entry(\"E\",strategy.long)\nstrategy.order(\"O\",strategy.long,qty=na)"
        ));
        let mut runtime = HistoricalRuntime::new(&hir)
            .with_external_accounts(vec![account(0)])
            .unwrap();
        runtime.append_bar(bar(0)).unwrap();
        assert!(runtime.external_intents().unwrap().is_empty());
    }
}

#[test]
fn omitted_and_na_quantities_use_cash_and_authoritative_equity_defaults() {
    for kind in ["cash", "percent_of_equity"] {
        for command in ["entry", "order"] {
            for quantity in ["", ",qty=na"] {
                let hir = compile(&format!(
                    "//@version=6\nstrategy(\"Default needed\",default_qty_type=strategy.{kind},default_qty_value=25)\nstrategy.{command}(\"L\",strategy.long{quantity})"
                ));
                let mut runtime = HistoricalRuntime::new(&hir)
                    .with_external_accounts(vec![account(0)])
                    .unwrap();
                runtime.append_bar(bar(0)).unwrap();
                let expected = if kind == "cash" { 2.5 } else { 25.0 };
                assert_eq!(runtime.external_intents().unwrap()[0].qty, Some(expected));
            }
        }
    }
}

#[test]
fn negative_external_quantity_remains_an_error() {
    for command in ["entry", "order"] {
        let hir = compile(&format!(
            "//@version=6\nstrategy(\"Invalid\")\nstrategy.{command}(\"L\",strategy.long,qty=-1)"
        ));
        let mut runtime = HistoricalRuntime::new(&hir)
            .with_external_accounts(vec![account(0)])
            .unwrap();
        let error = runtime.append_bar(bar(0)).unwrap_err();
        assert!(
            error
                .message
                .contains("quantity must be positive and finite")
        );
    }
}
