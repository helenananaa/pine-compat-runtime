# Metadata、增量输出与零深度历史修复（2026-10-04）

比较基线为已提交的 `d4fab72a91ee594b1a70aee7e778ba5346bb61e0`。冻结 6281 份源码和 501 份 release 产物后修复；最终 6288 份源码同时用于 Windows/Linux 门禁。现有 owned API、JSON 顺序、浮点和默认省略规则保持兼容。

## 修复

- Metadata 默认值借用静态字符串，按 PineValue 变体判断，避免重复构造默认 String；plot/fill/hline/header 共用类型化规则。Gradient 单样本直接写 sink，保留 serde_json 的 1.0、-0.0、非有限 null 表示。
- 新增 write_public_runtime_changes_json。所有 series fields、drawing、gradient、事件与策略 splice 直接编码；保留动作、可选字段、省略规则及 absent/present-empty 区别。现有 public_runtime_changes_json 使用单遍可增长 Vec 返回 String。
- Prepared metadata 复用有效 retention，只有实际保留历史的 series 才记录当前样本；跳过 Some(0) 的克隆、HashMap 写入和提交丢弃。当前符号、var/varip、求值、执行步数及动态历史诊断保持原路径，支持稀疏和手写 HIR。

## 验证

| 平台 | Rust | Python | 工具测试 / skipped |
|---|---:|---:|---:|
| windows | 7251 | 785 | 166 / 0 |
| linux | 7251 | 785 | 166 / 1 |

两平台 canonical verify 均通过 fmt、严格 Clippy、完整 workspace 测试、结构/工具、host parity、真实 WASM/Node、新 wheel 与新 venv 的 Python 测试。结构检查 396 个生产 Rust 文件；没有调整结构阈值或现有黄金数据。

292/292 组差分通过：9,636 份 snapshot 与 9,344 份 delta 逐字相等。每组同时检查 owned/view、confirmed owned/view 和 replica 重建；覆盖 unlimited/retained、forming 替换和 confirmed 提交。长窗口 seed 为 6,000 bars，600-plot metadata 输入为 64 bars。

新增 14 项 Rust 测试：六项零深度语义边界；五份来自冻结旧库的独立 delta 参考字节；全动作/全部 series fields/typed defaults/Unicode、控制符、空 splice、非有限数和负零；逐字节故障、短写、dyn Write 与 WriteZero；大标题和 4,096 个 gradient samples 的 sink 编码无临时堆分配。分配检查只涵盖已有 changes 或 slice-backed view 到 sink，不包含创建输入、view headers、持久树游标或返回 String。

## 本机受控性能

40 cases、220 个新进程，通常每版本 3 次交替运行；staged 归因每版本 1 次。另有三个 delta 压力 case、18 个新进程。测量时编译、完整门禁和差分均已结束。所有数值及样本包含在 JSON 报告，表格保留对照与慢化项。

### 执行与历史状态

