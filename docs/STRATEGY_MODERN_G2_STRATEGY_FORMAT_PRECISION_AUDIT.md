# Strategy Modern G2 Strategy Format Precision Audit

阶段与切片 ID：阶段 2 / 切片 `strategy_format_precision` / `E_CALL_ARG_NAME`  
状态：closed  
实际基线 HEAD：`adc74860f`（G2 `input_float_history_offset` 之后）  
已有工作区变更：未跟踪 `AGENTS.md` 保持不暂存。  
本轮目标：只接受 `strategy(..., format=..., precision=...)` 这一声明参数形式，
与 `indicator` 的 format/precision 规则对齐。  
非目标：不混入 `fill(transp=...)`、`strategy.entry(when=...)`、`strategy(scale=...)`
或非 NONE currency。  
文件白名单：`crates/pine-builtins`、`pine-sema`、`pine-cli`、`pine-wasm`、
`python/tests`、`scripts/host_parity_required.txt`、相关 fixture/snapshot、
`tests/fixtures/conformance.tsv`、`docs/BUILTIN_SIGNATURES.md`、
`docs/LANGUAGE_SCOPE.md`、`docs/CONFORMANCE.md`、本 audit、执行计划状态行。

语料 revision：`modern-strategy-r1`  
manifest SHA-256：`cd41cf7fd95a307ee8dacc983bf9ffd0bda37027dc8397ea78abc8c9b68446bb`

## 问题与证据

上一轮合并排名中可实施的下一语言根因是 `E_CALL_ARG_NAME`（2 脚本），但不是同一形式。
本切片只取 `permissive.how_to_set_backtest_date_range` 的
`strategy(..., format = format.price, precision = 8)`。
TOTT 的 `fill(transp)` / `strategy.entry(when)` 留在队列。

官方依据（查阅日期 2026-09-06）：TradingView `strategy()` 与 `indicator()` 一样接受
`format`（`format.inherit` / `format.price` / `format.percent` / `format.volume`）
和 `precision`（const int 0–16）。它们是图上数值显示声明，不改变成交账本。
项目内 `indicator` 已按该集合校验且不写入公共 JSON；本切片复用同一规则。

G4 审计已把该项标为阶段 2 语言切片，不是账户能力。

## 设计

接受：named/optional const `format` 与 const int `precision`（0–16）。
为避免插入 overlay 之后的位置参数槽，参数追加在现有 `STRATEGY_PARAMS` 末尾，
不移动 `max_bars_back` 等既有位置绑定。

继续拒绝：`format.mintick` 及其他未知 format、precision 越界、series 类型、
`risk_free_rate`、`fill_orders_on_standard_ohlc`、非 NONE currency。

不把 format/precision 扩进公共 runtime JSON。

## 实际修改

- `STRATEGY_PARAMS` 追加 `format`、`precision`。
- `validate_strategy_declaration_args` 校验允许的 format 常量与 0–16 precision。
- 正向：`supported_strategy_format_precision.pine`、
  `strategy_format_precision.pine`。
- 负向：`unsupported_strategy_format.pine`（`format.mintick`）、
  `unsupported_strategy_precision.pine`（17）。
- CLI/Python/WASM 快照与 `host_parity_required.txt` 登记。
- conformance / matrix / BUILTIN_SIGNATURES / LANGUAGE_SCOPE / CONFORMANCE 同步。

## 验证

| 命令 | 退出码 | 有效测试 |
| --- | --- | --- |
| `cargo test -p pine-builtins --lib registers_strategy_declaration_signature` | 0 | 1 |
| `cargo test -p pine-sema --test fixtures accepts_supported_strategy_format_precision_fixture` | 0 | 1 |
| `cargo test -p pine-sema --test fixtures reports_unsupported_strategy_format_fixture` | 0 | 1 |
| `cargo test -p pine-sema --test fixtures reports_unsupported_strategy_precision_fixture` | 0 | 1 |
| `cargo test -p pine-runtime --test incremental runtime_fixtures_match_incremental` | 0 | 1 |
| `cargo test -p pine-cli runtime_outputs_match_golden_snapshots` | 0 | 1 |
| `python3 scripts/check_host_parity.py` | 0 | 862 CLI / 566 required |
| `scripts/verify.sh` | 0 | workspace tests + 654 pytest；日志 `.local/five-stage-evidence/stage2d/verify.log` |

## 语料前后

Combined freeze（含本地 permissive，不入库）：

| Metric | Before | After |
| --- | --- | --- |
| parse | 510/510 | 510/510 |
| sema | 501/510 | 502/510 |
| run | 503/512 | 504/512 |
| comparability | 0/512 | 0/512 |
| consistency | N/A | N/A |

阶段转换：

- `permissive.how_to_set_backtest_date_range.default`：sema failed → parse/sema/run passed。

`E_CALL_ARG_NAME` 从 2 脚本降为 1（剩余 TOTT 的 `fill(transp)` / `strategy.entry(when)`，
下一切片，不扩大本轮）。无独立成交参考，不移交阶段 3 行为修复。

剩余限制：`scale`、`risk_free_rate`、`fill_orders_on_standard_ohlc` 仍拒绝；
format/precision 不改变公共 JSON。
