use pine_runtime::{
    Bar, BarUpdate, ChartContext, ExternalAccountFrame, ExternalBarState,
    ExternalEvaluationTrigger, ExternalExecutionOptions, ExternalPass, ExternalPassContext,
    ExternalScheduledPass, HistoricalRuntime, InMemoryRequestDataProvider, ObservedRequestUpdate,
    RealtimeRuntime, RequestEnvironment, RequestKey, RequestTimeframe, public_runtime_result_json,
};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;
use std::sync::Arc;

fn program(body: &str) -> pine_ir::HirProgram {
    let analysis = analyze_source(&SourceFile::new("stream.pine", body));
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    analysis.hir.unwrap()
}

fn bar(time: i64, price: f64) -> Bar {
    Bar {
        time,
        open: price,
        high: price,
        low: price,
        close: price,
        volume: 100.0,
    }
}

fn account(bar: Bar, position: f64) -> ExternalAccountFrame {
    ExternalAccountFrame {
        time: bar.time,
        position_size: position,
        position_avg_price: (position != 0.0).then_some(bar.open),
        initial_capital: 1000.0,
        equity: 1000.0,
        netprofit: 0.0,
        openprofit: 0.0,
    }
}

fn inputs(
    state: ExternalBarState,
) -> (
    Vec<Bar>,
    Vec<ExternalAccountFrame>,
    Vec<Vec<ExternalScheduledPass>>,
) {
    let mut bars = vec![];
    let mut accounts = vec![];
    let mut groups = vec![];
    for i in 0..2 {
        let open = 10.0 + 10.0 * i as f64;
        let canonical = Bar {
            time: i * 60000,
            open,
            high: open + 2.0,
            low: open - 1.0,
            close: open + 1.0,
            volume: 100.0 * (i + 1) as f64,
        };
        let mut high = open;
        let mut low = open;
        let mut group = vec![];
        for (tick, price) in [open, canonical.high, canonical.low, canonical.close]
            .into_iter()
            .enumerate()
        {
            high = high.max(price);
            low = low.min(price);
            let observed = Bar {
                high,
                low,
                close: price,
                volume: canonical.volume * (tick + 1) as f64 / 4.0,
                ..canonical
            };
            let trigger = match state {
                ExternalBarState::Historical => ExternalEvaluationTrigger::HistoricalTick,
                ExternalBarState::Realtime if tick == 3 => ExternalEvaluationTrigger::BarClose,
                ExternalBarState::Realtime => ExternalEvaluationTrigger::RealtimeTick,
            };
            group.push(ExternalScheduledPass {
                evaluation: ExternalPass {
                    bar: observed,
                    account: account(observed, tick as f64),
                    event_time_ms: canonical.time + 59999 * tick as i64 / 3,
                    confirmed: tick == 3,
                    request_data: None,
                    request_updates: None,
                },
                context: ExternalPassContext {
                    bar_state: state,
                    trigger,
                    is_new: state == ExternalBarState::Historical || tick == 0,
                },
            });
        }
        accounts.push(group.last().unwrap().evaluation.account.clone());
        bars.push(canonical);
        groups.push(group);
    }
    (bars, accounts, groups)
}

