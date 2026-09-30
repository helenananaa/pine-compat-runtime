# UT Bot ETHUSD weekly parameter expansion

Captured and verified 2026-09-30. Local qualification for the named source,
settings and closed history; the continuous expansion goal remains active.

## Independent native evidence

The unchanged 43-line Pine v4 [QuantNomad UT Bot Strategy](https://www.tradingview.com/script/VKMgZdYr-UT-Bot-Strategy/)
was executed in authenticated TradingView Chrome on COINBASE:ETHUSD, 1W.
Source SHA256: `0e6ea37251e0a35b45de238427c6a9692a64b62abc502198aad6dad7a514d0c1`.
Native chart and trade CSVs were downloaded to `I:\sys\下载` and copied intact
to `.local/ut-bot-ethusd-weekly-20260930/`. Each case retains screenshots,
applied Inputs, Properties and UNIX-timestamp export settings.

Explicit matched properties: USD 1,000,000 capital, fixed quantity 1,
pyramiding 1, zero commission/slippage, infinite long/short leverage,
on-bar-close calculation and one-tick execution delay. The initially visible
100,000 capital and a transient appended 1T value are retained as pre-correction
evidence only. Both precede the qualified captures; neither establishes a
native default-capital discrepancy.

All four exports have the same 540 closed OHLCV bars, from 2016-05-23 through
2026-09-21 UTC. Each raw export retains its 541st, forming 2026-09-28 week.
Only that forming row is omitted from local execution. The comparator asserts
that no native entry or exit occurred on the omitted week, so no native trade
is excluded and no warmup prefix is dropped.

## Results

| Case | Sensitivity | ATR | HA | Closed trades | Signal cells | Open entries checked |
| --- | ---: | ---: | --- | ---: | ---: | ---: |
| default | 1 | 10 | false | 49 | 1,080 | 1 |
| sensitivity2 | 2 | 10 | false | 27 | 1,080 | 1 |
| atr14 | 1 | 14 | false | 47 | 1,080 | 1 |
| heikin_ashi | 1 | 10 | true | 37 | 1,080 | 1 |

The four parameter cases total 160 closed trades and 4,320 signal cells over
one shared 540-bar history. Batch, incremental and realtime-history execution
produce 12 complete JSON outputs, byte-identical within each case. Buy/Sell
booleans, entry identifiers, direction, quantity, entry/exit timestamps,
fill prices and final open-entry fields agree with native output. Fill-price
error is zero; maximum displayed native PnL error is 4.55e-13 USD.

The HA branch runs the original `security(heikinashi(...), ...)` expression.
An external deterministic provider fixture supplies all 540 HA bars:
close=(O+H+L+C)/4; initial open=(O+C)/2; subsequent open=(previous HA open+
previous HA close)/2; high/low include the HA envelope. Its entire content,
canonical request key and receipt hash are verified. Data construction remains
outside the runtime core.

## Reproducibility and boundaries

`run_cases.py`, `compare.py`, `comparison.json`, execution receipts and
`freeze_verify.py` are retained beside the raw inputs. The freeze verifier
recomputes comparisons and checks source, executable, provider and artifact
hashes. Core patch and CLI remain identical to the preceding completed fee
gate. That gate's source pins and successful receipt are revalidated; this
round introduces no semantic patch, golden change or new full-gate run.

The prior gate passed 1,988 runtime, 242 CLI and 774 installed-wheel tests,
plus the WASM/host checks. Its twelve full Hull/equity/SSL output receipts
remain inherited and pinned.

This receipt qualifies native signals, closed trades and final open entry.
Native bar colors/styles, account equity, forming-week mark-to-market PnL,
fractional quantities and live tick parity are outside this receipt. Non-unit
point values remain explicitly unsupported. Earlier Hull displayed monetary
residuals and limit/stop/repeated-partial/reversal controls remain open.

The exact original editor draft was restored and verified, temporary UT Bot
was removed, BTCUSD daily was restored and the sampling tab was closed.
Restoration state and screenshot are retained. The older stale debugger tab
has no newly verified cleanup receipt.

Next: extend the same complete script to daily history and additional
unit-point-value markets, preserving independent native captures and explicit
closed-bar cutoffs.
