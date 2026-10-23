import pytest
import pine_compat


@pytest.mark.parametrize("version", [2, 3, 4, 5, 6])
def test_numeric_history_truncates_before_sign_check(version):
    declaration = "study" if version <= 4 else "indicator"
    source = f'''//@version={version}
{declaration}("numeric history")
plot(close[2.8])
plot(close[-0.8])
plot(close[close%3+0.8])
'''
    bars = [{"time": i*60000, "open": i+1, "high": i+1, "low": i+1,
             "close": i+1, "volume": 1} for i in range(6)]
    result = pine_compat.compile_script(source).run(bars)
    assert [p["values"] for p in result["plots"]] == [
        [None, None, 1, 2, 3, 4], [1, 2, 3, 4, 5, 6], [None, None, 3, 3, 3, 6]]


def test_large_timestamp_month_first_date_and_2400_session():
    source = '''//@version=6
indicator("date compatibility")
plot(timestamp(999999,12,31,23,59))
plot(timestamp("Jan 19 2024 00:00:01"))
plot(time("", "0000-2400:1234567", "UTC"))
plot(time("", "2400-1600:1234567", "UTC"))
plot(time("30S"))
plot(time("S"))
plot(timestamp("2021-01-13:05:00"))
plot(timestamp("01 Jan 2023 00:00 -3000"))
'''
    bars = [{"time": 1791711000000, "open": 1, "high": 1, "low": 1, "close": 1, "volume": 1}]
    result = pine_compat.compile_script(source).run(bars, chart_symbol="A", chart_timeframe="5")
    assert [p["values"] for p in result["plots"]] == [
        [31494784780740000], [1705622401000], [1791711000000], [1791711000000],
        [1791711000000], [1791711000000], [1610514000000], [1672639200000]]


@pytest.mark.parametrize("version", [5, 6])
def test_compact_dates_preserve_native_format_dependent_offsets(version):
    source = '//@version=5\nindicator("Compact date oracle 22",precision=0)\nplot(timestamp("15Aug 2022 14:00 +0000"),title="start")\nplot(timestamp("15Aug 2022 20:00 +0000"),title="end")\nplot(timestamp("15Aug 2022 14:00 +0530"),title="offset")\nplot(timestamp("15 Aug 2022 14:00 +0530"),title="spacedOffset")\nplot(timestamp("15Aug 2022 14:00 GMT+0530"),title="gmtOffset")\nplot(timestamp("2022-08-15T14:00:00+05:30"),title="isoOffset")\nplot(timestamp("1Jan 2024 05:30:01 +0530"),title="secondsOffset")\nplot(timestamp("15Aug 2022"),title="dateOnly")\nplot(timestamp("15Aug 2022 14:00:30 +0000"),title="seconds")\nplot(timestamp("15Aug 2022 14:00 -0400"),title="negativeOffset")\nplot(timestamp("15August 2022 14:00 +0000"),title="fullMonth")\nplot(timestamp("15aug 2022 14:00"),title="lowercase")\n'.replace("version=5", f"version={version}")
    result = pine_compat.compile_script(source).run([{ "time": 0, "open": 1, "high": 1, "low": 1, "close": 1, "volume": 1 }])
    assert [p["values"] for p in result["plots"]] == [[v] for v in [1660572000000, 1660593600000, 1660572000000, 1660552200000, 1660572000000, 1660552200000, 1704087001000, 1660521600000, 1660572030000, 1660572000000, 1660572000000, 1660572000000]]
