# Strategy modern G5 baseline review audit

2026-09-08 状态更新：本文为优化前历史基线；G3 后续六组参考已收口，main-chart price grid 已实现。参见 [G3 closeout](STRATEGY_MODERN_G3_CLOSEOUT_AUDIT.md) 与 [当前开发记录](STRATEGY_MODERN_NEXT_CYCLE_AUDIT.md)。文末的 G3 blocked 描述仅代表当时状态。

状态：测量基线完成。后续热点分析及 A/B 已完成，见
[checkpoint 优化审计](STRATEGY_MODERN_G5_CHECKPOINT_OPTIMIZATION_AUDIT.md)；不宣称普遍资源上界。
日期：2026-09-07。实际基线 HEAD：`31f16bb87`。
已有未跟踪 `AGENTS.md` 未改动、未暂存。本轮不提交、推送或发布。

## 问题与修正

旧基准的 `incrementalAppend` 反复 `Program.run(bars[:end])`，不是增量执行；
`formingReplace` 把新建会话、seed、首次 forming 和确认一起计时，没有连续替换。
三个简单输入、16/64/256 bars 和 3 次测量不足以证明计划中勾选的代表性覆盖与资源结论。
旧审计保留为历史记录，过宽的阶段 5 完成勾选已经更正。

本轮产物：

- `crates/pine-runtime/examples/strategy_benchmark.rs`：离线 Rust 探针，仅使用既有 API。
- `scripts/benchmark_modern_strategy.py` v2：每个场景独立子进程，固定输入/源码/二进制哈希，
  汇总原始计时、结果哈希、profile、RSS 和随规模变化的资源增量。
- `tests/fixtures/benchmark/`：原创趋势、密集订单、成交后重算、Magnifier 工作负载。
  另复用 matrix 与 `calc_on_every_tick` fixture，共六类。
- Python 单元与真实 Rust 探针集成测试，接入 `scripts/verify.sh`。
- [剩余语料阻塞复核](STRATEGY_MODERN_REMAINING_BLOCKERS_REVIEW.md)。

不改变 parser、sema、broker、公开 JSON 或 Python/WASM API，不增加依赖。

## 计时与正确性边界

| 相位 | 实际测量边界 |
| --- | --- |
| compile | 源码已加载之后的 analyze/lower；不包含磁盘读取 |
| historicalRun | 同一新 runtime 的 `append_bars`；构造、取结果和序列化在区间外 |
| incrementalAppend | 同一新 runtime 上每根 bar 一次 `append_bar`；不重跑历史前缀 |
| resultSnapshot | 获取 runtime 结果快照 |
| realtimeSeed | 新实时 runtime 的历史 seed，独立计时 |
| formingInitial | 首次 forming 更新，独立计时 |
| formingReplace | 同一个实时 runtime、同一时间戳连续改变 close，逐操作计时 |
| formingConfirm | 确认原始最终 bar，独立计时 |
| outputSerialization | Rust 公共 JSON 序列化；与执行分开 |

实时 update API 自身返回快照，其成本包含在相应 update 时间中，不能称为纯 VM
求值时间。普通未启用 `calc_on_every_tick` 的策略在 forming 时只返回已确认状态；
报告显式注明。`realtime` 策略和 collection 指标实际执行替换。
Magnifier 使用每根图表 bar 两根宿主 lower bars，仅测历史与增量；live 明确排除，
不把 fallback 当 magnification。重算场景必须实际产生重算，订单场景必须实际产生订单。

每次重复都比较完整 batch/incremental JSON；历史和相同实时序列的最终输出分别
检查重复稳定性。实时和历史输入语义不同，不断言二者整体相等。这些是内部正确性/
回归证据，不是 TradingView 独立结果。

## 资源口径

每个场景/规模在新进程测 Linux `/proc/self/status` 的 VmHWM（KiB），在最终探针
报告渲染之前采样；包含输入、编译、所有相位及验证开销，不是 runtime-only RSS。
其它平台缺该计数时标 unavailable，报告降为 partial，不填 0。

runtime profile 保留 series、集合、request cache、输出和策略重算等已有字段；
数量/容量不是字节。另记录 outputBytes、订单数量和相邻规模的 profile/RSS 增量。
输出和交易历史可以随 bars 线性增长，本轮不宣称恒定内存或一般复杂度上界。

## 复现命令

仓库根目录执行，实际 target 路径可由 `cargo metadata` 确认：

