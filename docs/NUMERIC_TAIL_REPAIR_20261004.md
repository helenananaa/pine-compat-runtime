# 滚动统计与加权数值恢复 — 2026-10-04

本轮修复大偏移成对统计中的部分抵消，以及加权扫描中可恢复的中间溢出。实现属于独立 Pine runtime，未引入宿主应用依赖或新的持久滚动状态。

经验证的差异包含 8 个 Rust 文件和 12 个新增测试。最终冻结源码包含 6,310 个文件；本报告及 JSON 在验证后生成，不在该构建、测试快照中。

## 数值合同与边界

成对统计保留原来的完全损失筛选，另加部分精度筛选：原始矩操作数的一个 epsilon 至少达到中心化标准差乘积的 2^-20。原始协方差超过候选预算时保留原路径。抽样范围下界可用 O(1) 排除普通候选，进入恢复前仍须以完整中心化标准差乘积确认。该条件是有界启发式，并非完整的滚动累加误差上界。稀疏异常可漏过抽样，随后完整扫描仍会拒绝恢复并保留旧结果。

加权恢复只在旧加权扫描结果非有限时进入。完整有限常量窗口可由现有元数据直接证明，其冷路径资格检查和返回为 O(1)；原 WMA 加权扫描保持 O(L)。非恒定窗口须逐项通过可逆二次幂归一化，并以 FMA 捕获乘积残差、以精确两分量展开保留加法残差；需要第三分量或归一化不可逆时拒绝恢复。有限可逆分子先恢复尺度再除法，以保留抵消后的小尾部；另一除法关联受输入范围约束。HMA 只在旧差值非有限且 full/half 均有限时改变关联，真正越界的阶段继续产生 NA。

旧有限运算、常量协方差残差和有符号零保留原运算次序与位型。RollingWindowState、RollingWindowKey 声明未变；恢复函数新增持久状态为零。资格仅覆盖下列显式输入与检查。

## 验证结果

| 门禁 | Rust 通过 | Python 通过 / 收集 | 工具通过 / 执行 / 跳过 |
|---|---:|---:|---:|
| windows | 7,327 | 785 / 785 | 166 / 166 / 0 |
| linux | 7,327 | 785 / 785 | 165 / 166 / 1 |

最终完整门禁中的 runtime 单元测试为 Windows 2,151、WSL/Linux 2,151 个通过；第二周期单独 targeted 复跑通过 2,151 个。targeted raw SHA 与封存的首次日志不同，报告生成时重新确认全部 6,310 个冻结文件的当前 SHA。表中计数均解析自最终实际日志；两平台完成 fmt、clippy、workspace 测试、结构与宿主输出检查、真实 WASM Node smoke，以及隔离 wheel 构建、安装与测试。

普通兼容性执行 412 个 fresh after 进程，对比 412 份经过 SHA 核实的旧 raw：13,596 个 snapshot 和 13,184 个 changes 字节一致。这是新 after 执行与旧基线记录比较，不是新执行 824 个 before/after 进程。

独立数学检查执行 132 个进程，每版本 62 个合格案例。before 通过 20，after 通过 62。每版本检查 1,267,824 个合格聚合值；before 有 128,861 个不匹配，after 为 0。

另有每版本 4 个诊断进程，检查 5,028 个诊断聚合值，before/after 分别保留 3,361/3,361 个诊断不匹配；合并诊断后的总检查量为每版本 1,272,852 个值。诊断案例不计入数学资格。参考以 Fraction.from_float 读取实际 f64 输入，再用精确中心矩、整数权重和规定的有限 HMA 舍入阶段计算；一般容差为 2e-12 相对误差或 8 ULP。只有标记 constantExact 的 ±1e308/±MAX 常量案例要求精确源位型，1e200 和 subnormal 常量使用普通容差。

额外真实 WASM 数值检查通过 3 个命名案例。wasm-bindgen 输入和转换后输出各有独立 SHA，转换链记录在 JSON 中，不要求两者字节相同。最终只读审计通过，审计器启动构建与 probe 进程均为零；原始记录的定位和 SHA-256 保留在 JSON。

基线封存 4 个已链接库，继承 4 个已审计精确 probe；本轮重新链接 2 个 after probe。归档仅覆盖明确的链接依赖与 probe，没有声称封存全部 release 产物。

最终周期新执行量为 724 个 native probe 进程：ordinary 412、math 132、focused benchmark 180。旧 ordinary raw 比较不作为 fresh 进程重复计数。

