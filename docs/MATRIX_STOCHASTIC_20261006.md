# Matrix stochastic 借用遍历 — 2026-10-06

基线 `11cab85f4615fcc447ffde64006a5ac0bf343d04`。matrix.is_stochastic 原先每次分配临时 Vec<f64>，转换全部单元，然后同时计算行与列条件。第三版保留第二版的数值/finite/非负验证与按原顺序的行、第一列求和融合，使用原 f64 Sum 的负零初值；全部单元合法后，行条件满足即返回 true，行条件失败且第一列失败则返回 false。单列已满足列条件时返回 true；单行的剩余列直接比较每个合法单元是否等于 1.0。其余情况再次按 row-major 遍历单元，只为第一列以外的列分配 f64 scratch，保持各列原先的 row 顺序，再检查所有剩余列和。新 ArrayValues 私有 slices iterator 借用连续页，Matrix 通过 flatten 在页内直接走 slice iterator，避免每个单元重新查找页面；既有 ArrayIter 和其他消费者保持不变。Int 转 f64、Float 原始值和精确相等条件保持原样，不增加 epsilon、缓存、任意窗口阈值或重新结合求和。空矩阵仍返回 false。

没有新增持久 runtime/storage 字段、public ABI/schema、Pine 矩阵规则、host 依赖或缓存策略。行成功、非法输入、第一列失败和单轴分支没有 numeric scratch；需要剩余列时只分配 (columns−1)×8 bytes，64 列为 504 bytes，原 65×64 转换缓冲为 33280 bytes。每列累加从负零开始，单元行顺序保持原样。这些分支由原 OR/columns-all 条件直接推导，不使用任意 cutoff。第一列成功后的剩余列累加，可能在某个较早剩余列最终失败时仍已计算后面的列；原 column all 可在该列失败后跳过后面的列。这些额外纯读取/加法不改变返回值，但需要实际性能验证。实现不再重复随机索引 paged columns；实际请求分配与列方向耗时仍以下面的正式收据为准。

新增 4 个独立旧 copy-buffer oracle 私有测试、4 个 public API 测试。私有覆盖两种矩形方向、Int/Float、全部不可转换 PineValue kinds、negative/nonfinite/NA、空维度、正负零、对求和顺序敏感的舍入、127/128/129 payload 边界及快照隔离；borrowed slices 另外核验原 variant、递归 Float bits、原引用 pointer、精确 logical length 和重复 exhaustion，以及 paged 257→129→128→127→1→0 缩短、regrowth 与 snapshot 保持。公开测试覆盖 v5/v6、typed Float plot bits、340 cells 的跨页 fill/set/copy/var history、同调用点四次 predicate 与参数 mutation、forming 替换、predicate 之后 runtime.error、完整 visible/confirmed 输出、profile/revision/last_changes 的失败原子回滚、确认和未来提交。源码冻结 6377 项（基线 6375）；限定 2 生产修改和 2 测试新增，原结构门禁及 800 行 helper 门槛保持不变。

## 完整门禁

- windows: 7496 Rust、794 Python、166 工具测试、0 项跳过、407 production files。
- linux: 7496 Rust、794 Python、166 工具测试、1 项跳过、407 production files。

两平台按原 canonical verify 完成 fmt、warnings-denied Clippy、workspace、结构、host parity、实际 Node/WASM、新 wheel、新 venv 和安装后 Python 测试。独立只读 checker 核验实际日志、新测试名、源码 tar/当前字节和 root completion；实际通过和静态源码审查分别记录。implementation-review.json 保存两个 actor 的有限只读源码审查与四个最终 source hash，该 PASS 表示静态审查，不充当执行资格。

## 同输入 native 对照

