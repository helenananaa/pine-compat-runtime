# Alert 增量复制与 valuewhen 逻辑预算修复（2026-10-05）

本轮减少 alert cursor 和公开增量的重复 owned 复制，增加可配置的 valuewhen 逻辑事件预算，并归还失败候选借走的 selection 工作区。受控结果来自完整通过审计的具体输入和公开调用；不能换算成解释器整体加速或进程内存上限。

基线 `557c48b0670af5143b4ffd12af65a69fc3ba809e`，6,326 份封存源码与 311 份封存编译产物；最终 6,331 份源码绑定双平台门禁、release、链接和采样。报告两文件在门禁后生成，不属于该源码快照。

## 修改及行为边界

- OutputCursor 的 alert 尾部改用可选 AppendHistory 共享根；空 alert 使用 None。capture 按当前 bar 裁剪逻辑前缀，diff 借用迭代器比较共同前缀，保留相同 payload 的 freq_all 出现次数，逆序 remove、顺序 add。边界物理叶仍可能保留少量过期 owned 字符串；这是共享存储，不是全量零复制承诺。
- 新增 apply_update_ref、apply_update_with_context_ref、带 execution_time 的 ref API，以及 apply_request_update_ref。runtime 将 delta 移入 last_changes，借用有效至下一次可变操作；原 owned API 继续显式 clone 返回独立 delta。Python/WASM 在借用仍有效时转换/编码，避免转换前再制造一份 owned delta。RuntimeChanges 自身拥有的字符串仍需构造，public JSON 也有编码成本。
- ValueWhenLimits.max_retained_values 默认 None，保留既有每调用点最多百万事件的规则，允许多个调用点合计超过百万。Some(limit) 按所有调用点的 retained Pine value 计数，NA 同样计一个事件，payload 大小不改变计数。请求保存的 before_last、bounded 同 context 子状态和执行中的 temporary evaluator 消耗执行状态树余量；替换释放旧份额，不重复算旧子状态与活动替换。
- ValueWhenBudget 用 O(1) checked 计数维护 local/requested/external 份额；entry 同时取得旧长度、校验后写入，保留旧热路径的 entry + occurrence get 两次 map 查找。默认仍有计数和验证工作，不能声称预算零开销。预算错误先不发布计数与事件；HistoricalRuntime 延续执行错误后 poisoned 生命周期，Realtime 在候选上执行并保留有效旧状态。
- Native setter/builder/getter、Python set_valuewhen_limit(None) 与属性、WASM setValueWhenLimit(null/undefined) 与方法公开同一合同。绑定拒绝 bool、负数、小数及平台范围外值；WASM 还要求 safe JS integer。配置拒绝保持旧配置与 Pine 状态；confirmed/forming 及独立 clone 各有各自 allowance。嵌套 Pine request 语言能力仍按现有支持边界。
- forming/confirm 候选失败将借走的 selection_scratch 移回旧 forming；request 外层回滚也把 live workspace 交给恢复状态，避开 Clone 有意省略 scratch。直接固定长度回归验证完整 profile 不变；requested backup Clone 可以合法 compact Small/rolling 短历史，测试保证 Pine值、事件数、cache、revision、可复用工作区与 Replica retry，不承诺所有物理容量不变。候选使用更大动态 selection 后，纯缓存也可能增长。

## 完成资格

| 平台 | Rust passed | Python passed | tooling Ran / skipped | 生产 Rust 文件 | Node WASM | actual exit |
|---|---|---|---|---|---|---|
| windows | 7385 | 794 | 166 / 0 | 403 | PASS | 0 |
| linux | 7385 | 794 | 166 / 1 | 403 | PASS | 0 |

最终 Windows 与 Linux canonical verify 的 source-bound phase 和实际 exit 都为 0；对应原始门禁日志和独立审计证明保存在 JSON。Native、实际 WASM/Node 与新 wheel/venv Python 验证 alert delta/缓存/Replica 语义以及限额配置、拒绝、forming/确认和 replay。绑定通过不代表其性能已测：下述五指标性能采样使用 native release 二进制。

