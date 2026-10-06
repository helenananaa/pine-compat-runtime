# Map.remove 与惰性 series header（2026-10-06）

主要收益是省分配，吞吐仍有未闭环负项：两 plot 既有 borrowed 更新每次减少 120B、8 次分配，peak/live 与无输出路径资源不变；正式满容量 borrowed 中位数 0.1198 → 0.1483ms（-23.79%），三次配对全部为负。这组证据不支持整体性能已提升。

基线 `6ce0e6d21e11db34f873823b468880afc5d861f6`。本轮 Map.remove 复用一次查找所得的 hash/slot，参数求值、原 String 容量计费、COW、删除与压缩顺序及现有 Void 返回保持。series header 改为 FnOnce 工厂，仅新增非空 series 调用；七处调用覆盖八类输出，既有 series delta 不再克隆 metadata。

修改四个生产文件，新增四个测试文件、十二项测试。Windows/Linux 门禁各 7516 Rust、794 Python、166 tooling；Linux tooling 实际跳过 1 项。六份原门禁脚本未变。源码 6387 份，加本报告与结果 JSON 共 6389 份。

公开 fixture 初次编译实际失败：root 51bf12/exit1、child101，原因是误用不存在的 keep_last；修复仅改测试为 keep_confirmed_bars。原源码与日志保留，随后十二项定向测试通过。正式独立审计 helper 初次实际失败 e4128f/exit1，原因是预检文件路径写错；窄修为实际 baseline-preflight-results.json/samples 后重跑，未改解释器、raw 收据或门槛。诊断 runner 在执行前从 ef39e07b… refreeze 为 74203caf…，强化资源约束，属于静态准备修正，未记作 runtime 失败。

正式对照 36 个进程与基线预检 6 个进程分别核验：正式 588 段完整 public、408 段 typed、384 个测量窗口；预检 98/68/64。公开值与 typed Float 位通过独立 oracle 核验；完整 public bytes 及所有 profile 字段跨基线、候选和重复精确一致。borrowed delta 的 allocated bytes/count/peak 只允许下降，live delta 必须相等，其余 raw 非时间资源严格相等。

