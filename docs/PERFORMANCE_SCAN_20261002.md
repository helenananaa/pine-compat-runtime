# Pine 解释器与测试流程性能扫描

本报告回答 2026 年 10 月 2 日的两个问题：解释器有哪些真实的性能债，以及为什么上一轮 Agent 的测试耗时很长、资源占用很大。用户进一步说明，主要观察到的是 Agent 跑测试时的成本。因此，测试流程是本次归因的主线，解释器的一般性能风险单独列出。

结论：上一轮运行的高成本有很大一部分来自反复执行完整验收、重复保存完整结果、全文件读取算哈希，以及未单独计时的历史控制和验证工作。不能把这些成本全部归为单次 Pine 解释执行。核心也存在值得修的历史缓冲、对象回收和元数据查找问题；本次已用小型探针确认其中几项，但它们并不是此前整轮验收耗时的已证明主要原因。

此次没有修改运行时、绑定或测试流水线代码，没有重跑完整验收，没有删除任何旧证据。新增此报告和 `.local/scan-20261002/` 下的诊断工具与结果。

## 当前版本与证据范围

扫描基于 `a0184633ae267be13aefda083a880ca7786f605c` 上的当前工作区。开始和结束时核对了最新 V14 结果中的 **533 个核心文件，全部 SHA-256 一致**。额外核对 source identity、冻结预算、计划、汇总及六个 surface report 的哈希，全部一致。

最终资源矩阵已完成，不再处于前一次对话所见的待填写状态：

- 216/216 个 trial 完成，单项时间和 worker RSS 均在冻结预算内。
- 最差 live phase P95 为 1.107369 ms；这是指定 RSI/Pivot 离线工作负载的结果。
- 总体仍为 `notPassed`：Linux Python、Pivot、四会话的 forming/replacement/confirmation 历史增长比分别为 4.288、4.578、4.456，超过限值 4。
- 对应长历史单次更新中位数约为 0.37、0.40、0.43 ms；增长失败不等于每次更新耗时数秒。
- 144 项为经过复用证明保留的 V13 Rust/WASM 结果，72 项为 V14 新跑的 Python 结果。没有把保留结果当作 216 次新跑。
- 当前单会话 worker 峰值约 196–486 MiB；四会话最大约 1,275 MiB。旧记录中的 9.21 GiB 不能直接当作当前核心占用。

本次复核的是现有收据、当前源代码和额外小型探针；没有重新执行 7,047 个 Rust 测试或重新证明整个兼容性矩阵。

来源：[最新资源结论](PERFORMANCE_RESOURCE_CLOSURE_20261002.md)、[机器可读结果](PERFORMANCE_RESOURCE_CLOSURE_RESULTS_20261002.json)、[本次版本及工具凭据](../.local/scan-20261002/scan-provenance.json)。

## 测试规模本身已经足够解释长时间运行

最终采用的 216 项由三种脚本设置、三档历史长度、单会话与四会话、Windows 与 Linux、Rust/Python/WASM、两次重复构成。它们并不等价于只运行一次指标。

| 最终采用的 surface | Trial 数 | 累计 trial wall |
| --- | ---: | ---: |
| Windows Rust | 36 | 18.01 分钟 |
| Windows Python | 36 | 16.32 分钟 |
| Windows WASM | 36 | 10.05 分钟 |
| Linux Rust | 36 | 30.80 分钟 |
| Linux Python | 36 | 15.84 分钟 |
| Linux WASM | 36 | 13.12 分钟 |
| 合计 | 216 | 104.15 分钟 |

10 万根历史这一档占 72/216 项，却消耗 92.66 分钟，即 **88.97%** 的累计 trial wall。所有 `matrix-*` 收据中，按对应输出路径去重后，共有 744 项带 wall 的运行，累计 **5.563 小时**。这个数还不包括构建、functional gates、最终 audit、额外诊断和没有完整落盘的中断。

以上是各条运行记录的耗时之和，不是 Agent 从开始到结束的实际钟表跨度，也不是 CPU 时间。

