# Committed-core script acceptance — 2026-09-30

This receipt remains pinned to the core below. The subsequent working-tree
request-refresh patch has a separate
[offline intrabar/MTF receipt](INTRABAR_MTF_ACCEPTANCE_20261001.md); this native
matrix does not automatically qualify that patch.

Core commit: `5c3ab3b628b626619c34481d23f30dd002e362fe`. The runtime source and
Cargo lockfile were unchanged throughout this qualification. The working-tree
changes add acceptance tooling and status documentation only.

The frozen core set comprises five complete scripts: Hull, UT Bot, SSL Hybrid,
official RSI and official Pivot Points Standard. There are twelve unchanged
complete-script settings, one read-only Pivot observer and two broker controls,
for fifteen matrix cases. Legacy v4 preview/emulation policy remains unchanged.

Every case passes complete public-output comparison across direct Rust API,
CLI, a freshly installed Python wheel and an actual generated WASM module under
Node. Each CLI case runs batch, incremental and realtime-history modes. These
are **90 primary complete outputs**, with zero cross-surface mismatches at
absolute tolerance `1e-9`, relative tolerance `1e-12`; keys, array lengths,
booleans, strings and missing values are checked separately. Additional RSI,
Pivot and SSL observer runs supplement this matrix.

All artifacts are Windows x86_64 **debug** builds from the selected core.
Runtime schema is 9 and render metadata version is 1. Linux, optimized
distribution, long-session resource budgets and general native live Tick
behavior remain unqualified by this round.

## Frozen matrix and observable native evidence

All date ranges below are UTC bar dates, inclusive. The machine-readable
[results](CORE_SCRIPT_ACCEPTANCE_RESULTS.json) retain exact first/last
millisecond timestamps, source and payload hashes, call-site input overrides,
output hashes, artifact hashes, execution modes and unresolved output categories.
The [plan](CORE_SCRIPT_ACCEPTANCE_PLAN.json) freezes 75 files; 46 Pine/CSV input
files were also checked against earlier capture manifests. No TradingView data
was recaptured and no old reference, comparator or execution receipt was edited.

| Script / settings | Symbol / timeframe | Confirmed data | Native evidence on the current core | Remaining category |
| --- | --- | --- | --- | --- |
| Hull THMA 89, Close, both directions | FX:EURUSD / 240 | 21,338 bars; 2013-01-02–2026-09-25 | 42,502 numerical plot positions; 584 closed trades with matching time, price, direction and quantity | Displayed PnL max difference 0.006690 USD |
| Hull THMA 89, HL2, both directions | FX:EURUSD / 240 | Same frozen range | 42,502 plot positions; 578 closed trades; no quantity mismatch | Displayed PnL max difference 0.005600 USD |
| Hull THMA 89, HLC3, both directions | FX:EURUSD / 240 | Same frozen range | 42,502 plot positions; 578 closed trades; no quantity mismatch | Displayed PnL max difference 0.006140 USD |
| UT Bot default | FX:EURUSD / 240 | 21,338 bars; 2013-01-02–2026-09-25 | 42,676 signal cells; 2,459 closed trades and final open entry match | Native live Tick / visuals |
| UT Bot sensitivity 2 | FX:EURUSD / 240 | Same frozen range | 42,676 signal cells; 1,074 closed trades and final open entry match | Native live Tick / visuals |
| UT Bot ATR 14 | FX:EURUSD / 240 | Same frozen range | 42,676 signal cells; 2,471 closed trades and final open entry match | Native live Tick / visuals |
| UT Bot Heikin Ashi request input | FX:EURUSD / 240 | Same chart range; frozen HA provider | 42,676 signal cells; 1,815 closed trades and final open entry match | Native live provider/Tick behavior |
| SSL VAMA 30, HL2 | FX:AUDUSD / 240 | 21,344 bars; 2013-01-02–2026-09-28 | 149,248 nonblank series positions; 3,890 closed trades; 2,112 explicit exits; five open legs | Native reversal transaction order list / visuals |
| SSL Kijun 30, CF/RMA 20, second Hull 55, dots, HL2 | COINBASE:BTCUSD / 1W | 615 bars; 2014-12-01–2026-09-21 | 4,700 nonblank series positions plus nine cross dots; 60 closed trades; 34 explicit exits; five open legs | Native reversal transaction order list / visuals |
| Official RSI default | BINANCE:BTCUSDT / 1M | 109 bars; 2017-08-01–2026-08-01 | 218 native reference positions; zero mismatches | Native visual gradient parity / broader options |
| Official RSI SMA+BB and divergence | BINANCE:BTCUSDT / 1M | Same frozen range | 852 comparable native positions including offset-aware outputs; zero mismatches | Native visuals / broader options |
| Official Pivot Traditional/Auto, original | BINANCE:BTCUSDT / 1M, requested 12M | 109 chart bars; 2017-08-01–2026-08-01; frozen annual snapshot | Original output agrees across surfaces; 99 labels and 99 lines; paired observer strips to the original output | Native line/label geometry / live provider / broader options |
| Pivot read-only observer | Same chart and annual provider | Same frozen range | 3,161 native reference positions; zero mismatches | Observer is supplementary evidence, not another adopted script |
| Percent-fee partial-close reversal control | FX:EURUSD / 240 | Eight bars; 2023-06-29–2023-06-30 | 232 plot cells and three closed allocations; local reversal transaction receipt 105,000 | Native order list and live alert capture |
| Per-contract-fee partial-close reversal control | FX:EURUSD / 240 | Same frozen range | 232 plot cells and three closed allocations; local reversal transaction receipt 105,000 | Native order list and live alert capture |

