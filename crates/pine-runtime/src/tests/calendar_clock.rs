use super::*;
use std::sync::Arc;

pub(super) fn millis(date: &str) -> i64 {
    chrono::DateTime::parse_from_rfc3339(date)
        .unwrap()
        .timestamp_millis()
}

fn tf(value: &str) -> RequestTimeframe {
    RequestTimeframe::parse(value).unwrap()
}

pub(super) fn days() -> [(i64, i64, i64); 4] {
    [
        (
            "2026-10-08T13:30:00Z",
            "2026-10-08T20:00:00Z",
            "2026-10-08T00:00:00Z",
        ),
        (
            "2026-10-09T13:30:00Z",
            "2026-10-09T20:00:00Z",
            "2026-10-09T00:00:00Z",
        ),
        (
            "2026-10-12T13:30:00Z",
            "2026-10-12T20:00:00Z",
            "2026-10-12T00:00:00Z",
        ),
        (
            "2026-10-13T13:30:00Z",
            "2026-10-13T20:00:00Z",
            "2026-10-13T00:00:00Z",
        ),
    ]
    .map(|(open, close, day)| (millis(open), millis(close), millis(day)))
}

pub(super) fn calendar(symbol: &str, timezone: &str, days: &[(i64, i64, i64)]) -> BarCalendar {
    let mut streams = vec![(
        tf("D"),
        days.iter()
            .map(|&(open, close, day)| BarTimeInterval::new(open, close, day))
            .collect(),
    )];
    for (name, duration) in [("1", 60_000), ("5", 300_000), ("60", 3_600_000)] {
        let intervals = days
            .iter()
            .flat_map(|&(open, close, day)| {
                (open..close)
                    .step_by(duration as usize)
                    .map(move |open| BarTimeInterval::new(open, (open + duration).min(close), day))
            })
            .collect();
        streams.push((tf(name), intervals));
    }
    BarCalendar::new(symbol, timezone, streams).unwrap()
}

pub(super) fn environment(timeframe: &str) -> RequestEnvironment {
    RequestEnvironment::default()
        .for_chart(ChartContext::new("NASDAQ:AAPL", tf(timeframe)))
        .with_bar_calendars(Arc::new(
            BarCalendarRegistry::new([calendar("NASDAQ:AAPL", "America/New_York", &days())])
                .unwrap(),
        ))
}

pub(super) fn at(time: i64) -> Bar {
    let mut value = bar(10.0);
    value.time = time;
    value
}

pub(super) fn program(body: &str) -> pine_ir::HirProgram {
    let analysis = analyze_source(&pine_syntax::SourceFile::new(
        "calendar.pine",
        format!("//@version=6\nindicator(\"calendar\",dynamic_requests=true)\n{body}\n"),
    ));
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    analysis.hir.unwrap()
}

#[test]
fn calendar_native_daily_boundary_and_exchange_timezone_controls() {
    let source = program(
        "plot(time_close-time)\nplot(time_close(\"1\")-time)\nplot(time_close(\"1\",timeframe_bars_back=1)-time)\nplot(time(\"1\",timeframe_bars_back=1)-time)\nplot(hour)\nplot(hour(time))\nplot(minute)\nplot(hour(time,\"UTC\"))\nplot(timestamp(year,month,dayofmonth,9,30)-time)\nplot(str.format_time(time,\"HH:mm\")==\"09:30\"?1:0)\nplot(syminfo.timezone==\"America/New_York\"?1:0)\nplot(time_close(\"D\",bars_back=1)-time)\nplot(time(\"D\",bars_back=-1)-time)",
    );
    let result = HistoricalRuntime::with_request_environment(&source, environment("D"))
        .run(&[at(days()[1].0)])
        .unwrap();
    for (plot, value) in result.plots.iter().zip([
        23_400_000,
        23_400_000,
        -63_000_000,
        -86_400_000,
        9,
        9,
        30,
        13,
        0,
        1,
        1,
        -63_000_000,
        259_200_000,
    ]) {
        assert_eq!(plot.values, [PineValue::Int(value)]);
    }
}

#[test]
fn calendar_requested_offsets_select_trading_intervals_then_daily_bounds() {
    let source = program(
        "plot(time(\"1\",timeframe_bars_back=1)-time)\nplot(time(\"1\",timeframe_bars_back=2)-time)\nplot(time(\"1\",timeframe_bars_back=-1)-time)\nplot(time_close(\"1\",timeframe_bars_back=-1)-time)\nplot(time_close(\"1\",timeframe_bars_back=2)-time)\nplot(time(\"60\",timeframe_bars_back=1)-time)\nplot(time_close(\"60\",timeframe_bars_back=1)-time)",
    );
    let result = HistoricalRuntime::with_request_environment(&source, environment("D"))
        .run(&[at(days()[1].0)])
        .unwrap();
    for (plot, value) in result.plots.iter().zip([
        -86_400_000,
        -86_400_000,
        0,
        23_400_000,
        -63_000_000,
        -86_400_000,
        -63_000_000,
    ]) {
        assert_eq!(plot.values, [PineValue::Int(value)]);
    }
}

