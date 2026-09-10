# Long stop margin admission: native reference slice

Status: scoped Windows development-artifact qualification passed, 2026-09-10.
Base: `c292e5ef591a43c42260423f4a6b1f92d768d680`.

## Independent observation

Chrome captures on OKX:BTCUSDT standard one-minute candles use original scripts
with source hashes and 1e-9 absolute/relative tolerances frozen before execution.
All scenarios use explicit 100% long/short margin, zero fees/slippage, unit point
value, 0.1 price tick and quantity 0.01. Raw exports remain local under
`.local/tv-goal-20260910/`; the public regression is an original synthetic case.

| Native control | Before repair | Observation |
| --- | --- | --- |
| `b1-capacity` | 399/1555 values differ | At capital 1200, four base exits occur but no new stop entries; both declaration orders are covered |
| `b1-capacity-enough` | 1565/1565 pass | Changing only capital to 2000 permits all eight trades |
| `b1-capacity-earlier-exit` | 408/1570 differ | Moving exits one tick earlier does not restore the rejected stop entries |
| `b1-capacity-admission` | 518/1580 differ | Stops requested while margin is occupied remain absent after the base position closes on an earlier bar; requests after release execute |
| `b1-capacity-threshold` | 384/1590 differ | Capital 1569.15, between close-based and stop-based combined margin thresholds, does not admit the first stop |
| `b1-capacity-threshold-upper` | 1600/1600 pass | Capital 1569.4, above the stop-based threshold, restores the entry |

Different export lengths reflect later chart downloads. Each comparison retains
its own full native input and excludes the final forming bar. These are distinct
reference batches, not additions to the frozen r1 synthetic-input denominator.

## Scoped implementation

An additional long stop requested while an existing long position occupies
margin must fit the combined exposure at its stop price. A future exit cannot
fund that request retroactively. The check runs after existing risk admission
and quantity clamping and uses the established margin diagnostic.

This slice changes only same-side occupied-long stop admission. It does not
change same-price candidate ordering, flat entries, reversals, generic orders,
short stops, limits, stop-limits, or commission formulas. It does not establish
all order-admission rules or private TradingView execution sequencing.

The two original Rust regression cases test rejection before a later exit,
sufficient funds, and successful creation after margin is released. The first
case failed before the repair; the failure log is retained. Initial broad
validation exposed an unnecessary flat-entry diagnostic difference; the repair
was narrowed to occupied-long positions and retained the existing message.
No expected runtime snapshot or native reference value was changed.

## Separate live-update result

The original four-update capture remains failed at 16/896 exit-price values.
The new `live-r2` source changes only its capture-start constant and passes
896/896 values on the unchanged candidate wheel, including four warmup bars,
eight observed updates and both confirmations. Trade entry/exit prices and
quantity also agree; the native trade CSV rounds profit and is not an exact
PnL oracle. Passing this new sample does not erase the original failure.

`scripts/replay_four_update_reference.py` reconstructs both captures from raw
CSV. Its reconstruction of the old execution inputs and all expected values
matches the retained payload exactly. Missing bars/samples, duplicate columns,
changed source hashes, non-finite data and regressive clocks have negative tests.

## Qualification and remaining work

- Complete Windows gate passes 6670 Rust / 715 fresh installed-wheel Python /
  130 tool tests and actual WASM. No runtime goldens were changed.
- A separately retained wheel in a fresh environment passes all six native
  controls: 9460/9460 values. CLI historical, incremental and realtime-history
  agree on complete outputs; generated WASM agrees with complete CLI output.
  Python agrees on all schema-defined common fields and independently matches
  every native plot value. Python-only plot metadata is not a CLI contract.
- Live-r2 remains 896/896 and the previous multi-fill control remains 448/448;
  the original single-entry capture remains failed at 16/896.
- Reports and hashes are in `.local/tv-goal-20260910/qualification.json`.
  These are working-source development builds, not a new committed release.
- Linux and release-profile qualification of this semantic change remain open.
- Continue the unresolved realtime price-input question and other reference
  gaps; no stable release or complete B1/r1 acceptance is claimed.
