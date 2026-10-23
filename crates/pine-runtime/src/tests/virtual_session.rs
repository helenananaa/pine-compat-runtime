use super::calendar_clock::{at, days, environment, millis, program};
use super::*;

fn check(timeframe: &str, time: i64, body: &str, expected: &[i64]) {
    let result =
        HistoricalRuntime::with_request_environment(&program(body), environment(timeframe))
            .run(&[at(time)])
            .unwrap();
    let actual: Vec<_> = result.plots.iter().map(|p| p.values[0].clone()).collect();
    assert_eq!(
        actual,
        expected
            .iter()
            .copied()
            .map(PineValue::Int)
            .collect::<Vec<_>>()
    );
}

#[test]
fn virtual_calendar_original_native_session_offset_gap_is_repaired() {
    check(
        "D",
        days()[1].0,
        r#"plot(time("1","0930-0945","America/New_York",timeframe_bars_back=1)-time)
plot(time_close("1","0930-0945","America/New_York",timeframe_bars_back=1)-time)
plot(time("1","1500-1600","America/New_York",timeframe_bars_back=1)-time)
plot(time_close("1","1500-1600","America/New_York",timeframe_bars_back=1)-time)"#,
        &[-86400000, -85500000, -66600000, -63000000],
    );
}

#[test]
fn virtual_calendar_native_daily_grid_and_default_weekend_controls() {
    check(
        "D",
        days()[1].0,
        r#"plot(dayofmonth)
plot(na(time("1","1500-1600","America/New_York"))?999:0)
plot(na(time_close("1","1500-1600","America/New_York"))?999:0)
plot(time("1","0930-0945","America/New_York",timeframe_bars_back=-1)-time)
plot(time_close("1","0930-0945","America/New_York",timeframe_bars_back=-1)-time)
plot(time("1","0930-0945","America/New_York",timeframe_bars_back=16)-time)
plot(time("1","0930-0945","America/New_York",timeframe_bars_back=-16)-time)
plot(time("1","1500-1600","America/New_York",timeframe_bars_back=-1)-time)
plot(time_close("1","1500-1600","America/New_York",timeframe_bars_back=-1)-time)
plot(time("1","24x7","America/New_York")-time)
plot(time_close("1","24x7","America/New_York")-time)
plot(time("1","0930-0945:23456","America/New_York",timeframe_bars_back=-16)-time)"#,
        &[
            9, 999, 999, 0, 900000, -172800000, 86400000, 19800000, 23400000, -34200000, 52200000,
            259200000,
        ],
    );
}

#[test]
fn virtual_calendar_native_intraday_phase_short_bar_and_split_controls() {
    check(
        "60",
        millis("2026-10-09T19:30:00Z"),
        r#"plot(hour*100+minute)
plot(time("60","1500-1600","America/New_York")-time)
plot(time_close("60","1500-1600","America/New_York")-time)
plot(time("30","1515-1600","America/New_York")-time)
plot(time_close("30","1515-1600","America/New_York")-time)
plot(time("60","1500-1600","America/New_York",timeframe_bars_back=1)-time)
plot(time("30","1500-1600","America/New_York",timeframe_bars_back=1)-time)
plot(time_close("30","1500-1600","America/New_York",timeframe_bars_back=-1)-time)
plot(time("60","0900-1000,1500-1600","America/New_York",timeframe_bars_back=1)-time)"#,
        &[
            1530, -1800000, 1800000, -900000, 900000, -88200000, -1800000, 86400000, -23400000,
        ],
    );
}

#[test]
fn virtual_calendar_native_daily_weekly_monthly_aggregate_controls() {
    check(
        "D",
        days()[1].0,
        r#"plot(time("D","0930-0945","America/New_York",timeframe_bars_back=1)-time)
plot(time_close("D","0930-0945","America/New_York",timeframe_bars_back=1)-time)
plot(time("D","0900-1000,1500-1600","America/New_York")-time)
plot(time_close("D","0900-1000,1500-1600","America/New_York")-time)
plot(time("W","0930-0945:23456","America/New_York")-time)
plot(time_close("W","0930-0945:23456","America/New_York")-time)
plot(time("M","0930-0945:23456","America/New_York")-time)
plot(time_close("M","0930-0945:23456","America/New_York")-time)"#,
        &[
            -86400000, -85500000, -1800000, 23400000, -345600000, 900000, -691200000, 1815300000,
        ],
    );
}