6 个独立基线语义预检进程通过，无性能摘要。正式 36 个独立进程（6 案例 × 2 版本 × 3 次；版本顺序交替）均实际退出 0：588 份完整公开 JSON、408 份 typed sidecar、384 个测量窗口、180 次实际 Replica 应用与独立 delta 重放、240000 次 warm 更新。所有显式完整输出、typed Float bits/NA、完整 profile 和状态检查跨版本与重复均一致，没有 profile 字段豁免。matrix_identity_control64 与 SMA17/SWMA4 small_control 每个原始非时间分配指标严格等于 baseline0；独立审计核验正式 126 个、预检 21 个控制原始窗口，identity 控制没有 warm 窗。其他分配差异原样保留。

全部案例 seed6000。row64 为 64×65，各行 64 个 1/64 后接零，行和为 1、列条件不满足；col64 为 65×64，首行零，其余 64 个 1/64，行条件不满足、列和为 1。invalid64 为 64×64 对角矩阵，每 bar 修改最后一格为 NA、负数或不满足随机矩阵条件的有限值。small 为 2×3 与 3×2 动态矩阵；identity64 调用未修改的 matrix.is_identity，SMA17/SWMA4 是另一控制。

表中为本机 Windows 64-bit instrumented native ms 中位数，正百分比代表候选更快。forming 每进程各 3 个原始窗口先取中位数，再汇总 3 进程；历史/seed 每进程一次。row64、col64、small 和 SMA 各另测一个 10000 次 warm 无输出更新块，块除以 10000 仅表示平均吞吐耗时，不是单次更新分位数。JSON 保留全 6 案例的 analyze/prepared 和所有 runtime 阶段摘要、范围、三次配对方向，以及 36 进程全部原始时间与分配指标；没有筛选最快重复。

| 案例 | 历史及完整输出 | seed 无输出 | forming 无输出 | forming borrowed delta | warm 10000 次块 |
|---|---:|---:|---:|---:|---:|
| matrix_row64 | 95.0818 → 69.8700 (+26.52%) | 93.9728 → 69.6683 (+25.86%) | 0.1036 → 0.0928 (+10.42%) | 0.1054 → 0.1104 (-4.74%) | 190.9056 → 151.2023 (+20.80%) |
| matrix_col64 | 92.7949 → 89.0795 (+4.00%) | 89.1229 → 86.7744 (+2.64%) | 0.1030 → 0.0882 (+14.37%) | 0.1097 → 0.1104 (-0.64%) | 184.0248 → 181.5181 (+1.36%) |
| matrix_invalid64 | 94.2867 → 85.9127 (+8.88%) | 98.1558 → 82.8967 (+15.55%) | 0.1294 → 0.1159 (+10.43%) | 0.1381 → 0.1343 (+2.75%) | — |
| matrix_small | 38.5696 → 39.4387 (-2.25%) | 39.2031 → 38.7585 (+1.13%) | 0.1364 → 0.1283 (+5.94%) | 0.1304 → 0.1433 (-9.89%) | 196.1638 → 193.1786 (+1.52%) |
| matrix_identity_control64 | 60.1588 → 60.2599 (-0.17%) | 59.8374 → 59.1914 (+1.08%) | 0.0895 → 0.0877 (+2.01%) | 0.0991 → 0.1017 (-2.62%) | — |
| small_control | 15.6400 → 16.0744 (-2.78%) | 16.7514 → 17.1424 (-2.33%) | 0.0863 → 0.0924 (-7.07%) | 0.0929 → 0.0967 (-4.09%) | 92.6372 → 95.3044 (-2.88%) |

任一中位数或配对出现负方向的观测均列出，不能用其他阶段的收益覆盖：

