# Host bar calendars (schema 1)

The interpreter accepts immutable, host-supplied trading intervals. The host
provides exchange facts and data acquisition; the runtime chooses intervals,
applies Pine offsets, constructs explicit-session virtual bars, and confirms requested
values. No symbol name triggers an exchange lookup or inferred holiday schedule.

Rust callers construct `BarCalendar` / `BarCalendarRegistry` and attach an
`Arc<BarCalendarRegistry>` with `RequestEnvironment::with_bar_calendars`.
`for_chart` preserves the registry for same-symbol, foreign-symbol and nested
contexts, including incremental checkpoints and realtime candidate/replay state.
An environment's registry cannot be changed after construction.

CLI callers pass `--bar-calendars calendars.json` with any run mode. Python
callers supply `request_bars={"$barCalendars": input, ...}`; WASM direct and
prepared CSV methods use the same `$barCalendars` member of request-bars JSON.
The input format is identical across bindings:

```json
{
  "schemaVersion": 1,
  "symbols": [{
    "symbol": "NASDAQ:AAPL",
    "timezone": "America/New_York",
    "timeframes": [{
      "timeframe": "D",
      "intervals": [{
        "open": 1791466200000,
        "close": 1791489600000,
        "tradingDay": 1791417600000
      }]
    }]
  }]
}
```

All timestamps are integer UTC milliseconds. `tradingDay` is the UTC-midnight
date label of the interval's trading day, including overnight sessions. A
weekly/monthly interval uses its last trading-day label. For ordinary exchange-calendar queries, supply every queried
timeframe and sufficient previous/future intervals for the requested offsets.
Explicit Pine session strings define a separate deterministic virtual grid;
those queries need the chart calendar for chart offsets, but do not require
ordinary requested-timeframe intervals or price streams for the virtual grid.
Intraday `time` / `time_close` queries on D/W/M charts also need the selected
symbol's D intervals. The runtime selects the requested intraday interval
first, then uses its containing daily open/close. Ordinary intraday chart bars
use their supplied close, including the final shortened session bar.

Intervals must have positive duration, sorted nonoverlapping opens/closes,
nondecreasing trading-day labels, and valid timestamp/timezone ranges. Duplicate
symbols and equivalent timeframe aliases (such as D and 1D), empty streams,
unknown fields and schema versions are rejected. Bar streams must open at an
exact supplied interval. Calendar input does not contain prices and does not
replace request bars or permit a missing market-data stream.

An absent symbol retains the existing continuous UTC profile. A registered
symbol requires explicit calendar coverage: missing timeframes or timestamps
outside the first-open/last-close range fail rather than falling back to UTC.
Known closures inside that range produce no clock interval. Query offsets count
supplied trading intervals, so they skip weekends, holidays and session breaks.
The first covered merge bar has no predecessor (`i64::MIN` merge boundary);
an explicit past-offset query outside coverage remains an error.

The shared clock also governs ordinary `time_close`, `time_tradingday`, calendar
components, `syminfo.timezone`, implicit numeric `timestamp`, `str.format_time`,
`timeframe.change`, implicit VWAP day anchors, security merge boundaries,
incremental request horizons and realtime request-feed trimming. Trimming
computes all fallible decisions before mutating request streams; failed chart
replays retain the previous confirmed session.

## Current qualification limits

The initial native controls cover AAPL regular-session daily/lower-intraday
opens and closes, default exchange timezone, and previous/future trading
intervals. Controlled tests also cover DST, weekends, shortened hourly bars,
overnight trading-day anchors, requests and realtime rollback. These controls
do not qualify full market or trade-list parity.

Explicit session queries apply chart offsets using the chart calendar, then
count requested virtual intervals independently. Session starts align the grid;
the last partial interval closes at the session end. Positive/future offsets
outside a session anchor to the nearest virtual bar in the respective direction
before applying the full offset. Default days are weekdays in v4 and earlier,
all days in v5/v6; explicit masks override them. Overnight masks use the closing
trading date. IANA boundaries account for DST; equal nonzero start/end times
represent a full overnight day. Split sessions keep independent intraday phases.

On D/W/M charts, lower intraday queries return the selected virtual session's
open/close. D queries aggregate each admitted day's spans. W/M queries on
D/W/M charts end at the last admitted session close; on intraday charts they
end at the next virtual period's first admitted opening. For the latter,
nonzero offsets outside the daily session use the next virtual-period anchor.
Native probes and independent binding/runtime tests record these distinctions.

Date/period preparation consumes the existing deterministic evaluation-step
budget; minute/second intervals are counted and selected arithmetically rather
than materialized. Missing main-chart calendar coverage still fails explicitly.
Lower daily/weekly time_close queries on larger chart periods select the
requested ordinary interval or explicit virtual-session grid. Missing requested
calendar timeframes/coverage remain explicit host-input errors. Multi-day group anchoring, all exotic overlapping sessions,
ambiguous/gapped local-opening parity, unavailable-intrabar fallback and currency
conversion remain separate unqualified areas. No full market/broker/trade-list
qualification is claimed.
