use pine_runtime::{
    Bar, BarUpdate, HistoricalRuntime, RealtimeRuntime, RuntimeResult, public_runtime_result_json,
    run_historical,
};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

fn compile(source: &str) -> pine_ir::HirProgram {
    let analysis = analyze_source(&SourceFile::new("na-prices.pine", source));
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    analysis.hir.unwrap()
}

fn bars() -> Vec<Bar> {
    (0..6)
        .map(|i| Bar {
            time: 1_700_000_000_000 + i * 60_000,
            open: 100.0 + i as f64,
            high: 102.0 + i as f64,
            low: 98.0 + i as f64,
            close: 100.0 + i as f64,
            volume: 100.0,
        })
        .collect()
}

fn runs(source: &str) -> Vec<RuntimeResult> {
    let program = compile(source);
    let bars = bars();
    let mut incremental = HistoricalRuntime::new(&program);
    let mut realtime = RealtimeRuntime::new(&program);
    let mut forming = RealtimeRuntime::new(&program);
    for bar in &bars {
        incremental.append_bar(*bar).unwrap();
        realtime.update(BarUpdate::historical(*bar)).unwrap();
        forming.update(BarUpdate::forming(*bar)).unwrap();
        forming.update(BarUpdate::forming(*bar)).unwrap();
        forming.update(BarUpdate::confirmed(*bar)).unwrap();
    }
    vec![
        run_historical(&program, &bars).unwrap(),
        incremental.result(),
        realtime.result(),
        forming.result(),
    ]
}

fn assert_equivalent(reference: &str, candidate: &str) {
    for (expected, actual) in runs(reference).into_iter().zip(runs(candidate)) {
        let comparable = |result: &RuntimeResult| {
            let mut value: serde_json::Value =
                serde_json::from_str(&public_runtime_result_json(result)).unwrap();
            // Adding a cast allocates an extra call site, so plot IDs differ
            // between these distinct programs; all values remain comparable.
            for plot in value["plots"].as_array_mut().unwrap() {
                plot.as_object_mut().unwrap().remove("id");
            }
            value
        };
        assert_eq!(comparable(&actual), comparable(&expected), "{candidate}");
        assert!(actual.strategy.unwrap().diagnostics.is_empty());
    }
}

#[test]
fn absent_prices_match_market_limit_and_stop_orders_in_all_execution_modes() {
    for version in [5, 6] {
        for command in ["entry", "order"] {
            for direction in ["long", "short"] {
                let script = |prices: &str| {
                    format!(
                        "//@version={version}\nstrategy(\"prices\")\nif bar_index == 0\n    strategy.{command}(\"E\", strategy.{direction}, qty=1{prices})\nif bar_index == 3\n    strategy.close_all()\nplot(strategy.position_size)\n"
                    )
                };
                for prices in [
                    ", limit=na",
                    ", stop=na",
                    ", limit=float(na)",
                    ", stop=float(na)",
                    ", limit=na, stop=na",
                    ", limit=bar_index == 0 ? float(na) : close",
                ] {
                    assert_equivalent(&script(""), &script(prices));
                }
                for (reference, candidate) in [
                    (", limit=100", ", limit=100, stop=na"),
                    (", stop=102", ", limit=na, stop=102"),
                ] {
                    assert_equivalent(&script(reference), &script(candidate));
                }
                let result = run_historical(&compile(&script("")), &bars()).unwrap();
                let strategy = result.strategy.unwrap();
                assert_eq!(strategy.trades.len(), 1);
                assert_eq!(strategy.orders[0].qty, 1.0);
            }
        }
    }
}

#[test]
fn na_defaults_keep_cash_and_percent_sizing_including_same_bar_close() {
    for command in ["entry", "order"] {
        for sizing in ["cash", "percent_of_equity"] {
            for on_close in [false, true] {
                let script = |args: &str| {
                    format!(
                        "//@version=6\nstrategy(\"sizing\", initial_capital=10000, default_qty_type=strategy.{sizing}, default_qty_value=25, slippage=2, process_orders_on_close={on_close})\nif bar_index == 0\n    strategy.{command}(\"E\", strategy.long{args})\nif bar_index == 3\n    strategy.close_all()\nplot(strategy.position_size)\n"
                    )
                };
                assert_equivalent(&script(""), &script(", qty=na, limit=na, stop=na"));
            }
        }
    }
}

#[test]
fn replacing_unfilled_limit_with_na_switches_to_market_and_preserves_oca() {
    for command in ["entry", "order"] {
        let script = |prices: &str| {
            format!(
                "//@version=5\nstrategy(\"replacement\")\nif bar_index == 0\n    strategy.{command}(\"E\", strategy.long, qty=1, limit=1, oca_name=\"g\", oca_type=strategy.oca.cancel)\n    strategy.{command}(\"other\", strategy.long, qty=1, limit=2, oca_name=\"g\", oca_type=strategy.oca.cancel)\nif bar_index == 1\n    strategy.{command}(\"E\", strategy.long, qty=1{prices}, oca_name=\"g\", oca_type=strategy.oca.cancel)\nif bar_index == 3\n    strategy.close_all()\nplot(strategy.position_size)"
            )
        };
        assert_equivalent(&script(""), &script(", limit=na, stop=float(na)"));
        let result = run_historical(&compile(&script(", limit=na")), &bars()).unwrap();
        assert_eq!(result.strategy.unwrap().trades.len(), 1);
    }
}

#[test]
fn price_expressions_are_evaluated_once_and_invalid_prices_remain_diagnostics() {
    for command in ["entry", "order"] {
        let script = format!(
            "//@version=5\nstrategy(\"evaluation\")\nvar calls = array.new_int()\nprice() =>\n    array.push(calls, bar_index)\n    float(na)\nif bar_index == 0\n    strategy.{command}(\"E\", strategy.long, limit=price(), stop=price())\nplot(array.size(calls))"
        );
        let result = run_historical(&compile(&script), &bars()).unwrap();
        assert_eq!(result.strategy.unwrap().orders.len(), 1);
        assert!(
            result.plots[0]
                .values
                .iter()
                .all(|v| v.as_f64() == Some(2.0))
        );
        for price in ["0", "-1"] {
            let result = run_historical(&compile(&format!(
                "//@version=6\nstrategy(\"invalid\")\nstrategy.{command}(\"E\", strategy.long, limit=bar_index * 0 + ({price}))"
            )), &bars()).unwrap();
            let strategy = result.strategy.unwrap();
            assert!(strategy.orders.is_empty());
            assert!(
                strategy
                    .diagnostics
                    .iter()
                    .any(|d| d.code == "E_STRATEGY_PRICE")
            );
        }
    }
}
