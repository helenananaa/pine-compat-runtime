# 策略准确性后续开发执行计划

状态：阶段 A、C 已关闭；阶段 B 已完成 session 复核修复及重新验收；阶段 D1 冻结清单与五项指标已由五阶段计划阶段 1 补齐，D2 不实施新行为切片。本文不新增兼容性声明。

编写日期：2026-09-06。

后续执行入口：[现代策略五阶段执行计划](MODERN_STRATEGY_FIVE_STAGE_EXECUTION_PLAN.md)。
新计划承接本文件未完成的 D1，单列语料驱动的语言补缺、独立结果验证、账户与报告、
性能优化。本文保留 A/B/C 收口与原 D2 范围记录，不因新计划而勾选尚未完成的步骤。

规划基线：本地 HEAD `deec36102`；开始实现前必须重新记录实际 HEAD。
Stage 18g 真正 OHLC 路径和 Stage 23 Bar Magnifier 已有收口记录，不能重复当作待开发功能。
本计划未核验远端分支、保护规则或最新 CI 状态。

## 1. 目标与边界

目标是把已有策略子集做得可组合、可解释、可验证，而不是扩大参数接受范围。
阶段顺序为：

1. A：混合订单类型 OCA。
2. B：宿主中立的交易时段输入与风险窗口。
3. C：普通图表跨 bar 跳空语义。
4. D：真实现代策略语料驱动的缺口补齐。

A 与 C 都依赖现有统一成交内核，但 C 不硬性依赖 B；上述顺序是默认推进顺序。
语料整理可以提前开展，语料驱动的新功能实现等 A/B/C 各自收口后再选择。
不同时修改 OCA、交易日和跳空三类核心行为。

本仓库只负责 Pine 语义、确定性执行和宿主中立契约。数据获取、交易所日历维护、
数据库、远端订单、Webhook 投递、重试调度、密钥和 CandleScope 集成均留在宿主。

全计划默认不做：

- Pine v1-v4 策略兼容或其他大规模源版本扩张。
- 新的真实经纪商连接、成交量流动性模型或随机撮合模型。
- 公开 pending-order、reservation、内部游标或内部订单键。
- 无关解析器、指标函数或模块的大重构。
- 自动提交、创建分支、推送、合并、改版本或发布；这些动作另行授权。

## 2. 如何使用本文

每次只执行一个编号步骤，完成其产物和门槛后再进入下一步。
复选框只在证据落盘后勾选；测试通过但行为证据不足时标记 partial，不标记 closed。
步骤失败时先归因，不通过刷新全部快照消除差异。

路径均相对仓库根目录。标为“计划新增”的文件和测试不是现有支持证据。
执行记录至少包含：实际基线、步骤、文件白名单、命令、退出码、测试数量、
行为证据、快照差异解释、剩余限制和下一步。

建议为每个阶段新增独立 audit，记录如下模板：

```text
状态：planned / in-progress / partial / closed
基线 HEAD：
工作区已有变更：
本次完成步骤：
行为锁定证据及日期：
允许修改的文件：
验证命令、退出码和有效用例数量：
预期输出变化及原因：
未解决问题与兼容性限制：
下一步及其前置条件：
```

## 3. 规划基线的实现依据与风险

下表是 `deec36102` 的历史观察，不是 A/B/C 实现后的现状；后续结果分别见
`STRATEGY_MIXED_OCA_BEHAVIOR_AUDIT.md`、`STRATEGY_SESSION_RISK_BEHAVIOR_AUDIT.md`
和 `STRATEGY_INTERBAR_GAP_BEHAVIOR_AUDIT.md`。

| 位置 | 已观察到的实现 | 对后续工作的意义 |
| --- | --- | --- |
| `crates/pine-runtime/src/runtime/strategy_path.rs` | OHLC/OLHC 路径及 Magnifier 输入 | 继续复用，不能另造撮合器 |
| `crates/pine-runtime/src/strategy/broker/candidates.rs` | 统一候选、排序及 generation | OCA 改变订单后必须保证旧候选不再执行 |
| `crates/pine-runtime/src/strategy/broker/order_book.rs` | exit 成交处理遍历 Exit 成员，entry/order 成交处理遍历 Order 成员 | 跨类型组的效果传播是明确审查点，不等于已证明所有混合组合都错误 |
| `crates/pine-runtime/src/strategy/broker/oca.rs` | 组分配、exit 数量预留和同组检查 | 必须与跨类型减量一并审查 |
| `crates/pine-runtime/src/strategy/broker/risk.rs` | UTC 日键及较高周期风险窗口处理 | 当前不等于交易所 session 风险语义 |
| `docs/STRATEGY_INTERNAL_STAGE23_BAR_MAGNIFIER_FILL_WIRING_AUDIT.md` | Magnifier 局部跳空及跨宿主收口记录 | 不代表普通图表跨 bar 跳空整体已完成 |

