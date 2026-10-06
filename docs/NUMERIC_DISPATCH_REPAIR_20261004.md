# TA 操作码、HMA 扫描与矩阵数值恢复（2026-10-04）

比较基线 `e76cf5f3213639911fac4fa913bd4541831ee62c`：冻结 6299 份源码与 501 份 release 产物；最终 6302 份源码同时用于 Windows/Linux 门禁。before 探针按原字节继承上一轮最终 after 产物，并核对源码、库、可执行文件和来源 receipt SHA。

## 改动

- 64 个 TA builtin 在 PreparedProgram 中选出 typed opcode，执行时使用枚举分派；每个旧 kernel 的调用及取参顺序逐项保留。手写 HIR 的 callee guard、缺失/稀疏/冲突 ID fallback、同名布局 AND 和 unknown 参数不求值保持。命名覆盖仍由原 RuntimeArgs 处理。
- HMA 将全窗和半窗加权均值融合为一次遍历，各自仍按 oldest→newest 乘加；累加初值保持旧 Iterator::sum 的 -0.0。保留窗口及 undo 结构，不采用会改变浮点末位的递推。WMA 未改；HMA 仍 O(length)。
- matrix.mult 的两个输入与 inv/eigenvalues/eigenvectors/pinv 的只读输入改为借用，避免先克隆全部 PineValue 再转 f64。结果仍独立分配，别名输入及后续修改不影响已生成结果。
- rank 仅在旧消元出现非有限值时重试位可逆的行、列二次幂缩放；复用工作数组并只取一次原值备份。两种方向均不可靠时返回 na，避免报告污染后的整数 rank。
- eigen 内部分开数值失败与 complex 失败；数值失败可按逐元素位可逆的二次幂缩放恢复，complex 失败不会被重解释。恢复依据原始精确对称性；精确三角矩阵的失败路径直接取特征多项式的对角根，保留混合尺度的微小对角值。修正 QR norm 溢出和 2×2 NaN 判别式伪造有限双根的反例。

普通成功路径的数值 kernel 次序保持；这不意味着所有旧有限输出不变：旧 NaN.max(0) 可伪造有限特征值，norm=inf 可产生有限全零向量，正是本轮有意修正的结果。未调整既有 golden、结构阈值和主机边界。

## 验证

| 平台 | Rust | Python | 工具测试 / skipped |
|---|---:|---:|---:|
| windows | 7290 | 785 | 166 / 0 |
| linux | 7290 | 785 | 166 / 1 |

两平台 canonical verify 均通过 fmt、严格 Clippy、完整 workspace 测试、结构/工具、host parity、真实 WASM/Node、新 wheel 与新 venv 的 Python 测试；结构检查 399 个生产 Rust 文件。新增 18 项 Rust 测试，覆盖 TA 解码/回退/副作用、HMA 独立旧双扫和三窗参考、QR/特征值解析答案及 Av=λv、可逆缩放和 rank 无法恢复时的公开 na。

ordinary 兼容性：366/366 组，12,078 snapshot 与 11,712 delta 逐字相等；含 unlimited/retained、forming 替换/confirmed 提交，以及 owned/view、confirmed owned/view 和 replica 的内部一致性。具体源、SHA 和 seed 全部写入 JSON；矩阵 256 bars、metadata 64 bars，其余新压力/定向输入 6,000 bars。

数值修正单独核验：13 个公开输入 × 2 retention modes，共 26 cases、52 个新进程，两版本各 858 snapshot、832 delta。before 行为实际运行，after 每个 bar 对照独立解析答案；它们不混入普通兼容性分母。包含上一轮 4 个修复控制与 2 个遗留反例，以及混合 row rank、不可逆 rank、2×2 判别式溢出、tiny nonsymmetric、混合三角对角、不可逆 eigen failure 和 small complex off-diagonal。

| 关键反例 | 基线结果 | 本轮结果 |
|---|---|---|
| 1e154 下三角 Jordan 型矩阵 | [na,H,H] | [H,H,H] |
| 含 1e308 / 1e-308 的 4×4 满秩输入 | rank=3 | rank=4 |
| 行、列缩放均会丢位的 rank 输入 | 污染后 rank=3 | na |
| [[H,H],[H/2,H]], H=1e154 | [H,H] | [(1+√0.5)H,(1−√0.5)H] |
| [[1e308,1e308],[0,1e-308]] | [na,na] | [1e308,1e-308]；vectors 仍 na |