完整性能资格为 126 个 fresh 进程：alert-owned 10 cases × before/after × 3 = 60，alert-borrowed 同 10 cases × after-only × 3 = 30，budget 六 cases × before/after × 3 = 36。合计 4,032 次原始阶段测量、20,160 个五指标值、3,492 份原始 public JSON；gzip mtime0 无损保存并核对原始 SHA/bytes。owned 的 1,020 份对应 JSON、budget 的 216 份对应 JSON，以及 borrowed 对 after-owned 的 1,020 份 JSON 均逐 decoded byte 相同，完整对应 profile 的全部 102 字段一致。borrowed 是新 API 的 after-only 对 after-owned 比较，不称作旧版 borrowed before/after。

独立审计不导入 process runner、不启动探针，重读原始 stdout、exit、hash receipts 与 decoded JSON，并重新计算所有原始样本中位数、比较与范围；算术和 schema guard 复用 alert_validation.py/budget_validation.py，不宣称公式验证实现完全独立，也不另行实现 native RuntimeReplica。生成器绑定 independent-audit-results.json、auditor/checker SHA 与外层真实退出 0 凭证；门禁计数另从最终日志解析，审计原件不添加推定字段。完整各阶段范围和原始 process samples 见 JSON；3 次 fresh 描述性中位数不构成统计显著性或波动归因。

## 公开调用与测量边界

所有 family 每进程各 5 个单次阶段与 3×9 个重复阶段，共 32 次。Alert 测 analyze、seedWithoutOutput、9 formingWithoutOutput、9 formingApply、9 returnedChangesClone、confirmed、nextHistorical、nextFormingAfterConfirmation；budget 测 analyze、historicalSeed、9 historicalClone、realtimeSeedWithoutOutput、9 formingWithoutOutput、9 formingApplyUpdate、confirmedWithoutOutput、nextHistoricalWithoutOutput。各调用返回后立即读取 Instant 时长并记录 allocator 快照；累计请求量/次数与净 live 使用入口、出口之差，额外 peak 使用窗口最高 LIVE 相对入口的增量。全局 allocator 持续计数；caller-owned runtime/delta 销毁、profile、公式/Replica核验、编码和文件写入均在测量窗口外，runtime 内部状态替换/销毁仍计入该调用。

Alert 先 seed 3 根历史 bars，seed 不产生告警；Bar.time=1_700_000_000 + index×60（毫秒），这是 60ms 人工 fixture，不是市场一分钟节奏。prefixChars 只计 ASCII x 前缀，不包含 |price|loopindex 后缀；完整输入、修订序列及所有输出核验保存在 JSON。

Budget 的 K=1/8/64 个独立全局调用点 × N=10,000/20,000 bars，单 plot 汇总 ta.valuewhen(true,close+j,bar_index%5000)。price=100+i/4；独立整数 quarter 公式逐 IEEE 位核对全历史、所有 clone、confirmed/next 和实际 forming delta。seed 的 valuewhenStateSlots=K、Values=K×N；64×20,000=1,280,000 证明默认 aggregate 没有偷偷百万截断。预算 probe 只使用基线公共 API，不调用新 getter/setter。

### Alert owned 前后

