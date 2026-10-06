# Map.put 单次 lookup 复用 — 2026-10-06

基线 `5f09ef097d26d324ef0d84628354812600ad5282`。map.put 原先在满容量检查、payload 压力计费和实际修改中重复 find/hash；新实现等接收 map id、key 与 value 全部表达式求值结束后，取得单次操作的私有 hash/slot，并复用于容量检查、压力计费和写入。已有键只覆盖 value，保持原 key bits 与插入位置；新键以原 hash 追加。压力仍在 map-store get_mut/COW 前记录，key/value 参数里的 clear、删除或重插入必须先完成，禁止跨表达式副作用或后续操作缓存 slot。

不增加持久 runtime/storage 字段、heap allocation、public ABI/schema、Pine map 规则、host 依赖或缓存策略。Float 正负零仍按原数值相等与规范化 hash 定位，覆盖保留原键的正负零 bits；collision bucket 保持实际 key equality，删除与 tombstone compaction 的 slot/order 策略保持原样。map.remove 和 map.put_all 未改变语义或策略；GC/压力计费及 COW 行为仍由本轮精确对照核验。

新增 4 个独立 ordered Vec oracle 私有测试、4 个 public API 测试。私有覆盖键与值顺序、signed zero 原键 bits、hash collision、压力/共享页面及原容量、删除/compaction 与 snapshot。公开覆盖 v5/v6 Int/Float/String keys、顶层循环在同 put 调用点之前 clear/reinsert Map 以改变目标位置或使其不存在、key/value UDF 的数组计数器副作用与可观察求值顺序、128→129 与 full/partial paged tail 的 copy 隔离、alias/var/varip 的 realtime 变化、50000-key 满容量已有键覆盖与新键报错、put 之后 runtime.error、forming 替换、完整 visible/confirmed JSON 与两份完整 profile/revision/last_changes 原子回滚、确认及未来提交与 fresh batch。当前 analyzer 不支持 UDF body 内 Map 修改，因此不声称公开测试验证参数 UDF 内的 Map mutation。源码冻结 6381 项（基线 6379）；限定 2 生产修改、2 测试新增，原 canonical 门禁及 800 行 helper 门槛保持不变。

## 完整门禁

- windows: 7504 Rust、794 Python、166 工具测试、0 项跳过、407 production files。
- linux: 7504 Rust、794 Python、166 工具测试、1 项跳过、407 production files。

两平台按原 canonical verify 完成 fmt、warnings-denied Clippy、workspace、结构、host parity、实际 Node/WASM、新 wheel、新 venv 和安装后 Python 测试。独立只读 checker 核验实际日志、新测试名、源码 tar/当前字节和 root completion；实际执行资格与静态源码审查分别记录。implementation-review.json 保存两个 actor 的有限只读源码审查与四个 source hash；静态 PASS 不充当测试或性能资格。

## 同输入 native 对照

6 个独立基线语义预检进程通过，无性能摘要。正式 36 个独立进程（6 案例 × 2 版本 × 3 次；版本顺序交替）均实际退出 0：588 份完整公开 JSON、408 份 typed sidecar、384 个测量窗口、180 次实际 Replica 应用与独立 delta 重放、240000 次 warm 更新。显式完整 public、typed Float/Int/NA/signed-zero bits、完整 profile 与状态跨版本/重复均精确一致，不排除 profile 字段。全部五个 Map 案例及 SMA/SWMA 控制的每个原始非时间请求分配指标严格等于该 case baseline0，独立审计核验正式 384 个、预检 64 个原始窗口。

全部案例 seed6000。map_int8192 与 map_capacity50000 各首 bar 初始化 8192/50000 个唯一 Int key；map_string8192 初始化 8192 个 String key，使用 256-byte 公共 prefix 加十进制 key。三者每 bar 同一个 put site 覆盖 key0 共 16 次。map_append_tail129 初始化 129 个 entry，历史覆盖 key0，forming/future 将 absent key129 追加到共享 partial tail。map_small_sideeffects 为两键 Float map，key/value UDF 修改数组计数器，顶层代码每 7 bar 删除 target，plot 显示插入顺序与保留的 signed-zero key。small_control 为 SMA17/SWMA4。

