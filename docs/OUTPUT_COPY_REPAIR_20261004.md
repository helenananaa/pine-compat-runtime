# 滚动窗口复制与极值 JSON 输出修复 — 2026-10-04

本轮降低大滚动窗口 checkpoint、forming 更新与 strategy checkpoint 的 payload 复制成本，并将极值和极小浮点数改为紧凑且可往返的 JSON token。Pine 语义仍由独立 runtime 核心提供，未引入具体宿主依赖。

默认短队列保留 flat VecDeque，长度超过 128 后切换到共享 page directory 与 128 值 page；分页队列缩短后仍保持分页，直到清空。克隆共享已存在 payload，后续端点修改复制目录引用和受影响的有界 page。最终分页遍历按连续切片 fold，保留原来的浮点计算顺序。普通 float 和正负零保留原 Display 字节；非零有限绝对值处于 [1e-32, 1e32) 之外时使用紧凑 stack formatter，非有限值仍输出 null。

最终验证冻结 6,316 个文件，相对基线改变 12 个文件；新增 4 个源码/测试文件。本报告及 JSON 在门禁后生成，额外 2 个 docs 文件没有进入对应构建/测试快照。

## 最终验证

| 平台 | Rust 通过 | Python 通过 | 工具通过 / 执行 / 跳过 |
|---|---:|---:|---:|
| windows | 7,344 | 785 | 166 / 166 / 0 |
| linux | 7,344 | 785 | 165 / 166 / 1 |

两平台完成 canonical fmt、clippy、workspace tests、结构和 host parity、真实 WASM Node smoke，以及隔离 wheel 构建、安装、Python tests。6 个独立 phase receipts 在执行前后验证同一冻结 manifest/tar，并绑定实际 command、exit、log、link source 和 library SHA。

最终 native probe 共 628 个 fresh 进程：412 ordinary、66 numeric compatibility、42 serialization、108 focused benchmark。另比较 478 份已保存旧 raw（412 + 66），没有把这些比较重复计算成 fresh before 执行。

412 ordinary 的 13,596 个 snapshot 与 13,184 个 changes 保持字节一致。66 numeric compatibility 共比较 2,178 个 snapshot 与 2,112 个 changes，保留原来 62 个合格与 4 个不合格诊断案例；21 个案例仍逐字一致，45 个只改变极值数值 token 的词法，其中 1,485 个 snapshot 存在词法变化。全部重新比较 IEEE f64 位型和 JSON 结构，未提升 4 个诊断案例的数学资格。

真实 WASM 分页验证另外调用 7 次 fresh compiled batch，核对 7 个完整 snapshot、6 次 replica delta 与 6 次重复传输，以及 4 次 forming 时 confirmed 字节不变。SMA257/SMA5000 以整数 quarter 累加参考独立核验 39,576 个有限位型、42,040 个 warmup null；WMA5000/HMA512 仅通过完整 batch/realtime/replica 字节差分，不作为独立数值 oracle。Wasm 输入、转换后的 background Wasm、generator、JS、Node 和 raw 都有独立 SHA。

## 大窗口复制与普通运行成本

18 个 focused 案例全部列出。每版本三次独立 fresh 采样，before/after 次序交替；clone/apply/update 时间先取每进程内 31 次的中位数，再取三个进程的中位数，seed/result 每进程测量一次。retained 连续替换同一 forming bar，使用相同 incoming 输入，尚未验证变化输入的延迟或吞吐。时间是本机描述性中位数，heap 为申请字节，不代表 RSS、allocator 开销或完整 resource matrix。historical 的完整 runtime clone 与 result 分开；retained 公共输出保留 64 根 confirmed bar。

