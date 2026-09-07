# Strategy Modern G2 Commission Value Default Percent Audit

阶段与切片 ID：阶段 2 / 切片 `strategy_commission_value_default_percent` / `E_CALL_ARG_VALUE`  
状态：closed  
实际基线 HEAD：`c733dea40`（G2 `strategy_currency_usd` 之后）  
已有工作区变更：未跟踪 `AGENTS.md` 保持不暂存。  
本轮目标：`strategy(..., commission_value=N)` 在省略 `commission_type` 时使用官方默认
`strategy.commission.percent`。  
非目标：不新增佣金模式；不改已支持的显式 cash_per_contract / cash_per_order /
percent 路径。  
文件白名单：`crates/pine-sema`、`pine-runtime`、`pine-cli`、`pine-wasm`、
`python/tests`、`scripts/host_parity_required.txt`、相关 fixture/snapshot、
`tests/fixtures/conformance.tsv`、`docs/BUILTIN_SIGNATURES.md`、
`docs/LANGUAGE_SCOPE.md`、`docs/CONFORMANCE.md`、本 audit、执行计划状态行。

语料 revision：`modern-strategy-r1`  
manifest SHA-256：`cd41cf7fd95a307ee8dacc983bf9ffd0bda37027dc8397ea78abc8c9b68446bb`

## 问题与证据

上一轮后 `E_CALL_ARG_VALUE` 仍占 1 脚本：`permissive.buysell_vol_strategy`
首个诊断为
`` `strategy` argument `commission_value` requires a supported commission_type ``。
源码为 `strategy(..., commission_value=0.1)`，无 `commission_type`。

官方依据：TradingView `strategy()` 的 `commission_type` 默认
`strategy.commission.percent`，`commission_value` 默认 0。项目内 percent 佣金
`qty * fill_price * N / 100` 已实现；缺的是省略 type 时套用该默认。

## 设计

接受：仅提供有限非负 const `commission_value` 时，设置为
`StrategyCommission::Percent(value)`，与显式
`commission_type=strategy.commission.percent` 相同。

继续拒绝：未知 `commission_type`；负的 `commission_value`。

不把 FX 或新佣金种类扩进来。

## 实际修改

- 去掉 “commission_value 必须伴随 commission_type” 的硬拒绝。
- 正向：`supported_strategy_commission_value_default_percent.pine`、
  `strategy_commission_value_default_percent.pine`。
- 运行时测试：省略 type 的 `commission_value=10` 与显式 percent=10 利润/佣金相同。
- CLI/Python/WASM 快照与 `host_parity_required.txt` 登记；该 fixture 使用
  `strategy_next_tick_close_bars.csv`。
- conformance / BUILTIN_SIGNATURES / LANGUAGE_SCOPE / CONFORMANCE 同步。

## 验证

| 命令 | 退出码 | 有效测试 |
| --- | --- | --- |
| `cargo test -p pine-sema --test fixtures accepts_supported_strategy_commission_value_default_percent_fixture` | 0 | 1 |
| `cargo test -p pine-sema --test fixtures reports_unsupported_strategy_commission_unknown_fixture` | 0 | 1 |
| `cargo test -p pine-runtime --lib omitted_commission_type_defaults_to_percent_commission` | 0 | 1 |
| `cargo test -p pine-runtime --test incremental runtime_fixtures_match_incremental` | 0 | 1 |
| `cargo test -p pine-cli runtime_outputs_match_golden_snapshots` | 0 | 1 |
| `python3 scripts/check_host_parity.py` | 0 | 868 CLI / 572 required |
| `scripts/verify.sh` | 0 | workspace tests + 659 pytest；日志 `.local/five-stage-evidence/stage2h/verify.log` |

## 语料前后

Combined freeze（含本地 permissive，不入库）：

| Metric | Before | After |
| --- | --- | --- |
| parse | 510/510 | 510/510 |
| sema | 504/510 | 505/510 |
| run | 506/512 | 507/512 |
| comparability | 0/512 | 0/512 |
| consistency | N/A | N/A |

阶段转换：

- `permissive.buysell_vol_strategy.default`：sema failed → parse/sema/run passed。

`E_CALL_ARG_VALUE` 退出合并排名。剩余排名首位是 `E_IMPORT_MISSING_LIBRARY`
（host_data_contract，缺库源，非本仓库语言切片）。无独立成交参考，不移交阶段 3。

剩余限制：未知 commission_type 仍拒绝；公共成交 JSON 仍无 `commission` 字段。
