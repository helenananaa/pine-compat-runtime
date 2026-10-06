# 第二轮解释器技术债修复与验证

已修复第二轮审计的 12 项问题，覆盖前端栈保护、执行预算、集合资源边界、request 修正、失败生命周期和公共分析结果。Windows/Linux 完整门禁通过；本轮仍为未提交工作区，基线为 `e155e18bf493c45326d7eb2294da072cc75dd6d5`。工作始于 2026-10-03，收尾于 2026-10-04。

| 审计项 | 最终行为与回归证据 |
| --- | --- |
| 1. 解析递归导致进程栈溢出 | 语句深度 32、表达式深度 64，共享 `2 * stmt_depth + expr_depth <= 64` 预算。1 MiB 线程覆盖 else-if、嵌套 if、inline switch、括号、单目及混合结构。超限返回诊断；16 层结构与 32 层括号/单目仍接受。语义深度保护用浅解析、深 AST 的左结合表达式独立验证。 |
| 2. for 无界及嵌套循环总量 | 每 chart bar 默认 10,000,000 求值步骤、1,000,000 次合计循环；普通 for/while 另有 100,000 次局部限额。保留 v6 动态边界，request 全历史/增量/有界路径、foreach 和填单重算共享父预算。11 项集成回归。 |
| 3. array.join 先展开再检查 | 借用元素与 UDT 字段，在追加前检查 Unicode 字符数；共享 UDT 图验证有成功节点 memo，循环/非法引用仍拒绝。str.repeat 空结果直接返回，耗时不再随 repeat count 增长；保留实参求值、类型与负数检查。 |
| 4. request 裁剪丢失 intrabars/保留未来 HTF | 以保留 chart 末柱的 nominal close 为边界，月周期使用日历。confirmed 前缀末柱必须自行在边界前关闭，避免依赖已丢弃后继而使重复 replay 改变结果；仍保留边界内的 LTF extras 与尚未关闭的 forming request。提前后继、重复 replay、月边界及闰年有回归。 |
| 5. 错误文本被误当 break/continue | 控制信号保存在类型化内部状态，只有执行到相应 HIR 节点才产生；`runtime.error` 的旧 sentinel 字符串照常返回错误。保留公开 `RuntimeError { message }` 构造接口。 |
| 6. pinv 绝对阈值破坏小尺度矩阵 | 先按二进制幂归一化 Gram 运算，按最大特征值的相对阈值截断，再恢复尺度。覆盖 1e-200 至 1e200、秩亏、矩形、subnormal 与伪逆恒等式；10 份已有完整 JSON 快照去除末尾空白后全文一致，未改 golden。 |
| 7. input 元数据常量与 options 不完整 | 有界求值 immutable const、运算、静态内建常量及选定纯标量调用。options 完整解析或保持 unknown；重赋值别名、过大字符串、依赖 chart 的格式与不支持调用保持 unknown。共享 HIR walker，不执行原脚本块/UDF/request。 |
| 8. 库诊断错误源坐标 | 保留物理源身份，CLI/Python/WASM 使用对应源的 UTF-8 字节 offset、Unicode 行列。JSON schema 6 的库 span 增加 sourceId/libraryKey/sourceName，根诊断形状保持原契约；配套 host-parity 检查显式读取 UTF-8 源码，避免 Windows 默认 GBK 解码失败。 |
| 9. map 缺少容量限额 | 最多 50,000 键值对；满容量可以覆盖旧键。put_all 预检新增不同键，超限不留下半次合并；自合并为无操作。4 项容量回归。 |
| 10. historical 失败后隐藏状态污染 | 真正执行失败后低层 HistoricalRuntime 进入不可继续执行状态，后续返回 E_RUNTIME_POISONED；输入预检失败可重试。避免每 bar 克隆整个 evaluator。RealtimeRuntime 在候选对象求值，失败保留先前有效 session。 |
| 11. 非尾部 slice concat 重复复制父数组 | 一次批量插入并重建存储，保留 slice alias/checkpoint 行为。计数回归将复制旧格从 1,081,216 降为 4,096；尾部快路径保留。 |
| 12. replay 冗余回滚备份 | 候选对象完成全部可失败工作后提交，删除额外整份 session 备份；非法时钟在 chart Vec 复制前拒绝。资源回归覆盖 20k 历史，以及 2,000 var + 65,536 格 array 的普通 append。 |

