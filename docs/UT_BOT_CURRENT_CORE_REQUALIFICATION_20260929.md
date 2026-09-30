# UT Bot Strategy v4 on the current broker and extrema patch

Verified 2026-09-29. The unchanged 43-line public Pine v4
[UT Bot Strategy by QuantNomad](https://www.tradingview.com/script/VKMgZdYr-UT-Bot-Strategy/)
was executed again against preserved independent Chrome/TradingView captures
from 2026-09-27. Twelve named settings across three symbol/period families
have fresh runs in all three historical modes and fresh native comparator
results. **All 36 complete outputs are byte-identical to their previous
outputs.** No runtime diagnostics or native comparison errors were found.

These are new current-source execution receipts for existing captures.
No new TradingView capture was obtained during this requalification;
Chrome communication recovery remains pending for new weekly VAMA sampling.

## Original source, settings and feed identity

All three original source files have SHA-256
`0e6ea37251e0a35b45de238427c6a9692a64b62abc502198aad6dad7a514d0c1`.
The original v4 boolean `strategy.entry` directions, `iff`/`nz` history
state, ATR trailing stop, and
`security(heikinashi(syminfo.tickerid), timeframe.period, close, lookahead=false)`
branch are retained without script rewrite.

Frozen native properties were 1,000,000 USD initial capital, fixed order
size 1, pyramiding 1, zero commission and slippage, on-bar-close execution.
The original reports record these settings. Local execution uses the
unchanged original strategy declaration and the explicitly recorded input
and host metadata arguments. Comparison is of exported signals and trades;
this does not qualify arbitrary strategy property overrides.

The original reports are also frozen as `original-capture-report.md` in
each new evidence family. `property-records.json` records their hashes,
documented native properties and the current execution-matrix hash. This
preserves recorded settings provenance, without claiming a fresh UI
property capture while Chrome is unavailable. The narrow freeze helper is
`.local/freeze_ut_bot_property_records_20260929.py`.

| Family | Actual chart feed | Timeframe | Price grid | Bars | Original receipt |
| --- | --- | --- | --- | ---: | --- |
| EURUSD four-hour | FX:EURUSD | 240 | 1/100000 | 21,338 | [Original](UT_BOT_FX_FOURHOUR_EXPANSION_20260927.md) |
| AAPL daily | BATS:AAPL | 1D | 1/100 | 11,533 | [Original](UT_BOT_AAPL_DAILY_EXPANSION_20260927.md) |
| AAPL weekly | BATS:AAPL | 1W | 1/100 | 2,390 | [Original](UT_BOT_AAPL_WEEKLY_EXPANSION_20260927.md) |

All local metadata uses USD, integer quantity precision, point value 1.
The AAPL picker/trade report identifies NASDAQ:AAPL while the actual chart
export is BATS:AAPL, “NASDAQ by Cboe One.” This provider identity remains
explicit. EURUSD history begins 2013-01-02 02:00 UTC; AAPL daily begins
1980-12-12 14:30 UTC and weekly 1980-12-08 14:30 UTC. All histories end at
the closed-market capture on 2026-09-25, or its weekly session. No initial
prefix or final forming row is removed.

## Fresh native comparison on current source

Each setting checks every Buy/Sell bit and every closed trade's direction,
entry/exit time or session date, price within `1e-9`, fixed quantity, and
net PnL within the precision of the native CSV. Each final open entry is
checked in direction, time/date, price and quantity; its unrealized PnL
is excluded. Intraday report timestamps use UTC+8 minutes; daily/weekly
reports use native session dates.

| Family | Setting | Closed trades | Signal cells checked | Open entries | Errors |
| --- | --- | ---: | ---: | ---: | ---: |
| EURUSD 4h | Default sensitivity 1, ATR10 | 2,459 | 42,676 | 1 | 0 |
| EURUSD 4h | Sensitivity 2 | 1,074 | 42,676 | 1 | 0 |
| EURUSD 4h | ATR14 | 2,471 | 42,676 | 1 | 0 |
| EURUSD 4h | Heikin Ashi | 1,815 | 42,676 | 1 | 0 |
| AAPL daily | Default sensitivity 1, ATR10 | 1,585 | 23,066 | 1 | 0 |
| AAPL daily | Sensitivity 2 | 683 | 23,066 | 1 | 0 |
| AAPL daily | ATR14 | 1,591 | 23,066 | 1 | 0 |
| AAPL daily | Heikin Ashi | 1,197 | 23,066 | 1 | 0 |
| AAPL weekly | Default sensitivity 1, ATR10 | 315 | 4,780 | 1 | 0 |
| AAPL weekly | Sensitivity 2 | 139 | 4,780 | 1 | 0 |
| AAPL weekly | ATR14 | 309 | 4,780 | 1 | 0 |
| AAPL weekly | Heikin Ashi | 211 | 4,780 | 1 | 0 |

These settings share histories; counts must not be summed as independent
bars. Chart OHLC and time are checked for all native exports. Volume also
matches except the documented final AAPL weekly HA revision: native
186,838,541 versus earlier 193,764,266. The script does not read volume,
and the unchanged original weekly comparator allows only that exact
case/last-row location. The revision is retained in the fresh comparison.

## Host-neutral Heikin Ashi adapter

The original `build_ha_provider.py` was rerun independently for each family.
Every rebuilt provider CSV is byte-identical to the original provider. HA
close is OHLC/4, initial HA open is OC/2, later HA open is the previous HA
open/close mean, and high/low use the envelopes. The CLI receives these
external bars through `--request-bars`; the core does not fetch market data
or depend on a concrete application.

| Family | Provider CSV SHA-256 |
| --- | --- |
| EURUSD 4h | 3561f5ed7d7041d5fbab8dbcc389c375c3215117e50b0d6ec81e7375d943d036 |
| AAPL daily | 7edc11228530c0e712b668a03a87a64d1f0eef1f562cdbeed193cf5542aca0a9 |
| AAPL weekly | b24da55da147a5896bbf26e7fa98baa013c9f897d178b8fa235178361a823919 |

Provider request keys preserve `{"chart":"heikinashi","symbol":"FX:EURUSD"}:240`
and the corresponding BATS:AAPL `1D`/`1W` keys. This qualifies the original
script's requested close series with the supplied deterministic host data;
it does not independently qualify every native Heikin Ashi OHLC field.

## Current source and reproducibility

Base HEAD: `2f9a83a0c333092167c7253571f56ee82a7ec0d0`.
Current core patch SHA-256:
`827e195c3a7db4422bbd657a5634990adb9d8d1db69658f6331df41a97cb2636`.
Immutable CLI SHA-256:
`584ae27bb25fed15c038c5c55a9356720c1bff90b8db29ce8b403e8a08010f36`.
This includes the current broker fixes and extrema NA repair. No additional
core fix was required for these UT Bot settings.

The identical-source VAMA full gate is reused, with its log hash checked:
1,984 runtime tests, 242 CLI tests, 130 tool tests, WASM Node smoke,
host parity and 774 wheel tests. The full gate was not rerun here.

New artifacts live under `.local/ut-bot-current-core-20260929/`, separately
from the unchanged original evidence directories. Each family retains
native exports, original source, input bars, external provider/adapter,
the unchanged comparator, 12 new mode outputs/terminal command receipts,
fresh native comparison and current verification. Capture provenance
records original comparison identity and hashes every copied native/source
artifact, bars, adapter, provider and available original screenshots.

`.local/requalify_ut_bot_20260929.py` performs the actual executions and
native comparisons, checks original export hashes, rebuilds the providers,
and binds results to current source. Its aggregate is
`.local/ut-bot-current-core-20260929/matrix-verification.json`.
`.local/audit_ut_bot_requalification_20260929.py` is a read-only audit of
current core/CLI identity, native capture chain, explicit host metadata,
input overrides and provider keys, exact output equality, comparisons,
and the reused gate. Later core changes invalidate this qualification until
verified again.

The result covers this unchanged public v4 script, these exact settings,
provider inputs and closed histories. Other scripts, arbitrary parameters,
visual rendering/alerts and native live forming ticks remain separate work.