旧设计文档中的“当前不支持”可能是历史描述。以当前代码、fixture、快照和
`tests/fixtures/conformance.tsv` 为准，不机械恢复已经放开的限制。

## 4. 步骤 0：锁定基线与验证环境

- [x] 0.1 在仓库根目录执行并保存结果：

```bash
git status --short
git rev-parse HEAD
git log -5 --oneline
git diff --stat
git diff --cached --stat
git diff --check
rustc --version
cargo --version
python3 --version
command -v maturin
python3 scripts/check_structure.py
python3 scripts/check_host_parity.py
```

- [x] 0.2 阅读适用的 AGENTS.md，并检查准备修改的子目录是否有更深层指令。
- [x] 0.3 为本轮列文件白名单。规划时已有未跟踪 `AGENTS.md`，不得顺带暂存、删除或覆盖。
- [x] 0.4 阅读 `scripts/verify.sh` 及 `scripts/check_wasm_node.sh`，确认 Rust、Python、
  maturin、WASM/Node 等实际前置条件；不凭工具存在判断环境完整。
- [x] 0.5 运行一次 `scripts/verify.sh`，记录基线结果；不要在最终验收时才发现基线失败。

门槛：已有变更已分类；基线失败已区分环境问题、既有失败和目标行为缺口。
相关基线失败未解决或不可隔离时，不扩展功能。无关失败单独记录，不顺手修复。

## 5. 阶段 A：混合订单类型 OCA

### A1：先锁定行为，不先改参数检查

- [x] 新增 `docs/STRATEGY_MIXED_OCA_BEHAVIOR_AUDIT.md`，初始状态 planned。
- [x] 重新查阅 TradingView 官方策略文档的 OCA 段和对应函数参考，记录实际查阅日期。
- [x] 对文档未明确的组合制作原创最小 Pine 样本，保存可获得的公开输出证据。
  需要用户提供 Tester 输出时明确提出，不伪造实验结果。
- [x] 每个结论标为官方文字、样本观察、项目确定性规则或未验证推断。
- [x] 对照现有 sema/runtime fixture，建立“已支持／接受但缺交互证据／拒绝／待确认”清单。

必须锁定的问题：

1. 组名与 OCA 类型如何共同决定成员归属；同名不同类型和空名称怎样处理。
2. `entry` 与 `order` 哪些参数和组合已经接受，哪些还需要扩展。
3. `exit` 实际允许哪些 OCA 配置；不得为凑矩阵新增其不支持的 `oca_type` 参数。
4. `entry/order` 与 `exit` 能否通过合法配置属于同一组，以及效果是否双向传播。
5. 减量依据是委托数量、实际成交数量还是其他数量；反转成交与净仓变化如何区分。
6. 一个 exit 命令绑定多笔交易时，哪些是独立成员，哪些是同一命令的内部子项。
7. 替换同 ID 订单、改组、取消、重新创建订单后的身份和组成员生命周期。
8. 同价竞争的可观察保证；内部稳定排序不能直接当作 TradingView 内部顺序证据。

门槛：每个计划支持的组合均有预期结果及证据级别；未解决组合明确排除。
若合法跨类型配置不能被证实，缩小阶段范围并记录原因，不能为了计划强行实现。

### A2：补表征测试与失败用例

- [x] 先为现有单类型 none/cancel/reduce 写或复用表征测试，锁住旧行为。
- [x] 增加最小跨类型失败用例，断言成交价格、成交数量、剩余可成交量、仓位及交易记录。
- [x] 对只涉及内部状态的行为使用 broker 单测，不为了测试扩展公共 JSON。
- [x] 运行定向测试，保存“预期失败”的具体断言，排除脚本未接受或输入错误造成的假失败。

建议测试名统一包含 `mixed_oca`；测试放入已有 OCA 测试模块，过大时新增独立测试模块。
新模块必须注册，过滤测试运行结果为 0 不算通过。

最低用例矩阵（只执行 A1 证实合法的组合，其余保留负向或排除证据）：

