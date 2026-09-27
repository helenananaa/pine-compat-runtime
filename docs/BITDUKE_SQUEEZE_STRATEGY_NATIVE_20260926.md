# Bitduke Squeeze Momentum Strategy: v4 native qualification

2026-09-26. Public 99-line Pine v4 strategy (848 boosts when collected):
https://www.tradingview.com/script/5tuGpzpd-Squeeze-Momentum-Strategy-based-on-Indicator-LazyBear-Bitduke/
The source was copied from the TradingView read-only Pine Editor and preserved
unchanged in the ignored local evidence directory.

## Compatibility corrections

- In v4, the second positional `strategy.close(id, condition)` argument binds
  to `when`, not modern `qty`. The analyzer and lowered call use the same
  versioned binding.
- In the public script, `strategy.exit` supplies `profit=1000`, `loss=600`,
  and `trail_points=20` without `trail_offset`. Native fills demonstrate that
  the closer 20-tick favorable target competes with the fixed profit target.
  Local execution now uses the closer target for this v4 form while retaining
  the fixed loss trigger. This changed the previously wrong exit prices.
- A standalone v4 `trail_points` call without an offset or other working
  trigger produces no exit order. This is tested separately and agrees with
  the [TradingView v5 migration guide](https://www.tradingview.com/pine-script-docs/migration-guides/to-pine-version-5/),
  which says such calls had no effect before the v5 compiler rejected them.

The runtime remains host neutral: no market-data acquisition or application
service was added to the core.

## Evidence

On COINBASE:BTCUSD daily history, the unchanged original analyzes with zero
diagnostics (73 supported features, no unsupported features). Local results
have 14 closed trades. The TradingView strategy report also shows 14. All 14
visible report rows were captured by scrolling the report, then compared
programmatically on direction, entry date, entry and exit price to cent, and
net profit to displayed cent. All matched. Order comments, intraday timestamps,
precise quantity beyond the report display, excursions and plot cells were not
qualified in this receipt. Native report export did not create a downloadable
file in this browser session; the preserved DOM row transcription is the native
trade evidence. It is not represented as a CSV export.

Original full local outputs agree between historical, incremental and
realtime-history modes. The latter replays historical data and does not prove
live feed behavior. Targeted tests cover both versioned close binding and v4
unpaired-trail behavior. Full suite and workspace-check logs are kept with the
ignored evidence.

Evidence: `.local/continued-popular-20260926/squeeze-bitduke-original.pine`,
`bitduke-native-trade-rows.json`, `compare_bitduke_rows.py`,
`squeeze-bitduke-daily-final.json`, and `test-bitduke-final.log`.
