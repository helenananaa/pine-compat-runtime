# UT Bot ETHUSD daily parameter expansion

Verified 2026-09-30 from fresh authenticated Chrome/TradingView captures.
This extends the preceding weekly receipt to daily history with the complete,
unchanged 43-line Pine v4 [QuantNomad UT Bot Strategy](https://www.tradingview.com/script/VKMgZdYr-UT-Bot-Strategy/).
Source SHA256: `0e6ea37251e0a35b45de238427c6a9692a64b62abc502198aad6dad7a514d0c1`.

## Native inputs and history

Actual feed is COINBASE:ETHUSD, 1D; USD, price grid 1/100, point value 1.
All native properties are captured after applying USD 1,000,000 initial
capital, fixed quantity 1, pyramiding 1, zero commission/slippage,
infinite leverage, on-bar-close calculation and one-tick execution delay.
The earlier visible 100,000 capital is retained separately. The original
strategy source was not rewritten to set properties.

The qualified raw chart exports contain 3,783 rows. Local execution retains
all 3,782 closed daily bars from 2016-05-23 to 2026-09-29 UTC; only the forming
2026-09-30 row is excluded. No warmup prefix or native trade is discarded.
The comparator explicitly asserts no native fill on the omitted day.

| Parameters | Closed trades | Buy/Sell cells | Final open entry |
| --- | ---: | ---: | ---: |
| Sensitivity 1, ATR10, ordinary candles | 387 | 7,564 | 1 |
| Sensitivity 2 | 149 | 7,564 | 1 |
| ATR14 | 391 | 7,564 | 1 |
| Heikin Ashi signals | 279 | 7,564 | 1 |

Totals are **1,206 closed trades and 30,256 signal cells** across four
parameter cases sharing one history. All entries, exits, directions,
identifiers, fixed quantities and prices agree. Maximum fill-price error is
zero and maximum displayed native net-PnL error is 7.39e-13 USD. Each last
open entry agrees in direction, date, price and quantity. Batch, incremental
and realtime-history complete outputs are byte-identical within each case:
12 outputs over four independent native cases.

## Export completeness and feed revisions

The first HA chart export contained only 2,332 rows beginning 2020-05-13,
while its trade report began in 2016. The comparator rejected it before
qualification. `tv-chart-heikin_ashi-incomplete.csv` and its export/range
records are retained. Reapplying the full date range produced all 3,783 rows;
the qualified HA chart is download `COINBASE_ETHUSD, 1D (20).csv` and the
fresh trade report is `UT_Bot_Strategy_COINBASE_ETHUSD_2026-09-30 (8).csv`.

All qualified closed time/OHLC values match exactly. Three closed volume
values changed between the baseline and HA recapture:

| UTC date | Baseline volume | HA recapture volume |
| --- | ---: | ---: |
| 2026-09-19 | 62674.21050402 | 62675.34184247 |
| 2026-09-20 | 70488.73837808 | 70512.72275196 |
| 2026-09-22 | 124769.85332204 | 124754.89937609 |

`ha-recapture-feed-deltas.json` records these exact three deltas. The
comparator requires their case, row, time and values and rejects any other
closed OHLCV difference. The original UT source has no volume reference.
Thus this receipt does not claim byte-identical volume across all captures.
The external HA provider uses baseline OHLCV and is independently checked
at every row; only its requested close is read by the script. HA close is
OHLC/4, seed open OC/2, later open the preceding HA open/close mean, with
high/low envelopes. Concrete data preparation remains outside the core.

## Frozen evidence and current core

Raw downloads, screenshots, applied settings, metadata, source, local bars,
provider, CLI commands and hashes are retained in
`.local/ut-bot-ethusd-daily-20260930/`. `compare.py` recomputes all four native
comparisons; `freeze_verify.py` pins evidence and verifies the inherited
core/executable/full-gate receipts. The core patch and executable match the
preceding completed fee gate: no new semantic fix, golden update or full-gate
run is needed. That gate passed 1,988 runtime, 242 CLI and 774 installed-wheel
tests, plus WASM/host checks; its full-script guards remain pinned.

The exact original editor draft was restored and verified, UT Bot removed,
BTCUSD daily restored, and the sampling tab closed. Restored state and PNG
are retained. The older stale debugger tab remains without verified cleanup.

Qualification covers native signals, closed trades and open-entry fields.
It does not cover native bar colors/styles, equity, forming-day unrealized
PnL, fractional quantities or live Tick parity. Non-unit point value is
still explicitly unsupported, and earlier Hull displayed monetary residuals
and additional order-type/fee controls remain open. The expansion goal stays
active; next broaden complete-script crypto/forex intraday coverage and
collect native limit/stop/reversal controls as needed.