| 维度 | 必须覆盖 |
| --- | --- |
| 订单类型 | entry→order、order→entry、entry/order→exit、exit→entry/order；保留 exit→exit 对照 |
| 组关系 | 同名同类型、同名不同类型、不同名、空名；none 独立性 |
| 结果 | cancel；reduce 后正数、恰为零、低于零钳制；无关组不变 |
| 数量 | 整数与小数；实际成交量不同于请求量；反转与净额处理 |
| 身份 | 同 ID 替换、改组、取消后重建、多笔 entry、相同 entry ID |
| 触发 | market、limit、stop、stop-limit；bracket/trailing 选择有交互价值的代表 |
| 时间 | 不同路径位置、同价、同 bar 连续成交、跨 bar 待成交 |

不要求所有维度做笛卡尔积；每个关键交互必须有具名用例，省略项需解释。

### A3：统一内部效果传播

- [x] 审查 `OcaGroupKey`、`OcaMember`、`OcaPeerEffects` 和成员清理路径。
- [x] 明确一次成交的源成员、所属组、实际成交量以及待修改的成员集合。
- [x] 消除成交后处理对 Order/Exit 类型的隐式隔离，保留经 A1 锁定的类型特有规则。
- [x] 让取消、减量及归零清理覆盖成员身份和相应数量字段；不得仅修改 OCA 映射。
- [x] 保留薄包装或做局部抽取即可，不重写整个订单簿。

门槛：A2 目标失败用例转绿，已有单类型表征不变；成员映射无悬空对象，
减量非负，异组不受影响，同一成交不会重复产生 OCA 效果。

### A4：接入成交生命周期、预留与候选失效

- [x] 追踪所有 `apply_oca_after_fill`、`apply_oca_after_exit_fill` 调用者，绘制实际调用顺序。
- [x] 核实成交应用、OCA 传播、数量预留调整、候选失效与重算之间的时序。
- [x] 保证下一候选或脚本重算看到一致状态；不能成交一半时暴露中间状态。
- [x] 验证取消／减量后旧 generation 的候选不会按原数量成交。
- [x] 验证 entry/order 减量同步到后续净额处理；exit 减量同步到有效预留和撤单清理。
- [x] 对多 entry 分配及 bracket 内部子项检查重复扣减，不能既按命令又按子项双扣。
- [x] 增加 ledger／聚合仓位／权益／佣金一致性断言，覆盖多空、部分退出和反转。
- [x] 回归 `cancel`、`cancel_all`、订单替换、风险平仓和强平，不擅自把后两者纳入用户 OCA 组。

门槛：没有幽灵成交、负数量、重复预留释放或账本不一致；未参与新组合的结果不变。

### A5：覆盖路径、重算和回滚

- [x] 标准 OHLC 的 high-first 和 low-first 各有混合 OCA 用例。
- [x] Magnifier 不同 lower bar 上，早先成交可以影响后续订单。
- [x] `calc_on_order_fills` 重算只消费剩余路径；被取消订单不复活，新订单不回填已消费价格。
- [x] `process_orders_on_close` 和合法 `immediately` 场景保留既有行为。
- [x] 批量与 incremental append 对同一历史输入产生一致结果。
- [x] 历史 realtime seed 与批量一致；live forming replacement 单独验证回滚，不套用历史 tick 假设。
- [x] 回滚恢复组成员、订单数量、预留和候选身份；确认 bar 后无重复成交与重复事件。

门槛：模式间相同输入语义的结果一致；有意不同的历史／实时行为在 audit 中写明。

### A6：公开接受范围和跨宿主证据一起收口

- [x] 只有 runtime 行为完成后，才修改确实需要调整的 sema 接受边界。
  已经接受的组合无需人为新增开关，应补行为与文档限制。
- [x] 保留 series `oca_name` 等本阶段未实现变体的负向测试。
- [x] 新增原创 Pine/CSV fixture 和有预期解释的 goldens，并注册 CLI 快照。
- [x] 在 Python/WASM 对同一 fixture 断言同一公共输出，不只断言执行成功。
- [x] 同步 `tests/fixtures/conformance.tsv`、矩阵快照、执行语义和语言边界文档。
- [x] 仅按仓库既有约定记录未发布变更，不改版本或发布状态。
- [x] 公共 schema 默认保持不变；如确需改动，先拆独立契约设计，不在本阶段顺带完成。

### A7：阶段验收

先执行定向门禁，再扩大范围。以下命令中的过滤名以实际新增测试注册为准：

