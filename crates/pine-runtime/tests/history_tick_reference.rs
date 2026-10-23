//! Standard-chart observations captured from TradingView on 2026-10-22.
//! This bounded fixture does not qualify other chart types or realtime feeds.
use pine_runtime::{Bar, HistoricalRuntime};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

fn capture_orders(body: &str, fills: bool, closing: bool) -> pine_runtime::RuntimeResult {
    let mut source = format!(
        r#"//@version=6
strategy("Historical order observations", calc_on_every_history_tick=true, calc_on_order_fills={fills}, process_orders_on_close={closing}, initial_capital=1000000000)
varip int observedTime = na
varip int perbar = 0
varip array<float> samples = array.new_float(16, na)
if na(observedTime) or observedTime != time
    observedTime := time
    perbar := 0
perbar += 1
{body}
int offset = (perbar-1)*4
array.set(samples, offset, strategy.position_size)
array.set(samples, offset+1, strategy.position_avg_price)
array.set(samples, offset+2, strategy.netprofit)
array.set(samples, offset+3, strategy.openprofit)
"#
    );
    for i in 0..16 {
        source.push_str(&format!("plot(array.get(samples, {i}))\n"));
    }
    let analysis = analyze_source(&SourceFile::new("history-tick-orders.pine", source));
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let hir = analysis.hir.unwrap();
    let mut input = bars()[1..].to_vec();
    input.push(Bar {
        time: 1792636800000,
        open: 82928.29,
        high: 82928.30,
        low: 82815.43,
        close: 82846.20,
        volume: 17.99206946,
    });
    let mut runtime = HistoricalRuntime::new(&hir);
    for bar in input {
        runtime.append_bar(bar).unwrap();
    }
    runtime.result()
}

fn assert_observation(
    result: &pine_runtime::RuntimeResult,
    bar: usize,
    tick: usize,
    expected: [Option<f64>; 4],
) {
    for (field, expected) in expected.into_iter().enumerate() {
        let actual = result.plots[tick * 4 + field].values[bar].as_f64();
        match (actual, expected) {
            (None, None) => {}
            (Some(a), Some(b)) if (a - b).abs() < 1e-8 => {}
            _ => panic!("bar {bar} tick {tick} field {field}: {actual:?} != {expected:?}"),
        }
    }
}

#[test]
fn native_history_market_fills_match_actual_vertex_observations_without_duplicate_callbacks() {
    let body = "if perbar == 1\n    strategy.entry(\"L\", strategy.long, qty=1)\nif perbar == 2 and strategy.position_size > 0\n    strategy.close(\"L\")";
    // The true case is directly observed; false is a deduplication control.
    for fills in [true, false] {
        let result = capture_orders(body, fills, false);
        assert_observation(&result, 0, 0, [Some(0.0), None, Some(0.0), Some(0.0)]);
        assert_observation(
            &result,
            0,
            1,
            [Some(1.0), Some(82951.77), Some(0.0), Some(1.79)],
        );
        for tick in [2, 3] {
            assert_observation(&result, 0, tick, [Some(0.0), None, Some(1.79), Some(0.0)]);
        }
        assert_observation(&result, 1, 0, [Some(0.0), None, Some(1.79), Some(0.0)]);
        assert_observation(
            &result,
            1,
            1,
            [Some(1.0), Some(82866.33), Some(1.79), Some(-78.77)],
        );
        for tick in [2, 3] {
            assert_observation(&result, 1, tick, [Some(0.0), None, Some(-76.98), Some(0.0)]);
        }
    }
}

#[test]
fn native_history_limit_fill_uses_continuous_crossing_and_next_vertex_feedback() {
    let result = capture_orders(
        "if perbar == 1\n    strategy.entry(\"L\", strategy.long, qty=1, limit=open-0.02)\nif strategy.position_size > 0\n    strategy.close(\"L\")",
        true,
        false,
    );
    for tick in [0, 1] {
        assert_observation(&result, 0, tick, [Some(0.0), None, Some(0.0), Some(0.0)]);
    }
    assert_observation(
        &result,
        0,
        2,
        [Some(1.0), Some(82951.75), Some(0.0), Some(-246.70)],
    );
    assert_observation(&result, 0, 3, [Some(0.0), None, Some(-246.70), Some(0.0)]);
    assert_observation(
        &result,
        1,
        1,
        [Some(1.0), Some(82866.31), Some(-246.70), Some(-78.75)],
    );
    assert_observation(&result, 1, 2, [Some(0.0), None, Some(-325.45), Some(0.0)]);
}