| 正式案例/阶段 | before 中位数 ms | after 中位数 ms | 改善方向 | 三次配对方向 |
|---|---:|---:|---:|---|
| map_int8192/analyze | 0.6607 | 0.6800 | -2.92% | -70.52%, +1.59%, -16.68% |
| map_int8192/prepared | 0.0305 | 0.0318 | -4.26% | -89.82%, -0.66%, -16.48% |
| map_int8192/historicalFullOutput | 99.6667 | 100.1410 | -0.48% | -1.63%, +6.72%, -5.28% |
| map_int8192/streamSeedWithoutOutput | 97.6605 | 99.7965 | -2.19% | -4.82%, +2.08%, -3.19% |
| map_int8192/formingWithoutOutput | 0.1073 | 0.1069 | +0.37% | -3.24%, -2.79%, +1.40% |
| map_int8192/formingBorrowedDelta | 0.1251 | 0.1161 | +7.19% | +5.03%, +12.48%, +7.19% |
| map_int8192/warmBlockWithoutOutput | 275.4830 | 270.6977 | +1.74% | -2.83%, -1.88%, +3.44% |
| map_string8192/analyze | 0.7466 | 0.6767 | +9.36% | -4.12%, +0.36%, +9.36% |
| map_string8192/prepared | 0.0352 | 0.0334 | +5.11% | -1.73%, -10.23%, +8.52% |
| map_string8192/historicalFullOutput | 120.2748 | 117.2662 | +2.50% | -1.23%, +2.50%, +0.59% |
| map_string8192/streamSeedWithoutOutput | 119.4002 | 120.4788 | -0.90% | -5.31%, -0.90%, +0.55% |
| map_string8192/formingWithoutOutput | 0.1471 | 0.1605 | -9.11% | -13.87%, -6.93%, -4.48% |
| map_string8192/formingBorrowedDelta | 0.1506 | 0.1616 | -7.30% | -8.10%, +10.12%, -7.93% |
| map_string8192/warmBlockWithoutOutput | 386.2202 | 378.5746 | +1.98% | -5.34%, +1.98%, +0.04% |
| map_append_tail129/analyze | 0.6414 | 0.6829 | -6.47% | -4.25%, -6.47%, -4.15% |
| map_append_tail129/prepared | 0.0339 | 0.0297 | +12.39% | +1.16%, +12.39%, -5.78% |
| map_append_tail129/historicalFullOutput | 28.7681 | 27.6113 | +4.02% | -4.89%, +10.83%, +4.02% |
| map_append_tail129/streamSeedWithoutOutput | 29.0449 | 27.8527 | +4.10% | +4.10%, +4.78%, +0.99% |
| map_append_tail129/formingWithoutOutput | 0.1108 | 0.1107 | +0.09% | +0.18%, +23.14%, +0.00% |
| map_append_tail129/formingBorrowedDelta | 0.1348 | 0.1303 | +3.34% | +0.37%, +5.58%, +6.31% |
| map_append_tail129/warmBlockWithoutOutput | 182.4881 | 183.9611 | -0.81% | -1.87%, -0.81%, -0.88% |
| map_capacity50000/analyze | 0.6779 | 0.6385 | +5.81% | -8.61%, +7.98%, -5.58% |
| map_capacity50000/prepared | 0.0326 | 0.0292 | +10.43% | -7.06%, +14.41%, -11.45% |
| map_capacity50000/historicalFullOutput | 149.9415 | 144.8179 | +3.42% | +0.69%, +3.42%, +0.97% |
| map_capacity50000/streamSeedWithoutOutput | 145.3006 | 148.7198 | -2.35% | -2.23%, -2.69%, -4.69% |
| map_capacity50000/formingWithoutOutput | 0.1146 | 0.1264 | -10.30% | -10.99%, +7.81%, -21.66% |
| map_capacity50000/formingBorrowedDelta | 0.1198 | 0.1483 | -23.79% | -14.61%, -22.24%, -27.19% |
| map_small_sideeffects/analyze | 1.0033 | 1.0267 | -2.33% | -11.16%, +0.38%, -2.33% |
| map_small_sideeffects/prepared | 0.0363 | 0.0367 | -1.10% | -0.26%, +0.84%, -1.10% |
| map_small_sideeffects/historicalFullOutput | 57.0962 | 56.8335 | +0.46% | +0.97%, +0.57%, +0.44% |
| map_small_sideeffects/streamSeedWithoutOutput | 56.2350 | 56.9997 | -1.36% | -0.19%, -1.36%, -0.02% |
| map_small_sideeffects/formingWithoutOutput | 0.1501 | 0.1451 | +3.33% | +8.73%, -14.07%, +2.56% |
| map_small_sideeffects/formingBorrowedDelta | 0.1469 | 0.1416 | +3.61% | +3.61%, +3.80%, -6.38% |
| small_control/analyze | 0.5036 | 0.4379 | +13.05% | +3.71%, +4.50%, +13.05% |
| small_control/prepared | 0.0351 | 0.0313 | +10.83% | -24.33%, +10.83%, +19.44% |
| small_control/historicalFullOutput | 16.9829 | 17.3072 | -1.91% | -2.79%, -1.91%, -0.25% |
| small_control/streamSeedWithoutOutput | 18.0922 | 17.3271 | +4.23% | +0.52%, +3.04%, +5.32% |
| small_control/formingWithoutOutput | 0.0907 | 0.0932 | -2.76% | -0.22%, -2.87%, -20.35% |
| small_control/formingBorrowedDelta | 0.0937 | 0.0979 | -4.48% | -0.71%, -4.48%, -3.28% |
| small_control/warmBlockWithoutOutput | 100.3174 | 100.3042 | +0.01% | +1.64%, +0.01%, -2.63% |

