# 常量窗口的精确快速路径（2026-10-05）

本轮为 WMA/HMA、variance/stdev 及非有限求和恢复加入保守的常量窗口证书。只有能证明原计算结果不变时才跳过扫描；有限的小数残差和不满足资格的窗口继续执行原计算。没有引入 host 依赖或修改公开 HIR/ABI。

基线 `0052cf0583d941202bcc428c928e2a88d8077521`；封存 6,342 份源码和 311 份编译产物。候选 6,345 份源码绑定完整门禁、release、链接和采样；本报告两文件在资格验证之后生成。完整原始指标、逐进程收据、来源 SHA 和 probe 源码见 [结果 JSON](WINDOW_CERTIFICATES_RESULTS_20261005.json)。

## 资格条件与兼容性

- WMA：将有限非零常量写成 `sign × odd_significand × 2^e`。三角数 T 用 u128 计算；只有原始 `odd_significand × T` 不超过 53 位且 `x × T` 有限时，每一项和每个同号部分和才可证明精确，最后除法逐位等于 x。不能只检查最终系数去掉尾零后的位宽；普通十进制常量、±0 和其他未资格样本保留扫描。HMA 必须全窗口与 half window 均通过，保留既有差值与 smooth 窗口更新。

- 方差：先计算原 mean 与 `diff = x - mean`，只有 `diff * diff == 0` 才证明旧扫描每项都为正零，包括平方下溢。保留 unbiased 长度 1 的 NaN、十进制常量的小残差和非有限 mean 处理；没有改成 Q-S*mean 或通用递推。

- 求和：使用实际保留数量 n 的下界 `p = 2^floor(log2(n))`。仅当 `abs(x) > MAX/p` 才证明旧恢复必然拒绝；等号继续扫描，可保持 n=p 时的 ±MAX。没有浮点 count 近似，也没有关闭 weighted 消费者的 sum 维护，因此保留缩窗、共享 key 与 cold tail 恢复路径。

- 普通 push 与 open append 交错可能破坏原计数器的可靠性。新标记一旦遇到交错即永久关闭新增证书，跨 bar/discard/clone 不重新信任；既有 is_constant_ready 与旧恢复语义保留。此处只避免新增捷径误用旧元数据，未重定义手写 HIR 共用 key 的旧更新合同。

新增 15 项测试：8 项纯算术边界/有界生成输入，以及 7 项独立冻结旧算法对照。VecDeque oracle 复制基线的数值与状态机，逐位核对旧 sum、variance、WMA、full/tail、evicted 和撤销前聚合；覆盖阈值邻值、partial warmup、次正规数、十进制残差、NA、signed zero、mixed scale、缩窗、同 bar 替换、discard、clone 和交错反例。

## 实际验证与保留的失败

Windows/Linux canonical verify 均实际退出 0：每个平台 7,423 Rust、794 Python、166 工具测试，406 份 production Rust 文件通过结构检查；Linux 工具测试 skip 1，Windows skip 0。包含 fmt、Clippy、workspace、host parity、真实 Node/Wasm 与全新 wheel/venv。

第一轮两平台的 Rust 测试通过，但窗口文件 845 行违反原有 800 行上限。将原内联 tests 原样移到独立文件，保留模块名后降为 596 行；再次冻结并跑两套完整门禁。失败日志、6,344 文件快照和真实退出码保留；结构阈值及生产算法没有为该修复改变。

独立审计曾错误要求公共 JSON 中每个 Float 必须被 Python 解析为 float，拒绝了既有 Display 编码的整数形式 Float（方差 0）；另错误要求 Windows Built wheel 日志的换行路径位于单行。修正审计器按 JSON 数值投影核对 sidecar 位模式，并保留 -0 的词法符号；在下一 canonical 命令前的有界日志块核对构建路径。Int/NA 类型、浮点位模式及原始 JSON 字节差分继续严格核对。保留两次失败审计脚本和具体样本，运行时、probe、采样与门禁日志未改变。修正后的独立审计验证了这些格式边界及两平台的真实门禁收据。

## 公开 API 计时

8 cases × 2 版本 × 3 fresh 进程，共 48 个进程，全部实际退出 0；before/after 顺序交替。每进程 6,000 bars、L=3,000，动态 case 在 3,000/1,536 间切换。保存 1,584 份完整公共 JSON 和 1,056 份 typed sidecar，1,056 个测量窗口；独立重读 bytes/SHA、IEEE bits、类型/FNV、profile/revision，并完成 528 次公共 delta replay。实际 Rust RuntimeReplica 另在每次 borrowed delta/confirm/next 与生产者严格相等。

