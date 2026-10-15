# 保存策略库的解释器缺口审计

日期：2026-10-10。读取 Codex 对话「寻找并验证热门 Bot 策略」
`01a1252f-cd43-70e3-b307-270e82e40f49`，以及其 MarketForge 本地源码库、
87 个具名候选、前轮 223 个 Pine 样本的报告。

结论：存在明确的解释器兼容缺口，最有覆盖价值的是旧版订单条件参数、
旧函数别名、完整退出组合、动态订单 ID、函数及表达式语法。
少数热门公式的简化适配版能运行，不能替代完整策略原版的兼容证明。
同时，缺少请求数据、库源码、执行时钟，或宿主不支持挂单/双腿执行，
需要分别处理，不能全部归入解释器缺口。

## 本次证据范围

- 当前源码 HEAD：`28ecae3c645e9269e2099d27cc9abbb1f52ee9ea`。
- 从当前工作区重新构建 `pine-cli`，版本 `0.3.2`。原有 CLI 实际为
  `0.3.0`，本次没有使用它作结论依据。
- CLI SHA256：`924b7fa841a5d345ab6beb3492cdb10f301b092bad41a4eb131d2b11d9652224`。
- 保留工作区原有 Cargo profile、宿主接口文档与资源脚本改动；未修改解释器实现、
  外部策略原文件、MarketForge 插件或默认市场配方。未提交、未推送。
- 输入为 2,211 个归档文件、5,358 个提取片段和 7 个 TradingView 转录文件，
  合计 7,576 个条目。原保存文件的哈希全部匹配。
- 只在审计副本去掉明确的网页标题/行号/Expand 尾部、FMZ `/*backtest...*/`
  配置头，并把 NBSP 转为普通空格。逐文件保留处理记录和前后哈希。
  这些是**提取后的代码结果**，不能称为夹带网页文字的原始文件直接运行成功。
- 242 个条目没有可独立执行的脚本声明，包含片段和归档识别误报。
  剩余 7,334 个条目按提取后源码哈希去重，为 7,255 份源码。
- 全部独立源码完成语义分析；4,624 份通过分析的源码全部进行了运行检查。
  每份使用固定 seed=731、512 根合成震荡 OHLCV、1 分钟、`V-BTC-SPOT`、
  默认策略参数。进程有 8 秒运行上限，本轮没有运行超时。

按去重源码计算：

| 首轮结果 | 数量 | 含义 |
|---|---:|---|
| 默认输入运行通过 | 4,119 | 运行检查通过；不是盈利、Pine 数值等价或生产 bot 资格 |
| 分析拒绝 | 2,631 | 同时包含能力缺口、源码错误、组合脚本和缺少依赖，不能全部算解释器 bug |
| 缺 request 输入 | 356 | 分析已通过；缺少指定商品/周期/变换后的序列 |
| 缺执行时钟 | 39 | `timenow` 要求确定的宿主执行时间 |
| 账户/图表币种不匹配 | 28 | 默认 USD 图表与脚本币种不同，涉及配置或换汇能力 |
| 日期字符串解析子集 | 53 | 运行时拒绝特定 `timestamp(dateString)` 格式 |
| 其他运行拒绝 | 29 | 空数组、非法日期、时间/会话子集、输出上限等，未全部确认为解释器缺陷 |

输入补充复查：39 份时钟失败中，显式提供逐 bar 执行时间后 **35 份通过**，
另 2 份暴露请求数据缺失、2 份暴露空数组问题。28 份币种失败中，匹配图表币种后
**26 份通过**，2 份继续要求高周期数据。因此至少另有 61 份无需改解释器即可通过
相应输入下的运行检查；首轮默认输入统计保持原样，不混入补充测试。

## 具名策略直接判断