- matrix_row64/prepared: 中位数方向 +7.01%；三次配对方向 -1.85%, +7.62%, +4.09%。
- matrix_row64/formingBorrowedDelta: 中位数方向 -4.74%；三次配对方向 -19.44%, +4.93%, +19.30%。
- matrix_col64/analyze: 中位数方向 +2.72%；三次配对方向 -6.38%, -2.25%, +2.72%。
- matrix_col64/prepared: 中位数方向 -18.86%；三次配对方向 +0.84%, -11.62%, -25.59%。
- matrix_col64/historicalFullOutput: 中位数方向 +4.00%；三次配对方向 -3.31%, -0.05%, +4.00%。
- matrix_col64/formingBorrowedDelta: 中位数方向 -0.64%；三次配对方向 -1.81%, -0.64%, +3.00%。
- matrix_col64/warmBlockWithoutOutput: 中位数方向 +1.36%；三次配对方向 +3.52%, +1.83%, -1.49%。
- matrix_invalid64/analyze: 中位数方向 +0.40%；三次配对方向 -12.66%, +3.93%, +0.40%。
- matrix_invalid64/prepared: 中位数方向 -7.60%；三次配对方向 -16.07%, -20.00%, +2.74%。
- matrix_invalid64/formingBorrowedDelta: 中位数方向 +2.75%；三次配对方向 +0.89%, +4.56%, -0.14%。
- matrix_small/analyze: 中位数方向 -19.33%；三次配对方向 -23.22%, -5.43%, -9.16%。
- matrix_small/prepared: 中位数方向 -13.28%；三次配对方向 -22.88%, -14.98%, +19.77%。
- matrix_small/historicalFullOutput: 中位数方向 -2.25%；三次配对方向 -0.24%, -3.86%, -3.47%。
- matrix_small/streamSeedWithoutOutput: 中位数方向 +1.13%；三次配对方向 +2.46%, -2.53%, +1.13%。
- matrix_small/formingWithoutOutput: 中位数方向 +5.94%；三次配对方向 +8.06%, +6.76%, -9.25%。
- matrix_small/formingBorrowedDelta: 中位数方向 -9.89%；三次配对方向 -15.59%, -3.23%, -9.89%。
- matrix_small/warmBlockWithoutOutput: 中位数方向 +1.52%；三次配对方向 +3.35%, -0.09%, +2.46%。
- matrix_identity_control64/analyze: 中位数方向 -10.26%；三次配对方向 +4.07%, -16.25%, -16.80%。
- matrix_identity_control64/prepared: 中位数方向 -8.28%；三次配对方向 +1.02%, +0.29%, -8.28%。
- matrix_identity_control64/historicalFullOutput: 中位数方向 -0.17%；三次配对方向 +2.63%, -9.82%, +0.43%。
- matrix_identity_control64/streamSeedWithoutOutput: 中位数方向 +1.08%；三次配对方向 +2.49%, -3.42%, +1.41%。
- matrix_identity_control64/formingWithoutOutput: 中位数方向 +2.01%；三次配对方向 +8.38%, +2.45%, -10.84%。
- matrix_identity_control64/formingBorrowedDelta: 中位数方向 -2.62%；三次配对方向 +5.90%, -38.38%, -2.62%。
- small_control/analyze: 中位数方向 -18.56%；三次配对方向 -8.40%, -10.98%, -18.69%。
- small_control/prepared: 中位数方向 -18.79%；三次配对方向 -28.40%, -25.53%, -7.05%。
- small_control/historicalFullOutput: 中位数方向 -2.78%；三次配对方向 -0.79%, +3.95%, -13.93%。
- small_control/streamSeedWithoutOutput: 中位数方向 -2.33%；三次配对方向 +2.07%, +6.25%, -6.45%。
- small_control/formingWithoutOutput: 中位数方向 -7.07%；三次配对方向 -7.76%, -1.99%, -8.81%。
- small_control/formingBorrowedDelta: 中位数方向 -4.09%；三次配对方向 +7.11%, -0.75%, -22.76%。
- small_control/warmBlockWithoutOutput: 中位数方向 -2.88%；三次配对方向 -4.59%, -1.75%, -2.04%。

各 runtime 阶段原始请求分配指标的进程中位数如下，变化由实际收据读取；列案例的收益/负方向与表中耗时一并保留：