#[test]
fn calendar_crosses_weekend_and_dst_and_clips_final_hour() {
    let days = [
        (
            "2024-03-08T14:30:00Z",
            "2024-03-08T21:00:00Z",
            "2024-03-08T00:00:00Z",
        ),
        (
            "2024-03-11T13:30:00Z",
            "2024-03-11T20:00:00Z",
            "2024-03-11T00:00:00Z",
        ),
    ]
    .map(|(a, b, c)| (millis(a), millis(b), millis(c)));
    let env = environment("60").with_bar_calendars(Arc::new(
        BarCalendarRegistry::new([calendar("NASDAQ:AAPL", "America/New_York", &days)]).unwrap(),
    ));
    let source = program(
        "plot(time_close-time)\nplot(hour)\nplot(hour(time,\"UTC\"))\nplot(time_tradingday)",
    );
    let result = HistoricalRuntime::with_request_environment(&source, env)
        .run(&[at(days[0].1 - 1_800_000), at(days[1].0)])
        .unwrap();
    assert_eq!(
        result.plots[0].values,
        [PineValue::Int(1_800_000), PineValue::Int(3_600_000)]
    );
    assert_eq!(
        result.plots[1].values,
        [PineValue::Int(15), PineValue::Int(9)]
    );
    assert_eq!(
        result.plots[2].values,
        [PineValue::Int(20), PineValue::Int(13)]
    );
    assert_eq!(result.plots[3].values, days.map(|d| PineValue::Int(d.2)));
}

#[test]
fn calendar_sessions_without_offsets_and_known_closures() {
    let source = program(
        "plot(time_close(\"1\",\"0930-1600\")-time)\nplot(time_close(\"1\",\"0930-0945\")-time)\nplot(time(\"1\",\"1500-1600\"))",
    );
    let result = HistoricalRuntime::with_request_environment(&source, environment("D"))
        .run(&[at(days()[1].0)])
        .unwrap();
    assert_eq!(result.plots[0].values, [PineValue::Int(23_400_000)]);
    assert_eq!(result.plots[1].values, [PineValue::Int(900_000)]);
    assert_eq!(result.plots[2].values, [PineValue::Na]);
    let calendar = calendar("TEST", "UTC", &days());
    assert_eq!(
        calendar.select(&tf("D"), days()[1].1 + 60_000, 0).unwrap(),
        None
    );
}

#[test]
fn calendar_missing_coverage_is_explicit() {
    for (body, time, message) in [
        (
            "plot(time_close(\"15\"))",
            days()[1].0,
            "missing bar calendar timeframe",
        ),
        (
            "plot(time_close(\"D\",bars_back=1))",
            days()[0].0,
            "missing bar calendar coverage",
        ),
        ("plot(close)", days()[3].1, "missing bar calendar coverage"),
    ] {
        let err = HistoricalRuntime::with_request_environment(&program(body), environment("D"))
            .run(&[at(time)])
            .unwrap_err();
        assert!(err.message.contains(message), "{}", err.message);
    }
}

#[test]
fn calendar_foreign_request_close_merge_and_nested_context_use_registry() {
    let day = days()[1];
    let aapl = RequestKey::new("NASDAQ:AAPL", tf("D"));
    let other = RequestKey::new("OTHER", tf("D"));
    let provider = InMemoryRequestDataProvider::from_streams([
        (aapl, vec![at(day.0)]),
        (other, vec![at(day.0)]),
    ])
    .unwrap();
    let registry = BarCalendarRegistry::new([
        calendar("NASDAQ:AAPL", "America/New_York", &days()),
        calendar("OTHER", "America/Chicago", &days()),
    ])
    .unwrap();
    let env = RequestEnvironment::new(ChartContext::new("UTC:TEST", tf("1")), Arc::new(provider))
        .with_bar_calendars(Arc::new(registry));
    let source = program(
        "plot(request.security(\"NASDAQ:AAPL\",\"D\",time_close-time))\nplot(request.security(\"OTHER\",\"D\",hour))\ninner=request.security(\"\",\"\",hour)\nplot(request.security(\"NASDAQ:AAPL\",\"D\",inner))",
    );
    let result = HistoricalRuntime::with_request_environment(&source, env)
        .run(&[at(day.1 - 120_000), at(day.1 - 60_000)])
        .unwrap();
    assert_eq!(
        result.plots[0].values,
        [PineValue::Na, PineValue::Int(23_400_000)]
    );
    assert_eq!(result.plots[1].values, [PineValue::Na, PineValue::Int(8)]);
    assert_eq!(result.plots[2].values, [PineValue::Na, PineValue::Int(9)]);
}

