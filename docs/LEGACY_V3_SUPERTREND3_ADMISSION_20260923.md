# Super Trend 3 v3 admission, 2026-09-23

The complete open-source [Super Trend 3 by mhannigan](https://www.tradingview.com/script/bwF5bCek-Super-Trend-3/)
was inspected through TradingView's Source code tab. Its 62-line Pine v3
source is retained only in ignored
`.local/super-trend-3-v3-20260923/original.pine` (SHA-256
`4AD651018D4D3D9CD5C28B3A1DD6B0DECDC1F3FB887D7B43C4917CA415EF8047`);
the public source is not redistributed in the repository.

Before this change, CLI analysis returned two `E_UNSUPPORTED_FEATURE`
diagnostics on the higher-timeframe `security` expressions containing `hl2`
and `atr(Pd)`. Reduced probes showed that `atr(Pd)` and input captures were
admitted, while `hl2` alone failed in provider context. The request analyzer
now admits the four synthetic OHLC sources `hl2`, `hlc3`, `hlcc4`, and `ohlc4`
there. A runtime fixture verifies that `hl2 - Factor * atr(Pd)` evaluates on
host-provided higher-timeframe bars, with input captures and historical
alignment. The unchanged complete script now analyzes as executable v3
strategy with zero diagnostics.

This is an **admission** result. The complete script also uses a 120-minute
requested stream, stop/profit exits, and chart-dependent settings. Its full
plots and trade list have not yet been compared with native TradingView output
on a matched chart and request dataset. The host remains responsible for
providing those bars; the runtime does not acquire market data.