## 本机受控性能

56 cases、316 个新进程；每版本通常 3 次交替，staged 归因每版本 1 次。编译、门禁和差分结束后测量，保留全部样本、慢化项、WMA/stdev 和输出流程控制。TA opcode pressure 每 bar 48 次有效便宜调用，20k bars 共 960k 调用；不使用 length=0 等提前退出制造收益。

确定的分配收益：matrix.mult seed 请求 bytes 1,652,388→603,812（-63.5%），分配次数 4,196→3,684；tall/wide 的 pinv+eigenvectors 请求 bytes 4,779,032→3,197,976（-33.1%），分配次数 7,262→6,748；保留逻辑堆均不变。HMA5000 historical seed 16.1602→14.3992 ms（-10.9%），HMA 空间和分配本轮不变。

TA positional mix 中位数 414.4835→420.0589 ms（+1.3%）、named 411.9248→406.6984 ms（-1.3%），opcode pressure 423.4944→421.9949 ms（-0.4%）；未证明整体 TA 吞吐提升。HMA5000 realtime 完整 owned update 0.1760→0.1883 ms（+7.0%），这是实际慢化项，不能宣称所有实时场景改善。

| case | bars / mode | seed 或 apply/update ms：前 → 后 | 请求分配 bytes：前 → 后 | 保留逻辑堆 bytes：前 → 后 |
|---|---|---:|---:|---:|---:|
| highest_16 | 24000 / historical | 50.106 → 52.369 | 3931188 → 3931188 | 1067941 → 1067941 |
| highest_512 | 24000 / historical | 50.437 → 50.565 | 5937136 → 5937136 | 77505 → 77505 |
| highest_5000 | 24000 / historical | 50.452 → 52.119 | 6269992 → 6269992 | 223881 → 223881 |
| lowest_16 | 24000 / historical | 49.774 → 48.727 | 3934772 → 3934772 | 1069732 → 1069732 |
| lowest_512 | 24000 / historical | 50.644 → 49.261 | 7200096 → 7200096 | 77696 → 77696 |
| lowest_5000 | 24000 / historical | 51.371 → 51.728 | 7532952 → 7532952 | 224072 → 224072 |
| highestbars_16 | 24000 / historical | 51.100 → 48.350 | 3422580 → 3422580 | 818601 → 818601 |
| highestbars_512 | 24000 / historical | 51.017 → 52.163 | 7426976 → 7426976 | 837637 → 837637 |
| highestbars_5000 | 24000 / historical | 54.117 → 50.743 | 8105432 → 8105432 | 984973 → 984973 |
| lowestbars_16 | 24000 / historical | 51.657 → 50.148 | 3422244 → 3422244 | 818344 → 818344 |
| lowestbars_512 | 24000 / historical | 52.596 → 51.403 | 8689600 → 8689600 | 837572 → 837572 |
| lowestbars_5000 | 24000 / historical | 52.257 → 52.716 | 9368056 → 9368056 | 984908 → 984908 |
| rci_512 | 6000 / historical | 38.059 → 37.677 | 781724 → 781724 | 222961 → 222961 |
| rci_5000 | 6000 / historical | 43.228 → 43.699 | 1026780 → 1026780 | 356209 → 356209 |
| mode_512 | 6000 / historical | 29.332 → 28.193 | 769468 → 769468 | 214770 → 214770 |
| mode_5000 | 6000 / historical | 34.086 → 33.355 | 855740 → 855740 | 250674 → 250674 |
| common_plot | 100000 / historical | 158.391 → 160.366 | 12624232 → 12624232 | 3382631 → 3382631 |
| common_sma14 | 100000 / historical | 184.408 → 173.433 | 12630348 → 12630348 | 3386081 → 3386081 |
| common_ema14 | 100000 / historical | 184.372 → 184.535 | 22230464 → 22230464 | 3386293 → 3386293 |
| common_rsi14 | 100000 / historical | 179.801 → 179.486 | 12631936 → 12631936 | 3386997 → 3386997 |
| common_strategy | 100000 / historical | 206.452 → 204.093 | 28611936 → 28611936 | 7495454 → 7495454 |
| highest_descending_5000_historical | 24000 / historical | 56.328 → 56.327 | 8053624 → 8053624 | 912857 → 912857 |
| highest_descending_5000_realtime | 24000 / realtime | 0.013 → 0.014 / 0.508 → 0.522 | 24764 → 24764 / 1556724 → 1556724 | 2064873 → 2064873 |
| highest_descending_5000_retained | 24000 / retained | 0.009 → 0.009 / 0.009 → 0.009 | 24652 → 24652 / 24708 → 24708 | 1414825 → 1414825 |
| lowest_ascending_5000_historical | 24000 / historical | 55.349 → 54.719 | 8053624 → 8053624 | 912856 → 912856 |
| lowest_ascending_5000_realtime | 24000 / realtime | 0.015 → 0.013 / 0.515 → 0.491 | 24764 → 24764 / 1556724 → 1556724 | 2064872 → 2064872 |
| lowest_ascending_5000_retained | 24000 / retained | 0.009 → 0.009 / 0.009 → 0.009 | 24652 → 24652 / 24708 → 24708 | 1414824 → 1414824 |
| unchanged_control_wma_512 | 20000 / historical | 42.026 → 41.524 | 2526700 → 2526700 | 683025 → 683025 |
| unchanged_control_stdev_512 | 20000 / historical | 38.240 → 39.062 | 2529436 → 2529436 | 683027 → 683027 |
| math_dispatch | 20000 / append | 204.922 → 197.211 | probe 未计量 | probe 未计量 |
| ta_mix_positional | 20000 / historical | 414.483 → 420.059 | 19379288 → 19379288 | 718961 → 718961 |
| ta_mix_named | 20000 / historical | 411.925 → 406.698 | 19379288 → 19379288 | 718961 → 718961 |
| ta_opcode_pressure | 20000 / historical | 423.494 → 421.995 | 5115436 → 5115436 | 706463 → 706463 |
| hma_16_historical | 6000 / historical | 9.998 → 10.009 | 764980 → 764980 | 211569 → 211569 |
| hma_16_realtime | 6000 / realtime | 0.011 → 0.010 / 0.138 → 0.148 | 19096 → 19096 / 399056 → 399056 | 499585 → 499585 |
| hma_16_retained | 6000 / retained | 0.006 → 0.006 / 0.007 → 0.007 | 18984 → 18984 / 19040 → 19040 | 301025 → 301025 |
| hma_512_historical | 6000 / historical | 13.210 → 12.790 | 751668 → 751668 | 204337 → 204337 |
| hma_512_realtime | 6000 / realtime | 0.014 → 0.013 / 0.142 → 0.127 | 27336 → 27336 / 407296 → 407296 | 492353 → 492353 |
| hma_512_retained | 6000 / retained | 0.008 → 0.007 / 0.008 → 0.008 | 27224 → 27224 / 27280 → 27280 | 309409 → 309409 |
| hma_5000_historical | 6000 / historical | 16.160 → 14.399 | 722012 → 722012 | 184945 → 184945 |
| hma_5000_realtime | 6000 / realtime | 0.045 → 0.046 / 0.176 → 0.188 | 99912 → 99912 / 479872 → 479872 | 472961 → 472961 |
| hma_5000_retained | 6000 / retained | 0.040 → 0.037 / 0.049 → 0.044 | 99800 → 99800 / 99856 → 99856 | 433825 → 433825 |
| matrix_tall_workspace | 256 / historical | 5.085 → 4.666 | 4779032 → 3197976 | 1637979 → 1637979 |
| matrix_wide_workspace | 256 / historical | 4.911 → 4.749 | 4779032 → 3197976 | 1637979 → 1637979 |
| matrix_rank_guard | 256 / historical | 1.109 → 1.131 | 568044 → 568044 | 21064 → 21064 |
| matrix_mult_inputs | 256 / historical | 1.527 → 1.336 | 1652388 → 603812 | 559388 → 559388 |

