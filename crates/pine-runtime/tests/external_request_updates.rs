use std::sync::Arc;

use pine_runtime::{
    Bar, BarUpdateKind, ChartContext, ExternalAccountFrame, ExternalPass, ExternalRequestStream,
    HistoricalRuntime, InMemoryRequestDataProvider, ObservedRequestUpdate, RequestEnvironment,
    RequestKey, RequestTimeframe,
};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

fn bar(index: i64, close: f64) -> Bar {
    Bar {
        time: index * 60_000,
        open: close,
        high: close,
        low: close,
        close,
        volume: 1.0,
    }
}

fn update(index: i64, close: f64) -> ObservedRequestUpdate {
    ObservedRequestUpdate {
        symbol: "B".into(),
        timeframe: "1".into(),
        bar: bar(index, close),
        kind: BarUpdateKind::Confirmed,
        available_at_ms: (index + 1) * 60_000 - 1,
    }
}

fn frame(index: i64) -> ExternalAccountFrame {
    ExternalAccountFrame {
        time: index * 60_000,
        position_size: 0.0,
        position_avg_price: None,
        equity: 1000.0,
        initial_capital: 1000.0,
        netprofit: 0.0,
        openprofit: 0.0,
    }
}

fn environment() -> RequestEnvironment {
    let timeframe = RequestTimeframe::parse("1").unwrap();
    let provider = InMemoryRequestDataProvider::from_streams([
        (RequestKey::new("B", timeframe.clone()), vec![]),
        (
            RequestKey::new("B", RequestTimeframe::parse("2").unwrap()),
            vec![],
        ),
        (
            RequestKey::new("C", timeframe.clone()),
            vec![bar(0, 1.0), bar(1, 1.0)],
        ),
    ])
    .unwrap();
    RequestEnvironment::new(ChartContext::new("A", timeframe), Arc::new(provider))
}