| 案例/阶段 | allocated bytes | allocations | peak additional live bytes | live delta bytes |
|---|---:|---:|---:|---:|
| matrix_row64/historicalFullOutput | 2.00728e+08 → 1.0475e+06 | 30494 → 24494 | 535732 → 535732 | 535732 → 535732 |
| matrix_row64/streamSeedWithoutOutput | 2.00631e+08 → 951012 | 30494 → 24494 | 439184 → 439184 | 439184 → 439184 |
| matrix_row64/formingWithoutOutput | 36284 → 3004 | 32 → 31 | 34804 → 2988 | 0 → 0 |
| matrix_row64/formingBorrowedDelta | 39992 → 6712 | 39 → 38 | 34804 → 3664 | 0 → 0 |
| matrix_row64/warmBlockWithoutOutput | 3.6284e+08 → 3.004e+07 | 320000 → 310000 | 34804 → 2988 | 0 → 0 |
| matrix_col64/historicalFullOutput | 2.00728e+08 → 4.0715e+06 | 30494 → 30494 | 535732 → 535732 | 535732 → 535732 |
| matrix_col64/streamSeedWithoutOutput | 2.00631e+08 → 3.97501e+06 | 30494 → 30494 | 439184 → 439184 | 439184 → 439184 |
| matrix_col64/formingWithoutOutput | 36284 → 3508 | 32 → 32 | 34804 → 2988 | 0 → 0 |
| matrix_col64/formingBorrowedDelta | 39992 → 7216 | 39 → 39 | 34804 → 3664 | 0 → 0 |
| matrix_col64/warmBlockWithoutOutput | 3.6284e+08 → 3.508e+07 | 320000 → 320000 | 34804 → 2988 | 0 → 0 |
| matrix_invalid64/historicalFullOutput | 1.98811e+08 → 2.95937e+06 | 55259 → 50759 | 1.12272e+06 → 1.12272e+06 | 1.12272e+06 → 1.12272e+06 |
| matrix_invalid64/streamSeedWithoutOutput | 1.9833e+08 → 2.47833e+06 | 55253 → 50753 | 641628 → 641628 | 641628 → 641628 |
| matrix_invalid64/formingWithoutOutput | 55844 → 23076 | 88 → 87 | 42172 → 19400 | 0 → 0 |
| matrix_invalid64/formingBorrowedDelta | 59676 → 26908 | 101 → 100 | 42172 → 19400 | 0 → 0 |
| matrix_small/historicalFullOutput | 5.48643e+06 → 4.95843e+06 | 110793 → 104793 | 2.5081e+06 → 2.5081e+06 | 2.5081e+06 → 2.5081e+06 |
| matrix_small/streamSeedWithoutOutput | 4.2363e+06 → 3.7083e+06 | 110775 → 104775 | 1.25791e+06 → 1.25791e+06 | 1.25791e+06 → 1.25791e+06 |
| matrix_small/formingWithoutOutput | 54332 → 54244 | 138 → 137 | 39656 → 39656 | 0 → 0 |
| matrix_small/formingBorrowedDelta | 58412 → 58324 | 163 → 162 | 39656 → 39656 | 0 → 0 |
| matrix_small/warmBlockWithoutOutput | 5.4332e+08 → 5.4244e+08 | 1.38e+06 → 1.37e+06 | 39656 → 39656 | 0 → 0 |
| matrix_identity_control64/historicalFullOutput | 1.04336e+06 → 1.04336e+06 | 24492 → 24492 | 533636 → 533636 | 533636 → 533636 |
| matrix_identity_control64/streamSeedWithoutOutput | 946868 → 946868 | 24492 → 24492 | 437088 → 437088 | 437088 → 437088 |
| matrix_identity_control64/formingWithoutOutput | 3004 → 3004 | 31 → 31 | 2988 → 2988 | 0 → 0 |
| matrix_identity_control64/formingBorrowedDelta | 6712 → 6712 | 38 → 38 | 3664 → 3664 | 0 → 0 |
| small_control/historicalFullOutput | 2.29686e+06 → 2.29686e+06 | 49174 → 49174 | 1.18244e+06 → 1.18244e+06 | 1.18244e+06 → 1.18244e+06 |
| small_control/streamSeedWithoutOutput | 1.81582e+06 → 1.81582e+06 | 49168 → 49168 | 701340 → 701340 | 701340 → 701340 |
| small_control/formingWithoutOutput | 26908 → 26908 | 60 → 60 | 19648 → 19648 | 0 → 0 |
| small_control/formingBorrowedDelta | 30740 → 30740 | 73 → 73 | 19648 → 19648 | 0 → 0 |
| small_control/warmBlockWithoutOutput | 2.6908e+08 → 2.6908e+08 | 600000 → 600000 | 19648 → 19648 | 0 → 0 |

