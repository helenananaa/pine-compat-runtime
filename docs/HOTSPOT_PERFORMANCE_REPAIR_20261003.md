# 解释器热点修复与验证

本轮修复此前审计确认的长窗口查找、集合复制、策略历史、正则重复编译和运行时边界问题。Windows/Linux 完整门禁通过；与修复前相比，222 组实时对照的 7,326 份完整快照逐字节一致。主要收益在长窗口执行和实时更新，map 索引及分页存储也带来了明确的初始化和内存代价。

修复前基线为 `af636c7456e7a0aa89a5befce06fe3c71b515a00`；修复后为未提交的工作区源码。冻结清单覆盖 5,752 个代码、测试、脚本和夹具文件，身份 SHA-256 为 `be4879ad96bc973bc7289119c9a8fe7f2c6797a8fe8b88866d63b7c0533acbef`。见 [源码身份](../.local/hotspot-repair-20261003/source-identity.json)、[探针及库身份](../.local/hotspot-repair-20261003/probe-provenance.json) 和 [完整结果数据](HOTSPOT_PERFORMANCE_RESULTS_20261003.json)。

## 已修复

- **长窗口**：highest/lowest 及 bars 变体借用历史树的逆序游标，避免每个 offset 从树根查找及克隆值；median/percentile 使用可复用工作区和秩选择，保留 NA、相等值顺序和正负零的位级结果。
- **map**：有序载荷分页共享，持久化 AVL 哈希索引替代全表查找；碰撞仍检查键相等性，删除后重新插入仍排在末尾。删除同步移除索引槽，墓碑按阈值压缩，避免同一键反复删写积累无效槽。
- **matrix**：大矩阵共享 128 元素页，稀疏写入仅复制受影响页和页目录；reshape 只改维度。小矩阵继续使用平坦 Vec。
- **策略历史**：position_size 独立计算所需历史深度，使用共享 AppendHistory；无关的动态 close 历史不再使 position_size 保留整段历史。订单填单后的额外执行及形成中回滚有回归覆盖。
- **GC**：在原 1,024 个新 handle 触发之外，加入默认 2 MiB 的保守分配压力触发。构造、copy、写入、共享页复制及 String 载荷均计账；纯标量历史跳过根扫描。回收仍发生在成功执行完整 bar、提交历史之后。
- **字符串与 request**：每个 callsite 缓存当前正则 pattern；形成中替换复用纯缓存。集合字符串格式化借用载荷。现代 request 新增纯标量字符串调用资格，逐个检查参数类型与依赖，不放行集合、持久状态或嵌套 request；去掉一次重复 tuple 依赖扫描。
- **执行准备**：统一 HIR 遍历，预绑定调用所属命名空间；新增 PreparedProgram 共享 HIR、不可变元数据及保留需求。CompileCache 新增共享分析接口，命中时借用源码计算身份并核对完整键，避免复制源码键及 HIR；旧 owned 接口保留。
- **宿主边界**：running-alert 配置、投递、重试、webhook 等移到可选 pine-host-support；依赖方向仅为 host-support → runtime。核心保留 Pine 事件和中立模板渲染。
- **绑定开销**：Python 纯 Rust 执行与快照阶段释放 GIL；新增无初始结果快照的 seed 接口。WASM streamSnapshot 直接写入最终字符串缓冲，减少完整内层 JSON 字符串的中间复制。

## 同条件测量

Windows release，新旧相同探针、脚本与编译参数 `--edition=2024 -O -C lto=thin -C codegen-units=1`。每侧三轮，顺序为旧/新、新/旧、旧/新，共 66 个新进程。每轮 execution 进程依次跑 27 个固定案例，10 个 heap 案例各用独立进程；取进程间中位数。编译、输入构造和完整结果物化不计入 execution 时间。clone/update 在各进程内另取 31 次样本的中位数。

| 负载与指标 | 修复前 | 修复后 | 变化 |
| --- | ---: | ---: | ---: |
| highest，L=5,000，24,000 bars，执行 | 1,824.21 ms | 452.767 ms | -75.2% |
| median，L=5,000，24,000 bars，执行 | 1,824.12 ms | 592.848 ms | -67.5% |
| 简单 indicator，100,000 bars，执行 | 470.888 ms | 266.231 ms | -43.5% |
| 恒定 str.match，8,192 bars，执行 | 353.198 ms | 23.953 ms | -93.2% |
| map 构建，8,192 个键 | 17.509 ms | 6.707 ms | -61.7% |
| 65,536 格 matrix，单格 forming 更新 | 0.7546 ms | 0.0136 ms | -98.2% |
| 同矩阵 apply_update 分配 | 2,108,240 B | 19,360 B | -99.1% |
| 100k 动态 close 策略，checkpoint 新增堆 | 803,040 B | 2,920 B | -99.6% |
| 同策略 apply_update 分配 | 2,418,828 B | 18,708 B | -99.2% |
| 每 bar 临时 array(256)，1,023 bars 后保留堆 | 8,646,714 B | 2,189,114 B | -74.7% |

