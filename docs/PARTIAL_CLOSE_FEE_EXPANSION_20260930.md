# Partial-close commission coverage

Verified 2026-09-30 against new official Chrome/TradingView captures. The
current qualified core matches three additional complete probe scripts in
nine mode outputs. This round expands native evidence for partial closes,
cash-per-contract fees and v6 percentage fees. The expansion Goal remains active.

## Source, history and effective properties

The probes use FX:EURUSD / FXCM / 240 minutes, 2,000,000 USD initial capital,
two entries, zero margin requirements, zero slippage, bar-close execution,
and one-tick order delay. All quantities are explicit. Native v6 Properties
display a 100-percent default size, while v5 displays fixed quantity 1;
these defaults do not determine the probes' explicit quantities and this
receipt does not qualify omitted-size v6 behavior.

| Case | Version / direction | Commission |
| --- | --- | --- |
| v5-cash-short | v5 short | 10 USD per order |
| v5-contract-long | v5 long | 0.0001 USD per contract |
| v6-percent-short | v6 short | 0.1 percent |

E1 enters 100000 units at 1688029200000 ms / 1.09143. E2 enters 300000 at
1688043600000 / 1.08812. `strategy.close("E1", qty_percent=25)` closes
25000 E1 units at 1688058000000 / 1.08708. Later `strategy.close_all()`
closes the remaining 75000 E1 and 300000 E2 units at 1688086800000 / 1.08723.
The three resulting closed allocations retain their original entry times
and identifiers. Their signed quantities are checked independently against
the native trade CSV direction and unsigned quantity columns.

Raw official chart and trade downloads, all submitted sources, full UI
states, all three loaded Properties dialogs, shared EURUSD/240 UNIX export
settings, and screenshots are preserved under
`.local/partial-fees-native-20260930/`. Early cash Properties/export snapshots
were taken before their dialogs loaded and are explicitly named
`*-before-dialog.txt`; they are not effective-setting evidence. The loaded
cash Properties capture and the loaded contract/percent export settings
provide the effective records. The cash CSV's Unix timestamps, symbol/timeframe
file identity and exact OHLCV extraction are verified independently.

Eight native historical OHLCV rows are replayed, from 1688014800000 through
1688115600000 ms. Full original exports remain intact; the local execution
starts at the first signal bar, before any exposure exists. No forming row
is replayed. Local metadata uses USD, price grid 1/100000, integer quantity
precision and point value 1. No market-data integration is introduced into
the runtime core.

## Independently observed allocations

For fixed cash fees, E1's initial 10 USD entry fee allocates 2.5 USD to the
first close and retains 7.5 USD on its remaining 75000 units. The first
25000-unit close pays its own full 10 USD exit fee. The later single
375000-unit `close_all` pays one 10 USD exit fee: 2 USD to E1 and 8 USD to E2.

| Case | Closed commissions, allocation order | Final native net profit USD |
| --- | --- | ---: |
| v5-cash-short | 12.5 / 9.5 / 18 | 650.75 |
| v5-contract-long | 5 / 15 / 60 | -770.75 |
| v6-percent-short | 54.46275 / 163.3995 / 652.605 | -179.71725 |

After the partial close, open E1 commission, size, individual profit and
profit percentage reflect the remaining allocation. Individual profit
includes remaining entry commission and the fee for an estimated complete
exit at the marked price. Total open profit remains gross price profit;
net profit accounts for paid entry fees on remaining exposure. Native closed
profit percentages use the allocated entry fee in capital. Before any close,
the tested closed profit / percentage / commission / size fields return zero.

All of these values match the existing ledger allocation and the preceding
commission-profit repair. This round requires no new runtime semantic change
and no golden snapshot update.

## Complete output verification

Each source exports 29 plots: equity, total/individual profit and percentages,
position and average price, open/closed counts and commissions, plus remaining
open quantities and all three closed allocation quantities. Every plot value
and missing state is compared at every native bar. There are no exceptions
for zero versus missing values.

Batch, incremental and realtime-history each pass all three sources.
Complete JSON outputs are byte-identical within each case. All nine closed
allocations across the three independent cases agree on identity, direction,
quantity, entry/exit time and price. Maximum monetary error is below 2.4e-10
USD, percentage error below 3.3e-14 percentage points and fill-price error
below 2.3e-16. Native two-decimal trade PnL differs by at most
0.004999999983 USD; the unrounded native chart-profit fields match within
floating-point precision.

The preceding full gate was revalidated against the unchanged core and
snapshot pins before this round's documentation update. Its 1,988 runtime,
242 CLI, actual Node WASM and 774 installed-wheel tests remain evidence for
that same core; no new gate run is claimed here. The twelve preceding full
Hull / equity / SSL outputs retain their hashes and the frozen receipts.

- Current HEAD: `2f9a83a0c333092167c7253571f56ee82a7ec0d0`
- Unchanged core patch SHA-256: `be5cf60745bc497967b9b1f7499de3b0b35ba62436443d76872e88e873076e0e`
- Reused qualified executable SHA-256: `b714dd49ff77f588bbc49e387b2d36559d3f9b50f155cbb018f9d71dce33bc66`
- Revalidated gate log SHA-256: `0a07bdc487e6b5455ee7ca80d071809d612c5cc3808a28901abf05d9dfe5f82a`

The new immutable verifier pins the core, original official captures,
extracted fixtures, mode receipts and reused executable/gate provenance.
It rechecks all 29 fields and all three allocation trade records per case:

```powershell
python .local/partial-fees-native-20260930/freeze_verify.py
```

The exact original editor draft was restored and compared through the
clipboard; the temporary strategy was removed, Coinbase BTCUSD daily restored,
and this round's sampling tab closed. Restoration screenshot and UI state
are retained. Cleanup of the older stale sampling tab remains unverified.

## Next expansion and remaining boundaries

Continue with complete public scripts on additional unit-point-value assets
and periods, retaining independent native signals and trades. Non-unit
point-value profiles are explicitly rejected by the current CLI and remain
an unsupported core capability; no futures compatibility is claimed.
Limit/stop partial fills, repeated partial closes and reversal fee allocation
need independent controls beyond this market-close experiment. Full Hull
monetary display residuals remain open. The new v5/v6 probe receipts do not
establish arbitrary-script or TradingView live-tick compatibility.