另做 72 进程诊断：三个角色 × 四个容量/读写案例 × quiet/materialized 两种调用方 × 三次；基线始终为 6ce，旧 5f 仅历史参考。1296 个测量窗口、294912 次 warm 更新，与正式 36 进程分开。此结果不能把上一轮容量退化归结为单一原因。

满容量 quiet borrowed 单次/warm 方向 +4.50%/+3.16%，materialized 则为 -1.34%/-0.86%；quiet 无输出 warm -13.33%，三个配对全负。不能据此推断 header 或 cache 是唯一原因。

| 诊断案例/调用方/API/阶段 | baseline ms | legacy参考 ms | candidate ms | 改善方向 | 三次配对方向 |
|---|---:|---:|---:|---:|---|
| write8192/quiet/formingWithoutOutput/singleCalls | 0.0280 | 0.0285 | 0.0280 | +0.00% | -5.24%, +0.00%, +1.12% |
| write8192/quiet/formingWithoutOutput/warmContinuous | 53.5265 | 55.3830 | 54.6748 | -2.15% | -2.30%, -2.15%, -2.73% |
| write8192/quiet/formingBorrowedDelta/singleCalls | 0.0283 | 0.0286 | 0.0274 | +3.00% | -1.76%, +3.00%, +1.15% |
| write8192/quiet/formingBorrowedDelta/warmContinuous | 59.1682 | 58.0404 | 55.8366 | +5.63% | +5.09%, +12.53%, +5.64% |
| write8192/materialized/formingWithoutOutput/singleCalls | 0.1087 | 0.1031 | 0.1093 | -0.55% | -0.97%, +0.69%, -1.82% |
| write8192/materialized/formingWithoutOutput/warmContinuous | 55.5168 | 54.6925 | 54.4920 | +1.85% | -3.90%, -0.09%, +10.46% |
| write8192/materialized/formingBorrowedDelta/singleCalls | 0.1114 | 0.1055 | 0.1275 | -14.55% | -2.33%, -1.04%, -14.55% |
| write8192/materialized/formingBorrowedDelta/warmContinuous | 58.7923 | 57.7153 | 58.5187 | +0.47% | +2.10%, +4.31%, -2.41% |
| write49999/quiet/formingWithoutOutput/singleCalls | 0.0295 | 0.0300 | 0.0301 | -1.86% | +6.46%, -2.39%, +3.73% |
| write49999/quiet/formingWithoutOutput/warmContinuous | 58.0942 | 60.2305 | 56.8725 | +2.10% | +2.30%, +2.10%, +2.40% |
| write49999/quiet/formingBorrowedDelta/singleCalls | 0.0323 | 0.0314 | 0.0306 | +5.42% | +19.18%, -10.53%, +5.50% |
| write49999/quiet/formingBorrowedDelta/warmContinuous | 61.9283 | 66.1903 | 61.2698 | +1.06% | +3.58%, -6.62%, +6.54% |
| write49999/materialized/formingWithoutOutput/singleCalls | 0.1042 | 0.1080 | 0.1166 | -11.85% | -18.57%, -11.85%, +8.07% |
| write49999/materialized/formingWithoutOutput/warmContinuous | 60.2463 | 61.8795 | 60.9051 | -1.09% | -1.09%, +0.52%, -1.83% |
| write49999/materialized/formingBorrowedDelta/singleCalls | 0.1240 | 0.1207 | 0.1193 | +3.75% | +3.15%, +20.81%, +2.13% |
| write49999/materialized/formingBorrowedDelta/warmContinuous | 63.8107 | 65.8101 | 61.8886 | +3.01% | +3.01%, +9.65%, -2.75% |
| write50000/quiet/formingWithoutOutput/singleCalls | 0.0311 | 0.0316 | 0.0315 | -1.13% | +3.31%, +1.72%, -67.68% |
| write50000/quiet/formingWithoutOutput/warmContinuous | 62.1548 | 63.6087 | 70.4398 | -13.33% | -2.71%, -10.95%, -15.77% |
| write50000/quiet/formingBorrowedDelta/singleCalls | 0.0323 | 0.0350 | 0.0308 | +4.50% | +0.47%, +6.20%, +7.51% |
| write50000/quiet/formingBorrowedDelta/warmContinuous | 63.3852 | 67.7699 | 61.3852 | +3.16% | +3.16%, +3.99%, +14.50% |
| write50000/materialized/formingWithoutOutput/singleCalls | 0.1132 | 0.1157 | 0.1071 | +5.34% | +6.00%, +7.51%, -54.51% |
| write50000/materialized/formingWithoutOutput/warmContinuous | 66.3315 | 68.2698 | 63.0870 | +4.89% | +8.21%, +15.77%, -35.95% |
| write50000/materialized/formingBorrowedDelta/singleCalls | 0.1303 | 0.1207 | 0.1321 | -1.34% | +4.16%, +6.64%, -1.34% |
| write50000/materialized/formingBorrowedDelta/warmContinuous | 66.1611 | 66.5691 | 66.7303 | -0.86% | -1.02%, -0.86%, -9.71% |
| read50000/quiet/formingWithoutOutput/singleCalls | 0.0136 | 0.0135 | 0.0129 | +4.78% | +7.30%, -4.38%, +4.78% |
| read50000/quiet/formingWithoutOutput/warmContinuous | 26.7455 | 28.6080 | 27.3441 | -2.24% | -0.63%, -8.29%, +0.26% |
| read50000/quiet/formingBorrowedDelta/singleCalls | 0.0147 | 0.0139 | 0.0133 | +9.52% | +9.52%, +13.36%, -44.18% |
| read50000/quiet/formingBorrowedDelta/warmContinuous | 29.5502 | 29.0851 | 26.9451 | +8.82% | +9.04%, +8.35%, -32.97% |
| read50000/materialized/formingWithoutOutput/singleCalls | 0.0748 | 0.0762 | 0.0743 | +0.67% | -0.88%, +3.61%, -12.68% |
| read50000/materialized/formingWithoutOutput/warmContinuous | 26.3303 | 26.2513 | 26.5154 | -0.70% | +0.59%, -1.75%, +2.09% |
| read50000/materialized/formingBorrowedDelta/singleCalls | 0.0880 | 0.0910 | 0.0866 | +1.53% | +1.53%, -2.60%, -4.94% |
| read50000/materialized/formingBorrowedDelta/warmContinuous | 30.3926 | 30.5940 | 28.5569 | +6.04% | +6.04%, +5.67%, +0.60% |

JSON 保留每阶段三次进程数据、全部 raw 时间与所有负向配对（正式 32 行，诊断 26 行），未筛选负数。原大 payload/profile sidecar 留在 ignored 收据目录，以 SHA 绑定；没有复制进 Git。

后续仍有 COW 页目录 O(pages) 复制：50000 与 8192 项相差 327 个目录条目，每次额外 2616B，与 327×8B 一致，2048 次 warm 多 5357568B。非空 varip 槽号 to_vec 也是候选，预计可省一次分配与 4×K 声明槽字节；两项本轮未修，varip 收益未测。

静态审查与实际门禁、独立审计、root tool completion 分列于 JSON。warm 块衡量整体吞吐，未验证每个中间输出；分配计量不是 RSS。没有 TradingView、宿主、策略全范围或技术债全部解决的承诺。

结果：[MAP_REMOVE_LAZY_HEADER_RESULTS_20261006.json](MAP_REMOVE_LAZY_HEADER_RESULTS_20261006.json)；SHA256 `fb2d504452986c6bb7482299824633f9b1130fd06669b190e8c53940d74ebbc8`。
