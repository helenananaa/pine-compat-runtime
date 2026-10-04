# 直接输出编码修复（2026-10-04）

上一批已提交为 `80d6e501ab4dc761616ac290b209724cdb3624cd`，未推送。本轮继续减少输出临时分配，比较基线为该提交的冻结源码与上一轮最终 release 产物；保留原 owned ABI、字段顺序、浮点表示和转义规则。

## 修复

六类 drawing 直接编码标量、table cells、merged cells 和 polyline points，移除每个 snapshot 的临时 String、重复包装和字符串截取。策略 order/trade/position/equity/fill-alert、普通 alerts 与 diagnostics 也直接写 sink。完整 owned/view/consuming 输出与 String-returning delta helpers 共用记录规则，减少重复代码。静态 metadata 及完整 delta String 仍有分配，不宣称全输出零分配。

## 验证

| 平台 | Rust | Python | 工具测试 / skipped |
|---|---:|---:|---:|
| windows | 7237 | 785 | 166 / 0 |
| linux | 7237 | 785 | 166 / 1 |

Windows/Linux 均通过 canonical verify 门禁：fmt、严格 Clippy 全 workspace/all targets、完整 Rust、工具、结构、host parity、真实 WASM/Node、新建 wheel/venv 的 Python 测试。结构检查 394 个生产 Rust 文件；未修改阈值或黄金数据。

288/288 组冻结产物差分通过：9,504 份完整 snapshot 与 9,216 份实时 delta 逐字一致，包含保留历史、forming 替换和 confirmed 提交。前后 probe 都另验证 owned/view 与 confirmed owned/view 相等及 replica 重建一致。

新增独立固定字节检查：字段顺序、Unicode/ASCII 控制字符、可选字符串、非有限数 null、负零 -0。新增资源检查：slice-backed 输出包含 1,024 个 table cells、10,000 个 polyline points 及长 Unicode/控制字符文本、告警和诊断，直接 sink 编码无临时堆分配。运行时 10k label snapshots 编码仅有小额树遍历游标分配，请求总量小于 1 KiB。验证每次最多写 7 bytes 的 sink 在大型 payload 内失败，输出前缀正确，失败后不再调用 sink，借用结果不变。

## 本机受控性能

6 cases、24 个独立新进程，pipeline 每版本 3 次交替执行，staged 每版本 1 次。前后都使用借用视图和同样 count+encode 路径；本轮没有把上轮省 owned 复制的收益重复计入。下面包含所有阶段与样本的中位数。测量期间无 Cargo/完整门禁编译。

策略历史 pipeline 中位数 673.812 → 160.194 ms，drawing 历史 395.199 → 57.416 ms；请求分配分别 532.760 → 49.337 MiB 与 264.977 → 29.354 MiB。纯 plot 对照耗时基本持平，本轮进程峰值 RSS 也基本不变：完整结果 String 仍决定主要峰值，不能把请求分配下降解释为 RSS 等比例下降。

| case / stage | ms：前 → 后 | 请求分配 MiB：前 → 后 | 额外逻辑峰值 MiB：前 → 后 | 结束 RSS MiB：前 → 后 | 累计进程峰值 RSS MiB：前 → 后 |
|---|---:|---:|---:|---:|---:|
| output_heavy_pipeline / dataset | 0.997 → 1.042 | 4.578 → 4.578 | 4.578 → 4.578 | 8.566 → 8.555 | 8.566 → 8.555 |
| output_heavy_pipeline / analyze | 0.516 → 0.612 | 0.071 → 0.071 | 0.025 → 0.025 | 9.738 → 9.746 | 9.738 → 9.746 |
| output_heavy_pipeline / init | 0.034 → 0.040 | 0.005 → 0.005 | 0.004 → 0.004 | 9.816 → 9.816 | 9.816 → 9.816 |
| output_heavy_pipeline / seed | 764.214 → 771.348 | 105.562 → 105.562 | 26.385 → 26.385 | 37.648 → 37.555 | 37.648 → 37.555 |
| output_heavy_pipeline / hostJsonPipeline | 116.245 → 116.940 | 15.682 → 15.682 | 15.653 → 15.653 | 53.426 → 53.340 | 53.426 → 53.340 |
| output_heavy_pipeline / parseJson | 66.909 → 68.126 | 156.582 → 156.582 | 80.585 → 80.585 | 125.766 → 126.000 | 126.707 → 126.941 |

