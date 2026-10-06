# Bitduke EURUSD 4H: zero-distance exit parameter

Date: 2026-09-27. Baseline commit:
`6a7a2ac24bf9222cc3e77ca08363956bb3c45b48`.
This extends the [four-hour default-setting comparison](FX_BITDUKE_FOURHOUR_20260927.md)
using the unchanged public Pine v4 [Bitduke strategy](https://www.tradingview.com/script/5tuGpzpd-Squeeze-Momentum-Strategy-based-on-Indicator-LazyBear-Bitduke/)
(source SHA-256
`b6aef6dbf531dac2c0a591a198060be4df554a400d98a4952549a723b77b1eb2`).

On the FXCM `FX:EURUSD` 240-minute chart, the sole input change was
`Trailing Stop=0` (input call site `27`); the 2019-01-01 through 2019-12-29
date filter and every other input remained at the script default. TradingView
was run with its usual strategy properties: 1,000 USD initial capital, 1,000
USD default cash order size, 0.075% commission, zero slippage, default
historical bar detail, one-tick order delay, and unlimited leverage. The
report timestamps display in UTC+8.

The script comment says zero disables a risk input, but its derived `tr_stop`
variable is unused: `strategy.exit` receives `trail_points=trailing_stop`
directly. In this observed Pine v4 combination with a fixed profit target,
TradingView still issues exits when the value is zero. The native first trade
enters and exits at `1.13351` on the same 2019-01-03 four-hour bar. We report
the observed behavior rather than treating the comment as an executable rule.

The native trade CSV has SHA-256
`52806873afb8953e1abebdf42336740e69b7f9d450ec4e7954087100f9fedb22`.
The independently exported chart CSV is byte-identical to the default-input
chart export (SHA-256
`cbe87f67ba5cfd0458104534a498074a064b2942f06df82f29c56f4b4b8ff4cc`):
13,637 rows beginning 2017-12-15 14:00 UTC, with the final possible forming
bar excluded from the local input. The local runtime received the 13,636
confirmed bars (input SHA-256
`1adde39f63c05197318bd7bed501e213db136be68234fa965b0a8093508d25ba`),
chart symbol `FX:EURUSD`, timeframe `240`, price grid `1/100000`, and integer
quantity precision `0`.

Before repair, the core rejected a zero relative exit tick distance, yielding
only 60 local trades and 119 public order events; the native report had 64
trades. The runtime now accepts a finite **non-negative** tick distance while
continuing to reject negative or non-finite values. A focused broker test
checks that a zero-tick profit exit fills at entry price. Existing tests that
exercise invalid tick distances now use negative values.

The reproducible `compare_native.py` checks every native entry/exit signal,
displayed timestamp, price, and quantity; every trade's displayed net profit,
entry notional, commission, cumulative profit, and duration; and both plot
series after the first 30 supplied bars. The warmup exclusion accounts for
state retained by TradingView before the exported chart window. Native money
values are rounded to cents and checked within half a cent; finite plot values
use absolute and relative tolerance `1e-9`. The receipt reports **64 matching
trades, 128 matching order rows, and 27,212 matching plot cells**, zero
mismatches, and zero diagnostics.

Batch, incremental, and realtime-history full JSON outputs are byte-identical
(SHA-256 `6be91b4f395bff2a9a39b493a50a4d990c541c27fe7392c6b3b19f3c8d809d0a`).
The filtered 891 strategy tests pass. Source, native CSVs, bars, three local
outputs, comparison script/receipt, and screenshot are retained under ignored
`.local/fx-fourhour-trailing-zero-20260927/`.

This historical comparison does not reveal TradingView's private intrabar
ordering, prove favorable/adverse excursion parity, or validate realtime
forming-bar behavior. The general zero-tick change is based on this observed
complete-script behavior plus the focused local test; other zero-distance
exit forms remain candidates for separate native qualification.
