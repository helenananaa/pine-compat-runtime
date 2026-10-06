# Fractional EMA and strategy fill-time semantics

Local source qualification, 2026-09-26. This extends the
[Reaction/Memory collection](REACTION_MEMORY_NATIVE_PARITY_20260926.md).
It does not qualify a release artifact or native live-tick execution.

## Public source and admission

[Fractional EMA Cross Backtest](https://www.tradingview.com/script/adstHm9N/)
by Yesid_Correa_Cano is an unchanged 2,222-line v6 strategy (52 boosts when
collected). Source SHA-256:
`2fa96e0a312c5d48771430d1e7367483093f65fba6a5892d7229018a444ab477`.
Public sources and downloaded data remain in ignored local evidence, not in
the distributed runtime.

Three table parameters now accept input/series strings: `table.set_position`
position, `table.cell` text_font_family, and `table.cell_set_text_font_family`
text_font_family. An independent native v6 table probe accepts both qualifiers;
runtime and semantic tests retain invalid-constant and numeric-type rejection.
This removes all 76 admission diagnostics in each of Fractional EMA and the
[Ichimoku five-rules strategy](https://www.tradingview.com/script/oG5a3MJU/).
Ichimoku still needs native 1H data and numerical comparison; its intentional
daily-timeframe error is preserved.

## Native strategy findings

Two separate native probes established two scheduler/account errors:

1. A fill-triggered recalculation marks strategy equity and open profit at the
   current execution price. Historical `close` remains the completed bar's
   close. Immediate market closes created during that callback also use the
   current execution price. On 2018-04-25 the probe reports entry 9,645,
   historical close 8,865.98, equity 99,999.90 and open profit zero. Its immediate
   exit fills at 9,645, for a net loss of 0.20 in commissions.
2. An immediate close submitted by the regular historical closing pass fills
   without introducing another script pass. The independent reentry probe
   reports one closing pass and zero flat-position passes. The old runtime
   incorrectly generated an extra entry. Intrabar callbacks, realtime handling
   and explicit process-on-close scheduling retain their separate paths.

The account mark is scoped to fill callbacks and restored on success/error.
Ordinary closing passes continue marking at close. Regression tests cover long
and short positions, immediate exits, subsequent close marks and phantom entries.

## Comparison scope

COINBASE:BTCUSD, daily, 4,283 confirmed bars from 2014-12-01 through 2026-09-25,
price grid 1/100 and quantity precision 6; original default inputs and strategy
settings, including cash commission 0.10 per order. Native downloads are under
`I:\sys\下载`, with evidence copies in `.local/continued-popular-20260926`.

| Comparison | Scope | Differences |
| --- | --- | --- |
| Original strategy closed trades | All 70; entry/exit dates and prices, direction, quantity, net profit | 0 |
| Original strategy plots | 1,337 overlapping confirmed bars, 4 plots, 5,348 cells | 0 |
| Fill account probe | 1,337 bars, 4 plots, 5,348 cells | 0 |
| Closing reentry probe | 1,337 bars, 2 plots, 2,674 cells | 0 |

Native trade profit is exported to cents (absolute tolerance 0.00501); price
tolerance is 1e-8 and quantity tolerance 1e-9. Plot comparison uses relative and
absolute tolerance 1e-9 with exact missing-value positions. Native CSVs (69),
(70), (71) include a forming bar, excluded from comparison. Overlapping OHLC
matches exactly; four volume values changed between downloads. None of these
three sources reads volume, so this is explicitly price-input qualification.
Trade export: `EMA_X_BT_COINBASE_BTCUSD_2026-09-26.csv`.

This is not a 4,283-bar native plot comparison: CSV export retained only the
1,337 confirmed-bar overlap. The strategy report covers the full history.
Native drawing geometry, table contents for the complete strategy, alert
delivery, excursions and forming-tick behavior remain outside this receipt.

## Validation

`cargo test -p pine-builtins -p pine-sema -p pine-runtime -p pine-cli --locked
--quiet`: 6,089 tests passed. Formatting and diff checks passed.
Fractional EMA produces identical complete JSON in batch, incremental and
realtime-history modes. Three previously qualified MACD/Harmonic scenarios
retain identical complete JSON after rebuilding the CLI.

Local reproducibility: `compare_trades.py`, `verify_native_strategy.py`,
`verify_strategy_modes.py`, `test-strategy-final.log`, and
`strategy-evidence-manifest.json` in the evidence directory. No wheel/WASM
distribution rebuild or remote publication is claimed.

Next: obtain real hourly data for Ichimoku; address the TASC indicator's plot
linestyle, display-mask subtraction and optional alertcondition arguments,
then supply independent benchmark data through the existing host-neutral
request contract. Continue collecting additional public indicators/strategies.