表中为本机 Windows 64-bit instrumented native ms 中位数，正百分比代表候选更快。forming 每进程各 3 个原始窗口先取中位数，再汇总 3 个进程；历史/seed 每进程一次。Int/String/tail/SMA 四案例各另测一个 10000 次 warm 无输出更新块，除以 10000 只表示平均吞吐耗时，不是单次更新分位数。JSON 保留全 6 案例 analyze/prepared 与所有 runtime 阶段的中位数、范围、三次配对方向，以及 36 进程全部原始时间与分配指标，没有筛选最快重复。

| 案例 | 历史及完整输出 | seed 无输出 | forming 无输出 | forming borrowed delta | warm 10000 次块 |
|---|---:|---:|---:|---:|---:|
| map_int8192 | 102.1954 → 99.3100 (+2.82%) | 103.6862 → 100.0494 (+3.51%) | 0.1095 → 0.1186 (-8.31%) | 0.1195 → 0.1264 (-5.77%) | 287.0006 → 282.2994 (+1.64%) |
| map_string8192 | 133.7305 → 123.0741 (+7.97%) | 131.9690 → 121.6701 (+7.80%) | 0.1621 → 0.1595 (+1.60%) | 0.1652 → 0.1766 (-6.90%) | 400.6594 → 385.1587 (+3.87%) |
| map_append_tail129 | 29.5914 → 29.4145 (+0.60%) | 29.7298 → 29.5088 (+0.74%) | 0.1100 → 0.1216 (-10.55%) | 0.1224 → 0.1283 (-4.82%) | 192.6255 → 187.7580 (+2.53%) |
| map_capacity50000 | 162.5729 → 150.5350 (+7.40%) | 163.3727 → 150.4254 (+7.93%) | 0.1183 → 0.1238 (-4.65%) | 0.1210 → 0.1450 (-19.83%) | — |
| map_small_sideeffects | 58.3019 → 58.1331 (+0.29%) | 57.1590 → 58.9489 (-3.13%) | 0.1411 → 0.1297 (+8.08%) | 0.1434 → 0.1486 (-3.63%) | — |
| small_control | 18.3347 → 18.2190 (+0.63%) | 18.2178 → 18.8108 (-3.26%) | 0.0946 → 0.0972 (-2.75%) | 0.0946 → 0.0979 (-3.49%) | 105.3344 → 104.4057 (+0.88%) |

任一中位数或配对出现负方向的观测均列出；其他阶段的收益不替代这些观测：