| case | bars / mode | seed 或 apply/update ms：前 → 后 | 请求分配 bytes：前 → 后 | 保留逻辑堆 bytes：前 → 后 |
|---|---|---:|---:|---:|
| highest_16 | 24000 / historical | 107.044 → 56.511 | 9309532 → 3931188 | 1069089 → 1067941 |
| highest_512 | 24000 / historical | 108.075 → 56.688 | 11315480 → 5937136 | 78653 → 77505 |
| highest_5000 | 24000 / historical | 106.828 → 57.176 | 11648336 → 6269992 | 225029 → 223881 |
| lowest_16 | 24000 / historical | 111.168 → 56.002 | 9313116 → 3934772 | 1070880 → 1069732 |
| lowest_512 | 24000 / historical | 109.627 → 57.403 | 12578440 → 7200096 | 78844 → 77696 |
| lowest_5000 | 24000 / historical | 110.404 → 59.585 | 12911296 → 7532952 | 225220 → 224072 |
| highestbars_16 | 24000 / historical | 107.496 → 56.412 | 8800924 → 3422580 | 819749 → 818601 |
| highestbars_512 | 24000 / historical | 110.150 → 59.033 | 12805320 → 7426976 | 838785 → 837637 |
| highestbars_5000 | 24000 / historical | 106.527 → 59.718 | 13483776 → 8105432 | 986121 → 984973 |
| lowestbars_16 | 24000 / historical | 112.714 → 56.919 | 8800588 → 3422244 | 819492 → 818344 |
| lowestbars_512 | 24000 / historical | 107.723 → 57.145 | 14067944 → 8689600 | 838720 → 837572 |
| lowestbars_5000 | 24000 / historical | 107.982 → 55.900 | 14746400 → 9368056 | 986056 → 984908 |
| rci_512 | 6000 / historical | 48.800 → 42.014 | 2224312 → 781788 | 224353 → 223025 |
| rci_5000 | 6000 / historical | 54.114 → 48.319 | 2469368 → 1026844 | 357601 → 356273 |
| mode_512 | 6000 / historical | 39.658 → 31.471 | 2212056 → 769532 | 216162 → 214834 |
| mode_5000 | 6000 / historical | 45.238 → 36.302 | 2298328 → 855804 | 252066 → 250738 |
| common_plot | 100000 / historical | 304.831 → 177.473 | 36626756 → 12624232 | 3383959 → 3382631 |
| common_sma14 | 100000 / historical | 334.567 → 189.611 | 36632936 → 12630412 | 3387473 → 3386145 |
| common_ema14 | 100000 / historical | 356.424 → 218.108 | 46233052 → 22230528 | 3387685 → 3386357 |
| common_rsi14 | 100000 / historical | 340.729 → 211.910 | 36634524 → 12632000 | 3388389 → 3387061 |
| common_strategy | 100000 / historical | 343.905 → 211.917 | 52614460 → 28611936 | 7496782 → 7495454 |
| highest_descending_5000_historical | 24000 / historical | 118.464 → 65.309 | 13431968 → 8053624 | 914005 → 912857 |
| highest_descending_5000_realtime | 24000 / realtime | 0.018 → 0.019 / 0.540 → 0.633 | 26136 → 24764 / 1558096 → 1556724 | 2066021 → 2064873 |
| highest_descending_5000_retained | 24000 / retained | 0.012 → 0.010 / 0.013 → 0.011 | 26024 → 24652 / 26080 → 24708 | 1415973 → 1414825 |
| lowest_ascending_5000_historical | 24000 / historical | 111.674 → 60.433 | 13431968 → 8053624 | 914004 → 912856 |
| lowest_ascending_5000_realtime | 24000 / realtime | 0.021 → 0.017 / 0.671 → 0.605 | 26136 → 24764 / 1558096 → 1556724 | 2066020 → 2064872 |
| lowest_ascending_5000_retained | 24000 / retained | 0.013 → 0.010 / 0.013 → 0.011 | 26024 → 24652 / 26080 → 24708 | 1415972 → 1414824 |
| unchanged_control_wma_512 | 20000 / historical | 78.232 → 51.930 | 7329288 → 2526764 | 684417 → 683089 |
| unchanged_control_stdev_512 | 20000 / historical | 80.061 → 47.143 | 7332024 → 2529500 | 684419 → 683091 |
| math_dispatch | 20000 / append | 408.214 → 254.687 | probe 未计量 | probe 未计量 |

### 完整输出各阶段

前后使用同样的 borrowed view 和 count+encode 路径，没有重复计入上一轮省 owned 复制的收益。单次 staged 保留先前阶段对象，不能当作独立 pipeline 峰值。

