# Strategy Modern G2 Generic Input Source Audit

阶段与切片 ID：阶段 2 / 切片 `generic_input_source` / `E_CALL_ARG_TYPE`  
状态：closed  
实际基线 HEAD：`8d4b72def`（G1）  
已有工作区变更：未跟踪 `AGENTS.md` 保持不暂存。  
本轮目标：实现泛型 `input(close)` 源输入推断，使排名最高的语言阻塞进入运行。  
非目标：不实现 `input.source` 宿主覆盖；不接受 series int/bool/string defval；不一次修完后续 `dynamic_history_offset`。  
文件白名单：`crates/pine-sema`、`pine-builtins`、`pine-runtime`、`pine-cli`、`pine-wasm`、
`python/tests`、`scripts/host_parity_required.txt`、相关 fixture/snapshot、
`tests/fixtures/conformance.tsv`、`docs/BUILTIN_SIGNATURES.md`、
`docs/LANGUAGE_SCOPE.md`、本 audit、执行计划检查项。

语料 revision：`modern-strategy-r1`  
manifest SHA-256：`cd41cf7fd95a307ee8dacc983bf9ffd0bda37027dc8397ea78abc8c9b68446bb`

## 问题与证据

G1 合并排名首位是 `E_CALL_ARG_TYPE`（3 个 permissive 脚本）。共同首个诊断为
`` `input` argument `defval` expects const int/float/bool/string/color, got series float ``，
源码形式为 `src = input(close, title="Source")`。未把完整私有策略写入公开树。

官方依据（查阅日期 2026-09-06）：
[TradingView Inputs](https://www.tradingview.com/pine-script-docs/concepts/inputs/)
写明 `sourceInput = input(close, "Source")`，且泛型 `input()` 的返回为
`input int/float/bool/color/string | series float`。`input.source` 的 `defval`
接受 series float。项目内 `input.source(close)` 已支持；本切片只补泛型重载推断。

## 设计

接受：`input(close)`、`input(hl2)` 等 series float defval，返回 `series float`，
运行时求值 defval（与现有 `input.source` 相同，无宿主 source override）。

继续拒绝：series int（`input(bar_index)`）、series bool、非标量。const 标量
defval 仍提升为 `input` qualifier。

## 实际修改

- `Accepts::InputDefval` 增加 series float。
- `input_return_for_arg` 对 series float 返回 series float。
- 正向 fixture：`supported_input_defval_source.pine`、`generic_input_source.pine`、
  `strategy_generic_input_source.pine`。
- 负向：`unsupported_input_defval_series.pine` 改为 `input(bar_index)`。
- CLI/Python/WASM 快照与 `host_parity_required.txt` 登记。
- 增量扫描覆盖新 runtime fixture；realtime forming 回滚覆盖
  `tests/fixtures/realtime/generic_input_source.pine`。
- conformance / BUILTIN_SIGNATURES / LANGUAGE_SCOPE 同步。
- `tests/snapshots/matrix.json` 仅因 `input` 行 notes/fixtures 更新。

## 验证

| 命令 | 退出码 | 有效测试 |
| --- | --- | --- |
| `cargo test -p pine-sema --test fixtures reports_unsupported_input_defval` | 0 | 1 |
| `cargo test -p pine-sema --test fixtures accepts_supported_input_defval_source` | 0 | 1 |
| `cargo test -p pine-runtime --lib runs_generic_input_series_float_source_defval` | 0 | 1 |
| `cargo test -p pine-runtime --test realtime generic_source_input` | 0 | 1 |
| `cargo test -p pine-runtime --test incremental runtime_fixtures_match_incremental` | 0 | 1 |
| `cargo test -p pine-cli runtime_outputs_match_golden_snapshots` | 0 | 1 |
| `python3 scripts/check_host_parity.py` | 0 | 857 CLI snapshots / 561 required |
| `scripts/verify.sh` | 0 | 完整门禁，日志 `.local/five-stage-evidence/stage2/verify.log` |

## 语料前后

Combined freeze（含本地 permissive，不入库）：

| Metric | Before | After |
| --- | --- | --- |
| parse | 510/510 | 510/510 |
| sema | 499/510 | 500/510 |
| run | 501/512 | 502/512 |
| comparability | 0/512 | 0/512 |
| consistency | N/A | N/A |

阶段转换：

- `permissive.keltner_channels_strategy.default`：sema failed → parse/sema/run passed。
- `permissive.macd_reloaded_strategy` 与 `permissive.twin_optimized_trend_tracker_strategy_tott`
  的 `E_CALL_ARG_TYPE` 已不再是首个阻塞；下一层为
  `E_UNSUPPORTED_FEATURE` `dynamic_history_offset`。这是下一切片，不扩大本轮。

`E_CALL_ARG_TYPE` 已退出合并排名首位。下一语言根因：`E_CALL_ARG_VALUE`（2 脚本）
或 `dynamic_history_offset`（2 脚本）。无独立成交参考，不移交阶段 3 行为修复。

剩余限制：宿主 `input.source` 覆盖仍不支持；series 非 float defval 仍拒绝。
