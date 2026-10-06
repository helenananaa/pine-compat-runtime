# Popular profile indicators: native comparison, 2026-09-26

Two complete Pine v6 indicators selected from TradingView's Popular / Open-source
listing now execute unchanged. Their default historical state agrees with native
exports on 4,283 confirmed `COINBASE:BTCUSD` daily bars, from 2014-12-01 through
2026-09-25. This work repairs three core semantic issues; it adds no host services,
data acquisition or CandleScope dependency.

## Sources

| Published script | Observed boosts when selected | Source SHA-256 |
| --- | ---: | --- |
| [Volume Delta Pivot Matrix, BigBeluga](https://www.tradingview.com/script/0p0kXntP-Volume-Delta-Pivot-Matrix-BigBeluga/) | 686 | `50f1455ab921d76bf3a13b357fc5d91985ae8679ceb487c1893d5ed5f23e941e` |
| [Buyers & Sellers Profile + Dynamic S/R, Zeiierman](https://www.tradingview.com/script/bEcZR3nB-Buyers-Sellers-Profile-Dynamic-S-R-Zeiierman/) | 285 | `5562d1f205379ca943046ddce2541f42a56dc8480032689df3fe09fd6f778b80` |

Both publications declare CC BY-NC-SA 4.0. Complete source copies remain in ignored
`.local/popular-20260926/`, with editor-rendered nonbreaking spaces converted to
ordinary spaces. No third-party source is added to tracked tests. Regression tests
use original minimal controls.

Chrome was used to read sources, compile native probes and export chart CSVs to
`I:\sys\下载\`. The final forming September 26 bar is excluded. Each comparator
checks timestamp, OHLC and volume equality before checking script values. Local
price grid: `1/100`; timeframe: `1D`. The confirmed input `chart-bars.csv` has SHA-256
`872f06656ef147307ae754023b18a5ecb3aab474b851b76a4670f87766379eb1`.

## Repairs

1. **Bounded drawing-style function returns.** Zeiierman's
   `line.set_style(..., lstyle(lvlStyle))` was rejected although the helper returns
   only valid line styles. Drawing-enum validation now joins bounded user-function
   return branches, including final `if` branches. It does not capture caller names
   or constants from another invocation. Parameter/local dependent return domains,
   recursive functions and ambiguous overloads remain fail-closed when their valid
   domain cannot be proven.
2. **History across skipped function calls.** History-bearing function series now
   retain the last evaluated value on intervening chart bars after their first
   evaluation. An explicit evaluated `na` remains `na`. Previously these series
   advanced only when evaluated, so `x[5]` could mean five calls ago instead of the
   value carried five chart bars back. Native v4/v6 controls establish the observed
   behavior. Existing checkpoints preserve the active-series set; regression tests
   cover provisional-value rollback, explicit `na`, locals and expression history.
3. **Invocation-specific history offsets.** A nested window function called with
   an input-derived offset and elsewhere with zero could reuse the latter call's
   analyzed constant. At index 1,048 (2017-11-17), the local profile incorrectly
   included the current high of 7,988.50 instead of the intended window high of
   7,898.00. Lowering now folds only syntax-local integer constants in parameterized
   bodies; other offsets use the current call's bindings. The same rule applies to
   legacy `offset()`. This favors correctness over constant-offset retention
   optimizations in those bodies: dynamic offsets may retain more history when no
   bound is configured.

The first remaining discrepancy after repair 2 was a false support update that
propagated into later price/score aggregates. Repair 3 removes it. The initial
4,282-bar comparison had 11,849 differing cells; after repair 2 the refreshed
4,283-bar comparison had 1,683; after repair 3 it has zero.

## Native results

Readonly probes expose existing indicator state without changing its decisions.
Zeiierman's probe sums retained level prices and scores, counts levels and supports,
and exports four creation/break flags. BigBeluga's outputs cover level count, price
sum, delta sum, support count and both pivots. Its probe runs before original
last-bar presentation trimming, so the confirmed last row has the same meaning in
a historical local run and a live native chart.

| Comparison | Confirmed bars | Compared cells | Mismatches |
| --- | ---: | ---: | ---: |
| Zeiierman: eight state outputs | 4,283 | 34,264 | 0 |
| BigBeluga: six state outputs | 4,283 | 25,698 | 0 |
| Gapped function history, Pine v4: ten outputs | 4,283 | 42,830 | 0 |
| Gapped function history, Pine v6: seven outputs | 4,283 | 29,981 | 0 |
| Nested window control: result and direct reference | 4,283 | 8,566 | 0 |

Tolerance is `abs_tol=1e-9, rel_tol=1e-9`. Zeiierman's maximum absolute score-sum
error is `1.5916157281026244e-12`; every other compared column above is exact. The
nested control's separate last-bar-only output is excluded: the native chart has
an additional forming bar, unlike the confirmed-only local input.

Both original sources execute with zero diagnostics. Removing added plots from
each probe's JSON gives exactly the original script's JSON, including drawings
and other outputs. BigBeluga's original local result contains seven lines and 31
labels; Zeiierman contains 336 line records and 124 label records (record totals
do not imply all are simultaneously visible). Zeiierman's full batch, incremental
and realtime-history JSON results are exactly equal.

CSV parity does **not** establish native drawing geometry, label text or styling
parity. Other settings, other symbols/timeframes, live forming-tick behavior of
complete scripts and packaged Python/WASM distributions are unverified. These
results do not establish general Pine v4/v6 compatibility or change release
qualification.

## Evidence and reproduction

Names below are relative to `.local/popular-20260926/`.

| Native download | Retained copy | SHA-256 |
| --- | --- | --- |
| `COINBASE_BTCUSD, 1D (48).csv` | `zeiierman-native-full.csv` | `31ba410a5ffaa2c8ef4a8c0d0c4c3867502807bebd476e45db449d79451c6b3d` |
| `COINBASE_BTCUSD, 1D (56).csv` | `bigbeluga-native.csv` | `907bd739667f47b381f037c09a7391a4a5d389fdb2d04d2a5ee3c231be99865c` |
| `COINBASE_BTCUSD, 1D (53).csv` | `gapped-history-native-v4.csv` | `516afaef57cc20f1ad56114fe417574281fe9c70a01fa7f1fcab42fdaff72924` |
| `COINBASE_BTCUSD, 1D (52).csv` | `gapped-history-native-v6.csv` | `630ace663c571382b64efff2716f1d47b72de18afee0433aaa4897ecd93f8fce` |
| `COINBASE_BTCUSD, 1D (55).csv` | `history-call-binding-native.csv` | `42305dd101670efec50374bee47ec4a8ffa8438cb4709be230737212a25689b7` |

Intermediate debug exports and probes are retained alongside these files.
`evidence-sha256.json` also identifies the two readonly probes. From the repository
root, with this retained evidence:

```powershell
cargo build -p pine-cli --locked
python .local/popular-20260926/run_candidate.py zeiierman-state-probe-v6.pine chart-bars.csv zeiierman-candidate.json
python .local/popular-20260926/compare_candidate.py
python .local/popular-20260926/run_candidate.py bigbeluga-state-probe-v6.pine chart-bars.csv bigbeluga-candidate.json
python .local/popular-20260926/compare_plot_csv.py bigbeluga-native.csv bigbeluga-candidate.json comparison-bigbeluga.json
```

`final_evidence.py` checks original/probe equivalence and the retained three
execution-mode outputs. Source/probe acquisition is local evidence, not a network
dependency of the runtime or tests.

Verification passed:

- `cargo test -p pine-sema -p pine-runtime -p pine-cli --locked --quiet`: all
  library/integration/doc tests, including 1,265 semantic-analysis tests, 1,948
  runtime library tests and 2,248 conformance fixtures. Log:
  `full-package-tests-final.log`.
- After adding the final legacy-offset regression and strengthening the alias
  control, `cargo test -p pine-runtime --lib --locked --quiet`: 1,949 passed.
  Log: `runtime-lib-final.log`.
- `cargo fmt --all -- --check` and `git diff --check`.
