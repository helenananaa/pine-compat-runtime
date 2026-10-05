# 同 bar 窗口撤销的共享前页恢复（2026-10-05）

基线提交 `305d2b8a408919c000642ebafee718d468a37aae`。在生产修改或构建之前封存 6,347 个源码文件与 311 个编译产物；4 个直接链接库与上一轮已审计产物一致，已编译的 6,345 个文件是基线的未变子集，另 2 个仅为上一轮报告。候选冻结 6,348 个文件，最终两份报告在验证完成后生成。

## 实现和边界

RollingWindowState 撤销本 bar 输入时原本对每个 evicted 样本执行 push_front。分页 pop_front 常常只推进逻辑 offset，原 cell 仍存在；原 push_front 会通过 Arc::make_mut 复制共享目录/页，再写入同一个样本。

新增仅适用于 SharedDeque<Option<f64>> 的 restore_front：Paged 的 front_offset > 0，且原 cell 与待恢复样本的 Option 变体、完整 f64::to_bits 都一致时，只修改当前队列的 offset 与 len。比较不执行浮点运算，保留正负零与 NaN payload。Small、空队列、已移除的前页、offset=0 或 cell 位模式改变时继续原 push_front。泛型 push_front 不变，因此没有假设一般 Clone/PartialEq 元素可互换。

undo 顺序保持：pop_back，赋回 prev sum/NA/nonzero/change counters，evicted.pop_back 逆序恢复。没有改变算术顺序、窗口 key、open-bar、资格标记、公共 schema 或 host 边界。新增 8 项测试检查 Arc 目录/页身份、正负零/NaN/NA、覆写回退、129/385/1024 项跨页恢复、单页 stale tail、两个 checkpoint 的逻辑隔离、最终 Weak 释放，以及 4,000 步 VecDeque 位模式对照。最终释放测试检查全部 strong 引用释放后的 Weak 状态，不据此宣称逐次 drop 或 RSS 行为。

## 验证

算法定向测试实际退出 0，83 项通过。Windows 与 Ubuntu-22.04 完整 canonical verify 均实际退出 0：各 7,431 Rust、794 Python、166 工具测试，406 个 production Rust 文件通过结构门禁。Windows 工具 skip 0，Linux skip 1。包含 fmt、warnings-denied Clippy、workspace、host parity、真实 Node/Wasm 与新 wheel/venv。既有源码、测试和结构阈值保持。

正式测量前 8 个基线 fixture 均通过，预检计时不计入结果。正式 8 fixtures × 2 版本 × 3 fresh 进程，48 个进程实际退出 0；before/after 顺序交替。完整公共 JSON 1,584 份、typed sidecar 1,056 份、测量窗口 1,056 个，before/after 与重复间逐字节/逐位一致；原 Rust oracle 和 RuntimeReplica 校验全部通过。独立 auditor 另从原文件核对来源、311 份产物、编译/链接/实际退出码、源码差异、typed/FNV/profile/revision，并独立 replay 528 份 delta 和重算所有中位数。

每个 fixture 固定 6,000 bars，长窗 L=3,000；dynamic fixture 切换 3,000/1,536 并含 NA。history 包含 append_bars + owned result；seed、forming 无输出和 borrowed delta 分开计时。构造、drop、oracle、profile、完整编码、Replica 与 I/O 在窗口外。forming 每进程 9 次取中位数，再取 3 fresh 进程的中位数；未计时 confirm/next 改值检查未来状态。正式采样不与门禁、构建、链接或其他探针重叠。

## 公开 API 观察

单位 ms；正百分比表示候选耗时减少。三次 fresh 进程仅给出描述性观察，原始范围和请求分配指标见 JSON。

