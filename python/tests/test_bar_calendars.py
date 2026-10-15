import copy
from datetime import datetime

import pine_compat
import pytest


def ms(value):
    return int(datetime.fromisoformat(value).timestamp() * 1000)


def calendar_input():
    days = [
        (ms(f"2026-10-{day:02}T13:30:00+00:00"),
         ms(f"2026-10-{day:02}T20:00:00+00:00"),
         ms(f"2026-10-{day:02}T00:00:00+00:00"))
        for day in [8, 9, 12]
    ]
    intervals = lambda duration: [
        dict(open=start, close=min(start + duration, close), tradingDay=day)
        for open_, close, day in days for start in range(open_, close, duration)
    ]
    return dict(schemaVersion=1, symbols=[dict(
        symbol="NASDAQ:AAPL", timezone="America/New_York", timeframes=[
            dict(timeframe="D", intervals=[dict(open=o, close=c, tradingDay=d) for o, c, d in days]),
            dict(timeframe="1", intervals=intervals(60_000)),
            dict(timeframe="60", intervals=intervals(3_600_000)),
        ],
    )])


def bar(time):
    return dict(time=time, open=10, high=10, low=10, close=10, volume=1)


@pytest.mark.parametrize("version", [5, 6])
def test_calendar_native_daily_boundary_timezone_and_weekend_offsets(version):
    source = f'''//@version={version}
indicator("calendar")
plot(time_close-time)
plot(time_close("1")-time)
plot(time_close("1",timeframe_bars_back=1)-time)
plot(time("1",timeframe_bars_back=1)-time)
plot(hour)
plot(hour(time))
plot(minute)
plot(hour(time,"UTC"))
plot(timestamp(year,month,dayofmonth,9,30)-time)
plot(str.format_time(time,"HH:mm")=="09:30"?1:0)
plot(syminfo.timezone=="America/New_York"?1:0)
plot(time_close("D",bars_back=1)-time)
plot(time("D",bars_back=-1)-time)
'''
    program = pine_compat.compile_script(source)
    host = {"$barCalendars": calendar_input()}
    result = program.run([bar(ms("2026-10-09T13:30:00+00:00"))],
                         chart_symbol="NASDAQ:AAPL", chart_timeframe="D", request_bars=host)
    assert [p["values"] for p in result["plots"]] == [
        [v] for v in [23_400_000, 23_400_000, -63_000_000, -86_400_000,
                      9, 9, 30, 13, 0, 1, 1, -63_000_000, 259_200_000]
    ]
    assert program.host_requirements()["chart"]["barCalendars"] == "continuousUtcDefaultOrHostSuppliedTradingIntervalsV1"


@pytest.mark.parametrize("body, time, message", [
    ('plot(time_close("15"))', "2026-10-09T13:30:00+00:00", "missing bar calendar timeframe"),
    ('plot(time_close("D",bars_back=1))', "2026-10-08T13:30:00+00:00", "missing bar calendar coverage"),
])
def test_calendar_missing_input_is_explicit(body, time, message):
    program = pine_compat.compile_script(f'//@version=6\nindicator("calendar")\n{body}\n')
    with pytest.raises(ValueError, match=message):
        program.run([bar(ms(time))], chart_symbol="NASDAQ:AAPL", chart_timeframe="D",
                    request_bars={"$barCalendars": calendar_input()})


@pytest.mark.parametrize("case", ["schema", "unknown", "overlap", "timezone"])
def test_calendar_rejects_malformed_inputs(case):
    value = copy.deepcopy(calendar_input())
    if case == "schema":
        value["schemaVersion"] = 2
    elif case == "unknown":
        value["unknown"] = True
    elif case == "overlap":
        value["symbols"][0]["timeframes"][0]["intervals"][1]["open"] -= 86_400_000
    else:
        value["symbols"][0]["timezone"] = "not/a/timezone"
    program = pine_compat.compile_script('//@version=6\nindicator("calendar")\nplot(close)\n')
    with pytest.raises(ValueError, match="bar calendar|bar calendars"):
        program.run([bar(ms("2026-10-09T13:30:00+00:00"))],
                    request_bars={"$barCalendars": value})


@pytest.mark.parametrize("version", [5, 6])
def test_virtual_calendar_native_session_offsets_and_weekend_mask(version):
    source = f'''//@version={version}
indicator("virtual")
plot(time("1","0930-0945",timeframe_bars_back=1)-time)
plot(time_close("1","0930-0945",timeframe_bars_back=1)-time)
plot(time("1","1500-1600",timeframe_bars_back=1)-time)
plot(time_close("1","1500-1600",timeframe_bars_back=1)-time)
plot(time("1","0930-0945",timeframe_bars_back=-16)-time)
plot(time("1","0930-0945:23456",timeframe_bars_back=-16)-time)
plot(time("1","24x7")-time)
'''
    result = pine_compat.compile_script(source).run(
        [bar(ms("2026-10-09T13:30:00+00:00"))], chart_symbol="NASDAQ:AAPL",
        chart_timeframe="D", request_bars={"$barCalendars": calendar_input()})
    assert [p["values"] for p in result["plots"]] == [[v] for v in
        [-86_400_000, -85_500_000, -66_600_000, -63_000_000,
         86_400_000, 259_200_000, -34_200_000]]


@pytest.mark.parametrize("version, expected", [(4, 999), (5, 0), (6, 0)])
def test_virtual_session_native_versioned_default_days(version, expected):
    declaration = "study" if version == 4 else "indicator"
    source = f'''//@version={version}
{declaration}("days")
x=time("1","0000-0015","UTC")
plot(na(x)?999:x-time)
plot(time("1","0000-0015:1234567","UTC")-time)
'''
    result = pine_compat.compile_script(source).run(
        [bar(ms("2026-10-10T00:00:00+00:00"))], chart_timeframe="D")
    assert [p["values"] for p in result["plots"]] == [[expected], [0]]


@pytest.mark.parametrize("time, expected", [
    ("2026-03-08T00:00:00+00:00", [-7_200_000, 75_600_000]),
    ("2025-11-02T00:00:00+00:00", [-10_800_000, 79_200_000]),
])
def test_virtual_session_native_dst_without_calendar(time, expected):
    source = '''//@version=6
indicator("dst")
plot(time("1","1700-1700","America/New_York")-time)
plot(time_close("1","1700-1700","America/New_York")-time)
'''
    result = pine_compat.compile_script(source).run([bar(ms(time))], chart_timeframe="D")
    assert [p["values"] for p in result["plots"]] == [[v] for v in expected]


def test_virtual_calendar_intraday_phase_and_weekly_nominal_closes():
    source = '''//@version=6
indicator("phase")
plot(time("30","1515-1600")-time)
plot(time_close("30","1515-1600")-time)
plot(time_close("W","0930-0945:23456")-time)
plot(time("W","0930-0945:23456",timeframe_bars_back=1)-time)
plot(time_close("M","0930-0945:23456")-time)
'''
    result = pine_compat.compile_script(source).run(
        [bar(ms("2026-10-09T19:30:00+00:00"))], chart_symbol="NASDAQ:AAPL",
        chart_timeframe="60", request_bars={"$barCalendars": calendar_input()})
    assert [p["values"] for p in result["plots"]] == [[v] for v in
        [-900_000, 900_000, 237_600_000, -367_200_000, 2_055_600_000]]