#[test]
fn virtual_calendar_native_overnight_full_day_and_combined_offset_controls() {
    check(
        "D",
        days()[1].0,
        r#"plot(time("1","0900-1000,1500-1600","America/New_York")-time)
plot(time_close("1","0900-1000,1500-1600","America/New_York")-time)
plot(time("1","0900-1000,1500-1600","America/New_York",timeframe_bars_back=31)-time)
plot(time_close("1","0900-1000,1500-1600","America/New_York",timeframe_bars_back=31)-time)
plot(time("1","1800-1000:23456","America/New_York")-time)
plot(time_close("1","1800-1000:23456","America/New_York")-time)
plot(time("1","1700-1700:23456","America/New_York")-time)
plot(time_close("1","1700-1700:23456","America/New_York")-time)
plot(time("1","0930-0945","America/New_York",bars_back=1,timeframe_bars_back=1)-time)"#,
        &[
            -1800000, 1800000, -66600000, -63000000, -55800000, 1800000, -59400000, 27000000,
            -172800000,
        ],
    );
}

#[test]
fn virtual_calendar_native_outside_session_directional_offset_controls() {
    check(
        "60",
        millis("2026-10-09T19:30:00Z"),
        r#"plot(time("1","0930-0945","America/New_York",timeframe_bars_back=1)-time)
plot(time("1","0930-0945","America/New_York",timeframe_bars_back=-1)-time)
plot(time("60","0930-0945","America/New_York",timeframe_bars_back=1)-time)
plot(time("60","0930-0945","America/New_York",timeframe_bars_back=-1)-time)
plot(time("60","1600-1700","America/New_York",timeframe_bars_back=1)-time)
plot(time("60","1600-1700","America/New_York",timeframe_bars_back=-1)-time)
plot(time("1","1600-1700","America/New_York",timeframe_bars_back=-1)-time)"#,
        &[
            -20820000, 64860000, -108000000, 151200000, -171000000, 88200000, 1860000,
        ],
    );
}

#[test]
fn virtual_calendar_native_daily_chart_full_week_and_month_controls() {
    check(
        "D",
        days()[1].0,
        r#"plot(time("W","1800-1000:23456","America/New_York")-time)
plot(time_close("W","1800-1000:23456","America/New_York")-time)
plot(time("W","0000-0000:23456","America/New_York")-time)
plot(time_close("W","0000-0000:23456","America/New_York")-time)
plot(time("M","1700-1700:23456","America/New_York")-time)
plot(time_close("M","1700-1700:23456","America/New_York")-time)
plot(time("W","0900-1000,1500-1600:23456","America/New_York")-time)
plot(time_close("W","0900-1000,1500-1600:23456","America/New_York")-time)"#,
        &[
            -401400000, 1800000, -379800000, 52200000, -750600000, 1841400000, -347400000, 23400000,
        ],
    );
}