- map_int8192/analyze: 中位数方向 +5.71%；三次配对方向 +2.64%, +5.71%, -4.26%。
- map_int8192/prepared: 中位数方向 -12.88%；三次配对方向 -8.05%, -12.88%, -9.23%。
- map_int8192/formingWithoutOutput: 中位数方向 -8.31%；三次配对方向 -8.31%, +4.40%, -14.14%。
- map_int8192/formingBorrowedDelta: 中位数方向 -5.77%；三次配对方向 +13.78%, -5.77%, -11.33%。
- map_string8192/analyze: 中位数方向 -3.57%；三次配对方向 -7.23%, +4.23%, -6.07%。
- map_string8192/prepared: 中位数方向 -6.57%；三次配对方向 -41.43%, +6.91%, -13.72%。
- map_string8192/formingWithoutOutput: 中位数方向 +1.60%；三次配对方向 +1.60%, -1.07%, -12.04%。
- map_string8192/formingBorrowedDelta: 中位数方向 -6.90%；三次配对方向 -22.70%, +3.77%, -12.56%。
- map_append_tail129/prepared: 中位数方向 -3.02%；三次配对方向 -2.40%, -3.02%, -9.06%。
- map_append_tail129/historicalFullOutput: 中位数方向 +0.60%；三次配对方向 +7.48%, -1.11%, -2.12%。
- map_append_tail129/formingWithoutOutput: 中位数方向 -10.55%；三次配对方向 -3.75%, -12.91%, -11.79%。
- map_append_tail129/formingBorrowedDelta: 中位数方向 -4.82%；三次配对方向 +1.58%, +3.27%, -5.42%。
- map_capacity50000/prepared: 中位数方向 +8.90%；三次配对方向 +10.98%, -8.48%, +13.61%。
- map_capacity50000/formingWithoutOutput: 中位数方向 -4.65%；三次配对方向 +5.50%, +9.21%, -27.63%。
- map_capacity50000/formingBorrowedDelta: 中位数方向 -19.83%；三次配对方向 -12.93%, -27.97%, -14.63%。
- map_small_sideeffects/analyze: 中位数方向 -0.89%；三次配对方向 +1.83%, -0.89%, +8.00%。
- map_small_sideeffects/prepared: 中位数方向 -19.16%；三次配对方向 -13.22%, -15.71%, -148.03%。
- map_small_sideeffects/historicalFullOutput: 中位数方向 +0.29%；三次配对方向 +3.87%, +2.29%, -0.75%。
- map_small_sideeffects/streamSeedWithoutOutput: 中位数方向 -3.13%；三次配对方向 -2.38%, -0.26%, -3.13%。
- map_small_sideeffects/formingWithoutOutput: 中位数方向 +8.08%；三次配对方向 +8.72%, +8.98%, -3.62%。
- map_small_sideeffects/formingBorrowedDelta: 中位数方向 -3.63%；三次配对方向 +13.55%, -3.63%, -9.48%。
- small_control/analyze: 中位数方向 +0.10%；三次配对方向 -9.04%, -2.92%, +6.58%。
- small_control/prepared: 中位数方向 +3.33%；三次配对方向 -16.27%, +3.33%, -1.38%。
- small_control/streamSeedWithoutOutput: 中位数方向 -3.26%；三次配对方向 +5.48%, -8.15%, +2.28%。
- small_control/formingWithoutOutput: 中位数方向 -2.75%；三次配对方向 -10.47%, -0.10%, +3.90%。
- small_control/formingBorrowedDelta: 中位数方向 -3.49%；三次配对方向 +12.28%, -24.42%, +0.22%。
- small_control/warmBlockWithoutOutput: 中位数方向 +0.88%；三次配对方向 +0.88%, -0.21%, +0.99%。

各 runtime 阶段请求分配指标的进程中位数如下，数字从实际收据读取；全部原始非时间指标严格一致，不把 lookup 减少表述为分配减少：

