# 解释器核心性能修复与验证

后续的绘图 checkpoint、大数组和历史树修复及其独立验证见 [实时绘图与大数组性能修复](STATE_PERFORMANCE_REPAIR_20261003.md)。本文保留上一轮候选的测量和资格边界。

本次接续 [性能扫描](PERFORMANCE_SCAN_20261002.md) 和 [测试工具降本修复](RESOURCE_TOOLING_REPAIR_20261002.md)，处理解释器本身的历史复制、重复查找、集合回收和绘图扫描。改动保持核心独立于宿主，使用原冻结资源输入与预算，没有缩短历史、tail 或删减完整输出。

## 修复内容

- **历史存储**：深度不超过 128 的小窗口使用环形队列；更深和动态历史使用现有持久化历史结构。提交不再移动整段窗口，checkpoint 共享历史页。逻辑历史窗口、动态回看、分支隔离和 NaN 相等行为均保留。
- **运行时元数据**：预先建立共享、不可变的符号索引、历史需求和持久槽位表，避免逐 bar 反复线性查询；历史提交和资源预算投影共用一次排序结果。稀疏与重复符号 ID 仍遵循原查找语义。
- **集合存储和回收**：map、matrix、UDT 及字段 varip 元数据使用持久化 ID 存储，checkpoint 共享未修改对象。周期性标记回收不可达集合，遍历有效历史、嵌套引用、循环、切片父数组、调用状态和请求捕获。对象 ID 单调分配，回收后不复用。
- **实时和请求边界**：形成中 bar 保留字段 varip 对象及其嵌套图，普通字段继续回滚；父子请求运行时分别管理 ID 空间，请求返回值在子运行时回收前冻结为原协议允许的自有值。
- **绘图索引**：label、line、box 用共享的活动 ID 集合维护数量、淘汰和 `*.all`；按 ID 查找使用二分。完整历史仍保留，旧 ID、删除、复制、淘汰、保留策略与形成中回滚有回归覆盖。

相对修复前的核心候选，19 个既有文件修改、3 个文件新增、514 个文件不变，当前核心身份共 536 个文件。清单见 [源码差异](../.local/core-performance-20261003/source-delta.json) 和 [源码身份](../.local/core-performance-20261003/source-identity.json)。

## 同条件前后测量

前后探针使用相同 harness、输入和 `rustc --edition=2024 -O -C lto=thin -C codegen-units=1`。旧 release 库已核对此前构建凭据；新旧探针及库的哈希记录在 [探针身份](../.local/core-performance-20261003/probe-provenance.json)。每个工作负载、尺寸分别运行三个新进程，以下取中位数。

| 诊断负载 | 修复前 | 修复后 |
| --- | ---: | ---: |
| 16,384 bars，每 bar 新建 label、活动上限 1，历史执行 | 2,602.18 ms | 55.22 ms |
| 800 个简单变量、2,000 bars，执行 | 532.12 ms | 121.98 ms |
| 深度 5,000，50,000 次直接 history commit | 123.08 ms | 23.00 ms |
| 深度 5,000，50,000 bars 完整脚本 | 296.34 ms | 199.11 ms |
| 16,384 bars 动态历史，单次 checkpoint 新分配 | 527,264 B | 3,040 B |
| 每 bar 临时 map，16,384 bars 后保留堆 | 6,097,756 B | 564,395 B |
| 每 bar 临时 matrix，16,384 bars 后保留堆 | 4,787,036 B | 562,979 B |
| 每 bar 临时 UDT，16,384 bars 后保留堆 | 4,283,052 B | 566,365 B |

matrix 案例在测量端点的对象槽位从 16,384 降为 1；回收按周期执行，这不表示任意时点都只有一个槽位。动态历史仍保留全部 16,384 个值。label 案例保留堆仍约 39.5 MB，checkpoint 分配仍为 658,032 B，加速来自扫描路径的修复，未删除历史来换取数字。

存在局部取舍：深度 1,024 的 50,000 次直接 history commit 从 7.59 ms 增到 8.58 ms，约慢 13%；同深度完整脚本从 209.98 ms 降到 179.28 ms，约快 15%。树路径、引用计数和分配可能增加固定开销，但尚未单独归因。动态历史案例保留堆从 1,257,820 B 略增至 1,274,284 B，降低的是 checkpoint 复制成本。全部原始数值见 [诊断对照](../.local/core-performance-20261003/hotspot-comparison.json)。这些堆数据来自 counting allocator，**不是 RSS**；诊断结果不能代替完整资源资格。

