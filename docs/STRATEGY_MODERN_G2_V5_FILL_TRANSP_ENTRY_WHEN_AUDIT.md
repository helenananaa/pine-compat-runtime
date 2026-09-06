# Strategy Modern G2 v5 Fill Transp and Entry When Audit

阶段与切片 ID：阶段 2 / 切片 `v5_fill_transp_entry_when` / `E_CALL_ARG_NAME`  
状态：closed  
实际基线 HEAD：`6ca27ef35`（G2 `strategy_format_precision` 之后）  
已有工作区变更：未跟踪 `AGENTS.md` 保持不暂存。  
本轮目标：TOTT 的两个 v5 hidden 命名参数：`fill(..., transp=...)` 与
`strategy.entry(..., when=...)`。官方 v5 仍接受、v6 删除。  
非目标：不把 `transp`/`when` 扩到 v6；不把 `when` 扩到 `strategy.order/close`。  
文件白名单：`crates/pine-builtins`、`pine-sema`、`pine-runtime`、`pine-cli`、
`pine-wasm`、`python/tests`、`scripts/host_parity_required.txt`、相关
fixture/snapshot、`tests/fixtures/conformance.tsv`、`docs/BUILTIN_SIGNATURES.md`、
`docs/LANGUAGE_SCOPE.md`、`docs/CONFORMANCE.md`、本 audit、执行计划状态行。

语料 revision：`modern-strategy-r1`  
manifest SHA-256：`cd41cf7fd95a307ee8dacc983bf9ffd0bda37027dc8397ea78abc8c9b68446bb`

## 问题与证据

上一轮后 `E_CALL_ARG_NAME` 只剩 TOTT：`fill` 无 `transp`，`strategy.entry` 无 `when`。
源码为 v5：`fill(..., color=longFillColor, transp=90)` 与
`strategy.entry(..., when=Timerange())`。

官方依据（查阅日期 2026-09-06）：
- v5 隐藏 `transp`，v6 从 fill/plot/bgcolor 等删除；颜色文档仍示例
  `plot(..., transp=40)` 作为 v5 等价写法。
- `when` 在 v5 弃用但仍工作，v6 从 strategy.* 删除；true 才下单，默认 true。

项目内 v1-v4 fill 已通过 `$legacy_transp` 应用透明度；v5 现代签名缺 `transp`。
`eval_fill` 已有 `apply_legacy_transparency`。本切片补签名与 v5 求值路径。

## 设计

接受（v5）：`fill(..., transp=simple int)` 在颜色无嵌入 alpha 时应用 0–100
透明度；省略 `transp` 保持传入颜色（不套用 v4 默认 90）。
`strategy.entry(..., when=bool)` 仅当条件为 true 时下单；false/na/其他为 no-op。

继续拒绝：v6 的 `transp`/`when`（`E_UNSUPPORTED_FEATURE`）；v5 series transp
（simple int 边界）。

不把 `when` 加到 close/order/exit。

## 实际修改

- `FILL_PARAMS` 追加 optional `transp`；`eval_fill` 读取第 8 槽/`transp`。
- `STRATEGY_ENTRY_PARAMS` 追加 optional `when`；`eval_strategy_entry` 用
  `call_arg_expr(..., 10, "when")`（lowering 会去掉参数名）。
- v6 分析器拒绝这两个参数。
- 正向/负向 fixture、CLI/Python/WASM 快照、host_parity、conformance/matrix。

## 验证

| 命令 | 退出码 | 有效测试 |
| --- | --- | --- |
| `cargo test -p pine-sema --test fixtures accepts_supported_fill_transp_fixture` | 0 | 1 |
| `cargo test -p pine-sema --test fixtures reports_unsupported_fill_transp_v6_fixture` | 0 | 1 |
| `cargo test -p pine-sema --test fixtures accepts_supported_strategy_entry_when_fixture` | 0 | 1 |
| `cargo test -p pine-sema --test fixtures reports_unsupported_strategy_entry_when_v6_fixture` | 0 | 1 |
| `cargo test -p pine-runtime --lib entry_when_false_skips_order` | 0 | 1 |
| `cargo test -p pine-runtime --test incremental runtime_fixtures_match_incremental` | 0 | 1 |
| `cargo test -p pine-cli runtime_outputs_match_golden_snapshots` | 0 | 1 |
| `python3 scripts/check_host_parity.py` | 0 | 864 CLI / 568 required |
| `scripts/verify.sh` | 0 | workspace tests + 656 pytest；日志 `.local/five-stage-evidence/stage2e/verify.log` |

## 语料前后

Combined freeze（含本地 permissive，不入库）：

| Metric | Before | After |
| --- | --- | --- |
| parse | 510/510 | 510/510 |
| sema | 502/510 | 503/510 |
| run | 504/512 | 505/512 |
| comparability | 0/512 | 0/512 |
| consistency | N/A | N/A |

阶段转换：

- `permissive.twin_optimized_trend_tracker_strategy_tott.default`：sema failed →
  parse/sema/run passed。

`E_CALL_ARG_NAME` 退出合并排名。下一语言根因：`ta.wma` input-float length
（macd_reloaded）或 `E_CALL_ARG_VALUE` currency。无独立成交参考，不移交阶段 3。

剩余限制：v6 删除这两个参数；`when` 未接到 close/order/exit；fill `transp` 非 series。
