use super::*;

fn compiled(source: &str) -> pine_ir::HirProgram {
    let analysis = analyze_source(&modern_source_file("schedule", source));
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    analysis.hir.unwrap()
}

fn row(time: i64, close: f64) -> Bar {
    Bar {
        time,
        open: 10.0,
        high: 12.0,
        low: 9.0,
        close,
        volume: 100.0,
    }
}

fn frame(time: i64, qty: f64) -> ExternalAccountFrame {
    ExternalAccountFrame {
        time,
        position_size: qty,
        position_avg_price: (qty != 0.0).then_some(10.0),
        equity: 1000.0,
        initial_capital: 1000.0,
        netprofit: 0.0,
        openprofit: 0.0,
    }
}

fn pass(
    bar: Bar,
    qty: f64,
    commit: bool,
    state: ExternalBarState,
    trigger: ExternalEvaluationTrigger,
    is_new: bool,
) -> ExternalScheduledPass {
    ExternalScheduledPass {
        evaluation: ExternalPass {
            account: frame(bar.time, qty),
            bar,
            event_time_ms: bar.time + bar.close as i64,
            confirmed: commit,
            request_data: None,
            request_updates: None,
        },
        context: ExternalPassContext {
            bar_state: state,
            trigger,
            is_new,
        },
    }
}

#[test]
fn scheduled_historical_fills_separate_script_confirmation_from_commit() {
    let hir = compiled(
        "strategy(\"fills\",calc_on_order_fills=true)\nvar int ordinary=0\nvarip int count=0\nordinary+=1\ncount+=1\nplot(ordinary)\nplot(count)\nplot(barstate.isconfirmed?1:0)\nplot(barstate.isnew?1:0)\nplot(barstate.ishistory?1:0)\nplot(barstate.isrealtime?1:0)\nif strategy.position_size==0\n    strategy.entry(\"L\",strategy.long,qty=2)",
    );
    let mut runtime = HistoricalRuntime::new(&hir)
        .with_external_schedule(
            vec![frame(0, 0.0)],
            vec![vec![
                pass(
                    row(0, 10.0),
                    2.0,
                    false,
                    ExternalBarState::Historical,
                    ExternalEvaluationTrigger::OrderFill,
                    true,
                ),
                pass(
                    row(0, 12.0),
                    0.0,
                    true,
                    ExternalBarState::Historical,
                    ExternalEvaluationTrigger::BarClose,
                    true,
                ),
            ]],
            ExternalExecutionOptions::default(),
        )
        .unwrap();
    runtime.append_bar(row(0, 12.0)).unwrap();
    let result = runtime.result();
    for (plot, expected) in result.plots.iter().zip([1, 2, 1, 1, 1, 0]) {
        assert_eq!(plot.values, vec![PineValue::Int(expected)]);
    }
    assert!(result.strategy.is_none());
    assert_eq!(runtime.strategy_broker.public_fill_event_count(), 0);
    assert_eq!(runtime.external_intents().unwrap().len(), 1);
    assert_eq!(runtime.external_intents().unwrap()[0].pass_index, Some(1));
}

#[test]
fn explicit_history_ticks_use_historical_state_and_only_final_market_commit() {
    let hir = compiled(
        "//@version=6\nstrategy(\"history ticks\",calc_on_every_history_tick=true)\nvar int ordinary=0\nvarip int count=0\nvarip int confirms=0\nordinary+=1\ncount+=1\nconfirms:=confirms*10+(barstate.isconfirmed and barstate.isnew?1:0)\nplot(ordinary)\nplot(count)\nplot(confirms)\nplot(close)\nif strategy.position_size==0\n    strategy.entry(\"L\",strategy.long,qty=1)",
    );
    let mut schedule = Vec::new();
    for (index, close) in [10.0, 12.0, 9.0, 11.0].into_iter().enumerate() {
        let mut item = pass(
            row(0, close),
            if index == 1 { 1.0 } else { 0.0 },
            index == 3,
            ExternalBarState::Historical,
            ExternalEvaluationTrigger::HistoricalTick,
            true,
        );
        item.evaluation.event_time_ms = index as i64;
        item.evaluation.bar.volume = 25.0 * (index + 1) as f64;
        schedule.push(item);
    }
    assert!(
        HistoricalRuntime::new(&hir)
            .with_external_accounts(vec![frame(0, 0.0)])
            .is_err()
    );
    let mut runtime = HistoricalRuntime::new(&hir)
        .with_external_schedule(
            vec![frame(0, 0.0)],
            vec![schedule],
            ExternalExecutionOptions::default(),
        )
        .unwrap();
    runtime.append_bar(row(0, 11.0)).unwrap();
    for (plot, expected) in runtime.result().plots.iter().zip([1.0, 4.0, 1111.0, 11.0]) {
        assert_eq!(plot.values[0].as_f64(), Some(expected));
    }
    assert_eq!(
        runtime
            .external_intents()
            .unwrap()
            .iter()
            .map(|intent| intent.pass_index)
            .collect::<Vec<_>>(),
        [Some(0), Some(2), Some(3)]
    );
}