| 案例/阶段 | allocated bytes | allocations | peak additional live bytes | live delta bytes |
|---|---:|---:|---:|---:|
| map_int8192/historicalFullOutput | 3.62077e+06 → 3.62077e+06 | 65915 → 65915 | 2.17283e+06 → 2.17283e+06 | 2.17283e+06 → 2.17283e+06 |
| map_int8192/streamSeedWithoutOutput | 3.13973e+06 → 3.13973e+06 | 65909 → 65909 | 1.69173e+06 → 1.69173e+06 | 1.69173e+06 → 1.69173e+06 |
| map_int8192/formingWithoutOutput | 27420 → 27420 | 87 → 87 | 23744 → 23744 | 0 → 0 |
| map_int8192/formingBorrowedDelta | 31252 → 31252 | 100 → 100 | 23744 → 23744 | 0 → 0 |
| map_int8192/warmBlockWithoutOutput | 2.742e+08 → 2.742e+08 | 870000 → 870000 | 23744 → 23744 | 0 → 0 |
| map_string8192/historicalFullOutput | 3.65684e+07 → 3.65684e+07 | 233487 → 233487 | 6.36713e+06 → 6.36713e+06 | 6.36713e+06 → 6.36713e+06 |
| map_string8192/streamSeedWithoutOutput | 3.60874e+07 → 3.60874e+07 | 233481 → 233481 | 5.88604e+06 → 5.88604e+06 | 5.88604e+06 → 5.88604e+06 |
| map_string8192/formingWithoutOutput | 64831 → 64831 | 232 → 232 | 56786 → 56786 | 0 → 0 |
| map_string8192/formingBorrowedDelta | 68663 → 68663 | 245 → 245 | 56786 → 56786 | 0 → 0 |
| map_string8192/warmBlockWithoutOutput | 6.4831e+08 → 6.4831e+08 | 2.32e+06 → 2.32e+06 | 56786 → 56786 | 0 → 0 |
| map_append_tail129/historicalFullOutput | 2.72208e+06 → 2.72208e+06 | 73662 → 73662 | 1.40273e+06 → 1.40273e+06 | 1.40273e+06 → 1.40273e+06 |
| map_append_tail129/streamSeedWithoutOutput | 1.85649e+06 → 1.85649e+06 | 73650 → 73650 | 537088 → 537088 | 537088 → 537088 |
| map_append_tail129/formingWithoutOutput | 41436 → 41436 | 132 → 132 | 30340 → 30340 | 0 → 0 |
| map_append_tail129/formingBorrowedDelta | 45392 → 45392 | 151 → 151 | 30340 → 30340 | 0 → 0 |
| map_append_tail129/warmBlockWithoutOutput | 4.1436e+08 → 4.1436e+08 | 1.32e+06 → 1.32e+06 | 30340 → 30340 | 0 → 0 |
| map_capacity50000/historicalFullOutput | 1.22804e+07 → 1.22804e+07 | 152147 → 152147 | 8.21292e+06 → 8.21292e+06 | 8.21292e+06 → 8.21292e+06 |
| map_capacity50000/streamSeedWithoutOutput | 1.17993e+07 → 1.17993e+07 | 152141 → 152141 | 7.73182e+06 → 7.73182e+06 | 7.73182e+06 → 7.73182e+06 |
| map_capacity50000/formingWithoutOutput | 30036 → 30036 | 87 → 87 | 26360 → 26360 | 0 → 0 |
| map_capacity50000/formingBorrowedDelta | 33868 → 33868 | 100 → 100 | 26360 → 26360 | 0 → 0 |
| map_small_sideeffects/historicalFullOutput | 2.85254e+07 → 2.85254e+07 | 104414 → 104414 | 2.23554e+06 → 2.23554e+06 | 2.23554e+06 → 2.23554e+06 |
| map_small_sideeffects/streamSeedWithoutOutput | 2.72752e+07 → 2.72752e+07 | 104396 → 104396 | 985348 → 985348 | 985348 → 985348 |
| map_small_sideeffects/formingWithoutOutput | 56932 → 56932 | 162 → 162 | 40048 → 40048 | 0 → 0 |
| map_small_sideeffects/formingBorrowedDelta | 61012 → 61012 | 187 → 187 | 40048 → 40048 | 0 → 0 |
| small_control/historicalFullOutput | 2.29686e+06 → 2.29686e+06 | 49174 → 49174 | 1.18244e+06 → 1.18244e+06 | 1.18244e+06 → 1.18244e+06 |
| small_control/streamSeedWithoutOutput | 1.81582e+06 → 1.81582e+06 | 49168 → 49168 | 701340 → 701340 | 701340 → 701340 |
| small_control/formingWithoutOutput | 26908 → 26908 | 60 → 60 | 19648 → 19648 | 0 → 0 |
| small_control/formingBorrowedDelta | 30740 → 30740 | 73 → 73 | 19648 → 19648 | 0 → 0 |
| small_control/warmBlockWithoutOutput | 2.6908e+08 → 2.6908e+08 | 600000 → 600000 | 19648 → 19648 | 0 → 0 |

## 同冻结版本的独立复测

主批 capacity formingBorrowedDelta 退化后，同一源码、库、probe executable 与 runner 另做完整 36 进程复测，实际 completion exit 0 / chunk `de079d`。主批 36 与副批 36 合计 72 个 native 对照进程；主 formalCounts 仍是原来的 36/588/408/384/180/240000，不把副批计入、替换原摘要或挑较快的重复。副批另有同样 588 份 public、408 份 typed、384 窗口、180 Replica 与 240000 warm updates，所有完整 public/typed bytes、profile/state 也精确对照主批各 case baseline0，原冻结 guards 完全相同。独立 followup audit actual exit 0 / chunk `edeb6f`，完整复算副批摘要并核验 384 个原始非时间分配窗口严格一致。

下表只读副批自己的全部案例/阶段摘要；JSON 单独保留副批全部 analyze/prepared/runtime 中位数、范围、三次配对方向与 36 份 raw metrics/receipt hashes。