#[test]
fn virtual_calendar_preparation_work_and_extreme_offset_are_bounded() {
    let source = program(r#"plot(time("1","0930-0945",timeframe_bars_back=9223372036854775807))"#);
    let err = HistoricalRuntime::with_request_environment(&source, environment("D"))
        .run(&[at(days()[1].0)])
        .unwrap_err();
    assert!(err.message.contains("out of range"), "{}", err.message);
    let source = program(r#"plot(time("1","0930-0945",timeframe_bars_back=100000))"#);
    let err = HistoricalRuntime::with_request_environment(&source, environment("D"))
        .with_execution_limits(ExecutionLimits {
            max_steps_per_bar: 100,
            max_loop_iterations_per_bar: 1_000_000,
        })
        .run(&[at(days()[1].0)])
        .unwrap_err();
    assert!(
        err.message.contains("E_EXECUTION_BUDGET"),
        "{}",
        err.message
    );
}

#[test]
fn virtual_calendar_native_intraday_week_month_closes_and_gap_offsets() {
    check(
        "60",
        millis("2026-10-09T19:30:00Z"),
        r#"plot(time("W","1700-1700:23456","America/New_York")-time)
plot(time_close("W","1700-1700:23456","America/New_York")-time)
plot(time("W","0930-0945:23456","America/New_York")-time)
plot(time_close("W","0930-0945:23456","America/New_York")-time)
plot(time("M","0930-0945:23456","America/New_York")-time)
plot(time_close("M","0930-0945:23456","America/New_York")-time)
plot(time("W","0930-0945:23456","America/New_York",timeframe_bars_back=1)-time)
plot(time_close("W","0930-0945:23456","America/New_York",timeframe_bars_back=-1)-time)
plot(time("W","0930-1600:23456","America/New_York",timeframe_bars_back=1)-time)
plot(time_close("W","0930-1600:23456","America/New_York",timeframe_bars_back=-1)-time)
plot(time("W","0930-0945:23456","America/New_York",timeframe_bars_back=2)-time)
plot(time_close("W","0930-0945:23456","America/New_York",timeframe_bars_back=-2)-time)
plot(time("W","24x7","America/New_York",timeframe_bars_back=1)-time)
plot(time_close("W","24x7","America/New_York",timeframe_bars_back=-1)-time)
plot(time("M","0930-1600:23456","America/New_York",timeframe_bars_back=1)-time)
plot(time_close("M","0930-1600:23456","America/New_York",timeframe_bars_back=-1)-time)"#,
        &[
            -426600000,
            178200000,
            -367200000,
            237600000,
            -712800000,
            2055600000,
            -367200000,
            1447200000,
            -972000000,
            842400000,
            -972000000,
            2055600000,
            -1006200000,
            808200000,
            -3304800000,
            4561200000,
        ],
    );
}

#[test]
fn virtual_calendar_native_daily_chart_week_month_offset_controls() {
    check(
        "D",
        days()[1].0,
        r#"plot(time("W","1700-1700:23456","America/New_York")-time)
plot(time_close("W","1700-1700:23456","America/New_York")-time)
plot(time("W","0930-0945:23456","America/New_York",timeframe_bars_back=1)-time)
plot(time_close("W","0930-0945:23456","America/New_York",timeframe_bars_back=-1)-time)"#,
        &[-405000000, 27000000, -950400000, 605700000],
    );
}

#[test]
fn virtual_calendar_daily_aggregate_includes_intraday_break() {
    check(
        "60",
        millis("2026-10-09T16:30:00Z"),
        r#"plot(time("D","0900-1000,1500-1600","America/New_York")-time)
plot(time_close("D","0900-1000,1500-1600","America/New_York")-time)"#,
        &[-12600000, 12600000],
    );
}

#[test]
fn virtual_session_native_dst_crossings_without_exchange_calendar() {
    let source = program(
        r#"plot(time("1","1700-1700","America/New_York")-time)
plot(time_close("1","1700-1700","America/New_York")-time)"#,
    );
    let env = RequestEnvironment::default().for_chart(ChartContext::new(
        "UTC:TEST",
        RequestTimeframe::parse("D").unwrap(),
    ));
    for (date, expected) in [
        ("2026-03-08T00:00:00Z", [-7200000, 75600000]),
        ("2025-11-02T00:00:00Z", [-10800000, 79200000]),
    ] {
        let result = HistoricalRuntime::with_request_environment(&source, env.clone())
            .run(&[at(millis(date))])
            .unwrap();
        assert_eq!(
            result
                .plots
                .iter()
                .map(|p| p.values[0].clone())
                .collect::<Vec<_>>(),
            expected.map(PineValue::Int)
        );
    }
}

#[test]
fn virtual_session_native_versioned_default_days_and_explicit_weekend_override() {
    for (version, expected) in [(4, 999), (5, 0), (6, 0)] {
        let source = pine_syntax::SourceFile::new(
            "days.pine",
            format!(
                "//@version={version}\n{}(\"days\")\nx=time(\"1\",\"0000-0015\",\"UTC\")\nplot(na(x)?999:x-time)\nplot(time(\"1\",\"0000-0015:1234567\",\"UTC\")-time)\n",
                if version == 4 { "study" } else { "indicator" }
            ),
        );
        let analysis = analyze_source(&source);
        assert!(
            analysis.diagnostics.is_empty(),
            "{:?}",
            analysis.diagnostics
        );
        let env = RequestEnvironment::default().for_chart(ChartContext::new(
            "UTC:TEST",
            RequestTimeframe::parse("D").unwrap(),
        ));
        let result = HistoricalRuntime::with_request_environment(&analysis.hir.unwrap(), env)
            .run(&[at(millis("2026-10-10T00:00:00Z"))])
            .unwrap();
        assert_eq!(result.plots[0].values, [PineValue::Int(expected)]);
        assert_eq!(result.plots[1].values, [PineValue::Int(0)]);
    }
}

#[test]
fn virtual_session_realtime_forming_confirmation_replay_and_failed_offset_are_atomic() {
    let source = program(
        r#"plot(time("30","1515-1600")-time)
plot(time_close("30","1515-1600")-time)
plot(time("1","0930-0945",bars_back=bar_index==1?100:0,timeframe_bars_back=1)-time)"#,
    );
    let env = environment("60");
    let value = at(millis("2026-10-09T19:30:00Z"));
    let mut runtime = RealtimeRuntime::with_request_environment(&source, env.clone());
    runtime.update(BarUpdate::forming(value)).unwrap();
    let result = runtime.update(BarUpdate::confirmed(value)).unwrap();
    assert_eq!(result.plots[0].values, [PineValue::Int(-900000)]);
    assert_eq!(result.plots[1].values, [PineValue::Int(900000)]);
    let revision = runtime.revision();
    let before = runtime.result();
    assert!(runtime.update(BarUpdate::forming(at(days()[2].0))).is_err());
    assert_eq!(runtime.revision(), revision);
    assert_eq!(runtime.result(), before);
    let batch = HistoricalRuntime::with_request_environment(&source, env)
        .run(&[value])
        .unwrap();
    assert_eq!(
        runtime.replay_historical(&[value]).unwrap().plots,
        batch.plots
    );
}
