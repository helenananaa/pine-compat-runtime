# Strategy Modern G2 v5 UDF array.unshift Audit

阶段与切片 ID：阶段 2 / 切片 `udf_v5_array_unshift` / `function_side_effect`  
状态：closed  
实际基线 HEAD：`5f93af05c`（G2 `udf_v5_box_new` 之后）  
已有工作区变更：未跟踪 `AGENTS.md` 保持不暂存。  
本轮目标：v5/v6 UDF 体内允许 `array.unshift` 命名空间与方法形式，对应
captain_backtest_model 的 `reward_box.unshift(box.new(...))`。  
非目标：不把 `array.push`/`pop`/`set` 等其他集合变更扩进 UDF；不实现
`call_result.set_right`；不伪造 G3 Tester 参考。  
文件白名单：`crates/pine-sema`、`pine-runtime`、`pine-cli`、`pine-wasm`、
`python/tests`、`scripts/host_parity_required.txt`、相关 fixture/snapshot、
`tests/fixtures/conformance.tsv`、`docs/BUILTIN_SIGNATURES.md`、
`docs/LANGUAGE_SCOPE.md`、本 audit、执行计划状态行。

语料 revision：`modern-strategy-r1`  
manifest SHA-256：`cd41cf7fd95a307ee8dacc983bf9ffd0bda37027dc8397ea78abc8c9b68446bb`

## 问题与证据

上一轮后 captain 首个诊断为 UDF 内 `array.unshift`。源码形式为
`reward_box.unshift(box.new(...))`（方法 + 已支持的 `box.new`）。

项目依据：v4 UDF 已允许命名空间 `array.unshift`；顶层 v5 方法 `.unshift()`
已实现。官方 v5 允许函数内变更 array。`array.push` 等其他变更继续拒绝。

## 设计

接受：v5/v6 `function_depth > 0` 时 `array.unshift` 命名空间与方法（含
call-result `.unshift()`）。共享 array 引用就地插入，forming-bar 回滚。

继续拒绝：v5 UDF `array.push` 等；v4 方法语法仍按 v5+ 方法规则拒绝。

## 实际修改

- `allows_udf_collection_mutation_side_effect`：v4 既有子集 + v5/v6
  `array.unshift`。接到 builtin、array method、array call-result 三条路径。
- 正向：`supported_udf_array_unshift.pine`、`udf_array_unshift.pine`、
  `strategy_udf_array_unshift.pine`、realtime rollback。
- 负向：`unsupported_array_function_side_effect.pine`（`array.push`）仍拒绝。
- 巨型 call-result 负向 fixture 去掉 UDF unshift 诊断（255→254）。
- CLI/Python/WASM 快照与 host_parity。
- conformance 新行 `udf.v5.array_unshift`。

## 验证

| 命令 | 退出码 | 有效测试 |
| --- | --- | --- |
| `cargo test -p pine-sema --test fixtures accepts_supported_udf_array_unshift_fixture` | 0 | 1 |
| `cargo test -p pine-sema --test fixtures reports_unsupported_array_function_side_effect_fixture` | 0 | 1 |
| `cargo test -p pine-sema --test fixtures reports_unsupported_builtin_array_call_result_reads_fixture` | 0 | 1 |
| `cargo test -p pine-runtime --lib udf_array_unshift_prepends_like_top_level_unshift` | 0 | 1 |
| `cargo test -p pine-runtime --lib udf_namespace_array_unshift_matches_method_form` | 0 | 1 |
| `cargo test -p pine-runtime --test realtime udf_array_unshift_fixture_rolls_back_forming_size` | 0 | 1 |
| `cargo test -p pine-runtime --test incremental runtime_fixtures_match_incremental` | 0 | 1 |
| `cargo test -p pine-cli runtime_outputs_match_golden_snapshots` | 0 | 1 |
| `python3 scripts/check_host_parity.py` | 0 | 872 CLI / 576 required |
| `scripts/verify.sh` | 0 | workspace tests + 663 pytest；日志 `.local/five-stage-evidence/stage2j/verify.log` |

## 语料前后

Combined freeze（含本地 permissive，不入库）：

| Metric | Before | After |
| --- | --- | --- |
| parse | 510/510 | 510/510 |
| sema | 505/510 | 505/510 |
| run | 507/512 | 507/512 |
| comparability | 0/512 | 0/512 |
| consistency | N/A | N/A |

阶段转换：无脚本升到 run。

首个诊断转换：

- `permissive.captain_backtest_model_[tfo].default`：UDF `array.unshift` →
  `call_result.set_right`（需先绑定 receiver）。sema 仍 failed。

`function_side_effect` 退出合并排名。下一语言根因：`call_result.set_right`。
`E_IMPORT_MISSING_LIBRARY` 仍是 host 库源。G3 仍 blocked。

剩余限制：UDF 内 `array.push` 等其他集合变更仍拒绝。
