# UT Bot ETHUSD four-hour parameter expansion

Verified 2026-09-30 from four fresh authenticated Chrome/TradingView cases.
The full unchanged 43-line Pine v4 [QuantNomad UT Bot Strategy](https://www.tradingview.com/script/VKMgZdYr-UT-Bot-Strategy/)
has source SHA256 `0e6ea37251e0a35b45de238427c6a9692a64b62abc502198aad6dad7a514d0c1`.
This expands crypto intraday coverage beyond the preceding daily/weekly
receipts and the earlier EURUSD four-hour family.

## Native source and execution settings

Actual exported feed: COINBASE:ETHUSD, 240; USD, price grid 1/100,
quantity precision 4, point value 1. Native applied Properties are captured
for every case: capital 1,000,000 USD, fixed quantity 1, pyramiding 1,
zero commission/slippage, infinite leverage, on-bar-close calculation,
one-tick execution delay. The original v4 strategy declaration remains
unchanged. The earlier visible 100,000 capital is retained separately.

All four qualified chart exports contain 21,353 raw rows. Local execution
uses all 21,352 closed bars, from 2017-01-01 00:00 UTC through
2026-09-30 04:00 UTC. The single forming 08:00 UTC bar remains in each raw
CSV and is explicitly omitted from local historical runs. No warmup prefix
is dropped. The comparator asserts that no native entry or exit occurs on
the omitted bar, so all native closed trades and the final open entry are
compared without excluding trades.

TradingView report timestamps use UTC+8 to the minute. Each is mapped
uniquely to the exported UNIX timestamp; daily-date matching is not used.
The four closed time/OHLCV histories match exactly, with no feed-revision
exception needed. The observed chart range returns to recent dates after
parameter changes. Each nondefault case therefore retains a range check
and explicitly reloads full history before chart export.

## Independent comparisons

| Native inputs | Closed trades | Buy/Sell cells | Open entry checked |
| --- | ---: | ---: | ---: |
| Sensitivity 1, ATR10, ordinary candles | 2,232 | 42,704 | 1 |
| Sensitivity 2 | 899 | 42,704 | 1 |
| ATR14 | 2,236 | 42,704 | 1 |
| Heikin Ashi | 1,686 | 42,704 | 1 |

Total: **7,053 closed trades and 170,816 signal cells** across four parameter
cases sharing one 21,352-bar history. All signal bits, entry identifiers,
directions, signed fixed quantities, entry/exit timestamps and fill prices
agree. Maximum fill-price error is zero; maximum displayed native PnL error
is 8.82e-13 USD. All four final open-entry fields also agree. The changing
unrealized PnL is excluded from native qualification.

Batch, incremental and realtime-history each execute the entire unchanged
source. All three complete JSON outputs are byte-identical within each
case: twelve outputs and no runtime diagnostics.

The original HA request expression uses externally prepared bars through
the canonical key `{"chart":"heikinashi","symbol":"COINBASE:ETHUSD"}:240`.
All 21,352 provider rows are independently recomputed and verified: HA close
OHLC/4, seed open OC/2, later open the preceding HA open/close mean, and
high/low envelopes. Time and volume are copied exactly. Data preparation
remains an external fixture; no market-data acquisition enters the core.

## Evidence and qualification limits

`.local/ut-bot-ethusd-fourhour-20260930/` retains raw downloads, source,
bars/provider, applied Inputs and Properties, export/range records,
screenshots, local commands/receipts, comparisons and frozen hashes.
`capture-provenance.json` records original filenames in `I:\sys\下载`.
`freeze_verify.py` reruns every comparison and verifies source/artifact pins.

No core change or golden update was needed. The core patch, CLI executable
and preceding completed fee gate still match inherited pins. That existing
gate's successful receipt is revalidated; no new full-gate run is claimed.
It passed 1,988 runtime, 242 CLI and 774 installed-wheel tests, plus WASM/host
checks. Earlier complete-script outputs remain pinned through the manifest
chain.

During sampling, the editor draft changed externally from the initial mark
probe to `Pyne WMA gap confirmation 20260930`. Both drafts and the observed
state are retained. The latest WMA draft was preserved without rewriting,
and a second clipboard read confirmed exact equality. Original-draft
restoration was superseded by that concurrent change. Only this round's
temporary UT strategy was removed; BTCUSD daily was restored and the
sampling tab closed, with state/PNG evidence. Older stale debugger-tab
cleanup remains unverified.

This receipt qualifies native signals, closed trades and final open entries;
native colors/styles, account equity, forming-bar PnL, fractional quantities
and live Tick parity are outside it. Non-unit point value remains explicitly
unsupported. Earlier Hull displayed monetary residuals and additional
limit/stop/repeated-partial/reversal controls remain open. The expansion goal
stays active; next broaden complete-script market coverage and independently
probe remaining order semantics.
