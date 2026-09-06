# Strategy Modern G2 Input Float History Offset Audit

阶段与切片 ID：阶段 2 / 切片 `input_float_history_offset` / `dynamic_history_offset`  
状态：closed  
实际基线 HEAD：`d06252c79`（G2 `default_qty_short_order` 之后）  
已有工作区变更：未跟踪 `AGENTS.md` 保持不暂存。  
本轮目标：接受 `input.int / 2` 这类 input/simple float 作为历史偏移，使
`src[zxLag]` 不再以 `dynamic_history_offset` 为首个阻塞。  
非目标：不把 series float（`close[close]`）或 const 非整数偏移收成支持；
不一次把 `ta.wma(..., length / 2)` 的 integer-compatible 长度检查扩进来。  
文件白名单：`crates/pine-sema`、`pine-runtime`、`pine-cli`、`pine-wasm`、
`python/tests`、`scripts/host_parity_required.txt`、相关 fixture/snapshot、
`tests/fixtures/conformance.tsv`、`docs/LANGUAGE_SCOPE.md`、
`docs/HISTORY_SERIES_AUDIT.md`、本 audit、执行计划状态行。

语料 revision：`modern-strategy-r1`  
manifest SHA-256：`cd41cf7fd95a307ee8dacc983bf9ffd0bda37027dc8397ea78abc8c9b68446bb`

## 问题与证据

G2-round2 之后合并排名中，可实施的下一语言根因是
`E_UNSUPPORTED_FEATURE` `dynamic_history_offset`（2 脚本：
`macd_reloaded_strategy`、`twin_optimized_trend_tracker_strategy_tott`）。
共同形式为 ZLEMA：`length = input.int(...)` 后
`zxLag = length / 2 == math.round(length / 2) ? length / 2 : (length - 1) / 2`，
再 `src[zxLag]`。v5 `/` 得到 input float。未把完整私有策略写入公开树。

项目依据（查阅日期 2026-09-06）：runtime `eval_history_offset` 已接受
`PineValue::Float` 且 `value >= 0 && value.fract() == 0`。缺的是分析器。
`close[close]` 等 series float 仍应静态拒绝。

## 设计

接受：qualifier 为 `input` 或 `simple` 的 float 历史偏移。运行时仍要求非负整数
（小数部分为 0）。

继续拒绝：series float、bool、string、const 非整数 float（保留 UDT `1.5`
字段等负向诊断）。

不把 `ta.wma`/`ta.sma` 的 `length` integer-compatible 扩到 input float；
那是下一层 `E_CALL_ARG_TYPE`。

## 实际修改

- `validate_history_offset` 在 Int 之外接受 Input/Simple Float。
- 正向：`supported_dynamic_history_input_float_offset.pine`、
  `dynamic_history_input_float_offset.pine`、
  `strategy_dynamic_history_input_float_offset.pine`、
  realtime rollback fixture。
- 负向：`unsupported_dynamic_history.pine`（`close[close]`）仍拒绝。
- CLI/Python/WASM 快照与 `host_parity_required.txt` 登记。
- 增量扫描覆盖新 runtime fixture；realtime forming 回滚覆盖新 realtime fixture。
- v5 `close[5 / input.int]` 的分析测试改为通过（值是否为整数留给 runtime）。

## 验证

| 命令 | 退出码 | 有效测试 |
| --- | --- | --- |
| `cargo test -p pine-sema --test fixtures accepts_supported_dynamic_history_input_float_offset_fixture` | 0 | 1 |
| `cargo test -p pine-sema --test fixtures reports_unsupported_dynamic_history_fixture` | 0 | 1 |
| `cargo test -p pine-sema --lib accepts_input_float_history_offset` | 0 | 1 |
| `cargo test -p pine-runtime --lib runs_input_float_history_offset` | 0 | 1 |
| `cargo test -p pine-runtime --test realtime dynamic_history_input_float_offset_rolls_back_forming_history` | 0 | 1 |
| `cargo test -p pine-runtime --test incremental runtime_fixtures_match_incremental` | 0 | 1 |
| `cargo test -p pine-cli runtime_outputs_match_golden_snapshots` | 0 | 1 |
| `python3 scripts/check_host_parity.py` | 0 | 861 CLI / 565 required |
| `scripts/verify.sh` | 0 | workspace tests + 653 pytest；日志 `.local/five-stage-evidence/stage2c/verify.log` |

## 语料前后

Combined freeze（含本地 permissive，不入库）：

| Metric | Before | After |
| --- | --- | --- |
| parse | 510/510 | 510/510 |
| sema | 501/510 | 501/510 |
| run | 503/512 | 503/512 |
| comparability | 0/512 | 0/512 |
| consistency | N/A | N/A |

首个阻塞转换（仍停在 sema，不宣称已能运行）：

- `permissive.macd_reloaded_strategy.default`：`dynamic_history_offset` →
  `E_CALL_ARG_TYPE` `` `ta.wma` argument `length` expects integer-compatible, got input float ``。
- `permissive.twin_optimized_trend_tracker_strategy_tott.default`：
  `dynamic_history_offset` → `E_CALL_ARG_NAME` `` `fill` has no argument named `transp` ``。

`dynamic_history_offset` 已退出合并排名。下一语言根因：`E_CALL_ARG_NAME`
（2 脚本）或 macd 的 `ta.wma` input-float length。`E_IMPORT_MISSING_LIBRARY`
是宿主输入，跳过。无独立成交参考，不移交阶段 3 行为修复。

剩余限制：series float 偏移仍拒绝；非整数 float 在 runtime 报
`history offset must be an int`；`ta.*` length 的 input float 仍拒绝。