## 首次恢复方案（已封存并被替代）

首次方案已闭合全部门禁、数学、字节比较、计时与只读审计，并封存在 E:\projects\pine-interpreter\.local\numeric-tail-20261004\superseded-first-recovery-20261004T111658。该周期有 724 个 fresh native probe 进程及 412 份旧 raw 比较。其源码、after probe、产物和汇总均独立封存；raw 目录保持原位。它不并入最终周期的 fresh 数量或最终源码资格。

首次计时显示极值常量恢复重复冷扫描的成本，因此最终方案增加已有元数据证明的常量冷路径。下表完整保留首次四个 corrected weighted 案例中位数；所有原始样本仍在其归档汇总中。这里是修正输出的成本比较，旧 NA 到新有限结果不能描述成等价输出的性能回退或加速。

| 首次案例 | Seed ms before → first | Apply ms before → first | Update ms before → first |
|---|---:|---:|---:|
| corrected_weighted512_historical | 48.6962 → 109.794 (2.25467×) | — | — |
| corrected_weighted512_retained | — | 0.0146 → 0.0252 (1.72603×) | 0.0157 → 0.0259 (1.64968×) |
| corrected_weighted5000_historical | 190.706 → 296.427 (1.55437×) | — | — |
| corrected_weighted5000_retained | — | 0.1242 → 0.2357 (1.89775×) | 0.1308 → 0.2327 (1.77905×) |

归档身份文件：.local/numeric-tail-20261004/superseded-first-recovery.json；SHA-256：96ebe8332dfbb72b92053b9c7ac8518deb1ff81d31c1008772c9af99d21d14fd。首次 ordinary、math、benchmark 和 audit 汇总各自的归档 locator 与 SHA 保留在 JSON，未把旧周期重新标成最终结果。

两轮既有 math raw 逐字比较全部一致：132 对，其中 before 66、after 66。比较新启动进程为零、新增数学资格分母为零，支持常量优化保留已有修正结果。比较记录 .local/numeric-tail-20261004/cycle-output-comparison.json 和独立复核 .local/numeric-tail-20261004/cycle-output-audit.json 的 SHA、原始路径及 helper 源码均保留在 JSON。首次与最终 after 成本仅作本机跨轮描述，不能当作同一时间区块内严格配对的因果性能测量。

## 最终 focused 性能

下表保留全部 30 个案例、180 个 focused 进程。每版本三次 fresh 采样，before/after 次序交替。JSON 保留每个样本、指标和中位数；本轮未执行全 resource matrix。corrected 路径的变化表示修正后输出的成本，NA 到有限值的变化不能称为等价输出加速。

historical 模式物化公共历史；retained 模式保留 64 根已确认公共 bar。heap 记录申请字节数，未含 allocator 额外开销。三次样本中位数和 ≥10% 阈值只作描述，不代表统计显著性。

