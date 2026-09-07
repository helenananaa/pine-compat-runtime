# Strategy Modern G2 ta.wma Input Float Length Audit

阶段与切片 ID：阶段 2 / 切片 `ta_wma_input_float_length` / `E_CALL_ARG_TYPE`  
状态：closed  
实际基线 HEAD：`f236832b9`（G2 `v5_fill_transp_entry_when` 之后）  
已有工作区变更：未跟踪 `AGENTS.md` 保持不暂存。  
本轮目标：`ta.wma` 的 `length` 接受 numeric-compatible 值，并按 Pine `int()`
向零截断，使 macd_reloaded 的 `ta.wma(..., length / 2)` 进入运行。  
非目标：不把 `ta.sma` / `ta.hma` / `ta.vwma` 的 integer-compatible 长度扩到
float；不把数组下标或 `ta.pivothigh` 一并放宽。  
文件白名单：`crates/pine-builtins`、`pine-sema`、`pine-runtime`、`pine-cli`、
`pine-wasm`、`python/tests`、`scripts/host_parity_required.txt`、相关
fixture/snapshot、`tests/fixtures/conformance.tsv`、`docs/BUILTIN_SIGNATURES.md`、
`docs/LANGUAGE_SCOPE.md`、`docs/CONFORMANCE.md`、本 audit、执行计划状态行。

语料 revision：`modern-strategy-r1`  
manifest SHA-256：`cd41cf7fd95a307ee8dacc983bf9ffd0bda37027dc8397ea78abc8c9b68446bb`

## 问题与证据

上一轮后合并排名仍有 `E_CALL_ARG_TYPE`（1 脚本）：
`permissive.macd_reloaded_strategy` 首个诊断为
`` `ta.wma` argument `length` expects integer-compatible, got input float ``。
源码形式为 `ta.wma(2 * ta.wma(src, length / 2) - ta.wma(src, length), math.round(math.sqrt(length)))`。
v5 `/` 得到 input float。未把完整私有策略写入公开树。

项目依据：`Accepts::IntCompatible` 只接受 Int kind；`PineValue::as_i64()`
同样忽略 Float，`unwrap_or(0)` 会把 float 长度变成 0 并返回 na。Pine `int()`
对有限浮点向零截断（`2.9→2`，`-2.9→-2`）。

## 设计

接受：仅 `ta.wma` 使用专用 `TA_WMA_PARAMS`，`length` 为 `NumericCompatible`。
运行时 `eval_average_source_length` 用 `as_trunc_i64` 向零截断（有限浮点；
Na/Inf/溢出为 na 路径 length 0）。

继续拒绝：series bool 等非 numeric 长度（`unsupported_ta_wma_length.pine` 改为
`ta.wma(close, close > open)`）；`ta.sma` / `ta.hma` / `ta.vwma` 仍走
`TA_SOURCE_DYNAMIC_LENGTH_PARAMS` 的 IntCompatible。v4 `sma(close, 5.0)` 仍
`E_CALL_ARG_TYPE`；v4 `wma` 作为 `ta.wma` 别名继承 numeric 截断。

## 实际修改

- `TA_WMA_PARAMS` 仅挂到 `ta.wma`。
- `PineValue::as_trunc_i64` 对齐 `int()`。
- 正向：`supported_ta_wma_input_float_length.pine`、
  `wma_input_float_length.pine`、`strategy_wma_input_float_length.pine`、
  realtime rollback fixture。
- 负向：`unsupported_ta_wma_length.pine` 仍拒绝非 numeric；sma/hma 负向保持。
- CLI/Python/WASM 快照与 `host_parity_required.txt` 登记。
- 增量扫描覆盖新 runtime fixture；realtime forming 回滚覆盖新 realtime fixture。
- conformance / BUILTIN_SIGNATURES / LANGUAGE_SCOPE 同步。

## 验证

| 命令 | 退出码 | 有效测试 |
| --- | --- | --- |
| `cargo test -p pine-sema --test fixtures accepts_supported_ta_wma_input_float_length_fixture` | 0 | 1 |
| `cargo test -p pine-sema --test fixtures reports_unsupported_ta_wma_length_fixture` | 0 | 1 |
| `cargo test -p pine-sema --test fixtures reports_unsupported_ta_sma_length_fixture` | 0 | 1 |
| `cargo test -p pine-sema --test fixtures reports_unsupported_ta_hma_length_fixture` | 0 | 1 |
| `cargo test -p pine-sema --lib integer_division_rejects_float_operands_and_nonconst_modern_qualifiers` | 0 | 1 |
| `cargo test -p pine-runtime --lib truncates_wma_numeric_length_toward_zero_like_int` | 0 | 1 |
| `cargo test -p pine-runtime --test realtime wma_input_float_length_fixture_rolls_back_forming_close` | 0 | 1 |
| `cargo test -p pine-runtime --test incremental runtime_fixtures_match_incremental` | 0 | 1 |
| `cargo test -p pine-cli runtime_outputs_match_golden_snapshots` | 0 | 1 |
| `python3 scripts/check_host_parity.py` | 0 | 866 CLI / 570 required |
| `scripts/verify.sh` | 0 | workspace tests + 658 pytest；日志 `.local/five-stage-evidence/stage2f/verify.log` |

## 语料前后

Combined freeze（含本地 permissive，不入库）：

| Metric | Before | After |
| --- | --- | --- |
| parse | 510/510 | 510/510 |
| sema | 503/510 | 504/510 |
| run | 505/512 | 506/512 |
| comparability | 0/512 | 0/512 |
| consistency | N/A | N/A |

阶段转换：

- `permissive.macd_reloaded_strategy.default`：sema failed → parse/sema/run
  passed。

`E_CALL_ARG_TYPE` 退出合并排名。下一语言根因：`E_CALL_ARG_VALUE` currency
（`buysell_vol_strategy`，仅 `currency.NONE`）。无独立成交参考，不移交阶段 3。

剩余限制：`ta.sma` / `ta.hma` / `ta.vwma` 长度仍为 integer-compatible；wma 非
numeric 长度仍拒绝。
