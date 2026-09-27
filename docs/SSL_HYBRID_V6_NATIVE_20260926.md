# SSL Hybrid v6 native daily comparison (2026-09-26)

The complete public [SSL Hybrid indicator by Mihkel00](https://www.tradingview.com/script/C3MlAWCw-SSL-Hybrid/) had 16,922 boosts when inspected in Chrome. Its 391-line v6 source was copied unchanged to ignored `.local/continued-popular-20260926/ssl-hybrid-v6-original.pine` (SHA-256 `471dff1bd64611661228ff6d475ac08b354299a663d5e215517b095b70292856`). The default settings are `Full Display`, HMA baseline length 60, JMA SSL2 length 5, and Kijun divider 1.

The original source initially produced 16 repeated analysis errors from `ta.lowest(len / kidiv)` and `ta.highest(len / kidiv)` in the baseline helper. Both operands are integers, but their v6 quotient is `input float`. The core already had a context-specific v5 translation for integer-operand quotients used as the length of these two extrema functions. Extending that exact syntax rule to v6 admits this source and lowers only the length argument through `int()`; the same quotient outside that context remains fractional, and an arbitrary float length remains rejected. The original indicator now analyzes and runs with zero diagnostics or unsupported features. A focused v6 regression tests both extrema and the ordinary quotient.

TradingView exported 300 daily rows of `COINBASE:BTCUSD` to `I:\sys\下载\COINBASE_BTCUSD, 1D (79).csv` (SHA-256 `c27aa1badfe9ed522d1a71a37cc2ae157ca276897c155fad03bfd58b262d421d`). A copy is ignored `.local/continued-popular-20260926/ssl-hybrid-v6-native-daily.csv`. The local run uses 4,283 historical daily bars through 2026-09-25 from `.local/continued-popular-20260926/chart-bars.csv`; the extra native 2026-09-26 bar was forming and has no local counterpart.

On the 299 overlapping bars, all OHLC cells match. `Baseline`, `Upper Channel`, `Lower Channel`, `SSL1`, and `SSL2` each have 299 matching values; the maximum absolute floating error is `1.46e-11`. `Exit Arrows` has 43 matching nonempty cells; `Signal Diamonds` has 299 matching Boolean cells. `+ATR` and `-ATR` are blank on both sides under default settings. The comparison receipt `.local/continued-popular-20260926/ssl-hybrid-v6-comparison.json` has zero mismatches.

## Kijun v2 length context

The exact same public indicator was also configured in TradingView with `Baseline Type = Kijun v2`, `Baseline Length = 61`, and `Kijun MOD Divider = 4`. This executes `ta.lowest(len / kidiv)` and `ta.highest(len / kidiv)` with the fractional quotient `61 / 4 = 15.25`, probing the new contextual integer conversion. The native export is `I:\sys\下载\COINBASE_BTCUSD, 1D (81).csv` (SHA-256 `b258479e9cfc9a34c21843f717130f45aeea4e5060a1c42cb4784da187b2b43d`), copied to ignored `.local/continued-popular-20260926/ssl-hybrid-v6-kijun-native-daily.csv`.

The local run supplies only three host-neutral input overrides: call site 7 `Kijun v2`, call site 8 `61`, and call site 27 `4`. Its 299 overlapping OHLC rows, five populated plot series, 43 Exit Arrow cells, and 299 Signal Diamond cells match the native export with zero mismatches. The receipt is `.local/continued-popular-20260926/ssl-hybrid-v6-kijun-comparison.json`. The CSV also contains a second `SSL1` column from an unrelated strategy already on the chart; the comparator preserves both duplicate headers and uses the first, which belongs to the indicator under test. The active Kijun path is therefore covered, while other settings and forming ticks remain unqualified.

To reproduce from the repository root:

```powershell
$b = '.local/continued-popular-20260926'
target/debug/pine-compat.exe analyze "$b/ssl-hybrid-v6-original.pine" --format json
target/debug/pine-compat.exe run "$b/ssl-hybrid-v6-original.pine" --bars "$b/chart-bars.csv" --chart-symbol COINBASE:BTCUSD --chart-timeframe D > "$b/ssl-hybrid-v6-local-daily.json"
python "$b/compare_ssl_hybrid_v6.py"
target/debug/pine-compat.exe run "$b/ssl-hybrid-v6-original.pine" --bars "$b/chart-bars.csv" --chart-symbol COINBASE:BTCUSD --chart-timeframe D --input-override '7=Kijun v2' --input-override '8=61' --input-override '27=4' > "$b/ssl-hybrid-v6-kijun-local-daily.json"
python "$b/compare_ssl_hybrid_v6.py" kijun
```
