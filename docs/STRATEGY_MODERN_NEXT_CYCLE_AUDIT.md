# 现代策略下一轮开发记录

日期：2026-09-08。状态：closed（本轮冻结范围）；不代表全 Pine 或全语料兼容。
起始 HEAD：`45cbd6586a9e0b6d7e9f137945bacb2eaf9ab346`。

## 交付范围与基线

本轮承接 G3 收口、真实策略独立参考扩展和一个语料驱动语言切片。
原有 Windows 工具链暂存变更与 G3 工作区变更已经分别保存补丁和起始状态，
记录在忽略目录 `.local/next-cycle-20260908/`。本轮保持原暂存内容；工作区
验证不冒充 clean-HEAD、远端 CI、合并或发布验收。

开始时完整 `scripts/verify.ps1` exit 0，包括 Rust workspace、工具测试、
结构与宿主检查、真实 WASM/Node，以及新构建 Python wheel 的 672 项测试。
日志：`baseline-verify.log`。

G3 上轮六组参考的冻结分母保持独立，见
[G3 closeout](STRATEGY_MODERN_G3_CLOSEOUT_AUDIT.md)。公共 r1 不改写：
480 脚本 / 482 场景，旧参考覆盖仍为 0/482。上一轮六组参考不进入该分母。
B1 内部顺序继续保留 `UNVERIFIED_INTERNAL_ORDER`。

## 语言切片：显式 series 标量函数参数