#[test]
fn final_vertex_order_waits_for_next_open_or_explicit_closing_price() {
    let body = "if perbar == 4\n    strategy.entry(\"L\", strategy.long, qty=1)\nif perbar == 1 and strategy.position_size > 0\n    strategy.close(\"L\")";
    for closing in [false, true] {
        let result = capture_orders(body, true, closing);
        assert_observation(
            &result,
            1,
            0,
            [Some(1.0), Some(82866.33), Some(0.0), Some(0.0)],
        );
        assert_observation(&result, 1, 1, [Some(0.0), None, Some(0.0), Some(0.0)]);
        let entry = if closing { 82925.92 } else { 82928.29 };
        let profit = if closing { 2.37 } else { 0.0 };
        assert_observation(
            &result,
            2,
            0,
            [Some(1.0), Some(entry), Some(0.0), Some(profit)],
        );
        assert_observation(&result, 2, 1, [Some(0.0), None, Some(profit), Some(0.0)]);
    }
}

#[test]
fn history_tick_declaration_is_v6_named_const_bool_and_reported_to_hosts() {
    for source in [
        "//@version=5\nstrategy(\"v5\",calc_on_every_history_tick=true)",
        "//@version=6\nstrategy(\"type\",calc_on_every_history_tick=close>open)",
        "//@version=6\nstrategy(\"input\",calc_on_every_history_tick=input.bool(true))",
        "//@version=6\nindicator(\"indicator\",calc_on_every_history_tick=true)",
    ] {
        let analysis = analyze_source(&SourceFile::new("invalid-history-tick.pine", source));
        assert!(!analysis.diagnostics.is_empty(), "accepted {source}");
    }
    let analysis = analyze_source(&SourceFile::new(
        "host-history-tick.pine",
        "//@version=6\nstrategy(\"history\",calc_on_every_history_tick=true)",
    ));
    let hir = analysis.hir.unwrap();
    assert!(
        pine_runtime::host_requirements(&hir)
            .execution
            .calc_on_every_history_tick
    );
}

#[test]
fn tick_price_flow_and_ta_commit_once_and_cloned_continuation_matches_batch() {
    let source = SourceFile::new(
        "history-tick-state.pine",
        r#"//@version=6
strategy("Historical tick state",calc_on_every_history_tick=true)
var float ordinary=0.0
ordinary += close
varip array<float> persistent=array.new_float(1,0)
array.set(persistent,0,array.get(persistent,0)+1)
plot(ordinary)
plot(array.get(persistent,0))
plot(ta.cum(volume))
plot(ta.sma(close,2))
plot(ta.obv)
plot(ta.pvt)
plot(close[1])
"#,
    );
    let hir = analyze_source(&source).hir.unwrap();
    let input = bars();
    let mut batch = HistoricalRuntime::new(&hir);
    for bar in &input {
        batch.append_bar(*bar).unwrap();
    }
    let mut prefix = HistoricalRuntime::new(&hir);
    prefix.append_bar(input[0]).unwrap();
    let mut restored = prefix.clone();
    for bar in &input[1..] {
        restored.append_bar(*bar).unwrap();
    }
    assert_eq!(
        pine_runtime::public_runtime_result_json(&batch.result()),
        pine_runtime::public_runtime_result_json(&restored.result())
    );
    let plain = SourceFile::new(
        "single-pass-state.pine",
        source.text().replace(
            "calc_on_every_history_tick=true",
            "calc_on_every_history_tick=false",
        ),
    );
    let baseline_hir = analyze_source(&plain).hir.unwrap();
    let mut baseline = HistoricalRuntime::new(&baseline_hir);
    for bar in &input {
        baseline.append_bar(*bar).unwrap();
    }
    for i in [0, 2, 3, 4, 5, 6] {
        assert_eq!(
            batch.result().plots[i].values,
            baseline.result().plots[i].values,
            "plot {i}"
        );
    }
}