| case / stage | ms：前 → 后 | 请求分配 MiB：前 → 后 | 额外逻辑峰值 MiB：前 → 后 | 结束 RSS MiB：前 → 后 | 累计峰值 RSS MiB：前 → 后 |
|---|---:|---:|---:|---:|---:|
| output_heavy_pipeline / dataset | 1.354 → 1.343 | 4.578 → 4.578 | 4.578 → 4.578 | 8.555 → 8.562 | 8.555 → 8.562 |
| output_heavy_pipeline / analyze | 0.835 → 0.788 | 0.071 → 0.071 | 0.025 → 0.025 | 9.742 → 9.742 | 9.742 → 9.742 |
| output_heavy_pipeline / init | 0.055 → 0.050 | 0.005 → 0.005 | 0.004 → 0.004 | 9.812 → 9.820 | 9.812 → 9.820 |
| output_heavy_pipeline / seed | 974.799 → 815.545 | 105.562 → 82.671 | 26.385 → 26.384 | 37.637 → 37.621 | 37.637 → 37.621 |
| output_heavy_pipeline / hostJsonPipeline | 147.170 → 151.900 | 15.682 → 15.682 | 15.653 → 15.653 | 53.422 → 53.410 | 53.422 → 53.410 |
| output_heavy_pipeline / parseJson | 87.656 → 90.116 | 156.582 → 156.582 | 80.585 → 80.585 | 125.648 → 125.965 | 126.590 → 126.906 |
| output_heavy_staged / dataset | 1.173 → 1.141 | 4.578 → 4.578 | 4.578 → 4.578 | 8.555 → 8.559 | 8.555 → 8.559 |
| output_heavy_staged / analyze | 0.576 → 0.545 | 0.071 → 0.071 | 0.025 → 0.025 | 9.734 → 9.746 | 9.734 → 9.746 |
| output_heavy_staged / init | 0.034 → 0.034 | 0.005 → 0.005 | 0.004 → 0.004 | 9.805 → 9.824 | 9.805 → 9.824 |
| output_heavy_staged / seed | 904.787 → 721.478 | 105.562 → 82.671 | 26.385 → 26.384 | 37.637 → 37.609 | 37.637 → 37.609 |
| output_heavy_staged / cowClone | 0.064 → 0.059 | 0.004 → 0.002 | 0.004 → 0.002 | 37.707 → 37.680 | 37.707 → 37.680 |
| output_heavy_staged / ownedResult | 50.589 → 49.541 | 74.771 → 74.771 | 74.771 → 74.771 | 120.348 → 120.281 | 120.348 → 120.281 |
| output_heavy_staged / serializeOwnedResult | 122.835 → 125.696 | 15.654 → 15.653 | 15.652 → 15.652 | 136.094 → 136.031 | 136.094 → 136.031 |
| output_heavy_staged / parseJson | 80.383 → 77.820 | 156.582 → 156.582 | 80.585 → 80.585 | 208.316 → 208.621 | 209.258 → 209.625 |
| strategy_history_pipeline / dataset | 1.458 → 1.416 | 4.578 → 4.578 | 4.578 → 4.578 | 8.551 → 8.559 | 8.551 → 8.559 |
| strategy_history_pipeline / analyze | 0.676 → 0.611 | 0.043 → 0.043 | 0.020 → 0.020 | 9.801 → 9.820 | 9.801 → 9.820 |
| strategy_history_pipeline / init | 0.046 → 0.043 | 0.004 → 0.004 | 0.003 → 0.003 | 9.875 → 9.902 | 9.875 → 9.902 |
| strategy_history_pipeline / seed | 637.457 → 439.767 | 182.185 → 159.294 | 50.990 → 50.989 | 72.152 → 72.062 | 72.152 → 72.062 |
| strategy_history_pipeline / hostJsonPipeline | 198.219 → 195.720 | 49.337 → 49.337 | 49.329 → 49.329 | 121.629 → 121.539 | 121.629 → 121.539 |
| strategy_history_pipeline / parseJson | 409.380 → 404.166 | 337.112 → 337.112 | 315.113 → 315.113 | 480.844 → 480.664 | 480.844 → 480.664 |
| strategy_history_staged / dataset | 1.265 → 1.378 | 4.578 → 4.578 | 4.578 → 4.578 | 8.555 → 8.562 | 8.555 → 8.562 |
| strategy_history_staged / analyze | 0.612 → 0.622 | 0.043 → 0.043 | 0.020 → 0.020 | 9.801 → 9.836 | 9.801 → 9.836 |
| strategy_history_staged / init | 0.039 → 0.040 | 0.004 → 0.004 | 0.003 → 0.003 | 9.875 → 9.918 | 9.875 → 9.918 |
| strategy_history_staged / seed | 606.361 → 419.500 | 182.185 → 159.294 | 50.990 → 50.989 | 72.172 → 72.082 | 72.172 → 72.082 |
| strategy_history_staged / cowClone | 0.041 → 0.040 | 0.003 → 0.002 | 0.003 → 0.002 | 72.242 → 72.145 | 72.242 → 72.145 |
| strategy_history_staged / ownedResult | 54.948 → 49.879 | 45.729 → 45.729 | 45.729 → 45.729 | 127.293 → 127.270 | 127.293 → 127.270 |
| strategy_history_staged / serializeOwnedResult | 184.715 → 183.383 | 49.329 → 49.328 | 49.328 → 49.328 | 176.727 → 176.703 | 176.727 → 176.703 |
| strategy_history_staged / parseJson | 390.884 → 396.625 | 337.112 → 337.112 | 315.113 → 315.113 | 536.004 → 535.680 | 536.004 → 535.680 |
| drawing_history_pipeline / dataset | 1.404 → 1.276 | 4.578 → 4.578 | 4.578 → 4.578 | 8.559 → 8.555 | 8.559 → 8.555 |
| drawing_history_pipeline / analyze | 0.631 → 0.648 | 0.037 → 0.037 | 0.016 → 0.016 | 9.797 → 9.785 | 9.797 → 9.785 |
| drawing_history_pipeline / init | 0.042 → 0.050 | 0.004 → 0.004 | 0.004 → 0.004 | 9.875 → 9.871 | 9.875 → 9.871 |
| drawing_history_pipeline / seed | 607.358 → 412.061 | 141.227 → 117.773 | 54.753 → 54.751 | 75.988 → 76.121 | 75.988 → 76.121 |
| drawing_history_pipeline / hostJsonPipeline | 76.154 → 70.371 | 29.354 → 29.353 | 29.350 → 29.350 | 105.461 → 105.598 | 105.461 → 105.598 |
| drawing_history_pipeline / parseJson | 276.352 → 263.771 | 224.467 → 224.467 | 216.467 → 216.467 | 353.957 → 353.992 | 353.957 → 353.992 |
| drawing_history_staged / dataset | 1.619 → 2.212 | 4.578 → 4.578 | 4.578 → 4.578 | 8.555 → 8.551 | 8.555 → 8.551 |
| drawing_history_staged / analyze | 0.709 → 0.839 | 0.037 → 0.037 | 0.016 → 0.016 | 9.789 → 9.785 | 9.789 → 9.785 |
| drawing_history_staged / init | 0.047 → 0.071 | 0.004 → 0.004 | 0.004 → 0.004 | 9.867 → 9.871 | 9.867 → 9.871 |
| drawing_history_staged / seed | 587.574 → 417.514 | 141.227 → 117.773 | 54.753 → 54.751 | 76.000 → 76.148 | 76.000 → 76.148 |
| drawing_history_staged / cowClone | 0.040 → 0.032 | 0.004 → 0.001 | 0.004 → 0.001 | 76.070 → 76.211 | 76.070 → 76.211 |
| drawing_history_staged / ownedResult | 59.157 → 52.557 | 56.733 → 56.733 | 56.733 → 56.733 | 143.441 → 143.547 | 143.441 → 143.547 |
| drawing_history_staged / serializeOwnedResult | 74.864 → 70.747 | 29.350 → 29.350 | 29.350 → 29.350 | 172.875 → 172.984 | 172.875 → 172.984 |
| drawing_history_staged / parseJson | 262.630 → 244.072 | 224.467 → 224.467 | 216.467 → 216.467 | 421.344 → 421.398 | 421.344 → 421.398 |
| gradient_history_pipeline / dataset | 1.325 → 1.298 | 4.578 → 4.578 | 4.578 → 4.578 | 8.551 → 8.555 | 8.551 → 8.555 |
| gradient_history_pipeline / analyze | 0.792 → 0.771 | 0.079 → 0.079 | 0.027 → 0.027 | 9.852 → 9.859 | 9.852 → 9.859 |
| gradient_history_pipeline / init | 0.046 → 0.042 | 0.006 → 0.006 | 0.005 → 0.005 | 9.926 → 9.941 | 9.926 → 9.941 |
| gradient_history_pipeline / seed | 649.887 → 469.414 | 88.573 → 65.680 | 15.816 → 15.814 | 26.773 → 26.750 | 26.773 → 26.750 |
| gradient_history_pipeline / hostJsonPipeline | 63.158 → 47.940 | 34.530 → 10.115 | 10.108 → 10.108 | 37.027 → 36.988 | 37.027 → 36.988 |
| gradient_history_pipeline / parseJson | 75.204 → 76.001 | 95.900 → 95.900 | 79.900 → 79.900 | 119.895 → 119.883 | 119.895 → 119.883 |
| gradient_history_staged / dataset | 1.309 → 1.237 | 4.578 → 4.578 | 4.578 → 4.578 | 8.559 → 8.559 | 8.559 → 8.559 |
| gradient_history_staged / analyze | 0.668 → 0.647 | 0.079 → 0.079 | 0.027 → 0.027 | 9.863 → 9.855 | 9.863 → 9.855 |
| gradient_history_staged / init | 0.037 → 0.041 | 0.006 → 0.006 | 0.005 → 0.005 | 9.938 → 9.938 | 9.938 → 9.938 |
| gradient_history_staged / seed | 658.577 → 435.385 | 88.573 → 65.680 | 15.816 → 15.814 | 26.789 → 26.566 | 26.789 → 26.566 |
| gradient_history_staged / cowClone | 0.041 → 0.038 | 0.004 → 0.002 | 0.004 → 0.002 | 26.859 → 26.633 | 26.859 → 26.633 |
| gradient_history_staged / ownedResult | 10.298 → 9.070 | 21.364 → 21.364 | 21.364 → 21.364 | 48.285 → 48.055 | 48.285 → 48.055 |
| gradient_history_staged / serializeOwnedResult | 58.638 → 40.493 | 34.522 → 10.108 | 10.107 → 10.107 | 58.504 → 58.258 | 58.504 → 58.258 |
| gradient_history_staged / parseJson | 79.698 → 67.436 | 95.900 → 95.900 | 79.900 → 79.900 | 141.672 → 141.465 | 141.672 → 141.465 |
| metadata_many_pipeline / dataset | 0.003 → 0.003 | 0.003 → 0.003 | 0.003 → 0.003 | 4.008 → 4.008 | 4.008 → 4.008 |
| metadata_many_pipeline / analyze | 5.263 → 5.373 | 5.119 → 5.119 | 1.969 → 1.969 | 6.602 → 6.613 | 7.289 → 7.285 |
| metadata_many_pipeline / init | 0.124 → 0.136 | 0.104 → 0.104 | 0.062 → 0.062 | 6.746 → 6.719 | 7.289 → 7.285 |
| metadata_many_pipeline / seed | 36.625 → 38.081 | 6.632 → 6.615 | 1.794 → 1.793 | 9.230 → 9.223 | 9.230 → 9.223 |
| metadata_many_pipeline / hostJsonPipeline | 5.968 → 5.218 | 0.593 → 0.357 | 0.357 → 0.357 | 9.750 → 9.727 | 9.750 → 9.727 |
| metadata_many_pipeline / parseJson | 1.556 → 1.672 | 2.717 → 2.717 | 1.587 → 1.587 | 10.855 → 10.832 | 10.855 → 10.832 |
| metadata_many_staged / dataset | 0.003 → 0.002 | 0.003 → 0.003 | 0.003 → 0.003 | 4.004 → 4.008 | 4.004 → 4.008 |
| metadata_many_staged / analyze | 5.508 → 4.845 | 5.119 → 5.119 | 1.969 → 1.969 | 6.484 → 6.590 | 7.285 → 7.234 |
| metadata_many_staged / init | 0.142 → 0.165 | 0.104 → 0.104 | 0.062 → 0.062 | 6.629 → 6.746 | 7.285 → 7.234 |
| metadata_many_staged / seed | 35.913 → 33.150 | 6.632 → 6.615 | 1.794 → 1.793 | 9.199 → 9.094 | 9.199 → 9.094 |
| metadata_many_staged / cowClone | 0.043 → 0.031 | 0.003 → 0.001 | 0.003 → 0.001 | 9.270 → 9.156 | 9.270 → 9.156 |
| metadata_many_staged / ownedResult | 1.563 → 1.412 | 2.680 → 2.680 | 2.680 → 2.680 | 11.910 → 11.832 | 11.910 → 11.832 |
| metadata_many_staged / serializeOwnedResult | 5.895 → 4.989 | 0.666 → 0.430 | 0.357 → 0.357 | 12.391 → 12.297 | 12.391 → 12.297 |
| metadata_many_staged / parseJson | 1.633 → 1.502 | 2.717 → 2.717 | 1.587 → 1.587 | 13.715 → 13.430 | 13.715 → 13.430 |