| 案例 | 历史及完整输出 | seed 无输出 | forming 无输出 | forming borrowed delta | warm 10000 次块 |
|---|---:|---:|---:|---:|---:|
| map_int8192 | 102.5382 → 97.2773 (+5.13%) | 103.8125 → 97.4395 (+6.14%) | 0.1143 → 0.1182 (-3.41%) | 0.1231 → 0.1264 (-2.68%) | 283.6731 → 277.3173 (+2.24%) |
| map_string8192 | 134.1667 → 121.5052 (+9.44%) | 132.7056 → 120.2888 (+9.36%) | 0.1508 → 0.1651 (-9.48%) | 0.1576 → 0.1566 (+0.63%) | 394.7139 → 380.0441 (+3.72%) |
| map_append_tail129 | 28.9247 → 28.8321 (+0.32%) | 29.5959 → 28.5621 (+3.49%) | 0.1134 → 0.1191 (-5.03%) | 0.1431 → 0.1274 (+10.97%) | 185.8718 → 183.2008 (+1.44%) |
| map_capacity50000 | 163.7758 → 149.7257 (+8.58%) | 157.1138 → 148.7757 (+5.31%) | 0.1131 → 0.1194 (-5.57%) | 0.1254 → 0.1400 (-11.64%) | — |
| map_small_sideeffects | 58.8398 → 57.9979 (+1.43%) | 58.3473 → 58.0973 (+0.43%) | 0.1341 → 0.1620 (-20.81%) | 0.1558 → 0.1493 (+4.17%) | — |
| small_control | 17.5457 → 17.7262 (-1.03%) | 17.1541 → 17.8633 (-4.13%) | 0.0897 → 0.0891 (+0.67%) | 0.1057 → 0.1033 (+2.27%) | 101.6388 → 99.6961 (+1.91%) |

副批任一中位数或配对为负的观测同样完整保留：

- map_int8192/analyze: 中位数方向 +2.42%；三次配对方向 -7.33%, +5.19%, +5.47%。
- map_int8192/prepared: 中位数方向 -3.17%；三次配对方向 -5.11%, -1.90%, -3.17%。
- map_int8192/formingWithoutOutput: 中位数方向 -3.41%；三次配对方向 +1.01%, -17.56%, +5.42%。
- map_int8192/formingBorrowedDelta: 中位数方向 -2.68%；三次配对方向 -57.05%, -10.97%, +3.25%。
- map_string8192/analyze: 中位数方向 +8.79%；三次配对方向 +21.53%, -3.30%, +6.19%。
- map_string8192/prepared: 中位数方向 -5.64%；三次配对方向 +11.57%, -33.55%, -1.14%。
- map_string8192/formingWithoutOutput: 中位数方向 -9.48%；三次配对方向 -19.80%, -9.48%, +4.77%。
- map_string8192/formingBorrowedDelta: 中位数方向 +0.63%；三次配对方向 +0.32%, -6.06%, +1.65%。
- map_append_tail129/analyze: 中位数方向 +4.68%；三次配对方向 +4.68%, +14.59%, -1.87%。
- map_append_tail129/prepared: 中位数方向 +1.90%；三次配对方向 -2.66%, +12.34%, -9.52%。
- map_append_tail129/historicalFullOutput: 中位数方向 +0.32%；三次配对方向 +1.98%, +0.32%, -0.54%。
- map_append_tail129/formingWithoutOutput: 中位数方向 -5.03%；三次配对方向 -7.49%, +0.33%, -0.62%。
- map_capacity50000/analyze: 中位数方向 -7.86%；三次配对方向 +27.18%, -7.86%, -80.19%。
- map_capacity50000/prepared: 中位数方向 -8.63%；三次配对方向 +16.09%, -23.64%, -37.70%。
- map_capacity50000/streamSeedWithoutOutput: 中位数方向 +5.31%；三次配对方向 +5.63%, +5.31%, -8.06%。
- map_capacity50000/formingWithoutOutput: 中位数方向 -5.57%；三次配对方向 -45.18%, -4.55%, -4.12%。
- map_capacity50000/formingBorrowedDelta: 中位数方向 -11.64%；三次配对方向 -56.08%, -6.30%, -0.43%。
- map_small_sideeffects/analyze: 中位数方向 +2.25%；三次配对方向 +2.25%, +12.93%, -16.75%。
- map_small_sideeffects/prepared: 中位数方向 +5.87%；三次配对方向 +5.64%, +26.32%, -4.89%。
- map_small_sideeffects/historicalFullOutput: 中位数方向 +1.43%；三次配对方向 -1.13%, +3.82%, +4.03%。
- map_small_sideeffects/formingWithoutOutput: 中位数方向 -20.81%；三次配对方向 -20.72%, +0.82%, -23.37%。
- map_small_sideeffects/formingBorrowedDelta: 中位数方向 +4.17%；三次配对方向 +4.17%, -3.61%, +15.79%。
- small_control/analyze: 中位数方向 +0.87%；三次配对方向 +0.87%, -5.00%, -6.04%。
- small_control/prepared: 中位数方向 -2.03%；三次配对方向 -2.03%, -7.87%, -3.93%。
- small_control/historicalFullOutput: 中位数方向 -1.03%；三次配对方向 -2.56%, -2.46%, +7.70%。
- small_control/streamSeedWithoutOutput: 中位数方向 -4.13%；三次配对方向 -2.79%, -0.49%, -5.44%。
- small_control/formingWithoutOutput: 中位数方向 +0.67%；三次配对方向 +7.76%, -8.92%, +3.48%。
- small_control/formingBorrowedDelta: 中位数方向 +2.27%；三次配对方向 +36.73%, -10.72%, -2.27%。
- small_control/warmBlockWithoutOutput: 中位数方向 +1.91%；三次配对方向 +2.57%, +0.27%, -2.59%。