| case | formingApply ms 前→后 | 累计请求 bytes 前→后 | allocations 前→后 | 额外 live peak bytes 前→后 |
|---|---|---|---|---|
| noalerts | 0.0279 → 0.0281 | 7,108 → 7,108 | 30 → 30 | 4,608 → 3,664 |
| count1_prefix16 | 0.0282 → 0.0342 | 8,476 → 8,278 | 69 → 63 | 5,196 → 4,459 |
| count1_prefix4096 | 0.0444 → 0.0408 | 53,356 → 44,998 | 77 → 71 | 21,516 → 20,779 |
| count1_prefix16384 | 0.1189 → 0.11 | 188,524 → 155,590 | 79 → 73 | 70,668 → 69,931 |
| count16_prefix16 | 0.0831 → 0.067 | 24,311 → 21,131 | 524 → 458 | 11,480 → 8,940 |
| count16_prefix4096 | 0.2732 → 0.2037 | 619,991 → 486,251 | 532 → 466 | 272,600 → 270,060 |
| count16_prefix16384 | 1.0193 → 0.9956 | 2,414,039 → 1,887,083 | 534 → 468 | 1,059,032 → 1,056,492 |
| count256_prefix16 | 0.8471 → 0.7636 | 291,707 → 240,503 | 7,739 → 6,716 | 115,784 → 87,620 |
| count256_prefix4096 | 3.9166 → 3.7722 | 9,700,187 → 7,560,023 | 7,747 → 6,724 | 4,293,704 → 4,265,540 |
| count256_prefix16384 | 19.8889 → 15.2647 | 38,036,315 → 29,604,695 | 7,749 → 6,726 | 16,876,616 → 16,848,452 |

### Borrowed 对 after-owned

| case | formingApply ms owned→ref | 累计请求 bytes owned→ref | 额外 live peak bytes owned→ref | returnedChangesClone bytes owned→ref |
|---|---|---|---|---|
| noalerts | 0.0281 → 0.0243 | 7,108 → 6,148 | 3,664 → 3,664 | 960 → 960 |
| count1_prefix16 | 0.0342 → 0.0363 | 8,278 → 7,104 | 4,459 → 4,459 | 1,174 → 1,174 |
| count1_prefix4096 | 0.0408 → 0.0541 | 44,998 → 35,664 | 20,779 → 20,779 | 9,334 → 9,334 |
| count1_prefix16384 | 0.11 → 0.1508 | 155,590 → 121,680 | 69,931 → 69,931 | 33,910 → 33,910 |
| count16_prefix16 | 0.067 → 0.0853 | 21,131 → 16,735 | 8,940 → 8,940 | 4,396 → 4,396 |
| count16_prefix4096 | 0.2037 → 0.2181 | 486,251 → 351,295 | 270,060 → 270,060 | 134,956 → 134,956 |
| count16_prefix16384 | 0.9956 → 0.7971 | 1,887,083 → 1,358,911 | 1,056,492 → 1,056,492 | 528,172 → 528,172 |
| count256_prefix16 | 0.7636 → 0.8769 | 240,503 → 183,955 | 87,620 → 87,620 | 56,548 → 56,548 |
| count256_prefix4096 | 3.7722 → 2.8112 | 7,560,023 → 5,414,515 | 4,265,540 → 4,265,540 | 2,145,508 → 2,145,508 |
| count256_prefix16384 | 15.2647 → 10.419 | 29,604,695 → 21,167,731 | 16,848,452 → 16,848,452 | 8,436,964 → 8,436,964 |

returnedChangesClone 仍测 caller 对真实 delta 的显式 owned clone；ref API 省去 runtime→caller 的隐式返回 clone，没有消除 caller 明确要求保留 owned 输出的成本。

### 默认预算兼容成本

这些阶段测量本轮前后版本的整次公开调用，包含执行、历史管理及状态 clone；未隔离 valuewhen 计数器的单独成本。默认路径仍执行 checked 计数和总额校验，以下保留其实际正负变化。

| case / seed events | historicalSeed ms 前→后 | clone ms 前→后 | forming without ms 前→后 | apply ms 前→后 | seed累计 bytes 前→后 |
|---|---|---|---|---|---|
| budget-k1-n10000 / 10,000 | 24.3457 → 22.8763 | 0.0013 → 0.0012 | 0.0092 → 0.0079 | 0.0348 → 0.0247 | 1,315,124 → 1,315,124 |
| budget-k1-n20000 / 20,000 | 47.4696 → 49.2216 | 0.0012 → 0.0016 | 0.0091 → 0.0105 | 0.0355 → 0.0363 | 2,619,532 → 2,619,532 |
| budget-k8-n10000 / 80,000 | 69.2395 → 68.5098 | 0.0013 → 0.0013 | 0.02 → 0.0198 | 0.0468 → 0.0428 | 5,904,700 → 5,904,700 |
| budget-k8-n20000 / 160,000 | 138.7787 → 130.0709 | 0.0013 → 0.0014 | 0.0204 → 0.0197 | 0.0651 → 0.0507 | 11,680,764 → 11,680,764 |
| budget-k64-n10000 / 640,000 | 407.8956 → 444.4886 | 0.0025 → 0.0029 | 0.1182 → 0.1146 | 0.3075 → 0.1475 | 42,606,812 → 42,606,812 |
| budget-k64-n20000 / 1,280,000 | 939.518 → 877.9197 | 0.003 → 0.0026 | 0.1253 → 0.1188 | 0.2163 → 0.1811 | 84,156,124 → 84,156,124 |