### Delta String 与 sink

三类输入各运行多次编码：series 256 次，gradient/drawing 各 64 次。表中 ms 与请求分配为整段循环总量，每版本三进程的中位数；每次输出都与旧版完整 JSON 字节相等。旧 API 没有 sink，before sink 测量包含旧 String API 和写入计数 sink。

| case / stage | ms：前 → 后 | 请求分配 MiB：前 → 后 | 额外逻辑峰值 MiB：前 → 后 | 累计峰值 RSS MiB：前 → 后 |
|---|---:|---:|---:|---:|
| series / ownedJson | 83.827 → 21.635 | 40.510 → 16.996 | 0.049 → 0.033 | 4.086 → 4.059 |
| series / sink | 83.405 → 18.653 | 40.510 → 0.000 | 0.049 → 0.000 | 4.340 → 4.152 |
| gradient / ownedJson | 23.115 → 10.829 | 102.275 → 33.999 | 0.479 → 0.266 | 4.492 → 4.426 |
| gradient / sink | 20.444 → 5.220 | 102.275 → 0.000 | 0.479 → 0.000 | 4.492 → 4.426 |
| drawing / ownedJson | 100.190 → 65.667 | 252.680 → 67.999 | 1.227 → 0.531 | 5.566 → 5.223 |
| drawing / sink | 96.263 → 33.317 | 252.680 → 0.000 | 1.227 → 0.000 | 5.566 → 5.223 |

