# G5 checkpoint 局部优化验收

日期：2026-09-07。状态：本轮限定工作负载的优化验收通过；不代表 G3 独立准确性通过。
基线 HEAD：`31f16bb87`，基准工具与本次核心修改仍在工作区，未提交或推送。

## 改动与状态保护

`historical.rs` 仅在策略启用 `calc_on_order_fills` 时保存求值 checkpoint。
原来所有策略每根 bar 都复制求值状态，而未启用该设置的策略在
`recalculate_after_fill` 中直接返回，不消费 checkpoint。现在省去该路径的复制和恢复。
成交后重算策略保持原路径；不改变订单顺序、风险检查、实时回滚或公开接口。
没有新缓存、失效规则或额外常驻副本；省去的工作与被复制的求值状态大小相关。

临时插桩仅用于定位，未保留在核心代码。1024 bars 趋势探针的历史、增量和实时种子
路径共记录 3072 次 snapshot 和 3072 次 restore，各自累计 2.816 ms 和 2.791 ms。
插桩会扰动执行，不能据此推算总耗时比例；正式 A/B 使用未插桩 release 二进制。

新增 `strategy_checkpoint_equivalence.rs` 比较同一无订单状态策略开启/关闭
`calc_on_order_fills` 两条路径：2048 bars 的 var、SMA、EMA、RSI、历史引用结果一致；
rolling 保留值不超过 64、容量不超过 128；64 bars 种子之后连续 100 次形成中替换及
确认逐次比较完整输出。原有成交后重算和 guardrail 测试通过完整门禁验证。
这项测试保护本次路径差异，不证明所有脚本都具有相同内存上界。

## 固定验收与重复测量

正式 A/B 前保存 `acceptance.json`：1024 bars 下 trend、dense、realtime、magnifier
增量耗时改善的中位数至少 5%；collection 和 recalculation 各自回退不超过 10%；
每对进程峰值 RSS 增长不超过 `max(20%, 2048 KiB)`；源码、输入及完整历史/实时结果
哈希相同。后续同时确认 profile 非容量字段一致。

初轮为 64/256/1024 bars、两轮交替顺序，共 36 对：目标改善 21.05%，但 collection
回退 12.33%，**未通过**。该指标路径未改动，不能用平均值掩盖失败。保留全部记录，
在不改代码与预算的条件下把 1024 bars 扩为六轮交替顺序，共 36 对；复测通过。
这不是统计显著性或跨机器保证；初轮波动也保留为复测理由和测量限制。

每个子进程固定 warmup=2、iters=10、replacements=100，排除编译与序列化时间。
下表是各轮 `incrementalAppend` 中位数再取中位数，单位 ms：

| 场景 | 基线 | 修改后 | 改善 |
| --- | ---: | ---: | ---: |
| trend | 12.939 | 9.271 | 28.35% |
| dense | 8.049 | 6.206 | 22.90% |
| realtime | 5.951 | 4.576 | 23.10% |
| magnifier | 8.320 | 6.140 | 26.20% |
| collection（对照） | 7.238 | 7.260 | -0.30% |
| recalculation（对照） | 8.867 | 8.405 | 5.22% |

四个目标场景改善中位数 **24.65%**。两次矩阵中完整历史/实时结果哈希均一致，
非容量 profile 字段一致；RSS 预算通过。容量字段不是完全一致：trend 的
`rollingWindowValueCapacity` 从 25 到 40，避免 clone 后容器保留了原容量；
`currentSeriesCapacity` 也有分配布局差异。不能报告成所有 profile 相同。
集合、输出和交易历史随规模的增长见基线审计，不宣称全部内存恒定。

## 复现与证据

可复用工具 `scripts/compare_strategy_benchmarks.py` 保存原始配对结果、二进制哈希
及阈值，超预算或输出/保留数量不一致时退出非零。它需要分别构建的前后探针，
不能把当前同一个二进制充当两组。基线探针在核心修改前已复制保存。

```bash
python3 scripts/compare_strategy_benchmarks.py \
  --baseline .local/five-stage-evidence/stage5-optimization/baseline-probe \
  --candidate .local/five-stage-evidence/stage5-optimization/candidate-probe \
  --rounds 6 --bars 1024 \
  --output-dir .local/five-stage-evidence/stage5-optimization/reproduce
cargo test -p pine-runtime --test strategy_checkpoint_equivalence
python3 -m unittest scripts/tests/test_compare_strategy_benchmarks.py
scripts/verify.sh
```

本地证据目录 `.local/five-stage-evidence/stage5-optimization/`：

- `historical.before.rs`、`historical.profiled.rs`、`profile.stderr`：原始版本与插桩证据。
- `acceptance.json`：正式对照前预算；`attempt1/`：初轮未通过记录。
- `paired-results.json`、`comparison.json`、`validated-comparison.json`：六轮原始结果与复核。
- `optimized-build.log`、`final-verify.log`：release 构建及第一轮完整门禁，均 exit 0。
- `final-verify-v2.log`：加入两项状态回归和四项比较器测试后的最终完整门禁，exit 0；
  包括 fmt/clippy/workspace tests、工具测试、WASM/Node 及 Python wheel，Python 665 passed。

| 二进制 | SHA-256 |
| --- | --- |
| baseline | `3a3e11258cc7acffa96f405b650384c40ff325eca23c7c129bab454b13aec7bc` |
| candidate | `da429b004f666451b1eebfd7a2808e1daae8eb2166fd472322ce9e30bb1a3a2a` |

原始结果只存本地，不作为公开 CI 必需文件。普通 CI 检查语义与比较器失败分支，
不以此机器的绝对耗时为阈值。
