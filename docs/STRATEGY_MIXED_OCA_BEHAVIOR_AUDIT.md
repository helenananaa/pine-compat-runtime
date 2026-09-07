# Strategy Mixed-Family OCA Behavior Audit

状态：closed

基线 HEAD：`deec361023d4af687bbe243c7e3bbf0eba9d68eb`

工作区已有变更：未暂存规划入口文档；未跟踪 `AGENTS.md`（未提交）。

本次完成步骤：A1–A7。

行为锁定证据及日期：2026-09-06 重新查阅
https://www.tradingview.com/pine-script-docs/concepts/strategies/
OCA groups 段及 `strategy.oca.cancel` / `strategy.oca.reduce` / `strategy.oca.none`。

## Evidence legend

- 官方文字：TradingView 概念文档 2026-09-06 原文。
- 样本观察：本仓库可运行 fixture / broker 单测观察到的当前行为。
- 项目确定性规则：本仓库已锁定并有测试的内部规则。
- 未验证推断：无 Tester 输出、不得据此扩大公开支持。

未向用户索取 TradingView Tester 输出；本阶段不伪造成交价或成交顺序实验。

## Locked questions

### 1. Group identity

官方文字：同一 OCA 组必须同时指定相同 `oca_name` **和** OCA type。相同名称、不同 type 是两个组；cancel / reduce / none 不能混在同一组。

项目确定性规则（Stage 20a，仍有效）：`OcaGroupKey = (name, type)`。空名称不入组（`assign_pending_order_oca_named` 对空串直接返回）。

### 2. `entry` / `order` acceptance

官方文字：`strategy.entry()` 与 `strategy.order()` 都接受 `oca_name` 和 `oca_type`。`strategy.oca.none` 是两者的默认 type。

样本观察：当前只接受 `strategy.order` 的 const/simple `oca_name` 与 const `oca_type` ∈ {none, cancel, reduce}。`strategy.entry` 签名没有这两个参数，命名传入会被拒绝。series `oca_name` 仍拒绝。

本阶段将按官方文字为 `strategy.entry` 补齐与 `strategy.order` 相同的 const/simple `oca_name` + const type 子集。不接受 series `oca_name`。

### 3. `exit` OCA

官方文字：`strategy.exit()` 订单默认属于 `strategy.oca.reduce` 组；可用 `oca_name` 把多个 exit 放进同一 reduce 组。公开签名没有 `oca_type`。

项目确定性规则（Stage 20e）：exit 组类型固定为 Reduce；省略 `oca_name` 时各组互斥预留。不为凑矩阵给 exit 增加 `oca_type`。

### 4. Mixed-family membership

官方文字：entry 与 order 只要 name+type 相同即同一组。exit 永远是 reduce，因此只有 `oca_type=strategy.oca.reduce` 且 `oca_name` 相同的 entry/order 才能与 exit 同组。

合法并计划支持：

| 组合 | 组键 | 证据 | 成交后效果 |
| --- | --- | --- | --- |
| entry↔order 同名 none | `(name, none)` | 官方文字 | 互不影响 |
| entry↔order 同名 cancel | `(name, cancel)` | 官方文字 | 一方成交后取消另一方 |
| entry↔order 同名 reduce | `(name, reduce)` | 官方文字 | 按实际成交量减对方剩余量，≤0 删除 |
| order↔exit 同名 reduce | `(name, reduce)` | 官方文字（exit 恒为 reduce） | 双向减量 |
| entry↔exit 同名 reduce | `(name, reduce)` | 官方文字（v6 entry 有 OCA type；exit 恒为 reduce） | 双向减量 |
| exit↔exit 同名 reduce | `(name, reduce)` | 样本观察 / Stage 20e | 保持既有减量 |

明确排除：

| 组合 | 原因 |
| --- | --- |
| 同名不同 type | 官方文字：两个组，不交互 |
| entry/order cancel 与 exit | exit 不能进入 cancel 组 |
| entry/order none 与 exit | none 与 reduce 是两个组 |
| 空名称 | 不入组 |
| series `oca_name` | 本阶段不实现 |
| 给 `strategy.exit` 增加 `oca_type` | 官方签名无此参数 |
| 风险平仓 / 强平加入用户 OCA 组 | 项目规则，保持独立 |

### 5. Quantity basis

官方文字：reduce 按已成交的 contracts/shares/lots/units 减少未成交订单数量。

项目确定性规则（Stage 20d）：使用实际成交数量；跨零 generic 成交用绝对成交量；减到 ≤0 删除；margin 拒绝的成交不传播。

未验证推断（排除）：部分成交一笔委托的中间态价格路径。当前内核一笔委托一次吃完剩余量。

### 6. One exit command, multiple children