#[test]
fn scheduled_realtime_ticks_without_fill_recalculation_rollback_and_keep_varip() {
    let hir = compiled(
        "strategy(\"ticks\",calc_on_every_tick=true)\nvar int ordinary=0\nvarip float prices=0\nvarip int news=0\nvarip int lasts=0\nordinary+=1\nprices:=prices*100+close\nnews:=news*10+(barstate.isnew?1:0)\nlasts:=lasts*10+(barstate.islast?1:0)\nplot(ordinary)\nplot(prices)\nplot(news)\nplot(lasts)\nplot(barstate.ishistory?1:0)\nplot(barstate.isrealtime?1:0)\nplot(barstate.isconfirmed?1:0)",
    );
    assert!(
        HistoricalRuntime::new(&hir)
            .with_external_accounts(vec![frame(0, 0.0)])
            .is_err()
    );
    let mut runtime = HistoricalRuntime::new(&hir)
        .with_external_schedule(
            vec![frame(0, 0.0)],
            vec![vec![
                pass(
                    row(0, 10.0),
                    0.0,
                    false,
                    ExternalBarState::Realtime,
                    ExternalEvaluationTrigger::RealtimeTick,
                    true,
                ),
                pass(
                    row(0, 11.0),
                    0.0,
                    false,
                    ExternalBarState::Realtime,
                    ExternalEvaluationTrigger::RealtimeTick,
                    false,
                ),
                pass(
                    row(0, 12.0),
                    0.0,
                    true,
                    ExternalBarState::Realtime,
                    ExternalEvaluationTrigger::BarClose,
                    false,
                ),
            ]],
            ExternalExecutionOptions::default(),
        )
        .unwrap();
    runtime.append_bar(row(0, 12.0)).unwrap();
    let plots = runtime.result().plots;
    for (plot, expected) in plots
        .iter()
        .zip([1.0, 101112.0, 100.0, 111.0, 0.0, 1.0, 1.0])
    {
        assert_eq!(plot.values[0].as_f64(), Some(expected));
    }
}

#[test]
fn explicit_tick_groups_can_exceed_fill_callback_limits_with_bounded_observations() {
    for historical in [true, false] {
        let flag = if historical {
            "calc_on_every_history_tick"
        } else {
            "calc_on_every_tick"
        };
        let hir = compiled(&format!(
            "//@version=6\nstrategy(\"many ticks\",{flag}=true)\nvarip int count=0\ncount+=1\nplot(count)"
        ));
        let group = (0..2048)
            .map(|index| {
                let mut item = pass(
                    row(0, 11.0),
                    0.0,
                    index == 2047,
                    if historical {
                        ExternalBarState::Historical
                    } else {
                        ExternalBarState::Realtime
                    },
                    if historical {
                        ExternalEvaluationTrigger::HistoricalTick
                    } else {
                        ExternalEvaluationTrigger::RealtimeTick
                    },
                    historical || index == 0,
                );
                item.evaluation.event_time_ms = index;
                item
            })
            .collect();
        let mut runtime = HistoricalRuntime::new(&hir)
            .with_external_schedule(
                vec![frame(0, 0.0)],
                vec![group],
                ExternalExecutionOptions::default(),
            )
            .unwrap();
        runtime.append_bar(row(0, 11.0)).unwrap();
        assert_eq!(runtime.result().plots[0].values[0].as_f64(), Some(2048.0));
        assert_eq!(runtime.profile().strategy_max_passes_on_bar, 2048);
    }
}