旧 WSL 镜像及探针没有出现在本机原路径。本轮直接读取
[TradingView 官方 TechnicalRating v3 源码](https://www.tradingview.com/script/jDWyb5PG-TechnicalRating/)，
保存页面可见的全部 214 行，保留缩进、源码许可声明和版本。
源码仅用于本地调查，未复制进公开 fixture。

首个错误为第 27 行 `series float` 参数的 `E_PARSE_EXPECTED`。原创最小测试
先复现失败。切片接受 v5/v6 本地和导入函数的显式 `series int/float/bool/string/color`；
保留限定符到语义分析与 HIR 内联绑定。常量或 input 实参不能将其弱化到 simple。
参数值种类校验、数值提升、独立调用点历史、持久局部变量和 forming 回滚均需验证。

依据：[函数参数类型与限定符](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/)、
[v5 类型系统](https://www.tradingview.com/pine-script-docs/v5/language/type-system/)。
本轮不扩展 simple/const/input 参数限定符、默认实参、显式限定的引用类型或方法参数。
不改变输出或 analysis schema；AST 的 `FunctionParam.type_name` 保留限定的类型拼写。

修复后官方库的首个解析错误后移至第 157 行的默认参数 `= 0.5`，原第 27 行阻塞消失。
这证明一个根因已排除，不代表整库或依赖 `TradingView/ta/9` 可以执行。

## 本轮真实策略参考批次

新 revision：`g3-real-strategy-r3`，两种 TradingView 内置真实策略，三个派生场景。
原始代码从编辑器读取，分别为 MovingAvg2Line Cross（界面 revision 30）和
ChannelBreakOutStrategy（revision 28）。原文、版本显示与派生文件保持独立。
原 WSL 语料镜像不可用，因此这是明确的新来源批次，不冒充恢复旧 permissive overlay。

派生仅为冻结比较条件：增加 2026-09-04 00:00 UTC 至 09-06 00:00 UTC 信号窗口，
窗口外撤单/平仓、显式资金/数量/费用/保证金配置和观测 plots；保留实际均线交叉、
通道上下界及原始止损入场规则。通道 v5 另为版本控制，不算第三种独立策略。

| 场景 | 独立平仓交易 | 输出序列 | 比较 bar 值 | 差异 |
| --- | ---: | ---: | ---: | ---: |
| ma：SMA 9/18 多空交叉、百分比手续费、pyramiding=0 | 11 | 11 | 5,565 | 0 |
| channel：5 bar 通道、双向 stop、每单固定费用 | 27 | 9 | 4,573 | 0 |
| channel-v5：同规则同配置 v5 控制 | 27 | 9 | 4,573 | 0 |
| 合计 | 65 | 29 | 14,711 | 0 |

图表为 OKX:BTCUSDT、15 分钟、标准 K 线，显示 UTC+8；原始时间为 UTC epoch 秒，
运行时转毫秒，显式 price grid 1/10。加载 9 月 3 日历史后，按事先确定的
`1788785100000` 截止，保留 509 根已收盘 bars。只有 SMA 输出跳过最初 17 根，
通道输出跳过最初 4 根；交易窗口开始前已有充分预热，全部账户/持仓输出从首 bar 比较。

身份/时间字段精确一致，数量/价格/利润及 plots 保持既有绝对/相对 1e-9 容差。
原生交易 CSV 的数量按进场方向转换为有符号数量，交易编号顺序不按 runtime 重排。
百分比费用的 CSV 利润精度不足：提高 precision 后 CSV 仍舍入；保存两次原始文件，
并补导 TradingView 自身的 closedtrades.profit/commission/entry_time/exit_time 序列。
每笔必须满足 entry/exit 时间一致、平仓计数恰好增加一，才使用该 bar 的精确利润；
再交叉检查其两位小数与原生 CSV 一致。这是独立参考格式归一化，不放宽 runtime 容差。
未用本运行时输出生成预期，也未以空交易或最终收益替代逐笔验证。

初次均线执行被 pyramiding=0 阻塞；修复后 11 笔成交通过。随后逐 bar 发现
不存在交易时 profit 返回 na/0 的差异，保留 `*-before-profit-fix.json`；
channel 与 channel-v5 的负索引、越界及第一笔出现前控制均证明参考值为 0。
修复仅覆盖有效整数索引找不到交易时的 `closedtrades.profit`，na 索引及其他身份字段不变。
五组已有快照的审查只允许对应 profit plot 的 null→0；已有成交价格、费用和账本值不变。

三场景另做 18 次模式检查：完整历史的真实增量、realtime-history、forming 替换，
以及各场景首次进场、首次出场和最后出场的活动 bar 前缀替换/确认，均全输出一致。
真实新 Python wheel 和 WASM/Node 对三场景做 6 次完整 JSON 比较，均与 CLI 一致。
这是宿主回滚与三端一致性证据，不等于 TradingView 真实 tick 或 Magnifier 验收。

原始源码、CSV、设置、失败报告、比较工具和哈希在 `.local/next-cycle-20260908/`。
`compare_capture.py ma|channel|channel-v5` 重跑原始参考比较；`verify_hosts.py` 与
`verify_hosts.cjs` 重跑本轮保留模块的宿主比较。公开 CI 仅依赖原创回归，不依赖私有导出。

## 下一轮优先项

- TechnicalRating v3 下一首错为函数默认参数；需独立默认值绑定与调用点语义切片，
  后续还需实际核对 TradingView/ta/9 依赖，不能把本轮 series 支持当作整库解锁。
- 扩展跨周期 request 和 Magnifier 的完整匹配输入包；本轮没有覆盖它们。
- 账户扩展与性能优化继续按独立差异或测量选择。B1 内部顺序保持未知。


## 参考扩展发现的额外声明缺口

均线参考副本显式使用 TradingView 接受的 `pyramiding=0`，本地首次运行却被
`E_CALL_ARG_VALUE` 拒绝。该值表示一个方向只允许首次入场，并非禁止所有入场。
本轮保留冻结输入并修复验证与有效容量转换：非负整数可接受，0 转为内部容量 1，
负数与小数仍拒绝。多空重复入场、反转、0/1 等价、增量和 forming 确认均有回归。
依据：[官方参数说明](https://in.tradingview.com/pine-script-reference/v5/)。
该切片额外覆盖 CLI/Python/WASM，共用原创 fixture 与完整输出快照。

## 最终验收与交付边界

最终 `scripts/verify.ps1` 非 UPDATE_SNAPSHOTS 模式 exit 0：Rust 6,543 项、
新 Python wheel 675 项、工具 101 项，以及真实 WASM/Node smoke 全部通过。
宿主清单覆盖 877 个 CLI runtime 快照、581 个必需 runtime golden 和 5 个 legacy analysis golden。
最终日志为 `final-verify-v5.log`；早期格式、快照序列化与旧 profit 断言失败日志保留，
修复后重跑完整门禁，没有降低测试要求。

本轮重跑上轮 G3 六场景的四种模式，24/24 与上轮冻结输出一致。
最终重测公共 r1：解析 480/480、语义 479/480、运行 481/482、独立参考 0/482；
剩余根库版本冲突未通过修改输入或版本校验隐藏。旧负向分类保持 89/93 拒绝，
四个已被后续能力接受的旧标签仍单列。

快照改动边界：新增 series 参数、pyramiding=0、缺失交易 profit 三个原创 golden；
matrix 仅同步这三项范围。profit 修正更新的五个已有 golden 是
`runtime_strategy_slippage`、`runtime_strategy_exit_slippage`、
`runtime_strategy_limit_verification_exit`、`runtime_strategy_closedtrades_fields`、
`runtime_strategy_closedtrades_fields_pyramiding`，仅对应 profit 的 null→0。
开始时已有的价格网格/费用快照差异保持单独来源记录。

起始与结束的暂存补丁逐字节一致。没有暂存、提交、推送、合并或发布；
这是可审查工作区交付，尚非已提交版本的发布验收。
`verification-summary.json`、`manifest.json`、实现文件归档和结束状态一起保留在本地证据目录。