#[test]
fn calendar_same_symbol_lower_requests_do_not_include_closed_session_suffix() {
    let day = days()[1];
    let provider = InMemoryRequestDataProvider::from_streams([(
        RequestKey::new("NASDAQ:AAPL", tf("1")),
        vec![at(day.0), at(day.1 - 60_000), at(days()[2].0)],
    )])
    .unwrap();
    let env = environment("D").for_chart(ChartContext::new("NASDAQ:AAPL", tf("D")));
    let env = RequestEnvironment::new(env.chart().clone(), Arc::new(provider))
        .with_bar_calendars(Arc::new(env.bar_calendars().clone()));
    let source = program(
        "plot(request.security(\"NASDAQ:AAPL\",\"1\",time))\nplot(array.size(request.security_lower_tf(\"NASDAQ:AAPL\",\"1\",time)))",
    );
    let result = HistoricalRuntime::with_request_environment(&source, env)
        .run(&[at(day.0)])
        .unwrap();
    assert_eq!(result.plots[0].values, [PineValue::Int(day.1 - 60_000)]);
    assert_eq!(result.plots[1].values, [PineValue::Int(2)]);
}

#[test]
fn calendar_realtime_confirmation_replay_and_failed_update_preserve_clock() {
    let source = program("plot(time_close-time)\nplot(hour)");
    let env = environment("60");
    let mut realtime = RealtimeRuntime::with_request_environment(&source, env.clone());
    let value = at(days()[1].1 - 1_800_000);
    realtime.update(BarUpdate::forming(value)).unwrap();
    let confirmed = realtime.update(BarUpdate::confirmed(value)).unwrap();
    assert_eq!(confirmed.plots[0].values, [PineValue::Int(1_800_000)]);
    let before = realtime.result();
    let revision = realtime.revision();
    assert!(
        realtime
            .update(BarUpdate::forming(at(days()[3].1)))
            .is_err()
    );
    assert_eq!(realtime.result(), before);
    assert_eq!(realtime.revision(), revision);
    assert!(realtime.replay_historical(&[at(days()[3].1)]).is_err());
    assert_eq!(realtime.result(), before);
    assert_eq!(realtime.revision(), revision);
    let batch = HistoricalRuntime::with_request_environment(&source, env)
        .run(&[value])
        .unwrap();
    assert_eq!(
        realtime.replay_historical(&[value]).unwrap().plots,
        batch.plots
    );
}

#[test]
fn calendar_json_validates_schema_aliases_order_and_timestamps() {
    let interval = serde_json::json!({"open":0,"close":60_000,"tradingDay":0});
    let symbol = serde_json::json!({"symbol":"TEST","timezone":"UTC","timeframes":[{"timeframe":"D","intervals":[interval.clone()]}]});
    let valid = serde_json::json!({"schemaVersion":1,"symbols":[symbol.clone()]});
    assert!(bar_calendars_from_json(&valid.to_string()).is_ok());
    for invalid in [
        serde_json::json!({"schemaVersion":2,"symbols":[]}),
        serde_json::json!({"schemaVersion":1,"symbols":[symbol.clone(),symbol.clone()]}),
        serde_json::json!({"schemaVersion":1,"symbols":[{"symbol":"TEST","timezone":"UNKNOWN","timeframes":[]}]}),
        serde_json::json!({"schemaVersion":1,"symbols":[{"symbol":"TEST","timezone":"UTC","timeframes":[{"timeframe":"D","intervals":[interval.clone()]},{"timeframe":"1D","intervals":[interval.clone()]}]}]}),
        serde_json::json!({"schemaVersion":1,"symbols":[{"symbol":"TEST","timezone":"UTC","timeframes":[{"timeframe":"D","intervals":[interval.clone(),interval.clone()]}]}]}),
        serde_json::json!({"schemaVersion":1,"symbols":[{"symbol":"TEST","timezone":"UTC","timeframes":[{"timeframe":"D","intervals":[{"open":0,"close":0,"tradingDay":0}]}]}]}),
        serde_json::json!({"schemaVersion":1,"symbols":[{"symbol":"TEST","timezone":"UTC","timeframes":[{"timeframe":"D","intervals":[{"open":0,"close":60_000,"tradingDay":1}]}]}]}),
        serde_json::json!({"schemaVersion":1,"symbols":[],"unknown":true}),
    ] {
        assert!(
            bar_calendars_from_json(&invalid.to_string()).is_err(),
            "{invalid}"
        );
    }
}

