# Ichimoku strategy: native trade and state qualification

2026-09-26. Public v6 Ichimoku 5 Rules Backtest by Yesid_Correa_Cano:
https://www.tradingview.com/script/oG5a3MJU/ (24 boosts at collection).
The unchanged 2,064-line source has SHA256
`846951d911fc034adacdd8cbec4fbc4bc15e6895b1ba8df822c489ea446c62fe`.

## Evidence and scope

Chrome native exports from COINBASE:BTCUSD, 1h, cover 23,970 confirmed bars
beginning 2024-01-01 UTC; the final forming row is excluded. Exact timestamp,
OHLC and volume equality was checked against the local input. Price grid is
1/100 and quantity precision is six decimals. Native trade dates use UTC+08:00.

- All 150 closed trades agree on entry/exit time, entry/exit price, quantity,
  direction and net profit (native profit is rounded to cents).
- Ten appended read-only plots agree across 239,700 cells: conversion/base
  lines, cloud A/B, MACD/signal, active stop/target, bars held and equity.
- Removing those plots from the instrumented result reproduces the original
  complete result. Historical, incremental and realtime-history modes agree.
- A separate immediate-close probe agrees across 95,880 cells on the same
  confirmed bars: before/after position, closed trade count and equity.

Comparators use relative/absolute 1e-9 for state values, 1e-8 for trade prices,
1e-9 for quantity and 0.00501 for rounded native net profit. This receipt does
not qualify drawing appearance, tables, order comments, trade excursions or
live forming updates. Realtime-history is a replay mode, not live validation.

## Runtime correction

`strategy.close` and `strategy.close_all` with `immediately=true` enqueue a
current-tick fill processed after all statements in the current script pass.
Subsequent statements in that pass still observe its pre-fill account state.
Fill-triggered passes retain their execution-price mark. Historical closing
passes continue to avoid the previously identified extra phantom-entry pass.

Before this correction all trades agreed, but 138 time exits produced three
state mismatches each: stop, target and equity (the exit commission). All 414
mismatches now disappear. This demonstrates why trade-only parity is insufficient.

Regression coverage checks both close APIs, position/trade/equity visibility,
partial and short closes, process_orders_on_close, risk-day accounting and
fill scheduling. Seven existing golden snapshots change only plotted values;
all their broker outputs remain identical. Three earlier MACD/Harmonic results
remain fully unchanged. Fractional EMA trades and plots remain unchanged;
labels change, and native drawing qualification remains outstanding. Its three
execution modes agree after the correction.

## Local evidence

Ignored directory `.local/continued-popular-20260926/` contains the original
source, hourly-bars.csv, ichimoku-native-trades.csv, ichimoku-native-state.csv,
ichimoku-state-probe.pine, immediate-close-native.csv, comparison reports,
mode logs and test-immediate-full.log. CSV originals were downloaded through
Chrome into `I:\sys\下载`. Sources/large native exports are not vendored.