| Case | history before→after | 变化 | seed before→after | 变化 |
|---|---:|---:|---:|---:|
| dyadic_wma_hma | 18.0322 → 17.9432 | +0.49% | 17.7720 → 16.6392 | +6.37% |
| variance_exact_zero | 29.2191 → 29.1068 | +0.38% | 26.2726 → 28.6620 | -9.09% |
| decimal_fallback | 47.9380 → 46.9216 | +2.12% | 45.1540 → 46.5756 | -3.15% |
| varying_control | 49.0701 → 45.7086 | +6.85% | 45.7609 → 43.7611 | +4.37% |
| cold_sum_warming | 11.4440 → 11.3410 | +0.90% | 11.7797 → 11.8330 | -0.45% |
| dynamic_na | 43.8287 → 48.9030 | -11.58% | 40.2427 → 42.7846 | -6.32% |
| same_callsite_loop | 37.4534 → 35.2110 | +5.99% | 34.8730 → 34.8907 | -0.05% |
| zero_control | 41.2509 → 41.0961 | +0.38% | 39.4187 → 39.7177 | -0.76% |

| Case | forming 无输出 before→after | 变化 | borrowed before→after | 变化 |
|---|---:|---:|---:|---:|
| dyadic_wma_hma | 0.0814 → 0.0853 | -4.79% | 0.1016 → 0.0971 | +4.43% |
| variance_exact_zero | 0.0863 → 0.0904 | -4.75% | 0.1118 → 0.1097 | +1.88% |
| decimal_fallback | 0.1065 → 0.1235 | -15.96% | 0.1213 → 0.1291 | -6.43% |
| varying_control | 0.1330 → 0.1310 | +1.50% | 0.1430 → 0.1391 | +2.73% |
| cold_sum_warming | 0.0581 → 0.0743 | -27.88% | 0.0917 → 0.0874 | +4.69% |
| dynamic_na | 0.1396 → 0.1432 | -2.58% | 0.1397 → 0.1458 | -4.37% |
| same_callsite_loop | 0.0917 → 0.0903 | +1.53% | 0.1072 → 0.1031 | +3.82% |
| zero_control | 0.1171 → 0.1052 | +10.16% | 0.1261 → 0.1280 | -1.51% |

所有较慢观察保留在表与 JSON 中；普通 forming 从 confirmed clone 执行单次调用，不等同于同调用点 loop 的多次同 bar undo。这里不把普通 fixture 的时间变化归因于新恢复路径。

| Case | 阶段 | 分配指标 | before | after | 差值 |
|---|---|---|---:|---:|---:|
| same_callsite_loop | formingBorrowedDelta | allocatedBytes | 18996 | 14820 | -4176 |
| same_callsite_loop | formingBorrowedDelta | allocations | 80 | 76 | -4 |
| same_callsite_loop | formingBorrowedDelta | peakAdditionalLiveBytes | 15072 | 10896 | -4176 |
| same_callsite_loop | formingWithoutOutput | allocatedBytes | 15164 | 10988 | -4176 |
| same_callsite_loop | formingWithoutOutput | allocations | 67 | 63 | -4 |
| same_callsite_loop | formingWithoutOutput | peakAdditionalLiveBytes | 15072 | 10896 | -4176 |

这些指标为 System allocator 成功 Layout 请求：累计请求 bytes、调用次数、阶段入口以上的请求 live peak、退出 live delta；不包括分配器开销、realloc 短暂双缓冲和进程 RSS。历史/seed 的普通无共享写入与 forming 的共享前页写入分别观察，不外推通用解释器或跨平台吞吐。

## 慢项的追加核查

初批发现 dynamic/NA 历史 +11.58%、variance seed +9.09%、cold sum 无输出 forming +27.88%。保留原批全部样本，使用同一二进制与 fixture，对 loop、dynamic/NA、variance 和 cold sum 各追加 9 对 fresh 进程，共 72 个进程。全部实际退出 0；新增 2,376 public、1,584 typed、1,584 timed windows，均与原批 before-0 固定 reference 完全一致。独立审计另完成 792 次 delta replay、重算 9-process 中位数和逐对变化，并始末验证原批与二进制哈希不变。两批统计分别保存。

| Case | history 变化 | seed 变化 | 无输出 forming 变化 | borrowed 变化 |
|---|---:|---:|---:|---:|
| same_callsite_loop | +1.43% | +1.07% | +3.68% | +0.94% |
| dynamic_na | -3.61% | -0.09% | -4.18% | +2.97% |
| variance_exact_zero | +0.46% | +1.62% | +6.54% | +5.26% |
| cold_sum_warming | +0.64% | -5.17% | +0.00% | -10.37% |