matrix var 构造和首 bar 初始化包含在 history/seed 窗内，warm 内不重复初始化。Session 构造、独立普通 Vec 单元旧 predicate oracle、序列化、profile、Replica 和 I/O 位于对应测量窗外；historical 窗包含 owned result。warm 输入预构造并保留到窗口结束，仅核验显式初始/最终快照、最终 profile/revision，未检查每次中间更新；普通 forming、confirm 和未来提交按各自显式快照核验。allocated bytes/count、相对入口请求存活峰值及退出存活差属于 System Layout 申请指标，不能替代 RSS。

## 本轮资格失败与修正

- targeted-fixture-failure.json: `targetedMatrixInitial` root tool actual exit 1 / chunk `ae46b2`，原始 child runner exit 101。原因：New fixture used series int column=bar_index%17; matrix.set/get requires simple int and semantic analysis rejected it with E_CALL_ARG_TYPE. 修正：Use fixed simple column 16 in the Pine fixture and independent flat model; retain 340 cells, cross-page mutation/copy/history checks. Production semantics remain unchanged. 原始日志、runner exit 与修改前后 test source hash 独立保留。

## 成功执行但拒绝的第一版

第一版保留完整验证，再读取 row view；不满足行条件时继续随机索引列。它通过 7496 Rust 与两平台完整门禁、36 个正式 native 进程、全部公开/typed/profile 精确对照和独立审计，但 warm 行与列路径三次配对全部退化，因此没有提交该实现。此处属于性能资格不满足，不归类为实际 tool failure；第一版收据、源码 archive、实际库 capture 和完整摘要通过 optimization-attempts.json 及独立 hash receipts 保留。

第一版独立审计最初发生实际 helper failure：root tool exit 1 / chunk `079493`，CRLF 解码文本未 normalize 导致 FAILED$ regex 不匹配。只对解码后的文本 normalize 后审计通过，原始日志字节、child exit 101、初版 auditor source 和失败收据保留。该 helper 失败与前述运行通过但性能退化分别记录。

