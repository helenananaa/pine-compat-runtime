# Strategy Modern Corpus Behavior Audit

状态：partial（D1 冻结清单与五项指标已由五阶段计划阶段 1 补齐；D2 仍无独立成交参考，不实现新行为切片）

基线 HEAD：`8120777b6c04c48cbbadf413cd3fa68ad62b4ce2`  
D1 测量更新 HEAD：`2d0c6e80f8e948c694580420878a26c1fd8fd98e`（步骤 0）加上阶段 1 工具提交。

本次完成步骤：仓库 fixture 和本地候选的汇总盘点、D2 粗分类；D1 逐样本冻结与五项独立指标见
[Strategy Modern Corpus R1 G1 Audit](STRATEGY_MODERN_CORPUS_R1_G1_AUDIT.md)。不实现新行为切片。

行为锁定证据及日期：2026-09-06。未使用伪造的 TradingView Tester 输出。

## D1 inventory

### In-repo original strategy fixtures

License class: original project fixtures. Source: this repository.

以下为原盘点记录的文件/快照数量，不是本次重新执行所得的阶段通过率，不能作为 D1 验收结果。

| Inventory category | Recorded count |
| --- | ---: |
| runtime Pine files | 373 `tests/fixtures/runtime/strategy_*.pine` |
| positive fixture files | 373 runtime fixtures plus 101 `tests/fixtures/sema/supported_strategy_*.pine` |
| recorded CLI snapshots | 335 plus unspecified unit coverage |
| recorded public JSON goldens | 335 |
| independently validated result samples | 0 |

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

## Remaining D1 work

1. ~~Produce a per-sample manifest~~ Done for revision `modern-strategy-r1`
   (`tests/fixtures/modern-strategy-corpus/r1/manifest.jsonl`, SHA-256
   `cd41cf7fd95a307ee8dacc983bf9ffd0bda37027dc8397ea78abc8c9b68446bb`).
2. ~~Freeze and deduplicate~~ N=480 scripts, M=482 scenarios; negative
   controls 93; excluded 2. No eligible public sample is missing OHLCV.
3. ~~Run parse/sema/runtime and separate comparability/consistency~~ Public
   rates: parse 480/480, sema 479/480, run 481/482, comparability 0/482,
   consistency N/A. Two reruns matched.
4. ~~Rank failure roots~~ Public freeze has one eligible sema conflict
   (imported v6 script vs v5 library). Combined local ranking is led by
   `E_CALL_ARG_TYPE` (3 scripts). Details in the G1 audit.

These inventory and measurement tasks do not require Tester output. Do not mark
Stage D complete merely because D1 measurement exists without an independent oracle.

## Behavioral implementation blocker

D2 accepted-but-wrong fill work requires an authorized v5/v6 strategy, frozen bars,
and independent expected fills. That input is missing; stop accepted-but-wrong fill
implementation. Language blockers from the G1 ranking proceed under the five-stage
plan. Do not claim Stage D fully implemented.

下一步前置条件：用户提供或授权一份带独立 Tester/参考输出的 v5/v6 策略及对应 K 线。
