use pine_runtime::{
    Bar, ChartContext, ExternalAccountFrame, ExternalBarState, ExternalEvaluationTrigger,
    ExternalExecutionOptions, ExternalPass, ExternalPassContext, ExternalRequestWindowHistory,
    ExternalScheduledPass, HistoricalRuntime, InMemoryRequestDataProvider, ObservedRequestUpdate,
    RequestEnvironment, RequestKey, RequestTimeframe, public_runtime_result_json,
};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;
use std::sync::Arc;

fn program(body: &str) -> pine_ir::HirProgram {
    let analysis = analyze_source(&SourceFile::new("feedback.pine", body));
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
        volume: 4.0,
    }
}
fn observations(b: Bar) -> Vec<ExternalScheduledPass> {
    (0..4)
        .map(|tick| ExternalScheduledPass {
            evaluation: ExternalPass {
                account: ExternalAccountFrame {
                    time: b.time,
                    position_size: f64::from(tick > 0),
                    position_avg_price: (tick > 0).then_some(b.close),
                    equity: 1000.0,
                    initial_capital: 1000.0,
                    netprofit: 0.0,
                    openprofit: 0.0,
                },
                bar: Bar {
                    volume: (tick + 1) as f64,
                    ..b
                },
                event_time_ms: b.time + tick * 19999,
                confirmed: tick == 3,
                request_data: None,
                request_updates: Some(if tick == 0 {
                    vec![ObservedRequestUpdate {
                        symbol: "B".into(),
                        timeframe: "1".into(),
                        bar: bar(b.time, b.close * 10.0),
                        kind: pine_runtime::BarUpdateKind::Confirmed,
                        available_at_ms: b.time + 59999,
                    }]
                } else {
                    vec![]
                }),
            },
            context: ExternalPassContext {
                bar_state: ExternalBarState::Historical,
                trigger: ExternalEvaluationTrigger::HistoricalTick,
                is_new: true,
            },
        })
        .collect()
}
fn environment() -> RequestEnvironment {
    let provider = InMemoryRequestDataProvider::from_streams([(
        RequestKey::new("B", RequestTimeframe::parse("1").unwrap()),
        vec![],
    )])
    .unwrap();
    RequestEnvironment::new(
        ChartContext::new("A", RequestTimeframe::parse("1").unwrap()),
        Arc::new(provider),
    )
}

