# Strategy Ordinary-Chart Inter-bar Gap Behavior Audit

状态：closed

基线 HEAD：`ff84b09aa492e81a442bb8f35c92d12bd4cf2aab`

本次完成步骤：C1–C3。预修复失败记录：在 `8120777b6^` 的 scheduler 上保留当前 17 个 `strategy_ordinary_chart` 用例，得到 12 个断言失败、5 个通过（无跳空、恰在触发价、stop-limit 不成交、limit verification、incremental）。

行为锁定证据及日期：2026-09-06
https://www.tradingview.com/pine-script-docs/concepts/strategies/

官方文字（Broker emulator）：跨 bar 跳空没有中间 tick；价格单若在一根 bar 的收盘到下一根开盘之间穿越触发价，则在下一根开盘成交，而不是在触发价成交。

## Evidence legend

- 官方文字：上列 Broker emulator 段。
- 样本观察：本仓库 Magnifier 局部跳空 fixture 与 Stage 18g 路径。
- 项目确定性规则：统一候选/成交入口、既有 candidate 顺序、风险平仓与 OCA 在开盘成交后的既有规则。
- 未验证推断：TradingView 内部同价排序。本仓库沿用既有 candidate 顺序，不把它当作 Tester 顺序证据。

## Paths

| 路径 | 现状 | Stage C |
| --- | --- | --- |
| Magnifier lower-bar 间 | `MagnifierHostGap` 点事件，成交在 next open | 保持 |
| Magnifier 某 chart bar 首个 lower open | 该 lower 的 open | 若与上一 host close 不同，走同一 gap 入口 |
| 普通图表 bar N close → bar N+1 open | 先前未走 gap 入口；OHLC 路径从 open 开始 | `StrategySchedulerState.last_host_bar` 在走当前 host 序列前调用 `MagnifierHostGap::between` / `observe_host_open_gap` |

跳空不是可交易的 close-to-open 线段。Stop-limit 只在 gap 点激活，不回填激活前价格。Trailing 不消费不存在的中间价。价格单与待成交 market 单各参与一次开盘处理：market 仍走 `pre_script` 的 open 成交，价格单走 gap 点事件。

## Locked table

| 情形 | 成交价 | 证据 |
| --- | --- | --- |
| 上跳穿越 long stop | 下一开盘 | 官方文字 + `strategy_ordinary_chart_up_gap_fills_stop_at_next_open` |
| 下跳穿越 long limit | 下一开盘 | 官方文字 + `strategy_ordinary_chart_down_gap_fills_limit_at_next_open` |
| 下一开盘恰等于触发价 | 该开盘 | `strategy_ordinary_chart_gap_at_trigger_fills_stop_at_open` |
| 无跳空（open == 上一 close） | 路径上的触发价 | `strategy_ordinary_chart_no_gap_stop_still_fills_at_trigger` |
| 平坦 bar 上跳 | 下一开盘 | `strategy_ordinary_chart_flat_bar_gap_fills_stop_at_next_open` |
| 空头下跳穿越 stop | 下一开盘 | `strategy_ordinary_chart_short_down_gap_fills_stop_at_next_open` |
| stop-limit 跳空激活 | 不使用激活前价格成交 | `strategy_ordinary_chart_gap_stop_limit_activates_without_preactivation_fill` |
| bracket limit 被跳空穿越 | 下一开盘 | `strategy_ordinary_chart_gap_bracket_fills_limit_at_next_open` |
| 已激活 trailing 被下跳穿越 | 下一开盘，不是触发价 | `strategy_ordinary_chart_gap_trailing_fills_at_next_open_not_trigger` |
| slippage | 下一开盘再加 tick | `strategy_ordinary_chart_gap_stop_applies_slippage_to_next_open` |
| limit verification | 开盘未越过验证阈值则不成交 | `strategy_ordinary_chart_gap_limit_verification_requires_tick_beyond_open` |
| `calc_on_order_fills` | 新单不回填已消费的跳空价格 | `strategy_ordinary_chart_gap_calc_on_order_fills_does_not_backfill_gap_prices` |
| 同组 OCA cancel | 开盘只成交一个 peer | `strategy_ordinary_chart_gap_oca_cancel_fills_one_peer` |
| `max_intraday_filled_orders(1)` | 第一笔开盘成交后按既有规则强平 | `strategy_ordinary_chart_gap_max_intraday_filled_orders_trips_after_first_open_fill` |
| 待成交 market + stop | 各成交一次 | `strategy_ordinary_chart_pending_market_and_stop_each_fill_once_at_open` |
| 增量 append | 与批量一致 | `strategy_ordinary_chart_gap_incremental_append_matches_batch` |
| 普通图表 / Magnifier 1:1 | 同一 next-open 成交价 | `strategy_ordinary_chart_and_magnifier_share_next_open_gap_fill` |