副批 runtime 请求分配中位数如下；不将 lookup 减少说成分配减少：

| 案例/阶段 | allocated bytes | allocations | peak additional live bytes | live delta bytes |
|---|---:|---:|---:|---:|
| map_int8192/historicalFullOutput | 3.62077e+06 → 3.62077e+06 | 65915 → 65915 | 2.17283e+06 → 2.17283e+06 | 2.17283e+06 → 2.17283e+06 |
| map_int8192/streamSeedWithoutOutput | 3.13973e+06 → 3.13973e+06 | 65909 → 65909 | 1.69173e+06 → 1.69173e+06 | 1.69173e+06 → 1.69173e+06 |
| map_int8192/formingWithoutOutput | 27420 → 27420 | 87 → 87 | 23744 → 23744 | 0 → 0 |
| map_int8192/formingBorrowedDelta | 31252 → 31252 | 100 → 100 | 23744 → 23744 | 0 → 0 |
| map_int8192/warmBlockWithoutOutput | 2.742e+08 → 2.742e+08 | 870000 → 870000 | 23744 → 23744 | 0 → 0 |
| map_string8192/historicalFullOutput | 3.65684e+07 → 3.65684e+07 | 233487 → 233487 | 6.36713e+06 → 6.36713e+06 | 6.36713e+06 → 6.36713e+06 |
| map_string8192/streamSeedWithoutOutput | 3.60874e+07 → 3.60874e+07 | 233481 → 233481 | 5.88604e+06 → 5.88604e+06 | 5.88604e+06 → 5.88604e+06 |
| map_string8192/formingWithoutOutput | 64831 → 64831 | 232 → 232 | 56786 → 56786 | 0 → 0 |
| map_string8192/formingBorrowedDelta | 68663 → 68663 | 245 → 245 | 56786 → 56786 | 0 → 0 |
| map_string8192/warmBlockWithoutOutput | 6.4831e+08 → 6.4831e+08 | 2.32e+06 → 2.32e+06 | 56786 → 56786 | 0 → 0 |
| map_append_tail129/historicalFullOutput | 2.72208e+06 → 2.72208e+06 | 73662 → 73662 | 1.40273e+06 → 1.40273e+06 | 1.40273e+06 → 1.40273e+06 |
| map_append_tail129/streamSeedWithoutOutput | 1.85649e+06 → 1.85649e+06 | 73650 → 73650 | 537088 → 537088 | 537088 → 537088 |
| map_append_tail129/formingWithoutOutput | 41436 → 41436 | 132 → 132 | 30340 → 30340 | 0 → 0 |
| map_append_tail129/formingBorrowedDelta | 45392 → 45392 | 151 → 151 | 30340 → 30340 | 0 → 0 |
| map_append_tail129/warmBlockWithoutOutput | 4.1436e+08 → 4.1436e+08 | 1.32e+06 → 1.32e+06 | 30340 → 30340 | 0 → 0 |
| map_capacity50000/historicalFullOutput | 1.22804e+07 → 1.22804e+07 | 152147 → 152147 | 8.21292e+06 → 8.21292e+06 | 8.21292e+06 → 8.21292e+06 |
| map_capacity50000/streamSeedWithoutOutput | 1.17993e+07 → 1.17993e+07 | 152141 → 152141 | 7.73182e+06 → 7.73182e+06 | 7.73182e+06 → 7.73182e+06 |
| map_capacity50000/formingWithoutOutput | 30036 → 30036 | 87 → 87 | 26360 → 26360 | 0 → 0 |
| map_capacity50000/formingBorrowedDelta | 33868 → 33868 | 100 → 100 | 26360 → 26360 | 0 → 0 |
| map_small_sideeffects/historicalFullOutput | 2.85254e+07 → 2.85254e+07 | 104414 → 104414 | 2.23554e+06 → 2.23554e+06 | 2.23554e+06 → 2.23554e+06 |
| map_small_sideeffects/streamSeedWithoutOutput | 2.72752e+07 → 2.72752e+07 | 104396 → 104396 | 985348 → 985348 | 985348 → 985348 |
| map_small_sideeffects/formingWithoutOutput | 56932 → 56932 | 162 → 162 | 40048 → 40048 | 0 → 0 |
| map_small_sideeffects/formingBorrowedDelta | 61012 → 61012 | 187 → 187 | 40048 → 40048 | 0 → 0 |
| small_control/historicalFullOutput | 2.29686e+06 → 2.29686e+06 | 49174 → 49174 | 1.18244e+06 → 1.18244e+06 | 1.18244e+06 → 1.18244e+06 |
| small_control/streamSeedWithoutOutput | 1.81582e+06 → 1.81582e+06 | 49168 → 49168 | 701340 → 701340 | 701340 → 701340 |
| small_control/formingWithoutOutput | 26908 → 26908 | 60 → 60 | 19648 → 19648 | 0 → 0 |
| small_control/formingBorrowedDelta | 30740 → 30740 | 73 → 73 | 19648 → 19648 | 0 → 0 |
| small_control/warmBlockWithoutOutput | 2.6908e+08 → 2.6908e+08 | 600000 → 600000 | 19648 → 19648 | 0 → 0 |