#[test]
fn actual_account_selector_rebases_all_state_and_the_feed_before_a_stable_final_pass() {
    let hir = program(
        "//@version=6\nstrategy(\"feedback\",calc_on_every_history_tick=true,calc_bars_count=1)\nvarip int n=0\nn+=1\nstrategy.entry(str.tostring(n),strategy.long,qty=1)\ns=strategy.position_size>0?\"A\":\"B\"\nplot(request.security(s,\"1\",ta.cum(close),calc_bars_count=2))\nplot(n)\nplot(strategy.position_size)",
    );
    let bars = vec![bar(0, 10.0), bar(60000, 20.0), bar(120000, 30.0)];
    let headers = vec![ExternalRequestWindowHistory {
        symbol: "B".into(),
        timeframe: "1".into(),
        times_ms: vec![0, 60000, 120000],
    }];
    let mut stream = HistoricalRuntime::with_request_environment(&hir, environment())
        .with_external_schedule(vec![], vec![], ExternalExecutionOptions::default())
        .unwrap();
    assert_eq!(
        stream
            .initialize_external_stream_dataset_with_observed_feedback(&bars, &headers)
            .unwrap(),
        2
    );
    let initial = stream.external_window_feedback_status().unwrap();
    assert!(initial.requires_discovery && !initial.rebase_required);
    assert!(stream.rebase_external_stream_window(false).is_err());
    stream
        .begin_external_bar(bars[2], ExternalBarState::Historical)
        .unwrap();
    let first = observations(bars[2]);
    stream.observe_external_bar(first[0].clone()).unwrap();
    assert_eq!(stream.external_intents().unwrap().len(), 1);
    assert!(stream.observe_external_bar(first[1].clone()).is_err());
    assert!(
        stream
            .external_window_feedback_status()
            .unwrap()
            .rebase_required
    );
    assert!(stream.rebase_external_stream_window(true).is_err());
    assert_eq!(stream.rebase_external_stream_window(false).unwrap(), 1);
    assert!(!stream.external_bar_open());
    assert!(stream.external_intents().unwrap().is_empty());
    assert!(stream.result().plots.is_empty());
    assert!(
        stream
            .begin_external_bar(bars[2], ExternalBarState::Historical)
            .is_err()
    );
    // A regressed provider clock would fail here if reset retained the first
    // attempt's requested bar at 120000 instead of its original empty feed.
    for pass in 0..2 {
        for b in &bars[1..] {
            stream
                .begin_external_bar(*b, ExternalBarState::Historical)
                .unwrap();
            for observation in observations(*b) {
                stream.observe_external_bar(observation).unwrap();
            }
            stream.commit_external_bar().unwrap();
        }
        assert_eq!(
            stream.result().plots[1]
                .values
                .iter()
                .map(|v| v.as_f64())
                .collect::<Vec<_>>(),
            vec![Some(4.0), Some(8.0)]
        );
        assert_eq!(stream.external_intents().unwrap().len(), 8);
        assert!(
            !stream
                .external_window_feedback_status()
                .unwrap()
                .rebase_required
        );
        if pass == 0 {
            assert_eq!(stream.rebase_external_stream_window(true).unwrap(), 1);
            assert!(
                !stream
                    .external_window_feedback_status()
                    .unwrap()
                    .requires_discovery
            );
            assert!(stream.external_intents().unwrap().is_empty());
        }
    }
    let groups: Vec<_> = bars.iter().copied().map(observations).collect();
    let accounts = groups
        .iter()
        .map(|g| g.last().unwrap().evaluation.account.clone())
        .collect();
    let mut complete = HistoricalRuntime::with_request_environment(&hir, environment())
        .with_external_schedule(accounts, groups, ExternalExecutionOptions::default())
        .unwrap();
    complete.initialize_historical_dataset(&bars, None).unwrap();
    for b in &bars[1..] {
        complete.append_bar(*b).unwrap();
    }
    assert_eq!(
        public_runtime_result_json(&stream.result()),
        public_runtime_result_json(&complete.result())
    );
    assert_eq!(
        serde_json::to_value(stream.external_intents().unwrap()).unwrap(),
        serde_json::to_value(complete.external_intents().unwrap()).unwrap()
    );
}

#[test]
fn ordinary_pine_errors_cannot_be_recovered_by_window_reset_or_seal() {
    let hir = program(
        "//@version=6\nstrategy(\"failure\",calc_on_every_history_tick=true,calc_bars_count=1)\ns=strategy.position_size>0?\"A\":\"B\"\nplot(request.security(s,\"1\",close,calc_bars_count=1))\nruntime.error(\"actual script failure\")",
    );
    let b = bar(0, 10.0);
    let mut stream = HistoricalRuntime::with_request_environment(&hir, environment())
        .with_external_schedule(vec![], vec![], ExternalExecutionOptions::default())
        .unwrap();
    stream
        .initialize_external_stream_dataset_with_observed_feedback(&[b], &[])
        .unwrap();
    stream
        .begin_external_bar(b, ExternalBarState::Historical)
        .unwrap();
    assert!(
        stream
            .observe_external_bar(observations(b)[0].clone())
            .unwrap_err()
            .message
            .contains("actual script failure")
    );
    assert!(
        !stream
            .external_window_feedback_status()
            .unwrap()
            .rebase_required
    );
    assert!(stream.rebase_external_stream_window(false).is_err());
    assert!(stream.rebase_external_stream_window(true).is_err());
    assert!(stream.is_execution_poisoned());
}
