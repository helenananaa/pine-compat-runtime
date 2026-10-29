use std::sync::Arc;

use pine_runtime::{
    Bar, BarUpdate, BarUpdateKind, ChartContext, ChartType, HistoricalRuntime,
    InMemoryStandardOhlcProvider, RealtimeRuntime, RequestEnvironment, StandardOhlcObservation,
    StandardOhlcPair, StandardOhlcProvider, host_requirements, public_runtime_result_json,
    run_historical, standard_ohlc_from_json,
};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

fn compile(source: &str) -> pine_ir::HirProgram {
    let analysis = analyze_source(&SourceFile::new("standard.pine", source));
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
        high: price + 2.0,
        low: price - 2.0,
        close: price,
        volume: 10.0,
    }
}

fn pair(
    chart: Bar,
    standard: Bar,
    kind: BarUpdateKind,
    execution_time: Option<i64>,
) -> StandardOhlcPair {
    StandardOhlcPair {
        observation: StandardOhlcObservation {
            chart,
            kind,
            execution_time,
        },
        standard,
    }
}

fn environment(pairs: Vec<StandardOhlcPair>) -> RequestEnvironment {
    RequestEnvironment::default()
        .for_chart(ChartContext::default().with_chart_type(ChartType::HeikinAshi))
        .with_standard_ohlc_provider(Arc::new(
            InMemoryStandardOhlcProvider::from_pairs(pairs).unwrap(),
        ))
}