Stage 18g 真正 OHLC 与 Stage 23 Bar Magnifier 保持关闭，不重写路径内核。

## Host goldens

新增 CLI/Python/WASM 公共输出：

| 快照 | 行为 |
| --- | --- |
| `runtime_strategy_ordinary_chart_up_gap_stop.json` | long stop 在 10→12 跳空以 12 成交 |
| `runtime_strategy_ordinary_chart_down_gap_limit.json` | long limit 在 12→10 跳空以 10 成交 |
| `runtime_strategy_ordinary_chart_no_gap_stop.json` | 无跳空仍以触发价 11 成交 |
| `runtime_strategy_ordinary_chart_gap_stop_limit.json` | stop-limit 只激活，不成交 |

## Existing default-bar snapshot changes

默认 `bars.csv` 为 1→2→3→4 的单点 bar，相邻 close≠open，因此是跳空。59 个既有策略快照改为在 next open 成交，而不是在触发价或插值价成交。272 个未改动的 `runtime_strategy_*.json` 是无跳空或不受价格单跳空影响的样本。

分组（全部是 next-open 规则，没有无关会计改写）：

1. Stop-limit 入场提前一根 bar、在开盘 3 而不是 4 成交：`runtime_strategy_entry_stop_limit.json`、`runtime_strategy_order_stop_limit_default_quantity.json`，以及反转/净额 `runtime_strategy_entry_stop_limit_reverses_short.json`、`runtime_strategy_order_stop_limit_long_against_short.json`、`runtime_strategy_order_stop_limit_long_flatten_short.json`。
2. 同 bar 退出从触发价 2.5/1.5/1 改为该 bar 开盘 2（上一 close 1→open 2）：`runtime_strategy_exit_active_entry_attachment.json`、`runtime_strategy_exit_metadata.json`、`runtime_strategy_exit_stop.json`、`runtime_strategy_exit_bracket_creation_bar.json`、`runtime_strategy_exit_bracket_stop_limit_stop_fill.json`、`runtime_strategy_exit_limit_short.json`、`runtime_strategy_exit_bracket_stop_limit_limit_fill_short.json`，以及 reservation/qty 同 bar 成交族。
3. 下一根 bar 退出从 2.5 改为开盘 3：limit/profit/bracket/qty/omitted 退出族，以及空头 loss/stop。
4. 更晚的 profit/bracket/limit-verification/pyramiding 退出从 3/3.5/5 改为对应 next open 4 或 6。
5. `runtime_strategy_exit_active_entry_loss_attachment.json` 的入场从 bar 2@3 改为 bar 1@2，因为 1→2 跳空已使 stop 入场可成交。
6. CLI 形状断言与 `matrix.json` 同步上述 next-open 成交价；`conformance.tsv` 增加跳空说明与四个 ordinary-chart fixture。

## Remaining limits

- 未验证 TradingView 内部同价排序。
- 不宣布 Magnifier 覆盖之外的真实交易所日历跳空。
- `fill_orders_on_standard_ohlc` 仍不支持。
- 公开 schema 不变。

下一步：阶段 D 合法语料与五个独立指标；若没有已接受却执行错误的最小根因，停在 blocker，不宣称完全实现。