capacity/formingBorrowedDelta 两批均变慢：主批 0.1210 → 0.1450 ms（-19.83%），配对方向 -12.93%, -27.97%, -14.63%；副批 0.1254 → 0.1400 ms（-11.64%），配对方向 -56.08%, -6.30%, -0.43%。该 forming 路径是后续未定位热点，没有将退化归为仅有噪声，也没有用 String 或其他阶段的收益覆盖它。

Map var 构造与首 bar 初始化包含在 history/seed 窗；cold 初始化同时含新键插入/hash，不能只归因为 overwrite。warm 不重复初始化，输入提前构造并保留到窗口结束。Session、独立线性 ordered Vec oracle、序列化、profile、Replica 与 I/O 位于各自测量窗外，historical 窗包含 owned result。warm 仅核验显式初始/最终 snapshot、最终 profile/revision，不检查每次中间 update；普通 forming、confirm 和未来提交按各自显式快照核验。allocated bytes/count、相对入口请求存活峰值与退出存活差属于 System Layout 申请指标，不能替代 RSS。

## 本轮实际资格失败与修正

- baseline-preflight-initial-failure.json: `baselinePreflightInitial` root tool actual exit 1 / chunk `6a101a`，原始 child runner exit 1。原因：The small-case benchmark used generic type annotations on UDF arguments rejected by the sealed baseline parser (E_PARSE_EXPECTED). 修正：Use unannotated UDF c/target arguments with identical evaluation effects and map oracle; no production parser or Map semantic change for this fixture correction. 原始日志与 runner exit 独立保留。
- baseline-preflight-untyped-failure.json: `baselinePreflightUntyped` root tool actual exit 1 / chunk `c2c8af`，原始 child runner exit 1。原因：Unannotated UDF parameters parse, but the sealed baseline analyzer rejects map.remove inside a UDF as function_side_effect. 修正：Move the identical periodic map.remove to top-level immediately before outer map.put; preserve array counter key/value UDF effects and independent ordered-map oracle. 原始日志与 runner exit 独立保留。
- targeted-map-initial-failure.json: `targetedMapInitial` root tool actual exit 1 / chunk `9e5cb4`，原始 child runner exit 101。原因：The public fixture used map.clear/map.put in key/value UDFs; the current analyzer rejects Map mutation in UDFs as function_side_effect. 修正：Use supported array counter effects in key/value UDFs to verify evaluation order, and perform map reset/reinsertion at top-level before the same outer put site. Other three tests and all production/private-test sources remain unchanged. 原始日志与 runner exit 独立保留。