原生 worker 每个会话执行实时 seed/tail、replica 检验，以及 seed+append、完整 batch、known-context step 三种历史控制。RSI 的 100k+10k 案例约执行 460k 次 chart bar/tick 求值：100k live seed、30k live ticks、330k historical controls。单独 live 的工作量是 130k；控制组令 chart 求值工作量约为其 **3.54 倍**，尚未计入请求上下文内的求值。

来源：[逐项统计](../.local/scan-20261002/matrix-cost.json)、[所有候选累计](../.local/scan-20261002/matrix-all-recorded-cost.json)、[原生控制组实现](../scripts/product_resource_probe.rs)。

## 约一半耗时没有细分阶段

从最终 216 项 metadata 的原始计时数组累计得到：

| 工作 | 累计时间 | 占 trial wall |
| --- | ---: | ---: |
| seed，包含对应接口要求的结果物化 | 17.10 分钟 | 16.42% |
| forming、replacement、confirmation、request | 23.62 分钟 | 22.68% |
| replica apply | 3.43 分钟 | 3.29% |
| live snapshot 和 serialization | 9.02 分钟 | 8.67% |
| compile、append tail、known-context tail | 1.00 分钟 | 0.96% |
| 未单独分段的剩余工作 | 49.97 分钟 | 47.98% |

剩余部分包含历史控制的 seed/batch、replica 完整序列化、报告复制、比较、哈希、进程启动和控制程序校验。不能把这 48% 全算成 I/O，也不能全算成解释器执行；目前计时边界不足以进一步分摊。

例如 Linux Rust 的 RSI default、100k+10k、四会话 repeat 0，总 wall 249.62 秒，已计时阶段合计只有 54.28 秒。其余 195.34 秒缺少足够细分。优化前应先把这些阶段单独计量。

## 完整输出重复保存造成明显放大

110,000 根输出的一路 RSI default JSON 为 51,071,095 字节，alternate 为 54,685,058 字节；Pivot 为 550,457 字节。完整输出规模和指标计算状态规模是两回事。

最终采用的 216 项，按路径去重后的 2,274 个证据文件共 **36.694 GiB**：

| 文件类别 | 逻辑大小 |
| --- | ---: |
| 主报告内嵌完整结果 | 14.892 GiB |
| 1,620 个完整 spool 输出 | 21.632 GiB |
| metadata、progress、汇总 | 约 0.170 GiB |

原生 RSI default 四会话的一个长案例，主报告约 821 MB，20 个 parts 文件约 1,022 MB，合计约 1.716 GiB。worker 先保存每份结果，再把 live 和三种 historical control 完整复制进主报告；后续又读取和算哈希。

依据已保存的输出 SHA-256 分组，1,620 个 spool 只有 **40 种不同内容**，这些不同内容合计约 **0.485 GiB**。最大的约 51/55 MB 输出各保存了 72 份。这次只读取已验证 surface report 中的哈希并检查文件大小，没有重新散列全部 21.63 GiB，因此这是可信收据下的去重潜力统计，而不是一次完整的内容重验。

整个 `.local/resource-follow-up-20261002/` 目录还保留了旧候选、失败和诊断，共 51,964 个文件，文件逻辑大小合计 **146.86 GiB**。这是磁盘文件大小之和，不是当前 RAM，也不是 NTFS 实际分配簇大小。

应保留每次独立产生和比较的结果。归档时可以用完整内容文件加 trial manifest，避免主报告再次内嵌结果，并在验证后按内容寻址去重；不能以相同哈希替代所需的独立执行和数值对比。

来源：[去重潜力统计](../.local/scan-20261002/matrix-spool-dedup-potential.json)、[目录统计](../.local/scan-20261002/disk-inventory.json)。

## 当前哈希工具会制造数百 MiB 的额外分配

这是本次直接验证的测试工具问题。`scripts/requalify_core_scripts.py:27` 的 `sha()` 使用 `Path.read_bytes()`，先把整个文件读进内存，再算 SHA-256。resource controller 和最终 audit 都复用这个函数。