output_heavy_pipeline 输出大小 16,411,594 bytes，前后相同；完整字节一致性由独立差分验证。

| output_heavy_staged / dataset | 0.965 → 1.021 | 4.578 → 4.578 | 4.578 → 4.578 | 8.562 → 8.555 | 8.562 → 8.555 |
| output_heavy_staged / analyze | 0.516 → 0.516 | 0.071 → 0.071 | 0.025 → 0.025 | 9.734 → 9.746 | 9.734 → 9.746 |
| output_heavy_staged / init | 0.033 → 0.045 | 0.005 → 0.005 | 0.004 → 0.004 | 9.812 → 9.816 | 9.812 → 9.816 |
| output_heavy_staged / seed | 759.437 → 767.104 | 105.562 → 105.562 | 26.385 → 26.385 | 37.641 → 37.551 | 37.641 → 37.551 |
| output_heavy_staged / cowClone | 0.070 → 0.058 | 0.004 → 0.004 | 0.004 → 0.004 | 37.707 → 37.621 | 37.707 → 37.621 |
| output_heavy_staged / ownedResult | 43.355 → 43.107 | 74.771 → 74.771 | 74.771 → 74.771 | 120.312 → 120.227 | 120.312 → 120.227 |
| output_heavy_staged / serializeOwnedResult | 106.709 → 107.122 | 15.654 → 15.654 | 15.652 → 15.652 | 136.055 → 135.973 | 136.055 → 135.973 |
| output_heavy_staged / parseJson | 66.386 → 66.892 | 156.582 → 156.582 | 80.585 → 80.585 | 208.684 → 208.395 | 209.625 → 209.336 |

output_heavy_staged 输出大小 16,411,594 bytes，前后相同；完整字节一致性由独立差分验证。

| strategy_history_pipeline / dataset | 0.996 → 1.022 | 4.578 → 4.578 | 4.578 → 4.578 | 8.559 → 8.555 | 8.559 → 8.555 |
| strategy_history_pipeline / analyze | 0.516 → 0.525 | 0.043 → 0.043 | 0.020 → 0.020 | 9.805 → 9.805 | 9.805 → 9.805 |
| strategy_history_pipeline / init | 0.033 → 0.032 | 0.004 → 0.004 | 0.003 → 0.003 | 9.887 → 9.879 | 9.887 → 9.879 |
| strategy_history_pipeline / seed | 490.913 → 490.686 | 182.185 → 182.185 | 50.990 → 50.990 | 72.070 → 72.180 | 72.070 → 72.180 |
| strategy_history_pipeline / hostJsonPipeline | 673.812 → 160.194 | 532.760 → 49.337 | 49.329 → 49.329 | 121.570 → 121.656 | 121.570 → 121.656 |
| strategy_history_pipeline / parseJson | 310.772 → 327.887 | 337.112 → 337.112 | 315.113 → 315.113 | 480.734 → 480.906 | 480.734 → 480.906 |

strategy_history_pipeline 输出大小 51,724,395 bytes，前后相同；完整字节一致性由独立差分验证。

| strategy_history_staged / dataset | 1.033 → 1.084 | 4.578 → 4.578 | 4.578 → 4.578 | 8.535 → 8.551 | 8.535 → 8.551 |
| strategy_history_staged / analyze | 0.536 → 0.523 | 0.043 → 0.043 | 0.020 → 0.020 | 9.777 → 9.801 | 9.777 → 9.801 |
| strategy_history_staged / init | 0.037 → 0.030 | 0.004 → 0.004 | 0.003 → 0.003 | 9.859 → 9.875 | 9.859 → 9.875 |
| strategy_history_staged / seed | 500.427 → 499.573 | 182.185 → 182.185 | 50.990 → 50.990 | 72.031 → 72.004 | 72.031 → 72.004 |
| strategy_history_staged / cowClone | 0.034 → 0.028 | 0.003 → 0.003 | 0.003 → 0.003 | 72.098 → 72.074 | 72.098 → 72.074 |
| strategy_history_staged / ownedResult | 42.853 → 41.671 | 45.729 → 45.729 | 45.729 → 45.729 | 127.164 → 127.121 | 127.164 → 127.121 |
| strategy_history_staged / serializeOwnedResult | 674.343 → 155.026 | 532.751 → 49.329 | 49.329 → 49.328 | 176.652 → 176.555 | 176.652 → 176.555 |
| strategy_history_staged / parseJson | 321.420 → 316.241 | 337.112 → 337.112 | 315.113 → 315.113 | 535.895 → 535.688 | 535.895 → 535.688 |

