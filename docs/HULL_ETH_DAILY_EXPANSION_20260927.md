# Hull Suite Strategy: ETHUSD daily parameters

Date: 2026-09-27. This is a local historical comparison, not a live-tick or distribution qualification.

The unchanged public Pine v4 [Hull Suite Strategy by DashTrader](https://www.tradingview.com/script/Q9OQye4C-Hull-Suite-Strategy/) source (SHA-256 `235a18e8a69aa6479d446bebc8c023eb0ce1b4a6e293e706cec079293354037d`) was run on `COINBASE:ETHUSD`, `1D`. This expands the earlier [BTCUSDT default-direction comparison](LEGACY_V4_STRATEGY_AUDIT_20260923.md) to another symbol, both trade directions, and the EHMA variation. The original source is retained at `.local/hull-eth-daily-20260927/hull-suite-strategy-original.pine`; it was not edited to make the cases pass.

The chart export contains 3,780 rows from 2016-05-23; the 2026-09-27 forming bar was excluded. All 3,779 confirmed rows through 2026-09-26 were passed to the runtime, with no comparison warmup exclusion. The chart's initial capital was set to **1,000,000 USD** to match the runtime default. Other strategy properties remained at the source/default values: 100% equity order size, pyramiding 1, zero commission and slippage, on-bar-close execution, and default bar detail. The local host supplied the exported OHLCV, `COINBASE:ETHUSD`, `1D`, USD, price grid `1/100`, and quantity precision 4.

| Inputs | Native closed trades | Runtime closed trades | Entry/exit date, price, direction differences | Numeric plot values compared | Plot differences |
| --- | ---: | ---: | ---: | ---: | ---: |
| `Strategy Direction=long`, `Hull Variation=Hma` | 74 | 74 | 0 | 7,436 | 0 |
| `Strategy Direction=all`, `Hull Variation=Hma` | 148 | 148 | 0 | 7,436 | 0 |
| `Strategy Direction=all`, `Hull Variation=Ehma` | 136 | 136 | 0 | 7,436 | 0 |

The native CSV also has one open position in each case; it is excluded from closed-trade comparisons. Plot comparison checks `MHULL` and `SHULL` from the first chart bar, including missing-value positions, with absolute tolerance `1e-8`. Native prices are displayed to cents; the comparison uses half-cent tolerance. Native quantities and PnL are display-rounded: the largest quantity relative difference is `1.57e-9`, and the largest displayed PnL relative difference is `2.26e-6`. These figures do not assert bit-exact native broker internals.

The local comparison is reproducible with `.local/hull-eth-daily-20260927/compare.py`; its machine-readable receipt is `comparison.json`. The official export hashes are:

| Evidence file | SHA-256 |
| --- | --- |
| `tv-chart.csv` (Hma) | `84521dc8ad0e33816c31eb9e4a4eba5287d3db77e6332603db608134770fc2dc` |
| `tv-chart-ehma.csv` | `930d156c3bdd4701b5a13e75e4ff612b2985ad8d27388bbcb404d6226a8a9758` |
| `tv-trades-long.csv` | `137d4083ab3308788c98d0f2b3c5b613d8b2817f8f3dfeea0bf9b50e3bd01248` |
| `tv-trades-all.csv` | `48e652141c2cd73e382145473cfd297f37670c1143544cf7279008c3722402a6` |
| `tv-trades-ehma-all.csv` | `0306981d6fe0ba0cf915f6be65468daf2a241a56f984540cabdcde0a4769f9b4` |

The runtime previously rejected the original source's unselected EHMA function: a v4 `ema` length of the form `input int / const int` inferred as `input float`. The v4 EMA length context now truncates that integer-operand quotient, while ordinary division retains its fractional result. The focused semantic regression test covers the same expression pattern. No host integration was added.

For `Ehma/all`, `run`, `run-incremental`, and `run-realtime-history` produced byte-identical full JSON (SHA-256 `6ea14494ffd694087fe31bde543313c092cd5a757015d6c34975c70fe8c7064c`). This checks confirmed historical bars, not live ticks, chart color/fill appearance, or other symbols and timeframes.