The Hull monetary residual is explicitly open. Curve/trade/quantity agreement
does not qualify full monetary display parity, and this round does not establish
that the residual is rounding alone. UT Bot monetary comparisons preserve the
original native display-precision tolerance; raw decimal equality is not claimed.

## SSL comparator repair and preserved failures

Both frozen SSL comparators matched a native open-trade **leg size** against a
runtime entry-order **transaction size**. The reversal receipt repair changes
that transaction field, so both old checks fail on this core despite unchanged
closed allocations and matching final exposure. Their scripts, failed reports
and stderr remain in the evidence directory.

The new comparator runs the old check first, then adds a read-only Pine observer
of `strategy.opentrades`: count, size, entry ID, entry price and entry time.
All five open legs in each case match the independent native trade CSV. Removing
only the observer plots reproduces the entire original script output, including
broker state and all original drawings. Signed leg sizes also sum to the final
position. The weekly case maps native UTC dates to the frozen chart bars and
retains the confirmed boundary; the intraday case uses the recorded UTC+8 trade
timestamps. A date-format failure during comparator development is retained too.

This establishes open-leg evidence without using a native trade CSV as an order
list. New native reversal order/alert capture remains a separate unverified field.
No runtime repair or reference-tolerance relaxation was needed.

## Validation, artifacts and reproduction

The fresh canonical `scripts/verify.ps1` gate passes rustfmt, clippy, workspace
and integration tests, 1,989 runtime unit tests, 242 CLI tests, 130 tooling tests,
the 940-snapshot parity guard with 591 required runtime and five legacy-analysis
assertions, actual Node/WASM smoke tests and 774 installed-wheel Python tests.
The retained freshly built wheel separately passes those 774 tests. After adding
three false-positive comparator regression tests, all **133 tooling tests** pass.
The Rust probe uses the workspace-locked dependency versions. Its first failed
build and both old SSL comparator failures are retained separately from success.

Evidence: `.local/core-requalification-20260930/`. It includes source/artifact
provenance, all commands and output hashes, immutable native inputs copied into
new case directories, original failure reports, successful observer checks,
before-resume reports, full gate logs, a reference-chain receipt, an evidence
manifest and the final audit. The final matrix has no unresolved execution or
comparison-tool failure; the semantic/coverage gaps above remain explicit.

The reusable entry points are:

```powershell
# Requires the frozen local corpus, MSVC/Rust/maturin/Python and Node on PATH.
powershell.exe -NoProfile -ExecutionPolicy Bypass -File scripts/build_core_script_acceptance.ps1 `
  -OutputDirectory E:\projects\pine-interpreter\.local\core-acceptance-next `
  -Python C:\Users\MECHREVO\AppData\Local\Programs\Python\Python310\python.exe

# Re-audit existing complete outputs and rerun native comparisons on identical pins.
python scripts/requalify_core_scripts.py --artifacts .local/core-requalification-20260930 --resume
```

The builder requires a new evidence directory and a committed, unchanged core.
The plan is pinned to the selected commit; changing the target requires an
explicitly reviewed new plan. Missing local references are errors, not passes.
This local acceptance set is not a portable release corpus or distribution.

## Current status versus historical evidence

This matrix is the current-core acceptance index. Earlier receipts keep their
own source, artifact, input and platform pins. SMC and Double Tap have later
execution/native evidence in their [community record](COMMUNITY_SCRIPT_COVERAGE_20260923.md);
they are not requalified here and must not be described as generally unable to
run from their older admission failures. Earlier Pivot admission blockers are
historical; current Pivot runs and passes the named paired comparison above.

Next acceptance work is native intrabar/multi-timeframe combinations and the
already [frozen resource plan](PRODUCT_STANDARD_CANDLE_RESOURCE_PLAN.json), with
current Linux and optimized artifact qualification tracked separately. Market
data acquisition, reconnection, persistence and scheduling remain host-owned.