```bash
cargo build --release -p pine-runtime --example strategy_benchmark
python3 scripts/benchmark_modern_strategy.py \
  --binary target/release/examples/strategy_benchmark \
  --output .local/five-stage-evidence/stage5-review/final-bench-1.json
python3 scripts/benchmark_modern_strategy.py \
  --binary target/release/examples/strategy_benchmark \
  --output .local/five-stage-evidence/stage5-review/final-bench-2.json
python3 -m unittest scripts/tests/test_benchmark_modern_strategy.py scripts/tests/test_strategy_benchmark_probe.py
scripts/verify.sh
```

`.local/` 已核实由 `.git/info/exclude` 忽略。它是本地证据，不作为公开 CI 依赖。
采集配置：seed=1，64/256/1024 bars，warmup=2，iters=10，replacements=100。
共 18 个场景/规模组合；每个非 Magnifier 组合有 1000 个测量内的替换操作。
p95 是合并操作样本的延迟，不是 1000 次独立进程实验。

## 验证记录

- 起始 `scripts/verify.sh`：exit 0，日志 `baseline-verify.log`。
- `cargo build --release -p pine-runtime --example strategy_benchmark`：exit 0，日志 `release-build.log`。
- 两次正式 benchmark：exit 0，均为 18 个 measured 场景；对应源码、输入、历史结果和
  实时最终结果 hash 全部一致。每个历史结果还逐次验证 batch == incremental。
- `python3 -m unittest scripts/tests/test_benchmark_modern_strategy.py scripts/tests/test_strategy_benchmark_probe.py`：
  8 tests，exit 0；其中真实 Rust 测试覆盖六个工作负载、语义分析错误和循环限额错误。
- 最终 `scripts/verify.sh`：exit 0，日志 `final-verify.log`，包括 workspace fmt/clippy/tests、
  工具单测、真实探针、WASM/Node、隔离 Python wheel；Python 665 passed。
- 结构检查：316 个 production Rust 文件；host parity：874 CLI 快照、578 runtime + 5 legacy assertions。
- `git diff --check`、相关文档链接与 Bash 代码块语法检查通过。

公开报告未扩 schema；v2 版本仅属于独立 benchmark 工具报告。
本地证据根为 `.local/five-stage-evidence/stage5-review/`，其中 `source-snapshot/`
保留 runner、Rust probe、Cargo.lock 与基准源码，`summary.json` 保存两轮哈希对照。

| 证据 | SHA-256 |
| --- | --- |
| release probe | `3a3e11258cc7acffa96f405b650384c40ff325eca23c7c129bab454b13aec7bc` |
| final-bench-1.json | `a7cb347728ffadacb26f53b781c61321a157c932288eacfac252fb7df103b408` |
| final-bench-2.json | `7239671dff4a94d338508d8404d9cfc9bbba9f31353cc6ef5e3b6212d1332f83` |

首轮 1024-bar 观察（仅供该环境基线使用，无提速对照）：

| 场景 | 订单事件数 | 重算次数 | 真增量总时长中位数 ms | 进程峰值 RSS KiB |
| --- | ---: | ---: | ---: | ---: |
| trend | 91 | 0 | 11.468 | 14244 |
| dense | 256 | 0 | 7.569 | 15944 |
| collection | 0 | 0 | 6.987 | 12856 |
| recalculation | 256 | 256 | 8.725 | 16096 |
| realtime | 1 | 0 | 5.628 | 11544 |
| magnifier | 512 | 0 | 8.813 | 18396 |

集合场景在 64/256/1024 bars 下，matrixSlots 为 192/768/3072，matrixCells 为
768/3072/12288；plotValues 为 64/256/1024。本轮如实记录该存储增长，不将其
描述为已回收或恒定空间。outputBytes 指规范化 JSON 的 UTF-8 字节数，独立于
Rust 序列化计时，也不能直接当作 runtime 堆占用。


## 本基线记录的范围与后续

- 本文记录优化前基线；后续 profiling、单热点优化与 A/B 见上方审计。
- 未证明任意真实策略的资源上界或生产容量。
- 基准工具自身失败、超时、错误计时数量或正确性不一致会失败，不能报告“基线完成”。
- 测量器有循环限额真实失败回归，但没有新增长状态模型或改变 runtime guardrail。
- G3 已找到独立导出，但缺匹配源码/设置；G4 仍是 deferred，不因性能工作而关闭。