#[test]
fn synthetic_script_prices_stay_separate_from_orders_equity_and_sizing() {
    let chart: Vec<_> = (0..5).map(|i| bar(i, 1000.0 + i as f64)).collect();
    let standard: Vec<_> = (0..5).map(|i| bar(i, 100.0 + i as f64)).collect();
    let env = environment(
        chart
            .iter()
            .zip(&standard)
            .map(|(a, b)| pair(*a, *b, BarUpdateKind::Historical, None))
            .collect(),
    );
    for version in [5, 6] {
        for direction in ["long", "short"] {
            for command in ["entry", "order"] {
                for prices in ["", ", limit=101", ", stop=101", ", limit=101, stop=101"] {
                    for on_close in [false, true] {
                        for on_fills in [false, true] {
                            let source = format!(
                                "//@version={version}\nstrategy(\"dual\",fill_orders_on_standard_ohlc=true,initial_capital=10000,default_qty_type=strategy.cash,default_qty_value=1000,process_orders_on_close={on_close},calc_on_order_fills={on_fills})\nif bar_index == 0\n    strategy.{command}(\"E\",strategy.{direction}{prices})\nif bar_index == 3\n    strategy.close_all()\nplot(close)\nplot(strategy.equity)\nplot(chart.is_heikinashi ? 1 : 0)"
                            );
                            let program = compile(&source);
                            let mut runtime =
                                HistoricalRuntime::with_request_environment(&program, env.clone());
                            runtime.append_bars(&chart).unwrap();
                            let actual = runtime.result();
                            let reference = run_historical(&program, &standard).unwrap();
                            let actual_json: serde_json::Value =
                                serde_json::from_str(&public_runtime_result_json(&actual)).unwrap();
                            let reference_json: serde_json::Value =
                                serde_json::from_str(&public_runtime_result_json(&reference))
                                    .unwrap();
                            assert_eq!(
                                actual_json["strategy"], reference_json["strategy"],
                                "{source}"
                            );
                            assert_eq!(actual.plots[1].values, reference.plots[1].values);
                            assert_eq!(actual.plots[0].values[0].as_f64(), Some(1000.0));
                            assert_eq!(actual.plots[2].values[0].as_f64(), Some(1.0));
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn realtime_uses_each_observed_standard_price_including_broker_only_updates() {
    for every_tick in [false, true] {
        let program = compile(&format!(
            "//@version=6\nstrategy(\"live\",fill_orders_on_standard_ohlc=true,calc_on_every_tick={every_tick})\nif bar_index == 0\n    strategy.entry(\"E\",strategy.long,qty=1)\nif bar_index == 2\n    strategy.close_all(immediately=true)\nplot(close)"
        ));
        let pairs = vec![
            pair(
                bar(0, 1000.0),
                bar(0, 100.0),
                BarUpdateKind::Historical,
                None,
            ),
            pair(
                bar(1, 1001.0),
                bar(1, 101.0),
                BarUpdateKind::Forming,
                Some(61000),
            ),
            pair(
                bar(1, 1002.0),
                bar(1, 102.0),
                BarUpdateKind::Forming,
                Some(62000),
            ),
            pair(
                bar(1, 1003.0),
                bar(1, 103.0),
                BarUpdateKind::Confirmed,
                Some(119999),
            ),
            pair(
                bar(2, 1004.0),
                bar(2, 104.0),
                BarUpdateKind::Confirmed,
                Some(179999),
            ),
        ];
        let mut runtime = RealtimeRuntime::with_request_environment(&program, environment(pairs));
        runtime.seed_historical(&[bar(0, 1000.0)]).unwrap();
        let first = runtime
            .update_with_execution_time(BarUpdate::forming(bar(1, 1001.0)), 61000)
            .unwrap();
        assert_eq!(first.strategy.unwrap().orders[0].price, 101.0);
        let second = runtime
            .update_with_execution_time(BarUpdate::forming(bar(1, 1002.0)), 62000)
            .unwrap();
        assert_eq!(second.strategy.unwrap().orders.len(), 1);
        runtime
            .update_with_execution_time(BarUpdate::confirmed(bar(1, 1003.0)), 119999)
            .unwrap();
        let result = runtime
            .update_with_execution_time(BarUpdate::confirmed(bar(2, 1004.0)), 179999)
            .unwrap();
        let strategy = result.strategy.unwrap();
        assert_eq!(strategy.trades.len(), 1);
        assert_eq!(strategy.trades[0].entry_price, 101.0);
        assert_eq!(strategy.trades[0].exit_price, 104.0);
        assert_eq!(
            result.plots[0].values.last().unwrap().as_f64(),
            Some(1004.0)
        );
    }
}

#[test]
fn missing_or_wrong_observation_is_retryable_and_never_falls_back_to_chart_prices() {
    let program = compile(
        "//@version=5\nstrategy(\"missing\",fill_orders_on_standard_ohlc=true)\nstrategy.entry(\"E\",strategy.long)",
    );
    let mut runtime = HistoricalRuntime::with_request_environment(
        &program,
        environment(vec![pair(
            bar(0, 1000.0),
            bar(0, 100.0),
            BarUpdateKind::Historical,
            None,
        )]),
    );
    let error = runtime.append_bar(bar(0, 1001.0)).unwrap_err();
    assert!(error.message.contains("E_STANDARD_OHLC_INPUT"));
    assert!(!runtime.is_execution_poisoned());
    assert!(runtime.result().strategy.unwrap().orders.is_empty());
    runtime.append_bar(bar(0, 1000.0)).unwrap();
    let mut live = RealtimeRuntime::with_request_environment(
        &program,
        environment(vec![pair(
            bar(0, 1000.0),
            bar(0, 100.0),
            BarUpdateKind::Historical,
            None,
        )]),
    );
    // A completed historical input must never satisfy a forming observation.
    assert!(
        live.update_with_execution_time(BarUpdate::forming(bar(0, 1000.0)), 1)
            .is_err()
    );
}

#[test]
fn false_option_keeps_synthetic_fills_and_standard_charts_need_no_extra_prices() {
    let source = "//@version=5\nstrategy(\"disabled\",fill_orders_on_standard_ohlc=false)\nif bar_index==0\n    strategy.entry(\"E\",strategy.long,qty=1)";
    let program = compile(source);
    let mut runtime = HistoricalRuntime::with_request_environment(&program, environment(vec![]));
    runtime
        .append_bars(&[bar(0, 1000.0), bar(1, 1001.0)])
        .unwrap();
    assert_eq!(runtime.result().strategy.unwrap().orders[0].price, 1001.0);
    let program = compile(&source.replace("=false", "=true"));
    let result = run_historical(&program, &[bar(0, 100.0), bar(1, 101.0)]).unwrap();
    assert_eq!(result.strategy.unwrap().orders[0].price, 101.0);
    assert_eq!(
        host_requirements(&program).execution.standard_ohlc,
        "requiredPerObservationOnHeikinAshiNativeBrokerSchema1"
    );
}

#[test]
fn input_schema_rejects_invalid_pairs_and_ambiguous_forming_observations() {
    let valid = pair(
        bar(0, 1000.0),
        bar(0, 100.0),
        BarUpdateKind::Historical,
        None,
    );
    let input = serde_json::json!({"schemaVersion":1,"observations":[valid]});
    let provider = standard_ohlc_from_json(&input.to_string()).unwrap();
    assert_eq!(
        provider.standard_bar(valid.observation),
        Some(valid.standard)
    );
    assert!(InMemoryStandardOhlcProvider::from_pairs([valid, valid]).is_err());
    assert!(
        InMemoryStandardOhlcProvider::from_pairs([pair(
            valid.observation.chart,
            bar(1, 100.0),
            BarUpdateKind::Historical,
            None
        )])
        .is_err()
    );
    assert!(
        InMemoryStandardOhlcProvider::from_pairs([pair(
            valid.observation.chart,
            valid.standard,
            BarUpdateKind::Forming,
            None
        )])
        .is_err()
    );
    let mut invalid = valid;
    invalid.standard.high = 0.0;
    assert!(InMemoryStandardOhlcProvider::from_pairs([invalid]).is_err());
}

#[test]
fn declaration_is_versioned_constant_and_history_tick_combination_fails_explicitly() {
    for declaration in [
        "//@version=4\nstrategy(\"old\",fill_orders_on_standard_ohlc=true)",
        "//@version=6\nstrategy(\"dynamic\",fill_orders_on_standard_ohlc=close>open)",
    ] {
        assert!(
            analyze_source(&SourceFile::new("invalid.pine", declaration))
                .hir
                .is_none()
        );
    }
    let program = compile(
        "//@version=6\nstrategy(\"ticks\",fill_orders_on_standard_ohlc=true,calc_on_every_history_tick=true)",
    );
    let mut runtime = HistoricalRuntime::with_request_environment(&program, environment(vec![]));
    assert!(
        runtime
            .append_bar(bar(0, 1000.0))
            .unwrap_err()
            .message
            .contains("calc_on_every_history_tick")
    );
}