## 全部保留的正成本

以下完整列出所有阶段三 fresh 进程中位数时延增加 >5% 的项目，包括 no-alert 控制和 analyze/seed/显式返回 clone。范围重叠的项目也保留。borrowed 行比较 after-owned→after-borrowed，alert-owned 行比较 before-owned→after-owned，budget 行比较 before→after。本节范围是三个独立进程各自阶段中位数的 min–max；重复阶段每进程的 9 次原始值在 JSON 完整保留。

| family / case / stage | ms 中位数前→后 | 变化 | 三个进程中位数的 min–max 前→后 |
|---|---|---|---|
| alert-owned / noalerts / confirmed | 0.0233 → 0.0253 | +8.58% | 0.0204–0.0373 → 0.0214–0.0313 |
| alert-owned / noalerts / formingWithoutOutput | 0.0143 → 0.0152 | +6.29% | 0.0142–0.0192 → 0.0128–0.0197 |
| alert-owned / count1_prefix16 / analyze | 0.5727 → 0.7525 | +31.40% | 0.5668–0.8499 → 0.7354–0.8305 |
| alert-owned / count1_prefix16 / confirmed | 0.0257 → 0.0396 | +54.09% | 0.0256–0.0484 → 0.0299–0.0537 |
| alert-owned / count1_prefix16 / formingApply | 0.0282 → 0.0342 | +21.28% | 0.0251–0.0416 → 0.0292–0.0354 |
| alert-owned / count1_prefix16 / formingWithoutOutput | 0.0209 → 0.0224 | +7.18% | 0.0194–0.0212 → 0.0209–0.0228 |
| alert-owned / count1_prefix16 / nextFormingAfterConfirmation | 0.0235 → 0.0284 | +20.85% | 0.0219–0.0726 → 0.0269–0.0303 |
| alert-owned / count1_prefix16 / nextHistorical | 0.0175 → 0.0205 | +17.14% | 0.0166–0.0322 → 0.0183–0.0225 |
| alert-owned / count1_prefix16 / returnedChangesClone | 0.001 → 0.0011 | +10.00% | 0.0009–0.001 → 0.001–0.0014 |
| alert-owned / count1_prefix16 / seedWithoutOutput | 0.1498 → 0.2097 | +39.99% | 0.1489–0.1776 → 0.2–0.2161 |
| alert-owned / count1_prefix4096 / analyze | 0.6218 → 0.6899 | +10.95% | 0.541–0.6841 → 0.5397–0.9448 |
| alert-owned / count1_prefix4096 / confirmed | 0.0513 → 0.0636 | +23.98% | 0.0416–0.1082 → 0.0415–0.0719 |
| alert-owned / count1_prefix4096 / returnedChangesClone | 0.0013 → 0.0014 | +7.69% | 0.0012–0.0018 → 0.0014–0.0016 |
| alert-owned / count1_prefix4096 / seedWithoutOutput | 0.1673 → 0.2057 | +22.95% | 0.1396–0.1824 → 0.18–0.2651 |
| alert-owned / count1_prefix16384 / nextFormingAfterConfirmation | 0.1277 → 0.1603 | +25.53% | 0.1152–0.2193 → 0.1018–0.1901 |
| alert-owned / count1_prefix16384 / returnedChangesClone | 0.0047 → 0.0145 | +208.51% | 0.003–0.0075 → 0.0103–0.0172 |
| alert-owned / count16_prefix16 / formingWithoutOutput | 0.056 → 0.0634 | +13.21% | 0.0544–0.0662 → 0.0538–0.0679 |
| alert-owned / count16_prefix16 / seedWithoutOutput | 0.18 → 0.1943 | +7.94% | 0.1447–0.1845 → 0.1768–0.2537 |
| alert-owned / count16_prefix4096 / nextFormingAfterConfirmation | 0.2479 → 0.2629 | +6.05% | 0.2325–0.3168 → 0.1557–0.3459 |
| alert-owned / count16_prefix4096 / returnedChangesClone | 0.0141 → 0.019 | +34.75% | 0.0116–0.0176 → 0.0178–0.0203 |
| alert-owned / count16_prefix4096 / seedWithoutOutput | 0.173 → 0.1884 | +8.90% | 0.1701–0.1918 → 0.1736–0.1915 |
| alert-owned / count16_prefix16384 / returnedChangesClone | 0.2027 → 0.2424 | +19.59% | 0.1902–0.2057 → 0.2372–0.2428 |
| alert-owned / count256_prefix16 / returnedChangesClone | 0.0421 → 0.0548 | +30.17% | 0.0419–0.0439 → 0.0539–0.0565 |
| alert-owned / count256_prefix4096 / returnedChangesClone | 0.4455 → 0.9249 | +107.61% | 0.4264–0.4664 → 0.9086–0.9454 |
| alert-owned / count256_prefix16384 / analyze | 0.6429 → 0.8921 | +38.76% | 0.6023–0.7044 → 0.5939–0.9812 |
| alert-borrowed / noalerts / analyze | 0.4805 → 0.5255 | +9.37% | 0.4067–0.9584 → 0.5228–0.8777 |
| alert-borrowed / noalerts / nextFormingAfterConfirmation | 0.0196 → 0.022 | +12.24% | 0.0147–0.0215 → 0.0177–0.0267 |
| alert-borrowed / noalerts / nextHistorical | 0.0204 → 0.0225 | +10.29% | 0.0168–0.0213 → 0.0208–0.0234 |
| alert-borrowed / noalerts / seedWithoutOutput | 0.1689 → 0.2395 | +41.80% | 0.1634–0.3788 → 0.1857–0.3253 |
| alert-borrowed / count1_prefix16 / analyze | 0.7525 → 0.8621 | +14.56% | 0.7354–0.8305 → 0.7246–0.8826 |
| alert-borrowed / count1_prefix16 / formingApply | 0.0342 → 0.0363 | +6.14% | 0.0292–0.0354 → 0.0301–0.0427 |
| alert-borrowed / count1_prefix16 / nextFormingAfterConfirmation | 0.0284 → 0.0299 | +5.28% | 0.0269–0.0303 → 0.0258–0.049 |
| alert-borrowed / count1_prefix16 / nextHistorical | 0.0205 → 0.0224 | +9.27% | 0.0183–0.0225 → 0.0219–0.0409 |
| alert-borrowed / count1_prefix4096 / formingApply | 0.0408 → 0.0541 | +32.60% | 0.0398–0.0473 → 0.0422–0.0598 |
| alert-borrowed / count1_prefix4096 / formingWithoutOutput | 0.0327 → 0.0374 | +14.37% | 0.0313–0.0415 → 0.0329–0.0436 |
| alert-borrowed / count1_prefix4096 / nextFormingAfterConfirmation | 0.0569 → 0.1237 | +117.40% | 0.0553–0.089 → 0.0501–0.178 |
| alert-borrowed / count1_prefix4096 / nextHistorical | 0.0257 → 0.0578 | +124.90% | 0.0206–0.0286 → 0.0248–0.0593 |
| alert-borrowed / count1_prefix4096 / returnedChangesClone | 0.0014 → 0.0015 | +7.14% | 0.0014–0.0016 → 0.0014–0.0017 |
| alert-borrowed / count1_prefix16384 / analyze | 0.5762 → 0.6785 | +17.75% | 0.5545–0.5951 → 0.6619–0.7479 |
| alert-borrowed / count1_prefix16384 / confirmed | 0.126 → 0.1985 | +57.54% | 0.1209–0.1935 → 0.1732–0.234 |
| alert-borrowed / count1_prefix16384 / formingApply | 0.11 → 0.1508 | +37.09% | 0.1045–0.123 → 0.1432–0.2128 |
| alert-borrowed / count1_prefix16384 / formingWithoutOutput | 0.0684 → 0.1092 | +59.65% | 0.0646–0.0763 → 0.0957–0.1285 |
| alert-borrowed / count1_prefix16384 / nextFormingAfterConfirmation | 0.1603 → 0.1772 | +10.54% | 0.1018–0.1901 → 0.1659–0.2616 |
| alert-borrowed / count1_prefix16384 / seedWithoutOutput | 0.1507 → 0.1899 | +26.01% | 0.1313–0.1587 → 0.1436–0.2028 |
| alert-borrowed / count16_prefix16 / confirmed | 0.0635 → 0.1148 | +80.79% | 0.0625–0.075 → 0.0922–0.1718 |
| alert-borrowed / count16_prefix16 / formingApply | 0.067 → 0.0853 | +27.31% | 0.0575–0.0862 → 0.0729–0.141 |
| alert-borrowed / count16_prefix16 / nextFormingAfterConfirmation | 0.0588 → 0.0863 | +46.77% | 0.0533–0.0777 → 0.0819–0.1398 |
| alert-borrowed / count16_prefix16 / nextHistorical | 0.019 → 0.0361 | +90.00% | 0.0159–0.023 → 0.0301–0.0376 |
| alert-borrowed / count16_prefix16 / returnedChangesClone | 0.0032 → 0.0039 | +21.88% | 0.0028–0.0041 → 0.0035–0.0051 |
| alert-borrowed / count16_prefix4096 / analyze | 0.6591 → 0.7337 | +11.32% | 0.6115–0.7158 → 0.7132–0.8151 |
| alert-borrowed / count16_prefix4096 / confirmed | 0.1674 → 0.2681 | +60.16% | 0.1581–0.1706 → 0.1854–0.2681 |
| alert-borrowed / count16_prefix4096 / formingApply | 0.2037 → 0.2181 | +7.07% | 0.1992–0.2081 → 0.2113–0.349 |
| alert-borrowed / count16_prefix4096 / formingWithoutOutput | 0.0922 → 0.1216 | +31.89% | 0.0887–0.1117 → 0.1035–0.1232 |
| alert-borrowed / count16_prefix4096 / nextFormingAfterConfirmation | 0.2629 → 0.2915 | +10.88% | 0.1557–0.3459 → 0.1402–0.3334 |
| alert-borrowed / count16_prefix16384 / nextHistorical | 0.2972 → 0.3433 | +15.51% | 0.2042–0.3748 → 0.2719–0.3877 |
| alert-borrowed / count16_prefix16384 / returnedChangesClone | 0.2424 → 0.2573 | +6.15% | 0.2372–0.2428 → 0.2257–0.2701 |
| alert-borrowed / count256_prefix16 / formingApply | 0.7636 → 0.8769 | +14.84% | 0.7519–0.7908 → 0.7119–1.0644 |
| alert-borrowed / count256_prefix16 / formingWithoutOutput | 0.5842 → 0.6217 | +6.42% | 0.584–0.6576 → 0.553–0.7054 |
| alert-borrowed / count256_prefix16 / nextFormingAfterConfirmation | 0.6655 → 0.7937 | +19.26% | 0.626–0.6913 → 0.6517–0.8015 |
| alert-borrowed / count256_prefix16 / nextHistorical | 0.1302 → 0.1763 | +35.41% | 0.1084–0.1447 → 0.0862–0.1869 |
| alert-borrowed / count256_prefix16384 / nextHistorical | 2.0947 → 3.0726 | +46.68% | 1.9118–2.5121 → 1.9027–3.2201 |
| budget / budget-k1-n20000 / formingWithoutOutput | 0.0091 → 0.0105 | +15.38% | 0.0091–0.0095 → 0.0087–0.0108 |
| budget / budget-k1-n20000 / historicalClone | 0.0012 → 0.0016 | +33.33% | 0.0012–0.0013 → 0.0013–0.0019 |
| budget / budget-k1-n20000 / realtimeSeedWithoutOutput | 47.9181 → 50.3514 | +5.08% | 47.5854–48.3163 → 50.3005–52.2653 |
| budget / budget-k8-n10000 / realtimeSeedWithoutOutput | 63.8949 → 67.3479 | +5.40% | 63.7587–70.463 → 65.5786–71.2062 |
| budget / budget-k8-n20000 / historicalClone | 0.0013 → 0.0014 | +7.69% | 0.0012–0.0034 → 0.0014–0.0015 |
| budget / budget-k64-n10000 / historicalClone | 0.0025 → 0.0029 | +16.00% | 0.0024–0.0031 → 0.0025–0.0069 |
| budget / budget-k64-n10000 / historicalSeed | 407.8956 → 444.4886 | +8.97% | 404.4234–450.8247 → 415.5352–453.4723 |
| budget / budget-k64-n10000 / realtimeSeedWithoutOutput | 417.8589 → 443.731 | +6.19% | 408.7908–446.7837 → 415.439–473.7524 |
| budget / budget-k64-n20000 / confirmedWithoutOutput | 0.6184 → 0.7126 | +15.23% | 0.475–0.7525 → 0.3921–0.7718 |

