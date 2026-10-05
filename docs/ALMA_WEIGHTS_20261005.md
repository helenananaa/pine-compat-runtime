# ALMA 权重缓存 — 2026-10-05

基线 `dc63842173161f3f8e40280cb9d432ce6412207a`。只修改纯运行时的 ALMA 计算缓存、realtime 纯缓存沿用和容量统计，新增 10 项回归。offset/sigma/floor 保持 simple 参数合同；公开场景只动态改变 length。每个 ALMA 调用仍普通 push，包括同调用点循环中的每次调用。

## 实现与精确性

原 ALMA 每次 ready 调用都为窗口逐项重新 exp。现在每个 resolved call-site 保留最近完整参数 key（length、offset/sigma bits、floor）；首次 ready 记 pending 并执行原公式，第二次同 key ready 建立 immutable 权重，随后复用。未就绪的新调用点不建立 map 或按 length 预分配；未就绪且 key 改变时清掉旧项，同 key 的源 NA 可以保留权重。连续变化参数不会累积历史 kernel。缓存 map 延迟分配，checkpoint 共享 Arc；命中不 COW。

保持原 center/floor/scale/exp 表达式和每条累加链的 +0 初值及 oldest→newest 顺序。不预归一化、不 FMA、不重组公式。分母按原逐项顺序预计算，缓存只保存纯系数；输出依然来自当前 Pine 窗口。forming 沿用前次成功分支的纯缓存，窗口照常从 confirmed 回滚；miss 的 metadata COW 保留失败前分支。without-output 和 borrowed delta 共用该执行路径。

10 新测试包含 6 组独立 VecDeque/每次 direct-exp 的 exact Float bits 或 NA-tag 回归，以及 4 组 cache 资源/共享/失效测试。覆盖极端有限参数、±0、动态长度、循环每调用消费、NA/恢复、巨大未就绪长度、realtime 冷到热/失败重试/Replica/replay、request chart-context 隔离和旧 snapshot。定向 12/12 通过；当前源码 6366 项，生产 Rust 407 项，原结构门禁及 800 行 helper 门槛不变。新增字段改变私有 runtime 结构，本轮没有布局测量或 ABI 尺寸承诺。

## 门禁及正式对照

Windows/Linux 完整门禁各通过 7472 Rust、794 Python、166 工具测试；Linux 另有 1 skip，Windows 0 skip。包括 fmt、warnings-denied Clippy、结构/host parity、实际 Node/WASM、新 wheel、新 venv 和安装后的 Python 测试。源冻结及两平台原日志由只读 checker 独立核验。

封存 clean 基线 6363 项，其中 6361 项逐字节匹配上一轮实际编译清单，另外 2 项是上轮报告；311 份 Rust 产物和直接链接库身份校验通过。8 个基线预检进程通过，不计入性能摘要。正式 48 个独立进程（每版每案例 3 次）全部实际退出 0：1584 份完整公开 JSON、1056 份 typed Float bits、1056 测量窗口及完整 profile、528 次实际 Replica 应用与独立 delta 重放通过。

完整公开数据/typed bits、revision 和 oracle 在跨版及所有重复中一致；同版完整 profile 一致。跨版只允许 rollingWindowValueCapacity 按独立 Python pending/active/rollback 模型精确增加当前 coefficient Float slots，其他 profile 字段逐项一致。容量统计没有隐藏新权重；Box 切片长度 L 对应 8L B coefficient payload，另有 metadata/Arc 成本。当前根之外的旧 checkpoint 仍可持有旧权重，这不是全进程内存上限。

以下为仪表化 native 中位数（ms），正百分比表示候选更快。forming 每进程 9 次取中位数，再取 3 个进程中位数；历史/seed 每进程一次。两个 forming 阶段各 9 个原始窗口（每进程共 22 个测量窗口）、进程范围、慢项和申请变化保留在 JSON。

| 案例 | 历史追加及完整输出 | seed 无输出 | forming 无输出 | forming borrowed delta |
|---|---:|---:|---:|---:|
| alma_fixed_long | 62.4822 → 16.5398 (+73.53%) | 60.2578 → 16.4196 (+72.75%) | 0.0667 → 0.0603 (+9.60%) | 0.0742 → 0.0655 (+11.73%) |
| alma_large | 178.8603 → 174.1349 (+2.64%) | 179.0700 → 175.8548 (+1.80%) | 0.6160 → 0.2538 (+58.80%) | 0.6363 → 0.2777 (+56.36%) |
| alma_dynamic_length | 30.1674 → 17.1978 (+42.99%) | 30.6572 → 17.2265 (+43.81%) | 0.0695 → 0.0686 (+1.29%) | 0.0779 → 0.0736 (+5.52%) |
| alma_constant_parameters | 34.0362 → 22.7302 (+33.22%) | 32.9746 → 21.7624 (+34.00%) | 0.0794 → 0.0738 (+7.05%) | 0.0735 → 0.0727 (+1.09%) |
| alma_same_callsite_loop | 28.5836 → 22.3213 (+21.91%) | 28.7553 → 22.6250 (+21.32%) | 0.0564 → 0.0615 (-9.04%) | 0.0599 → 0.0573 (+4.34%) |
| alma_small | 10.6285 → 10.5988 (+0.28%) | 11.0666 → 10.5757 (+4.44%) | 0.0516 → 0.0530 (-2.71%) | 0.0536 → 0.0552 (-2.99%) |
| small_control | 13.9426 → 13.7950 (+1.06%) | 14.0018 → 13.5138 (+3.49%) | 0.0601 → 0.0610 (-1.50%) | 0.0652 → 0.0646 (+0.92%) |
| zero_control | 8.7385 → 8.3449 (+4.50%) | 9.0222 → 8.5724 (+4.99%) | 0.0359 → 0.0371 (-3.34%) | 0.0464 → 0.0466 (-0.43%) |

