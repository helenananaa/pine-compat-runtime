# Strategy Modern G2 Strategy Currency USD Audit

阶段与切片 ID：阶段 2 / 切片 `strategy_currency_usd` / `E_CALL_ARG_VALUE`  
状态：closed  
实际基线 HEAD：`e16a95f60`（G2 `ta_wma_input_float_length` 之后）  
已有工作区变更：未跟踪 `AGENTS.md` 保持不暂存。  
本轮目标：接受 `strategy(..., currency=currency.USD)` 作为与当前
`syminfo.currency` 相同的无换算路径，使该声明参数不再是首个阻塞。  
非目标：不实现 FX 换算；不接受 `currency.EUR` 等交叉货币；不把
`commission_value` 缺省 `commission_type` 混进本切片。  
文件白名单：`crates/pine-sema`、`pine-runtime`、`pine-cli`、`pine-wasm`、
`python/tests`、`scripts/host_parity_required.txt`、相关 fixture/snapshot、
`tests/fixtures/conformance.tsv`、`docs/BUILTIN_SIGNATURES.md`、
`docs/LANGUAGE_SCOPE.md`、`docs/CONFORMANCE.md`、`docs/SEMANTIC_MODEL.md`、
`docs/EXECUTION_SEMANTICS.md`、本 audit、执行计划状态行。

语料 revision：`modern-strategy-r1`  
manifest SHA-256：`cd41cf7fd95a307ee8dacc983bf9ffd0bda37027dc8397ea78abc8c9b68446bb`

## 问题与证据

上一轮后合并排名仍有 `E_CALL_ARG_VALUE`（1 脚本）：
`permissive.buysell_vol_strategy` 首个诊断为
`` `strategy` argument `currency` only supports currency.NONE in the current no-conversion subset ``。
源码形式为 `strategy(..., currency=currency.USD, default_qty_type=strategy.cash, ...)`。
未把完整私有策略写入公开树。

官方/项目依据：`syminfo.currency` 当前固定为 `"USD"`。`currency.NONE`（或省略）
继承品种货币；显式 `currency.USD` 与品种货币相同，换算恒等。`strategy.account_currency`
已返回 `syminfo.currency`；`convert_to_account` / `convert_to_symbol` 已是 identity。
交叉货币（`currency.EUR`）仍需汇率，继续拒绝。

G4 已把货币换算标为语言阻塞而非合格账本需求；本切片只做同币种声明语义。

## 设计

接受：const `currency.NONE` 与 const 当前品种货币（`"USD"` / `currency.USD`）。
运行时仍走无换算恒等：`strategy.account_currency == "USD"`，
`convert_to_account(close) == close`。不扩展公共 JSON。

继续拒绝：`currency.EUR` 及其他与品种货币不同的值；settings override；FX。

不把 `commission_value` 无 `commission_type` 扩进来。

## 实际修改

- `validate_strategy_declaration_args` 的 `currency` 接受 `NONE` 或 `"USD"`。
- 正向：`supported_strategy_currency_usd.pine`、`strategy_currency_usd.pine`。
- 负向：`unsupported_strategy_currency.pine` 改为 `currency.EUR`。
- `unsupported_strategy_declaration_properties.pine` 的 `currency="USD"` 不再报
  currency；剩余 `risk_free_rate` / `fill_orders_on_standard_ohlc`。
- 运行时单元测试证明 USD 声明下 account_currency 与 conversion 恒等。
- CLI/Python/WASM 快照与 `host_parity_required.txt` 登记。
- conformance / BUILTIN_SIGNATURES / LANGUAGE_SCOPE / SEMANTIC_MODEL 同步。

## 验证

| 命令 | 退出码 | 有效测试 |
| --- | --- | --- |
| `cargo test -p pine-sema --test fixtures accepts_supported_strategy_currency_usd_fixture` | 0 | 1 |
| `cargo test -p pine-sema --test fixtures reports_unsupported_strategy_currency_fixture` | 0 | 1 |
| `cargo test -p pine-sema --test fixtures reports_unsupported_strategy_declaration_properties_fixture` | 0 | 1 |
| `cargo test -p pine-runtime --lib strategy_currency_conversions_are_identity_for_explicit_symbol_currency` | 0 | 1 |
| `cargo test -p pine-runtime --test incremental runtime_fixtures_match_incremental` | 0 | 1 |
| `cargo test -p pine-cli runtime_outputs_match_golden_snapshots` | 0 | 1 |
| `python3 scripts/check_host_parity.py` | 0 | 867 CLI / 571 required |
| `scripts/verify.sh` | 0 | workspace tests + 659 pytest；日志 `.local/five-stage-evidence/stage2g/verify.log` |

## 语料前后

Combined freeze（含本地 permissive，不入库）：

| Metric | Before | After |
| --- | --- | --- |
| parse | 510/510 | 510/510 |
| sema | 504/510 | 504/510 |
| run | 506/512 | 506/512 |
| comparability | 0/512 | 0/512 |
| consistency | N/A | N/A |

阶段转换：无脚本从 sema failed 升到 run（完整 `buysell_vol_strategy` 仍失败）。

首个诊断转换：

- `permissive.buysell_vol_strategy.default`：`currency` 仅 NONE →
  `commission_value` 需要受支持的 `commission_type`。sema 仍 failed。
- 负向 `unsupported_strategy_declaration_properties`：currency 值错误 →
  `risk_free_rate` / `fill_orders_on_standard_ohlc` 的 `E_CALL_ARG_NAME`。

公开 min-repro `strategy_currency_usd.pine` parse/sema/run 通过。改写后的
`unsupported_strategy_currency.pine` 使冻结 combined-manifest 的该负向样本
`sourceSha256` 失效（`hash_mismatch`）；它是负向对照，不计入 N/M。

`E_CALL_ARG_VALUE` 仍占 1 脚本，下一形式是 `commission_value` 缺省
`commission_type`。无独立成交参考，不移交阶段 3。

剩余限制：交叉货币与 FX 仍拒绝；公共 JSON 不增加 currency 字段。
