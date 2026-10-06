# KivancOzbilgic SuperTrend Pine v4 native parity (2026-09-25)

The original open-source [SuperTrend indicator](https://www.tradingview.com/script/r6dAP7yi/)
and [SuperTrend STRATEGY](https://www.tradingview.com/script/P5Gu6F8k/)
were taken from their TradingView Source code tabs. They have 82,247 and
25,218 boosts, respectively, in the observed September 25 UI. Both published
scripts declare Pine v4. Their source text is retained in the ignored
`.local/community-coverage-20260923/` directory as
`kivanc-supertrend-v4-20260925.pine` (SHA-256
`3ddd7b5dc61b285ff7c3932255bd0d66ce427a9e17edcca87322b05a384a6ce8`)
and `kivanc-supertrend-strategy-v4-20260925.pine` (SHA-256
`7e343cf8b46cb5aab4eb84ee28dcd76c1af2e2af8461dec02271b6a89133c00a`).
Only HTML nonbreaking spaces were normalized. Neither script needed a source
rewrite, and both analyze with zero diagnostics.

The original scripts were added separately to a `COINBASE:BTCUSD` 1D chart
with their published default inputs. TradingView's chart export was expanded
to December 1, 2014 through September 25, 2026. Each CSV has 4,283 rows; the
last row is the forming September 25 bar. The runtime used the same 4,282
confirmed OHLCV bars, ending September 24. Every matched bar had identical
OHLC values between native and local files. The native indicator export has
SHA-256 `d14ba40274e2d888e49c9fc97e4c00a7f24ecf5401c398a593fadd75ac45958a`;
the strategy plot export has SHA-256
`516bc637e991801a940a543ee03a6f02660fb5372ed568e3bfc77cc3dcb84e2d`.

Both complete scripts ran locally without runtime diagnostics. On each of the
4,282 confirmed bars, all seven exported output fields matched at `1e-8`:
`Up Trend`, `UpTrend Begins`, `Buy`, `Down Trend`, `DownTrend Begins`, `Sell`,
and the untitled `ohlc4` plot. This includes blank positions and numerical
signal levels. The ignored `compare_kivanc_supertrend.py` and
`compare_kivanc_supertrend_strategy_plots.py` produce the retained comparison
JSON reports.

TradingView's strategy trade export (SHA-256
`1e59aedbd1225b0f8bb8298383e9ece220b7299a40ac914f5f5030aa5cb1b059`)
contains 68 closed trades and one open long entry. The local strategy has the
same 68 closed trades and open long. Every closed trade matches by entry and
exit date, long/short direction, entry and exit price, quantity, and net PnL
at the exported CSV precision. The open long matches its July 22, 2026 entry,
price 66,516.17, and quantity 1. The ignored
`compare_kivanc_supertrend_strategy_trades.py` produces a zero-issue receipt.

This comparison covers confirmed daily bars on one symbol, default inputs,
the exported numerical plots and signal levels, and the native trade list.
It does not establish pixel-level fill/color parity, alternative ATR method,
other chart timeframes, or forming-bar updates. The temporary studies and
date-range changes were undone; the research layout was saved and the tabs
closed.
