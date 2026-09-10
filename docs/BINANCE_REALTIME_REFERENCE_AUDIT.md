# Binance realtime reference, 2026-09-10

Status: new discriminating evidence; runtime correction remains unproven.
The user authorized switching to Binance and its matching TradingView chart.
No runtime code, reference tolerance or earlier OKX capture was changed.

## Frozen capture

Native chart: BINANCE:BTCUSDT spot, ordinary one-minute candles. Script:
`Binance Realtime Update Trace 64`, Pine v6, initial capital 1000000, margins
100/100, calc_on_every_tick true, calc_on_order_fills false, zero fees and
slippage. Capture starts at 2026-09-10 11:25 UTC, with a three-minute bound
and 64-execution limit. All 64 executions occurred in the first minute.
Source SHA-256:
`938cbc33e091eef76769de12e9d4a49650441e84e6b66d83d5eafab5bbcd2ff0`.
Actual editor contents match after newline normalization. Native execution
sequence is contiguous, with no event-buffer truncation or parse failure.
The final 15 fields match the independent chart CSV export exactly.

Independent input uses Binance's public `btcusdt@trade` stream at
`data-stream.binance.vision`, with REST metadata and closed klines from
`data-api.binance.vision`. No account credentials or trading endpoints are used.
Metadata fixes tick size 0.01 and quantity step 0.00001. These public data
surfaces are documented in the [official Binance repository](https://github.com/binance/binance-spot-api-docs/blob/master/faqs/market_data_only.md).

## Input reconciliation

The complete bounded stream contains 6412 trades with no ID gaps. The capture
minute contains 1036 trades, IDs 6671068710 through 6671069745 inclusive.
Their exact decimal OHLCV equals both the exchange REST bar and native chart
CSV: 78020.97 / 78020.97 / 78012.43 / 78012.44 / 2.84335. The REST trade count
also matches. This establishes full trade coverage for the relevant minute.

Of 64 script observations, 63 have nonzero volume matching an exact trade
prefix. The first observation has zero volume. Only 44 observations also
match that prefix's complete OHLC, so a volume boundary alone cannot identify
the native execution's price/timing state in every update. Exchange transaction
timestamps and native execution clocks must not be treated as interchangeable.

## Runtime comparison and discriminators

The retained Windows optimized wheel from `6c31e2b22` compares 975 values at
the unchanged 1e-9 tolerances. Seven values differ, all repeated entry prices
from two fills. Thirty of 32 observed fills match supplied close.

| Execution | Native entry | Supplied close | Previous low | New low | First trade after preceding volume boundary |
| --- | --- | --- | --- | --- | --- |
| 34 | 78018 | 78020 | 78018.01 | 78018 | 78018.01 |
| 62 | 78012.43000000001 | 78012.44 | 78013.15 | 78012.43 | 78013.15 |

The matching volume intervals contain 82 and 81 real transactions respectively.
Native fill prices occur within those intervals, but neither equals their first
transaction. Execution 34 has exact prefix OHLC on both surrounding observations;
execution 62 has a previous close mismatch. This rejects a simple rule that fills
at the first transaction after the prior volume boundary for this capture. It
does not establish when the native broker considered each order eligible.

Both discrepancies equal a newly observed low. Next use directional controls
covering new-high-only, new-low-only and both-extreme updates with long and short
market orders to distinguish native extreme traversal from execution timing.
Do not implement a universal low-price rule from these two cases.

Evidence, replay and reconciliation helpers are retained under
`.local/binance-reference-20260910/`; `manifest.json` hashes the local files.
The original editor script was restored, and browser network observation was
disabled. No trade was placed on Binance, and no script was published.
The old OKX 16/896 and 14/1856 failures remain separate unresolved references.
