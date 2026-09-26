# Order Blocks & Breaker Blocks v5 compatibility check (2026-09-25)

The public [Order Blocks & Breaker Blocks [LuxAlgo]](https://www.tradingview.com/script/piIbWMpY-Order-Blocks-Breaker-Blocks-LuxAlgo/) source is a 202-line Pine v5 indicator. Its exact published source is retained only in the ignored local evidence directory as `luxalgo-order-blocks-breaker-v5-20260925.pine` (SHA-256 `21DB6144FD7A7FFDF72E788B5561D7B5D019312C85E55207943128A91081BA6B`). The publication attributes the source to LuxAlgo and carries a CC BY-NC-SA license; it is not copied into the repository.

The original source revealed three missing language behaviors:

- A method's receiver declares a type, while subsequent parameters may omit it, such as `method display(ob id, css, break_css)`.
- User methods can use the built-in `color` type as their receiver, as in `method notransp(color css)`.
- Destructuring a function-returned UDT tuple must preserve each element's UDT identity for field access and method calls.

The parser, analyzer, and method-call lowering now handle these cases. A focused runtime test exercises all three. The exact original source analyzes with zero diagnostics and runs on 4,282 retained confirmed `COINBASE:BTCUSD` daily bars with default inputs. The local final state contains 9 boxes, 18 lines, and no labels or runtime diagnostics.

To obtain native numeric outputs, a local-only derivative appends six `plot()` probes to the original source: Bull Count, Bear Count, Swing Top, Swing Bottom, Bull Break, and Bear Break. Its SHA-256 is `CE5BB16D4DEBDA8877B6EEE87C40D2D7DB6CBC1163DD6A6FFF6EC5D40D63F6DD`. TradingView compiled this derivative in the Pine Editor and exported 300 daily chart rows; the export is `luxalgo-order-blocks-breaker-probe-native-v5-20260925.csv` (SHA-256 `327DEC5BFA4B97DB4D1B2A06D65A22E84F96031E0A7B276BFDA090EFA537EA14`). The last row was a forming September 25 bar and was excluded. All 299 confirmed rows align with the full local run's OHLC; each of the six probe series matches at every aligned bar within `1e-8` (1,794 values, zero mismatches and zero maximum observed error). The local run also has 9 boxes and 18 lines.

Ignored evidence under `.local/community-coverage-20260923/` includes the original source, derivative, full-run JSON, TradingView CSV, and `compare_luxalgo_order_blocks.py`. Run the latter to repeat the aligned comparison. This establishes default-input historical parity for the six numeric probes across the overlapping chart window. The native export does not expose box/line coordinates, so drawing geometry, alternative inputs, other symbols/timeframes, and forming-bar updates remain unverified.
