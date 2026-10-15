# 策略库兼容缺口修复与验证

状态：当前改动按 2026-10-15 收尾，最新结果见文末工作区收尾记录及对应机器回执。前二十六批记录保留为各自构建上的历史证据。

分支：`codex/strategy-library-compat-20261010`。基线：`28ecae3c645e9269e2099d27cc9abbb1f52ee9ea`。
当前工作区内容整理为本地提交，包含此前已有的资源审计与开发构建 profile 改动。下文各历史批次中的 Goal active、未提交及未推送描述均为当时状态。

## 实现范围

| 原缺口 | 本轮处理 | 验证边界 |
|---|---|---|
| 旧技术/数学别名 | 补 `cum/roc/sar/sin/cmo/dev/correlation/bbw/kc/cog/percentrank/lowestbars/highestbars/dmi` 等版本映射 | 官方 v1 基础别名与 v3 `cmo` 拒绝探针；未把 v4 专属函数放宽到所有旧版本 |
| 旧订单条件与方向 | `order/exit/cancel/cancel_all` 支持旧版 `when`；v4 `long` 映射方向 | 条件为假不下单、不撤单、不发外部账户意图；v6 已删除参数仍拒绝 |
| 动态订单身份 | `id/from_entry` 接受 series 字符串 | 按实际字符串管理挂单、替换和撤单 |
| 组合退出 | 固定止损/止盈与 trailing 同一保留数量、订单身份和 OCA 生命周期；绝对/相对参数按 v5/v6 选择 | 多空、部分数量、退出注释、撤销/替换、后续加仓、实时已观察价格；完整官方成交清单尚未比对 |
| 退出位置参数 | 源码第三参数映射 `qty`，不再误作 `stop`；内部规范化槽位保持稳定 | v4 第十四参数为 `when`；v5/v6 为 `comment_profit`。v5 具名 `when` 接受，22 个位置参数拒绝；官方探针已核实 |
| 可选退出参数 `na` | 数量与价格参数接受 Pine 的缺省占位 | 数量优先级和部分平仓回归 |
| 返回值语法 | 表达式 `else if`；单行函数局部声明与逗号语句 | 原脚本与最小值返回用例 |
| 函数局部数组修改 | v4 数组构造/修改；现代函数补 `insert/reverse/sort`；v4/v5/v6 函数内 `concat` | 拼接返回目标身份、自身拼接先快照；非法元素类型和全局标量重赋值仍拒绝 |
| `matrix.sum` | 矩阵相加或矩阵加标量，返回独立矩阵；混合 float 提升类型 | 形状、NA、输入不变、分配预算及 CLI/Python/WASM 输出回归 |
| 局部矩阵调用链 | 用已分析的接收者类型降低链式调用 | Gaussian 原源码与局部块作用域回归 |
| 日期字符串 | ISO 无时区/数字偏移、带空格数字日期、UTC/GMT 前缀形式 | 官方编译确认 `Z` 后缀拒绝，因此未把非法日期纳入支持 |
| 声明/绘图兼容 | `behind_chart` 宿主中立元数据、零初始资金、三角标签 | 不提供宿主绘图或实际账户服务 |
| 旧 bool/int 比较 | 按源版本处理；v4 混合比较不强制数值等价 | 官方 v4 `true == 1`、`false == 0` 都为假；v5 对照及 v6 拒绝边界 |
| 请求表达式 | requested timeframe 标量、不可变输入捕获、隔离的标量 UDF 局部状态 | 请求状态仅随提供者 bar 推进；可变外部捕获仍拒绝 |
| 具名请求参数 | `symbol/timeframe/expression` 按名称绑定，包括元组与低周期数组元组 | 缺参数、重复参数和未知参数拒绝；提供者数据输入仍显式要求 |

## 官方对照

使用已登录 TradingView 的未保存测试脚本。没有保存、发布或执行真实交易。

- [退出数量 v5 探针](evidence/strategy-library-20261010/exit-qty-v5.pine)：下单 2、退出第三参数 1，官方图上剩余仓位 1、闭合交易数 1。
- [组合退出探针](evidence/strategy-library-20261010/combined-exit.pine)：固定止损与 trailing 同时存在时，官方成交仍可由固定止损触发。
- [局部状态 v4 探针](evidence/strategy-library-20261010/stateful-v4.pine)：年线请求计数与月线图表计数独立。
- `mixed-matrix.png`：`matrix<int> + 0.5` 返回 float 矩阵，结果 2.5。
- `bool-equality-v4.png` 与日期截图记录版本语义和非法 `Z` 后缀。

这些是受控官方语义证据；不是全库、所有参数或全部历史交易与 TradingView 一致的证明。

