# Pivot 分页逆向游标 — 2026-10-06

基线 `6a7d20f685e441bc4d772df999e71c36c9ed2911`。原 paged next_back 每个样本重新 locate 和查页面；现在仅在第一次逆向消费时定位逻辑尾端，借用当前页的 slice 逐项反向移动，页面耗尽后才进入上一页。非零 nth_back 跳跃清除反游标并直接重定位；旧 nth_back 已经是 O(1)，没有把它描述为线性跳跃。remaining 统一限制两个方向和范围，防止越过逻辑端点或读取隐藏头尾单元。Pine pivot 的 nearest-NA 截断、左侧允许相等、右侧严格比较、ordinary 每调用 push 均保持原样。

没有新增持久 runtime / SharedDeque storage 字段、heap 分配、public ABI/schema、缓存规则或 host 依赖。临时迭代器布局确有变化，Windows 64-bit MSVC 实际独立布局收据：`PagedIter<Option<f64>>: 56 → 80 bytes（+24），align 8 → 8；Iter<Option<f64>>: 56 → 80 bytes（+24），align 8 → 8；SharedDeque<Option<f64>>: 48 → 48 bytes（+0），align 8 → 8；PagedStorage<Option<f64>>: 48 → 48 bytes（+0），align 8 → 8；slice::Iter<Option<f64>>: 16 → 16 bytes（+0），align 8 → 8`；该平台记录不能作为所有目标架构的布局声明。

新增 6 个独立 VecDeque 逆向模型测试、4 个 public-runtime 测试。前者覆盖 range/Take/Skip、零与大跳跃、前后混合和剩余 fold、快照端点缩减/再生长、ZST、短路恢复、signed zero/NaN payload。后者覆盖 3000 左/右窗口、v5/v6、nearest NA 屏障、左右相等差异、signed-zero 原始 Float bits、动态左右长度、同调用点每 bar 三次调用、forming 替换、失败后完整 visible/confirmed 输出与 profile/revision 不变、确认及后续 130 次提交。源码冻结 6373 项（基线 6371）；限定 1 生产修改和 2 测试新增，原结构门禁及 800 行 helper 门槛保持不变。

## 完整门禁

- windows: 7488 Rust、794 Python、166 工具测试、0 项跳过、407 production files。
- linux: 7488 Rust、794 Python、166 工具测试、1 项跳过、407 production files。

两平台按原 canonical verify 完成 fmt、warnings-denied Clippy、workspace、结构、host parity、实际 Node/WASM、新 wheel、新 venv 和安装后 Python 测试。独立只读 checker 核验实际日志、新测试名、源码 tar/当前字节和 root completion；实际通过和静态源码审查分别记录。

## 同输入 native 对照

6 个独立基线语义预检进程通过，无性能摘要。正式 36 个独立进程（6 案例 × 2 版本 × 3 次；版本顺序交替）均实际退出 0：588 份完整公开 JSON、408 份 typed sidecar、384 个测量窗口、180 次实际 Replica 应用与独立 delta 重放、240000 次 warm 更新。所有显式完整输出、typed Float bits/NA、完整 profile 和状态检查跨版本与重复均一致，没有 profile 字段豁免。pivot_small_control 和 SMA17/SWMA4 small_control 的每个原始非时间分配指标严格等于 baseline0；其他分配差异原样保留。

表中为本机 Windows 64-bit instrumented native ms 中位数，正百分比代表候选更快。forming 每进程各 3 个原始窗口先取中位数，再汇总 3 进程；历史/seed 每进程一次。四个案例各另测一个 10000 次 warm 无输出更新块，块除以 10000 仅表示平均吞吐耗时，不是单次更新分位数。JSON 保留全 6 案例的 analyze/prepared 和所有 runtime 阶段摘要、范围、三次配对方向，以及 36 进程全部原始时间与分配指标；没有筛选最快重复。