### 完整实时返回的补充测量

为核对上述 +7% 慢化，额外运行 5 场景 × 每版本 7 个新进程 = 70 processes，保持同一冻结代码、库与 probe，并保留原 316 样本。TAG 交替、奇数次反转 case 顺序；审计在测量结束后开始。

HMA5000 historical 再测 16.7164→15.6212 ms（-6.6%），确认历史吞吐收益。realtime apply 0.0483→0.0465 ms（-3.7%），但 owned update 0.1939→0.2018 ms（+4.1%）；retained owned update 0.0471→0.0485 ms（+3.0%），与原矩阵的 -10.1% 方向不同。样本范围有重叠；WMA 完整返回对照 -3.9%、plot 对照 +1.6%。保留融合扫描的历史执行收益，同时把完整返回的慢化和实时收益不稳定列为后续归因点；不把重叠解释为没有回归。

| 补测 case | seed ms：前 → 后 | apply ms：前 → 后 | owned update ms：前 → 后 |
|---|---:|---:|---:|
| hma_5000_historical | 16.716 → 15.621 | — | — |
| hma_5000_realtime | — | 0.048 → 0.046 | 0.194 → 0.202 |
| hma_5000_retained | — | 0.042 → 0.043 | 0.047 → 0.049 |
| wma_512_realtime_control | — | 0.013 → 0.013 | 0.157 → 0.151 |
| plot_6000_realtime_control | — | 0.009 → 0.010 | 0.148 → 0.150 |

