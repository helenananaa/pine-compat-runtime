# SSL Channel Pro v6 native parity (2026-09-24)

The unmodified public `SSL Channel Pro [TradingFinder]` v6 source was loaded
into the signed-in TradingView `BINANCE:BTCUSDT` 1h chart with default inputs.
TradingView compiled it and exported its chart data. The source and market
exports are local evidence only:

| Evidence | SHA-256 |
| --- | --- |
| `.local/community-coverage-20260923/tradingfinder-ssl-channel-pro-v6.pine` | `FF30BEA30D8CE5EA52FBFF44F9596BE860F899B28A54CB0DD80656DD5E33B0C6` |
| `I:\sys\下载\BINANCE_BTCUSDT, 60.csv` | `884B8DFB45668B121472FEE9C7D20D00789002B224899F928140CEF825256BDC` |
| `.local/ssl-20260924/prefixed-hourly-bars.csv` | `B08CE2A19B12BD36AEABCA7B221B8FE84803D8BDAF48B7FBA8FD75DB4A753934` |

The native CSV contains 2,310 rows, with the last forming hour excluded from
the comparison. The remaining 2,309 confirmed hours run from June 20 through
September 24, 2026. A prior TradingView 1h export supplied 1,861 earlier
bars as recursive-state warmup. Its 1,124 overlapping hours had identical
OHLC apart from its former final forming hour; the fresh export takes
precedence for all overlapping hours. Four volume strings differed in the
overlap, including the old forming hour; the fresh export also takes
precedence for volume. The local run has 4,170 bars in total. Its chart
context is `BINANCE:BTCUSDT`, `60`, with a 0.01 price grid. The script's
default 60-minute `request.security` therefore uses the same chart context.

`.local/ssl-20260924/compare.py` aligns the runtime's five plots and two
shape series with the seven consecutive native CSV columns by time and
position. It compares blanks as blanks, then numeric values with
`abs <= max(1e-8, 1e-10 * abs(native))`.

| Series | Compared positions | Nonblank | Blank mismatches | Numeric mismatches |
| --- | ---: | ---: | ---: | ---: |
| SSL Line | 2,309 | 2,138 | 0 | 0 |
| High Boundary Glow | 2,309 | 2,138 | 0 | 0 |
| Low Boundary Glow | 2,309 | 2,138 | 0 | 0 |
| High MA | 2,309 | 2,138 | 0 | 0 |
| Low MA | 2,309 | 2,138 | 0 | 0 |
| BUY Marker | 2,309 | 14 | 0 | 0 |
| SELL Marker | 2,309 | 13 | 0 | 0 |
| **Total** | **16,163** | **10,717** | **0** | **0** |

The largest absolute difference among nonblank numeric pairs was
`5.82e-11`. Batch, incremental, and historical realtime execution produced
identical complete JSON output on the 4,170-bar input. The local source
analysis has zero diagnostics. Each runtime mode retains three distinct
`E_UNSUPPORTED_ALERT_PLACEHOLDER` diagnostics: the published script's six
alert templates cite `Trend Confidence`, `Buy Score`, and `Sell Score` plot
titles that the source does not define. The runtime suppresses events whose
template cannot be resolved, as specified by its alert contract. No native
alert firing or label-text comparison was performed, so this receipt proves
only the seven exported numerical series and local mode consistency. The
temporary study was removed, the Pine Editor buffer and daily chart restored,
and the TradingView layout saved.

## Four-hour confirmation on an hourly chart (2026-09-25)

The same unmodified public v6 source was run on `BINANCE:BTCUSDT` 1h with `Enable HTF Confirmation=true` and `Higher Timeframe=4 hours`; all other inputs remained at their defaults. The native hourly export is `I:\sys\下载\BINANCE_BTCUSDT, 60 (5).csv`, retained as `.local/community-coverage-20260923/tradingfinder-ssl-htf240-native-hourly-v6-20260925.csv` (SHA-256 `69FC75E6093BB8451F85AA5D50B643C308FA938E889CFD59B302E84690CC1594`). Its last, forming hour was excluded. The independent 4h chart export is `I:\sys\下载\BINANCE_BTCUSDT, 240.csv`, retained as `tradingfinder-ssl-htf240-native-4h-v6-20260925.csv` (SHA-256 `E3E3387FA14FD0141BD636E89A11CC60B309735E6ADE4E4F1D6D9DE045072A44`). All 299 complete 4h bars that overlap the hourly input match hourly OHLCV aggregation exactly.

The 299 confirmed native hourly bars include one BUY and two SELL markers. With older hourly warmup and the native 4h request provider, the local unmodified script matches **all 2,093** exported positions across five plots and two marker series; blank positions are checked as blank. The 273 hourly bars shared with the older warmup export have identical OHLCV. The local batch, incremental, and historical realtime results are JSON-equal on the resulting 4,196-hour input, including the same three unresolved alert-template diagnostics described above.

Ignored reproduction inputs, outputs, and `build_tradingfinder_ssl_htf240_20260925.py` plus `compare_tradingfinder_ssl_htf240_20260925.py` are in `.local/community-coverage-20260923/`. The comparator asserts native OHLC, every plot and marker position, signal counts, and equality across the three local execution modes. This qualifies historical 4h confirmation for this symbol, setting, and window; label text, alert delivery, and forming updates remain unverified. The temporary study was removed and the original chart layout saved.
