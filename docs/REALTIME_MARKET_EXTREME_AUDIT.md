# Realtime market-order range expansion

Status: Windows/Linux-qualified correction at `a2a1ba5fb`, 2026-09-10.

## Native evidence

The first Binance trace has two long entry discrepancies: new lows of 78018
and 78012.43 versus supplied closes of 78020 and 78012.44. A second, frozen
128-execution trace alternates long and short round trips. Its short entry at
execution 118 fills at the new high 78047.96 instead of close 78047.95. The
unchanged optimized wheel differs in four repeated entry-price fields out of
1935 values. Of 64 observed fills, 63 match close and all 64 match the
single-new-extreme hypothesis. The two-sided-expansion case is not observed.

Second source SHA-256:
`2df8c37883e3146c97488863f59f9f07c6c29dc2dff5da489672988ab2022a11`.
Its window starts 2026-09-10 11:34 UTC on BINANCE:BTCUSDT one-minute candles.
Source, exact identity checks and 1e-9 numeric tolerances are frozen before
runtime changes. All 128 execution records are contiguous and from one study;
the last record equals the independent native CSV export. There was no browser
event truncation or parser error. Public Binance trade IDs are continuous:
8915 total captured trades. The two execution minutes contain 1543 and 509
trades; their exact decimal OHLCV and counts match exchange REST, and OHLCV
matches the native closed bars. Per-update volume-prefix reconciliation is
126/128, and full OHLC is 81/128, so exchange and native update clocks remain
distinct. Evidence is under `.local/binance-directional-20260910/`.

## Scoped correction

The scheduler retains the previous successful realtime observation separately
from the historical path cursor. When the next observation has the same bar
time and expands exactly one extreme, pending market entries and market closes
use that new high or low. Repeated extremes and first observations use close.
Broker and observation state remain transactional with the realtime session;
no extra script evaluation or invented transaction is introduced.

Price-condition orders continue to evaluate close. Simultaneous high/low
expansion retains close and is not claimed as TradingView-qualified. Historical
execution is unchanged. This is a market-order correction, not a complete
realtime path model. Further price-condition and two-sided controls remain
required before extending the behavior.

Regression tests exercise long/short entries on new highs, new lows, unchanged
ranges, repeated extremes, and the retained two-sided fallback. New native
comparisons and full Windows qualification pass: 6677 Rust / 717 installed
Python / 130 tools / actual WASM. The separately built optimized wheel also
passes all 717 Python tests.

The unchanged original OKX capture now passes 896/896 (previously 16 failed),
and the five-round-trip OKX sample passes 1856/1856 (previously 14 failed).
The first Binance trace passes 975/975 (previously seven failed), and the
directional trace passes 1935/1935 (previously four failed). The earlier
late-attachment control retains 975/975 with its documented opening context.
No reference, denominator, tolerance or price-grid setting changed. Earlier
failed receipts are retained. These demonstrate closure of the named failures,
not full realtime broker compatibility.

Ubuntu-native full verification also passes 6677 Rust / 717 installed Python /
actual WASM; 130 tool tests run with the one Windows-only probe skipped.
Both optimized wheels pass 717 installed Python tests and the same 19 retained
native scenarios, 867418 values per platform with zero mismatches. This includes
the 12 account/admission controls, both formerly failed OKX captures, both new
Binance captures, the passing repeat, late attachment and multi-fill controls.
Raw receipt paths and artifact hashes are indexed in
`.local/binance-directional-20260910/qualification.json`.