| 案例 | Bars | 模式 | Seed ms before → after | Apply ms before → after | Update ms before → after | 保留 heap bytes before → after |
|---|---:|---|---:|---:|---:|---:|
| control_sma14_historical | 100,000 | historical | 179.473 → 180.647 (1.00654×) | — | — | 3.38614e+06 → 3.38614e+06 (1×) |
| control_rsi14_historical | 100,000 | historical | 190.914 → 182.423 (0.955529×) | — | — | 3.38706e+06 → 3.38706e+06 (1×) |
| control_stdev512_historical | 20,000 | historical | 40.4397 → 40.8094 (1.00914×) | — | — | 683,091 → 683,091 (1×) |
| ordinary_wma512_historical | 6,000 | historical | 12.708 → 12.1487 (0.955988×) | — | — | 210,737 → 210,737 (1×) |
| ordinary_wma512_retained | 6,000 | retained | — | 0.0075 → 0.0076 (1.01333×) | 0.0081 → 0.0081 (1×) | 308,961 → 308,961 (1×) |
| ordinary_wma5000_historical | 6,000 | historical | 14.4605 → 13.9474 (0.964517×) | — | — | 185,201 → 185,201 (1×) |
| ordinary_wma5000_retained | 6,000 | retained | — | 0.0436 → 0.0386 (0.885321×) | 0.0423 → 0.045 (1.06383×) | 431,841 → 431,841 (1×) |
| ordinary_hma512_historical | 6,000 | historical | 13.7275 → 12.6259 (0.919752×) | — | — | 204,401 → 204,401 (1×) |
| ordinary_hma512_retained | 6,000 | retained | — | 0.0073 → 0.0081 (1.10959×) | 0.0079 → 0.0085 (1.07595×) | 309,473 → 309,473 (1×) |
| ordinary_hma5000_historical | 6,000 | historical | 15.3164 → 14.6116 (0.953984×) | — | — | 185,009 → 185,009 (1×) |
| ordinary_hma5000_retained | 6,000 | retained | — | 0.0452 → 0.0414 (0.915929×) | 0.0499 → 0.0493 (0.987976×) | 433,889 → 433,889 (1×) |
| ordinary_corr512_historical | 20,000 | historical | 47.487 → 48.4946 (1.02122×) | — | — | 75,865 → 75,865 (1×) |
| ordinary_corr512_retained | 20,000 | retained | — | 0.0081 → 0.0078 (0.962963×) | 0.0085 → 0.0082 (0.964706×) | 993,929 → 993,929 (1×) |
| ordinary_corr5000_historical | 6,000 | historical | 15.6836 → 15.6643 (0.998769×) | — | — | 447,353 → 447,353 (1×) |
| ordinary_corr5000_retained | 6,000 | retained | — | 0.0944 → 0.0951 (1.00742×) | 0.1213 → 0.1107 (0.912613×) | 693,993 → 693,993 (1×) |
| orthogonal_cov_guard8192_historical | 9,192 | historical | 21.913 → 22.2011 (1.01315×) | — | — | 425,354 → 425,354 (1×) |
| orthogonal_cov_guard8192_retained | 9,192 | retained | — | 0.1617 → 0.1649 (1.01979×) | 0.1631 → 0.1757 (1.07725×) | 843,226 → 843,226 (1×) |
| orthogonal_cov_guard100000_historical | 101,000 | historical | 260.375 → 254.399 (0.977048×) | — | — | 6.48446e+06 → 6.48446e+06 (1×) |
| orthogonal_cov_guard100000_retained | 101,000 | retained | — | 1.457 → 1.5903 (1.09149×) | 1.4824 → 1.5705 (1.05943×) | 1.11485e+07 → 1.11485e+07 (1×) |
| corrected_paired37_historical | 20,000 | historical | 81.3271 → 79.5581 (0.978248×) | — | — | 1.43849e+06 → 1.43849e+06 (1×) |
| corrected_paired37_retained | 20,000 | retained | — | 0.0117 → 0.0119 (1.01709×) | 0.0124 → 0.0127 (1.02419×) | 988,412 → 988,412 (1×) |
| corrected_paired257_historical | 20,000 | historical | 157.872 → 156.448 (0.990981×) | — | — | 1.52374e+06 → 1.52374e+06 (1×) |
| corrected_paired257_retained | 20,000 | retained | — | 0.0157 → 0.0158 (1.00637×) | 0.0166 → 0.0164 (0.987952×) | 1.03116e+06 → 1.03116e+06 (1×) |
| corrected_paired5000_historical | 6,000 | historical | 103.622 → 104.172 (1.00531×) | — | — | 889,820 → 889,820 (1×) |
| corrected_paired5000_retained | 6,000 | retained | — | 0.2982 → 0.3013 (1.0104×) | 0.2973 → 0.2948 (0.991591×) | 1.09305e+06 → 1.09305e+06 (1×) |
| corrected_weighted512_historical | 6,000 | historical | 50.3742 → 50.2158 (0.996856×) | — | — | 45,960 → 59,464 (1.29382×) |
| corrected_weighted512_retained | 6,000 | retained | — | 0.0149 → 0.0156 (1.04698×) | 0.0162 → 0.0163 (1.00617×) | 314,728 → 314,728 (1×) |
| corrected_weighted5000_historical | 6,000 | historical | 191.479 → 191.518 (1.0002×) | — | — | 293,256 → 305,352 (1.04125×) |
| corrected_weighted5000_retained | 6,000 | retained | — | 0.1408 → 0.1315 (0.933949×) | 0.1383 → 0.1339 (0.968185×) | 562,024 → 562,024 (1×) |
| constant_cov5000_historical | 6,000 | historical | 11.3431 → 11.605 (1.02309×) | — | — | 456,568 → 456,568 (1×) |

本轮保留明确的普通路径成本：ordinary_hma512_retained 的 apply 中位数 0.0073 → 0.0081 ms（+11.0%）；before 样本 [0.0073, 0.0076, 0.0072]，after [0.0084, 0.0078, 0.0081]。绝对时间小不能自动把它归为噪声；其余达到阈值的项目完整列在下表，未将普通路径统称为没有退步。