### 完整输出各阶段

这些未改的输出流程作为控制；staged 持有之前阶段对象，不能当作独立 pipeline 峰值。

| case / stage | ms：前 → 后 | 请求分配 MiB：前 → 后 | 额外逻辑峰值 MiB：前 → 后 | 结束 RSS MiB：前 → 后 | 累计峰值 RSS MiB：前 → 后 |
|---|---:|---:|---:|---:|---:|
| output_heavy_pipeline / dataset | 1.041 → 1.246 | 4.578 → 4.578 | 4.578 → 4.578 | 8.559 → 8.559 | 8.559 → 8.559 |
| output_heavy_pipeline / analyze | 0.607 → 0.624 | 0.071 → 0.071 | 0.025 → 0.025 | 9.742 → 9.754 | 9.742 → 9.754 |
| output_heavy_pipeline / init | 0.036 → 0.047 | 0.005 → 0.005 | 0.004 → 0.004 | 9.812 → 9.836 | 9.812 → 9.836 |
| output_heavy_pipeline / seed | 665.718 → 678.266 | 82.671 → 82.671 | 26.384 → 26.384 | 37.590 → 37.617 | 37.590 → 37.617 |
| output_heavy_pipeline / hostJsonPipeline | 124.701 → 125.487 | 15.682 → 15.682 | 15.653 → 15.653 | 53.387 → 53.410 | 53.387 → 53.410 |
| output_heavy_pipeline / parseJson | 71.518 → 72.881 | 156.582 → 156.582 | 80.585 → 80.585 | 125.980 → 126.000 | 126.922 → 126.941 |
| output_heavy_staged / dataset | 1.204 → 1.291 | 4.578 → 4.578 | 4.578 → 4.578 | 8.555 → 8.562 | 8.555 → 8.562 |
| output_heavy_staged / analyze | 0.521 → 0.564 | 0.071 → 0.071 | 0.025 → 0.025 | 9.734 → 9.750 | 9.734 → 9.750 |
| output_heavy_staged / init | 0.030 → 0.041 | 0.005 → 0.005 | 0.004 → 0.004 | 9.805 → 9.832 | 9.805 → 9.832 |
| output_heavy_staged / seed | 646.769 → 649.175 | 82.671 → 82.671 | 26.384 → 26.384 | 37.605 → 37.617 | 37.605 → 37.617 |
| output_heavy_staged / cowClone | 0.051 → 0.052 | 0.002 → 0.002 | 0.002 → 0.002 | 37.668 → 37.680 | 37.668 → 37.680 |
| output_heavy_staged / ownedResult | 53.713 → 54.893 | 74.771 → 74.771 | 74.771 → 74.771 | 120.281 → 120.293 | 120.281 → 120.293 |
| output_heavy_staged / serializeOwnedResult | 112.791 → 115.342 | 15.653 → 15.653 | 15.652 → 15.652 | 136.027 → 136.043 | 136.027 → 136.043 |
| output_heavy_staged / parseJson | 75.085 → 79.123 | 156.582 → 156.582 | 80.585 → 80.585 | 208.402 → 208.281 | 209.344 → 209.223 |
| strategy_history_pipeline / dataset | 1.252 → 1.229 | 4.578 → 4.578 | 4.578 → 4.578 | 8.559 → 8.551 | 8.559 → 8.551 |
| strategy_history_pipeline / analyze | 0.546 → 0.549 | 0.043 → 0.043 | 0.020 → 0.020 | 9.809 → 9.809 | 9.809 → 9.809 |
| strategy_history_pipeline / init | 0.034 → 0.041 | 0.004 → 0.004 | 0.003 → 0.003 | 9.883 → 9.895 | 9.883 → 9.895 |
| strategy_history_pipeline / seed | 376.435 → 385.888 | 159.294 → 159.294 | 50.989 → 50.989 | 72.020 → 72.012 | 72.020 → 72.012 |
| strategy_history_pipeline / hostJsonPipeline | 166.833 → 171.363 | 49.337 → 49.337 | 49.329 → 49.329 | 121.504 → 121.496 | 121.504 → 121.496 |
| strategy_history_pipeline / parseJson | 342.551 → 350.647 | 337.112 → 337.112 | 315.113 → 315.113 | 480.660 → 480.680 | 480.660 → 480.680 |
| strategy_history_staged / dataset | 1.149 → 1.206 | 4.578 → 4.578 | 4.578 → 4.578 | 8.551 → 8.566 | 8.551 → 8.566 |
| strategy_history_staged / analyze | 0.481 → 0.587 | 0.043 → 0.043 | 0.020 → 0.020 | 9.805 → 9.824 | 9.805 → 9.824 |
| strategy_history_staged / init | 0.035 → 0.042 | 0.004 → 0.004 | 0.003 → 0.003 | 9.879 → 9.910 | 9.879 → 9.910 |
| strategy_history_staged / seed | 365.882 → 388.041 | 159.294 → 159.294 | 50.989 → 50.989 | 72.195 → 71.934 | 72.195 → 71.934 |
| strategy_history_staged / cowClone | 0.030 → 0.029 | 0.002 → 0.002 | 0.002 → 0.002 | 72.258 → 71.996 | 72.258 → 71.996 |
| strategy_history_staged / ownedResult | 43.884 → 46.457 | 45.729 → 45.729 | 45.729 → 45.729 | 127.316 → 126.973 | 127.316 → 126.973 |
| strategy_history_staged / serializeOwnedResult | 159.825 → 159.101 | 49.328 → 49.328 | 49.328 → 49.328 | 176.746 → 176.410 | 176.746 → 176.410 |
| strategy_history_staged / parseJson | 334.697 → 334.247 | 337.112 → 337.112 | 315.113 → 315.113 | 535.914 → 535.387 | 535.914 → 535.387 |
| drawing_history_pipeline / dataset | 1.265 → 1.162 | 4.578 → 4.578 | 4.578 → 4.578 | 8.555 → 8.555 | 8.555 → 8.555 |
| drawing_history_pipeline / analyze | 0.588 → 0.547 | 0.037 → 0.037 | 0.016 → 0.016 | 9.801 → 9.805 | 9.801 → 9.805 |
| drawing_history_pipeline / init | 0.035 → 0.040 | 0.004 → 0.004 | 0.004 → 0.004 | 9.879 → 9.898 | 9.879 → 9.898 |
| drawing_history_pipeline / seed | 365.510 → 365.011 | 117.773 → 117.773 | 54.751 → 54.751 | 76.047 → 76.102 | 76.047 → 76.102 |
| drawing_history_pipeline / hostJsonPipeline | 63.146 → 62.692 | 29.353 → 29.353 | 29.350 → 29.350 | 105.527 → 105.578 | 105.527 → 105.578 |
| drawing_history_pipeline / parseJson | 222.267 → 215.667 | 224.467 → 224.467 | 216.467 → 216.467 | 353.965 → 354.039 | 353.965 → 354.039 |
| drawing_history_staged / dataset | 0.970 → 0.994 | 4.578 → 4.578 | 4.578 → 4.578 | 8.555 → 8.562 | 8.555 → 8.562 |
| drawing_history_staged / analyze | 0.526 → 0.512 | 0.037 → 0.037 | 0.016 → 0.016 | 9.801 → 9.816 | 9.801 → 9.816 |
| drawing_history_staged / init | 0.038 → 0.038 | 0.004 → 0.004 | 0.004 → 0.004 | 9.879 → 9.910 | 9.879 → 9.910 |
| drawing_history_staged / seed | 348.204 → 359.671 | 117.773 → 117.773 | 54.751 → 54.751 | 76.133 → 76.039 | 76.133 → 76.039 |
| drawing_history_staged / cowClone | 0.025 → 0.026 | 0.001 → 0.001 | 0.001 → 0.001 | 76.195 → 76.102 | 76.195 → 76.102 |
| drawing_history_staged / ownedResult | 44.795 → 47.185 | 56.733 → 56.733 | 56.733 → 56.733 | 143.508 → 143.414 | 143.508 → 143.414 |
| drawing_history_staged / serializeOwnedResult | 56.648 → 62.315 | 29.350 → 29.350 | 29.350 → 29.350 | 172.941 → 172.852 | 172.941 → 172.852 |
| drawing_history_staged / parseJson | 208.685 → 206.716 | 224.467 → 224.467 | 216.467 → 216.467 | 421.340 → 421.332 | 421.340 → 421.332 |
| gradient_history_pipeline / dataset | 1.144 → 1.257 | 4.578 → 4.578 | 4.578 → 4.578 | 8.551 → 8.551 | 8.551 → 8.551 |
| gradient_history_pipeline / analyze | 0.577 → 0.695 | 0.079 → 0.079 | 0.027 → 0.027 | 9.852 → 9.863 | 9.852 → 9.863 |
| gradient_history_pipeline / init | 0.034 → 0.045 | 0.006 → 0.006 | 0.005 → 0.005 | 9.926 → 9.953 | 9.926 → 9.953 |
| gradient_history_pipeline / seed | 395.449 → 400.393 | 65.680 → 65.680 | 15.814 → 15.814 | 26.719 → 26.590 | 26.719 → 26.590 |
| gradient_history_pipeline / hostJsonPipeline | 40.842 → 41.944 | 10.115 → 10.115 | 10.108 → 10.108 | 36.973 → 36.844 | 36.973 → 36.844 |
| gradient_history_pipeline / parseJson | 61.503 → 62.120 | 95.900 → 95.900 | 79.900 → 79.900 | 119.914 → 119.727 | 119.914 → 119.727 |
| gradient_history_staged / dataset | 1.083 → 1.369 | 4.578 → 4.578 | 4.578 → 4.578 | 8.555 → 8.551 | 8.555 → 8.551 |
| gradient_history_staged / analyze | 0.623 → 0.714 | 0.079 → 0.079 | 0.027 → 0.027 | 9.855 → 9.863 | 9.855 → 9.863 |
| gradient_history_staged / init | 0.038 → 0.040 | 0.006 → 0.006 | 0.005 → 0.005 | 9.930 → 9.953 | 9.930 → 9.953 |
| gradient_history_staged / seed | 403.794 → 397.275 | 65.680 → 65.680 | 15.814 → 15.814 | 26.738 → 26.586 | 26.738 → 26.586 |
| gradient_history_staged / cowClone | 0.034 → 0.029 | 0.002 → 0.002 | 0.002 → 0.002 | 26.801 → 26.648 | 26.801 → 26.648 |
| gradient_history_staged / ownedResult | 8.884 → 7.774 | 21.364 → 21.364 | 21.364 → 21.364 | 48.230 → 48.074 | 48.230 → 48.074 |
| gradient_history_staged / serializeOwnedResult | 38.442 → 36.389 | 10.108 → 10.108 | 10.107 → 10.107 | 58.434 → 58.281 | 58.434 → 58.281 |
| gradient_history_staged / parseJson | 63.535 → 59.189 | 95.900 → 95.900 | 79.900 → 79.900 | 141.340 → 141.383 | 141.340 → 141.383 |
| metadata_many_pipeline / dataset | 0.002 → 0.002 | 0.003 → 0.003 | 0.003 → 0.003 | 4.004 → 4.004 | 4.004 → 4.004 |
| metadata_many_pipeline / analyze | 4.183 → 4.609 | 5.119 → 5.119 | 1.969 → 1.969 | 6.586 → 6.602 | 7.258 → 7.281 |
| metadata_many_pipeline / init | 0.108 → 0.125 | 0.104 → 0.104 | 0.062 → 0.062 | 6.684 → 6.742 | 7.258 → 7.281 |
| metadata_many_pipeline / seed | 30.219 → 30.474 | 6.615 → 6.615 | 1.793 → 1.793 | 9.188 → 9.219 | 9.188 → 9.219 |
| metadata_many_pipeline / hostJsonPipeline | 4.653 → 4.628 | 0.357 → 0.357 | 0.357 → 0.357 | 9.703 → 9.703 | 9.703 → 9.703 |
| metadata_many_pipeline / parseJson | 1.469 → 1.447 | 2.717 → 2.717 | 1.587 → 1.587 | 10.812 → 10.801 | 10.812 → 10.801 |
| metadata_many_staged / dataset | 0.002 → 0.002 | 0.003 → 0.003 | 0.003 → 0.003 | 4.008 → 4.008 | 4.008 → 4.008 |
| metadata_many_staged / analyze | 4.687 → 4.640 | 5.119 → 5.119 | 1.969 → 1.969 | 6.590 → 6.598 | 7.246 → 7.297 |
| metadata_many_staged / init | 0.113 → 0.115 | 0.104 → 0.104 | 0.062 → 0.062 | 6.719 → 6.754 | 7.246 → 7.297 |
| metadata_many_staged / seed | 31.842 → 31.813 | 6.615 → 6.615 | 1.793 → 1.793 | 9.207 → 9.203 | 9.207 → 9.203 |
| metadata_many_staged / cowClone | 0.033 → 0.027 | 0.001 → 0.001 | 0.001 → 0.001 | 9.270 → 9.266 | 9.270 → 9.266 |
| metadata_many_staged / ownedResult | 1.429 → 1.204 | 2.680 → 2.680 | 2.680 → 2.680 | 11.891 → 11.887 | 11.891 → 11.887 |
| metadata_many_staged / serializeOwnedResult | 4.744 → 4.524 | 0.430 → 0.430 | 0.357 → 0.357 | 12.320 → 12.355 | 12.320 → 12.355 |
| metadata_many_staged / parseJson | 1.629 → 1.506 | 2.717 → 2.717 | 1.587 → 1.587 | 13.773 → 13.738 | 13.773 → 13.738 |

