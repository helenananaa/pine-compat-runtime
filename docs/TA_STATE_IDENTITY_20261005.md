# TA 状态调用点隔离 — 2026-10-05

基线 `c57298550129b8455bd8a3a777d0f743c921141c`。本轮修复手工 HIR 中不同 callee 复用 CallSiteId 时的 TA 状态污染；不涉及 host 集成。

## 问题与修复

正常前端分配唯一调用点。手工 HIR 的派发兼容允许不同 callee 共用 ID，但窗口和递推缓存仍只使用该 ID：WMA 的普通追加会污染 SMA 的同 bar 撤销；EMA、cum、max 等也会读写彼此的状态。fixnan 同样使用 call_state，因此仅分离窗口模式不能完整阻断这类冲突。

CallPlan 在准备时收集发生 callee 冲突的实际 ID，按调用点和 TA opcode 的确定顺序分配内部状态 ID。分配排除所有实际 HIR 调用 ID，并使用空洞，不相信 next_call_site_id 或 max+1。所有冲突 TA 都使用独立槽；fixnan 及其他调用保留原始槽。只有 eval_ta_call 收到内部 ID。

同 ID、同 callee 仍表示一个逻辑状态调用点，保留既有循环调用和同 bar 替换规则。没有按语法出现次数拆分状态。公开 HIR、input override、plot、request、来源 ID 不被重写；公共输出 schema 不变。普通唯一 ID 继续走缓存派发，无额外别名查找。准备元数据新增空映射字段、state-site 字段和冲突扫描，本轮未量化这些字段的字节成本。

## 直接 release 证据

公开 API 探针仅将五个 TA 调用 ID 改为 u32::MAX；保持 SOURCE、参数、plot ID 和次序。close=1 时依次执行 WMA(1,2)、WMA(1,2)、SMA(2,2)、WMA(2,2)、SMA(99,2)。

- 封存基线实际退出 0，typed 输出为 NA / 1 / 1.5 / 2 / 50。
- 候选实际退出 0，同一个探针输出为 NA / 1 / NA / 5/3 / NA，浮点 bits 与明确预期一致。
- 两版原始 SOURCE 和 TA ID 列表一致，stderr 为空。候选最后 SMA 尚未得到两个 bar 的输入，因此正确值为 NA；不能把旧共享窗口的物理均值 50.5 当作修复目标。

## 验证

冻结 6352 个源文件；完整门禁前核对同一份 SHA256 清单。新增 11 个回归；定向过滤另覆盖 1 个已有 request 测试，12/12 通过。

独立参考 HIR 给每个 callee 分配不同的公开 ID，同 callee 仍复用同 ID，避免照抄内部别名分配器。逐 bar 比较完整公开 JSON 和 typed Float bits，覆盖稀疏 MAX、密集冲突、动态长度/NA、EMA/RMA/DEMA/TEMA、cum/max/min/barssince、highest/lowest、三种 cross 和 fixnan。

实时验证覆盖 forming 替换、确认、保留窗口、7 次 delta/replica、真正执行 runtime.error 后的回滚、后续更新、历史修正和重放。有界 equal-timeframe request 使用完整数据集并确认实际创建子运行时；input override 与公开 HIR 保持不变。结构回归覆盖空洞、确定性、同 callee 具名参数、声明外 fallback、别名数量及普通缓存路径。

| 平台 | Rust passed | Python passed | 工具检查 | 工具 skip | 生产 Rust 文件 | 实际退出 |
|---|---:|---:|---:|---:|---:|---:|
| windows | 7442 | 794 | 166 | 0 | 406 | 0 |
| linux | 7442 | 794 | 166 | 1 | 406 | 0 |

两端门禁包含 fmt、warnings-denied Clippy、workspace/结构检查、host parity、实际 Node/WASM 和新 wheel/新 venv/安装后的 Python 测试；没有放宽阈值。独立源码审计核对状态所有权和参考程序，收据解析核对真实门禁输出、退出和源文件身份。

## 保留的失败与边界

初次基线封存把 git archive 的原始 blob 与旧编译工作副本按字节比较，在 .gitignore 失败，尚未执行探针。103 个历史文件存在 CRLF/LF 差异；修复只允许换行等价，并保存这 103 个确切编译副本及哈希，不接受其他内容变化。

初次定向测试 11 passed / 1 failed：有界 request fixture 使用单 bar append，缺少已知历史终点。保留原测试、日志、Cargo 退出 101 与 PowerShell 外层退出 1；仅改用 append_bars 的完整数据集入口，随后 12/12 通过。收据 checker 初次误把 Cargo 101 当作外层 1，修复后分别核对；辅助诊断读取的编码错误同样单独保留，不计作门禁失败或运行时回归。

本轮是正确性修复，没有新的耗时、分配、RSS 或元数据大小测量；不能沿用上一轮数字声称本轮性能收益。仍未完成 TradingView 官方参考、完整资源矩阵或发布资格。相同 callee 的复用继续共享历史，独立逻辑调用应使用唯一 ID；这里也不承诺私有 eval_call 任意声明外 ID 的隔离。

页目录 COW 的 O(L/128) 引用复制、一般 WMA/HMA/variance 扫描仍保留；Map/Matrix 二级分派及递归 HIR 需进一步实际热点归因。

完整收据：[TA_STATE_IDENTITY_RESULTS_20261005.json](TA_STATE_IDENTITY_RESULTS_20261005.json)。JSON SHA256 `e080b312793657514b93fabf20f23183225258712224a0ad98c9a263d9f03291`。
