//! Live reference: a static root dependency follows the developing chart, not
//! the final chart tuple carried by a batch schedule.
use pine_runtime::{
    Bar, BarUpdate, ChartContext, ExternalAccountFrame, ExternalBarState,
    ExternalEvaluationTrigger, ExternalExecutionOptions, ExternalPass, ExternalPassContext,
    ExternalScheduledPass, HistoricalRuntime, InMemoryRequestDataProvider, RealtimeRuntime,
    RequestEnvironment, RequestKey, RequestTimeframe,
};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;
use std::sync::Arc;

fn program(field: &str, history: bool) -> pine_ir::HirProgram {
    let source = format!(
        r#"//@version=6
strategy("Static realtime root",calc_on_every_tick=true,calc_on_every_history_tick={history},dynamic_requests=false)
x=request.security(syminfo.tickerid,timeframe.period,{field})
y=request.security("B","1",x)
varip float direct=0
varip float nested=0
direct:=direct*100+x
nested:=nested*100+y
plot(direct)
plot(nested)
"#
    );
    let analysis = analyze_source(&SourceFile::new("static-realtime-root.pine", source));
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    analysis.hir.unwrap()
}

fn inputs() -> (
    Vec<Bar>,
    Vec<ExternalAccountFrame>,
    Vec<Vec<ExternalScheduledPass>>,
) {
    let mut bars = Vec::new();
    let mut frames = Vec::new();
    let mut groups = Vec::new();
    for index in 0..2 {
        let open = 10.0 + 10.0 * index as f64;
        let bar = Bar {
            time: index * 60000,
            open,
            high: open + 2.0,
            low: open - 1.0,
            close: open + 1.0,
            volume: 100.0 * (index + 1) as f64,
        };
        let frame = ExternalAccountFrame {
            time: bar.time,
            position_size: 0.0,
            position_avg_price: None,
            equity: 1000.0,
            initial_capital: 1000.0,
            netprofit: 0.0,
            openprofit: 0.0,
        };
        let mut visible = Bar {
            high: open,
            low: open,
            close: open,
            volume: 0.0,
            ..bar
        };
        let mut group = Vec::new();
        for (tick, price) in [open, bar.high, bar.low, bar.close].into_iter().enumerate() {
            visible.high = visible.high.max(price);
            visible.low = visible.low.min(price);
            visible.close = price;
            visible.volume = bar.volume * (tick + 1) as f64 / 4.0;
            group.push(ExternalScheduledPass {
                evaluation: ExternalPass {
                    bar: visible,
                    account: frame.clone(),
                    event_time_ms: bar.time + 59999 * tick as i64 / 3,
                    confirmed: tick == 3,
                    request_data: None,
                    request_updates: None,
                },
                context: ExternalPassContext {
                    bar_state: ExternalBarState::Realtime,
                    trigger: if tick == 3 {
                        ExternalEvaluationTrigger::BarClose
                    } else {
                        ExternalEvaluationTrigger::RealtimeTick
                    },
                    is_new: tick == 0,
                },
            });
        }
        bars.push(bar);
        frames.push(frame);
        groups.push(group);
    }
    (bars, frames, groups)
}

fn environment() -> RequestEnvironment {
    let tf = RequestTimeframe::parse("1").unwrap();
    let data = (0..2)
        .map(|i| Bar {
            time: i * 60000,
            open: 50.0,
            high: 50.0,
            low: 50.0,
            close: 50.0,
            volume: 1.0,
        })
        .collect();
    let provider =
        InMemoryRequestDataProvider::from_streams([(RequestKey::new("B", tf.clone()), data)])
            .unwrap();
    RequestEnvironment::new(ChartContext::new("A", tf), Arc::new(provider))
}

fn value(bar: Bar, field: &str) -> f64 {
    match field {
        "open" => bar.open,
        "high" => bar.high,
        "low" => bar.low,
        "close" => bar.close,
        "volume" => bar.volume,
        _ => unreachable!(),
    }
}

#[test]
fn realtime_static_root_uses_observed_ohlcv_and_matches_the_native_realtime_control() {
    let (bars, frames, groups) = inputs();
    for field in ["open", "high", "low", "close", "volume"] {
        for history in [false, true] {
            let hir = program(field, history);
            let mut runtime = HistoricalRuntime::with_request_environment(&hir, environment())
                .with_external_schedule(
                    frames.clone(),
                    groups.clone(),
                    ExternalExecutionOptions::default(),
                )
                .unwrap();
            let start = runtime.initialize_historical_dataset(&bars, None).unwrap();
            assert_eq!(start, 0);
            for bar in &bars {
                runtime.append_bar(*bar).unwrap();
            }
            let mut native = RealtimeRuntime::with_request_environment(&hir, environment());
            native.seed_historical(&[]).unwrap();
            let mut expected = Vec::new();
            let mut total = 0.0;
            for group in &groups {
                for item in group {
                    let update = if item.evaluation.confirmed {
                        BarUpdate::confirmed(item.evaluation.bar)
                    } else {
                        BarUpdate::forming(item.evaluation.bar)
                    };
                    native.update(update).unwrap();
                    total = total * 100.0 + value(item.evaluation.bar, field);
                }
                expected.push(Some(total));
            }
            for result in [runtime.result(), native.result()] {
                for plot in result.plots {
                    assert_eq!(
                        plot.values.iter().map(|v| v.as_f64()).collect::<Vec<_>>(),
                        expected,
                        "{field}, history={history}"
                    );
                }
            }
        }
    }
}

#[test]
fn static_root_append_retains_committed_history_and_invalid_input_can_be_retried() {
    let (bars, frames, groups) = inputs();
    for history in [false, true] {
        let hir = program("close", history);
        let mut runtime = HistoricalRuntime::with_request_environment(&hir, environment())
            .with_external_schedule(vec![], vec![], ExternalExecutionOptions::default())
            .unwrap();
        runtime
            .append_external_schedule(&bars[..1], frames[..1].to_vec(), groups[..1].to_vec(), None)
            .unwrap();
        let before = pine_runtime::public_runtime_result_json(&runtime.result());
        let mut invalid = groups[1..].to_vec();
        invalid[0][0].evaluation.event_time_ms = 0;
        assert!(
            runtime
                .append_external_schedule(&bars[1..], frames[1..].to_vec(), invalid, None)
                .is_err()
        );
        assert_eq!(
            pine_runtime::public_runtime_result_json(&runtime.result()),
            before
        );
        runtime
            .append_external_schedule(&bars[1..], frames[1..].to_vec(), groups[1..].to_vec(), None)
            .unwrap();
        let result = runtime.result();
        assert_eq!(result.plots[0].values, result.plots[1].values);
        assert_eq!(result.plots[0].values[0].as_f64(), Some(10120911.0));
    }
}
