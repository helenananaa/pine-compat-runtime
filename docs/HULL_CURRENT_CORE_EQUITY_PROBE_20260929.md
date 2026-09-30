# Hull v4: current-core residual and native equity probe

Superseded for current quantity status by the
[2026-09-30 budget rounding repair](HULL_PERCENT_BUDGET_FIX_20260930.md).
The original quantity residual is repaired; this document retains the evidence
from the preceding core. Three half-tick equity marks remain under investigation.

Follow-up: [isolated first-entry sizing controls](HULL_FIRST_ENTRY_SIZING_BOUNDARY_20260929.md)
reproduce the residual without reversal and bracket the native quantity change
between capital 1157890.70849 and 1157890.70851. A general rounding rule remains
unverified; the original residual is not repaired.

Verified 2026-09-29. This is a diagnostic result, not full compatibility
qualification. The residual in [the source-input expansion](HULL_FX_SOURCE_EXPANSION_20260927.md)
remains: Thma89 / HL2, FX:EURUSD four-hour, closed trade 265 has native
quantity 956,317 and runtime quantity 956,318.

## Current-core replay

The frozen unchanged v4 [Hull Suite Strategy](https://www.tradingview.com/script/Q9OQye4C-Hull-Suite-Strategy/)
source SHA-256 is `235a18e8a69aa6479d446bebc8c023eb0ce1b4a6e293e706cec079293354037d`.
Three original-source settings were rerun on the frozen 21,338 bars, with
direction all, Thma89 and source Close / HL2 / HLC3. Metadata remains
FX:EURUSD, 240, USD, price grid 1/100000, integer quantity, point value 1.
Initial capital is the runtime's default 1,000,000, matching the native
properties retained for the old capture; order sizing is 100% equity.

| Source | Closed trades | Plot cells | Plot/time/price/direction differences | Quantity differences |
| --- | ---: | ---: | ---: | ---: |
| Close | 584 | 42,502 | 0 | 0 |
| HL2 | 578 | 42,502 | 0 | 1 unit on trade 265 |
| HLC3 | 578 | 42,502 | 0 | 0 |

All nine executions terminate with exit zero, empty stderr and no diagnostics.
Three complete mode outputs agree within each setting and equal the retained
September 27 outputs byte for byte. Maximum displayed PnL differences remain
0.00669 / 0.01022 / 0.00614 USD respectively; these are reported residuals,
not treated as passing cent-display checks. Existing partial scope is preserved.

## New native diagnostic capture

The diagnostic source appends only plots of strategy.equity,
strategy.netprofit and strategy.position_size to the unchanged original.
Probe SHA-256: `e1d7d2118f2a43d408b631216cc55c2bbabee804ff95f77337a14ca06d5906d8`.
A current-core run of this probe has exactly the same strategy object as
the current original-source HL2 run. This establishes local noninterference;
the fresh native closed trade rows also agree with the original native capture.

The user's authenticated Chrome Pine Editor compiled the probe. Inputs are
all / Thma / 89 / HL2 with the original date window. Properties explicitly
confirm 1,000,000 USD, 100% equity, pyramiding 1, zero commission/slippage,
infinite leverage, bar-close execution and one-tick order delay. The first
capital entry was malformed by the UI formatter and caused a quantity-limit
error. It was corrected with Select All and individual key entry; only the
subsequent valid run was captured. No script was published.

Native downloads in I:\sys\下载, September 29 UTC+8:

- `FX_EURUSD, 240 (20).csv`, 22:00:06, 21,349 raw rows including forming data.
- `Hull_Suite_Strategy_FX_EURUSD_2026-09-29.csv`, 22:01:39, 578 closed pairs
  and one open pair. Closed records match the old native CSV in every field.
  Open display/excursion/duration differences remain recorded.

Every old timestamp is present. OHLCV matches through the entire quantity
boundary prefix. Outside that prefix, the last old bar (2026-09-25) has a
one-tick high revision and a volume revision: 1.14011 to 1.14012; 21,609 to
21,810. These revisions and the 11 additional rows are retained explicitly;
the fresh full history is not asserted equal to the old full history.

## What the probe establishes

Dates below use UTC+8, matching the native trade report.

| Bar | Date/time | Measure | Native | Current local |
| --- | --- | --- | ---: | ---: |
| 12526 | 2021-01-28 02:00 | Equity before next-open fill | 1157890.7082400022 | 1157890.7082400005 |
| 12527 | 2021-01-28 06:00 | Net profit after closing trade 264 | 157890.70824000204 | 157890.70824000117 |
| 12527 | 2021-01-28 06:00 | Short position | -956317 | -956318 |
| 12575 | 2021-02-09 06:00 | Net profit after closing trade 265 | 163905.94217000215 | 163905.9484600011 |

Across all bars before 12527, maximum equity / net-profit errors are
6.51925802230835e-09 / 3.892637323588133e-09 USD. Position sizes and both
Hull plots match exactly there. The first position and equity differences
occur at bar 12527; net profit first differs materially at bar 12575, after
the differently sized trade closes. Numeric ledger comparison uses 1e-7 USD;
Hull plots use 1e-10, positions are checked directly.

This evidence replaces the earlier hypothesis that an accumulating hidden
ledger difference caused the quantity boundary. Native full-precision ledger
values agree before the differing entry; rounded trade-report cumulative PnL
was insufficient to infer that ledger. The remaining investigation concerns
percent-of-equity sizing at the entry/reversal boundary. It does not yet prove
an equity truncation, fill-price rounding or other specific arithmetic rule.
An isolated native sizing control is required before changing broker semantics.

## Reproducibility and state

Frozen evidence: `.local/hull-fx-fourhour-current-core-20260929/`.
Original/probe sources, executable, patch, original/fresh native CSVs,
Inputs/Properties/export/chart snapshots, screenshot, all mode receipts,
current probe output, ledger audit and current-verification.json are retained.
The verifier checks current source/core/golden pins, native copies, exact
commands, mode hashes, unchanged-source replay, diagnostic noninterference,
prefix OHLCV/plot/ledger equality, first divergences, input settings and cleanup.

```powershell
python .local/hull-fx-fourhour-current-core-20260929/run_modes.py
python .local/hull-fx-fourhour-current-core-20260929/run_probe.py
python .local/hull-fx-fourhour-current-core-20260929/verify.py
```

Terminal logs: `.local/hull-current-core-modes-20260929.log`,
`.local/hull-current-core-probe-20260929.log`,
`.local/hull-current-core-verifier-20260929.log`. Verifier exits zero while
explicitly recording compatibility_qualified=false and the residual mismatch.

Base HEAD `2f9a83a0c333092167c7253571f56ee82a7ec0d0`; core patch
`827e195c3a7db4422bbd657a5634990adb9d8d1db69658f6331df41a97cb2636`;
CLI `584ae27bb25fed15c038c5c55a9356720c1bff90b8db29ce8b403e8a08010f36`.
Current pins match the previously passing VAMA full gate; no new core change
or full-gate run. Temporary diagnostic strategy was removed, and the browser
returned to Coinbase BTCUSD daily with the original two strategies present.
No commit or push. Goal remains active; sizing controls are the next step.