strategy_history_staged 输出大小 51,724,395 bytes，前后相同；完整字节一致性由独立差分验证。

| drawing_history_pipeline / dataset | 0.904 → 0.939 | 4.578 → 4.578 | 4.578 → 4.578 | 8.555 → 8.551 | 8.555 → 8.551 |
| drawing_history_pipeline / analyze | 0.517 → 0.499 | 0.037 → 0.037 | 0.016 → 0.016 | 9.781 → 9.785 | 9.781 → 9.785 |
| drawing_history_pipeline / init | 0.032 → 0.034 | 0.004 → 0.004 | 0.004 → 0.004 | 9.867 → 9.863 | 9.867 → 9.863 |
| drawing_history_pipeline / seed | 477.582 → 474.776 | 141.227 → 141.227 | 54.753 → 54.753 | 76.012 → 76.031 | 76.012 → 76.031 |
| drawing_history_pipeline / hostJsonPipeline | 395.199 → 57.416 | 264.977 → 29.354 | 29.350 → 29.350 | 105.500 → 105.504 | 105.500 → 105.504 |
| drawing_history_pipeline / parseJson | 212.475 → 204.267 | 224.467 → 224.467 | 216.467 → 216.467 | 353.945 → 353.977 | 353.945 → 353.977 |

drawing_history_pipeline 输出大小 30,775,041 bytes，前后相同；完整字节一致性由独立差分验证。

| drawing_history_staged / dataset | 1.171 → 1.040 | 4.578 → 4.578 | 4.578 → 4.578 | 8.555 → 8.555 | 8.555 → 8.555 |
| drawing_history_staged / analyze | 0.572 → 0.515 | 0.037 → 0.037 | 0.016 → 0.016 | 9.777 → 9.789 | 9.777 → 9.789 |
| drawing_history_staged / init | 0.041 → 0.033 | 0.004 → 0.004 | 0.004 → 0.004 | 9.863 → 9.867 | 9.863 → 9.867 |
| drawing_history_staged / seed | 465.951 → 470.543 | 141.227 → 141.227 | 54.753 → 54.753 | 76.059 → 75.984 | 76.059 → 75.984 |
| drawing_history_staged / cowClone | 0.036 → 0.030 | 0.004 → 0.004 | 0.004 → 0.004 | 76.125 → 76.055 | 76.125 → 76.055 |
| drawing_history_staged / ownedResult | 44.752 → 46.530 | 56.733 → 56.733 | 56.733 → 56.733 | 143.430 → 143.426 | 143.430 → 143.426 |
| drawing_history_staged / serializeOwnedResult | 389.725 → 57.309 | 264.973 → 29.350 | 29.350 → 29.350 | 172.887 → 172.859 | 172.887 → 172.859 |
| drawing_history_staged / parseJson | 205.204 → 214.406 | 224.467 → 224.467 | 216.467 → 216.467 | 421.391 → 421.254 | 421.391 → 421.254 |

drawing_history_staged 输出大小 30,775,041 bytes，前后相同；完整字节一致性由独立差分验证。

## 测量边界与剩余技术债

请求字节不包含分配器开销，也不表示 realloc 瞬时双缓冲峰值。RSS 为 Windows 工作集；process peak 自启动累计。单次 staged probe 刻意保留先前阶段对象，不能当作独立 pipeline 的峰值；普通 dataset/analyze/seed 和 parse 的小幅波动均原样保留。

持久化历史的树遍历游标仍有小额分配。完整 JSON String、解析后的 JSON 树、Python dict/list 仍需要输出规模内存。plot/fill/hline 静态 metadata 及 public_runtime_changes_json 的分段 String 是后续输出热点。更深层 HIR/内建分派、WMA/方差浮点资格和 pinv/SVD 数值资格仍需单独处理；本轮不改变这些数值语义。

API 见 [RUST_EMBEDDING.md](RUST_EMBEDDING.md)，完整样本/身份/probe/scenario 源码见 [DIRECT_OUTPUT_REPAIR_RESULTS_20261004.json](DIRECT_OUTPUT_REPAIR_RESULTS_20261004.json)。原输出、退出状态与冻结产物保留在 `.local/direct-encoding-20261004`。