零时间基线后的正时间单列，不能定义百分比：

本批无满足条件的项目。

所有累计请求 bytes、分配次数与额外 live peak 的正增长均列出，阈值不是 5%：

本批无满足条件的项目。

所有有符号 net live 增加单列，包含负基线变正；完整增减列表在 JSON：

| family / case / stage | net live bytes 前→后 | 净差 | 三个进程中位数的 min–max 前→后 |
|---|---|---|---|
| alert-owned / count1_prefix4096 / nextHistorical | -12,745 → -11,414 | 1,331 | -12,745–-12,745 → -11,414–-11,414 |
| alert-owned / count1_prefix16384 / nextHistorical | -61,897 → -48,278 | 13,619 | -61,897–-61,897 → -48,278–-48,278 |
| alert-owned / count16_prefix4096 / nextHistorical | -201,058 → -136,876 | 64,182 | -201,058–-201,058 → -136,876–-136,876 |
| alert-owned / count16_prefix16384 / nextHistorical | -803,170 → -542,380 | 260,790 | -803,170–-803,170 → -542,380–-542,380 |
| alert-owned / count256_prefix16 / nextHistorical | -77,446 → -54,388 | 23,058 | -77,446–-77,446 → -54,388–-54,388 |
| alert-owned / count256_prefix4096 / nextHistorical | -3,214,966 → -2,147,428 | 1,067,538 | -3,214,966–-3,214,966 → -2,147,428–-2,147,428 |
| alert-owned / count256_prefix16384 / nextHistorical | -12,664,438 → -8,451,172 | 4,213,266 | -12,664,438–-12,664,438 → -8,451,172–-8,451,172 |