#[test]
fn streaming_observations_match_complete_schedules_and_commit_history_once() {
    let hir = program(
        "//@version=6\nstrategy(\"stream\",calc_on_every_tick=true,calc_on_every_history_tick=true)\nvar int ordinary=0\nvarip int retained=0\nordinary+=1\nretained+=1\nvar a=array.new_float()\nvarip keep=array.new_float()\narray.push(a,close)\narray.push(keep,close)\nplot(ordinary)\nplot(retained)\nplot(array.size(a))\nplot(array.size(keep))\nplot(ta.cum(volume))\nplot(strategy.position_size)\nplot(close[1])\nstrategy.entry(str.tostring(retained),strategy.long,qty=1)\nalert(str.tostring(retained),alert.freq_all)",
    );
    for state in [ExternalBarState::Historical, ExternalBarState::Realtime] {
        let (bars, accounts, groups) = inputs(state);
        let mut complete = HistoricalRuntime::new(&hir)
            .with_external_schedule(
                accounts,
                groups.clone(),
                ExternalExecutionOptions::default(),
            )
            .unwrap();
        complete.initialize_historical_dataset(&bars, None).unwrap();
        for b in &bars {
            complete.append_bar(*b).unwrap();
        }
        let mut stream = HistoricalRuntime::new(&hir)
            .with_external_schedule(vec![], vec![], ExternalExecutionOptions::default())
            .unwrap();
        stream.initialize_historical_dataset(&bars, None).unwrap();
        for (index, (b, group)) in bars.iter().zip(groups).enumerate() {
            stream.begin_external_bar(*b, state).unwrap();
            assert!(stream.commit_external_bar().is_err());
            for (tick, pass) in group.into_iter().enumerate() {
                stream.observe_external_bar(pass).unwrap();
                assert_eq!(
                    stream.result().plots[1].values[index].as_f64(),
                    Some((index * 4 + tick + 1) as f64)
                );
            }
            assert!(stream.external_bar_open());
            assert!(stream.begin_external_bar(*b, state).is_err());
            stream.commit_external_bar().unwrap();
            assert!(!stream.external_bar_open());
        }
        assert_eq!(
            public_runtime_result_json(&stream.result()),
            public_runtime_result_json(&complete.result())
        );
        assert_eq!(
            serde_json::to_value(stream.external_intents().unwrap()).unwrap(),
            serde_json::to_value(complete.external_intents().unwrap()).unwrap()
        );
        let values: Vec<_> = stream
            .result()
            .plots
            .iter()
            .map(|p| p.values.iter().map(|v| v.as_f64()).collect::<Vec<_>>())
            .collect();
        assert_eq!(values[0], vec![Some(1.0), Some(2.0)]);
        assert_eq!(values[1], vec![Some(4.0), Some(8.0)]);
        assert_eq!(values[2], vec![Some(1.0), Some(2.0)]);
        assert_eq!(values[3], vec![Some(4.0), Some(8.0)]);
        assert_eq!(values[4], vec![Some(100.0), Some(300.0)]);
        assert_eq!(values[6], vec![None, Some(11.0)]);
    }
}

#[test]
fn realtime_chart_boundaries_follow_the_observed_bar_in_complete_and_streaming_execution() {
    let hir = program(
        "//@version=6\nstrategy(\"boundaries\",calc_on_every_tick=true)\nplot(last_bar_index)\nplot(last_bar_time)",
    );
    let (bars, accounts, groups) = inputs(ExternalBarState::Realtime);
    let mut complete = HistoricalRuntime::new(&hir)
        .with_external_schedule(
            accounts,
            groups.clone(),
            ExternalExecutionOptions::default(),
        )
        .unwrap();
    complete.initialize_historical_dataset(&bars, None).unwrap();
    for b in &bars {
        complete.append_bar(*b).unwrap();
    }
    let mut stream = HistoricalRuntime::new(&hir)
        .with_external_schedule(vec![], vec![], ExternalExecutionOptions::default())
        .unwrap();
    stream.initialize_historical_dataset(&bars, None).unwrap();
    let mut native = RealtimeRuntime::new(&hir);
    native.seed_historical(&[]).unwrap();
    for (b, group) in bars.iter().zip(groups) {
        stream
            .begin_external_bar(*b, ExternalBarState::Realtime)
            .unwrap();
        for pass in group {
            let update = if pass.evaluation.confirmed {
                BarUpdate::confirmed(pass.evaluation.bar)
            } else {
                BarUpdate::forming(pass.evaluation.bar)
            };
            native.update(update).unwrap();
            stream.observe_external_bar(pass).unwrap();
        }
        stream.commit_external_bar().unwrap();
    }
    for runtime in [&complete, &stream] {
        assert_eq!(runtime.result().plots, native.result().plots);
        assert_eq!(
            runtime.result().plots[0]
                .values
                .iter()
                .map(|v| v.as_f64())
                .collect::<Vec<_>>(),
            [Some(0.0), Some(1.0)]
        );
    }
}