```bash
cargo test -p pine-runtime mixed_oca
cargo test -p pine-runtime oca
cargo test -p pine-sema oca
cargo test -p pine-runtime magnifier
cargo test -p pine-runtime --test incremental
cargo test -p pine-runtime --test realtime
cargo test -p pine-cli runtime_outputs_match_golden_snapshots
cargo test -p pine-cli matrix_output_matches_golden_snapshot
python3 scripts/check_host_parity.py
scripts/verify.sh
git diff --check
```

- [x] 所有新增测试实际执行，目标缺口已有前后对比证据。
- [x] 全量验证在最终代码状态通过；失败不能仅因“不是新测试”而忽略。
- [x] 按文件白名单审查每个 golden 变化，不批量接受无解释结果。
- [x] 完成混合 OCA audit，列支持组合、排除组合、命令结果和输出变化。
- [x] 更新本计划状态及路线图入口，但仅将已验收部分标为 closed。

## 6. 阶段 B：交易时段输入与风险窗口

### B1：锁定窗口语义

- [x] 新增 `docs/STRATEGY_SESSION_RISK_BEHAVIOR_AUDIT.md`。
- [x] 列出现有 `trading_day_key`、风险窗口键及其调用者，不直接替换所有 UTC 运算。
- [x] 分别锁定 intraday loss、filled orders、consecutive loss days 的归属与重置规则。
- [x] 为隔夜时段、多时段交易、非交易日、日线和高于日线周期定义目标子集。

门槛：明确区分自然日、交易日、交易时段和图表 bar；不假定所有风险规则用同一个键。

### B2：先定义宿主契约

- [x] 评估宿主提供逐 bar 交易日/窗口标识，或提供已规范化的时段区间。
- [x] 写清 schema 版本、时间单位、时区责任、区间端点、覆盖范围与多时段归属。
- [x] 为重复、重叠、无序、缺失、未知版本和 live 更新冲突定义稳定诊断。
- [x] 决定缺失输入是保留已文档化的 UTC 子集还是拒绝特定模式；不得默默声称 session 精确支持。
- [x] 保证批量预检与增量未来输入规则一致，不要求流式宿主提供无限未来日历。

门槛：契约通过评审，所有输入可由本地 fixture 提供；核心不查询网络或维护交易所日历服务。

### B3：内部实现与模式一致性

- [x] 先实现输入验证和只读窗口查询，再接入风险状态转换。
- [x] 测试一个时段跨 UTC 午夜不错误重置、实际窗口切换只重置一次。
- [x] 验证休市间隔、夏令时切换、分段时段、小于/等于/大于日线周期。
- [x] 验证增量追加、重复 forming 更新、确认 bar 和 rollback 的窗口状态。
- [x] 保留无新输入的旧子集回归，并明确允许的行为变化。

### B4：宿主与验收

- [x] CLI、Python、WASM 使用同一契约解析规则和正负向 goldens。
- [x] 涉及 RealtimeSession 时先审查 ABI 版本策略，不默认沿用也不无理由升级。
- [x] 同步契约、诊断、风险文档和 conformance，运行定向测试及 `scripts/verify.sh`。
- [x] audit 只声明被覆盖的 session 子集，不宣布完整交易所日历兼容。

## 7. 阶段 C：普通图表跨 bar 跳空

### C1：行为锁定与差异表

- [x] 新增 `docs/STRATEGY_INTERBAR_GAP_BEHAVIOR_AUDIT.md`。
- [x] 对照普通图表开盘、Magnifier 首个 lower open、lower-bar 间跳空三条现有路径。
- [x] 按方向和订单类型记录跨越触发价后的成交价、激活顺序及限价验证行为。
- [x] 对照官方文档和最小输出样本，锁定 stop-limit 先激活后成交及 trailing 激活/跟踪规则。

门槛：明确跳空不是连续可交易价格段；无法验证的排序或边界单列为未确认，不猜测价格。

### C2：最小实现

- [x] 先增加普通图表跳空失败用例，再修改开盘候选收集或路径入口。
- [x] 复用统一候选和成交入口，避免普通路径与 Magnifier 各维护一套订单逻辑。
- [x] 验证价格订单和待成交 market 单只参与一次开盘处理。
- [x] 验证 stop-limit 不使用激活前价格回填；trailing 不消费不存在的中间价格。
- [x] 验证风险平仓、margin、OCA 与开盘成交竞争时的既有确定性规则。

### C3：矩阵和验收