主要契约见 [执行限额](EXECUTION_LIMITS.md)、[执行语义](EXECUTION_SEMANTICS.md)、[实时模型](REALTIME_MODEL.md) 和 [诊断及输入元数据](FRONTEND_DIAGNOSTICS_AND_INPUT_METADATA.md)。Rust `Diagnostic` 新增 `source: Option<Box<DiagnosticSource>>`；直接写结构体字面量的宿主需要加 `source: None`，这是 0.3 prerelease API 迁移。

HistoricalRuntime 的失败状态不承诺逐 bar 回滚：失败对象可能留下局部状态或输出，宿主应丢弃并重建；此前取得的 owned result 不会被修改。RealtimeRuntime 的候选提交保证先前 session 保持有效。

以下分配测量使用 Windows release，相同公开 API 探针与 `--edition=2024 -O -C lto=thin -C codegen-units=1`。join 使用 1,000 个字符串与 40,960 字符分隔符；concat 在 4,096 格父数组的非尾部 slice 插入 256 格。每版本/案例 5 个新进程，测量初始化 bar 0 之后的 bar 1 append，排除结果物化与序列化。replay 每版本 1 个新进程，只比较分配量。

| 案例与指标 | e155e18 基线 | 本轮修复 |
| --- | ---: | ---: |
| 超限 array.join，累计分配 | 83,971,121 B | 122,974 B |
| 超限 array.join，新增逻辑存活峰值 | 41,985,090 B | 41,054 B |
| 非尾部 slice concat，累计分配 | 138,834,876 B | 289,192 B |
| 非尾部 slice concat，新增逻辑存活峰值 | 435,848 B | 280,200 B |
| 非法 replay 时钟，累计分配 | 1,127,544 B | 104 B |
| 非法 correction 时钟，累计分配 | 2,879,960 B | 104 B |
| 20k 历史改为 1 bar replay，累计分配 | 1,136,156 B | 9,836 B |

累计分配按 allocator 请求大小计数，realloc 计完整新大小；新增逻辑存活峰值不包括 allocator 元数据或搬迁期间的底层临时缓冲，不是 RSS。concat 的输出仍为父数组长度 4,352、slice 长度 257、插入末值 9、后继原值 7。普通 historical append 的累计分配两版均为 528 B；失败后重试的契约现为显式拒绝。

普通负载开销复测采用 100k 次 historical append，每版本/案例 5 个新进程，交替运行先后顺序，求值时间排除分析、clone 和 result：

| 负载 | 基线中位数 | 修复中位数 | 变化 |
| --- | ---: | ---: | ---: |
| plot | 256.429 ms | 249.507 ms | -2.70% |
| ema | 282.849 ms | 272.307 ms | -3.73% |
| rsi | 279.432 ms | 273.781 ms | -2.02% |
| strategy | 299.113 ms | 294.066 ms | -1.69% |

这些少量进程样本用于观察本轮开销，分布及全部原始记录保留在结果 JSON。它们不能证明普遍加速或零预算开销。

两平台对同一份 6,263 文件冻结源码运行 `scripts/verify.ps1` / `scripts/verify.sh`，包括 fmt、workspace/all-targets clippy、Rust、结构门禁、工具测试、host parity、实际 WASM/Node 和新构建 Python wheel：

| 平台 | Rust 通过 | 工具测试 | Python 通过 | 生产 Rust 文件结构检查 | 完整脚本退出码 |
| --- | ---: | ---: | ---: | ---: | ---: |
| Windows | 7,209 | 166（跳过 0） | 785 | 385 | 0 |
| Linux/WSL | 7,209 | 166（跳过 1） | 785 | 385 | 0 |

没有放宽结构门禁或修改数值 golden。冻结身份 SHA-256 为 `267549b1663a4331447c9fa341d9b18def045877c65cd7aa9fa2d797a3b47f7b`。[可携带结果数据](DEBT_REPAIR_RESULTS_20261003.json) 保存改动文件哈希、门禁日志哈希、探针/rlib/exe 身份与原始测量；完整本机日志与探针在 Git 忽略的 `.local/second-debt-repair-20261003`。本报告及结果数据在代码验证后生成。

剩余性能债包括 highest/lowest 长窗口的 O(L) 扫描、WMA/stdev/mode/RCI 的线性或排序工作、递归 HIR 与调用参数查询，以及完整 public result/序列化复制。pinv 仍为 Gram/Jacobi，相对秩阈值不能替代稳定 SVD。预算计 HIR 工作和循环，不是全部内建 CPU 指令或墙钟超时；宿主生命周期管理仍在核心之外。本轮没有重新跑完整 216 轮资源资格矩阵，也没有新增 RSI/Pivot 长尾内存资格证明。
