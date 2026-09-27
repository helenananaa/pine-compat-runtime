# Original WaveTrend [LazyBear]: native qualification

2026-09-26. The unchanged public source at
https://www.tradingview.com/script/2KE8wTuF-Indicator-WaveTrend-Oscillator-WT/
was shown with 60,876 boosts when collected. It has no version directive, so
the local analyzer treats it as implicit v1. The 32-line source is retained in
the local evidence folder.

On COINBASE:BTCUSD daily bars, the original eight plots (five static levels,
two WaveTrend lines, and their difference) match all 2,392 native cells across
299 aligned closed bars. Input timestamps and OHLC agree exactly. The native
export has one additional forming bar that is absent from the local closed-bar
file. The maximum absolute floating-point difference is 2.28e-12; there are
zero NA disagreements. This validates numerical series output only, not color,
style, chart rendering, or live intrabar recalculation.

Reproduction evidence is in `.local/continued-popular-20260926/`:
`wavetrend-lazybear-original.pine`, `chart-bars.csv`,
`wavetrend-native-daily.csv`, `wavetrend-local.json`,
`compare_wavetrend.py`, and `wavetrend-comparison.json`. The original CSV was
downloaded through TradingView's chart export to `I:\sys\下载`. Public source and
market data are not vendored in the repository. This qualification required no
core change.

The next collected candidate is VuManChu Cipher B + Divergences (explicit v4,
523 lines, 24.5K boosts). Its multi-timeframe qualification is recorded in
`docs/VUMANCHU_CIPHER_B_NATIVE_20260926.md`.