主要规范：[策略语义](https://www.tradingview.com/pine-script-docs/concepts/strategies/)、
[v6 迁移](https://www.tradingview.com/pine-script-docs/migration-guides/to-pine-version-6/)、
[v5 退出接口](https://www.tradingview.com/pine-script-reference/v5/#fun_strategy.exit)、
[矩阵](https://www.tradingview.com/pine-script-docs/language/matrices/)。

## 语料与数据

沿用初次审计的 7,255 份独立规范化源码，原策略不作补丁。每次更换解释器都使用新的回执目录，避免只按源码 SHA 复用旧结果。

默认语料运行使用相同的 512 根合成 bar、默认输入及 8 秒超时。原基线为 release 构建；中间 debug 构建的超时不能直接认定为性能回退。

原策略另用 `I:\sys\下载\BINANCE_BTCUSDT, 1D (11).csv`：
3,280 根日线、完整 Volume，输入 SHA256 为
`5ea08056fb0b57a3aee69cef114d7131c8114efe83277d973065377683735a81`。
测试适配器将秒时间戳转成毫秒，并按 UTC 周一聚合 470 根周线；这是宿主侧测试输入，未放入解释器。

定向重放显式提供小数数量精度 8。为了核对声明为 USD 的原脚本，使用 USD 标记的受控价格上下文；未提供真实 USD/USDT 汇率，不据此做实际收益或官方收益比较。
Grid 另使用覆盖默认价格边界的合成 fixture；其成交不是 BTCUSDT 真实市场验证。

## 尚未资格化的范围

复杂请求已补静态/动态嵌套、不同选择器、元组函数、参数/输入/局部状态及循环值图。未确认合法性的剩余诊断仍包含外部可变捕获、引用对象和源码错误，不能把整个拒绝集合视作已修复或有效缺口。

缺少市场数据、时钟、汇率、账户输入和 import 库源码仍属于宿主/依赖契约；本轮没有在解释器中加入网络、数据库、调度器或 CandleScope 依赖。

`matrix.sum(matrix)` 旧实现的聚合语义不符合 Pine。依赖它的内部 read-chain 测试改用合法 `matrix.avg`；新增 `matrix_add.pine` 专门验证官方加法。用户脚本若依赖旧单参数扩展，需要改用合法聚合写法。

## 最终回执

前三批统计见 [机器回执](STRATEGY_LIBRARY_COMPAT_RESULTS_20261010.json)。相同 7,255 份规范化源码、512 根合成 bar、8 秒预算及 release 构建：

| 项目 | 基线 | 前三批修复后 |
|---|---:|---:|
| 可编译 | 4,624 | 5,379 |
| 运行通过 | 4,119 | 4,776 |
| 编译拒绝 | 2,631 | 1,876 |
| 缺请求数据 | 356 | 467 |
| 缺执行时钟 | 39 | 55 |
| 货币配置/换汇契约 | 28 | 44 |
| 日期字符串拒绝 | 53 | 3 |
| 其他运行拒绝 | 29 | 34 |

756 份新增编译通过、658 份新增运行通过。唯一一份原先通过、现在拒绝的脚本 SHA 为 `10564060ca4bb721e4001d636e65e38189b439527ef1e39deb2139c6c491191e`，原句 `strategy.exit("Sell", "Buy", bar_index[30])` 实际只指定数量，缺少退出价格；旧实现把第三参数误作 stop。此项是对官方位置参数的纠正，净增编译 755、运行 657。中间 debug 构建出现的三份超时在 release 复测均通过，最终没有超时。

第二批 `array.concat` 让 Pivot points 策略的原文件及 FMZ 提取版本新增编译通过；它们现在明确需要宿主提供 `4D` 请求数据，默认合成运行仍拒绝。两份源码 SHA 为 `2a01bf653d91742b260823e897c75600e8bf4d3bc1d6719ed4e098deb536ddeb` 和 `e1f6fc274b9dcc2b67dcb340bcf9b3583ccccbb9e905ab02b0a519ceb3b21786`。

宿主输入拒绝数量上升，主要因为更多源码越过编译阶段，并不意味着这些依赖应内置到解释器。

| 原策略（源码未修改） | 定向运行结果 |
|---|---|
| Grid Bot | 合成价格 fixture：81 个订单、38 笔平仓 |
| Ichimoku Pyramiding | 受控 OHLC 重放：4 个订单、2 笔平仓 |
| Bollinger Breakout | 受控 OHLC 重放：97 个订单、48 笔平仓 |
| Squeeze Momentum | 运行成功、2 条 plot、0 单；v4 原入场条件含不成立的 bool/int 混合比较，不能通过错误放宽解释器制造交易 |
| Gaussian | 运行成功；最终存活 2 条 polyline，100/31 个点全部有限；不是普通 plot，也不能用历史导出对象数量当存活图形数量 |

验证：第三批 Rust 全工作区 7,610 个测试、114 个测试组通过；clippy 全目标 `-D warnings`、fmt、结构门禁及 172 个 Python 脚本测试通过。592 个运行和 5 个 legacy 分析金样本跨 CLI/Python/WASM 登记通过。新构建 wheel 在独立 venv 中 831 个测试通过；实际 WASM 在 Node 中实例化、分析、执行、预编译/执行、组合宿主及 JS 异常烟测通过。WASM 先前 Windows `os error 5` 失败回执保留，后续重试成功，没有改安全设置或放宽测试。

第三批 release CLI SHA256：`a924afeb83c59d9688f8aef36d0961b28171a6f3626de5a30282fbfb91011431`。原五策略也用此构建重新执行，源码未修改、结果同上。

原 307 份复杂请求候选中，54 份现在编译通过（53 份等待宿主请求数据、1 份默认重放通过），253 份仍被拒绝；其中包含合法请求限制和非法源码，不能整体判成已完成。数组函数中的 `array.concat` 已在第二批补入并完成全部门禁和全库复测。goal 将继续处理尚未完成的请求兼容范围。

动态请求表达式扩展的完整门禁已经通过，包括新构建 wheel 的 829 项测试。第三批进一步实现静态请求的序列桥接，合并后的完整门禁、release 构建、全库复测及五个原策略重放均完成。

后续资格化已确认需要区分两个语义：v4/v5 静态请求独立计算内层调用，v6 默认动态请求会把内层请求置于外层上下文中。来源：[官方嵌套请求文档](https://www.tradingview.com/pine-script-docs/concepts/other-timeframes-and-data/#nested-requests)。不能通过统一放开“不同 selector”来实现两种行为。

本轮官方最小探针使用月线图、内层 `request.security("", "", timeframe.multiplier)`、外层 `request.security("BINANCE:BTCUSDT", "12M", inner)`：v5 `dynamic_requests=false` 输出 1/1，v6 默认动态输出 1/12。源码和截图分别为 `evidence/strategy-library-20261010/nested-static-v5.*` 与 `nested-dynamic-v6.*`，已清除临时图表实例。

v4 对照使用 `security(syminfo.tickerid, timeframe.period, timeframe.multiplier)` 的结果作为 `security(syminfo.tickerid, "12M", inner)` 的依赖，官方同样输出 1/1；证据为 `nested-static-v4.pine/png`。这类合法静态依赖已经加入实现、定向回归、完整门禁和原源码复测。

静态请求模式现在从声明及版本默认值传入 HIR，v4/v5 默认为静态，v6 默认为动态，并保留显式覆盖。运行时仅在存在静态请求依赖时保存主图输入历史，使用共享追加树供独立请求求值并按外层周期采样；不执行策略、输出或绘图语句。取消历史批次会去除未执行输入，增量扩展会失效依赖缓存，实时副本继承数据 feed 和 input override。profile 增加 `staticRequestChartBars` 和 `staticRequestChartBarCapacity`，使额外保留输入可观察。

新的官方时间探针 `nested-static-time-v5.pine/png` 在月线图上输出：inner/outer 当前值均为 `1790812800000`，outer[12] 为 `1733011200000`，time[12] 为 `1759276800000`。它证明历史年度请求取年末内层月线值、当前年度取可见最新值；用受控月份时间及实时年度 feed 复现的运行时回归逐项匹配这四个数值。另有 v4、外层 TA/内层局部计数状态、命名参数和内层 lookahead、增量扩展、tuple 依赖、取消批次及实时失败回滚回归通过。请求组 261 项回归、2,248 项语义 fixture、clippy 全工作区全目标均通过；之后增加的官方月份回归也通过。

代表源码 `54d35d2a0c3635570e3dfe8903033079e4d53b3782fc7198ba25e00865f42077`（Best Crypto DEMA）将 Heikin Ashi 请求作为高周期 DEMA 的依赖；`3e61439f908b08b59dda8735fc9dde4ba819e9ab33c127e238ffb4ee734bd8ba`（BTCUSDT-10M）将多个周期的请求结果作为其他周期 WMA 的依赖。这些形态需要进一步的官方编译/值对照和版本对应实现，不把它们归因于缺 OHLC 输入来回避解释器缺口。

第三批补齐数据的原源码重放：Best Crypto DEMA 使用 5,120 根受控 1m OHLC、宿主侧生成的 HA 流及 180m 流，运行成功；BTCUSDT-10M 使用同长度 1m 图及 D/3/5/8/10/11/12 流，运行成功，222 个订单、111 笔平仓、10 条 plot。两者均未修改源码，数据为合成受控输入，不作为真实收益或官方交易逐笔一致性证据。

第三批曾确认的核心缺口（第四批已修复）：相同 BTCUSDT-10M 原文件切换到受控 10m 主图后，即使已提供全部请求数据，仍因 `requested timeframe 11 must be an integer multiple of chart timeframe 10` 拒绝。这是运行时周期支持边界，需要官方非整倍数合并探针、实现及回归，不能归因于缺市场数据。回执在 `target/strategy-gap-final3-20261010/complex-request-results.json`；goal 保持 active。


## 第四批：非整倍数周期请求（完整门禁已通过）

官方未保存 v5 探针在 10 分钟图上接受 11 分钟请求。稳定导出的 300 根 bar 中，279 根历史 bar（剔除前 20 根预热和最后实时 bar）成为回归基准，每版比较 3,069 个值。v4/v5/v6 的受控等价用例已全部通过；官方原生运行版本为 v5。

- 取消较高周期必须为主图整倍数的限制；宿主能力报告改为 `sameOrLowerOrHigher`。所有周期仍须通过原有解析器。
- `gaps_on` 按每根主图 bar 的请求事件区间输出，避免只接受完全相等的开盘/收盘时间戳。月/周的前一周期边界使用日历计算。
- 静态嵌套请求读取已完成的根图表依赖值，避免非对齐周期误取尚未完成的低周期值；提供者的下一根开盘可以截短会话末尾请求 bar。
- 官方 [探针源码](evidence/strategy-library-20261010/uneven-export-v5.pine)、[逐 bar fixture](evidence/strategy-library-20261010/uneven-10m-official.csv)、[来源与哈希](evidence/strategy-library-20261010/uneven-export-v5.json) 可复核。初次导出尚未稳定的合并结果不参与验收。

这批修改尚未更新前三批全库统计。完整门禁已通过：7,611 个 Rust 测试、172 个脚本测试、新 wheel 的 833 个 Python 测试、实际 WASM/Node（含非整倍数请求 direct/prepared）以及 fmt/clippy/结构/宿主黄金值检查。第四批 release 的 BTCUSDT-10M 原源码在 10m 主图、512 根受控 bar 上运行成功，54 个订单、27 笔平仓、10 条 plot；Best Crypto DEMA 与 BTCUSDT-10M 原 1m 重放也仍通过，后者维持 222 单/111 笔平仓。历史导出对齐不等于实时请求、真实成交或全 Pine 语法已经全部验收。

第四批 CLI SHA256：`47fdc6aa7ef73ebc21179b8e74a154549ecec24fd88be75f833fd709c184a0eb`。回执：`target/strategy-gap-final4-20261011/complex-request-results.json`。源码 SHA 与输入生成方式已在回执记录；仍为受控合成输入，不是实际收益或官方逐笔交易一致性证明。

后续工作：旧版 `security` 的返回元组 UDF、现代 UDF 局部请求参数/别名等合法形态仍需官方及原源码核对。`1St Bar GAP+BkOut Screener v2 by RM` 的原源码因 tuple-returning UDF 请求被语义检查拒绝，不能视为缺行情数据或全部已完成。非对齐实时请求的完整官方值对照也尚未验收。

## 第五批：元组函数、参数请求与空单元格（完整检查及全库复测已通过）

已实现 v4 `security` 的元组返回 UDF 及逐元素类型传播；现代函数内的参数、不可变别名请求按当前调用重建；空单元格可由 `table.cell_set_*` 初始化，后续 `table.cell` 会覆写属性。参数来源独立于普通变量初始化记录，保留外部可变捕获、递归和绘图副作用的拒绝边界。

官方 v4 元组函数探针执行通过；v3 相同 `security` 形式被拒绝。官方 v5 参数别名请求在月线图请求 12M，显示 22；v5/v6 仅使用 setter 的空单元格显示成功。见 [探针回执](evidence/strategy-library-20261010/function-request-table-probes.json) 和 [官方 v4 元组支持说明](https://www.tradingview.com/blog/en/tuple-support-for-the-security-function-in-pine-script-18316/)。临时指标已移除，原图表策略和月线设置保留。

最小语义/执行回归已通过，完整门禁发现的类型缓存和旧版参数来源传播回归已修复，正在重新资格化。原 Gap Screener 源码已编译通过，但新 release 的完整运行、全库统计和最终完整门禁尚未完成；上一节第四批结论及第三批全库统计仍分别对应其原回执。goal 保持 active。

第五批补充：参数别名历史值回归同时检查 HIR 的浮点类型；函数内未声明类型的不可变标量按每次调用专门化，避免后一次常量调用污染前一次浮点调用。最新门禁日志已完成 7,621 个 Rust 测试、172 个脚本测试、836 个新 wheel Python 测试及实际 WASM/Node 检查；release 和全库复测仍在进行。

下一批官方证据：[命名嵌套请求回执](evidence/strategy-library-20261010/named-nested-probes.json) 确认 v5 静态与 v6 动态模式分别显示 1/1 与 1/12，包含 gaps/lookahead 参数。当前动态表达式准入仍只接受三个位置参数，这一缺口尚未实现；临时指标已移除。

第五批最终回执：release CLI `38356486ba629ad731501cf9da468412a34283b59207c1cbd820d73398dbc8cd`。同一 7,255 份源文件逐项校验哈希，编译通过 5,417 份（相较第三批增加 38），默认 512 bar 重放通过 4,778 份（增加 2）；此前通过的编译和重放均无退步。501 份缺请求输入、55 份缺执行时钟、45 份货币配置/转换、35 份其他运行拒绝、3 份日期格式限制；1,838 份分析拒绝仍需区分源码非法与解释器缺口。

原 Gap Screener 的 release 重放使用 720 根受控 1m 主图和 20 路股票流，源码哈希不变，4 个带尾随空格的股票默认设置通过 input override 去掉空格以匹配宿主数据键；最终 1 张活动表、4 个填充单元格、2 条 plot。五个重点原策略及三个复杂请求场景全部保持通过，订单及绘图结果与此前一致。数据和设置边界不变，不据此声称真实股票会话、收益或官方逐笔交易一致性。明细：`target/strategy-gap-final5-20261011/qualification.json`；后续命名动态嵌套请求及其他合法候选继续处理，goal 保持 active。

## 第六批：命名动态嵌套请求（完整检查及全库复测已通过）

先将内层 request.security 的 symbol/timeframe/expression 绑定到规范槽位，再执行静态或动态表达式检查，保留可选 gaps/lookahead 等策略。语义和运行最小回归已通过，官方 v6 探针和此前四处拒绝的 `f9538ed89f651a1533fa3b58c51244c01ab02eadd1f0e3b1ebb6bcbe2c0d5d4b` BTCUSDT 原文件均编译通过，源码未修改。完整检查、release 原策略运行和全库统计仍在进行；第五批 release 回执保持独立。

第六批最终回执：release CLI `79ec5d95b83503f53f6aec3002c95457bd5b0a8a3ca266be6ade264cbd4d97ac`，完整检查通过 7,623 个 Rust 测试、172 个脚本测试、840 个新 wheel Python 测试及实际 WASM/Node。7,255 份原文件重新验证，编译通过 5,419 份（比第五批增加 2），默认重放通过 4,778 份，此前通过项无退步。新增两份等待宿主数据（共 503）；全库原始回执与第五批独立保存。

新增 BTCUSDT 命名 v6 原文件的受控 replay 已通过，补齐其原本需要的 60m 行情后，1m/10m 主图分别输出 88/24 个订单、44/12 笔平仓，均 13 条 plot。其余五个重点策略、三场旧复杂请求、Gap Screener 仍通过。源码哈希不变，仍不作为真实收益、官方完整实时或逐笔交易一致性证明。

下一处官方确认的缺口：[v5 源输入对照](evidence/strategy-library-20261010/source-input-request-v5.json) 在月线主图显示 hl2=83806.78；input(hl2)、input.source(hl2) 的年度请求与直接年度 hl2 均为 77862.34。源码与截图已保存，临时指标已移除。第七批正在补 source input 的请求表达式准入并验证 input override；尚未完整资格化。legacy iff 请求依赖也发现独立于等价 ternary 的拒绝，待单独核对。goal 保持 active。

## 第七批：请求源输入（完整检查及全库复测已通过）

Generic input(hl2) 与 input.source(hl2) 现在按 defval 槽位识别并重建请求依赖，不将主图当前值当作请求值；命名 defval 不受 title/group 参数顺序影响。v5/v6 定向回归同时验证主图/提供者 OHLC 不同及 high override，实际 Python/WASM direct/prepared 也验证这两个输入路径。完整检查：7,624 个 Rust 测试、172 个脚本测试、844 个新 wheel Python 测试以及 fmt/clippy/结构/宿主黄金值/实际 WASM 检查通过。

release CLI `097c8e883085e3bc00072723690b78772aebeb12dff8b5b603ada86ac8dccd1a` 的 7,255 份原文件全库复测，5,466 份编译通过（比第六批增加 47），4,796 份默认重放通过（增加 18），此前通过项无退步。530 份缺请求输入、55 份缺执行时钟、47 份货币配置/转换、35 份其他运行拒绝、3 份日期格式限制，1,789 份仍分析拒绝。Kitchen 与 Bullish Divergence 原文件在 5,120 根受控 1m 主图以及 60/240/D 流等必要数据下运行通过，分别 6/3 条 plot、无订单；既有重点原策略及复杂请求全部保持通过。受控输入不构成收益或官方逐笔交易验收。

后续官方证据已准备：[v4 iff 独立状态](evidence/strategy-library-20261010/legacy-iff-request-v4.json) 显示主图 iff/ternary 计数均为 111，请求的前一年计数均为 9；[v5 for/while 请求](evidence/strategy-library-20261010/loop-request-v5.json) 的两个循环结果与直接历史表达式在主图和年度请求中分别一致。两个临时指标都已移除，下一批将实现这些合法请求依赖并验证运行预算与捕获边界；goal 保持 active。

## 第八批：旧版 iff 与函数内循环请求（完整检查及全库复测已通过）

已补 legacy eager iff、int/float/bool 标量转换，以及 UDF 内 for/while 语句和表达式循环请求准入，仍拒绝外部可变值和绘图副作用。新回归发现并修复历史保留推断缺陷：循环中不断更新的索引不能永远按循环前的初始常量计算，需在推断循环条件/重复体前失效相关常量绑定。修复后主图和提供者上下文的参数历史、while、自行更新索引的 for 语句/表达式、eager 分支及循环预算限制均通过；147 项请求相关运行回归及语义定向回归通过。完整检查正在进行，新的 release、原策略和全库结果尚未验收。

第八批最终回执：完整检查通过 7,629 个 Rust 测试、172 个脚本测试、847 个新 wheel Python 测试及实际 WASM/Node、fmt/clippy/结构/宿主黄金值检查。release CLI `10abcc6b4e1e923a9300676f7fb8884da11657a5eb85fa1987fd463cfd0dcfd1` 重新核验同一 7,255 份源码，5,479 份编译通过（增加 13），4,801 份默认重放通过（增加 5），此前通过项无退步。538 份缺请求数据、55 份缺时钟、47 份货币配置/转换、35 份其他运行拒绝、3 份日期限制，1,776 份分析拒绝仍需逐项判定。

Combo EMA、Hurst 和 Follow Line 原源码在受控输入下运行通过；Follow Line 初次因缺 15m 数据拒绝，保留回执并补宿主测试流后通过。其余原策略重放仍通过。回执为 `target/strategy-gap-final8-20261011/qualification.json`。这些结果不构成真实行情、收益或官方逐笔成交验收。

## 第九批：标量输入、旧式 VWAP 与同周期函数（完整检查及全库复测已通过）

v4 同周期函数调用、请求内直接 generic input、旧式 vwap，以及 v5 fixnan/input.int 请求均经官方未保存探针核实；[官方回执](evidence/strategy-library-20261010/scalar-request-probes.json)记录源码、截图和哈希。同周期全局可变值被官方拒绝，因此准入统一走捕获验证，纠正旧路径对直接 Identifier 的过宽接受。实现保留版本检查、调用点 input override 和提供者独立 fixnan 状态。定向运行/语义/Python/WASM 用例已添加，Rust 定向检查通过；完整检查、release 与全库原源码复测尚未完成。

同周期函数回归最初误按无预热 EMA 数值断言，HIR 确认选中分支正确；改用非线性价格并对比直接 EMA/SMA，验证函数分支和请求上下文，不将 EMA 的完整官方预热语义视为已验收。

第九批最终回执：7,633 个 Rust 测试、172 个脚本测试、850 个新 wheel Python 测试、实际 WASM/Node 与完整检查通过。release CLI `86cdfec568f58a9df8ec4de6b83efb4bcf7e970ea09d1850b738fd5a8bae9d1d` 对同一 7,255 份源文件重新验证，5,487 份编译通过（增加 8），4,802 份默认重放通过（增加 1），此前通过项无退步。545 份缺请求输入、55 份缺时钟、47 份货币配置/转换、35 份其他运行拒绝、3 份日期限制，1,768 份分析拒绝。七个新增代表原文件在明确的受控宿主流下均运行通过，其中 PPS 112 单/74 笔平仓，Coinrule 2,532 单/2,316 笔平仓，Kyrie 125 单/124 笔平仓；数量只描述受控运行，不证明策略收益或官方逐笔成交。周线流仅覆盖当前受控周，VWAP 相关策略的长期/有限值和真实股票/VIX/CME 行情仍未验收。

第十批官方 [值图探针](evidence/strategy-library-20261010/request-value-graph-v5.json) 接受同名局部 value/全局不可变 value、同一请求包装函数的两个非递归调用，以及 math.pi、color.new、month/dayofmonth/year(time_close)。当前解释器的对应最小用例除单次包装调用外仍拒绝；这些是待修复核心准入缺口。临时指标已移除并经新 DOM 确认原策略/月线保留，测试页已关闭。

## 第十批：请求值图与作用域（资格化进行中）

全局不可变别名现在使用排除函数/方法局部重赋值的全局预扫描；全局控制流及请求后面的全局修改仍拒绝。局部绑定的修改检查限于活动函数体。同一静态请求包装函数的非递归调用保留经调用方验证的参数集合，正常调用分析继续拒绝内部可变 request 表达式。请求值图补 math 常量、纯颜色构造/取分量、日历调用和提供者 time_close。276 项请求运行回归、7 项请求参数语义回归和完整官方探针编译均通过；实际 Python/WASM 对应回归已加入，完整检查、release、原源码和全库复测未完成。此批不宣称解决全部局部块同名捕获、非 UTC 交易所日历或外部符号元数据。


## 第十批完整回执及第十一批待补项

第十批 release SHA256 `802f92eeac64d2b1080c94696cee5f5138d41e74ef0dd62c1ce7c30759a01e83`，完整门禁终止码 0：7,638 个 Rust 测试、172 个脚本测试、855 个新 wheel Python 测试以及实际 WASM Node 检查通过。相同 7,255 份源码、512 根合成 bar、8 秒限制重新执行，无编译或运行通过集合回退。完整回执 `target/strategy-gap-final10-20261011/qualification.json`。

23 个受控原策略运行中 21 个通过，5 个命名原策略及 Gap Screener 通过。OTTs 的空 symbol 输入由宿主适配器按继承图表处理后，进一步暴露 10,000,000 步预算超限；Divergence 进一步暴露函数内 input 禁止。两者保留原始失败回执，不列为整策略运行通过。OpenClose 提供显式执行时钟后通过；时钟仍由宿主提供。

第十一批官方已核实 v4 函数内 table.new/cell/cell_set_text 合法；v5 函数内 input.int/input.source 合法，同一函数调用两次的设置面板仅有两个输入项，共享源码声明身份。实现及跨绑定验证待完成。


## 第十一批实现与验证进行中

函数内 input 已按官方 v3/v4/v5/v6 探针放行，同一个源码声明在两次 UDF 内联调用中共享 call site 与覆盖参数，metadata 去重；不同声明仍独立。导出 library 内 input 与非法 metadata 仍拒绝。v4 table 操作按普通函数版本与签名校验进入原有表格运行时。输入默认值、覆盖 source、高低价和表格返回身份回归通过。checkpoint 扩展仅处理确定性 scalar/input 操作，与强制全量重算逐次比较形成更新输出。

debug 原源码受控重放：v4 表格策略 14 个订单/7 个闭合交易/2 个 plot；Divergence 14/7/13，通过。OTTs 首次提供者整段预计算仍超预算，未宣称整策略运行通过；下一步限制端点无关表达式预计算到图表时点。完整门禁第一次仅 matrix notes golden 不同步失败，已按实际生成值同步；第二次验证进行中，release 与全库复测待完成。

历史索引核对：官方 v5 输入 n=19 的 `close[n/2+1]` 编译并显示 87648.22；扩展 offset/cast 控制探针因官方服务 Failed to fetch 两次尚未完成，不据此宣称除法截断。当前已记录兼容差异，具体转换规则继续核对。


## 第十一批完整回执及第十二批确认项

第十一批 release SHA256 `deb890d86f2095025ce021bfb8ed6eab53dfe31c73b35c70462966956c65d7b3`。相同 7,255 份源码重新执行，5,507 份编译通过（增加 13）、4,812 份默认重放通过（增加 9），此前通过集合无回退。5 个命名策略、Gap Screener 及 24 个复杂原策略中的 23 个受控运行通过；仅 OTTs 保留首次请求预算缺口。555 份缺请求输入、55 份缺时钟、47 份货币配置/转换、35 份其他运行拒绝、3 份日期限制、1,748 份分析拒绝仍按原始回执保留。

验证采用完整门禁与修正后独立门禁的合并证据：第三次完整检查中的 7,643 个 Rust 测试、116 个测试套件、172 个脚本测试，以及 fmt/clippy/结构/宿主黄金值通过；随后修正 WASM 测试请求 payload 为 JSON 对象，实际 Node 通过并观察到退出码 0；新 wheel 构建安装后 859 个 Python 测试通过，另一次同一新 wheel 的 pytest 取得退出码 0。没有声称某次完整 verify.ps1 单次全部退出 0。完整回执 `target/strategy-gap-final11-20261011/qualification.json`。

官方新探针已确认 v5/v6 的 input int 除法仍保留 10.5；历史索引另行截断转换，v6 的 close[2.8] 等于 close[2] 而非 close[3]，close[-0.8] 等于 close[int(-0.8)]，动态 series float 索引也编译运行通过。此前扩展探针的 Failed to fetch 是服务错误，已由新的成功控制探针补齐，不能用于推断语义拒绝。第十二批核心实现与原策略复测仍待完成。


## 第十二批实现及官方边界（完整资格化待完成）

历史索引在 v2/v5/v6 官方未保存探针中接受数值浮点，转换向零截断，再检查负数；v2 sqrt(8) 和 v6 的常量/输入/series float 已与整数控制项核对。分析、常量保留推断和运行时均已补齐，bool/string/reference 仍拒绝，负值及超范围数值保留错误。跨版本及 UDF 不同调用参数回归通过。

请求 checkpoint 限制到当前图表 nominal close 的提供者前缀，保留一条后继用于不规则收盘合并。稳定表达式不将部分结果存入完整 request cache，security_lower_tf 历史路径也逐次扩展；依赖数据末端、不可增量表达式和 calc_bars_count 保留完整求值路径。首次 12 个 5,120-bar 流在原 10,000,000 步预算下运行通过；不规则收盘、gaps/lookahead 与完整提供者求值逐 bar 比对通过。运行时完整检查首次发现两个旧缓存结构断言和一项 lower_tf 部分缓存回归，已修复；218 项 security 相关回归通过。profile 的 request cache entries/contexts/values/capacity 现在按未重复的完整缓存及 checkpoint 前缀计数，终端样本需重放而不持久计入前缀，未宣称这几个字段涵盖全部 checkpoint 内存。

官方日期回执确认 timestamp(999999,12,31,23,59)=31494784780740000、Jan 19 2024 00:00:01=1705622401000、2021-01-13:05:00=1610514000000、01 Jan 2023 00:00 -3000=1672639200000。固定时区数值 timestamp 改为 checked Gregorian i128 中间值再转 i64 毫秒；正常范围及溢出字段与 Chrono 控制等价，IANA 超出库范围继续明确拒绝。字符串解析补英文月份在前、ISO 日期与时间之间冒号、dateString 四位数扩展偏移，数字 timezone 参数范围保持原样。

0000-2400、0000-2400:1234567、2400-1600 经官方接受，2400 解析为午夜别名，运行关闭时点回归通过。time(30S)/time(S) open 查询可用宿主给定的图表时间计算；较低周期 time_close 尚未资格化，官方月线探针显示其可用数据粒度退化为一日收盘，不能直接以 nominal 30 秒闭合替代。临时探针已删除、原 CA 策略及月线经新 DOM 确认保留，测试页关闭。

官方证据为 docs/evidence/strategy-library-20261010/float-history-oracle.json 与 date-session-oracle.json。新 Python/WASM 对应回归已加入，完整门禁、新 release、原源码与全库复测尚未完成；goal 继续 active。


## 第十三批：旧版函数和源参数绑定（完整资格化进行中）

本批生产代码已冻结，针对性 runtime 回归 23 项和 legacy frontend 回归 100 项通过。完整 `verify.ps1`、新 release、同一 7,255 份源码的全库重跑和独立原策略验证进行中，尚不复用第十二批资格化结论。

- 增加版本化的 `percentile_nearest_rank/percentile_linear_interpolation/swma/tsi/tan/wpr/supertrend/tonumber/random` 映射；`cos` 已有映射的下界补到 v1。v3 的 `wpr/supertrend/tonumber/random` 官方拒绝，保持隔离。
- v1-v4 `input` 绑定重复参数：位置参数优先，同名具名参数取最后一个；未采用的参数仍检查类型。官方 v4 对不同默认值、标题、分组和非法重复默认类型都有独立回执。
- v4 起允许静态输入出现在 UDF 实参表达式及另一 UDF 体内；输入参数中的其他副作用仍拒绝，保留输入声明的源 callsite 和共享 override 身份。v3 直接内联输入仍拒绝。
- v4 数字 entry 方向通过已有数字转 bool 降低：1 多头、0 空头，兼容已有 bool 方向。v5/v6 不放宽。
- `strategy.close` 按源版本绑定位置参数，而非误用内部槽位：v1-v4 的六个源槽位、v5 的八个槽位、v6 移除 when。可选数量的 na 占位按 qty/qty_percent/full 优先级选择，内部运行时槽位稳定。
- 立即成交测试保留成交价和成交 bar 的验证；未启用 calc_on_order_fills 时，同次执行的 plot 先于立即成交，文档不再声称后续同次语句已经观察到成交。
- v3 NA 后续赋值最小例子已被基线支持。不把缺别名引发的连锁 NA 类型诊断扩大成新的类型系统缺口。

官方原始探针及截图见 `docs/evidence/strategy-library-20261010/legacy-binding-oracle.json`。不据此宣称完整指标逐值、原版逐交易或收益与 TradingView 一致。

## 第十三批完整回执及第十四批确认项

第十三批 release SHA256 `de43ce1eee00e7783757526f94f617acb58b6118abf332a99efc165dc037d9c0`。完整门禁退出码 0：7,662 个 Rust 测试、116 个套件、172 个脚本测试、877 个新 wheel Python 测试和实际 WASM/Node 通过。相同原源码编译 5,824、默认重放 5,107，通过集合相对第十二批无回退。24 份复杂请求、5 份重点原策略与 Gap Screener 都通过。IQR、NRVGI、内联输入和 DPO 长历史日线重放通过；3 RSI MA 的日线重放遇到无效数量，Low Scanner 的长度与 options 拒绝进一步经官方确认，未计整策略成功。

第十四批官方最小探针确认 v4 具名 input.resolution options 合法；v4/v5/v6 整数运算产生的分数/产品别名可用于 EMA 长度，普通值保留 9.5/19，而最终 EMA 长度截断到整数。显式 input float 与浮点操作数仍拒绝。实现沿整数运算及别名验证来源，保留 simple 限制、显式浮点及 series 拒绝；运行时已有的最终长度转换保持不变。resolution options 的具名扩展保留原有位置 metadata 槽位，未扩大未经官方验证的位置参数形式。26 项策略兼容运行回归与 100 项 legacy frontend 回归通过；完整验证进行中。

## 第十四批交付回执（历史）

最终 release SHA256：`6dd4f6bd833076bb4ae3f269815a19d3d40410a90d07d6c1bb5147ad7fdff543`。完整 `verify.ps1` 单次退出码 0：7,665 个 Rust 测试、116 个测试套件、172 个脚本测试、881 个新 wheel Python 测试及实际 WASM/Node 均通过；fmt、clippy 全目标、结构与宿主黄金值门禁通过。旧测试对 UDF 体内静态 input 的过时拒绝断言按官方证据更新，嵌套副作用拒绝用例保留。

| 相同 7,255 份源码 | 初次审计 | 最终 release |
|---|---:|---:|
| 编译通过 | 4,624 | 5,850 |
| 默认输入重放通过 | 4,119 | 5,126 |
| 分析拒绝 | 2,631 | 1,405 |
| 缺请求数据 | 356 | 595 |
| 缺执行时钟 | 39 | 64 |
| 币种配置/转换 | 28 | 47 |
| 日期格式拒绝 | 53 | 0 |
| 其他运行拒绝 | 29 | 18 |

净增 1,226 份编译通过、1,007 份默认重放通过。本批相对第十三批增加 26 份编译、19 份默认重放，此前通过集合无回退。与初次审计相比，一份原先误接受的退出源码已按官方 qty 槽位纠正并拒绝，原因及 SHA 见前三批记录。

25 场复杂请求原策略重放（23 份独立源码，含不同主图周期）、5 份重点原策略和 Gap Screener 均以本 release 独立重跑通过。OTTs 在原 10,000,000 步预算、5,120 根受控 bar 和 12 个提供者流下通过；没有提高预算来隐藏首轮成本。Gaussian 最终存活两条 polyline，100/31 个点均有限；Squeeze 原 v4 条件仍无订单，未制造交易。新增定向原源码仍有失败，逐项见机器回执，不列为通过。

Low Scanner 原文件在 5m 主图、1,024 根受控 bar 及 60m 提供者输入下运行通过，29 单/14 笔平仓；只给日线的独立测试明确缺 60m 输入。DPO 日历报表在充足日线历史下通过，53 单/36 笔平仓，短合成历史的空数组拒绝不通过篡改循环处理。3 RSI MA 原文件的长日线重放仍遇到无效 qty (NaN)，此数量/账户路径未作官方完整对照，不列为该输入下整策略通过，也不直接认定为源码错误。

源码哈希逐份保持不变。全库默认测试固定 512 根合成 bar、8 秒预算；具名策略另使用下载的日线或明确的合成宿主流。运行通过不代表盈利、实时全部资格化或全策略与 TradingView 逐笔一致。

剩余 analysis/runtime 拒绝仍需逐源码判定：缺宿主数据、时钟、库源码、货币契约；重复版本/声明模式和非法时间参数；历史不足导致的空数组；明确的资源上限。较低周期 time_close 的数据可用性语义单独保留未支持/未资格化边界。本轮关闭的是审计逐项确认并验证的解释器缺口，不是全部 Pine 语言或全库每个拒绝项。

完整机器回执：`docs/STRATEGY_LIBRARY_COMPAT_RESULTS_20261010.json` 的 latest_qualification 与 original_audit_final_coverage；原始运行回执：`target/strategy-gap-final14-20261011/`。TradingView 未保存探针已清理，原 CA 策略/月线保留。未提交、未推送，原有资源审计/profile 改动保留。

## 第十五批资格化回执及后续 Goal

本批修复旧版 `strategy.order` boolean/numeric 方向与 `long` 名称，以及 `table.clear` 独立省略结束坐标的默认行为。v5/v6 的订单方向类型约束、`table.merge_cells` 必填坐标、错误坐标的运行拒绝保持有效。没有引入具体宿主依赖。

Release SHA256：`127dbd5dde011119ffef5fe5a8db48650c49cafa26950893bf197cff4f2a9bf3`。完整门禁退出码 0：7,667 项 Rust、116 个套件、887 项新 wheel Python、172 项脚本测试、实际 WASM/Node、fmt/clippy/结构及宿主黄金值检查通过。首次签名文档失败日志保留，修正文档后完整重跑通过。

同一批 7,255 份独立源码、512 根默认合成 bar、8 秒预算：5,874 份编译、5,142 份默认运行通过，相对第十四批增加 24/16，此前通过集合无回退。原始 7,576 个路径哈希逐份核验保持不变。相对初次审计净增 1,250 份编译、1,023 份默认重放；历史上误接受的无触发条件 `strategy.exit` 源码拒绝记录继续保留。

26 场复杂请求重放（24 份独立源码）、5 份重点源码和 Gap Screener 全部以本 release 重跑通过。7-RSI 保持原源码，使用七个精确请求周期输入、明确的 `1D` 提供者及 96 根独立合成日线预热历史，在原 10,000,000 步预算下完成 15 单/11 笔平仓。首次缺少精确 `1D` 输入的失败保留，修正宿主测试输入后复跑；没有改变源码或用预算放大掩盖失败。表格原源码 Accurate BUY SELL 的默认坐标拒绝消失，但仍有 earnings/dividends/splits、局部 atr 和分支类型拒绝，未计整策略通过。

官方原始 3 RSI MA 日线也报负权益无法下单（native bar 995，本地受控输入 bar 950），因此确认其失败类别是账户/数据路径结果；不宣称完整逐笔和报错 bar 一致，也不把合法源码直接判成非法。证据见 `rsi3-native-negative-equity.{png,json}`。

低周期 `time_close` 已增加日线、周线、月线、5m、会话和偏移官方探针：UTC crypto 日/周/月线上低周期关闭时钟具有一天粒度，5m 上为请求周期；但下一根 30S 的偏移仍遵循 30S 的位置，关闭粒度不能直接替换请求周期。此实现缺口仍保留，Goal 继续；非 UTC 交易所会话尚未资格化。无 else 的 if 表达式、循环末尾赋值返回值、默认下单数量零的候选已保存，等待官方核实后再实现。

机器回执：`docs/STRATEGY_LIBRARY_COMPAT_RESULTS_20261010.json` 的 latest_qualification/fifteenth_batch；完整日志和原源码结果在 `target/strategy-gap-final15-20261011/`。当前只有本批修复被资格化，剩余拒绝项仍按源码或宿主条件逐项审查。临时 TradingView 脚本和标签已清理，原 CA 策略/月线保留。未提交、未推送。

## 第十六批资格化回执及剩余时钟契约

本批补齐表达式块末尾标量声明/赋值的返回值（if、switch、for、for…in、while，包括函数末尾循环），执行副作用一次，并保留分支选择、break/continue 和零次迭代行为。if 表达式允许缺少 else；数值结果保留 na，v4/v5 布尔保留 na，v6 布尔缺省为 false。默认下单数量允许 0；正显式 qty 仍能交易，负默认数量仍拒绝。类型查询拆为独立块返回值模块，结构上限保持不变，无具体宿主依赖。

Release SHA256：`824ee024252157e65fa4854cb55b7e1cf69af12df7c13e7f0123bd7d5f3d98aa`。完整门禁观察到退出码 0：7,672 项 Rust / 116 个套件、893 项新 wheel Python、172 项脚本测试、实际 WASM/Node direct/prepared、fmt/clippy/结构及宿主黄金值检查通过。旧单元测试和 12 份旧夹具曾把合法的末尾赋值当作错误；夹具源码未改，已更新测试预期并添加运行值控制。首次、第二次失败和矩阵黄金值失败日志均保留。矩阵最终仅修改本轮 6 项能力说明；schema、fixture 集合和无关元数据保持原值。

同一批 7,255 份独立原源码、512 根默认合成 bar、8 秒预算：5,924 份编译、5,185 份默认运行通过，相对第十五批增加 50/43，此前通过集合无回退。7,576 个原始路径哈希保持不变。26 场复杂请求重放、5 份重点源码和 Gap Screener 用同一 release 重跑；新候选 3C QFL、BBPB、888 BOT 的实际分析/重放结果另列 additional-original-results，不以消除一个诊断代替整策略通过。

官方最小语言探针和本地结果对照：循环函数 6、if 赋值 7、无 else 数值 na、循环赋值 10；v6 动态 false 条件的布尔缺省为 false；v4 percent_of_equity 默认量 0 合法且显式 qty=1 建仓。另有常量-only 布尔官方观测异常（native 1/1/0，本地值见回执），保留待核验，不宣称此处完整常量折叠一致。

低周期 time_close 仍未实现。新增 AAPL D 官方对照确认 chart/lower close 均为开盘后 23,400,000ms，前一请求 1m close 为当前开盘前 63,000,000ms。此前 BTC UTC D/W/M 与会话/偏移证据保留；统一 UTC 午夜舍入不足以覆盖股票交易日历。后续须接收宿主提供的品种/周期开闭盘边界，由核心执行 Pine 偏移和会话语义。设计边界和必测项见 `docs/evidence/strategy-library-20261010/lower-close-calendar-contract-16.md`，该文不计已交付能力。3 RSI MA 的负权益官方失败分类沿用第十五批证据，仍未宣称逐笔与报错 bar 一致。

机器回执：`docs/STRATEGY_LIBRARY_COMPAT_RESULTS_20261010.json` 的 latest_qualification/sixteenth_batch；完整日志、哈希冻结核验、同库结果和剩余候选在 `target/strategy-gap-final16-20261011/`。临时 TradingView 指标/策略和标签已清理，原 CA 策略/月线保留。未提交、未推送，Goal 继续 active。

## 第十七批旧版命名空间及连续 UTC 低周期时钟

v3/v4 普通 exact 函数别名与全局变量、函数参数、局部标量变量分别解析；同名用户函数仍优先，值别名仍允许词法变量遮盖。888 BOT 的同名 macd 变量、Accurate BUY SELL 的同名 atr 变量不再遮住相应函数。v4 裸 pvt 映射到已有 ta.pvt，保留历史状态；官方 v3 的 pvt 拒绝证据限制其版本范围，v5/v6 也不启用旧别名。旧的 3 项词法遮蔽测试预期据官方证据修正，原夹具源码保持不变。

低周期 time_close 的连续 UTC 日内 profile 已实现：D/W/M 图表按选定请求 bar 所在 UTC 日收盘；日内图表按请求日内周期收盘。先执行图表偏移，再执行请求周期偏移，随后选日边界、应用显式会话截断；保留 500 根未来引用限制。未扩大低周期 D/W 日历查询的原有拒绝范围。未推断品种交易时段；AAPL 6.5 小时和假日/DST 日历仍须后续宿主中立输入契约，不能把此 UTC 进展计作股票日历实现。

Release SHA256：`e09fbc35efb164ef5f480da6192b3aa52eec35c8cd880f0b694f2559651d2465`。完整门禁退出码 0：7,677 项 Rust / 116 个套件、900 项新 wheel Python、172 项脚本测试，以及实际 WASM/Node direct/prepared、fmt/clippy/结构、宿主黄金值检查通过。新增 5 项 Rust 和 7 项 Python 控制，WASM 同步覆盖别名与 UTC 时钟；定向 35 项策略兼容和 100 项旧前端测试通过。矩阵仅变更 4 项能力说明，其他元数据与源码夹具未改。早期 catalog 顺序校验失败、范围收窄时取消的门禁日志均保留，最终生产文件哈希冻结核验通过。

相同 7,255 份独立源码、512 根默认合成 bar、8 秒预算：编译 6,003、默认运行 5,243；相对第十六批新增 79/58，此前通过集合无回退，7,576 个原始路径哈希不变。26 场复杂请求、5 份重点源码、Gap Screener 和额外原源码重放均使用该 release，新候选的实际成功或独立阻碍见 additional-original-results。BBPB 和 888 BOT 原样补充受控 2020 年 1 分钟 OHLC、请求流及必要执行时钟后均完整运行：BBPB 10 笔订单、9 笔平仓；888 BOT 11 个绘图、0 笔订单，未验证交易触发。回执见 unblocked-original-results，不以最小语言探针代替整策略通过。

官方 v4 全局 macd/atr 变量与内建函数共存、v3 参数与局部变量同名函数共存均编译并运行；v3 pvt 负控拒绝。7 场 UTC 最小时钟对照使用此前原样保存的 Pine 探针重放，通过原始开闭盘差值及独立偏移/会话值核验。原始市场值仅作为官方语言可执行证据，不宣称这些移动市场值的全历史数值一致。股票日历、native unavailable-intrabar fallback、常量-only 布尔官方观测和 3C QFL 的独立默认参数/循环类型候选继续列入待办。

回执：`target/strategy-gap-final17-20261011/`；机器结果 latest_qualification/seventeenth_batch。本轮临时 TradingView 指标及 agent 标签已清理，原 CA 策略/月线保留。未提交、未推送，Goal active。

## 第十八批策略状态默认参数与数值循环

v5/v6 UDF 默认参数可引用已注册的标量策略状态，例如 strategy.position_avg_price 和 strategy.position_size；省略参数在各次调用时读取当前状态，显式位置或命名参数独立覆盖。保留 simple/series 类型检查、引用类型和任意默认计算/调用的拒绝规则；官方 v4 默认参数语法拒绝证据与原解析器版本范围一致。

for 支持整数或浮点起止值，起点决定计数器种类；整数起点仍拒绝浮点步长。小数终点不截断，浮点计数器保留小数，整数增量保留 i64 精度并在溢出时终止。浮点边界沿用 Pine 数值比较规则，修复 0.1→0.3 / by 0.1 的末次迭代遗漏；只改变比较，不量化算术。默认步长为正 1，显式步长必须为正；方向由初始边界确定，v6 保留动态终点，v4/v5 保留冻结终点，break/continue/结果与资源限制保持原语义。

官方 v4/v5/v6 最小探针给出 16.1、-9.2、3、3；v5 十进制步长次数为 3、3、4。官方负步长在 v4/v5 整数及 v6 浮点探针均报 RE10021，现不再取绝对值。旧的绝对值步长测试和 for_edges 内部夹具改为合法正步长，保留原下降范围与预期绘图值；新增负步长独立错误控制。上述内部夹具修正不涉及用户策略库原源码。

Release SHA256：`41e636277dffd323b3b9cfc557d1348dd7caf540b8c7593ab140d3f76b4402ad`。完整门禁退出码 0：7,686 项 Rust / 116 个套件、911 项新 wheel Python、172 项脚本，以及实际 WASM/Node、fmt/clippy/结构、941 项 CLI 快照和 592+5 宿主黄金值检查。新增 9 项 Rust、11 项 Python 控制，WASM 同步验证 direct/prepared；矩阵仅改变 for 和 expression-body functions 两项说明。精度探针发现遗漏前的门禁/构建回执保留，最终门禁使用修正后的 677 文件冻结集。9 场新官方语言探针控制及 7 场既有 UTC 时钟探针重放通过。

同一批 7,255 份独立源码、512 根合成 bar、8 秒预算：编译 6,020、默认运行 5,255；相对第十七批新增 17/12。通过集合变化以机器回执 newly_rejected / previously_runtime_passed_now_rejected 为准，负步长的官方拒绝修正单独记录。7,576 个用户原始路径哈希核验不变。26 场复杂请求、5 份重点源码、Gap Screener、既有额外源码及 BBPB/888 受控重放也全部使用该 release；新增编译后仍缺输入的 5 份源码也用受控普通/HA 请求流及一致的合成 EUR/USD 标签完整运行；Super Z、My RK、RSI Trend Reversal 与 VAWSI 的具体订单/平仓数量记录在 newly-hosted-original-results，不以 0 订单运行冒充交易触发覆盖。Zendog V3 等 12 份源码在原 512 根默认合成输入中直接通过。这些受控合成输入不代表真实市场交易/收益/外汇或全面 native 数值资格。

3C QFL 原主脚本 e7d7c56... 不再报策略状态默认参数或浮点终点错误。剩余为 jason5480/time_filters/1 外部库输入缺失及两项 tif 调用的关联诊断。官方编辑器显示 is_in_date_range / is_in_session 元数据并能执行导入探针，但源码发布页提示 Publication has ghosted；本轮未取得精确库源码、未提供替代实现，不宣称 QFL 整策略执行通过。股票交易日历和 unavailable-intrabar fallback 仍待实现，常量-only 布尔探针仍未资格化，Goal active。

回执：`target/strategy-gap-final18-20261011/`；机器结果 latest_qualification/eighteenth_batch。所有本轮临时指标与 agent 标签清理，原 CA 策略/月线保留。未提交、未推送。

## 第十九批宿主中立交易日历与交易所时钟

新增不可变 BarCalendarRegistry / BarCalendar / BarTimeInterval 输入契约，宿主提供品种时区、各周期实际开闭盘和交易日标签，核心验证覆盖与排序、选择和偏移 interval。默认无日历仍采用连续 UTC profile；不在核心获取行情、推断品种假日或依赖 CandleScope。CLI --bar-calendars、Python/WASM $barCalendars 共用 schemaVersion 1 的验证器，接口见 BAR_CALENDAR_INPUT.md。缺失日历周期或覆盖显式报错，已知闭市查询返回 na，feed/replay 失败先验证后提交。

time/time_close、时间组件、timestamp 默认时区、str.format_time 默认时区、syminfo.timezone、timeframe.change、time_tradingday 与 VWAP 交易日锚点接入日历。请求合并、低周期确认边界、incremental visible horizon 及 replay/feed trimming 使用提供的实际收盘并保留后继 bar。日线上的日内低周期查询先选择请求 interval，再返回其交易日开闭盘；本轮也按官方值修正连续 UTC time 的相应低周期开盘选择。

TradingView v6 最小官方探针确认 AAPL 6.5 小时日线、纽约时区默认值、前后偏移及周末跨越；三份可实现探针在 CLI 四种模式共 12 场与保存的官方数值一致。独立控制覆盖 DST、闭市、末尾部分小时、跨 UTC 午夜的交易日/VWAP、外部品种及继承式嵌套请求、forming/confirmed/replay 和失败原子性。另有 7 场既有 UTC 时钟重放通过。四场显式自定义时段偏移探针仍精确报出能力缺失：0930-0945 与 1500-1600 必须建立各自的虚拟 session bar 网格，不能直接筛选普通交易所 bar 或截断日线开盘。低周期日/周/月的原有拒绝范围保留；这些结果不代表完整市场日历、broker 交易时段风控或全历史交易数值资格。

Release SHA256：`62d5b1376a22cef9b5ac03f16e862577139a06f1ec201610910e0c74ab400eca`。完整门禁退出码 0：7,699 项 Rust / 116 个套件、920 项新 wheel Python、172 项脚本，实际 WASM/Node direct/prepared、fmt/clippy/结构、941 项 CLI 快照和 592+5 宿主黄金值检查通过。新增 13 项 Rust、9 项 Python 控制；矩阵仅改变 15 项能力说明，host_requirements schema 2 增加 chart.barCalendars。最终 5,742 个生产/夹具文件冻结哈希核验不变。

同一批 7,255 份独立源码、512 根默认合成 bar、8 秒预算：编译 6,020、默认运行 5,255；与第十八批通过集合精确相同，无新增或回退。7,576 个用户原始路径哈希不变。26 场复杂请求、5 份重点源码、Gap Screener、BBPB/888 及额外原源码重放使用本轮 release；第十八批新增编译的 5 份策略再次补充受控普通/HA 请求流和一致币种标签完成运行。0 订单案例未验证交易触发，合成 EUR 标签未验证真实外汇，受控执行不代表收益或官方完整交易一致。准备阶段读取不存在的本轮新增编译清单失败回执保留；修正为明确引用第十八批清单后，五份重放及终端退出码 0 独立核验。

Goal active：下一轮继续实现已确认的虚拟自定义 session interval 偏移；真实 3 RSI MA 数量/币种转换/交易追踪仍需精确数据对照，QFL 精确外部库源码仍缺，unavailable-intrabar fallback 与常量-only 布尔仍未资格化。回执：target/strategy-gap-final19-20261011/；机器结果 latest_qualification/nineteenth_batch。本轮临时官方指标和 agent 标签已清理，原 CA 策略/月线保留。未提交、未推送。

## 第二十批显式会话虚拟时钟与 3 RSI MA 数据起点核对

实现独立自定义 session bar 网格，time/time_close 的 timeframe_bars_back 按该网格选取，而非过滤普通交易所 interval。覆盖分段/跨午夜/整天 session、部分尾 bar、session 内外方向偏移、日周月图与日内图的聚合差异、v4 默认工作日与 v5/v6 默认全周、IANA 时区 DST。宿主仍负责普通市场 interval 与行情，核心不获取网络或推断假日；虚拟日历准备消耗现有 execution-step budget，极端偏移有界报错。一般多日对齐、复杂重叠时段、DST 模糊开盘及 broker 市场时段风控仍未完整资格化。接口范围见 BAR_CALENDAR_INPUT.md。

14 份保存的官方最小源码共 93 个数值，release historical/incremental/realtime-confirmed 请求模式 42 场全部匹配。14 场 forming 请求明确拒绝缺失的有序 intrabar feed，不能用静态 request-bars CSV 冒充该宿主输入。上一轮四场自定义时段偏移缺口已修复：既有四份 calendar 探针现在四种模式共 16 场通过，另有 7 场既有 UTC 时钟通过。新增 14 项 Rust、净增 7 项 Python；WASM direct/prepared 同步覆盖正向数值。

完整 verify.ps1 退出码 0：7,713 项 Rust / 116 套件、927 项新 wheel Python、172 项脚本，实际 WASM/Node、fmt/clippy/429 文件结构、941 CLI 快照、592+5 宿主黄金值检查通过。Release SHA256：`85a46055f3753eb6a6311563752209eb09b60a4de682ae579b559daca2101b9e`。5,744 个生产/夹具文件冻结哈希不变；能力矩阵仅 time、time_close、timeframe.change 三项说明改变，状态及夹具保持。

7,255 份独立源码重新分析并在 512 根默认合成 bar、8 秒预算下重放：编译 6,020、默认运行 5,255，与第十九批通过集合精确相同，无新增或回退。7,576 个用户原始路径哈希不变。26 场复杂请求、5 份重点源码、Gap Screener、BBPB/888、额外原源码及第十八批五份新增编译策略均用本轮 release 复查；额外组保留源码自身/能力错误，未宣称全部通过。零订单和合成币种案例不证明交易触发、真实 FX 或收益。

3 RSI MA 原源码 SHA 6ec508cf...：官方日线起点为 2017-08-17，本地为 2017-10-01，恰少 45 根。官方 bar995 与本地 bar950 都是 2020-05-08；在真实 USDT 标签下本地同日停于无效数量，官方既有回执为负权益导致负数量。45 根编号差异已解释，不能再据此认定 broker 日期偏差。官方 v4 默认账户币种属性为 USDT；核心默认继承图表币种已实现，该源码不要求外汇转换。完整暖机历史、权益数值、成交和账本逐笔一致性仍未资格化。证据：rsi-history-origin-oracle-20.json/.png、default-currency-v4-oracle-20.png。

Goal active：另一个官方 v6 月图探针已确认 time_close("D")/time_close("W") 可以运行，而本轮核心保留 lower daily-or-larger 拒绝；这是下一轮明确解释器缺口，证据 lower-calendar-close-oracle-21.json/.png 已保存，尚未计为修复。精确 QFL 外部库源码仍缺，unavailable-intrabar fallback、常量-only 布尔候选及完整市场交易一致性仍需独立资格。准备失败回执保留；native 两组 QA 日历路径覆盖问题已改为独立 virtual-calendar-control.json 并重跑通过。临时官方探针已移除、agent 标签关闭，原 CA 策略/月线保留。回执 target/strategy-gap-final20-20261012/，机器结果 twentieth_batch/latest_qualification；未提交、未推送。

## 第二十一批周月图上的低周期日周 time_close

删除对较大图表周期中日线及以上 time_close 查询的一概拒绝，普通请求按既有 requested bucket / 宿主 interval 选择收盘；显式 session 继续使用第二十批虚拟网格。chart bars_back 与 requested timeframe_bars_back 独立，缺少实际图表或请求周期日历仍显式报错。没有引入市场数据或网络依赖。

保存的 TradingView v6 月图探针含 11 个日/周开闭盘、前后 requested offset、chart offset 及显式短时段值；release 四种模式全部匹配。两项新增 Rust 控制覆盖官方值和受控交易所月图/D/W 日历，以及缺失 D 日历的错误契约；一项新 wheel Python 控制与 WASM direct/prepared 覆盖同一官方值。复杂多日组、其它市场的周月对齐、全部 DST/重叠 session 与 broker 时段仍未完整资格化。

完整 verify.ps1 退出码 0：7,715 项 Rust / 116 套件、928 项新 wheel Python、172 项脚本，实际 WASM/Node、fmt/clippy/429 文件结构、941 CLI 快照、592+5 宿主黄金值通过。Release SHA256：`5007e9276ff7428f853dcc73c8e3ed1f97254d93a07048810650bc5e380b92fc`。5,744 个生产/夹具文件最终冻结哈希不变。相对第二十批仅 10 文件改变，矩阵只有 time_close 说明变化；首次门禁因我将矩阵写成缩进格式而失败，已保留回执，改回 CLI 原始序列化并重跑完整门禁通过。

相同 7,255 份独立源码 fresh release 重放仍为 6,020 编译、5,255 默认运行，通过集合与第二十批精确相同，无回退；7,576 原始路径哈希不变。26 场复杂请求、5 份重点源码、Gap Screener、BBPB/888、第十八批五份补输入策略及额外源码均复查，额外组保留自身/能力拒绝。第二十批 42 场虚拟会话值匹配、14 项形成期 intrabar-feed 必需输入、16 场旧日历、7 场 UTC 时钟仍有独立 release 回执。真实 USDT 标签的 3 RSI MA 同日停止事实再次核验，不宣称完整逐笔交易或收益一致。

Goal active：下一轮明确缺口是 Morning Scalp 的合法紧凑日期字符串 15Aug 2022；官方 v5 可运行，核心仍拒绝。六个官方值说明紧凑日期与带空格/ISO 格式对时区后缀的处理不同，不能简单插入空格后视为相同语义；证据 compact-date-oracle-22.json/.png。另已确认 v5 bgcolor/plotshape 的 transp 参数官方接受，v6 官方拒绝；当前 33+17 份主诊断源码涉及这一版本兼容性缺口，尚未修复，不宣称整份源码因此可运行。证据 transparency-version-oracles-23.json/.png。外币转换能力仍显式受限，需要 host-neutral rate 输入与完整记账语义；精确 QFL 库、形成期 intrabar feed 属外部输入，其他分析/数组/布尔候选逐项查证。临时官方指标与 agent 标签已清理，原 CA/月线保留。回执 target/strategy-gap-final21-20261012/，结果 twenty_first_batch/latest_qualification；未提交、未推送。

## 第二十二批紧凑英文日期字符串

补齐 timestamp("15Aug 2022 14:00 +0000") 所用的紧凑日/月语法；受控范围包含短月份、完整英文月份、大小写、秒数、省略时间、数字/GMT 后缀。官方紧凑语法忽略数字后缀并按 UTC 计算，而带空格及 ISO 格式仍应用偏移；没有简单补空格改变语义。紧凑格式带 IANA 后缀显式拒绝，官方为编译错误、本核心沿既有 dateString 契约在执行时拒绝，不宣称拒绝阶段一致。自由日期语法全集未资格化。

三个保存的官方正例源码共 18 个值，release 四种模式的 12 场输出匹配；一份非法 IANA 正例变体在四种模式均明确拒绝。新增 2 项 Rust 和 2 项新 wheel Python 控制；WASM direct/prepared 覆盖 v5/v6 同值。完整 verify.ps1 退出码 0：7,717 项 Rust / 116 套件、930 项 wheel Python、172 项脚本，WASM/Node、fmt/clippy/结构和 941 CLI / 592+5 宿主黄金检查通过。Release SHA256：`57cf3feacad158b3065b7a25afe4f85d919fbead8cd7bdf1badf6047c65295ec`；5,744 个冻结生产/夹具哈希最终未变化，相对第二十一批只改变八文件，矩阵只更新 timestamp 说明。

相同 7,255 独立源码 fresh 512 bar / 8 秒 release 复测：6,020 编译、5,256 默认运行；新解锁 1 份默认运行，原通过集合无回退，7,576 原始路径哈希不变。原版 Morning Scalp 源码不修改，在原始 2022-08-15 日期窗口的 780 根受控 New York 分钟 bars 上，四种模式策略输出一致，均为 6 笔订单 / 3 笔已平仓交易；这证明日期缺口已解锁执行和订单触发，不证明真实市场逐笔或收益一致。

26 复杂请求、5 重点源码、Gap Screener、BBPB/888、此前五份补输入策略、额外原源码及保存日历/虚拟网格/UTC 控制全部以新 release 重查；形成期的 14 项明确保持宿主 intrabar-feed 要求。3 RSI MA 实际 USDT 下同日停止事实复查，完整权益账本仍未资格化。

下一轮 v5 transp 缺口已有六种输出及现代参数组合的官方运行证据，官方 series int 限制也已保存；版本适配尚未实施。外币转换需要 host-neutral 历史汇率能力和确定性记账；精确 QFL 依赖与 forming feed 为外部输入。回执 target/strategy-gap-final22-20261012/；结果 twenty_second_batch/latest_qualification。Goal active，未提交、未推送。

## 第二十三批 v5 命名透明度与现代 pane 参数

Pine v5 的 plot/bgcolor/plotshape/plotchar/plotarrow 命名 transp 独立转换到既有颜色执行语义，其余参数继续走现代签名校验。严格保留 const/input integer 或 na 及位置/命名顺序限制；plotshape 重复相同字面量 transp 在 v5 如 v4 一样安全去重，其他含糊重复仍显式拒绝；v6 仍拒绝这五种输出的 transp，barcolor 不开放该参数。修复此前 v5 plot 使用旧参数表时拒绝现代参数的问题。bgcolor、plotchar 增加 const bool force_overlay，并保存在宿主中立输出元数据中；运行时无需宿主服务。

六份保存官方源码包含四个合法正例和两个非法版本/series-int 控制；16 场 release 模式输出匹配保存的 11 个显示值，2 场分析拒绝匹配官方边界。新 Rust 控制还验证 transp 对颜色有实际作用并与本地 color.new 对应；此为核心内部颜色一致性，尚未证明 TradingView 渲染 alpha 的数值一致性。旧版完整位置参数表、任意填充 pane 组合未资格化。fill 在本批保持既有独立签名。

完整 verify.ps1 退出码 0：7,720 Rust / 118 套件、934 fresh wheel Python、172 脚本、实际 WASM direct/prepared、fmt/clippy/结构与 941 CLI / 592+5 宿主黄金值检查通过。Release SHA256：`4cdbfc5e54d09cf74acadb2b721ce102f2db4ea6bda625cfea95fde7be7ea564`；5,748 生产/夹具文件最终冻结未变，相对第二十二批 15 文件变化，矩阵仅五条说明变化，夹具列表/状态不变。

相同 7,255 独立源码 fresh release 512 bar / 8 秒复测：6,066 编译、5,290 默认运行；相对第二十二批新编译 46、新默认运行 34，原通过集合无回退，7,576 原始路径哈希不变。原 50 份 v5 bgcolor/plotshape transp 主诊断源码中 44 份新编译通过；另 6 份仍有重复 color、int/na 比较、indicator 中 strategy.entry 或 int 写入 bool 等独立诊断，尚未逐份官方判定。另各一份 plotarrow transp 与 bgcolor force_overlay 原源码被解锁，共 46 份新编译。不得将这 6 份笼统判为非法源码；保留下一轮查证。

此前 26 复杂请求、5 重点源码、Gap Screener、BBPB/888、五份补输入策略、额外原源码、日历/虚拟网格/UTC 和紧凑日期官方控制均以本 release 再测；Morning Scalp 四种模式仍产生相同 6 笔订单 / 3 笔已平仓交易，形成期 14 项仍明确需要宿主 intrabar feed。真实市场、收益和完整逐笔账本一致性未宣称。

官方 v6 fill 最小负例已确认本地完整分析链原本有正确版本拒绝，签名表疑似缺口被排除；无需增加生产修复。下一轮 bool/int 混合三元分支已有 v4/v5 合法、v6 拒绝的官方证据，19 份主诊断源码候选待修复；同时继续未资格化语义和外币能力。历史外币转换仍需 host-neutral 汇率输入与确定性记账；精确 QFL 库、形成期 feed 为外部输入；其它诊断逐项核验。证据 transparency-version-oracles-23.json 与 fill-version-oracles-24.json、bool-int-ternary-oracles-24.json；回执 target/strategy-gap-final23-20261012/。Goal active，未提交、未推送。

补充宿主输入复核：本批 46 份新编译源码中，34 份默认重放通过，其余 12 份全部因缺少 request 数据停止。保持这 12 份源码不变，提供明确的受控常规/HA/日线数据和时钟后，12/12 运行通过；原默认统计仍为 5,290，不混入补输入通过数。controlled 币种/数据仅说明输入契约和执行可达，不证明真实市场、收益或 native 全账本一致。回执 new-transparency-hosted-results.json。

首次第二十三批完整门禁在旧 v5 重复 transp 断言处失败，回执 verify-initial.log/.exit 保留；官方证明相同字面量重复仍合法，已扩展安全去重、修正旧断言、重新冻结，并完整重跑通过。fill 的签名表疑似缺口经完整语义链和官方负例排除，未额外修改核心。

## 第二十四批 v4/v5 bool 与数值三元表达式

v4/v5 的标量三元表达式在 bool 与 int/float 分支之间选择 bool 公共类型，运行时仅将所选数值分支转为布尔值。分析、带函数参数的类型查询和 lowering 保持一致；纯数值函数调用仍返回数值，限定符不放宽。该规则只应用于标量 ternary；不将同样规则泛化到 if/switch、tuple 返回或未资格化的 v1–v3。v6 的混合分支类型拒绝保持不变，v5 混合结果用于加法也按官方 series bool 类型拒绝。

六份保存官方探针包含四份正例与两份负例，16 场 fresh release 的四种执行模式匹配保存值，2 项分析拒绝符合官方边界。扩展正例覆盖两个条件方向、分支顺序、正负整数、零、小数、bool/na 的条件显示与 UDF 返回；额外本地测试覆盖别名、历史引用、重新赋值和同一函数的纯数值调用。无原源码修改。

同一 7,255 源码、512 synthetic bars、8 秒默认运行复测：6,085 编译 / 5,307 默认运行；相对第 23 批新编译 19 / 新默认运行 17，旧通过集合无回退。19 份原始 bool/int 主诊断候选中 17 份编译通过；剩余诊断逐份保留。新编译但默认未通过的 2 份另行运行受控宿主输入并保留结果，未混入默认通过数。7,576 原始路径哈希均不变。

完整 verify.ps1 退出码 0：7,724 Rust / 120 套件、937 fresh wheel Python、172 脚本、实际 WASM direct/prepared、fmt/clippy/结构和既有黄金值检查通过。Release SHA256：`77902007a7f25a5e564681343469aa99f996ae461894ac4ffeaa6807ca9074d4`。5,752 生产/测试/夹具文件冻结未变，相对第 23 批 12 文件变化。此前重点、复杂请求、Gap Screener、补数据策略以及日历/虚拟网格/输出/紧凑日期控制均以本 release 重新验证；Morning Scalp 四模式仍为相同 6 订单 / 3 已平仓交易；3 RSI MA 在实际 USDT 标签下仍于保存的相同日期停止。形成期 14 项仍需要宿主有序 intrabar feed。

精确 QFL 外部库源码和形成期 feed 为外部输入；历史外币转换和完整确定性账本是核心待补能力；其他独立诊断仍需官方证据。收益、完整真实市场交易账本一致性和更广泛语法未宣称。证据 bool-int-ternary-oracles-24.json；回执 target/strategy-gap-final24-20261012/。Goal active，未提交、未推送。

## 第二十五批告警普通字符串中的花括号

TradingView v4、v5、v6 三份原生探针均编译并显示 compiled=1，包含 alert 消息、别名和拼接，以及 alertcondition 标题中的已知占位符样式、未知样式和未闭合花括号。原校验错误地将这些普通文本也视为消息模板而拒绝。修复只将模板校验应用于 alertcondition 的 message 参数；alert 消息与 alertcondition 标题保留原文。既有类型、频率、函数作用域和未知消息占位符的显式拒绝保持。

三份探针四执行模式共 12 场新 release 验证通过；内部输出验证原文和既有 close 消息替换。官方证据限定于编译接受和图表标记，未创建或发送真实告警，未资格化官方通知载荷或送达一致性。

同一 7,255 源码、512 synthetic bars、8 秒默认运行复测：6,089 编译 / 5,309 默认运行；相对第 24 批新编译 4 / 新默认运行 2，旧通过集合无回退。7,576 原始路径哈希不变。新增源码状态逐份保存在 qualification.json 的 new_compile_resolution。

两份新编译的 PSAR 原源码默认复测缺显式执行时间。另行提供固定执行时间轨迹和 USD 标签的受控日线 OHLC，两份源码各四模式共八场通过且实际告警分支执行；消息保留占位符样式原文。这些受控通过未计入默认运行通过数，未宣称真实汇率、交易账本或官方通知载荷一致性。

完整 verify.ps1 退出码 0：7,727 Rust / 122 套件、941 fresh wheel Python、172 脚本、实际 WASM direct/prepared、fmt/clippy/结构和黄金值检查通过。Release SHA256：`bcd65ad80d3208503ec2c7d285366ab32016906b1cbb690ddf29437d5d3ca694`；5,756 生产/测试/夹具文件冻结未变。既有重点、复杂请求、Gap Screener、补宿主数据策略、日历/虚拟网格/输出/紧凑日期/三元表达式控制均以本 release 重测；形成期 14 项仍需要宿主有序 intrabar feed。

精确 QFL 外部库源码和形成期 feed 为外部输入；历史外币转换和完整确定性账本是核心待补能力。3 RSI MA 同日期停止已建立，但完整官方权益账本未资格化。证据 literal-alert-braces-oracles-25.json；回执 target/strategy-gap-final25-20261012/。Goal active，未提交、未推送。

## 第二十六批告警消息模板与显式时钟

新增普通 plot 声明的 plot_0 到 plot_19 消息绑定，按声明次序选择当前柱数值；模板在本次脚本语句执行结束后展开，使后声明的普通 plot 和命名 plot 可以提供当前值。告警事件保留源程序到达次序；策略在本次语句执行结束后、即时订单后处理前展开。编号绑定跨越 plotchar/shape/arrow/bar/candle 的混合布局仍显式报告未资格化能力，不给出猜测值。

timenow 消息占位符使用宿主显式 execution timestamp，只有告警条件实际为真才读取时钟；无时钟保持明确错误。宿主输入清单识别常量消息、别名和拼接中的 timenow。time 与 timenow 告警格式改为官方文档规定的 UTC 秒精度字面量 Z，通用 str.format_time 不变。旧告警黄金值的 +0000 被精确修正，初次完整门禁失败日志保留在 verify-initial.log，修正预期后完整重跑成功。

保存的 v4/v5/v6 三份原生探针均编译并显示 20 个恒定 plot 数值；12 场 fresh release 四模式与保存图表值一致，同时验证文档规定的消息绑定和受控执行时间格式。未创建真实通知、未捕获原生通知载荷，官方送达及载荷数值一致性未资格化。

同一 7,255 源码、512 synthetic bars、8 秒默认运行：6,089 编译 / 5,309 默认运行，相对第 25 批新编译 0 / 新默认运行 0，旧通过集合无回退；7,576 原始路径哈希不变。三份受影响源码的模板诊断消失，但全部仍有指标/研究脚本调用 strategy.entry 的独立错误；保存的 v4/v5 官方最小负向探针同样拒绝该用法。未将能力补齐算作整份源码通过，未修改原源码声明。逐份状态见 qualification.json 的 template_candidate_resolution。

完整 verify.ps1 退出码 0：7,731 Rust / 123 套件、945 fresh wheel Python、172 脚本、实际 WASM direct/prepared、fmt/clippy/结构及黄金值通过。Release SHA256：`2ba71bf8ab691892a5a77e981c44ef4dc3a50a631c677fb2e2155a46cd6e68fc`；5,761 生产/测试/夹具冻结文件最终未变。此前复杂请求、重点策略、Gap Screener、宿主补数据、日历/虚拟网格/紧凑日期/输出/三元表达式/普通告警控制及两份 PSAR 四模式均重新验证。

剩余外币账本属于核心能力；QFL 精确库源码、请求数据、显式时钟与形成期有序 feed 属宿主输入。其他语义诊断逐项核对，3 RSI MA 完整原生权益账本未资格化。证据 alert-template-oracles-26.json；回执 target/strategy-gap-final26-20261012/。Goal active，未提交、未推送。

## 2026-10-15 当前工作区收尾

本次收尾包含以上兼容性实现、既有开发构建调试信息配置，以及独立的宿主接口资源审计工具和历史测量记录。资源测量仍绑定原 0.3.1 wheel；其 `notEvaluated` 资格化边界保留。当前开发改动未发布为新版本。

代码审查和交叉复核修复以下已复现问题，每项均有回归控制：

| 问题 | 修复后的行为 |
|---|---|
| 请求参数规范化吞掉源参数顺序 | 在规范化前拒绝具名参数之后的位置参数；保留合法的具名重排 |
| strategy.exit/close 源参数顺序 | v4/v5/v6 均在改写参数名称前执行源顺序检查 |
| timestamp 数值 year 实参重复求值 | 每个实参只求值一次，带数组副作用的函数不再重复执行 |
| matrix.sum 的 float 标量值为 na 时丢失类型 | 保留 float 结果矩阵，后续小数写入可正常读回；原矩阵保持独立 |
| 低周期 lookahead_on 对未知日历 bucket 合桶 | 缺少必要历史日历覆盖时明确报错，避免返回任意远古首值 |
| strategy.exit 动态 qty/qty_percent 为 na | 按缺省数量及既有数量优先级处理 |
| 全 entry 组合退出错误扣除同名 OCA 预留 | 同组共享保留量的规则与普通退出一致 |
| 同 ID 加仓的后续交易缺少 OCA membership | 为每个 trade key 绑定和清理对应成员 |
| 具名组合退出复用第一笔开仓价 | 每笔交易独立解析相对止损、止盈和 trailing 参数 |
| 聚合与逐交易退出混合时重复扣留 | 与真实成交共用 FIFO/ANY 分配器；按挂单顺序模拟分配，同 OCA 组按每个 trade key 的覆盖并集扣留，移除零可用数量替换留下的旧挂单 |

策略专项最终 958 项通过。新增混合保留量测试覆盖 62 个场景，包括不同交易数量、固定/百分比数量、挂单与成交顺序、跨 entry 的 FIFO/ANY、同组与不同组以及不同交易键。Python 新 wheel 和 WASM direct/prepared 另有矩阵类型与 timestamp 副作用回归。

最终 `verify.ps1` 退出码 0：7,746 项 Rust / 125 套件、947 项新 wheel Python、172 项脚本测试，fmt、全目标 clippy `-D warnings`、431 个生产 Rust 文件结构检查、实际 WASM/Node 及 941 CLI / 592+5 宿主黄金值检查全部通过。6,252 个最终输入文件的冻结哈希保持不变。

新 release CLI SHA256 为 `8e8b6f2f8bdb8f57de4f1d7cba478e00402d82f35117370c4f7e8c76235c1460`。相同 7,255 份规范化源码、512 根合成 bar、8 秒预算的 fresh 重放仍为 6,089 编译 / 5,309 默认运行通过；编译集合、运行通过集合及逐源码状态与第二十六批精确相同，无回退。7,576 个原始源码路径哈希均保持不变。语料执行在最终完整门禁结束后开始，没有与编译负载混跑。

238 项保存控制均达到对应预期，关键状态、订单/平仓数量、绘图数量及实际值差异为 0。其中 206 项正向执行，其余保留既有拒绝、独立阻碍或宿主输入要求。准备目录最初漏复制一个控制 runner，退出 2 的回执已保留；仅补齐 target 中的 runner 后继续未执行组，所有最终结果均由同一固定 CLI 新执行，没有复制旧运行结果。

最终完整门禁、冻结输入、新 release CLI、相同语料的编译/运行集合，以及保存控制的结果由 [本次机器回执](WORKSPACE_CLOSEOUT_20261015.json) 记录。初次格式检查中止、修复前失败、修补前通过的门禁和中止的语料运行均保留，未混入最终验收。

资格化范围继续限定于已有保存官方探针、受控输入、独立核心测试及同库合成重放。历史外币转换与完整确定性账本仍是后续核心能力；精确 QFL 库、行情、显式时钟及形成期有序 feed 继续通过宿主中立输入契约提供。完整原生收益/逐笔账本、任意市场日历和任意 Pine 源码的兼容性仍未资格化。