#[test]
fn rejected_stream_input_preserves_observations_and_can_be_retried() {
    let hir = program(
        "//@version=6\nstrategy(\"retry\",calc_on_every_tick=true)\nvarip int n=0\nn+=1\nplot(n)",
    );
    let (bars, _, groups) = inputs(ExternalBarState::Realtime);
    let mut stream = HistoricalRuntime::new(&hir)
        .with_external_schedule(vec![], vec![], ExternalExecutionOptions::default())
        .unwrap();
    assert!(stream.observe_external_bar(groups[0][0].clone()).is_err());
    stream
        .begin_external_bar(bars[0], ExternalBarState::Realtime)
        .unwrap();
    stream.observe_external_bar(groups[0][0].clone()).unwrap();
    let before = public_runtime_result_json(&stream.result());
    let mut bad = groups[0][1].clone();
    bad.evaluation.event_time_ms = -1;
    assert!(stream.observe_external_bar(bad).is_err());
    assert_eq!(public_runtime_result_json(&stream.result()), before);
    assert!(stream.commit_external_bar().is_err());
    assert!(stream.append_bar(bars[1]).is_err());
    for pass in &groups[0][1..] {
        stream.observe_external_bar(pass.clone()).unwrap();
    }
    stream.commit_external_bar().unwrap();
    stream
        .begin_external_bar(bars[1], ExternalBarState::Realtime)
        .unwrap();
    for pass in &groups[1] {
        stream.observe_external_bar(pass.clone()).unwrap();
    }
    stream.commit_external_bar().unwrap();
    assert_eq!(
        stream.result().plots[0].values.last().unwrap().as_f64(),
        Some(8.0)
    );
}

#[test]
fn historical_requested_contexts_match_native_values_at_every_synthetic_vertex() {
    let source = "//@version=6\nstrategy(\"requested vertices\",calc_on_every_history_tick=true)\nvarip int n=0\nn+=1\nx=request.security(\"B\",\"1\",close)\nvarip array<float> seen=array.new_float(4,na)\narray.set(seen,(n-1)%4,x)\nplot(array.get(seen,0))\nplot(array.get(seen,1))\nplot(array.get(seen,2))\nplot(array.get(seen,3))";
    let hir = program(source);
    let tf = RequestTimeframe::parse("1").unwrap();
    let environment = |data| {
        RequestEnvironment::new(
            ChartContext::new("A", tf.clone()),
            Arc::new(
                InMemoryRequestDataProvider::from_streams([(
                    RequestKey::new("B", tf.clone()),
                    data,
                )])
                .unwrap(),
            ),
        )
    };
    let (bars, _, groups) = inputs(ExternalBarState::Historical);
    let requested = vec![bar(0, 100.0), bar(60000, 200.0)];
    let mut native =
        HistoricalRuntime::with_request_environment(&hir, environment(requested.clone()));
    let mut stream = HistoricalRuntime::with_request_environment(&hir, environment(vec![]))
        .with_external_schedule(vec![], vec![], ExternalExecutionOptions::default())
        .unwrap();
    assert_eq!(stream.initialize_external_stream_dataset(&bars).unwrap(), 0);
    for (index, (canonical, passes)) in bars.iter().zip(groups).enumerate() {
        native.append_bar(*canonical).unwrap();
        stream
            .begin_external_bar(*canonical, ExternalBarState::Historical)
            .unwrap();
        for (ordinal, mut pass) in passes.into_iter().enumerate() {
            pass.evaluation.request_updates = Some(if ordinal == 0 {
                vec![ObservedRequestUpdate {
                    symbol: "B".to_owned(),
                    timeframe: "1".to_owned(),
                    bar: requested[index],
                    kind: pine_runtime::BarUpdateKind::Confirmed,
                    available_at_ms: canonical.time + 59999,
                }]
            } else {
                vec![]
            });
            stream.observe_external_bar(pass).unwrap();
        }
        stream.commit_external_bar().unwrap();
    }
    assert_eq!(stream.result().plots, native.result().plots);
    for plot in &stream.result().plots {
        assert_eq!(
            plot.values
                .iter()
                .map(|value| value.as_f64())
                .collect::<Vec<_>>(),
            vec![Some(100.0), Some(200.0)]
        );
    }
}

