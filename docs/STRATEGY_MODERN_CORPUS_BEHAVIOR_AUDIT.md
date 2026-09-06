# Strategy Modern Corpus Behavior Audit

状态：partial（D1 仅初步盘点；冻结 manifest、去重报告和分阶段运行指标未完成；D2 未实现新行为）

基线 HEAD：`8120777b6c04c48cbbadf413cd3fa68ad62b4ce2`

本次完成步骤：仓库 fixture 和本地候选的汇总盘点、D2 粗分类。未完成逐样本冻结与五项独立指标，不实现新行为切片。

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

1. Produce a per-sample manifest containing source/license/revision/hash, Pine version,
   parameters, symbol/timeframe, bar-data hash and coverage; mark unavailable fields explicitly.
2. Freeze and deduplicate that manifest, recording excluded duplicates and missing-input samples.
3. Run the same frozen inputs through parse, sema and runtime, then separately record
   output comparability and independent correctness. Keep raw outcomes and commands.
4. Rank failure roots with affected sample counts, severity, dependencies and evidence quality.

These inventory and measurement tasks do not require Tester output. Do not mark them
complete merely because the next behavioral implementation has no independent oracle.

## Behavioral implementation blocker

D2 accepted-but-wrong fill work requires an authorized v5/v6 strategy, frozen bars,
and independent expected fills. That input is missing; stop new behavior implementation,
not the remaining D1 inventory/measurement work. Do not claim Stage D fully implemented.

下一步前置条件：用户提供或授权一份带独立 Tester/参考输出的 v5/v6 策略及对应 K 线。