历史阶段为 append_bars + owned result()；seed 和 forming 分别调用无输出及 borrowed delta API。构造、caller drop、oracle、profile、完整编码、Replica 与收据 I/O 均在计时窗口之外。每个 forming 路径取每进程 9 次中位数，再取 3 个 fresh 进程中位数。常量 fixture 在计时 forming 保持常量，未计时 confirm/next 改值以校验资格失效和未来状态。

下表单位 ms，正百分比表示候选耗时减少；原始范围见 JSON。三次 fresh 样本不提供置信区间或统计显著性。

| Case | 历史 before→after | 变化 | seed before→after | 变化 |
|---|---:|---:|---:|---:|
| dyadic_wma_hma | 32.301 → 16.227 | +49.76% | 30.369 → 15.197 | +49.96% |
| variance_exact_zero | 43.058 → 25.193 | +41.49% | 41.067 → 23.963 | +41.65% |
| decimal_fallback | 41.769 → 41.466 | +0.73% | 40.528 → 39.388 | +2.81% |
| varying_control | 41.356 → 41.599 | -0.59% | 40.429 → 40.292 | +0.34% |
| cold_sum_warming | 75.308 → 9.661 | +87.17% | 74.779 → 10.292 | +86.24% |
| dynamic_na | 41.235 → 39.745 | +3.61% | 36.481 → 37.571 | -2.99% |
| same_callsite_loop | 31.173 → 30.783 | +1.25% | 30.804 → 30.775 | +0.09% |
| zero_control | 37.017 → 36.931 | +0.23% | 35.281 → 35.690 | -1.16% |

| Case | forming 无输出 before→after | 变化 | borrowed delta before→after | 变化 |
|---|---:|---:|---:|---:|
| dyadic_wma_hma | 0.0945 → 0.0726 | +23.17% | 0.0976 → 0.0904 | +7.38% |
| variance_exact_zero | 0.1152 → 0.0809 | +29.77% | 0.1325 → 0.1018 | +23.17% |
| decimal_fallback | 0.1050 → 0.1013 | +3.52% | 0.1284 → 0.1234 | +3.89% |
| varying_control | 0.1363 → 0.1368 | -0.37% | 0.1480 → 0.1394 | +5.81% |
| cold_sum_warming | 0.0695 → 0.0502 | +27.77% | 0.0844 → 0.0715 | +15.28% |
| dynamic_na | 0.1319 → 0.1376 | -4.32% | 0.1378 → 0.1412 | -2.47% |
| same_callsite_loop | 0.0796 → 0.0821 | -3.14% | 0.1008 → 0.0921 | +8.63% |
| zero_control | 0.1040 → 0.1037 | +0.29% | 0.1167 → 0.1192 | -2.14% |

符合证书的常量 WMA/HMA 历史/seed 约减少 50% 耗时，方差约 41%，极端正数 sum 的 warmup/ready 约 86–87%。这些结果限定于上述 fixture。控制组保留较慢观察：dynamic/NA 的 seed +2.99%、无输出 forming +4.32%、borrowed +2.47%；same-callsite 无输出 +3.14%，zero seed +1.16% 与 borrowed +2.14%，varying 历史 +0.59%。没有据此宣称通用解释器提速，也没有把所有较慢值视为噪声。

## 分配与剩余工作

分配次数中位数全部一致。新增可靠性标记带来有限请求字节开销：历史/seed 累计请求增加 32–96 bytes，额外 peak 和阶段退出 live 增加 32–64 bytes；forming 累计请求/peak 增加 32–64 bytes，退出 live delta 不变。所有原始指标保留。这些是 System allocator 成功 Layout 请求计数，不包含分配器开销、realloc 短暂双缓冲或进程 RSS，不构成内存峰值产品资格。

- 非恒定及未资格常量 WMA/HMA、variance/stdev/deviation 仍扫描窗口。已有 2 ULP WMA 和 variance 反例约束通用递推改造，不能以本轮常量证书覆盖。
- 实时页目录 COW 仍有约 O(L/128) 的引用复制；同调用点同 bar undo 恢复物理上相同的 front sample 时可能仍复制端页。该局部候选只做源码审计，尚未修复或计时；普通 forming 从 confirmed clone 出发，不等同于同 bar undo。
- 手写 HIR 复用窗口 key 且混合更新方式时，既有 aggregate counters 可能失真。本轮永久关闭新资格；旧 key 所有权及更新方式隔离还需独立语义修复。
- Map/Matrix 二级字符串分派、递归 HIR 执行仍存在；此次未测它们的热度。完整资源矩阵、host/strategy/request 吞吐和 TradingView parity 未在本轮资格范围内。

原始工具日志、失败快照、冻结库、完整 payload 与审计脚本保存在 ignored `.local/window-certificates-20261005/`。本地提交，不 push。