- [x] 上跳、下跳、恰在触发价、无跳空、平坦 bar、多空各有代表用例。
- [x] limit、stop、stop-limit、bracket、trailing、slippage 和 limit verification 有交互覆盖。
- [x] 覆盖跨 bar 创建订单、成交后重算和标准路径/Magnifier 切换。
- [x] 无跳空样本结果不变；普通跳空变化逐个说明，不借此修复无关会计行为。
- [x] 完成跨宿主和执行模式 goldens、文档及 `scripts/verify.sh` 后收口。

## 8. 阶段 D：真实现代策略语料驱动补缺

### D1：建立合法可复现语料

- [x] 收集原创、明确授权或许可允许使用的 v5/v6 策略；私有脚本不提交公开仓库。
- [x] 记录来源、授权、版本、哈希、输入参数、symbol/timeframe 和数据覆盖。公共 freeze 为
      `tests/fixtures/modern-strategy-corpus/r1/manifest.jsonl`；缺失字段为明确状态。
- [x] 固定语料清单并去重；缺数据样本与运行失败样本分开计数。N=480，M=482。
- [x] 记录 parse、sema、runtime、输出可比对性和结果一致性五个独立指标。文件/快照数量不等于分阶段实测通过数。见 `docs/STRATEGY_MODERN_CORPUS_R1_G1_AUDIT.md`。

### D2：按根因选择下一小片

- [x] 将失败分为语言/类型、内置函数、request 数据契约、策略生命周期、账户精度和报告输出。
- [x] 给每个根因记录受影响脚本数、严重程度、依赖、证据可获得性和估计复杂度。公共 freeze 与本地 permissive 叠加排名见 G1 audit。
- [x] 优先修正已接受却执行错误的行为，再考虑新增接受范围。
- [x] 数量默认值、精度、货币换算或报告字段只有被语料证明有价值时才立独立计划。
- [ ] 每次只选一个最小根因，抽取原创最小复现，执行与 A 相同的完整验收闭环。当前没有带独立预期结果的已接受却执行错误样本；不实施新行为切片。D1 的冻结清单和测量工作仍可继续，不依赖 Tester 输出；见 `docs/STRATEGY_MODERN_CORPUS_BEHAVIOR_AUDIT.md`。

门槛：比较相同冻结语料和相同输入的前后结果；“成功运行率”不能冒充“结果兼容率”。
没有独立预期结果的样本只计可运行性，不计正确性。

## 9. 所有阶段共用的停止条件

遇到以下任一项，停在当前步骤并更新 audit：

- 关键行为仅有推断，继续实现会扩大公开支持范围。
- 必须引入具体宿主依赖、网络、持久化或调度基础设施。
- 需要新增公共字段或破坏 ABI，但尚无独立契约设计。
- 目标测试没有运行，或者只能通过整体刷新 goldens 才变绿。
- 候选排序取决于 HashMap 遍历、回滚后身份漂移或事件重复应用。
- 账本/聚合状态不一致，出现负预留、重复成交或未解释的权益变化。
- 工作区出现不明来源变更，无法安全划定修改范围。

停止不等于删除已有工作。保留证据，说明缺失前置条件；只修本阶段根因，
不要靠扩大项目范围绕过门槛。

## 10. 交付与集成清单

- [x] 源码、测试、fixture、goldens、矩阵和文档在同一行为切片内一致。
- [x] 每个已接受的新行为都有正常、边界、负向和必要的模式交互证据。
- [x] `scripts/verify.sh` 对最终代码成功执行，保留命令结果；不引用旧阶段通过数替代本次验证。
- [x] 本地开发完成与远端 CI/合并/发布完成分别报告。
- [ ] 用户授权集成后，重新核验远端、分支差异和保护规则，再采用小切片 PR。
- [x] 暂存只使用明确文件白名单；不把已有未跟踪文件或无关改动混入。
- [ ] PR 事件检查成功并按授权合并后，才报告集成完成；不自动发布版本。

推荐交付粒度为 A1-A2 行为与测试、A3-A4 内核、A5-A7 交互与公开收口。
这是审查边界，不要求每组都能独立宣称支持，也不允许将预期失败测试直接合入绿色主线。

## 11. 本计划编写时的验证记录

本次只编写执行计划并添加文档入口，不修改运行时行为或兼容矩阵。
当前结构及跨宿主登记检查已有通过结果；它们检查结构和覆盖登记，不替代运行测试。
执行阶段仍必须从步骤 0 开始重新验证，不能将本文视为实现完成证据。