| 案例 | Bars | Seed ms | Apply ms | Update ms | Clone ms | Clone heap bytes |
|---|---:|---:|---:|---:|---:|---:|
| control_sma14_historical | 100,000 | 166.434 → 169.422 (1.01795×) | — | — | 0.0009 → 0.0009 (1×) | 2,180 → 2,276 (1.04404×) |
| control_rsi14_historical | 100,000 | 170.547 → 178.161 (1.04465×) | — | — | 0.001 → 0.001 (1×) | 2,600 → 2,696 (1.03692×) |
| control_stdev512_historical | 20,000 | 37.4386 → 38.9653 (1.04078×) | — | — | 0.0011 → 0.0008 (0.727273×) | 10,132 → 2,036 (0.200947×) |
| clone_sma5000_historical | 6,000 | 8.9395 → 9.5958 (1.07342×) | — | — | 0.0251 → 0.0009 (0.0358566×) | 81,956 → 2,052 (0.0250378×) |
| clone_sma5000_retained | 6,000 | — | 0.0317 → 0.0067 (0.211356×) | 0.0348 → 0.0071 (0.204023×) | — | — |
| clone_sma100000_historical | 101,000 | 165.241 → 173.245 (1.04844×) | — | — | 0.3909 → 0.0009 (0.00230238×) | 1,601,956 → 2,052 (0.00128093×) |
| clone_sma100000_retained | 101,000 | — | 0.4496 → 0.013 (0.0289146×) | 0.4463 → 0.0134 (0.0300246×) | — | — |
| checkpoint_sma5000_historical | 6,000 | 89.2522 → 19.2904 (0.216134×) | — | — | 0.0022 → 0.0008 (0.363636×) | 81,956 → 2,052 (0.0250378×) |
| ordinary_wma512_historical | 6,000 | 11.419 → 12.2942 (1.07664×) | — | — | 0.0013 → 0.0008 (0.615385×) | 10,132 → 2,036 (0.200947×) |
| ordinary_wma512_retained | 6,000 | — | 0.0069 → 0.007 (1.01449×) | 0.0072 → 0.0072 (1×) | — | — |
| ordinary_wma5000_historical | 6,000 | 12.3411 → 12.826 (1.03929×) | — | — | 0.0236 → 0.0008 (0.0338983×) | 81,940 → 2,036 (0.0248474×) |
| ordinary_wma5000_retained | 6,000 | — | 0.0434 → 0.0104 (0.239631×) | 0.0411 → 0.0108 (0.262774×) | — | — |
| ordinary_hma512_historical | 6,000 | 12.1596 → 12.8667 (1.05815×) | — | — | 0.0011 → 0.0009 (0.818182×) | 10,500 → 2,404 (0.228952×) |
| ordinary_hma512_retained | 6,000 | — | 0.0077 → 0.0074 (0.961039×) | 0.0083 → 0.0079 (0.951807×) | — | — |
| ordinary_hma5000_historical | 6,000 | 14.065 → 14.3228 (1.01833×) | — | — | 0.0255 → 0.0009 (0.0352941×) | 83,076 → 3,172 (0.0381819×) |
| ordinary_hma5000_retained | 6,000 | — | 0.0403 → 0.0128 (0.317618×) | 0.0476 → 0.0131 (0.27521×) | — | — |
| ordinary_corr5000_historical | 6,000 | 15.3196 → 16.3915 (1.06997×) | — | — | 0.084 → 0.0009 (0.0107143×) | 241,940 → 2,036 (0.00841531×) |
| ordinary_corr5000_retained | 6,000 | — | 0.0773 → 0.0133 (0.172057×) | 0.1053 → 0.0135 (0.128205×) | — | — |

全部 seed allocation、allocation count、retained heap、rolling capacity、result materialization 和所有三次样本保留在 JSON。滚动 payload 的改善不能代表其他 runtime 状态或公共历史已完全免复制。

## 序列化与解析阶段归因

7 个案例、42 个 fresh 进程把 parse/analyze、seed、owned result、borrowed view construction、Sink/Vec/String、consume、delta、serde JSON parse 和 public parse 分开测量，共 20 阶段 × 5 指标。每个进程独立保留 full/delta raw，落盘不进入计时。Sink 使用 black-box counter，没有 result 大小的输出 buffer；Vec/String 和 parse 的 heap 单独计账。