allocatedBytes 与 allocations 是各调用中成功 System layout 请求（包括 realloc 新尺寸）的累计量与次数；累计申请 MB 不等于同时驻留 MB。peakAdditionalLiveBytes 是阶段入口请求 LIVE 以上的最高增量，liveDeltaBytes 是退出减入口的有符号净变化；入口对象不同会影响净变化。realloc 先从 logical live 减旧布局，再加入新布局；不观察分配器瞬间同时保留旧/新区域的峰值。所有这些量都不是 OS RSS、allocator residency、全 runtime 绝对 live bytes 或泄漏证明。

## 失败记录与排除

| 门禁 attempt | 真实原因 | Windows exit | Linux exit | 性能资格 |
|---|---|---|---|---|
| gate-attempt1 | Diagnostic code E_VALUEWHEN_BUDGET not registered | 1 | 1 | 0 |
| gate-attempt2 | New WASM test called revision getter as a function | 1 | 1 | 0 |

首次门禁缺少 E_VALUEWHEN_BUDGET 诊断登记，真实双端失败已封存。第二次新增 WASM 测试把 revision getter 当函数调用，Windows/Linux 实际退出均为 1；该次已通过的 release 与 before-links 绑定旧 smoke-test 源码，也随该 attempt 排除。修正后为最终 6,331 文件快照重新完成八个阶段，实际 exit 均为 0；两个失败 attempt 的性能进程均为 0。