## 功能验证

Windows 和 Linux 的完整检查均通过，各包含：

- 7,063 项 Rust 测试；166 项工具测试；776 项实际安装 wheel 测试。
- 实际 WASM/Node 加载、分析、执行、编译后执行及宿主异常烟测。
- 940 个 CLI runtime 快照，以及 Python/WASM 所需的 591 个 runtime、5 个 legacy-analysis golden 断言。
- 372 个生产 Rust 源文件的结构检查，以及格式和 clippy 检查。

新增回归覆盖历史窗口边界、动态历史和 checkpoint 独立性；无 array 分配时的临时集合回收；保留历史引用和循环图；varip 嵌套图与普通字段回滚；请求运行时 ID 隔离和跨回收阈值的返回数组；绘图 ID、保留策略及实时回滚。原 matrix profile 测试把“累计泄漏全部临时矩阵”写成了期望，本次改为验证所有 512 个输出仍为 9、临时槽位有界及原容量限制，未调整资源验收预算。

完整日志：[Windows](../.local/core-performance-20261003/gate-windows.log)、[Linux](../.local/core-performance-20261003/gate-linux.log)。修复过程中暴露的旧泄漏断言、测试夹具错误和未使用方法检查失败日志保留在同一证据目录，未覆盖。

## 完整资源验证

用上述冻结核心重新执行 Windows/Linux × Rust/Python/WASM × 原 18 个配置 × 两次重复，**216/216 轮全部通过**，没有复用旧轮次。各轮串行运行，保留原 1,024、16,384、100,000 根历史与 128/10,000 根 tail、单/四独立会话、完整 live/replica/原生控制输出。

[矩阵审计](../.local/core-performance-20261003/matrix/summary.json) 状态为 `passed`：资源与执行失败 0、未验证项 0、540 份跨平台 live 输出差异 0、108 次单/四会话隔离比较差异 0。额外将所有 live、replica、原生控制文件与修复前核心比较，**1,620/1,620 份完整输出逐字节相同**，未使用数值容差放宽。见 [完整输出对照](../.local/core-performance-20261003/baseline-output-comparison.json)。

| 测试面 | 完成轮次 | 最大长短历史延迟增长比 | 测试进程峰值 MiB |
| --- | ---: | ---: | ---: |
| Windows Rust | 36 | 3.191 | 1,025.03 |
| Windows Python | 36 | 2.495 | 1,097.00 |
| Windows WASM | 36 | 1.705 | 910.67 |
| Linux Rust | 36 | 3.327 | 1,236.04 |
| Linux Python | 36 | 2.713 | 1,278.11 |
| Linux WASM | 36 | 1.468 | 1,105.37 |

增长比冻结上限为 4。峰值包含完整测试输入、输出与校验，不是单个解释器的常驻内存；Windows 使用进程树峰值工作集之和的上界，Linux 使用 VmHWM，计数口径不同。完整轮次的 worker 加 controller wall 合计约 63 分钟，统一审计约 65 秒。没有并发编译或其他基准；期间两次短时文件检索造成的潜在 I/O 干扰如实记入 [总凭据](../.local/core-performance-20261003/repair-results.json)，没有覆盖收据或重跑以追求通过。矩阵用于原预算资格，不作为严格隔离的加速比实验。

这份完整矩阵是在下述输出写缓冲补充改动之前冻结并审计的。核心 536 个文件保持相同；写缓冲改动另列诊断验证，不冒称重新完成一套 216 轮矩阵。

此前 Linux Python Pivot 的三个增长比失败没有被覆盖。本次新核心的四会话 forming/replacement/confirmation 增长比分别约 2.29/2.34/2.33，request 约 2.71。在同一 collector 下，用旧二进制重新执行原短/长四会话输入各两次，前三项也只有约 2.15/2.19/2.22，request 约 3.60。见 [旧二进制复测](../.local/core-performance-20261003/pivot-before/results.json)。因此旧失败没有稳定复现，不能把增长比波动归因于某个已证明的代码缺陷，也不能把新矩阵的通过全部归功于核心改动。

### 为什么完整测试仍然耗时