下表突出 encoded payload、Sink 和公共 parser。其余全部 20 阶段及 requested bytes、allocations、peak additional live、live delta 的三次样本和中位数保留在 JSON。synthetic delta 是 append-one-point 的 encoder fixture，仅用于编码资格；真实 update 成本由上表单独测量。

| 案例 | Full JSON bytes | Owned Sink ms | View Sink ms | Owned String ms | JSON parse ms | Public parse ms | Public parse allocated bytes |
|---|---:|---:|---:|---:|---:|---:|---:|
| ordinary100k | 794,428 → 794,428 (1×) | 6.2216 → 6.3117 (1.01448×) | 6.3194 → 6.4829 (1.02587×) | 12.8992 → 12.2577 (0.950268×) | 3.8516 → 3.7767 (0.980554×) | 9.363 → 9.6235 (1.02782×) | 17,994,872 → 17,994,872 (1×) |
| mixed20k | 1,528,310 → 350,310 (0.229214×) | 1.0096 → 0.7496 (0.742472×) | 1.014 → 0.7642 (0.753649×) | 2.4534 → 1.556 (0.634222×) | 2.1935 → 1.1139 (0.507819×) | 3.3575 → 2.2383 (0.666657×) | 4,024,036 → 4,023,476 (0.999861×) |
| extreme20k | 6,200,310 → 140,310 (0.0226295×) | 0.8622 → 0.3476 (0.403155×) | 0.8823 → 0.3969 (0.449847×) | 3.8287 → 0.7022 (0.183404×) | 8.7125 → 0.8205 (0.094175×) | 10.811 → 1.8377 (0.169984×) | 4,024,036 → 4,023,416 (0.999846×) |
| subnormal20k | 6,260,310 → 140,310 (0.0224126×) | 0.9299 → 0.3479 (0.374126×) | 0.9329 → 0.3872 (0.41505×) | 4.02 → 0.7617 (0.189478×) | 6.3256 → 1.1126 (0.175888×) | 8.1972 → 2.6857 (0.327636×) | 4,023,416 → 4,023,416 (1×) |
| negative_zero20k | 60,310 → 60,310 (1×) | 0.3111 → 0.2871 (0.922854×) | 0.2715 → 0.2804 (1.03278×) | 0.7084 → 0.7028 (0.992095×) | 0.7649 → 0.6025 (0.787685×) | 2.1932 → 1.7743 (0.809001×) | 4,023,416 → 4,023,416 (1×) |
| nonfinite20k | 88,310 → 88,310 (1×) | 0.2584 → 0.2899 (1.1219×) | 0.2193 → 0.2777 (1.2663×) | 0.4979 → 0.5993 (1.20366×) | 0.6644 → 0.6548 (0.985551×) | 1.4905 → 1.8152 (1.21785×) | 4,023,416 → 4,023,416 (1×) |
| records4003 | 13,142,488 → 2,796,169 (0.212758×) | 3.0246 → 2.3459 (0.775607×) | 3.757 → 2.2708 (0.604418×) | 11.4088 → 5.778 (0.506451×) | 32.0937 → 22.7144 (0.707753×) | 70.6941 → 62.6359 (0.886013×) | 42,311,021 → 42,310,425 (0.999986×) |

IEEE 位型、非有限值 null 和有符号零均从直接 fixture 输入重新核验；record/drawing/plot 包含 17 个数值通道。极值旧 JSON 展开大量零造成的输出和 parser 分配没有被归到解释器窗口算法 heap。public parser 仍构造 serde Value 并克隆 object；紧凑 token 缩减了输入表示，不等于移除解析结构复制。

## 普通关键时间增加超过 5% 的全部项目

以下覆盖全部 focused 案例的 seed/apply/update/clone/result，以及 ordinary/负零/非有限 serialization 的全部阶段。列出三次样本、范围和中位数比例；小绝对时间或范围重叠不会自动消除已观察到的成本。此阈值只用于展示，不代表统计显著性。