定向过程保留 in-flight E0502、请求 active terminal 预算/NA 类型/不支持 nested 请求的 fixture错误、changed capture误用 Series依赖，以及 borrowed RuntimeResult 断言编译问题。retained-budget-targeted-transcript 为从真实工具回复复原的摘录与结果，并非原始 redirected stdout/stderr，也没有历史冻结源码资格。root 后续重定向 log 才作为相应原始日志；两者各自 SHA/定位在 JSON。

scratch 旧实现通过原始失败 log 复现 rolling capacity 16→8；修复保留 workspace 后最终源码回归通过。requested 的全 physical profile 相等断言曾失败（短历史 clone 合法 compact 8→6），保留该失败并收窄断言到语义、事件数、cache/revision、工作区容量与 retry。误用不存在的 RealtimeRuntime.clone 导致的编译失败也已封存并修正。这些失败不是合格性能进程；若有 incomplete sampling batch，整个旧batch明确排除，不混入最终126。

首次只读独立审计误把预算 fixture 的 plot ID 推定为 K，漏算 indicator 已占用 call-site id0；完整直接调用顺序为 indicator、K 个 ta.valuewhen、plot，实际 plot ID 为 K+1（2/9/65）。该审计实际退出 1，旧 auditor 源码、失败 traceback 与外层真实工具回执完整保留。修正仅从已校验完整 fixture 的直接调用顺序和冻结 lowering/allocator 源码计算 ID，仍检查 full snapshot 和 delta；最终资格只采用实际退出 0 的完整重审。126 个原始测量进程未因审计错误重跑，审计本身没有新增探针进程。

