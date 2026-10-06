# Range / ALMA 重复工作消除 — 2026-10-06

基线 `2e2bd4c00dae38c39fe024cc1139a34f65fe4742`。range 从两遍扫描变为一遍顺序 fold，按首个有效样本初始化各自 max/min 累加链，最后相减；没有把溢出 Float infinity 改为 NA。ALMA 以完整 bit-key 和单次借用回调消费权重，热态缓存只查一次；共用普通窗口 push helper 并保留窗口引用，消除 relookup。cold direct-exp 才计算 center/floor；scale 验证、每次调用 push、缓存 pending/active/COW、权重公式与累加顺序保持原样。没有新增 runtime 字段、长度阈值、缓存系数或 host 依赖。

新增 4 个 range 精确回归和 2 个 ALMA callback/owner 回归。range 独立 VecDeque 原两遍 oracle 覆盖 30941 原样本组合、signed zero、NaN/Infinity 原始样本、readiness/NA、有限相减溢出、分页头、快照分支、动态长度和同 bar 更新。原 ALMA 数值、动态长度、循环、回滚失败重试和 request-context 测试继续运行。源冻结 6369 项（基线 6368），限定 5 修改/1 新增；原结构及 800 行 helper 门槛不变。

## 完整门禁

- windows: 7478 Rust、794 Python、166 工具测试。
- linux: 7478 Rust、794 Python、166 工具测试（其中 1 项跳过）。

两平台运行原 canonical verify：fmt、warnings-denied Clippy、workspace、结构、host parity、实际 Node/WASM、新 wheel、新 venv、安装后 Python 测试。实际日志、冻结源码 tar、新增测试名和 root completion 由只读 checker 核验。

## 同输入 native 对照

6 个基线语义预检独立进程通过，无性能摘要；正式 36 个独立进程（6 案例 × 2 版本 × 3 次，版本顺序交替）全部实际退出 0。588 份完整公开 JSON、408 份 typed sidecar、384 个测量窗口、180 次实际 Replica 应用和独立 delta 重放通过。所有显式完整输出、typed Float bits/NA、完整 profile 与状态检查跨版及同版重复逐项相等，没有 profile 字段豁免。Float Infinity 在公开 JSON 投影为 null，typed 仍保留 Infinity bit/tag，不能把公开 null 当成 NA。

下表是 instrumented native ms 中位数，正百分比表示候选更快。forming 每进程各 3 个原始样本取中位数，再取 3 进程中位数；历史/seed 每进程一次。四个小案例另各测一个 warm 10000 次无输出更新块，输入预构造、调用者 Vec 保留到计时结束。块除以 10000 是平均吞吐耗时，不是单次更新分位数。所有原始样本、范围、配对方向、慢项和非时间申请指标都保留。

| 案例 | 历史及完整输出 | seed 无输出 | forming 无输出 | forming borrowed delta | warm 10000 次块 |
|---|---:|---:|---:|---:|---:|
| range_fixed_long | 62.2374 → 38.1374 (+38.72%) | 62.3108 → 39.3719 (+36.81%) | 0.0855 → 0.0794 (+7.13%) | 0.1005 → 0.0911 (+9.35%) | — |
| range_large | 223.6177 → 221.0118 (+1.17%) | 219.8973 → 226.1606 (-2.85%) | 0.7698 → 0.5510 (+28.42%) | 0.8082 → 0.5307 (+34.34%) | — |
| range_edges | 32.8439 → 29.9850 (+8.70%) | 31.4842 → 31.5427 (-0.19%) | 0.0998 → 0.1080 (-8.22%) | 0.1116 → 0.1088 (+2.51%) | 142.8120 → 141.8633 (+0.66%) |
| alma_same_callsite_loop | 28.6751 → 28.3511 (+1.13%) | 28.6674 → 27.7877 (+3.07%) | 0.0838 → 0.0855 (-2.03%) | 0.0946 → 0.0944 (+0.21%) | 94.7729 → 95.9260 (-1.22%) |
| alma_small | 13.4745 → 12.3762 (+8.15%) | 13.8225 → 13.5060 (+2.29%) | 0.0833 → 0.0704 (+15.49%) | 0.0945 → 0.0924 (+2.22%) | 72.1338 → 70.5670 (+2.17%) |
| small_control | 17.0071 → 17.4623 (-2.68%) | 17.2382 → 17.8384 (-3.48%) | 0.0861 → 0.1044 (-21.25%) | 0.1069 → 0.1073 (-0.37%) | 103.5518 → 100.0257 (+3.41%) |

构造、独立原两遍 range/direct-exp ALMA oracle、序列化、profile、Replica 和 I/O 在相应测量窗外；historical 窗口包含 owned result()。warm 共 240000 次更新，只验证初始/最终快照与最终 profile/revision，未逐项检查中间更新；普通 forming、确认及未来提交按各自显式快照检查。range100k seed100k，只有末个历史 bar 就绪；range3000/其他案例 seed6000。无 ALMA 控制组的每个原始非时间申请指标严格相等；其他案例的申请差异原样保留。申请 bytes/count、相对入口请求存活峰值和退出存活差是 System Layout 指标，不能替代 RSS。

本次基线是上一轮已缓存的 ALMA 实现。上一轮 ALMA17 warm 块曾慢约 1.5%，本轮不把不同批次百分比相减来承诺已恢复到更早的无缓存版本。只对本表的同输入基线/候选作结论，不外推通用解释器 CPU 吞吐。

本表同时保留退步：ALMA 同调用点循环的 warm 块慢 1.22%，range_edges forming 无输出慢 8.22%，small_control forming 无输出慢 21.25%。不能用其他阶段的收益覆盖这些观测。

只读核对第一次因编排顺序失败：native auditor 与 gate auditor 并行，先读取了尚未生成的 gate-receipts.json。实际 exit 1 与 FileNotFoundError 已保存在 audit-order-failure.json；门禁完成后按顺序运行原 native auditor。源码、probe helpers 与正式 36 进程样本均未因这次编排修正改变。

## 剩余与边界

分页逆向逐项定位、一般 TA 线性扫描、forming checkpoint/metadata 申请归因及页边界 COW、Map/Matrix 分派与递归 HIR 仍待处理。已确认 pivots.rs 的 left_ok 使用 Take.rev：pivothigh(close,3000,0) 在递增输入下每次就绪调用会完整逆扫 3000 项，paged next_back 每项重新 locate/查页面；这是源码重复工作证据，尚无速度测量。没有验证 TradingView、host/request/strategy 吞吐、跨平台性能、RSS、完整资源矩阵或发布资格；旧 runtime 编译器可执行文件未追溯封存。本轮 probe 编译器可执行文件/argv 和已封存库可核验。原始 ignored 数据位于 .local/range-alma-20261006；完整快照/profile 以收据哈希关联，避免重复塞入 tracked 文档。

两份报告在运行资格后添加，tracked 预期 6371 项。完整摘要及收据：[RANGE_ALMA_RESULTS_20261006.json](RANGE_ALMA_RESULTS_20261006.json)。JSON SHA256 `fe095e2f372c29c519c777e3df5c4b070bf01e7567f791b58633c0c533cf2875`。