使用现存的最大选中报告 `rsi-alternate-100000-4-1.json`，文件 883,131,313 字节，在两个新 Python 进程中比较：

| 实现 | Python tracemalloc 峰值 | SHA-256 |
| --- | ---: | --- |
| 当前完整 read_bytes | 883,140,633 B，约 842.23 MiB | 一致 |
| 1 MiB 分块读取对照 | 2,106,982 B，约 2.01 MiB | 一致 |

这不是估算，也不需要更改任何 Pine 语义。两次耗时约 1.29 和 1.31 秒，不据此声称分块一定更快；文件缓存和一次性重复不足以判断速度。确定的收益是将辅助校验的分配峰值从文件大小级降为块大小级。

这部分分配发生在 controller/audit 中，**不属于 worker RSS 预算**。因此即使报告中的 worker 全部低于预算，用户仍可能观察到整个测试流程更高的内存占用。

此外，`audit_resource_matrix.py:89` 通过 `read()` 完整解析 live JSON，保留一个 baseline 与当前结果；隔离比较还会解析 singleton。`read()` 本身采用 `read_text` 加 `json.loads`，临时字符串与 Python 对象树会重叠。`differences()` 对每个叶子递归构造路径、结果列表；即使全相等，也有 Python 调度和临时分配成本。这里尚未跑阶段内存 profile，不能给出其精确 RSS。

来源：[哈希探针](../.local/scan-20261002/hash-probe.py)、[实测结果](../.local/scan-20261002/hash-results.json)、[audit 实现](../scripts/audit_resource_matrix.py)、[共享 read 和 sha](../scripts/requalify_core_scripts.py)。

## 构建与文件系统成本没有消失

`scripts/verify.ps1:55` 开始依次运行 fmt、workspace/all-target Clippy、workspace tests、结构和工具测试、host parity、真实 WASM/Node，以及重新打包和安装 Python wheel 后测试。资源候选构建又需要 release native、WASM 和 wheel。

最终 Windows gate 的 86 个 Rust 测试程序所报执行时间合计约 35.16 秒，而四条构建完成记录合计 87.72 秒；Python 776 tests 为 13.08 秒。这些数是日志中的局部时长，不能简单等同于整个 gate。目录保留了 27 份 gate 日志、32 份 build 日志，也不代表每份都完整成功。

本次仅 fresh release 重编 runtime、其他依赖已缓存，仍花了约 1 分 37 秒。调试构建和 release、Windows 与 Linux、native 与 WASM 的工作并不能全部互相复用。编译器 CPU 高和内存高不能直接说明被编译程序运行慢。

现有 Linux trial 命令明确通过 WSL 访问 `/mnt/e/...`。同一 RSI default 长案例单会话的 live serialization，Windows Rust 为 0.480 秒，Linux Rust 为 5.569 秒；计时包括文件写入和 flush。同期 seed 分别约 4.593 和 3.313 秒。这提示序列化/文件系统路径值得检查，但还没有 WSL 本地目录或内存 sink 的配对，不能把全部差异判定为挂载盘问题。

来源：[gate 汇总](../.local/scan-20261002/gate-summary.json)、[verify 脚本](../scripts/verify.ps1)、[候选构建脚本](../scripts/build_resource_artifacts.py)。

## 核心中已复现的结构性问题

以下是一般脚本风险。它们不能自动解释此前 RSI/Pivot 资源矩阵的全部耗时。原生探针复用此次从当前源码新构建的 release 库；heap probe 对分配加了计数，绝对耗时不适合与正式预算比较。28 个工作负载分别在新进程运行，clone/update 取进程内 31 次中位数；seed、单次 result 只测一次。execution probe 另有三次重复。

