# 四项策略兼容修复与全库复测（2026-10-29）

按上一轮审计顺序完成 `entry/order` 的 `na` 价格、版本化 `nz` 命名参数、函数内 `alert()`、标准 OHLC 填单四项修复。相对当日最新 main 基线，原始源码新增 **33 份编译准入、30 份运行完成**，原有编译和运行通过集均未回退。

核心提交：`789b7fdfd2674467bc76b97d90633b7e3a61e341`。原审计记录保留在 [策略库基线报告](STRATEGY_CORPUS_RETEST_20261029.md)，本次可机读对照在 [修复结果 JSON](STRATEGY_COMPAT_FIXES_20261029.json)。后续证据提交仅补充报告及 Node 契约断言，不改变所测核心代码。

## 已实现行为

1. **`strategy.entry/order` 缺省价格。** 裸 `na`、带类型的 `na` 和当次求值为 `na` 的价格均视为未指定；按剩余有效价格选择市价、限价、止损或止损限价单。价格表达式只求值一次；默认数量、收盘填单、OCA、替换挂单和外部订单指令保留对应语义。非正价格检查仍在。
2. **`nz` 版本化绑定。** v3/v4 使用 `x/y`，v5/v6 使用 `source/replacement`；乱序命名参数先归一化，再按相同运行时逻辑执行。错误版本的名称、重复参数和非法位置关系继续拒绝。
3. **函数内 `alert()`。** 支持嵌套函数、不同调用点和动态消息，沿用频率控制及实时回滚。`alertcondition()`、图形输出等原有作用域规则分别保留。输出是运行时事件，不负责通知发送。
4. **标准 OHLC 填单。** v5/v6 接受常量命名参数 `fill_orders_on_standard_ohlc`。标准图表保持原行为；Heikin-Ashi 图表由宿主为每次观察提供标准 OHLC，Pine 表达式继续读取合成图表值，原生 broker 按标准价格成交、估值及计算默认数量。Rust、CLI、Python、WASM 均已接入。

第四项不是只接受声明：测试用约 1000 的图表价格与约 100 的标准价格，验证图表输出保持前者、实际成交使用后者。缺失数据报 `E_STANDARD_OHLC_INPUT`。宿主契约 schema 升至 **5**，结果 schema 9、增量变更 schema 4 不变。完整接口及 JSON 样例见 [标准 OHLC 宿主契约](STANDARD_OHLC_HOST_CONTRACT.md)。

Heikin-Ashi 标准填单与 `calc_on_every_history_tick`、外部账户执行的组合仍明确不支持；其他非标准图表类型也未开放。数据获取、订阅及宿主生命周期不在本仓库实现。

## 同一原始策略库的结果

| 指标 | main 基线 | 修复后 | 变化 |
| --- | ---: | ---: | ---: |
| 去重源码 | 7,255 | 7,255 | 0 |
| 编译准入 | 6,153 | 6,186 | +33 |
| 编译拒绝 | 1,102 | 1,069 | -33 |
| 按冻结口径运行完成 | 5,365 | 5,395 | +30 |
| 完成且无 `E_*` 诊断 | 5,054 | 5,105 | +51 |
| 完成但带有 `E_*` 诊断 | 311 | 290 | -21 |
| 缺少 request 输入 | 641 | 644 | +3 |
| 缺少执行时钟 | 79 | 79 | 0 |
| 缺少汇率输入 | 49 | 49 | 0 |
| 其他运行拒绝 | 19 | 19 | 0 |

没有超时或崩溃，没有新增编译拒绝或丢失原有运行完成脚本。新增的 3 份 request 缺失都来自本次刚获得编译准入的源码。

冻结的“运行完成”定义仍是进程返回 0 且没有明确 `severity=error` 的诊断。部分 broker 的 `E_*` 诊断不带 severity，因此另列严格视角，不能将 5,395 解读为无诊断通过或 TradingView 等价。

原候选组的结果如下；未通过项仍有其他阻塞，四种目标诊断均已消除。

| 原候选组 | 原源码数 | 新增编译准入 | 运行完成 | 缺宿主输入 | 其他编译阻塞 |
| --- | ---: | ---: | ---: | ---: | ---: |
| entry/order 显式 `na` | 9 | 7 | 5 | 2 | 2 |
| `nz` 命名参数 | 8 | 8 | 8 | 0 | 0 |
| 函数内 `alert()` | 10 | 8 | 8 | 0 | 2 |
| 标准 OHLC 声明 | 11 | 10 | 9 | 1 | 1 |

剩余阻塞包括 v3 未定型 `na` 推断及版本混用、旧企业行为函数名、非正的 `strategy.close` 数量、解析错误；不通过放宽这些检查提升本轮通过数。全库使用标准图表，表中的标准 OHLC 候选通过数是原库准入/冒烟证据；Heikin-Ashi 双价格行为由独立语义测试和各绑定测试验证。

## 订单变化与残余诊断

在原有 5,365 份完成源码中，**31 份的 orders/trades 数量改变**，均属于旧 `E_STRATEGY_PRICE` 集合；所有前后完成源码的 plots/alerts 数量保持一致。完整逐源码对照保存在 JSON 的 `counts_changed`。这不是全输出数值等价检查。