#[test]
fn same_context_request_keeps_canonical_history_while_chart_observations_develop() {
    let mut source = String::from(
        r#"//@version=6
strategy("Canonical history request",calc_on_every_history_tick=true)
varip int seen=na
varip int pass=0
varip array<float> samples=array.new_float(12,na)
if na(seen) or seen != time
    seen := time
    pass := 0
requested=request.security(syminfo.tickerid,timeframe.period,close)
average=request.security(syminfo.tickerid,timeframe.period,ta.sma(close,2))
array.set(samples,pass*3,close)
array.set(samples,pass*3+1,requested)
array.set(samples,pass*3+2,average)
pass+=1
"#,
    );
    for i in 0..12 {
        source.push_str(&format!("plot(array.get(samples,{i}))\n"));
    }
    let analysis = analyze_source(&SourceFile::new("canonical-history-request.pine", source));
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let hir = analysis.hir.unwrap();
    let mut runtime = HistoricalRuntime::new(&hir);
    let input = bars();
    for bar in &input {
        runtime.append_bar(*bar).unwrap();
    }
    let result = runtime.result();
    assert_eq!(
        pine_runtime::public_runtime_result_json(&result),
        pine_runtime::public_runtime_result_json(
            &pine_runtime::run_historical(&hir, &input).unwrap()
        )
    );
    for (i, bar) in input.iter().enumerate() {
        for pass in 0..4 {
            assert_eq!(
                result.plots[pass * 3 + 1].values[i].as_f64(),
                Some(bar.close)
            );
            if i > 0 {
                assert_eq!(
                    result.plots[pass * 3 + 2].values[i].as_f64(),
                    Some((bar.close + input[i - 1].close) / 2.0)
                );
            }
        }
    }
    assert_eq!(result.plots[0].values[0].as_f64(), Some(input[0].open));
}

#[test]
fn supplied_magnifier_observations_accumulate_chart_extremes_and_physical_volume() {
    let mut source = String::from(
        r#"//@version=6
strategy("History magnifier",calc_on_every_history_tick=true,use_bar_magnifier=true)
varip int n=0
var int ordinary=0
varip array<float> samples=array.new_float(24,na)
array.set(samples,n*3,high)
array.set(samples,n*3+1,low)
array.set(samples,n*3+2,volume)
n+=1
ordinary+=1
plot(n)
plot(ordinary)
"#,
    );
    for i in 0..24 {
        source.push_str(&format!("plot(array.get(samples,{i}))\n"));
    }
    let hir = analyze_source(&SourceFile::new("history-magnifier.pine", source))
        .hir
        .unwrap();
    let hosts = vec![
        Bar {
            time: 0,
            open: 100.0,
            high: 110.0,
            low: 90.0,
            close: 105.0,
            volume: 50.0,
        },
        Bar {
            time: 30000,
            open: 105.0,
            high: 120.0,
            low: 95.0,
            close: 115.0,
            volume: 80.0,
        },
    ];
    let magnifier =
        pine_runtime::magnifier_input_from_groups(vec![pine_runtime::MagnifierChartBarInput {
            chart_bar_index: 0,
            bars: hosts,
        }])
        .unwrap();
    let mut runtime = HistoricalRuntime::new(&hir).with_magnifier_input(magnifier);
    runtime.prepare_magnifier_chart_bar_count(1).unwrap();
    runtime
        .append_bar(Bar {
            time: 0,
            open: 100.0,
            high: 120.0,
            low: 90.0,
            close: 115.0,
            volume: 130.0,
        })
        .unwrap();
    let result = runtime.result();
    assert_eq!(result.plots[0].values[0].as_f64(), Some(8.0));
    assert_eq!(result.plots[1].values[0].as_f64(), Some(1.0));
    for (tick, expected) in [
        [100.0, 100.0, 12.5],
        [100.0, 90.0, 25.0],
        [110.0, 90.0, 37.5],
        [110.0, 90.0, 50.0],
        [110.0, 90.0, 70.0],
        [110.0, 90.0, 90.0],
        [120.0, 90.0, 110.0],
        [120.0, 90.0, 130.0],
    ]
    .into_iter()
    .enumerate()
    {
        for (field, value) in expected.into_iter().enumerate() {
            assert_eq!(
                result.plots[2 + tick * 3 + field].values[0].as_f64(),
                Some(value)
            );
        }
    }
}