| 问题 | 当前源码机制 | 本次实测 |
| --- | --- | --- |
| 静态历史窗口逐 bar 搬移 | `series.rs:73` 使用 Vec 头部 drain | 50k 次 SeriesStore 提交，深度 1 为 1.59 ms，深度 5,000 为 195.57 ms；完整 50k bars 的 close[1] 为 314 ms、close[5000] 为 516 ms |
| 符号元数据重复线性查找 | `persistence.rs:62`、`context.rs:330` 每声明/赋值扫描符号表 | 同样 2,000 bars，400 个简单变量 302 ms，800 个 996 ms；变量翻倍、时间约 3.29 倍。也混合 hash/排序成本，未归属到单函数 |
| 动态历史的 checkpoint 深复制 | SeriesStore 为普通 HashMap 加 Vec；动态 offset 未声明上限时保留全史 | `bar_index % 100` 的实际偏移小于 100，16,384 bars 后仍保留 16,384 个值；clone 新增堆分配 527,264 B。显式 max_bars_back=100 后只有 100 个值、clone 6,176 B |
| 临时 map/matrix 不回收且 checkpoint 深复制 | 普通 HashMap 内含 Vec payload，array GC 不回收它们 | 每 bar 只创建一只临时 2×2 matrix，16,384 bars 后仍有 16,384 只、65,536 cells；一次 clone 4,231,072 B 和 16,391 次分配。临时单项 map 一次 clone 2,396,064 B |
| UDT 不回收 | 持久化共享树只解决复制，不删除旧身份 | 两 float 的临时 UDT，16,384 bars 后 retained heap 约 4.08 MiB，普通 scalar 对照约 0.53 MiB；clone 仍很小，说明共享有效但累计保留存在 |
| 最大显示对象数不限制历史扫描 | labels 旧身份和 snapshots 保留；new/evict 反复扫描全部历史 | 每 bar label.new，max_labels_count=1，1,024 bars seed 约 10 ms，16,384 bars 约 3,445 ms；后者 retained heap 约 37.69 MiB。此项为一次计数探针测量，非稳定分布 |
| 完整 update 随输出放大 | update 调 result，apply_update 返回 delta | 单条 plot、100k bars、无限输出：一次 apply 分配约 12.5 KiB，完整 update 分配约 6.11 MiB；计数探针中位数 0.119 vs 3.048 ms |

输出保留 64 bars 的 realtime map/matrix 探针仍保留上述集合，证实显示窗口不等于总堆内存上限。chart_bars 还会为历史修订保留原始 bar，这属于现有功能契约。

临时数组对照在 16,384 bars 后只保留一只数组；普通 scalar 的 runtime clone 只有约 2.6 KiB。因此不能泛称所有集合都不回收，或所有 runtime clone 都复制全部输出。此前数组 GC、共享 plot history、UDT payload 和 broker 历史的优化确实有效。

来源：[执行探针源码](../.local/scan-20261002/execution/harness.rs)、[执行原始结果](../.local/scan-20261002/execution/results.jsonl)、[heap 探针](../.local/scan-20261002/heap-probe/main.rs)、[heap 原始结果](../.local/scan-20261002/heap-results.jsonl)。

## 其余源码发现与边界

| 发现 | 触发与成本 | 证据与限制 |
| --- | --- | --- |
| request fallback 仍存在 | loops、nested request、endpoint 等不能增量时重新求整段；增长上下文可能产生平方级累计成本 | `request_incremental.rs:54`。标准 Pivot 对 scalar UDF/time 的旧回退已修，不能继续用其解释当前标准 Pivot |
| request 静态依赖重复分析 | 增量入口重复收集 initializers、tuple dependencies、captures；requested bar 重收集符号 | `request_incremental.rs:43`、`requests.rs:838,937,1006`；占比未量化 |
| series 每 bar 整理两次 | 构建 ID Vec 并排序；表达式反复扫描 series_history | `context.rs:346,356,368,397`；可提前缓存计划，但未给加速倍数 |
| 部分 TA 扫描或排序整个窗口 | highest/lowest O(L)，median/percentile/RCI 常见 O(L log L) | `ta/flow.rs:782`、`ta/statistics.rs:274`；SMA/EMA/RSI 已有增量状态，不能一概而论 |
| 大数组写一个元素也会复制 payload | checkpoint 共享 Arc<Vec>，第一次写 make_mut 复制整个数组 | `id_store.rs:65`、`arrays/store.rs:158`；未实测当前 workload 占比 |
| map 查键使用顺序 Vec | get/put 线性寻找键，大量不同 key 插入可能平方级 | `builtins/maps.rs:97,127`；应兼顾插入顺序语义 |
| 内置函数按字符串逐类尝试 | 约 20 类处理器依次拒绝/接受 | `runtime/calls.rs:13`；固定常数开销，优先级低于上述增长问题 |
| 内存 profile 有盲区 | 没有 map/UDT/chart_bars/嵌套 request runtime 的完整计数；树 capacity 不含节点头 | `profile.rs:10`、`runtime/profile.rs:14`、`append_history.rs:50`；不能靠少数低计数宣称有界 |