#[test]
fn scheduled_deltas_match_a_causal_full_snapshot_oracle_across_merge_modes() {
    for requested in ["1", "2"] {
        for gaps in ["barmerge.gaps_on", "barmerge.gaps_off"] {
            for lookahead in ["barmerge.lookahead_on", "barmerge.lookahead_off"] {
                for expression in [
                    "ta.cum(close)",
                    "ta.ema(close,2)",
                    "close[1]",
                    "request.security(\"C\",\"1\",close)",
                ] {
                    let source = format!(
                        "//@version=6\nstrategy(\"oracle\",calc_on_order_fills=true)\nvalue=request.security(\"B\",\"{requested}\",{expression},gaps={gaps},lookahead={lookahead})\nvar values=array.new_float()\narray.push(values,nz(value))\nplot(value)\nplot(array.sum(values))\nplot(strategy.equity)\nif not na(value)\n    strategy.order(\"L\",strategy.long,qty=1)\n"
                    );
                    let analysis = analyze_source(&SourceFile::new("oracle", source));
                    assert!(
                        analysis.diagnostics.is_empty(),
                        "{:?}",
                        analysis.diagnostics
                    );
                    let hir = analysis.hir.unwrap();
                    let mut delta =
                        HistoricalRuntime::with_request_environment(&hir, environment())
                            .with_external_accounts_and_passes(vec![], Some(vec![]))
                            .unwrap();
                    let mut full = HistoricalRuntime::with_request_environment(&hir, environment())
                        .with_external_accounts_and_passes(vec![], Some(vec![]))
                        .unwrap();
                    let mut requested_bars = vec![];
                    for index in 0..6 {
                        let period = requested.parse::<i64>().unwrap();
                        let added = if (index + 1) % period == 0 {
                            let mut item = update(index + 1 - period, (index + 1) as f64 * 10.0);
                            item.timeframe = requested.into();
                            item.available_at_ms = (index + 1) * 60_000 - 1;
                            requested_bars.push(item.bar);
                            vec![item]
                        } else {
                            vec![]
                        };
                        let market = bar(index, 1.0);
                        let account = frame(index);
                        let pass = ExternalPass {
                            account: account.clone(),
                            bar: market,
                            event_time_ms: (index + 1) * 60_000 - 1,
                            confirmed: true,
                            request_data: None,
                            request_updates: Some(added),
                        };
                        let mut oracle = pass.clone();
                        oracle.request_updates = None;
                        oracle.request_data = Some(vec![
                            ExternalRequestStream {
                                symbol: "B".into(),
                                timeframe: requested.into(),
                                bars: requested_bars.clone(),
                            },
                            ExternalRequestStream {
                                symbol: "C".into(),
                                timeframe: "1".into(),
                                bars: vec![bar(0, 1.0), bar(1, 1.0)],
                            },
                        ]);
                        delta
                            .append_external_bars(
                                &[market],
                                vec![account.clone()],
                                Some(vec![vec![pass]]),
                            )
                            .unwrap();
                        full.append_external_bars(
                            &[market],
                            vec![account],
                            Some(vec![vec![oracle]]),
                        )
                        .unwrap();
                        assert_eq!(
                            delta.result(),
                            full.result(),
                            "{requested}/{gaps}/{lookahead}/{expression}/{index}"
                        );
                        assert_eq!(
                            serde_json::to_value(delta.external_intents().unwrap()).unwrap(),
                            serde_json::to_value(full.external_intents().unwrap()).unwrap()
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn observed_growth_keeps_committed_outputs_and_accumulates_requested_state() {
    let analysis = analyze_source(&SourceFile::new(
        "growth",
        "//@version=6\nstrategy(\"growth\")\nplot(request.security(\"B\",\"1\",ta.cum(close)))\n",
    ));
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let hir = analysis.hir.unwrap();
    let mut runtime = HistoricalRuntime::with_request_environment(&hir, environment())
        .with_external_accounts(vec![])
        .unwrap();
    for index in 0..2 {
        runtime
            .apply_external_request_updates(
                &[update(index, 100.0 * (index + 1) as f64)],
                (index + 1) * 60_000 - 1,
            )
            .unwrap();
        runtime
            .append_external_bars(&[bar(index, 1.0)], vec![frame(index)], None)
            .unwrap();
    }
    assert_eq!(
        runtime.result().plots[0]
            .values
            .iter()
            .map(|v| v.as_f64())
            .collect::<Vec<_>>(),
        vec![Some(100.0), Some(300.0)]
    );
}

#[test]
fn observed_growth_invalidates_transitive_fallback_request_caches() {
    let analysis = analyze_source(&SourceFile::new(
        "nested",
        "//@version=6\nstrategy(\"nested\")\nplot(request.security(\"C\",\"1\",ta.cum(request.security(\"B\",\"1\",close))))\n",
    ));
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let hir = analysis.hir.unwrap();
    let mut runtime = HistoricalRuntime::with_request_environment(&hir, environment())
        .with_external_accounts(vec![])
        .unwrap();
    for index in 0..2 {
        runtime
            .apply_external_request_updates(
                &[update(index, 100.0 * (index + 1) as f64)],
                (index + 1) * 60_000 - 1,
            )
            .unwrap();
        runtime
            .append_external_bars(&[bar(index, 1.0)], vec![frame(index)], None)
            .unwrap();
    }
    assert_eq!(
        runtime.result().plots[0]
            .values
            .iter()
            .map(|v| v.as_f64())
            .collect::<Vec<_>>(),
        vec![Some(100.0), Some(300.0)]
    );
}

#[test]
fn future_or_invalid_second_update_is_atomic_and_retryable() {
    let analysis = analyze_source(&SourceFile::new(
        "atomic",
        "//@version=6\nstrategy(\"atomic\")\nplot(request.security(\"B\",\"1\",close))\n",
    ));
    let hir = analysis.hir.unwrap();
    let mut runtime = HistoricalRuntime::with_request_environment(&hir, environment())
        .with_external_accounts(vec![])
        .unwrap();
    let before = runtime.result();
    assert!(
        runtime
            .apply_external_request_updates(&[update(0, 100.0), update(1, 200.0)], 59_999)
            .is_err()
    );
    assert_eq!(runtime.result(), before);
    let mut invalid = update(1, 200.0);
    invalid.bar.volume = -1.0;
    assert!(
        runtime
            .apply_external_request_updates(&[update(0, 100.0), invalid], 119_999)
            .is_err()
    );
    runtime
        .apply_external_request_updates(&[update(0, 100.0)], 59_999)
        .unwrap();
    runtime
        .append_external_bars(&[bar(0, 1.0)], vec![frame(0)], None)
        .unwrap();
    let before = runtime.result();
    assert!(
        runtime
            .apply_external_request_updates(&[update(0, 999.0)], 119_999)
            .is_err()
    );
    assert_eq!(runtime.result(), before);
    runtime
        .apply_external_request_updates(&[update(1, 200.0)], 119_999)
        .unwrap();
    runtime
        .append_external_bars(&[bar(1, 1.0)], vec![frame(1)], None)
        .unwrap();
    assert_eq!(
        runtime.result().plots[0]
            .values
            .iter()
            .map(|v| v.as_f64())
            .collect::<Vec<_>>(),
        vec![Some(100.0), Some(200.0)]
    );
}