#[test]
fn historical_tick_context_cannot_be_downgraded_or_admit_future_requested_observations() {
    let hir = compiled(
        "//@version=6\nstrategy(\"tick admission\",calc_on_every_history_tick=true,calc_on_order_fills=true)\nplot(close)",
    );
    let valid = pass(
        row(0, 11.0),
        0.0,
        true,
        ExternalBarState::Historical,
        ExternalEvaluationTrigger::HistoricalTick,
        true,
    );
    for trigger in [
        ExternalEvaluationTrigger::BarClose,
        ExternalEvaluationTrigger::OrderFill,
        ExternalEvaluationTrigger::RealtimeTick,
        ExternalEvaluationTrigger::ClosingFill,
    ] {
        let mut invalid = valid.clone();
        invalid.context.trigger = trigger;
        assert!(
            HistoricalRuntime::new(&hir)
                .with_external_schedule(
                    vec![frame(0, 0.0)],
                    vec![vec![invalid]],
                    ExternalExecutionOptions::default()
                )
                .is_err()
        );
    }
    let mut future = valid.clone();
    future.evaluation.event_time_ms = 0;
    future.evaluation.request_updates = Some(vec![ObservedRequestUpdate {
        symbol: "OTHER".to_owned(),
        timeframe: "1".to_owned(),
        bar: row(60000, 12.0),
        kind: BarUpdateKind::Confirmed,
        available_at_ms: 119999,
    }]);
    assert!(
        HistoricalRuntime::new(&hir)
            .with_external_schedule(
                vec![frame(0, 0.0)],
                vec![vec![future.clone()]],
                ExternalExecutionOptions::default()
            )
            .err()
            .unwrap()
            .message
            .contains("availability")
    );
    // The current historical chart close is visible even at the first vertex.
    future.evaluation.request_updates.as_mut().unwrap()[0].bar = row(0, 12.0);
    future.evaluation.request_updates.as_mut().unwrap()[0].available_at_ms = 59999;
    assert!(
        HistoricalRuntime::new(&hir)
            .with_external_schedule(
                vec![frame(0, 0.0)],
                vec![vec![future]],
                ExternalExecutionOptions::default()
            )
            .is_ok()
    );
    let mut invalid = valid.clone();
    invalid.context.is_new = false;
    assert!(
        HistoricalRuntime::new(&hir)
            .with_external_schedule(
                vec![frame(0, 0.0)],
                vec![vec![invalid]],
                ExternalExecutionOptions::default()
            )
            .is_err()
    );
    let plain = compiled("//@version=6\nstrategy(\"no history ticks\")\nplot(close)");
    assert!(
        HistoricalRuntime::new(&plain)
            .with_external_schedule(
                vec![frame(0, 0.0)],
                vec![vec![valid]],
                ExternalExecutionOptions::default()
            )
            .is_err()
    );
}

#[test]
fn scheduled_append_is_equivalent_and_invalid_feedback_does_not_change_runtime() {
    let hir = compiled(
        "strategy(\"append\",calc_on_every_tick=true)\nplot(close)\nplot(barstate.islast?1:0)\nstrategy.entry(\"L\",strategy.long,qty=1)",
    );
    let bars = [row(0, 10.0), row(60000, 12.0)];
    let frames = vec![frame(0, 0.0), frame(60000, 1.0)];
    let groups = vec![
        vec![pass(
            bars[0],
            0.0,
            true,
            ExternalBarState::Realtime,
            ExternalEvaluationTrigger::BarClose,
            true,
        )],
        vec![pass(
            bars[1],
            1.0,
            true,
            ExternalBarState::Realtime,
            ExternalEvaluationTrigger::BarClose,
            true,
        )],
    ];
    let mut full = HistoricalRuntime::new(&hir)
        .with_external_schedule(
            frames.clone(),
            groups.clone(),
            ExternalExecutionOptions::default(),
        )
        .unwrap();
    full.append_bars(&bars).unwrap();
    let mut fast = HistoricalRuntime::new(&hir)
        .with_external_schedule(vec![], vec![], ExternalExecutionOptions::default())
        .unwrap();
    fast.append_external_schedule(&bars[..1], frames[..1].to_vec(), groups[..1].to_vec(), None)
        .unwrap();
    let before = fast.result();
    let intents = serde_json::to_value(fast.external_intents().unwrap()).unwrap();
    let mut bad = groups[1..].to_vec();
    bad[0][0].context.trigger = ExternalEvaluationTrigger::OrderFill;
    assert!(
        fast.append_external_schedule(&bars[1..], frames[1..].to_vec(), bad, None)
            .is_err()
    );
    assert_eq!(fast.result(), before);
    assert_eq!(
        serde_json::to_value(fast.external_intents().unwrap()).unwrap(),
        intents
    );
    fast.append_external_schedule(&bars[1..], frames[1..].to_vec(), groups[1..].to_vec(), None)
        .unwrap();
    assert_eq!(fast.result(), full.result());
    assert_eq!(
        serde_json::to_value(fast.external_intents().unwrap()).unwrap(),
        serde_json::to_value(full.external_intents().unwrap()).unwrap()
    );
}

