# State-Dependent EMA: EURUSD four-hour exported-history comparison

Date: 2026-09-27. This is local historical indicator evidence, not a live-tick, alert, visual-style, or distribution qualification.

The unchanged public Pine v6 [State-Dependent EMA [BackQuant]](https://www.tradingview.com/script/jdVw4YmG-State-Dependent-EMA-BackQuant/) source has SHA-256 `a4e71b3ab6b9e706eaf934588ea6997128231eab4abab283202234275c309ccc`. TradingView ran the original indicator on `FX:EURUSD`, `240` minutes, with four independently exported input settings. The source and exports are retained under ignored `.local/state-ema-fx-fourhour-20260927/`; the source was not edited for any case.

Each chart CSV has 21,338 rows from 2013-01-02 02:00 UTC through 2026-09-25 17:00 UTC. A request to load back to 1970 returned the same first four-hour row. The first native State-Dependent EMA equals that row's close, showing initialization at the exported history origin. The market was closed at capture, so the last exported row is complete. No bars or comparison cells were removed for warmup. All four exports have identical time/OHLCV fields, verified row by row. The runtime used those bars with host symbol `FX:EURUSD`, timeframe `240`, and price grid `1/100000`.

| Native inputs | CLI overrides | Nonempty numeric cells | Differences |
| --- | --- | ---: | ---: |
| Published defaults (`Combined`, Close) | none | 170,682 | 0 |
| `Efficiency`, Close | `4=Efficiency` | 170,693 | 0 |
| `Efficiency`, HLC3 | `4=Efficiency`, `1=hlc3` | 170,693 | 0 |
| `Sustained Residual`, HLC3 | `4=Sustained Residual`, `1=hlc3` | 170,682 | 0 |

The **682,750** compared positions cover `Ribbon Reference`, `State-Dependent EMA`, `Adaptive Alpha`, `Effective EMA Length`, `Efficiency Ratio`, `Sustained Residual State`, `Variance Ratio`, and `Market State`. Missing values match exactly; finite values use absolute and relative tolerance `1e-9`. The chart's other indicators and hidden color/fill behavior are excluded. The nondefault inputs materially change the native EMA and state sequences. The input call sites (1 for Source, 4 for State Model) were rediscovered by analyzing this exact source revision and are not stable identifiers across source edits.

| Official export | SHA-256 |
| --- | --- |
| `tv-default.csv` | `3a0e410228228fb8c11c5a6f8ea9fa0a863abb749aceda491e73ff593f968628` |
| `tv-efficiency.csv` | `892419dba74a1b118810ac85dd6b94c2cfbec135cbad6d4a8fd67e777168fae3` |
| `tv-efficiency-hlc3.csv` | `8cbba70412c2158e92520b1aeb0f7e0e8f5bf1b4503f7f613c3d717d7df5c923` |
| `tv-residual-hlc3.csv` | `7c30d67f1decce82112a5d2e61b90ae77c359525821303df83c86fce85fec400` |

The retained `compare.py` checks every exported time/OHLCV row and indicator value, writing `comparison.json`. Each case has zero runtime diagnostics. For every setting, `run`, `run-incremental`, and `run-realtime-history` produced byte-identical complete JSON. Their per-case SHA-256 hashes are `e4388049ecde364fb57e6ffb16a34deb745815f3b3a3f50d7f2d716fe1388aa3` (default), `8868ea8a722a32adaa7076a0d9aa1ce263f0135fadd4ddeef298497515bf3e56` (Efficiency), `b50ac0934372929d0cc2fdaed120286c0b25e9b8bcf9f70b4f9ac28dd51be7ad` (Efficiency/HLC3), and `149d3f01eada12deb09d328724e15cded657df290c8756eb34c20854ac9dc9f6` (Sustained Residual/HLC3). This slice required no runtime code change.