- matrix_row64/analyze: +0.27%；三次配对 -1.26%, +5.48%, -7.93%。
- matrix_row64/prepared: -4.85%；三次配对 -6.21%, +12.30%, -18.68%。
- matrix_row64/historicalFullOutput: -9.08%；三次配对 -9.82%, -9.58%, -9.08%。
- matrix_row64/streamSeedWithoutOutput: -23.20%；三次配对 -23.20%, -23.85%, -18.46%。
- matrix_row64/formingWithoutOutput: +15.21%；三次配对 +18.56%, +15.21%, -31.71%。
- matrix_row64/formingBorrowedDelta: +16.72%；三次配对 +13.75%, -5.14%, +17.58%。
- matrix_row64/warmBlockWithoutOutput: -10.99%；三次配对 -6.83%, -14.18%, -8.81%。
- matrix_col64/analyze: -8.44%；三次配对 -20.30%, -21.31%, +6.34%。
- matrix_col64/prepared: +1.33%；三次配对 -1.66%, +1.98%, +9.00%。
- matrix_col64/historicalFullOutput: -14.32%；三次配对 -17.06%, -6.80%, -16.00%。
- matrix_col64/streamSeedWithoutOutput: -13.65%；三次配对 -20.88%, -13.57%, -8.11%。
- matrix_col64/formingWithoutOutput: -0.38%；三次配对 -47.09%, +9.47%, +6.39%。
- matrix_col64/warmBlockWithoutOutput: -12.14%；三次配对 -14.49%, -9.01%, -8.06%。
- matrix_invalid64/analyze: +1.43%；三次配对 -11.15%, +5.75%, -4.87%。
- matrix_invalid64/prepared: +7.38%；三次配对 -131.06%, +12.50%, +12.62%。
- matrix_invalid64/historicalFullOutput: -4.29%；三次配对 -8.61%, -5.99%, -0.60%。
- matrix_invalid64/formingWithoutOutput: -1.44%；三次配对 -6.98%, +6.63%, +38.88%。
- matrix_invalid64/formingBorrowedDelta: +7.52%；三次配对 -3.81%, +9.31%, +25.36%。
- matrix_small/analyze: -3.13%；三次配对 -3.13%, -19.89%, +14.36%。
- matrix_small/prepared: +7.36%；三次配对 +7.36%, -15.63%, +11.35%。
- matrix_small/historicalFullOutput: +0.96%；三次配对 +3.56%, -7.29%, +3.92%。
- matrix_small/streamSeedWithoutOutput: -3.74%；三次配对 -4.46%, -3.74%, -2.70%。
- matrix_small/formingWithoutOutput: -14.87%；三次配对 +4.47%, -16.65%, -37.16%。
- matrix_small/formingBorrowedDelta: +2.00%；三次配对 +13.02%, -0.59%, -6.99%。
- matrix_small/warmBlockWithoutOutput: +1.87%；三次配对 +5.46%, -0.08%, +2.82%。
- matrix_identity_control64/analyze: -3.14%；三次配对 +36.75%, +7.42%, -9.82%。
- matrix_identity_control64/historicalFullOutput: +3.39%；三次配对 +3.39%, -0.31%, +1.12%。
- matrix_identity_control64/streamSeedWithoutOutput: -4.16%；三次配对 -5.16%, +1.34%, -3.15%。
- matrix_identity_control64/formingWithoutOutput: +5.69%；三次配对 +13.70%, +5.69%, -9.19%。
- matrix_identity_control64/formingBorrowedDelta: -7.76%；三次配对 -6.81%, +3.96%, -21.47%。
- small_control/analyze: +8.74%；三次配对 +29.09%, -1.38%, +0.56%。
- small_control/prepared: -15.61%；三次配对 +3.33%, -15.61%, -4.98%。
- small_control/historicalFullOutput: -2.59%；三次配对 +12.57%, -2.59%, -9.41%。
- small_control/streamSeedWithoutOutput: +1.72%；三次配对 -2.54%, +0.95%, +1.72%。
- small_control/formingBorrowedDelta: +13.55%；三次配对 +20.84%, +25.62%, -12.44%。
- small_control/warmBlockWithoutOutput: -1.80%；三次配对 -5.79%, -1.36%, +0.63%。

最终 JSON 保留第一版全部案例/阶段摘要、范围、配对方向和 36 个 raw native receipt hashes；原始 metrics/full profiles/public/typed payloads 仍在第一版 ignored evidence 中，没有覆盖或用后续数字替换。第二版只有单次配对诊断，没有完整门禁或正式 36 进程资格；其原始诊断和未完成资格状态单独保留。第三版重新执行定向测试、完整两平台门禁和正式 36 进程，以上表格只读第三版实际结果。

## 未完成完整资格的第二版诊断

第二版诊断 root tool actual exit 0 / chunk `57d202`，8 个实际独立进程的显式语义/profile 对照通过；只有单次配对和诊断窗口，没有完整门禁、三次正式重复或独立正式审计，状态为 INCOMPLETE。列路径退化后没有提交。原始 8 个进程所有 stages/guards、单次摘要、源码 archive、21 份历史收据（17 原件与 4 captured direct libraries）完整保留。它属于诊断后的优化拒绝，不标作实际 tool failure，也不借用第三版通过结果补齐第二版资格。