#[test]
fn scheduled_context_cannot_smuggle_modes_or_add_unbounded_passes() {
    let hir = compiled("strategy(\"validation\",calc_on_order_fills=true)\nplot(close)");
    let base = pass(
        row(0, 12.0),
        0.0,
        true,
        ExternalBarState::Historical,
        ExternalEvaluationTrigger::BarClose,
        true,
    );
    let mut invalid = Vec::new();
    let mut bad = base.clone();
    bad.context.is_new = false;
    invalid.push(vec![bad]);
    let mut bad = base.clone();
    bad.evaluation.confirmed = false;
    invalid.push(vec![bad]);
    let mut bad = base.clone();
    bad.context.trigger = ExternalEvaluationTrigger::RealtimeTick;
    invalid.push(vec![bad]);
    let mut bad = base.clone();
    bad.context.trigger = ExternalEvaluationTrigger::ClosingFill;
    invalid.push(vec![bad]);
    invalid.push(vec![base.clone(); 66]);
    invalid.push(vec![]);
    for group in invalid {
        assert!(
            HistoricalRuntime::new(&hir)
                .with_external_schedule(
                    vec![frame(0, 0.0)],
                    vec![group],
                    ExternalExecutionOptions::default()
                )
                .is_err()
        );
    }
}

#[test]
fn scheduled_closing_fills_keep_immutable_bar_and_clock_guards() {
    let hir = compiled(
        "strategy(\"close\",calc_on_order_fills=true,process_orders_on_close=true)\nvarip int count=0\ncount+=1\nplot(count)",
    );
    let group = vec![
        pass(
            row(0, 12.0),
            0.0,
            true,
            ExternalBarState::Historical,
            ExternalEvaluationTrigger::BarClose,
            true,
        ),
        pass(
            row(0, 12.0),
            2.0,
            true,
            ExternalBarState::Historical,
            ExternalEvaluationTrigger::ClosingFill,
            true,
        ),
    ];
    let options = ExternalExecutionOptions {
        closing_ticks: true,
        risk_rules: false,
        ..Default::default()
    };
    let mut valid = HistoricalRuntime::new(&hir)
        .with_external_schedule(vec![frame(0, 0.0)], vec![group.clone()], options)
        .unwrap();
    valid.append_bar(row(0, 12.0)).unwrap();
    assert_eq!(valid.result().plots[0].values, vec![PineValue::Int(2)]);
    let mut bad = group;
    bad[1].evaluation.bar.close = 11.0;
    assert!(
        HistoricalRuntime::new(&hir)
            .with_external_schedule(vec![frame(0, 0.0)], vec![bad], options)
            .is_err()
    );
}

#[test]
fn scheduled_realtime_close_alert_waits_for_commit() {
    let hir = compiled(
        "strategy(\"alerts\",calc_on_every_tick=true)\nalert(str.tostring(close),alert.freq_once_per_bar_close)",
    );
    let mut runtime = HistoricalRuntime::new(&hir)
        .with_external_schedule(
            vec![frame(0, 0.0)],
            vec![vec![
                pass(
                    row(0, 10.0),
                    0.0,
                    false,
                    ExternalBarState::Realtime,
                    ExternalEvaluationTrigger::RealtimeTick,
                    true,
                ),
                pass(
                    row(0, 12.0),
                    0.0,
                    true,
                    ExternalBarState::Realtime,
                    ExternalEvaluationTrigger::BarClose,
                    false,
                ),
            ]],
            ExternalExecutionOptions::default(),
        )
        .unwrap();
    runtime.append_bar(row(0, 12.0)).unwrap();
    assert_eq!(runtime.result().alerts.len(), 1);
    assert_eq!(runtime.result().alerts[0].message, "12");
}

#[test]
fn scheduled_calculation_window_crops_context_with_market_feedback() {
    let hir = compiled(
        "strategy(\"window\",calc_on_every_tick=true,calc_bars_count=1)\nplot(barstate.ishistory?1:0)",
    );
    let bars = [row(0, 10.0), row(60000, 12.0)];
    let groups = vec![
        vec![pass(
            bars[0],
            0.0,
            true,
            ExternalBarState::Historical,
            ExternalEvaluationTrigger::BarClose,
            true,
        )],
        vec![pass(
            bars[1],
            0.0,
            true,
            ExternalBarState::Realtime,
            ExternalEvaluationTrigger::BarClose,
            true,
        )],
    ];
    let mut runtime = HistoricalRuntime::new(&hir)
        .with_external_schedule(
            vec![frame(0, 0.0), frame(60000, 0.0)],
            groups,
            ExternalExecutionOptions::default(),
        )
        .unwrap();
    runtime.append_bars(&bars).unwrap();
    assert_eq!(runtime.result().plots[0].values, vec![PineValue::Int(0)]);
}