| 案例 | 历史及完整输出 | seed 无输出 | forming 无输出 | forming borrowed delta | warm 10000 次块 |
|---|---:|---:|---:|---:|---:|
| pivot_long_left | 39.0452 → 35.4384 (+9.24%) | 40.5618 → 34.1789 (+15.74%) | 0.0855 → 0.0861 (-0.70%) | 0.0846 → 0.0911 (-7.68%) | 158.0668 → 140.4861 (+11.12%) |
| pivot_large_left | 221.9378 → 217.4523 (+2.02%) | 210.3341 → 220.4833 (-4.83%) | 0.4944 → 0.4376 (+11.49%) | 0.0836 → 0.0996 (-19.14%) | — |
| pivot_long_right | 75.4948 → 67.1654 (+11.03%) | 73.7727 → 68.6676 (+6.92%) | 0.0798 → 0.0792 (+0.75%) | 0.0851 → 0.0922 (-8.34%) | — |
| pivot_edges | 34.3318 → 33.6084 (+2.11%) | 32.4131 → 35.4986 (-9.52%) | 0.1120 → 0.1337 (-19.37%) | 0.1315 → 0.1343 (-2.13%) | 178.4768 → 184.3358 (-3.28%) |
| pivot_small_control | 17.8895 → 17.9495 (-0.34%) | 18.0041 → 17.5003 (+2.80%) | 0.0931 → 0.0925 (+0.64%) | 0.1019 → 0.0936 (+8.15%) | 107.1582 → 105.9934 (+1.09%) |
| small_control | 17.5026 → 16.7087 (+4.54%) | 17.3087 → 17.8965 (-3.40%) | 0.0901 → 0.0871 (+3.33%) | 0.1012 → 0.0994 (+1.78%) | 100.4732 → 98.1060 (+2.36%) |

任一中位数或配对出现负方向的观测均列出，不能用其他阶段的收益覆盖：

- pivot_long_left/analyze: 中位数方向 -6.24%；三次配对方向 +13.76%, -7.39%, +12.88%。
- pivot_long_left/formingWithoutOutput: 中位数方向 -0.70%；三次配对方向 +22.25%, +1.52%, -7.09%。
- pivot_long_left/formingBorrowedDelta: 中位数方向 -7.68%；三次配对方向 +15.34%, -7.68%, -45.57%。
- pivot_large_left/analyze: 中位数方向 -2.69%；三次配对方向 +0.54%, -2.69%, +2.19%。
- pivot_large_left/prepared: 中位数方向 +0.00%；三次配对方向 +0.30%, -7.87%, +10.86%。
- pivot_large_left/historicalFullOutput: 中位数方向 +2.02%；三次配对方向 -6.48%, +2.02%, +5.28%。
- pivot_large_left/streamSeedWithoutOutput: 中位数方向 -4.83%；三次配对方向 +0.91%, -4.83%, -5.66%。
- pivot_large_left/formingBorrowedDelta: 中位数方向 -19.14%；三次配对方向 +1.71%, -19.14%, -24.72%。
- pivot_long_right/streamSeedWithoutOutput: 中位数方向 +6.92%；三次配对方向 +7.28%, +6.89%, -0.41%。
- pivot_long_right/formingWithoutOutput: 中位数方向 +0.75%；三次配对方向 +6.48%, -6.87%, +0.75%。
- pivot_long_right/formingBorrowedDelta: 中位数方向 -8.34%；三次配对方向 -16.79%, +17.97%, -5.76%。
- pivot_edges/analyze: 中位数方向 +3.99%；三次配对方向 +4.23%, -2.09%, +6.53%。
- pivot_edges/prepared: 中位数方向 +2.34%；三次配对方向 -7.05%, -1.11%, +4.09%。
- pivot_edges/historicalFullOutput: 中位数方向 +2.11%；三次配对方向 -6.54%, +4.28%, +2.11%。
- pivot_edges/streamSeedWithoutOutput: 中位数方向 -9.52%；三次配对方向 -0.44%, -10.17%, -9.52%。
- pivot_edges/formingWithoutOutput: 中位数方向 -19.37%；三次配对方向 -26.52%, -6.61%, -16.16%。
- pivot_edges/formingBorrowedDelta: 中位数方向 -2.13%；三次配对方向 +20.17%, -13.75%, -2.13%。
- pivot_edges/warmBlockWithoutOutput: 中位数方向 -3.28%；三次配对方向 -3.28%, -3.86%, +4.03%。
- pivot_small_control/analyze: 中位数方向 -0.24%；三次配对方向 +8.31%, -0.51%, +8.09%。
- pivot_small_control/historicalFullOutput: 中位数方向 -0.34%；三次配对方向 +3.76%, -2.03%, +6.24%。
- pivot_small_control/streamSeedWithoutOutput: 中位数方向 +2.80%；三次配对方向 -28.23%, +5.21%, +0.29%。
- pivot_small_control/formingWithoutOutput: 中位数方向 +0.64%；三次配对方向 +20.13%, +0.21%, -9.47%。
- pivot_small_control/formingBorrowedDelta: 中位数方向 +8.15%；三次配对方向 +1.37%, +12.66%, -2.86%。
- pivot_small_control/warmBlockWithoutOutput: 中位数方向 +1.09%；三次配对方向 -6.68%, +2.53%, -4.75%。
- small_control/analyze: 中位数方向 -6.97%；三次配对方向 -5.00%, +13.61%, -8.18%。
- small_control/prepared: 中位数方向 -15.45%；三次配对方向 -8.94%, -15.45%, -1.57%。
- small_control/historicalFullOutput: 中位数方向 +4.54%；三次配对方向 +6.72%, -2.52%, -0.31%。
- small_control/streamSeedWithoutOutput: 中位数方向 -3.40%；三次配对方向 -7.31%, -1.89%, -6.75%。
- small_control/formingWithoutOutput: 中位数方向 +3.33%；三次配对方向 -7.69%, +5.11%, +7.73%。
- small_control/warmBlockWithoutOutput: 中位数方向 +2.36%；三次配对方向 +3.77%, -0.06%, +3.56%。

