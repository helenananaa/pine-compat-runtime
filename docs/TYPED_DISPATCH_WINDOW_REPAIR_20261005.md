# Math/Array 操作码与窗口长度边界修复（2026-10-05）

本轮将 Math/Array 二级分派接入既有调用计划，修复 WASM 大窗口截断、pivot 组合长度溢出及 WMA/HMA 权重分母溢出。公开输出与状态检查通过；这批计时没有证明解释器整体提速。

基线 `9448f0058c9bf0e06af20999ff520687cd34a66d`，封存 6,333 份源码、311 份编译产物。最终 6,340 份源码绑定门禁、候选 release、链接、实际 WASM 与采样；本报告两文件在验证后生成。完整原始阶段值、进程凭证和 SHA 见 [结果 JSON](TYPED_DISPATCH_WINDOW_REPAIR_RESULTS_20261005.json)。

## 行为与架构

- Math 27 个函数、Array 56 个固定操作及泛型 UDT 构造使用内部 opcode。正常准备路径预绑定；执行阶段仍保留 callee 一致性检查，稀疏/未绑定/冲突 ID 及 stale callee 走既有回退。没有修改公开 HIR/ABI，也没有引入 host 依赖。

- 保留参数源顺序、named/omitted 行为、未知函数不求值参数、随机流、sum 与 TA 状态隔离、数组别名/slice/copy 及 var/varip realtime replacement。泛型 array.new<...> 保留手写 HIR 的宽匹配，chart.point 的精确构造优先。数值 helper 保持原逻辑，math.sum 单独加入 checked 长度转换。

- 所有实际用于 TA 窗口或历史索引的 i64 长度改为 checked usize 转换，失败沿用现有 extremes 的 Na 规则及原有返回形状（BB tuple、rising/falling false 等），在参数求值后、窗口更新前返回。pivot 左右允许 0，但左右及 +1 的组合长度 checked addition。没有设置新的任意长度上限。

- Wilder RMA 先访问窗口，因而必须检查长度；EMA/MACD 仅在窗口 seed 分支检查。已 seed 的巨大长度浮点递推、纯浮点 DEMA/TEMA/KC/KCW/TSI 保留。RollingWindow 的零长度 push/同 bar push 不改变旧状态，空窗口不再认为长度 0 已 ready。

- 四处 weighted 分母共用整数三角数 helper，在加法与乘法前提升为 u128，再转 f64。保留旧可表示整数的舍入及扫描顺序，避免 32 位合法长度 65536 的乘法溢出，也覆盖 usize 最大值。

## 实际故障与验证

| 旧 WASM 公开输入 | 修复前实际结果 | 修复后 |
|---|---|---|
| WMA(close, 4294967296)，10 bars | READY 后超过 10 秒，子进程被终止 | 退出 0，全部 Na |
| WMA(close, 4294967303)，10 bars | 与长度 7 的完整 JSON 完全相同，产生错误有限值 | 退出 0，全部 Na |
| pivothigh(close, 4294967295, 0) | 退出 1，add overflow / Wasm unreachable | 退出 0，全部 Na |
| WMA(close, 65536)，65536 条常量 1 bars | 退出 1，weighted_mean multiply overflow | 65535 个 warmup Na，随后为 1 |

旧 Wasm 二进制 SHA 为 `e35328691989f1d980f066653f3652bb245b0b4b40ebccad467127ab5dac427f`。复现使用封存的真实 Wasm 和生成 bindings，不是浏览器模拟；超时与原始 stderr 均保留。候选独立复现 7 个进程全部实际退出 0。新增 Node 子进程回归覆盖宽长度函数矩阵、tuple/bool 形状、缩回正常长度后的历史恢复、纯浮点大长度控制，以及 65536 WMA/HMA 首次 ready；45 秒超时用于阻止将来卡死 CI，不是性能阈值。

Windows 与 Linux 的 canonical verify 均实际退出 0：每平台 7,408 Rust、794 Python、166 工具测试；Linux 工具测试 skip 1，Windows skip 0；405 份 production Rust 文件通过结构检查。包括 fmt、Clippy、workspace、host parity、实际 Node/Wasm 与全新 wheel/venv。两个 cfg32 Rust 测试不计入 64 位 native 数量；真实 32 位行为由 Node/Wasm 验证。

第一轮完整门禁揭示新增手写 HIR 测试漏声明前一根历史依赖；修复测试的 close/high/low/volume history 声明后，重新封存并跑完整门禁。保留两个失败日志和对应快照，生产 retention 断言没有放宽。最初 Linux 启动使用了不存在的 Ubuntu 名称，查询实际名称后使用 Ubuntu-22.04。六个基线 probe correctness preflight 均退出 0，未混入正式采样。

## 公开 API 计时与分配

主矩阵 6 cases × 2 版本 × 3 fresh 进程 = 36。每进程固定 20,000 bars：4 个单次 analyze/prepared/historicalFullOutput/streamSeedWithoutOutput 阶段，以及 9 个 formingWithoutOutput、9 个 borrowedDelta；另有不计时的确认和 next historical 状态闭合。每进程保存 22 个完整 snapshots、11 个实际 deltas，数学/种子序列 oracle 逐类型及 IEEE 位核对，Replica apply 后严格等于生产者完整结果。完整 profile、revision 和全部公共 JSON 在每 case 的前后版本及重复进程间一致；等价源码 pair 另比完整 snapshots 的 plots。