循环两种 forming 在原批与追加核查均稳定减少 4 次分配、4,176 请求 bytes 与 4,176 额外 peak bytes，阶段退出 live delta 不变。无输出从 15,164→10,988 bytes，borrowed 从 18,996→14,820 bytes。该资源节省可由两个窗口免复制前页解释；历史/seed 分配指标不变。

追加核查仍保留慢项：dynamic/NA 历史 +3.61%、无输出 forming +4.18%；cold sum seed +5.17%、borrowed +10.37%。初批 cold sum 无输出 forming 的 +27.88% 在追加中位数中变为 0.00%；variance seed 转为耗时减少 1.62%。这些场景每个调用点每 bar 只执行一次，没有调用新 restore_front，不能直接把耗时差归因于 bit 比较；具体原因尚未确定。没有覆盖原批、挑选更快批次作为唯一结论，或据此宣称通用 CPU 吞吐改善。代码布局、系统负载等原因仍需额外归因。

## 剩余语义问题

另一个离线公开 HIR 探针把 5 个 TA 调用的 ID 都改为 u32::MAX，保持 plot ID、参数和执行次序。close=1，使用 series 表达式形成数值输入 WMA(1,2)、WMA(1,2)、SMA(2,2)、WMA(2,2)、SMA(99,2)。在封存基线与候选 release 中，直接观测到同样的 typed NA/1/1.5/2/50。按未变的窗口操作顺序推导末尾内部样本为 [2,99]，其均值 50.5 与已保存 sum=100 不符；此内部状态来自源码推演，公开探针直接观察的是 Float 50。现有 open append 被普通 push 交错后还原了不对应样本的 sum/change metadata。该 reproduction 不计时，单独保留源码、库/程序哈希、实际退出码和原始 stdout/stderr。

复现探针首次使用 const source，被当前分析器以五条 E_CALL_ARG_TYPE 拒绝，尚未执行 runtime append。保留原源码、两版已链接程序、link identity、stderr/stdout、当时 qualified source manifest 等 13 项旧产物及实际 runner/probe exit 1。只把忽略目录里的探针 source 改为 close/close+1/close+98，保持数值 1/1/2/2/99；重新链接两版后成功复现。生产源码、门禁来源、正式 window fixture 与二进制都未改；独立审计验证失败和修复后的差异。

独立审计完成后的终端辅助摘要读取曾默认使用 Windows GBK，读取 UTF-8 收据失败；改为 UTF-8 后，又因猜测状态为 PASS 而拒绝了实际的 PASS_UNCHANGED_EXISTING_RELEASE_BEHAVIOR。辅助读取按实际 schema 修复并通过，原 audit、门禁、源码和采样字节不变。额外本地诊断收据保留：第一次分组命令最终退出 0，但中间诊断子进程的 exit 未单独捕获；第二次辅助断言实际 exit 1。它们发生在主审计后，未宣称由该次主审计独立复核。

正常 analyzer 为每次调用分配独立 ID，未发现正常源码的跨 callee ID 重用路径；runtime 的 manual-HIR 派发明确容纳重复 ID。debug 的计数下溢仍为源码推演，本轮仅实测 release 的错误结果。分离 per-bar 和普通 key 可以阻断这类混用，但普通有状态 callee 之间的 key 所有权也需定义；应作独立语义修复，并明确手工 HIR 的可观察兼容变化。计算状态 checkpoint 目前使用 Clone，未找到内部状态跨版本反序列化合同，不能无理由更改公共输出 schema。

页目录 COW 的 O(L/128) 引用复制仍保留；一般 WMA/HMA、variance 扫描受数值操作顺序约束。Map/Matrix 二级字符串分派和递归 HIR 执行仍需实际热点归因。本轮没有覆盖 TradingView parity、host/strategy/request 性能、RSS、完整资源矩阵或发布资格。

原始数据与 probe 源码：[WINDOW_UNDO_RESULTS_20261005.json](WINDOW_UNDO_RESULTS_20261005.json)。JSON SHA256 `1c48702766ccadfba2d7c1b26ccde5d3068d790e866d93f383d1f409bf2f61b8`。