零新增持久字段不能推出总 heap 不变。corrected weighted historical 的 L512 heap 45,960 → 59,464 bytes（+13,504）；L5000 为 293,256 → 305,352（+12,096）。两者 seed 申请字节分别增加 22,384/21,616，申请次数增加 8/10；两个 retained weighted 案例的 heap 保持相同。output/collect.rs 的 push_compact，以及 append_history.rs:269 insert_compact、:294 expand_repeat，在 warmup NA 转为有限输出时可能改变已有表示；这是源码解释候选，未隔离测量，不能将全部差额直接归因于它。

### 普通路径中位数变化至少 10% 的项目

下表同时列出增减项目的全部三次样本与范围。范围重叠、短阶段和系统波动限制了可作出的判断。

| 案例 | 指标 | Before 样本 (ms) | After 样本 (ms) | Before 范围 | After 范围 | 中位数比例 |
|---|---|---|---|---|---|---:|
| ordinary_wma5000_retained | applyUpdateMedianMs | 0.0352, 0.0436, 0.0577 | 0.0386, 0.0374, 0.042 | 0.0352, 0.0577 | 0.0374, 0.042 | 0.885321× |
| ordinary_hma512_historical | cloneMedianMs | 0.0012, 0.0014, 0.0014 | 0.0019, 0.0012, 0.0012 | 0.0012, 0.0014 | 0.0012, 0.0019 | 0.857143× |
| ordinary_hma512_historical | resultMs | 0.1594, 0.1466, 0.1687 | 0.1886, 0.1394, 0.1952 | 0.1466, 0.1687 | 0.1394, 0.1952 | 1.18319× |
| ordinary_hma512_retained | applyUpdateMedianMs | 0.0073, 0.0076, 0.0072 | 0.0084, 0.0078, 0.0081 | 0.0072, 0.0076 | 0.0078, 0.0084 | 1.10959× |
| ordinary_corr5000_historical | cloneMedianMs | 0.0845, 0.0883, 0.0793 | 0.0815, 0.1042, 0.12 | 0.0793, 0.0883 | 0.0815, 0.12 | 1.23314× |
| ordinary_corr5000_historical | resultMs | 0.2502, 0.2325, 0.1915 | 0.2012, 0.192, 0.2004 | 0.1915, 0.2502 | 0.192, 0.2012 | 0.861935× |
| constant_cov5000_historical | resultMs | 0.1797, 0.2044, 0.1752 | 0.2108, 0.1901, 0.204 | 0.1752, 0.2044 | 0.1901, 0.2108 | 1.13523× |

## 仍待处理的债务

- forming 与 strategy evaluator checkpoint 仍深复制 rolling HashMap 及窗口 VecDeque。源码定位在 runtime/realtime.rs、runtime/historical.rs 和 algorithms/rolling_window.rs；这是结构候选，尚未证明为本轮新测热点。
- 一般 variance/WMA/HMA 仍有 O(L) 扫描；常量恢复捷径只减少非有限旧扫描后的附加冷路径。
- 部分精度筛选仍是启发式，未建立所有 f64 累加、抵消和舍入的完整误差合同。
- Math/Array 字符串 dispatch 的潜在收益尚未在本轮测量。

- output/json/value_writer.rs:8 以 Display 写有限 float，1e308 会形成 309 位整数 token；records_writer.rs:10 属同类输出路径。每轮 132 份 math stdout 已核实共 305,632,802 bytes，可对 JSON 格式化与重复 full snapshot 传输后续分阶段量测。这是源码与输出大小确认的候选，不能把这些 bytes 当 runtime heap 或已测耗时瓶颈。

## 复现材料与资格范围

JSON 保留完整 30-case 性能 records/summary、ordinary 比较、before/after 数学诊断、helper/probe 源文本、生成的数值 Pine 输入、源码和二进制身份，以及原始记录 locator/SHA-256。audit 使用独立 raw 定位与 hash，未重复嵌入完整 artifact/check 表。

冻结材料位于 .local/numeric-tail-20261004，包含精确源码 tar 和已链接基线身份。build_report.py 只有在全部最终门禁、ordinary、数学、focused、只读审计和真实 WASM 数值记录通过，且首次方案已封存时才生成报告；生成器不启动构建、probe、benchmark 或 audit。

上述证据不构成完整滚动浮点误差上界、全部混尺度窗口资格、长会话/全 resource matrix 资格或全面 TradingView 原生一致性证明。实现边界保持在 Pine 语言与确定性 runtime。
