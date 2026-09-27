# Hull Suite Strategy: EURUSD four-hour source inputs

Date: 2026-09-27. This local historical comparison extends the [EURUSD four-hour Hull receipt](HULL_FX_FOURHOUR_EXPANSION_20260927.md) with two `input.source` choices. The unchanged public Pine v4 [Hull Suite Strategy by DashTrader](https://www.tradingview.com/script/Q9OQye4C-Hull-Suite-Strategy/) has source SHA-256 `235a18e8a69aa6479d446bebc8c023eb0ce1b4a6e293e706cec079293354037d`.

Both new TradingView runs use `FX:EURUSD`, `240` minutes, `Strategy Direction=all`, `Hull Variation=Thma`, length 89, and the script's default 2016-01-01 to 2030-12-30 backtest window. The source input alone changes from `close` to `(H+L)/2` or `(H+L+C)/3`. The properties dialog retained 1,000,000 USD initial capital, 100% equity order size, pyramiding 1, zero commission/slippage, and on-bar-close execution. Each official chart export has the same 21,338 UTC time/OHLCV rows as the earlier close-source export, checked field by field. The host runs use the exported bars, `FX:EURUSD`, `240`, USD, price grid `1/100000`, and integer quantity precision. Native trade-report times are UTC+8.

The analyzer identifies `Source` as `input.source` call site 11 for this exact source revision. The CLI uses `--input-override 11=hl2` and `--input-override 11=hlc3` respectively. These call-site IDs are compilation-specific and must be rediscovered if the Pine source changes.

| Source | Native closed trades | Runtime closed trades | Trade time, price, direction differences | Quantity differences | Numeric plot cells | Plot differences |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| HL2 | 578 | 578 | 0 | **1 trade, 1 unit** | 42,502 | 0 |
| HLC3 | 578 | 578 | 0 | 0 | 42,502 | 0 |

Each native report also has one open position, excluded from the closed-trade comparison. `MHULL` and `SHULL` match from the first chart bar, with missing values checked as missing and numeric absolute tolerance `1e-10`. In the HLC3 case all 578 entry quantities match exactly; maximum displayed PnL absolute difference is 0.00614 USD.

The HL2 residual is trade **265**, an entry short at 2021-01-28 06:00 UTC+8. TradingView reports quantity 956,317; the runtime reports 956,318. The other 577 closed-trade quantities match. After trade 264, the local cumulative equity is approximately 1,157,890.70824 USD; 956,318 units at the next 1.21078 USD entry price require 1,157,890.70804 USD. The native report displays previous cumulative PnL as 157,890.70 USD, but does not expose its full internal precision. This is a sizing boundary sensitive to less than one cent of ledger difference; the available export does not identify a safe general change to broker arithmetic. The largest displayed PnL difference in this case is 0.01022 USD, on the same trade. **HL2 therefore has plot and trade-timing parity, with one unresolved quantity difference.** No runtime rule was changed on the strength of this single boundary case.

The official files and comparison script are retained under `.local/hull-fx-fourhour-20260927/`. `compare.py` produces `comparison.json`; the additional export hashes are:

| File | SHA-256 |
| --- | --- |
| `tv-chart-thma89-hl2-all.csv` | `f4d2b896ccafbac2b531ec632eecefc2b8ec6ec0abdd567875a1ea369434af3d` |
| `tv-trades-thma89-hl2-all.csv` | `26b4a3245d39c36901e6ec80fae9bb91d58206045c1a6874cb6a7fa1466e8dc2` |
| `tv-chart-thma89-hlc3-all.csv` | `536ad9337b8196fbe907fcf1ded67fb64e5cbaa833f4090d7dfcf5356ae1bccd` |
| `tv-trades-thma89-hlc3-all.csv` | `82c01d29f55ab0553342a35b7684b024a69e3055b9aa4a2a982c0a1afdd2d499` |

For each source choice, `run`, `run-incremental`, and `run-realtime-history` produced byte-identical full JSON. The corresponding SHA-256 hashes are `d551ae0764a2bd24b5b88c858666d205bb6029d9015746f474ca2e72e00327b5` (HL2) and `f6b89b2842bf7c54862ca5789d9b0eafee8aaf5d1ee8f4268794dd0f1584654a` (HLC3). This does not qualify live ticks, another symbol, or visual styling.

Follow-up on 2026-09-27: both source settings and the four earlier Hull parameter settings were rerun against the price-grid change; all six full runtime JSON outputs remain byte-identical to the retained originals and all existing official comparisons retain their results. The HL2 quantity difference is unchanged. To isolate the hidden equity precision, `.local/hull-fx-fourhour-20260927/hull-equity-probe.pine` retains the unchanged source plus three diagnostic plots of `strategy.equity`, `strategy.netprofit`, and `strategy.position_size` (probe SHA-256 `e1d7d2118f2a43d408b631216cc55c2bbabee804ff95f77337a14ca06d5906d8`). It analyzes without diagnostics and produces exactly the same local orders and trades as the original script. On bar index 12526 (2021-01-28 02:00 UTC+8), immediately before trade 265's next-open fill at 2021-01-28 06:00 UTC+8, the local probe reports equity `1157890.7082400005` USD; the next entry is at `1.21078` USD. Exporting these diagnostic plots from TradingView with the same inputs would reveal the native equity before the one-unit boundary. The probe is for diagnosis only and does not replace the unchanged-script qualification above.

Additional report-field audit: `report-ledger-consistency.py` checks three
frozen native trade CSVs. The displayed change in cumulative PnL differs by
0.02 USD from the same trade's displayed net PnL in six HL2 rows, one HLC3
row, and seven close-source rows. The HL2 trade 264 is one of them: reported
cumulative PnL moves from 159,765.10 to 157,890.70 USD (−1,874.40 USD),
while that trade's net PnL displays −1,874.38 USD. These rounded report fields
cannot reconstruct the exact broker equity at trade 265. The reproducible
receipt is `.local/hull-fx-fourhour-20260927/report-ledger-consistency.json`;
the 1-unit discrepancy remains unresolved pending a native equity probe.

The follow-up `equity-precision-audit.py` compares the native displayed
cumulative PnL with the sum of local full-precision closed-trade PnL for
`close`, `hl2`, and `hlc3` controls. The first local value outside the native
cent-display rounding interval occurs at trades 10, 6, and 7 respectively;
before trade 265, there are 47, 34, and 22 such rows. At HL2 trade 264, the
local sum is 157,890.708240001 USD against the native displayed 157,890.70
USD, a 0.008240001 USD gap. Thus the ledger difference predates the one-unit
entry and also appears in settings whose quantities match. The export does
not expose native internal equity or fill-price precision, so this audit
does not establish which arithmetic rule differs. The native equity probe
remains necessary before changing broker semantics.