## 取舍、测量边界与剩余问题

最初 delta String 使用计数后精确分配。独立初测 18 个进程中 drawing 中位数 87.204 → 90.100 ms，慢化约 3.3%；最终改成单遍 Vec 增长，避免重复编码。初测、源码 archive、已链接库和探针均保留，完整初测样本附在 JSON。最终 String 仍会有增长分配与输出内存，不宣称 owned API 零分配。

请求字节只计 allocator 请求，不包含分配器开销或 realloc 瞬时双缓冲；Windows RSS 为工作集，process peak 自启动累计。小幅中位数波动不是统计显著性或 release 性能资格；累计请求分配下降不等于 RSS 同比例下降。所有 unchanged WMA/stdev 对照和阶段慢化项均保留。

首次定向编译暴露 writer import 和新测试 profile 字段名问题，已修正并保留失败日志。差分预检拒绝了继承 probe 的源文件 SHA：只有 CRLF/LF 不同；使用相同当前源重新链接冻结旧库，未重标身份、未改变黄金结果。

下一轮优先项：TA 家族仍按 callee 字符串分派并扫描命名实参，预处理参数快路径目前主要覆盖 math；WMA/HMA、variance/stdev/BB/correlation 仍扫描窗口，普通递推存在改变原浮点末位的已知反例，需要独立数值资格；pinv/eigen 仍使用 Gram/Jacobi，数值秩、近秩亏和极端尺度资格以及确定性内核预算仍未解决。本轮未完成字节码/VM 改造或数值语义变更。

API 见 [RUST_EMBEDDING.md](RUST_EMBEDDING.md)，所有测量、源/库/探针身份、差分计划与可复核 probe 源见 [METADATA_DELTA_REPAIR_RESULTS_20261004.json](METADATA_DELTA_REPAIR_RESULTS_20261004.json)。原始输出、失败记录与冻结产物保留在 `.local/metadata-delta-20261004`。