## 资格边界与剩余技术债

本轮仍不是完整 f64 数值资格：只有输入逐元素位可逆的缩放恢复子集被确认；不可逆非三角输入明确 na，近秩亏与固定阈值仍是遗留风险。恢复 eigenvalues 的归一化根有限，但回乘尺度可能溢出，公开层沿用逐元素 finite_float_or_na；不能据此承诺全部根可表示。普通 QR 的收敛和 pinv 的 Gram/Jacobi 阈值仍需独立资格与预算。

HMA/WMA 仍 O(length)，variance/stdev/BB/correlation 仍扫描窗口；要改为递推需先确定允许的浮点兼容性合同。typed opcode 只覆盖 TA，其他 builtin 家族和表达式执行仍有字符串/树解释开销；本轮没有实现字节码 VM。

额外 3 个已实测遗留缺陷：3 sources × seed 1/2/17/257 × before/after，共 24 个新进程、12 组逐字相同结果，作为独立诊断，不计入上述 366 ordinary / 26 correctness 分母。原始 receipt 与解析依据也保存在 JSON。

| 遗留输入 | 当前实际结果 | 数学结果 / 后续目标 |
|---|---|---|
| x 在 1e8 与 1e8+1 间交替，correlation(x,x,2) | 0；同窗 variance 正确为 0.25 | correlation=1；原始乘积均值差发生消去 |
| 前 2 bars 为 1e308，之后为 1，length=2 | bar_index>=3 后 variance/stdev/BB 仍全部 na（236 个合格 snapshot） | 旧大值已移出后应恢复 [0,0,1,1,1]；滚动 sum 非有限污染仍存在 |
| [[1,1e-8],[-1e-8,1]] | eigenvalues=[1,1]，vectors 有值 | 实根不存在，根为 1±i·1e-8；普通判别式消去与容差路径仍需资格 |

请求分配只计 allocator 请求，不含分配器开销；Windows RSS 为工作集，process peak 自启动累计。三个样本的中位数只描述本机受控结果，不构成统计显著性、严格最坏界或发布资格。

全部样本、兼容性计划、解析答案、源/库/探针身份与可重跑 probe 源见 [NUMERIC_DISPATCH_REPAIR_RESULTS_20261004.json](NUMERIC_DISPATCH_REPAIR_RESULTS_20261004.json)。原始 stdout/stderr、门禁日志和冻结产物保留在 `.local/numeric-dispatch-20261004`；报告追加不修改已验证代码。