| 对话中的候选 | 当前结果 | 解释器侧的实际阻碍 |
|---|---|---|
| `(IK) Grid Script` | 分析拒绝 | v4 函数中构造、填充并返回局部数组被拒绝；`strategy.entry(long=true)` 不接受。网格使用动态 ID，而独立探针确认 series ID 也被拒绝，修首个错误不足以保证完整网格可运行 |
| `2Tier Ichimoku Pyramiding [Takazudo]` | 分析拒绝 | 返回值形式的 `if / else if`、复杂 legacy `security` 等；还含 `initial_capital=0` 等其他拒绝项，需逐项处理 |
| `Bollinger Band Breakout` | 分析拒绝 | `strategy.cancel(..., when=...)` 缺失；这是旧版 API 兼容问题，而不是布林带公式不可计算 |
| `Squeeze Momentum Strategy` | 分析拒绝 | 缺 `cum` 旧别名；`f(...) => data = ...` 单行函数局部声明无法解析；还出现旧式 bool/int 比较拒绝，需按源版本核实 |
| James `gaussian_regression.ps` | 分析拒绝 | `matrix.sum(matrix, matrix)` 被当作单参数 float 汇总，随后 `.pinv()` 接收者类型错误；这不是缺少 `pinv` 函数本身 |
| Turtle 原策略 | 运行通过 | 本轮 25 个订单、12 笔闭合交易 |
| 三重 SuperTrend 原策略 | 运行通过 | 本轮 18 个订单、9 笔闭合交易 |
| James MA crossover / price channels | 运行通过 | 分别 17/8 与 13/6 个订单/闭合交易 |
| HYE VWAP mean reversion | 运行通过 | 本轮没有订单；通过不代表已验证交易触发 |
| HalfTrend / ALMA | 指标运行通过 | 指标构件仍需交易规则，不能直接计为完整 bot |
| 前轮 7 份 TradingView 热门转录 | 全部运行通过 | 包括 Squeeze **指标**、UT Bot **策略**；完整 Squeeze 策略的失败与指标通过并不矛盾 |

前轮原样本复核：everget 仍为 153/210 通过，另外 49 份分析拒绝、3 份缺时钟、
5 份缺请求输入；James 为 5/6 通过，1 份 Gaussian 回归拒绝。
因此升级安装号为 0.3.2 没有自动消除前轮的 58 个失败/缺输入项。

## 能力缺口的覆盖范围与修复价值

下表按**独立提取源码**统计诊断命中；一份源码可命中多行，数字不能相加。
数字表示观察到相应阻碍，不承诺修复这一项就能解锁全部命中脚本。

| 缺口/诊断集合 | 独立源码数 | 代表与处理重点 |
|---|---:|---|
| v1-v5 订单 API 缺 `when` | 292 | Bollinger breakout、3Commas/QFL 等；`exit/cancel/close` 等旧 API 条件应按版本降低到等价条件执行，v6 不应重新接纳已删除参数 |
| 已知旧技术/数学函数别名缺失 | 290 | `cum/roc/sar/sin` 等；先补版本门控别名。现代 `ta.cum/ta.roc/ta.sar/math.sin` 配对探针已经可以分析并运行，不能把别名失败说成公式能力缺失 |
| trailing 与固定退出触发不能组合 | 159 | Andean Scalping、Bollinger Bands Breakout、Classic Long Term Trend Following；需要实现竞争退出、优先级和订单保留语义，不能只放宽签名 |
| series 字符串订单 ID / from_entry 被限制为 simple | 72 | 网格、多腿分批、DCA、按序号管理订单；需要订单身份、替换、撤销与历史状态语义 |
| `if` 返回值中的 `else if` | 55 | Ichimoku pyramiding、分段信号与移动止损；独立 v6 最小语法探针重现 |
| 函数内集合修改限制 | 29 | Grid、BNB Burn Buyer、工具函数；限制依版本和调用形态而异。相同局部数组返回探针 v4 拒绝、v6 通过，v6 修改全局数组探针也通过，不能笼统称为“函数不能修改数组” |
| `behind_chart` 声明元数据 | 8 | 多个 everget 图形脚本；可定义宿主中立元数据与不渲染时的明确行为，属于低成本兼容项 |
| `matrix.sum` 的矩阵相加签名/返回值不兼容 | 1 | James Gaussian Process Regression；应返回矩阵，并审计内部把它用作求和汇总的代码，不能只删除第二参数 |
| 单行 UDF 局部变量声明 | 未单独全量计数 | Squeeze 原策略；最小 `f(x) => y = x * 2` 已重现解析拒绝 |
| 日期字符串子集 | 53 个运行失败 | BarUpDn、Adaptive Price Channel 等；`timestamp("2021-01-01T00:00:00")` 分析通过、运行拒绝。文档中的 `"2021-01-01"` 和数值日期探针通过，故不是所有 timestamp 都不可用 |