#[test]
fn historical_reveal_rejects_later_chart_data_and_realtime_keeps_observation_time() {
    let hir = program(
        "//@version=6\nstrategy(\"clock guards\",calc_on_every_history_tick=true,calc_on_every_tick=true)\nplot(request.security(\"B\",\"1\",close))",
    );
    let tf = RequestTimeframe::parse("1").unwrap();
    for state in [ExternalBarState::Historical, ExternalBarState::Realtime] {
        let provider =
            InMemoryRequestDataProvider::from_streams([(RequestKey::new("B", tf.clone()), vec![])])
                .unwrap();
        let mut stream = HistoricalRuntime::with_request_environment(
            &hir,
            RequestEnvironment::new(ChartContext::new("A", tf.clone()), Arc::new(provider)),
        )
        .with_external_schedule(vec![], vec![], ExternalExecutionOptions::default())
        .unwrap();
        let (bars, _, mut groups) = inputs(state);
        stream.begin_external_bar(bars[0], state).unwrap();
        let before = public_runtime_result_json(&stream.result());
        let mut pass = groups.remove(0).remove(0);
        let requested_time = if state == ExternalBarState::Historical {
            60000
        } else {
            0
        };
        pass.evaluation.request_updates = Some(vec![ObservedRequestUpdate {
            symbol: "B".to_owned(),
            timeframe: "1".to_owned(),
            bar: bar(requested_time, 100.0),
            kind: pine_runtime::BarUpdateKind::Confirmed,
            available_at_ms: requested_time + 59999,
        }]);
        assert!(stream.observe_external_bar(pass).is_err());
        assert_eq!(public_runtime_result_json(&stream.result()), before);
        assert!(!stream.is_execution_poisoned());
    }
}

#[test]
fn observed_request_transactions_between_passes_are_not_reapplied_by_pair_validation() {
    let hir = program(
        "//@version=6\nstrategy(\"feed\",calc_on_every_tick=true)\nvarip int n=0\nn+=1\nplot(n)\nplot(request.security(\"B\",\"1\",close))",
    );
    let tf = RequestTimeframe::parse("1").unwrap();
    let provider =
        InMemoryRequestDataProvider::from_streams([(RequestKey::new("B", tf.clone()), vec![])])
            .unwrap();
    let env = RequestEnvironment::new(ChartContext::new("A", tf), Arc::new(provider));
    let mut stream = HistoricalRuntime::with_request_environment(&hir, env)
        .with_external_schedule(vec![], vec![], ExternalExecutionOptions::default())
        .unwrap();
    let (bars, _, mut groups) = inputs(ExternalBarState::Realtime);
    stream
        .begin_external_bar(bars[0], ExternalBarState::Realtime)
        .unwrap();
    let mut first = groups.remove(0);
    first[0].evaluation.request_updates = Some(vec![ObservedRequestUpdate {
        symbol: "B".to_owned(),
        timeframe: "1".to_owned(),
        bar: bar(0, 100.0),
        kind: pine_runtime::BarUpdateKind::Forming,
        available_at_ms: 0,
    }]);
    stream.observe_external_bar(first[0].clone()).unwrap();
    stream
        .apply_external_request_updates(
            &[ObservedRequestUpdate {
                symbol: "B".to_owned(),
                timeframe: "1".to_owned(),
                bar: bar(0, 200.0),
                kind: pine_runtime::BarUpdateKind::Forming,
                available_at_ms: 1000,
            }],
            1000,
        )
        .unwrap();
    let mut bad = first[1].clone();
    bad.evaluation.event_time_ms = 500;
    assert!(stream.observe_external_bar(bad).is_err());
    for pass in &first[1..] {
        stream.observe_external_bar(pass.clone()).unwrap();
    }
    stream.commit_external_bar().unwrap();
    assert_eq!(stream.result().plots[0].values[0].as_f64(), Some(4.0));
    assert_eq!(stream.result().plots[1].values[0].as_f64(), Some(200.0));
}