项目确定性规则：同一 `strategy.exit` 命令展开的 stop/limit/bracket 子项是独立 `OcaMember::Exit`，同组 reduce 按成员而不是按命令再扣一次。不得既按命令又按子项双扣。

### 7. Identity lifecycle

项目确定性规则：同 ID 同向替换保留内部 key 与组成员；改组覆盖 membership；`strategy.cancel` / `cancel_all` 清 membership；clone/rollback 随 broker 快照恢复。

### 8. Same-price competition

项目确定性规则：内部 creation sequence + stable key 排序。不得把该顺序宣传为 TradingView 内部顺序证据。可观察保证仅为本仓库确定性：先创建的同价成员先成交，随后 OCA 作用于剩余成员。

## Current implementation gap

`apply_oca_after_fill` 只处理 `OcaMember::Order` 同伴；`apply_oca_after_exit_fill` 只处理 `OcaMember::Exit` 同伴。entry 成交路径（`enforce_pyramiding`）不调用 `apply_oca_after_fill`。因此即使内部把 entry 与 order 标成同一组成员，entry 成交也不会取消/减少同伴。

OCA 在 entry 成交上的传播必须发生在 `resolve_deferred_relative_exits_for_entry` 之前（项目确定性规则）：刚因本笔成交才落地的保护性 exit 不是该成交的既有同伴，不能被同一成交减掉。已经在簿上的异组成员仍按组规则处理。

## A2 named cases versus omitted Cartesian cells

必须具名覆盖：entry→order 与 order→entry 的 none/cancel/reduce；order→exit 与 exit→order reduce；entry→exit 与 exit→entry reduce；同名不同类型；空名；无关组；整数与小数；reduce 到零；反转绝对量；同 ID 替换；limit 同价；stop-limit 未激活同伴；Magnifier 跨 lower bar。

省略并解释：不跑全部触发类型 × 全部组关系笛卡尔积。trailing / bracket 混合只保留“reduce 组成员被成交减量”这一既有 exit 表征，不新增无证据的 trailing 价格猜测。`process_orders_on_close` 与合法 `immediately` 保持旧行为，不把它们改成新的 OCA 语义。

## Implementation

`apply_oca_after_fill` 与 `apply_oca_after_exit_fill` 共用 `apply_group_peer_effects`，对 `OcaMember::Order` 和 `OcaMember::Exit` 同伴一视同仁。entry 成交在 `resolve_deferred_relative_exits_for_entry` 之前传播 OCA。`strategy.entry` 接受与 `strategy.order` 相同的 const/simple `oca_name` / const `oca_type` 子集。

## Snapshot / output changes

| 产物 | 原因 |
| --- | --- |
| `runtime_strategy_mixed_oca_entry_order_cancel.json` | 同组 cancel：E 成交 qty 1 @ 3，O 被取消，仓位 1 |
| `runtime_strategy_mixed_oca_entry_order_reduce.json` | 同组 reduce：E qty 1，O 由 2 减到 1 后成交，仓位 2 |
| `runtime_strategy_mixed_oca_none.json` | 同组 none：E 与 O 均成交 qty 1，仓位 2 |
| `runtime_strategy_mixed_oca_order_exit_reduce.json` | 同组 reduce：L qty 2，TP 成交 qty 1，SL 被减量且未出现在公开成交，仓位 1 |
| `tests/snapshots/matrix.json` | conformance 行增加 mixed OCA fixture 与说明 |

既有单类型 none/cancel/reduce goldens 未改。公共 schema 未改。

## Commands

Pre-fix mixed_oca: 8 failed / 6 passed, assertion failures only. Log `{SCRATCH}/stageA-mixed_oca-before.log`.

Owner-local twice: `cargo test -p pine-runtime mixed_oca` 22 passed; `oca` 87 passed; `pine-sema oca` 41 passed; `magnifier` 43 passed; `--test incremental` 5 passed; `--test realtime` 35 passed; CLI goldens and matrix passed; host parity 851 CLI snapshots / 555 required runtime. Logs `{SCRATCH}/stageA-targeted-1.log` (host-parity sort miss then fixed) and `{SCRATCH}/stageA-targeted-2.log`.

Close-out: `scripts/verify.sh` EXIT:0. Python 637 passed. Host parity 851/555. Log `{SCRATCH}/stageA-verify.sh.log`. `git diff --check` clean.

## Remaining exclusions

- series `oca_name`
- `strategy.exit` `oca_type` 参数
- 同名不同类型（官方：两个组）
- 空名称不入组
- 风险平仓 / 强平不加入用户 OCA 组
- 公开 pending-order / OCA schema
- 无用户提供的 Tester 输出；attached exit 与刚成交 entry 的时序采用“先 OCA 后 resolve deferred”的项目规则

下一步：阶段 B 交易时段输入与风险窗口。前置：阶段 A 已关闭。
