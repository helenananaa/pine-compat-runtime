# Percent-of-equity budget rounding repair

Subsequent evidence: [independent half-tick marking repair](HULL_HALF_TICK_MARK_FIX_20260930.md)
removes the three equity residuals recorded below. This receipt retains its
original candidate and gate pins; monetary display differences remain open.

Verified 2026-09-30. The isolated quantity defect is repaired and the full
release gate passes. Full Hull compatibility remains incomplete because
three equity marks and native displayed monetary fields still differ.
This follows the [independent native controls](HULL_SIZING_MAGNITUDE_CONTROLS_20260929.md).

## Core change and native evidence

`percent_of_equity_order_qty` now rounds the percentage cash budget to ten
significant decimal digits before commission reservation, price division and
contract-grid truncation. Both actual entries and `strategy.default_entry_qty`
use this shared calculation. Explicit order quantities keep their existing path.

Nine frozen official controls now match across batch, incremental and
realtime-history execution: 27 complete outputs, with per-source byte equality.
They cover million-dollar and hundred-thousand-dollar budgets, both sides of
the observed boundaries, 100%/25% equity, both entry directions, v4 actual
entries and the v5 helper. The helper CSV has 299 observed rows plus one forming
bar with all probe fields missing; its exact missing position is checked.

A new Rust regression uses the independently exported signal/fill bars and
eight native entry boundaries plus the v5 helper. Its formerly failing cases
include both upward and downward quantity corrections.

## Full-history Hull replay

The unchanged published v4 Hull source and all 21,338 frozen bars were replayed
for all / Thma89 with Close, HL2 and HLC3. Each configuration has three complete
mode outputs with equal hashes. Both Hull plots match at every observed and
missing position. Closed-trade time, direction, price and quantity checks pass.
All three surviving positions also match native quantity, entry time, signal
and average price.

| Source | Closed trades | Quantity differences | Changed quantities | Maximum native displayed PnL difference USD |
| --- | ---: | ---: | --- | ---: |
| Close | 584 | 0 | None | 0.00669 |
| HL2 | 578 | 0 | Trade 265: -956318 -> -956317 | 0.00560 |
| HLC3 | 578 | 0 | None | 0.00614 |

Close and HLC3 complete outputs remain byte-identical to the prior qualified
core. HL2 output SHA-256 is
`0844344b4cc032985338ff2936a1989b7872d1cdfeeb33205398c65b85e79da9`.
Its only changed closed-trade quantity is the original defect on trade 265.

The full-precision diagnostic probe now matches native net profit on the entire
frozen window within 3.892637323588133e-09 USD. Position sizes match exactly on
every bar. This includes the period after the formerly different trade closes.

Two complete SSL batch guards also remain byte-identical: Coinbase BTCUSD
weekly Kijun/RMA20/second Hull55/dots/HL2, and FX:AUDUSD four-hour VAMA30/HL2.
These exercise fractional crypto quantities and integer forex quantities in
the unchanged explicit-quantity strategy path. Other historical receipts keep
their original source pins and are not relabeled as current-core replays.

## Remaining equity and monetary differences

Three equity plot values differ by approximately 4.91495 USD, while their
net profit and position size agree. All other equity values match within
1e-7 USD. The differing bars share position -982990 and closes on half ticks:

| Bar | Timestamp ms | Raw close | Native equity USD | Local equity USD |
| ---: | ---: | ---: | ---: | ---: |
| 16302 | 1688029200000 | 1.0881150000000002 | 1070398.09994 | 1070403.0148900002 |
| 16305 | 1688072400000 | 1.087225 | 1071272.9610400002 | 1071277.8759900003 |
| 16306 | 1688086800000 | 1.086995 | 1071508.8786400002 | 1071503.9636900004 |

An offline calculation using `round-half-away(close / mintick) * mintick`,
preserving floating-point division, reproduces all three native equity values
exactly. Multiplication by the reciprocal has a different midpoint result on
the third bar, so the arithmetic sequence matters. This is a retained inference
for the next independent mark-price control, not an implemented marking fix.

The displayed trade PnL residuals in the table remain reported, rather than
treated as passing cent-display checks. Neither this quantity repair nor the
release gate establishes arbitrary-script or realtime tick parity.

## Release checks and reproducibility

`scripts/verify.ps1` exits zero: formatting, clippy with warnings denied,
workspace tests (including 1,985 runtime and 242 CLI unit tests), structural
checks, 130 tool tests, host guards (940 registered snapshots; 591 required
runtime plus five legacy Python/WASM assertions), actual Node/WASM smoke,
and a freshly built/installed wheel with 774 Python tests.

Evidence directory: `.local/hull-sizing-candidate-20260930/`.
It retains the executable, all isolated/full-history outputs and receipts,
native chart/trade fixtures, explicit open-position audit, remaining mark-price
audit, two SSL guard outputs, release log/receipt and final source/artifact hashes.
`verification.json` explicitly records quantity_repair_qualified=true and
full_compatibility_qualified=false.

```powershell
python .local/hull-sizing-candidate-20260930/freeze_verify.py
```

Replay helpers are `run_candidate.py`, `run_full.py`, `run_guards.py` and
`audit_open.py`. Replaying rewrites outputs; the final verifier detects any
change to the frozen source, executable, outputs or gate log.

HEAD: `2f9a83a0c333092167c7253571f56ee82a7ec0d0`.
Current core patch: `bd0a6db291354ef3ec4bed50e1ba67dcedbcbb6003ac5b97276c0f2153fd0727`.
CLI: `fe5545fb23438b091f586e7b3df011709e9716906b4b6fb5e657fc713e5f265d`.
Release log: `a186241aec60fcc832bb6bbe2cf963ef7c539ea107d3033869b6d9f7da4be88d`.
Existing dirty work was preserved. No commit or push. Goal remains active;
the next defect to isolate is equity marking on half-tick input prices.