#[test]
fn market_confirmation_keeps_the_bar_open_for_explicit_closing_fill_feedback() {
    for every_tick in [false, true] {
        let hir = program(&format!(
            "//@version=6\nstrategy(\"closing\",calc_on_order_fills=true,process_orders_on_close=true,calc_on_every_tick={every_tick})\nvar int ordinary=0\nvarip int n=0\nordinary+=1\nn+=1\nplot(ordinary)\nplot(n)\nplot(strategy.position_size)\nplot(ta.cum(volume))"
        ));
        let canonical = bar(0, 10.0);
        let mut first = ExternalScheduledPass {
            evaluation: ExternalPass {
                bar: canonical,
                account: account(canonical, 0.0),
                event_time_ms: 59999,
                confirmed: true,
                request_data: None,
                request_updates: None,
            },
            context: ExternalPassContext {
                bar_state: ExternalBarState::Realtime,
                trigger: ExternalEvaluationTrigger::BarClose,
                is_new: true,
            },
        };
        let mut closing = first.clone();
        closing.context.is_new = false;
        closing.context.trigger = ExternalEvaluationTrigger::ClosingFill;
        closing.evaluation.account = account(canonical, 1.0);
        let options = ExternalExecutionOptions {
            closing_ticks: true,
            ..Default::default()
        };
        let mut stream = HistoricalRuntime::new(&hir)
            .with_external_schedule(vec![], vec![], options)
            .unwrap();
        stream
            .begin_external_bar(canonical, ExternalBarState::Realtime)
            .unwrap();
        stream.observe_external_bar(first.clone()).unwrap();
        let before = public_runtime_result_json(&stream.result());
        let mut invalid = closing.clone();
        invalid.evaluation.event_time_ms += 1;
        assert!(stream.observe_external_bar(invalid).is_err());
        assert_eq!(public_runtime_result_json(&stream.result()), before);
        stream.observe_external_bar(closing.clone()).unwrap();
        stream.commit_external_bar().unwrap();
        let values: Vec<_> = stream
            .result()
            .plots
            .iter()
            .map(|p| p.values[0].as_f64())
            .collect();
        assert_eq!(values, [Some(1.0), Some(2.0), Some(1.0), Some(100.0)]);
        let mut complete = HistoricalRuntime::new(&hir)
            .with_external_schedule(
                vec![closing.evaluation.account.clone()],
                vec![vec![first.clone(), closing]],
                options,
            )
            .unwrap();
        complete.append_bar(canonical).unwrap();
        assert_eq!(
            public_runtime_result_json(&stream.result()),
            public_runtime_result_json(&complete.result())
        );
        first.evaluation.bar.time = 60000;
        first.evaluation.account.time = 60000;
        first.evaluation.event_time_ms = 119999;
        stream
            .begin_external_bar(first.evaluation.bar, ExternalBarState::Realtime)
            .unwrap();
        stream.observe_external_bar(first).unwrap();
        stream.commit_external_bar().unwrap();
        assert_eq!(stream.result().plots[0].values[1].as_f64(), Some(2.0));
    }
}