| 类别 / 案例 | 指标 | Before 样本 ms | After 样本 ms | Before 范围 | After 范围 | 比例 |
|---|---|---|---|---|---|---:|
| runtime / control_rsi14_historical | resultMs | 2.3262, 2.7299, 2.3155 | 2.6852, 2.3701, 2.9956 | 2.3155–2.7299 | 2.3701–2.9956 | 1.15433× |
| runtime / clone_sma5000_historical | seedMs | 8.9395, 8.9326, 10.001 | 9.6986, 9.4639, 9.5958 | 8.9326–10.001 | 9.4639–9.6986 | 1.07342× |
| runtime / ordinary_wma512_historical | seedMs | 11.9151, 11.0769, 11.419 | 11.6859, 12.3458, 12.2942 | 11.0769–11.9151 | 11.6859–12.3458 | 1.07664× |
| runtime / ordinary_hma512_historical | resultMs | 0.1434, 0.1518, 0.1448 | 0.1568, 0.1671, 0.1598 | 0.1434–0.1518 | 0.1568–0.1671 | 1.10359× |
| runtime / ordinary_hma512_historical | seedMs | 11.9405, 12.1596, 12.4473 | 12.7335, 12.8667, 13.0266 | 11.9405–12.4473 | 12.7335–13.0266 | 1.05815× |
| runtime / ordinary_corr5000_historical | seedMs | 15.3196, 14.9346, 15.6316 | 16.5068, 16.3915, 16.152 | 14.9346–15.6316 | 16.152–16.5068 | 1.06997× |
| serialization / ordinary100k | consumedSourceClone | 2.188, 2.6072, 2.3212 | 2.2371, 2.6562, 2.4619 | 2.188–2.6072 | 2.2371–2.6562 | 1.06062× |
| serialization / ordinary100k | deltaConstruction | 0.0132, 0.0095, 0.0053 | 0.0102, 0.0056, 0.012 | 0.0053–0.0132 | 0.0056–0.012 | 1.07368× |
| serialization / ordinary100k | viewConstruction | 0.016, 0.0112, 0.0072 | 0.0182, 0.0108, 0.0127 | 0.0072–0.016 | 0.0108–0.0182 | 1.13393× |
| serialization / nonfinite20k | consumedSourceClone | 0.4463, 0.3842, 0.4967 | 0.4976, 0.5651, 0.4402 | 0.3842–0.4967 | 0.4402–0.5651 | 1.11495× |
| serialization / nonfinite20k | consumedString | 0.7122, 0.6432, 0.6648 | 0.7517, 1.3841, 0.6776 | 0.6432–0.7122 | 0.6776–1.3841 | 1.13072× |
| serialization / nonfinite20k | deltaConstruction | 0.0025, 0.0037, 0.0029 | 0.0035, 0.0037, 0.0033 | 0.0025–0.0037 | 0.0033–0.0037 | 1.2069× |
| serialization / nonfinite20k | deltaJsonParse | 0.0034, 0.0033, 0.0032 | 0.0041, 0.0038, 0.0063 | 0.0032–0.0034 | 0.0038–0.0063 | 1.24242× |
| serialization / nonfinite20k | deltaVec | 0.0043, 0.0035, 0.0041 | 0.006, 0.015, 0.0054 | 0.0035–0.0043 | 0.0054–0.015 | 1.46341× |
| serialization / nonfinite20k | ownedSink | 0.2584, 0.2612, 0.2422 | 0.2899, 0.757, 0.2699 | 0.2422–0.2612 | 0.2699–0.757 | 1.1219× |
| serialization / nonfinite20k | ownedString | 0.5173, 0.4979, 0.4798 | 0.5993, 0.6589, 0.4976 | 0.4798–0.5173 | 0.4976–0.6589 | 1.20366× |
| serialization / nonfinite20k | ownedVec | 0.3089, 0.317, 0.2974 | 0.3493, 0.5781, 0.2873 | 0.2974–0.317 | 0.2873–0.5781 | 1.13079× |
| serialization / nonfinite20k | publicParse | 1.4905, 1.4174, 1.5195 | 1.5225, 1.8152, 1.9424 | 1.4174–1.5195 | 1.5225–1.9424 | 1.21785× |
| serialization / nonfinite20k | viewSink | 0.2112, 0.2276, 0.2193 | 0.2777, 0.7445, 0.2175 | 0.2112–0.2276 | 0.2175–0.7445 | 1.2663× |
| serialization / nonfinite20k | viewString | 0.5251, 0.4665, 0.5326 | 0.5578, 0.7505, 0.5146 | 0.4665–0.5326 | 0.5146–0.7505 | 1.06227× |
| serialization / nonfinite20k | viewVec | 0.2176, 0.2715, 0.2207 | 0.296, 0.2835, 0.2238 | 0.2176–0.2715 | 0.2238–0.296 | 1.28455× |

