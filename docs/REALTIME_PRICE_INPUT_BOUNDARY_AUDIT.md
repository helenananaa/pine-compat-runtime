# Remaining realtime price evidence boundary

> Historical capture/progress record. Later a2a1ba5fb optimized wheels pass the
> original and subsequent named realtime references on Windows and Linux.
> See [final acceptance](TRADINGVIEW_GOAL_ACCEPTANCE.md) and the
> [current delivery ledger](DELIVERY_ROADMAP.md). Failed results below remain
> evidence of earlier builds, not current unresolved failures.

2026-09-10 follow-up: the user-authorized Binance source supplied continuous
trade data and directional native controls. The scoped market-order repair in
`REALTIME_MARKET_EXTREME_AUDIT.md` passes both original OKX failures unchanged
(896/896 and 1856/1856). The access limits and unresolved state below describe
the earlier investigation; they are no longer the current disposition of
these named comparisons. Two-sided range and price-condition semantics remain
outside the new qualification.

Status: unresolved; additional trade-level input or a discriminating native
control is required. The user currently has no additional data/API/proxy or
Ultimate access. No tolerance, frozen reference or fill-price heuristic changed.

## Captures retained

- Original four-update reference: 16/896 repeated exit-price differences.
  Native 78246.1 versus the supplied observed close 78246.2 remains failed.
- A separate unchanged-code repeat (`live-r2`) passes 896/896 values.
- Five new round trips (`live-r3-quotes`) compare 1856 values and retain
  14 repeated entry-price differences: native 78175.2 versus supplied close
  78175.3. Native closed-trade counts advance from 1 through 5. The actual
  editor source was checked against the frozen source before export.

The new price mismatch occurs between these observations on 2026-09-10 UTC:

| Script execution clock | Bar open/high/low/close | Cumulative volume |
| --- | --- | --- |
| 09:26:00.894 | 78175.3 / 78175.3 / 78175.3 / 78175.3 | 0.03618378 |
| 09:26:01.719 | 78175.3 / 78175.3 / 78175.2 / 78175.3 | 0.05992577 |

Browser-received public quote and primary chart updates were retained without
event-buffer truncation or parser failures. These show aggregate updates and
quote snapshots; they do not establish every intervening exchange transaction.
An independently downloaded native one-second bar at 09:26:00 contains
O/H/L/C 78175.3/78175.3/78175.2/78175.3 and volume 0.05992577. Thus even the
one-second archive combines the relevant price changes in this case.

## Why a price rule is not inferred from this sample

The unobserved volume between the two observations is 0.02374199. For example,
these **synthetic counterexamples**, not actual exchange records, both reproduce
the observed next OHLCV summary:

| Possible intervening price sequence | Corresponding positive quantities |
| --- | --- |
| 78175.2, 78175.3 | 0.01187099, 0.01187100 |
| 78175.3, 78175.2, 78175.3 | 0.00791399, 0.00791399, 0.00791401 |

Their first transaction prices differ. Aggregated OHLCV alone therefore cannot
identify the first underlying transaction. This is not proof that missing
transactions are the sole cause: native handling of aggregate observations or
execution timing remains a competing explanation. A guessed low-first path,
unconditional tick-floor operation, or a supplied expected fill price would
not resolve that uncertainty.

## Available access and next discriminating evidence

The current Premium account exposes one-second charts but requires Ultimate
for tick charts. No subscription was changed and no restricted endpoint was
used. The public OKX history-trades API is documented by the exchange, but
Windows/Chrome DNS lookup failed for its main host and a WSL request timed out.
The alternative official documentation host also failed local DNS resolution.
No exchange trade data was obtained or represented as native TradingView input.

The next useful input is an independently obtained, ordered BTC-USDT trade
sequence for the original 00:32 UTC and new 09:26 UTC windows. Its volumes and
OHLC must first reconcile with the captured TradingView observations. Only
then can it distinguish missing-event input from a runtime fill-rule defect.
The existing broker and source rollback fixes are not reverted or weakened.

Raw material is under `.local/tv-goal-20260910/live-r3-quotes/`, including
source/plan, replay report, native trade CSV, selected market events, clock
calibration, access limitation, and one-second data. Earlier failures remain
in their original evidence directories. B1 private ordering and the original
r1 synthetic denominator retain their separate unverified status.

The subsequent 64-execution trace contains 32 matching native fill prices and
exposes a separately repaired mid-bar opening-context gap. It has no equivalent
same-close/new-extreme case and cannot decide between the price explanations.
Both retained failed captures still reproduce on the final Windows optimized
wheel from `6c31e2b22`: 16/896 and 14/1856 respectively. The user confirmed that
no additional data source, proxy or Ultimate access is currently available.
See `REALTIME_OPENING_CONTEXT_AUDIT.md` for the independent context repair.
