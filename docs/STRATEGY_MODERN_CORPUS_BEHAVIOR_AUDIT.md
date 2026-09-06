# Strategy Modern Corpus Behavior Audit

状态：partial（D1 清单已落盘；D2 因缺少独立预期结果而停止）

基线 HEAD：`8120777b6c04c48cbbadf413cd3fa68ad62b4ce2`

本次完成步骤：D1 合法语料清单与五项独立指标；D2 分类后停止，不实现新行为切片。

行为锁定证据及日期：2026-09-06。未使用伪造的 TradingView Tester 输出。

## D1 inventory

### In-repo original strategy fixtures

License class: original project fixtures. Source: this repository.

| Metric | Count |
| --- | ---: |
| parse | 373 `tests/fixtures/runtime/strategy_*.pine` |
| sema | 373 runtime fixtures plus 101 `tests/fixtures/sema/supported_strategy_*.pine` |
| runtime | 335 CLI snapshots plus unit coverage |
| output-comparable | 335 public JSON goldens |
| result-consistent vs independent Tester | 0 |

369 runtime fixtures are v5; 4 are v6. 106 `unsupported_strategy_*` files remain negative controls. 40 runtime pines have unit coverage only and no CLI snapshot. Default data is synthetic `tests/fixtures/runtime/bars.csv` unless a fixture maps a custom CSV. These goldens compare this runtime to itself; they are not Tester compatibility.

### Other legal sources inspected, not ingested

- `.local/legacy-corpus-r2` and `.local/legacy-corpus-r3`: private authorized indicator corpora. Zero `strategy(` files. Not Stage D.
- `.local/upstream-pine-candidates`: local MIT GitHub mirrors, 90 files containing `strategy(` (15 v5, 22 v6, remainder v1–v4 or unversioned). Scripts were not copied into the public tree. A local analyze/run sweep on default bars produced 0 runtime successes; failures are parse/sema. No Tester reference outputs, symbol, timeframe, or bar coverage are recorded.

## D2 classification

Observed candidate failures fall into language/type and builtin-acceptance gaps. No sample is both accepted by the current analyzer and independently shown to fill incorrectly.

| Root cause class | Independent expected result | Next action |
| --- | --- | --- |
| language/type / unsupported builtin | no | out of this slice |
| request data contract | no | out of this slice |
| strategy lifecycle accepted-but-wrong | no evidenced sample | **blocker** |
| account precision / report fields | not proven valuable by corpus | no new plan |

## Blocker

D2 requires an authorized v5/v6 strategy, frozen bars, and independent expected fills. That input is missing. Remaining corpus work is open-ended and not independently checkable. Stop. Do not claim Stage D fully implemented.

下一步前置条件：用户提供或授权一份带独立 Tester/参考输出的 v5/v6 策略及对应 K 线。