此次 Windows 原生 `rsi-alternate-100000-4-1` 收据记录总 wall 77.375 秒，其中 worker 75.859 秒。worker 的额外历史 batch、known-prefix、seed 控制回放分别耗时 14.355、12.700、12.800 秒，合计约 39.86 秒；主 manifest 写入约 0.473 毫秒。controller 的结果校验、重复比较、输出哈希另耗时约 0.500、0.094、0.922 秒。

因此完整验收的耗时包含多份历史控制计算与完整结果处理，不能把 77 秒全解释为一次实时脚本执行。这些控制仍保留以验证语义。日常局部改动可使用已有 [快速预检](RESOURCE_PRECHECK.md)，但它不替代本次核心存储改动所需的完整跨平台验证。

### 补充修复：Linux 原生测试写出

测量期间观察到 Linux worker 在 WSL 的 `p9_client_rpc` 等待；相同 RSI default 长案例的历史序列化，Windows 约 3.48 秒，Linux 约 60.65 秒，而 Linux 的 seed 计算更快。完整输出当时使用默认小块写缓冲，跨文件系统写出是另一个具体成本。

在完整矩阵审计后，单独将原生 spool 缓冲提升到 1 MiB，两平台各跑三个短案例与一个四会话长案例、每个重复两次。16/16 轮完成，**140/140 份完整输出逐字节相同**。RSI alternate 的 100,000+10,000 bars、四会话对照中位数如下：

| 平台与指标 | 默认缓冲 | 1 MiB 缓冲 |
| --- | ---: | ---: |
| Linux，整轮 wall（包含 controller 校验与哈希） | 173.56 s | 99.21 s |
| Linux，历史控制序列化总时间 | 53.99 s | 6.45 s |
| Linux，live 序列化 p50 | 3,520.11 ms | 548.07 ms |
| Windows，整轮 wall | 75.70 s | 89.31 s |
| Windows，live 序列化 p50 | 340.45 ms | 359.96 ms |

Linux 两组峰值均约 1,236 MiB，没有通过省略结果减负。Windows 没有观察到收益，因此**最终仅 Linux 使用大缓冲，其他平台保留原来的 `BufWriter::new` 路径**。这组诊断没有单独分离 Windows 的 CPU/环境波动，不能将端到端变化全部归因于缓冲；也不据此声称 Windows 测试变快。

最终平台限定版本重新构建，两平台各 3 项原生报告单测通过；另跑 Windows 三个短案例和一个四会话长案例、Linux 三个短案例，**7/7 轮通过，50/50 份完整输出逐字节相同**。Linux 大缓冲的长案例性能证据来自前面的两次重复，最终验证不冒称另一套完整资源矩阵。原 uniform 候选、较慢的 Windows 测量和原矩阵均保留。见 [缓冲对照](../.local/core-performance-20261003/buffer-output-comparison.json) 与 [最终版本输出核对](../.local/core-performance-20261003/buffer-final-output-comparison.json)。

当前核心未变时，Windows 的日常原生快速预检可复用最终工件；输出目录必须是新目录：

```powershell
python scripts/resource_precheck.py --input-root .local/core-performance-20261003/matrix --artifacts .local/core-performance-20261003/buffer-final-artifacts/windows --output .local/precheck-core-next
```

核心代码再变更后须重建工件，预检会拒绝源码哈希不匹配。原矩阵绑定修改前 collector，不能用修改后的 collector 直接恢复那份 plan；它的源码快照、输入、工件及审计结果均保留。

## 尚存边界

- 绘图历史的完整 Vec 复制，以及实时 cursor 捕获与差异计算，仍可能随保留历史增大。此次修复活动对象查询，没有消除完整历史的成本。
- 动态历史和完整快照按语义保留数据，内存仍随实际保留量增长。周期回收不会删除仍可达对象，也不承诺无限会话或任意输入的恒定内存。
- 历史树前缀裁剪仍有分支管理成本；metadata 优化将每 bar 排序由两次减为一次，没有消除排序和预算投影。GC 根遍历也仍需访问有效历史。
- 大数组的页级写时复制、无输出 seed 路径、Python GIL 和部分 TA 算法不在此次修复范围。
- 合成资源输入验证当前冻结范围；不自动构成所有 TradingView 脚本兼容、真实宿主负载或发布资格证明。