另有两类广泛命中需要更谨慎地逐例确认：

- 307 份复杂 `security/request.security` 表达式被当前子集拒绝。与运行阶段缺少
  请求数据不同，这是分析阶段的表达式能力/限制集合；覆盖持久或可变状态捕获、
  UDF 内请求、引用对象捕获等。先确认源 Pine 版本是否合法，再扩大确定性请求上下文。
- 339 份命中 `strategy.entry(long=...)` 旧方向写法；需逐版本验证旧签名、布尔方向
  与数字方向的含义。迁移脚本也可处理，不能把所有命中直接认定为有效 Pine 缺口。

官方语义核对：
[v6 迁移说明](https://www.tradingview.com/pine-script-docs/migration-guides/to-pine-version-6/)
明确区分旧版 `when` 与 v6 删除行为；
[矩阵文档](https://www.tradingview.com/pine-script-docs/language/matrices/)
定义 `matrix.sum` 为矩阵/矩阵或矩阵/标量加法；
[函数文档](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/)
与[条件结构](https://www.tradingview.com/pine-script-docs/language/conditional-structures/)
用于核对函数和返回值控制流；
[绘图概述](https://www.tradingview.com/pine-script-docs/visuals/overview/)
说明 `behind_chart` 的视觉含义。

## 哪些首先需要宿主、依赖或源码处理

- 356 份独立源码首先缺 request 输入。自身多周期或 Heikin Ashi 可由宿主提供因果
  聚合/变换数据；真实 BTC/SPY 指数、链上 hash rate、财报、国债不在单现货＋对应
  永续的数据范围，不能补假数据。提供第一项输入后仍可能遇到下一项问题。
- 61 份缺 import 的库源码。解释器已有库源码注入契约；缺 MLExtensions、Kernel
  等库首先是依赖获取/注入问题，整个依赖闭包是否兼容仍未验证。
- 89 份包含重复版本声明，134 份命中声明模式与交易指令不一致；这是源码/提取
  清理集合，不能直接当作合法 Pine 的功能缺口。
- Python、Rust、C++、C#、JS 和 RL 候选不属于 Pine 语言执行范围。87 个具名入口中
  11 个为 Pine 文件，另外 76 个入口需要原生/其他语言适配或另行移植。
  Hummingbot、GLFT、ABIDES、BSE、VeighNa、Freqtrade 与执行算法不应通过向核心
  加入市场数据服务、账户服务、撮合调度器等方式来接入。
- 永续实际保证金、持续挂单、双腿成交协调、盘口/逐笔输入和费率结算是宿主适配
  工作。解释器已有 short、limit、bracket 的能力，宿主仍需正确映射订单和反馈。

建议先做版本门控的旧别名与 `when`、表达式/单行函数语法、`behind_chart`；
随后做动态订单身份与完整退出组合；最后按可供数据的实际策略扩大 request 子集、
矩阵/复杂算法能力。矩阵缺口覆盖小但确实存在；做市、基差和资金费率机器人接入
仍应由宿主适配推进。

## 文件证据与复现

[逐文件结果、错误位置、哈希、分组与最小探针](STRATEGY_LIBRARY_GAP_AUDIT_20261010.json)
保存全部 7,576 个条目的摘要；每份保留前 5 条分析诊断及完整诊断条数。
完整诊断、提取副本、运行回执与输入补充复查保存在：
`E:\projects\pine-interpreter\target\strategy-gap-audit-20261010\`。
该目录属于 Git 忽略的本地证据，执行 target 清理前应先保留。

```powershell
cargo build -p pine-cli --release
python target/strategy-gap-audit-20261010/audit.py
python target/strategy-gap-audit-20261010/probes.py
python target/strategy-gap-audit-20261010/recheck_inputs.py
python target/strategy-gap-audit-20261010/summarize.py
```

审计脚本复用相同源码哈希的本轮本地回执；更改解释器或输入后必须另建证据目录
或禁用缓存，不能把本轮缓存误当新版本实测。运行证据没有独立 TradingView 数值或
broker 对照，没有真实 MarketForge 撮合、盈利、长期负载、所有输入组合资格。