构造、独立整数索引 VecDeque pivot oracle、序列化、profile、Replica 和 I/O 位于对应测量窗外；historical 窗包含 owned result。warm 输入预构造并保留到窗口结束，仅核验显式初始/最终快照、最终 profile/revision，未检查每次中间更新；普通 forming、confirm 和未来提交按各自显式快照核验。100000 左窗 seed100001，末 bar 唯一就绪；3000 左/3000 右窗 seed12000，窗口长 6001，受右侧平台相等规则影响，末历史 bar 才出现唯一有效 pivot；其他案例 seed6000。long_left historical 有 3000 个真实有效输出，edges 的正/负零候选计数随收据验证。分配 bytes/count、相对入口请求存活峰值及退出存活差属于 System Layout 申请指标，不能替代 RSS。

本轮登记资格失败收据：gate-name-parser-failure.json；实际失败内容和重试通过的收据分别保存，不覆盖原始失败。

## 剩余与边界

一般 TA 线性扫描、forming checkpoint/metadata 分配归因和页边界 COW、Map/Matrix 分派、递归 HIR 仍待处理，本轮未修改这些模块。有限只读后续审查定位了两处重复工作：matrix.is_stochastic 每次把全部元素转换到临时 Vec<f64>，行条件已全部满足时仍扫描列；Map put/remove 的容量或压力计费与实际修改重复哈希、定位同一键。它们目前只有源码证据，尚未测量收益。

未验证 TradingView、host/request/strategy 吞吐、跨平台性能、RSS、完整资源矩阵或发布资格。旧 runtime 编译器可执行文件没有追溯封存；本轮 probe 编译器可执行文件/argv 和封存库有收据可核验。完整 profile/public/typed 原始快照位于 `.local/pivot-reverse-20261006`，通过哈希收据关联，没有重复复制到 tracked 文档。

两份报告在运行资格后添加，tracked 预期 6375 项。完整摘要及收据：[PIVOT_REVERSE_RESULTS_20261006.json](PIVOT_REVERSE_RESULTS_20261006.json)。JSON SHA256 `9786a1fa4b900fb7c81030755c9a91748d0068335706d6759c775868386560cc`。