两个无 ALMA 控制组的每个 raw 非耗时申请指标与基线精确相同。ALMA 新增申请全部保留；冷构建在实际执行窗内，没有从耗时或资源指标里扣除。以下示例是 repeat0 的原始 forming 无输出窗口，不能替代完整样本：

- ALMA100k 第 0 次（候选首次建权重）：申请 9220 → 809536 B，次数 43 → 47，相对入口请求存活峰值 8180 → 808496 B。
- ALMA100k 第 1 次（候选已缓存）：申请 9220 → 9220 B，次数 43 → 43，相对入口请求存活峰值 8180 → 8180 B。

ALMA100k seed100k，历史仅末个 bar ready，所以历史没有形成 active 权重缓存；首次 forming 冷建，后续沿用。动态长度 seed7000，固定长窗/其他场景 seed6000。性能仅适用于这些固定输入；未把控制项变化归为噪声，也未把 indicator 改善外推到通用解释器吞吐。请求 allocator 指标是成功 System Layout 的 bytes/count、相对入口存活峰值及退出存活差值，不能替代 RSS。构造、独立 oracle、序列化、Replica、profile 和 I/O 均在测量窗外；公开采样在门禁、构建和链接完成后串行执行。

## 小窗与循环连续更新补测

正式短窗测量中 ALMA64 循环的 forming 无输出中位数慢 9.04%，ALMA17 的两个 forming 阶段慢 2.71%/2.99%。因此额外使用同一封存库，串行跑 18 个新进程（3 案例 × 2 版本 × 3 次，版本顺序交替）。每个进程 seed6000 并在窗外成功 warmup1次，预先构建 10000 个同时间戳 forming 输入并保持 Vec 存活；单测量块只执行 10000 次公开 update_without_output 及成功检查。报告块中位数和范围，块 ms 除以 10000 是平均吞吐耗时，不是单次更新分位数。

| 案例 | 基线块中位数 ms / 3 进程范围 | 候选块中位数 ms / 3 进程范围 | 候选更快 |
|---|---:|---:|---:|
| alma_same_callsite_loop | 102.0714 / [102.0433, 104.2734] | 91.9461 / [90.4142, 93.0374] | +9.92% |
| alma_small | 66.8174 / [66.2646, 67.1416] | 67.8027 / [67.1433, 67.8752] | -1.47% |
| small_control | 98.7852 / [96.5680, 99.0018] | 95.9954 / [95.2831, 99.2151] | +2.82% |

18 个进程实际退出 0，共 180000 次计时更新、18 个测量块、54 份完整公开快照和 54 份 typed sidecar。这里只核对 seed/warm/final 的独立 direct-exp VecDeque oracle、跨版/重复完整输出与 typed bits、完整/confirmed profile、revision 和无输出后置条件，未逐项验证 10000 个中间快照，也未跑 Replica；正式 48 进程矩阵保持独立且逐字节未变。补测独立 checker 核验来源、原始结果、非时间控制指标及所有摘要计算。small_control 每个 raw 非时间申请指标严格等于基线；ALMA 的申请指标全部保留。两个批次的方向和差值按各自测量范围保留，不用补测覆盖原短窗慢项；这些观测没有建立跨环境稳定的百分比承诺。

本次补测循环案例三组配对均更快；ALMA17 三组配对均更慢（约 1.33%、1.47%、1.09%），块中位数增加约 0.9853 ms，即每次约 0.10 微秒。没有将这一退步解释成噪声，小窗缓存/查表开销仍需进一步归因。本轮接受长窗收益与这一已记录的小窗代价，不宣称所有 ALMA 场景更快。

辅助探针准备曾因 capacity-key 生成断言退出 1（01bc65），在全部目标写入前中止，未启动编译、链接或探针；原说明保留。随后只修生成逻辑，静态检查通过。没有把这次准备失败计为运行时或资格结果。

## 剩余

小窗 ALMA17 的缓存/查表代价、普通分页游标、一般 TA 线性扫描、页边界共享目录/端页 COW、range 重复极值、Map/Matrix 分派及递归 HIR 仍待归因处理。range 当前分别调用 highest/lowest，存在可合并的重复扫描；该源码发现尚无性能收益测量。未验证 TradingView 参考、RSS、host/strategy/request 吞吐、跨平台性能、完整资源矩阵或发布资格；旧 runtime 编译器可执行文件未追溯封存。原始 ignored 产物保留于 .local/alma-weights-20261005，报告记录哈希，不打包全部二进制/公开快照。

本轮两份报告在资格验证后添加，tracked 预期 6368 项。完整收据：[ALMA_WEIGHTS_RESULTS_20261005.json](ALMA_WEIGHTS_RESULTS_20261005.json)。JSON SHA256 `882a57ea66088baec7c19bed34cf069a8f701c99b01ac324cc79bc80d5318247`。