各项真实失败的原源码/日志、aggregate failure、子进程 raw stdout/stderr/process、manifest 与 helper/executable/link capture 通过收据中的原路径、字节数和 hash 完整关联。首轮 fixture 同时含受支持的 array<int> 参数和不受支持的 map<float,float> 参数头，解析失败来自 Map 泛型参数头；不把数组参数列为解析缺口。原始未完成预检样本不加入正式 36 进程计数或性能摘要。修正后的完整 6 进程语义预检成功原件 `baseline-preflight-final.json` 与 canonical baseline-preflight.json 精确一致；历史 executable 只核验 capture 字节，不把已经推进的 live 路径当作原件。脚本 fixture 的修正保持本轮性能范围，没有放宽 production parser 或 UDF mutator 限制。

## 独立准备命令失败

prep-static-command-failure.json 记录只读准备命令的 quoting SyntaxError：准备命令 actual exit 1 / chunk `ffdf98`。该命令没有执行 runtime 或 auditor main；改用 literal here-string 后继续准备，没有改生产源码或 auditor。此项独立关联，不放入实际资格失败 ledger。

report-helper-ast-failure.json 另记报告脚本作者的 AST 检查 actual exit 1 / chunk `f8c671`；分成两行但未续行的 assert 导致 IndentationError，改为合法单行后 AST 重检 actual exit 0 / chunk `ca2100`。失败初版脚本字节已 capture，未执行 helper main、runtime 或 auditor，未改 tracked source，不归入资格失败 ledger。

报告 writer 首次 main actual exit 1 / chunk `5b2d8d`，在写 docs 前把未测 warm 的 None 当作窗口而报错。只对非 BLOCK 案例、warmBlockWithoutOutput、候选与参考均为 None 的明确未测阶段跳过；其余 None 仍失败，384/64 窗口及全部原始非时间指标严格一致要求保持。旧 writer 源码已 capture；错误输出是从实际工具结果转录的文本，不宣称 raw stderr 文件。没有执行 runtime/auditor 或改变源码/已完成资格，不放入三项 qualification ledger。

## 剩余与边界

map.remove 的 pressure 与实际删除仍重复 hash/find；本轮没有改 remove，也没有测量 remove 收益。Map<K,V>/Matrix<T> 泛型 UDF 参数缺少解析及类型元数据支持，是独立兼容性缺口；Array<T> 参数头已有 parser/sema 处理，本轮 array<int> 注解不是失败原因。另一个独立限制是 analyzer 当前无条件拒绝 UDF 内 Map put/clear/remove/put_all，而受支持的数组/矩阵修改可通过，本轮没有扩展该语义行为。通用 TA 线性扫描、forming checkpoint/metadata 分配与页边界 COW、递归 HIR 仍待处理。map.put_all 原容量预检与操作策略保持原样，需要单独审查，不能把单 put 的私有 lookup 跨 mutation/compaction 复用。

未验证 TradingView、host/request/strategy 吞吐、跨平台性能、RSS、完整资源矩阵或发布资格。不推断 Map 的总 CPU 占比或所有路径加速。旧 runtime 编译器可执行文件没有追溯封存；本轮 probe 编译器 executable/argv 与封存库通过收据关联。完整 profile/public/typed 原始快照留在 `.local/map-put-20261006`，以哈希关联，不重复复制到 tracked 文档。

两份报告在执行资格后添加，tracked 预期 6383 项。完整摘要与收据：[MAP_PUT_LOOKUP_RESULTS_20261006.json](MAP_PUT_LOOKUP_RESULTS_20261006.json)。JSON SHA256 `a3abec57e99b1f6cd80ff7dfa116e80cca0da302bb397c9efd86e3fb0dc2ee65`。