统计措辞修正前的报告生成器按原字节封存；封存命令首次因 PowerShell Split-Path 参数组合错误退出 1，随后改用文件对象目录属性完成。该次保存的是实际工具回复字段与错误摘录，并非重定向原始 stdout/stderr；两次封存动作均未执行报告、审计或探针。相应源码、SHA 与工具摘录定位保存在 JSON。

## 报告存储与剩余工作

配套 JSON 完整保留每个 process 的原始 metrics、完整 prerequisites、payload 哈希/定位、probe/runner/include/validator/auditor源码与 SHA。102 字段 profile 字典按 canonical UTF-8 JSON SHA256 驻留 registry：24 个字典、7,218 个引用。这只是报告存储 normalization，不是新测量或抽样；生成前把 refs 展开，验证每 process 完整 metrics 和整份 family dataset 与原始 rows结构完全相同，再对序列化后的报告再次核验。原 source tar、rlib、exe、gzip/public JSON 留在 ignored 本地材料中，tracked只存SHA/定位，不复制二进制或大块输出。

- 本轮仅提供 valuewhen 逻辑事件预算，尚无覆盖全部 retained state/payload byte 的统一字节预算，也没有 OS RSS/full resource matrix 资格。
- Math/Array 二级字符串 dispatch 的剩余分支、WMA/HMA 与 variance 的 O(L) 工作仍需单独识别触发条件和受控测量。
- 任意大字符串/tuple、更多请求/数组层级、不同 order-fill 重算和 host并发负载不由本批数值/alert输入资格覆盖。Python/WASM路径语义经门禁，native收益不能直接换算成绑定CPU或真实主机吞吐。

本次材料只证明具体修复、最终双端语义门禁和规定公开API输入的受控性能；剩余项继续作为明确债务。