GC 案例在 1,023 bars 时保留数组从 1,023 个降到 255 个；1,024 bars 时两版都会触发 handle 周期回收。128 bars 的保留堆基本不变。阈值表示累计分配压力，**不是 RSS、峰值内存或严格的堆上限**；单 bar 循环、合法保留的历史及外部 checkpoint 仍可占用大量内存。

测量没有失败或缺项。全部 37 个案例及所有变慢指标保留于 [完整中位数表](../.local/hotspot-repair-20261003/measurements/median-summary.md) 和结果 JSON；以下是实质代价：

| 代价 | 修复前 | 修复后 |
| --- | ---: | ---: |
| map 512 键构建 | 0.4341 ms | 0.5492 ms |
| map 512 键保留堆 | 44,664 B | 86,597 B |
| map 8,192 键保留堆 | 536,184 B | 1,195,397 B |
| 65,536 格矩阵，初始化加 4 bars 历史执行 | 0.6443 ms | 1.8966 ms |
| 64 格矩阵，checkpoint 后 append | 0.0085 ms | 0.0100 ms |
| 临时数组，1,023 bars 执行 | 6.3116 ms | 7.2889 ms |

索引将 map 查找的增长速度降下来，但增加节点内存，小 map 构建也慢了约 26.5%。分页矩阵的初始化多出约 1.25 ms，换来 checkpoint 后修改一格不再复制完整载荷；它仍需复制页目录，不能称为严格 O(1)。GC 更早回收降低了中途保留量，但增加收集次数。正则缓存多保留约 8.9 KB。微秒级计时及 2%–6% 的变化只作描述，未单独排除调度波动。

## 验证与接口迁移

最终冻结源码的 [Windows 门禁](../.local/hotspot-repair-20261003/gate-windows-05.log) 和 [Linux 门禁](../.local/hotspot-repair-20261003/gate-linux-initial.log) 均退出 0：每平台 7,141 项 Rust 测试、166 项工具测试、结构/宿主对齐检查、实际 WASM 构建及 Node 装载测试、新构建 wheel 的 782 项 Python 测试全部通过。新回归包含别名、checkpoint、无效操作事务、字符串载荷回收、NA/tie/正负零、请求增量与强制完整计算对照、seed 失败原子性、replica 和 GIL 释放。

与基线分别跑 111 个既有脚本的 unlimited/保留 16 bars 两种模式，共 222 组。每组 seed 257 bars，再执行 8 bars，每 bar 三次 forming 和一次 confirmed；live 与 replica 逐步一致，新旧共 **7,326/7,326 份完整公共 JSON 快照逐字节相同**，未使用数值容差。见 [差分结果](../.local/hotspot-repair-20261003/differential-summary.json)。

调用方需要留意：

- Rust 使用告警配置、Delivery、Webhook 辅助 API 时，依赖和 import 改为 pine-host-support；详见 [迁移文档](HOST_SUPPORT_MIGRATION_20261003.md)。CLI/Python/WASM 原入口和 JSON 字段保持。
- Rust 多会话可创建一次 PreparedProgram，再用 HistoricalRuntime::from_prepared / RealtimeRuntime::from_prepared；共享 CompileCache 结果用 analyze_shared / analyze_input_shared。旧构造和 owned 分析接口继续可用。
- Python Program 自动共享程序数据。独立 run/会话执行可释放 GIL；同一 RealtimeSession 必须串行访问，执行期间并发读写可能触发 PyO3 借用冲突 RuntimeError。输入解析与 Python 结果转换仍持 GIL。
- 无需初始完整快照时，Python 使用 session.seed_state(bars, execution_times=...)，WASM 使用 seedState 或 seedStateWithExecutionTimes；Rust 使用 seed_historical_without_output 及带时钟变体。旧 seed 保持返回完整结果。共享准备、无快照 seed 和 WASM 缓冲改动有语义回归，本表没有单独量化这些 API 的性能收益。

## 剩余热点与资格边界

highest/lowest 仍是 O(L) 扫描；WMA/HMA 与 stdev/variance 仍是线性归约，mode/RCI 仍全排序。WMA 递推会改变浮点累加顺序，方差的 sum-of-squares 公式存在抵消误差，mode 的现有 EPSILON 分组不满足等价关系，不能直接替换为哈希计数。

median 工作区保留曾用过的最大容量；跨新 bar/request checkpoint 仍可能重新分配窗口与工作区。正则只缓存当前 pattern，交替 pattern 仍会重新编译，未设置总缓存字节预算。request 每次仍构建依赖计划，端点及其他不符合资格的表达式继续完整回放。调用计划只绑定命名空间，参数查找和递归 HIR 求值尚未改成预绑定参数或字节码。

完整 result/replica 仍复制自有输出；动态策略 update 的分配仍约 10.4 MB，绝大部分是完整结果。WASM 缓冲采用计数再写出的两遍序列化，节省中间字符串并未消除完整快照或序列化 CPU。

本轮完成双平台门禁、语义差分和 Windows 热点诊断；**没有在这份新源码上重跑 216 轮完整资源资格矩阵，也未重新认证原 RSI/Pivot 的长尾增长与峰值预算**。上一轮矩阵仅属于其原冻结源码，本次微基准不替代该资格。源码、测试、迁移说明和汇总结果纳入 Git；完整探针和原始日志保留在本地 `.local`，不包含在源码提交中。