例：`BTC Rapid fire strategy 1M Scalping.pine` 从 0 个订单、0 笔交易变为 8 个订单、7 笔交易；`Channel Break [for free bot].pine` 从 28/14 变为 45/30。变化与合法缺省价格不再丢单的修复一致，不代表策略收益或实盘质量提升。

| `E_*` 诊断涉及的完成源码数 | 基线 | 修复后 |
| --- | ---: | ---: |
| `E_STRATEGY_PRICE` | 35 | 3 |
| `E_STRATEGY_EXIT_PRICE` | 173 | 174 |
| `E_STRATEGY_EXIT_TICKS` | 61 | 61 |
| `E_STRATEGY_MARGIN` | 45 | 46 |
| `E_STRATEGY_EXIT_QTY` | 14 | 14 |
| `E_UNSUPPORTED_ALERT_PLACEHOLDER` | 1 | 1 |
| `E_STRATEGY_CLOSE_QTY` | 1 | 1 |

这些诊断按源码去重，同一源码可进入多行。32 份旧源码不再报告 `E_STRATEGY_PRICE`；其中两份订单/成交数量未变。另一份仍带非正限价诊断的源码恢复了部分订单。

剩余 3 份 `E_STRATEGY_PRICE` 的消息均为 entry 的限价/止损价必须为正，已不是把 `na` 当非有限价格的错误。这里只确认当前检查的触发原因，未新增原生回执证明该正价格边界对所有 Pine 场景都正确。

新增的 exit 价格诊断来自 `c7c89f8219698b6f2b2094d9d8638d2602869fc1b8bb41f68b2e63c82c145f37`：开仓恢复后，持仓条件下的 `strategy.exit(stop=center)` 报非有限价格。新增 margin 来源是一份刚准入的 `9057366f…` 脚本。二者均保留诊断，不计为无错误完成。

## 验证与证据

| 验证 | 结果 |
| --- | --- |
| `cargo test --locked --workspace --no-fail-fast` | **8,037 passed，0 failed** |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | 通过 |
| `cargo fmt --all -- --check`、结构检查 | 通过；检查 496 个生产 Rust 源文件 |
| Python 全套，加载本次构建的扩展 | **2,038 passed** |
| 真实 wasm32 构建与 Node smoke | 通过，含本轮四项行为 |
| CLI/Python/WASM 宿主一致性检查 | 通过 |
| 脚本工具 unittest | 184 项，183 passed、1 skipped |
| 原 56 份 `na` 控制脚本 × 4 模式 | **224/224** 完成，每次 1 笔成交、无 `E_*` |
| 8 份原始 `nz` 候选 × 4 模式 | **32/32** 完成，源码未改写 |
| 原始策略库 | 7,255 份全部重新分析；6,186 份全部重新运行 |

测试覆盖正反方向、entry/order、裸/类型/序列 `na`、一个有效价格与一个 `na` 的组合、数量默认值、挂单替换、OCA、函数调用频率、实时回滚、缺失标准数据的重试及错误输入。既有测试中将新增支持行为列为拒绝的断言已更新，仍保留非法参数和作用域的反例。

测试环境为 Windows、Rust 1.97.1、Python 3.12.7、Node 22.23.4。这是本地验证，不是新的远端 CI 或发布资格认证，也未取得新的 TradingView 原生对照回执。

固定证据：

- main 基线：`b646ced593a872e47586f0f506052f6d7c14cde9`。
- 修复二进制 SHA-256：`65d668ac920ccdcf6d424e4dbd5929e3df372a8650bdaa576ee3370f862615d1`。
- 行情 SHA-256：`d71c8d58d6ed80bc5a7492b8093bc4cc86d079516a2327d5229d9bc37b633fef`，与基线相同的 512 根合成 1 分钟 K 线、`V-BTC-SPOT`、默认参数。
- 7,576 个原始路径和 7,255 份规范化源码哈希全部校验通过，执行后源码字节未变；未复用旧执行回执。
- 全库执行时间：2026-10-29 **20:21:17–20:25:18**，Asia/Shanghai；分析超时 15 秒、运行超时 8 秒、4 个工作线程。
- 完整本地回执：`E:/projects/pine-interpreter/target/strategy-corpus-fixed-20261029/`，含冻结 CLI、行情、源码清单、逐次回执、`results.json`、`summary.json`、`triage.json`、`comparison.json`、`probe-results.json` 和验证日志。第三方 Pine 源码未加入仓库。

可用 `scripts/retest_saved_strategy_library.py` 重新执行，再通过 `scripts/summarize_saved_strategy_retest.py` 和 `scripts/compare_saved_strategy_retests.py` 生成分组与对照。重跑必须使用新输出目录。

## 下一步顺序

原计划前四项已完成。接下来按原排序核实**旧版成交量内建名称**：仍有 4 份候选，其中 3 份只有该错误族；应先建立版本、别名、首值及序列语义对照，再决定修复。之后是函数内 `max_bars_back`，3 份候选均叠加其他阻塞。

另将 **174 份 exit 价格诊断**列为下一轮结果正确性审计重点，先区分缺省 `na`、无有效退出参数、尚无持仓和源码计算结果，不能将该数量直接当作 174 个已确认缺陷。644 份 request、79 份时钟、49 份汇率缺失仍按宿主输入边界处理。