#[test]
fn scheduled_history_boundary_precedes_realtime_and_cannot_move_backwards() {
    let hir = compiled(
        "strategy(\"boundary\")\nplot(barstate.islastconfirmedhistory?1:0)\nplot(barstate.islast?1:0)",
    );
    let bars = [row(0, 12.0), row(60000, 12.0)];
    let historical = vec![pass(
        bars[0],
        0.0,
        true,
        ExternalBarState::Historical,
        ExternalEvaluationTrigger::BarClose,
        true,
    )];
    let realtime = vec![pass(
        bars[1],
        0.0,
        true,
        ExternalBarState::Realtime,
        ExternalEvaluationTrigger::BarClose,
        true,
    )];
    let frames = vec![frame(0, 0.0), frame(60000, 0.0)];
    let mut runtime = HistoricalRuntime::new(&hir)
        .with_external_schedule(
            frames.clone(),
            vec![historical.clone(), realtime.clone()],
            ExternalExecutionOptions::default(),
        )
        .unwrap();
    runtime.append_bars(&bars).unwrap();
    assert_eq!(
        runtime.result().plots[0].values,
        vec![PineValue::Int(1), PineValue::Int(0)]
    );
    assert_eq!(
        runtime.result().plots[1].values,
        vec![PineValue::Int(0), PineValue::Int(1)]
    );
    let mut backwards_groups = vec![historical.clone(), realtime.clone()];
    backwards_groups[0][0].context.bar_state = ExternalBarState::Realtime;
    backwards_groups[1][0].context.bar_state = ExternalBarState::Historical;
    assert!(
        HistoricalRuntime::new(&hir)
            .with_external_schedule(
                frames,
                backwards_groups,
                ExternalExecutionOptions::default()
            )
            .is_err()
    );
    let mut streaming = HistoricalRuntime::new(&hir)
        .with_external_schedule(vec![], vec![], ExternalExecutionOptions::default())
        .unwrap();
    streaming
        .append_external_schedule(&bars[1..], vec![frame(60000, 0.0)], vec![realtime], None)
        .unwrap();
    let before = streaming.result();
    let mut backwards = historical;
    backwards[0].evaluation.bar.time = 120000;
    backwards[0].evaluation.account.time = 120000;
    backwards[0].evaluation.event_time_ms = 120012;
    assert!(
        streaming
            .append_external_schedule(
                &[row(120000, 12.0)],
                vec![frame(120000, 0.0)],
                vec![backwards],
                None
            )
            .is_err()
    );
    assert_eq!(streaming.result(), before);
}

#[test]
fn scheduled_passes_rollback_heap_and_drawing_mutations() {
    let hir = compiled(
        "strategy(\"heap\",calc_on_every_tick=true)\nvar a=array.new_int()\nvarip b=array.new_int()\narray.push(a,1)\narray.push(b,1)\nline.new(bar_index,close,bar_index+1,close)\nplot(array.size(a))\nplot(array.size(b))\nplot(array.size(line.all))",
    );
    let bars = [row(0, 12.0), row(60000, 12.0)];
    let groups = bars
        .iter()
        .map(|bar| {
            [10.0, 11.0, 12.0]
                .iter()
                .enumerate()
                .map(|(index, close)| {
                    pass(
                        row(bar.time, *close),
                        0.0,
                        index == 2,
                        ExternalBarState::Realtime,
                        if index == 2 {
                            ExternalEvaluationTrigger::BarClose
                        } else {
                            ExternalEvaluationTrigger::RealtimeTick
                        },
                        index == 0,
                    )
                })
                .collect()
        })
        .collect();
    let mut runtime = HistoricalRuntime::new(&hir)
        .with_external_schedule(
            vec![frame(0, 0.0), frame(60000, 0.0)],
            groups,
            ExternalExecutionOptions::default(),
        )
        .unwrap();
    runtime.append_bars(&bars).unwrap();
    let result = runtime.result();
    for (plot, expected) in result.plots.iter().zip([[1, 2], [3, 6], [1, 2]]) {
        assert_eq!(plot.values, expected.map(PineValue::Int).to_vec());
    }
    assert_eq!(result.lines.len(), 2);
}
