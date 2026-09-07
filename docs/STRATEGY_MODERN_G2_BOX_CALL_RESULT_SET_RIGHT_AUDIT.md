# Strategy Modern G2 box call-result set_right Audit

阶段与切片 ID：阶段 2 / 切片 `box_call_result_set_right` / `call_result.set_right`  
状态：closed  
实际基线 HEAD：`e19fcdf88`（G2 `udf_v5_array_unshift` 之后）  
已有工作区变更：未跟踪 `AGENTS.md` 保持不暂存。  
本轮目标：Box 类型 call-result 上的 `.set_right(...)` 与已绑定
`id.set_right(x)` / `box.set_right(id, x)` 同语义，对应
captain_backtest_model 的 `risk_box.get(0).set_right(time)`。  
非目标：不放开 `call_result.set_left` 等其他 drawing call-result 方法；不把
namespace `array.get(...).set_right` 扩进当前 parse 子集；不把 UDF 内
`box.set_right` 放行；不伪造 G3 Tester 参考。  
文件白名单：`crates/pine-sema`、`pine-runtime`、`pine-cli`、`pine-wasm`、
`python/tests`、`scripts/host_parity_required.txt`、相关 fixture/snapshot、
`tests/fixtures/conformance.tsv`、`docs/BUILTIN_SIGNATURES.md`、
`docs/LANGUAGE_SCOPE.md`、`docs/CONFORMANCE.md`、`docs/EXECUTION_SEMANTICS.md`、
本 audit、执行计划状态行。

语料 revision：`modern-strategy-r1`  
manifest SHA-256：`cd41cf7fd95a307ee8dacc983bf9ffd0bda37027dc8397ea78abc8c9b68446bb`

## 问题与证据

上一轮后 captain 首个诊断为 `call_result.set_right`（需先绑定 receiver）。
源码形式为 `risk_box.get(0).set_right(time)` 与
`reward_box.get(0).set_right(time)`。绑定 `id = arr.get(0); id.set_right(x)`
与命名空间 `box.set_right(id, x)` 已实现。

解析器把标识符方法链 `id.get(0).set_right(x)` 做成 postfix call-result
（receiver 类型为 Box）。`analyze_array_call_result_method` 只处理 array
receiver，因此落到通用 bind-first 诊断。运行时 `eval_box_set_right` 已按
id 就地改 `snapshot.right`。

## 设计

接受：v5/v6 Box 类型 call-result 上的 `.set_right(x)`（含命名 `x=`），降低为
`box.set_right`，与绑定方法和命名空间形式同一变异。forming-bar 回滚沿用既有
box mutation 路径。

继续拒绝：`id.get(0).set_left(x)` 等其他 drawing call-result 方法仍要求先绑定；
namespace `array.get(...).set_right` 仍是 parse 子集外（`array.get` 不是
collection-producing callee）；UDF 内 `set_right` 仍是
`function_side_effect`。

## 实际修改

- `drawing_call_result_builtin_name`：仅 `(Box, "set_right")` → `box.set_right`。
- 分析、lowering、type_of 三条 call-result 路径接入该白名单。分析函数放到
  `calls/drawing_call_results.rs`，避免 `calls.rs` 超过 1500 行结构上限。
- 正向：`supported_box_call_result_set_right.pine`、
  `box_call_result_set_right.pine`、`strategy_box_call_result_set_right.pine`、
  realtime rollback。
- 负向：`unsupported_box_call_result_set_left.pine`（bind-first）；
  `unsupported_udf_box_call_result_set_right.pine`（UDF drawing）。
- CLI/Python/WASM 快照与 host_parity。
- conformance 新行 `box.call_result.set_right`。

## 验证

| 命令 | 退出码 | 有效测试 |
| --- | --- | --- |
| `cargo test -p pine-sema --lib box_call_result` | 0 | 2 |
| `cargo test -p pine-sema --test fixtures box_call_result` | 0 | 3 |
| `cargo test -p pine-runtime --lib box_call_result_set_right` | 0 | 1 |
| `cargo test -p pine-runtime --test realtime box_call_result_set_right` | 0 | 1 |
| `cargo test -p pine-runtime --test incremental runtime_fixtures_match_incremental` | 0 | 1 |
| `cargo test -p pine-cli runtime_outputs_match_golden_snapshots` | 0 | 1 |
| `python3 scripts/check_host_parity.py` | 0 | 874 CLI / 578 required |
| `scripts/verify.sh` | 0 | workspace tests + 665 pytest；日志 `.local/five-stage-evidence/stage2k/verify.log` |

公开 min-repro 快照：chained 与 bound 的 `box.get_right` 均为 `[0,1,2,3]`，
相等图为 `[1,1,1,1]`，box `right` 按 bar 变异。

## 语料前后

Combined freeze（含本地 permissive，不入库）：

| Metric | Before | After |
| --- | --- | --- |
| parse | 510/510 | 510/510 |
| sema | 505/510 | 506/510 |
| run | 507/512 | 508/512 |
| comparability | 0/512 | 0/512 |
| consistency | N/A | N/A |

阶段转换：`permissive.captain_backtest_model_[tfo].default` parse/sema/run
均 passed（本轮 bars.csv 上无成交，`noTrades=true`）。

`call_result.set_right` 退出合并排名。下一语言根因仍是 host
`E_IMPORT_MISSING_LIBRARY`、`TODO`、`E_LANGUAGE_VERSION_CONFLICT`。
G3 仍 blocked。

剩余限制：其他 drawing call-result 方法仍需先绑定；namespace
`array.get(...).set_right` 仍不解析；UDF `box.set_right` 仍拒绝。
