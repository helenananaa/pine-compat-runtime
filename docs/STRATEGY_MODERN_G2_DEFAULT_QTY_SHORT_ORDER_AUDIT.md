# Strategy Modern G2 Default Qty Short Order Audit

阶段与切片 ID：阶段 2 / 切片 `default_qty_short_order` / `E_CALL_ARG_VALUE`  
状态：closed  
实际基线 HEAD：`0673c3b67`（G5 基线完成后）  
已有工作区变更：未跟踪 `AGENTS.md` 保持不暂存。  
本轮目标：市场 `strategy.order(..., strategy.short)` 省略 `qty` 时使用与 long 相同的
`default_entry_qty`，解除排名中该 `E_CALL_ARG_VALUE` 语言阻塞。  
非目标：不实现 `buysell_vol_strategy` 的 currency/commission；不一次修完
`dynamic_history_offset`；不发明 Tester 参考。  
文件白名单：`crates/pine-sema`、`pine-runtime`、`pine-cli`、`pine-wasm`、
`python/tests`、`scripts/host_parity_required.txt`、相关 fixture/snapshot、
`tests/fixtures/conformance.tsv`、`docs/BUILTIN_SIGNATURES.md`、
`docs/LANGUAGE_SCOPE.md`、`docs/CONFORMANCE.md`、
`docs/PURE_INTERNAL_STRATEGY_ORDER_DESIGN.md`、
`docs/NEXT_INTERNAL_CAPABILITY_PLAN.md`、本 audit、执行计划状态行。

语料 revision：`modern-strategy-r1`  
manifest SHA-256：`cd41cf7fd95a307ee8dacc983bf9ffd0bda37027dc8397ea78abc8c9b68446bb`

## 问题与证据

G2-round1 之后合并排名中，可实施的下一语言根因是
`E_CALL_ARG_VALUE`（2 脚本）。`permissive.backtest_adapter` 的首个诊断来自
`strategy.order("Exit Long", strategy.short)` / `"Enter Short"` 省略 `qty`。
分析器把该形式误判为 “reduce-only strategy.short requires an explicit positive qty”。
`buysell_vol_strategy` 同码但是 currency/commission，属 G4，本轮跳过。

官方/项目依据（查阅日期 2026-09-06）：
`strategy.entry` 与 long `strategy.order` 省略 `qty` 已走
`StrategySettings::default_entry_qty(equity, close)`。TradingView
`strategy.order` 的 `qty` 为可选；省略时使用声明默认数量，不因方向变成必填。
本切片只去掉错误的 short 必填检查，并在 runtime 共用默认数量路径。

## 设计

接受：市场 `strategy.order(id, strategy.short)` 省略 `qty`，数量为配置的
fixed/cash/percent-of-equity 默认值；可在空仓开空或减少/打平现有多头。

继续拒绝：未知方向（`"sideways"`）、未知 `oca_type`、series `oca_name`。

不把 limit/stop/stop-limit short 省略 `qty` 扩成单独声称；runtime 在
`eval_strategy_order` 共用默认数量解析，分析器也不再按方向要求显式 `qty`。
本轮 fixture 只覆盖市场 short。

## 实际修改

- 删除 `validate_strategy_order_args` 中 “reduce-only strategy.short requires
  an explicit positive qty”。
- `eval_strategy_order` 在省略 `qty` 且方向为 `strategy.long` 或
  `strategy.short` 时调用 `default_entry_qty`。
- 正向：`supported_strategy_order_default_qty_short.pine`、
  `strategy_order_default_quantity_short.pine`、
  `strategy_order_default_quantity_short_reduce_long.pine`。
- 负向：`unsupported_strategy_orders.pine` 去掉 `MissingShortQty`，保留
  `BadDirection` 与未知 `oca_type`。
- CLI/Python/WASM 快照与 `host_parity_required.txt` 登记。
- conformance / matrix / BUILTIN_SIGNATURES / LANGUAGE_SCOPE / CONFORMANCE /
  PURE_INTERNAL_STRATEGY_ORDER_DESIGN 去掉 “omitted qty for strategy.short
  remains unsupported”。

编辑 `unsupported_strategy_orders.pine` 会使冻结 r1 对该负向样本的 source
hash 失效（测量器报 `hash_mismatch`）。这不是 live 文件解析回归；负向诊断仍在。
不回写 r1 manifest。

## 验证

| 命令 | 退出码 | 有效测试 |
| --- | --- | --- |
| `cargo test -p pine-sema --test fixtures reports_unsupported_strategy_order_fixture` | 0 | 1 |
| `cargo test -p pine-sema --test fixtures accepts_supported_strategy_order_fixture` | 0 | 1 |
| `cargo test -p pine-runtime --lib market_short_order` | 0 | 2 |
| `cargo test -p pine-runtime --test incremental runtime_fixtures_match_incremental` | 0 | 1 |
| `cargo test -p pine-cli runtime_outputs_match_golden_snapshots` | 0 | 1 |
| `python3 scripts/check_host_parity.py` | 0 | 859 CLI snapshots / 563 required |
| `scripts/verify.sh` | 0 | workspace tests + 651 pytest；日志 `.local/five-stage-evidence/stage2b/verify.log` |

## 语料前后

Combined freeze（含本地 permissive，不入库）：

| Metric | Before | After |
| --- | --- | --- |
| parse | 510/510 | 510/510 |
| sema | 500/510 | 501/510 |
| run | 502/512 | 503/512 |
| comparability | 0/512 | 0/512 |
| consistency | N/A | N/A |

阶段转换：

- `permissive.backtest_adapter.default`：sema failed → parse/sema/run passed。

`E_CALL_ARG_VALUE` 从 2 脚本降为 1（剩余 `buysell_vol_strategy`，currency，G4）。
下一语言根因：`dynamic_history_offset`（2 脚本：`macd_reloaded_strategy`、
`twin_optimized_trend_tracker_strategy_tott`）。`E_IMPORT_MISSING_LIBRARY`
是宿主输入，跳过。无独立成交参考，不移交阶段 3 行为修复。

剩余限制：price-based short 省略 `qty` 无单独 fixture；未知方向与未知 OCA
仍拒绝。