历史全输出阶段包含 append_bars + owned result()。session 构造、caller 销毁、profile/oracle、编码、Replica 和 I/O 在窗口外。System allocator 连续计数成功 Layout 请求，报告累计 bytes、次数、入口上方 peak 和净 live delta，不能视作进程 RSS。全部阶段四项分配指标中位数前后一致。

Fixture：控制组为算术；Math positional/named 每 bar 8 组 × 7 个 close 相关调用；stateful 为 16/32 sum 与 seed 7/13 random；数组为固定 8 elements 的 var array，每 bar 16 次循环 get/set，再 sum/avg/size，不增加数组长度。价格为精确 quarter，bar 时间为毫秒 epoch + index × 60000。随机序列 oracle 只验证已有合同，不是 TradingView parity。

下表为主矩阵中位数，单位 ms，正百分比表示候选较快。forming 先取每 fresh 进程内 9 次中位数，再取 3 进程中位数；原始范围保留在 JSON。

| Case | 历史全输出 before→after | 变化 | seed 无输出 before→after | 变化 |
|---|---:|---:|---:|---:|
| control | 176.760 → 178.936 | -1.23% | 180.134 → 190.583 | -5.80% |
| math_positional | 309.571 → 311.231 | -0.54% | 308.668 → 306.309 | +0.76% |
| math_named | 296.138 → 321.563 | -8.59% | 314.734 → 304.228 | +3.34% |
| math_stateful | 94.259 → 95.369 | -1.18% | 91.076 → 90.927 | +0.16% |
| array_namespace | 501.701 → 492.050 | +1.92% | 533.168 → 480.589 | +9.86% |
| array_method | 465.025 → 462.373 | +0.57% | 458.078 → 452.143 | +1.30% |

| Case | forming 无输出 before→after | 变化 | borrowed delta before→after | 变化 |
|---|---:|---:|---:|---:|
| control | 0.1007 → 0.0956 | +5.06% | 0.1241 → 0.1476 | -18.94% |
| math_positional | 0.1343 → 0.1392 | -3.65% | 0.1456 → 0.1670 | -14.70% |
| math_named | 0.1644 → 0.1135 | +30.96% | 0.1351 → 0.1575 | -16.58% |
| math_stateful | 0.1198 → 0.1290 | -7.68% | 0.1547 → 0.1546 | +0.06% |
| array_namespace | 0.1450 → 0.1655 | -14.14% | 0.2203 → 0.1543 | +29.96% |
| array_method | 0.1365 → 0.1353 | +0.88% | 0.1731 → 0.1537 | +11.21% |

初始 Math named 历史中位数较慢 8.59%，且控制组/其他阶段存在波动，故保留原数据并针对 control/math_named 各追加 5 pairs，共 20 fresh 进程。结合每版本每 case 8 进程，Math named 历史较慢 0.86%、seed 较快 0.85%；控制组历史较慢 1.80%、seed 较快 0.45%。实时微小阶段仍有较慢观察：Math named 无输出较慢 16.57%（约 18.05 微秒）、borrowed 较慢 21.10%（约 25.8 微秒）；控制组分别较慢 6.88%（约 6.35 微秒）、23.50%（约 25.6 微秒）。不能将这些较慢结果全部宣称为噪声，也不能据此归因到 Math opcode；仍需保留为性能观察。数组 namespace 的主样本 seed 中位数改善 9.86%，method 改善 1.30%，也不能泛化为所有数组脚本；method 已在 frontend lowering，不是新增运行时 method lookup 优化。

总资格为 56 fresh 进程、1,848 份完整公共 JSON、1,232 次五指标阶段测量。独立审计不导入 runner、不启动 probe，重读真实 exit/stdout/stderr、完整原始 payload 与全部 hashes，重新计算主矩阵及追加矩阵的中位数/比例，核对源码、封存产物、链接、门禁及 Wasm 复现。审计实际退出与 helper SHA 保存在结果 JSON。所有计时仅描述该 instrumented native fixture 与整体 patch 的关联，不能全部归因于 opcode，更不构成统计显著性、全资源矩阵或产品性能资格。

## 仍待处理的源码热点

- WMA ready 后扫描 L；HMA 全/半已融合为一次 L，再扫描 round(sqrt(L))。variance/stdev/BB/BBW 非全零路径扫描 L；correlation 的 variance 及数值恢复还需多遍扫描。既有数值反例表明直接改递推、平方和或归约顺序会改变末位，不能未经资格改成通用 O(1)。

- weighted-only WMA/HMA 仍维护泛型 sum；极端输入 warmup 可反复触发非有限总和恢复。后续可以评估仅在实际消费时恢复 sum，但要覆盖共享 key/manual HIR、动态长度、同调用点循环、NA、signed zero、request/strategy/realtime rollback。

- realtime 页目录 COW 的约 O(L/128) 引用复制与有界端页复制是独立成本。Map/Matrix 仍有二级字符串分派，源码已核对，本轮未测其热度。

复现与计时原件保存在 ignored `.local/typed-dispatch-20261005/`；结果 JSON 保留可审计的原始指标及来源 hashes。没有 push。