#[test]
fn calendar_native_continuous_utc_lower_open_offsets() {
    let source = program(
        "plot(time(\"1\",timeframe_bars_back=1)-time)\nplot(time(\"1\",timeframe_bars_back=2)-time)\nplot(time(\"1\",timeframe_bars_back=-1)-time)\nplot(time(\"1\",timeframe_bars_back=-2)-time)",
    );
    let env = RequestEnvironment::default().for_chart(ChartContext::new("UTC:TEST", tf("D")));
    let result = HistoricalRuntime::with_request_environment(&source, env)
        .run(&[at(days()[1].2)])
        .unwrap();
    for (plot, value) in result.plots.iter().zip([-86_400_000, -86_400_000, 0, 0]) {
        assert_eq!(plot.values, [PineValue::Int(value)]);
    }
}

#[test]
fn calendar_overnight_trading_day_keeps_vwap_across_utc_midnight() {
    let source = program("plot(ta.vwap)\nplot(time_tradingday)");
    let first = millis("2026-10-08T23:59:00Z");
    let day = millis("2026-10-09T00:00:00Z");
    let times = [first, day, day + 86_400_000];
    let labels = [day, day, day + 86_400_000];
    let cal = BarCalendar::new(
        "NIGHT",
        "UTC",
        [(
            tf("1"),
            times
                .into_iter()
                .zip(labels)
                .map(|(time, label)| BarTimeInterval::new(time, time + 60_000, label))
                .collect(),
        )],
    )
    .unwrap();
    let env = RequestEnvironment::default()
        .for_chart(ChartContext::new("NIGHT", tf("1")))
        .with_bar_calendars(Arc::new(BarCalendarRegistry::new([cal]).unwrap()));
    let bars = times
        .into_iter()
        .zip([10.0, 20.0, 30.0])
        .map(|(time, close)| {
            let mut value = at(time);
            value.open = close;
            value.high = close;
            value.low = close;
            value.close = close;
            value.volume = 1.0;
            value
        })
        .collect::<Vec<_>>();
    let result = HistoricalRuntime::with_request_environment(&source, env)
        .run(&bars)
        .unwrap();
    assert_eq!(
        result.plots[0].values,
        [
            PineValue::Float(10.0),
            PineValue::Float(15.0),
            PineValue::Float(30.0)
        ]
    );
    assert_eq!(result.plots[1].values, labels.map(PineValue::Int));
}

#[test]
fn lower_daily_weekly_closes_use_supplied_month_chart_calendars() {
    let open = millis("2026-10-01T13:30:00Z");
    let daily_close = millis("2026-10-01T20:00:00Z");
    let weekly_close = millis("2026-10-02T20:00:00Z");
    let month_close = millis("2026-10-30T20:00:00Z");
    let day = millis("2026-10-01T00:00:00Z");
    let month = vec![(tf("M"), vec![BarTimeInterval::new(open, month_close, day)])];
    let full = month
        .iter()
        .cloned()
        .chain([
            (tf("D"), vec![BarTimeInterval::new(open, daily_close, day)]),
            (
                tf("W"),
                vec![BarTimeInterval::new(
                    millis("2026-09-28T13:30:00Z"),
                    weekly_close,
                    millis("2026-09-28T00:00:00Z"),
                )],
            ),
        ])
        .collect();
    let make_env = |streams| {
        RequestEnvironment::default()
            .for_chart(ChartContext::new("NASDAQ:AAPL", tf("M")))
            .with_bar_calendars(Arc::new(
                BarCalendarRegistry::new([BarCalendar::new(
                    "NASDAQ:AAPL",
                    "America/New_York",
                    streams,
                )
                .unwrap()])
                .unwrap(),
            ))
    };
    let source = program(
        "plot(time_close(\"D\")-time)\nplot(time_close(\"W\")-time)\nplot(time_close(\"D\",\"0930-0945\")-time)",
    );
    let result = HistoricalRuntime::with_request_environment(&source, make_env(full))
        .run(&[at(open)])
        .unwrap();
    for (plot, value) in result.plots.iter().zip([23_400_000, 109_800_000, 900_000]) {
        assert_eq!(plot.values, [PineValue::Int(value)]);
    }
    let err = HistoricalRuntime::with_request_environment(&source, make_env(month))
        .run(&[at(open)])
        .unwrap_err();
    assert!(
        err.message.contains("missing bar calendar timeframe"),
        "{}",
        err.message
    );
}