JSON 同时保留未超过阈值的所有指标与样本，不将普通路径概括为无成本或全局更快。

## 已替代首轮与证据边界

首轮完成相同的双平台门禁、628 fresh native probes 和独立 raw 审计，随后被封存。首轮普通 float 分支与分页迭代出现成本，因此最终方案拆出 float 冷分支，并对分页连续 slice 使用原序 fold；最终重新跑完整门禁和全部 628 fresh probes。首轮 628 与 3 次 baseline tooling calibration 均不计入最终分母。

首轮 archive identity：E:\projects\pine-interpreter\.local\output-copy-20261004\superseded-first-cycle\archive-identity.json；SHA-256：f83a2218b59848c295a0e820d6fbc4a463c8823ccc8b79f64a51ad54bf0304cf。首轮 audit SHA-256：45081839d62c1253943738185b82fc730511c841466de9b3c6143c990df9420e。它的 artifacts、4 libs、raw locators 和首次回归指标仍独立保留，未把首轮性能数据重新标成最终结果，也未再次运行首轮 audit。

另有 3 个过早启动的 gate/release wrapper，在 source preflight 发现旧 manifest 对应 4 个已变文件时退出；build/test command 执行为 0。该失败 receipt 单独封存，在最终 snapshot 完成后重新启动，不计入 628 native probe 或门禁通过数量。

## 剩余技术债

- 一般 WMA/HMA、方差和数值恢复仍扫描 O(L) 活跃窗口；分页 fold 只降低遍历开销并保留旧舍入顺序。
- 大 checkpoint 的首次修改仍复制 O(ceil(L/128)) 个目录引用，受影响 page 最多复制 128 个 payload 值；短队列 flat clone 至多 128 值。没有声称 O(1) 的完整 update。
- 完整 result/update 与 JSON/public parser 仍按可见输出和历史增长；serde object clone 和其他状态 checkpoint 复制仍待处理。
- 滚动容量 profile 观察仍遍历 O(ceil(L/128)) 个 page；该源码可确认的观察成本位于本轮重点计时阶段之外。
- 数值资格仍是已有 62 合格、4 不合格诊断的显式范围，没有完整浮点误差上界；混合极值尺度或需要第三展开分量的情况未获得新资格。
- Math/Array dispatch、其他 checkpoint 数据结构和更广 QR/条件数覆盖仍是独立工作；本轮未执行 full resource matrix。

所有 stage bindings、final native/WASM receipts、每个 sample、SHA 与 helper/probe 源码在配套 JSON 中。ignored heap_runner 未使用，未放入复现 helper 集合。报告生成器启动 build、probe、audit 进程均为零。

配套数据：[OUTPUT_COPY_REPAIR_RESULTS_20261004.json](E:/projects/pine-interpreter/docs/OUTPUT_COPY_REPAIR_RESULTS_20261004.json)