#[test]
fn a_cloned_open_bar_keeps_rollback_and_varip_continuation_independent() {
    let hir = program(
        "//@version=6\nstrategy(\"clone\",calc_on_every_tick=true)\nvarip a=array.new_float()\narray.push(a,close)\nplot(array.sum(a))\nplot(ta.cum(volume))",
    );
    let (bars, _, groups) = inputs(ExternalBarState::Realtime);
    let mut stream = HistoricalRuntime::new(&hir)
        .with_external_schedule(vec![], vec![], ExternalExecutionOptions::default())
        .unwrap();
    stream
        .begin_external_bar(bars[0], ExternalBarState::Realtime)
        .unwrap();
    for pass in &groups[0][..2] {
        stream.observe_external_bar(pass.clone()).unwrap();
    }
    let mut clone = stream.clone();
    for pass in &groups[0][2..] {
        stream.observe_external_bar(pass.clone()).unwrap();
    }
    stream.commit_external_bar().unwrap();
    assert_ne!(
        public_runtime_result_json(&stream.result()),
        public_runtime_result_json(&clone.result())
    );
    for pass in &groups[0][2..] {
        clone.observe_external_bar(pass.clone()).unwrap();
    }
    clone.commit_external_bar().unwrap();
    assert_eq!(
        public_runtime_result_json(&stream.result()),
        public_runtime_result_json(&clone.result())
    );
}

#[test]
fn thousands_of_observations_do_not_consume_the_fill_loop_allowance() {
    let hir = program(
        "//@version=6\nstrategy(\"many\",calc_on_every_tick=true)\nvarip int n=0\nn+=1\nplot(n)",
    );
    let canonical = bar(0, 10.0);
    let mut stream = HistoricalRuntime::new(&hir)
        .with_external_schedule(vec![], vec![], ExternalExecutionOptions::default())
        .unwrap();
    stream
        .begin_external_bar(canonical, ExternalBarState::Realtime)
        .unwrap();
    for index in 0..2048 {
        let confirmed = index == 2047;
        stream
            .observe_external_bar(ExternalScheduledPass {
                evaluation: ExternalPass {
                    bar: canonical,
                    account: account(canonical, 0.0),
                    event_time_ms: index,
                    confirmed,
                    request_data: None,
                    request_updates: None,
                },
                context: ExternalPassContext {
                    bar_state: ExternalBarState::Realtime,
                    trigger: if confirmed {
                        ExternalEvaluationTrigger::BarClose
                    } else {
                        ExternalEvaluationTrigger::RealtimeTick
                    },
                    is_new: index == 0,
                },
            })
            .unwrap();
    }
    stream.commit_external_bar().unwrap();
    assert_eq!(stream.result().plots[0].values[0].as_f64(), Some(2048.0));
}

#[test]
fn execution_failure_poisons_the_stream_while_retaining_the_open_failure_state() {
    let hir = program(
        "//@version=6\nstrategy(\"error\",calc_on_every_tick=true)\nvarip int n=0\nn+=1\nplot(n)\nif n==2\n    runtime.error(\"deliberate failure\")",
    );
    let (bars, _, groups) = inputs(ExternalBarState::Realtime);
    let mut stream = HistoricalRuntime::new(&hir)
        .with_external_schedule(vec![], vec![], ExternalExecutionOptions::default())
        .unwrap();
    stream
        .begin_external_bar(bars[0], ExternalBarState::Realtime)
        .unwrap();
    stream.observe_external_bar(groups[0][0].clone()).unwrap();
    assert!(
        stream
            .observe_external_bar(groups[0][1].clone())
            .unwrap_err()
            .message
            .contains("deliberate failure")
    );
    assert!(stream.external_bar_open());
    assert!(
        stream
            .observe_external_bar(groups[0][2].clone())
            .unwrap_err()
            .message
            .contains("E_RUNTIME_POISONED")
    );
    assert!(
        stream
            .commit_external_bar()
            .unwrap_err()
            .message
            .contains("E_RUNTIME_POISONED")
    );
}