- matrix_row64/historicalFullOutput: 单次配对 +12.10%；101.4022 → 89.1368 ms。
- matrix_row64/warmBlockWithoutOutput: 单次配对 +10.01%；199.1877 → 179.2452 ms。
- matrix_col64/historicalFullOutput: 单次配对 -49.94%；91.8109 → 137.6610 ms。
- matrix_col64/warmBlockWithoutOutput: 单次配对 -29.44%；200.9685 → 260.1249 ms。
- matrix_invalid64/historicalFullOutput: 单次配对 -8.72%；99.8484 → 108.5504 ms。
- small_control/historicalFullOutput: 单次配对 -4.46%；15.8103 → 16.5149 ms。
- small_control/warmBlockWithoutOutput: 单次配对 -2.06%；95.5700 → 97.5390 ms。

## 隔离 reviewer harness 收据

隔离只读 diagnostic leaf reviewer harness 最初 exit 1 / chunk `73f87e`，漏注入既有 METRICS global；补足 harness globals 后 exit 0 / chunk `578f57` 通过。该修正没有改 auditor 或生产源码，修正时 auditor hash 保留在收据中；最终 auditor 后续路径适配以其独立最终 hash 关联。这是隔离 harness 运行，未执行 whole auditor，不放入 qualification-failure ledger。

## 第三版短诊断

第三版正式采样前另有 8 进程单次配对 precheck，原 precheck_performance.py 和 performance-precheck.json 通过哈希关联。以下原始诊断方向全部保留；它们不加入正式 36 进程表、正式计数或三次配对结论，最终资格依赖上面的完整门禁与正式采样。

- matrix_row64/historicalFullOutput: 单次配对 +25.94%；96.2912 → 71.3099 ms。
- matrix_row64/warmBlockWithoutOutput: 单次配对 +24.73%；204.6251 → 154.0299 ms。
- matrix_col64/historicalFullOutput: 单次配对 +0.19%；91.6896 → 91.5162 ms。
- matrix_col64/warmBlockWithoutOutput: 单次配对 -5.32%；187.1195 → 197.0666 ms。
- matrix_invalid64/historicalFullOutput: 单次配对 +6.25%；97.3075 → 91.2263 ms。
- small_control/historicalFullOutput: 单次配对 +4.61%；16.7945 → 16.0208 ms。
- small_control/warmBlockWithoutOutput: 单次配对 -0.92%；95.8818 → 96.7597 ms。

## 剩余与边界

一般 TA 线性扫描、forming checkpoint/metadata 分配归因与页边界 COW、Map 重复 lookup、递归 HIR 仍待处理。有限只读审查确认 Map put 通常为压力计费与修改各 find 一次，满容量覆盖还先 get；remove 的计费与删除也重复 find/hash，重复定位在 maps/storage.rs:194-211。可在 maps.rs:105-106 所有 key/value 参数副作用结束后为单次操作取得一个私有 hash/slot，复用于容量、压力和修改；压力计费仍必须在 get_mut/COW 前，禁止跨 eval_expr、后续 mutation 或 compaction 复用。必须保持 deferred GC/压力计费、±0 hash 与原键 bits、collision equality、插入顺序、tombstone compaction 和 put_all 全容量预检。本轮未修改 Map，也未测量 Map 收益。

未验证 TradingView、host/request/strategy 吞吐、跨平台性能、RSS、完整资源矩阵或发布资格。旧 runtime 编译器可执行文件没有追溯封存；本轮 probe 编译器可执行文件/argv 和封存库有收据可核验。完整 profile/public/typed 原始快照位于 `.local/matrix-stochastic-20261006-r3`，通过哈希收据关联，没有重复复制到 tracked 文档。

两份报告在运行资格后添加，tracked 预期 6379 项。完整摘要及收据：[MATRIX_STOCHASTIC_RESULTS_20261006.json](MATRIX_STOCHASTIC_RESULTS_20261006.json)。JSON SHA256 `02ef1a2c93713326453ea559951416de0b559b971b675d4ec4dcf3e01f7a370a`。