#[test]
fn history_tick_flag_does_not_enable_realtime_ticks_or_double_count_price_flow() {
    let input = bars();
    let mut results = Vec::new();
    for enabled in [true, false] {
        let source = SourceFile::new(
            "history-realtime-boundary.pine",
            format!(
                "//@version=6\nstrategy(\"History realtime boundary\",calc_on_every_history_tick={enabled})\nvar float total=0\ntotal+=close\nplot(total)\nplot(ta.obv)\nplot(ta.pvt)\nplot(ta.cum(volume))\n"
            ),
        );
        let hir = analyze_source(&source).hir.unwrap();
        let mut runtime = pine_runtime::RealtimeRuntime::new(&hir);
        runtime.seed_historical(&input[..2]).unwrap();
        let mut forming = input[2];
        forming.close = forming.open;
        forming.high = forming.open;
        forming.low = forming.open;
        forming.volume = 1.0;
        runtime
            .update(pine_runtime::BarUpdate::forming(forming))
            .unwrap();
        results.push(pine_runtime::public_runtime_result_json(
            &runtime
                .update(pine_runtime::BarUpdate::confirmed(input[2]))
                .unwrap(),
        ));
    }
    assert_eq!(results[0], results[1]);
}

fn bars() -> Vec<Bar> {
    [
        (
            1792635000000,
            83355.83,
            83368.21,
            82853.92,
            82944.97,
            126.40293883,
        ),
        (
            1792635600000,
            82951.77,
            82953.56,
            82705.05,
            82866.33,
            109.19643489,
        ),
        (
            1792636200000,
            82866.33,
            82982.97,
            82787.56,
            82925.92,
            74.21653215,
        ),
    ]
    .into_iter()
    .map(|(time, open, high, low, close, volume)| Bar {
        time,
        open,
        high,
        low,
        close,
        volume,
    })
    .collect()
}

#[test]
fn history_ticks_match_observed_progressive_prices_volume_and_rollback() {
    let source = r#"//@version=6
strategy("History tick reference", calc_on_every_history_tick=true)
var int ordinary = 0
varip int retained = 0
varip int observedTime = na
varip int perbar = 0
varip array<float> samples = array.new_float(28, na)
if na(observedTime) or observedTime != time
    observedTime := time
    perbar := 0
ordinary += 1
retained += 1
int offset = perbar * 7
array.set(samples, offset, open)
array.set(samples, offset+1, high)
array.set(samples, offset+2, low)
array.set(samples, offset+3, close)
array.set(samples, offset+4, volume)
array.set(samples, offset+5, ordinary)
array.set(samples, offset+6, barstate.ishistory and barstate.isconfirmed and barstate.isnew and not barstate.isrealtime ? 1 : 0)
perbar += 1
plot(ordinary)
plot(retained)
"#;
    let mut source = source.to_owned();
    for i in 0..28 {
        source.push_str(&format!("plot(array.get(samples, {i}))\n"));
    }
    let analysis = analyze_source(&SourceFile::new("history-tick-reference.pine", source));
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let hir = analysis.hir.unwrap();
    let bars = bars();
    let mut runtime = HistoricalRuntime::new(&hir);
    for bar in &bars {
        runtime.append_bar(*bar).unwrap();
    }
    let result = runtime.result();
    for (i, bar) in bars.iter().enumerate() {
        assert_eq!(result.plots[0].values[i].as_f64(), Some((i + 1) as f64));
        assert_eq!(
            result.plots[1].values[i].as_f64(),
            Some(((i + 1) * 4) as f64)
        );
        let middle = if i == 2 {
            [bar.low, bar.high]
        } else {
            [bar.high, bar.low]
        };
        let mut high = bar.open;
        let mut low = bar.open;
        for (tick, close) in [bar.open, middle[0], middle[1], bar.close]
            .into_iter()
            .enumerate()
        {
            high = high.max(close);
            low = low.min(close);
            let expected = [
                bar.open,
                high,
                low,
                close,
                bar.volume * (tick + 1) as f64 / 4.0,
                (i + 1) as f64,
                1.0,
            ];
            for (field, expected) in expected.into_iter().enumerate() {
                let actual = result.plots[2 + tick * 7 + field].values[i]
                    .as_f64()
                    .unwrap();
                assert!(
                    (actual - expected).abs() < 1e-8,
                    "bar {i} tick {tick} field {field}: {actual} != {expected}"
                );
            }
        }
    }
}
