# Strategy Session Risk Window Behavior Audit

状态：closed（2026-09-06 session 复核修复与全量门禁重新通过；只覆盖本文列出的宿主 id 子集）。

基线 HEAD：`4de0dfd8ed375c377724442a3d09a27908665df4`

本次完成步骤：B1–B4。

行为锁定证据及日期：2026-09-06。官方策略风险文档仍把 intraday 规则描述为交易时段窗口，consecutive loss days 描述为交易日。本仓库 Stage 22e 把两者都接到 UTC 日/高周期 bar 时间戳。本阶段不把那套 UTC 子集改写成 session 精确语义。

## Evidence legend

- 官方文字：TradingView 策略风险说明（intraday vs consecutive days）。
- 样本观察：Stage 22e/22f/22g 现有 fixture。
- 项目确定性规则：本仓库已锁定的 UTC 窗口与 reset 行为。
- 未验证推断：无 Tester 输出的交易所日历细节。

## Distinct keys

| 键 | 用途 | 无宿主输入（UTC 子集） | 有宿主输入 |
| --- | --- | --- | --- |
| 自然日 | 不直接使用 | UTC 日只是实现细节 | 不使用 |
| 交易时段 / 窗口 | `max_intraday_loss`、`max_intraday_filled_orders` | `intraday_window_key(time, tf)` | `windowId` |
| 交易日 | `max_cons_loss_days` | 与上列同一 UTC/bar 键 | `tradingDayId` |
| 图表 bar | 高周期 UTC 子集的窗口 | `tf > 1D` 时用 bar 时间戳 | 不替代宿主 id |

不得把四个概念当成同一个键。

## Locked subset

- 隔夜时段跨 UTC 午夜：同一 `windowId` 不重置 intraday 计数与权益基线。
- 多时段：同一 `tradingDayId` 下 `windowId` 变化只重置一次 intraday 状态；cons-loss 等到 `tradingDayId` 变化才结算。
- 非交易日：没有 bar 就不插入合成窗口；跳过的日历日不记作无交易重置日（沿用 22g）。
- 日线及更高周期：无输入时保持 22e UTC 子集；有输入时完全使用宿主 id。
- 缺失输入：保留已文档化 UTC 子集，不声称 session 精确。
- 非空输入缺 bar / 空 id / 重复 / 未知版本：失败关闭，不静默回退到 UTC 再宣称 session。

## Host contract v1

```json
{
  "schemaVersion": 1,
  "bars": [
    { "barIndex": 0, "windowId": "eth", "tradingDayId": "d1" }
  ]
}
```

- 时间单位与时区由宿主规范化后给出字符串 id；核心不查询网络、不维护交易所日历。
- 批量：输入非空时必须覆盖 `0..bar_count-1`。
- 增量：允许输入含尚未执行的未来 bar；`extend_session_windows` 合并新增输入，已执行 bar 的 id 不可改写。正在执行的 forming bar 身份同样冻结。
- 不要求流式宿主提供无限未来日历。
- RealtimeSession：新增可选 kwarg，不升级 ABI 版本（与 `magnifier_bars` 相同的加性可选输入）。

诊断：`E_SESSION_SCHEMA_VERSION`、`E_SESSION_MALFORMED`、`E_SESSION_DUPLICATE_BAR`、`E_SESSION_EMPTY_ID`、`E_SESSION_COVERAGE`、`E_SESSION_HISTORY_CHANGED`。

未验证、排除：完整交易所日历、夏令时内部换算、公开风险状态 schema。

## Implementation

`SessionWindowInput` is a host-owned optional map of `barIndex -> (windowId, tradingDayId)`.
`reset_risk_windows` splits intraday reset from trading-day cons-loss finalization.
CLI `--session-windows`, Python `session_windows=`, and WASM `$sessionWindows` share the same JSON parser. RealtimeSession ABI version stays 1; the argument is additive.
CLI `parses_run_options_with_session_windows` plus CLI/Python/WASM runs of
`strategy_session_overnight_filled_orders.pine` observe that the same
`windowId` across UTC midnight keeps the filled-order count, and that a
`windowId` change resets that count once.

## Commands

Owner-local: `cargo test -p pine-runtime --lib session_window` 7 passed; host overnight/session/trading-day tests passed; incremental 5 passed; CLI goldens unchanged.

Close-out: `scripts/verify.sh` after Stage B (recorded in `{SCRATCH}/stageB-verify.sh.log`).

## Remaining exclusions

- Exchange calendars, DST conversion, and silent session-accurate claims without host ids
- Public risk-state JSON
- Changing executed confirmed/forming-bar ids mid-run, or enabling host ids after UTC execution

## Review corrections

- Coverage validation moved before execution mutations: batch calls preflight only their
  appended interval; each single-bar call checks only its own index. No per-bar scan of the full prefix.
- Rust historical/realtime runtimes and Python RealtimeSession expose `extend_session_windows`.
  Missing next-bar coverage can be supplied and the failed bar retried without restarting.
- Extension validation is atomic. Conflicting executed ids are rejected before any future
  rows are merged; realtime confirmed and forming copies remain synchronized.
- Rust `with_session_windows` now returns `Result<Self, RuntimeError>` and validates
  replacement against executed history, including forming state. This is a Rust source-API
  correction requiring callers to handle the result, not a public JSON/schema change.
- Empty input continues to mean the existing UTC subset. Switching a previously executed
  UTC runtime to host-window mode requires a new runtime and replay; it is not an extension.
- Regression evidence: `crates/pine-runtime/tests/session_windows.rs`, the new interval
  coverage unit test, and Python realtime-session extension tests.

### Correction verification (2026-09-06)

Baseline HEAD: `24af9a3a6` with the pre-existing untracked `AGENTS.md` left untouched.
No commit, push or release was performed by this correction pass.

- Baseline `scripts/verify.sh`: EXIT:0; Python 644 passed.
- `cargo test -p pine-runtime --test session_windows`: 6 passed, covering batch
  preflight, retry, extension/batch equivalence, immutable history, UTC-mode protection,
  forming replacement and snapshot synchronization.
- `cargo test -p pine-runtime --lib session_window`: 8 passed, including the
  new sparse high-index interval check and existing overnight-window behavior.
- Final `scripts/verify.sh`: EXIT:0, including fmt, workspace clippy/tests,
  structure/parity checks, WASM Node smoke and a freshly built isolated Python wheel.
  Python 647 passed, including all 7 realtime-session tests.
- Host parity: 855 registered CLI snapshots, 559 required runtime and 5 required
  legacy-analysis assertions. Existing goldens and conformance matrix were unchanged.
- Local logs: `/tmp/pine-session-baseline.2b0mWB.log` and
  `/tmp/pine-session-final.dFvfzH.log`; these are temporary local artifacts, not
  durable CI evidence. Reproduce with the commands above if they have been removed.

The review's D1 inventory finding is addressed by reopening unsupported completion
checkboxes and recording the remaining manifest/measurement work in
`STRATEGY_MODERN_CORPUS_BEHAVIOR_AUDIT.md`, not by inventing independent corpus evidence.

下一步：阶段 C 普通图表跨 bar 跳空。