## 绑定和接口仍有可避免成本

Python 完整 snapshot 仍然是 Rust owned result 加独立 dict/list；当前优化逐 series 释放和复用连续相等的不可变标量，未改变全量输出的规模下界。WASM 常规 snapshot 为精确分配容量先编码计数、再正式编码，最后还要传入 JS String；消费型 replica 则用 chunks 后合并，降低源数据与结果重叠的同时付出一次合并复制。

测试 worker 调 seed 后立即丢弃公开结果，接着创建 replica 又物化一次。可新增“不返回完整结果”的 seed 路径，同时保留原 API，避免调用者不需要的结果。CLI 仍使用返回完整 String 的旧序列化接口，而原生 resource probe 使用新的 sink；原生矩阵资源通过不自动证明 CLI 具有相同峰值。

Python 入口内直接执行 Rust 运算，源码未释放普通 CPython 的 GIL。因此同进程多个 Python 线程不能据此获得独立 session 的 CPU 并行。矩阵中的四会话是串行交错调用，不是四线程吞吐验证。这里没有做新的并发性能对照。

位置：`pine-python/src/realtime.rs:155,237,289,387`、`pine-python/src/outputs/owned.rs:4`、`pine-wasm/src/snapshot.rs:8`、`pine-cli/src/commands/run.rs:136`、`product_resource_acceptance.py:132`、`product_resource_wasm.cjs:88`。

## 建议实施顺序

针对用户实际看到的测试成本，首先处理测试工具，后处理核心结构；无需推倒重写解释器。

1. **流式哈希，补齐控制程序和 audit 的内存计量。** 保持相同 SHA-256，消除已实证的 842 MiB 临时分配。worker、controller、audit 和构建进程分开记录，避免只看 worker 峰值。
2. **主报告使用版本化 manifest 引用完整 spool。** 保留每个 trial 的完整输出、上下文、阶段和哈希；消除目前约 14.9 GiB 的再次内嵌。完成比较后再设计按内容寻址的归档，旧证据不原地改写。
3. **补上未计时的 48%。** 分别记录 historical controls、replica export、文件写入、报告组装、哈希与 audit；进行 WSL 本地文件系统和挂载盘的同负载配对。不得把纯编码和含 I/O 的指标混比。
4. **建立快速预检与最终资格验证两层流程。** 修改期间先跑相关回归与热点资源预检；候选稳定后跑完整冻结矩阵。核心、构建、输入或测量边界变化时仍须重新验证；已有严格证明的无关 collector 变更才复用收据，保留新旧身份。
5. **核心优先修有增长性的结构。** series 环形/共享存储、map/matrix/UDT 可达性回收和 checkpoint、绘图活跃 ID/脏变更索引、符号与 request 依赖预计算。之后再考虑大数组页级写时复制、TA 算法和 builtin ID。
6. **保留原语义回归作为重构约束。** 回收要覆盖历史、varip、slice、request capture 和别名；数值算法要保持 Pine 数值/na 行为；数据结构更换不能通过减少完整结果字段来伪造资源收益。

目前仍未确定那三项 Linux Python Pivot 增长失败的单一原因，也没有新的无限会话或任意脚本内存保证。这些限制不影响本次已经确认的测试工具放大和核心结构问题。
