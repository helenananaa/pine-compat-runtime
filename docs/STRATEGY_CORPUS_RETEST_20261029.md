# 策略库重测与下一轮兼容优先级（2026-10-29）

本轮基于最新 `main` 的 `b646ced593a872e47586f0f506052f6d7c14cde9`，重新分析全部 7,255 份去重源码，并运行其中可执行的 6,153 份。与 10 月 18 日基线相比，编译准入和运行完成数量均未变化，没有原先完成的脚本掉队。

**下一步优先修复 `strategy.entry` / `strategy.order` 的 `na` 价格语义。** 当前实现既拒绝合法的裸 `na`，又会把已经接受的 `float(na)` 当成无效挂单价格，丢弃本应按市价提交的订单。其价值高于单纯提高编译通过数。第二项是修复 `nz` 的版本化命名参数，已有 8 份原库脚本的隔离证据。

本轮只增加审计工具、复现材料和报告，未修改解释器核心，也未把改写后的脚本计入原库通过数。可机读汇总见同目录 `STRATEGY_CORPUS_RETEST_20261029.json`。

## 1. 固定测试条件

| 项目 | 本轮条件 |
| --- | --- |
| 核心提交 | `b646ced593a872e47586f0f506052f6d7c14cde9`，重测前拉取，报告前再次核对远端 main |
| 构建 | `cargo build --locked --release -p pine-cli`，Rust 1.97.1，CLI 0.3.3 |
| CLI SHA-256 | `cadf0e5184204b753902c53e8bd8f96109d54600d665c3c2793c2af5b2a7d7e4` |
| 对照基线 | `strategy-gap-final45-20261018/results.json`，核心 `736996aac32b1da6f92af12dde4548ff0cbd45f9` |
| 源码 | 7,576 条来源记录；7,255 份去重、可独立分析的规范化源码 |
| 源码完整性 | 原始文件 7,576 个路径、规范化源码 7,255 份均校验；执行后规范化文件字节哈希不变 |
| 行情 | 原基线同一份 512 根合成 K 线；`V-BTC-SPOT`，1 分钟；默认参数 |
| 行情 SHA-256 | `d71c8d58d6ed80bc5a7492b8093bc4cc86d079516a2327d5229d9bc37b633fef` |
| 执行 | 全量历史 `run`；4 个工作线程；分析超时 15 秒，运行超时 8 秒 |
| 执行时间 | 2026-10-29 19:31:06–19:34:08，Asia/Shanghai；不含构建和前置哈希校验 |
| 新鲜度 | 所有分析和运行回执均重新生成，没有复用旧执行结果 |

“策略库”是保存集合的名称：实际包含 6,706 份 strategy、146 份现代 indicator、399 份 legacyIndicator，以及 4 份缺失或混合声明源码。因此 7,255 不能解释为纯策略数量。

## 2. 全库结果与变化

| 指标 | 10 月 18 日基线 | 最新 main | 变化 |
| --- | ---: | ---: | ---: |
| 编译准入 | 6,153 | 6,153 | 0 |
| 编译拒绝 | 1,102 | 1,102 | 0 |
| 按旧口径运行完成 | 5,365 | 5,365 | 0 |
| 缺少 request 外部输入 | 642 | 641 | -1 |
| 缺少执行时钟 | 79 | 79 | 0 |
| 缺少汇率输入 | 49 | 49 | 0 |
| 其他运行拒绝 | 18 | 19 | +1 |

本轮没有超时或进程崩溃。编译准入集合、运行完成集合均未变化；5,365 份前后完成的源码，其 orders、trades、plots、alerts **数量**均相同。本比较没有证明所有输出值相同，也没有证明与 TradingView 等价。

唯一分类变化来自 `Webhook Starter Kit [HullBuster].pine`（SHA `3388abeba6d500d8195b3617958d07d28ef9a6bfb08f07acc26b8c591759b8e1`）：旧版首先报缺少 60 分钟 request 数据，新版先报 `E_NATIVE_RISK_UNSUPPORTED`。该脚本传入 `strategy.risk.allow_entry_in` 的配置变量后来又被重新赋值，命中了当前原生风险注册的不可变配置边界。它原先也没有运行完成；这不是新增的通过集回退。相关设计见 `docs/NATIVE_RISK_REGISTRATION.md`，不应通过静默忽略风险规则来提升通过率。

### 运行完成不等于没有错误诊断

为保持可比性，沿用旧统计定义：进程返回 0，且没有明确标记 `severity=error` 的诊断，即记为 `runtime_passed`。部分 broker 诊断只有 `code` 和 `message`，所以会落入这个集合。

本轮 5,365 份完成源码中，**311 份带有至少一种 `E_*` 诊断，5,054 份未出现 `E_*`**。按诊断码计数如下，同一源码可出现在多行：

| 诊断 | 源码数 |
| --- | ---: |
| `E_STRATEGY_EXIT_PRICE` | 173 |
| `E_STRATEGY_EXIT_TICKS` | 61 |
| `E_STRATEGY_MARGIN` | 45 |
| `E_STRATEGY_PRICE` | 35 |
| `E_STRATEGY_EXIT_QTY` | 14 |
| `E_UNSUPPORTED_ALERT_PLACEHOLDER` | 1 |
| `E_STRATEGY_CLOSE_QTY` | 1 |

这 311 份不是 311 个已确认的解释器缺陷；其中可能包含源码参数无效、资金或合成行情触发的正常拒绝。后续必须同时看订单、成交与结构化诊断。尤其应对 35 份 `E_STRATEGY_PRICE` 做归因，而不能仅凭相同诊断就认定全部由 `na` 问题造成。

## 3. 剩余缺口分层

1,102 份编译拒绝按**第一条错误**分组如下，行间互斥。错误码只是筛选入口，不直接证明源码错误或核心缺陷。

| 首个错误族 | 源码数 | 处理方向 |
| --- | ---: | --- |
| 解析 / 词法错误 | 178 | 区分抓取内容、损坏语法与解析器缺口 |
| `E_LEGACY_VERSION_FEATURE` | 168 | 核对源码声明版本，避免把现代功能无条件开放给旧版本 |
| `E_CALL_ARG_TYPE` | 115 | 优先从已证实的合法 `na` 参数入手 |
| `E_STRATEGY_MODE` | 103 | 核对 indicator / strategy 声明与调用是否合法 |
| `E_LANGUAGE_VERSION_DUPLICATE` | 83 | 排查混合脚本或重复版本声明 |
| `E_CALL_ARG_DUPLICATE` | 67 | 排查真实重复参数和绑定规则 |
| `E_CALL_ARG_NAME` | 62 | 优先 `nz`，另列声明能力缺口 |
| `E_IMPORT_MISSING_LIBRARY` | 51 | 缺少依赖源码；全部诊断中涉及此码的源码为 61 份 |
| 其他 | 275 | 按最小复现和规范证据逐项判断 |

641 份 request 输入缺失、79 份时钟缺失、49 份汇率缺失，首先属于本轮离线运行没有提供能力的边界。核心应保留明确的输入契约和诊断；数据获取、网络连接和生命周期管理仍归宿主。不能将这些数量直接算作 769 个核心兼容缺陷。

19 份其他运行拒绝进一步分为：数组越界 10、负权益导致负下单量 2、超出数量上限 1、执行预算耗尽 1、日期/时间参数拒绝 4、风险配置不支持 1。它们需要分别核对源码和 Pine 语义，不应统一放宽检查。

## 4. 修复顺序

优先级首先看结果正确性，再看证据强度、覆盖面和改动成本。下表的“仅该错误族”仅指当前全部编译错误都属于这一组，不保证修复后一定运行完成；候选组之间可能重叠。

| 顺序 | 问题 | 命中源码 / 仅该错误族 | 本轮证据与建议 |
| --- | --- | --- | --- |
| **1** | `entry/order` 的缺省价格与 `na` 语义 | **9 / 7** | 已确认语义拒绝及接受后丢单两层问题；先修参数值到订单类型的映射 |
| **2** | `nz` 的版本化命名参数 | **8 / 8** | 8 份 v5 源码仅改参数名的隔离副本全部准入，4 模式共 32 次运行完成；适合紧随其后的小修复 |
| **3** | 用户函数内 `alert()` 被过度禁止 | **10 / 8** | v5/v6 最小复现拒绝；需要保留实时频率、回滚及消息输出语义 |
| **4** | `fill_orders_on_standard_ohlc` 声明与填单契约 | **11 / 10** | 不能只接受参数名；需要区分图表合成 OHLC 和标准成交 OHLC，并由宿主提供数据 |
| 5 | 旧版成交量内建名称 | 4 / 3 | 下一轮核对版本、别名、首值和序列行为 |
| 6 | 用户函数内 `max_bars_back` | 3 / 0 | 官方存在函数内用法，但当前 3 份都叠加其他阻塞，收益较低 |

用户函数的禁用清单应逐项依据语言规则处理，不能整体移除副作用检查。参见 [Pine 用户函数规范](https://www.tradingview.com/pine-script-docs/language/user-defined-functions/) 和 [函数内 max_bars_back 示例](https://www.tradingview.com/pine-script-docs/v5/error-messages/)。标准 OHLC 填单属于真正的成交语义，参见 [策略属性说明](https://www.tradingview.com/support/solutions/43000628599-strategy-properties/)。

## 5. 第一项：订单价格 `na` 的确定性复现

[官方 strategy.entry 参考](https://www.tradingview.com/pine-script-reference/v5/#fun_strategy.entry) 和 [strategy.order 参考](https://www.tradingview.com/pine-script-reference/v5/#fun_strategy.order)将未指定的 limit/stop 价格表示为 `na`；两者都没有有效价格时为市价单。本轮用固定 6 根 K 线，在第 0 根发单，第 2 根 `close_all`，显式 `qty=1`，隔离了资金和默认数量因素。

矩阵为 v5/v6 × entry/order × long/short × 7 种价格形式，共 56 份自编最小脚本。被接受的脚本分别跑历史、增量历史、实时历史、实时 forming 四种 CLI 模式。

| 价格形式 | 脚本数 | 当前结果 |
| --- | ---: | --- |
| 省略 limit/stop | 8 | 32 次执行全部正常；每次 1 个公开订单记录、1 笔已平仓交易 |
| `limit=na` 或 `stop=na` | 16 | 全部在语义阶段被 `E_CALL_ARG_TYPE` 拒绝 |
| `limit=float(na)`、`stop=float(na)`、两者均为 `float(na)`、当根求值为 na 的序列 limit | 32 | 128 次执行全部返回 0，但 orders=0、trades=0，伴随 `E_STRATEGY_PRICE` |

这里不是“没有任何报错”：结构化 broker 诊断存在，但旧完成统计没有将它计为失败。**已确认的问题是合法缺省价格被解释成错误价格，改变了成交结果。** 本轮没有取得新的 TradingView 原生回执，结论依据官方参数语义、固定输入控制实验和代码路径。

最小片段如下；删除 `limit=float(na)` 后，在同一输入上恢复成交。完整自编脚本及 6 根 K 线保存在 `docs/evidence/strategy-corpus-20261029/`。

```pine
//@version=5
strategy("probe")
if bar_index == 0
    strategy.entry("E", strategy.long, qty=1, limit=float(na))
if bar_index == 2
    strategy.close_all()
plot(strategy.position_size)
```

代码定位（均基于本轮 main）：

- `crates/pine-builtins/src/namespaces/strategy.rs:22`、`:27`、`:80`、`:85`：limit/stop 的数值参数约束拒绝裸 `na`。
- `crates/pine-runtime/src/builtins/strategy.rs:371`、`:409`、`:432`：entry 以参数表达式是否存在选择数量路径及订单类型。
- 同文件 `:522`、`:556`：order 同样保留 `Some(NaN)`，随后按 Some/None 选择订单类型。
- `crates/pine-runtime/src/strategy/broker/pending_entries.rs:638` 起：无效挂单价格被拒绝，产生 `E_STRATEGY_PRICE`，没有生成订单。

修复应把 Pine 的缺省价格值正确映射成可选价格，再选择 market/limit/stop/stop-limit；不能只允许语义分析通过。价格表达式应只求值一次，默认数量计算也要使用最终订单类型。保留对负价格等真实无效输入的诊断。

建议验收范围：裸 na、类型化 na、序列 na；一个有效价格搭配另一个 na；默认数量；多空和反手；同 ID 挂单替换；OCA；实时回滚和外部订单意图。先检查 Rust 核心，再验证对外绑定实际传递的订单类型和成交数据。四种执行模式必须验证交易和诊断，不能只检查退出码。

对原库 7 份“仅该错误族”源码，也生成了省略价格与类型化 na 的独立副本：全部编译通过，其中 5 份在四模式均完成但没有触发成交，另 2 份仍缺少日线 request 输入。故本轮不能承诺修完会新增 7 份成功回测，更不能用这些零成交副本证明原策略行为正确。核心影响由上述 6 根 K 线矩阵独立证明。

## 6. 第二项：`nz` 参数名

当前 `crates/pine-builtins/src/namespaces/core.rs:697` 将参数注册成 `x, replacement`。官方迁移表明确列出 v4 的 `nz(x, y)` 与 v5 的 `nz(source, replacement)`，因此应做版本化绑定，而非把全局 `x` 简单改成 `source`。参见 [官方 v5 迁移表](https://www.tradingview.com/pine-script-docs/v5/migration-guides/to-pine-version-5/)。

本轮 v4/v5/v6 控制用例显示：`source=` 均被拒绝；`x=..., replacement=...` 均接受；`x=..., y=...` 均被拒绝；位置参数均接受。8 份命中原库源码都是 v5，其中包含 DuDu95、SSL Hybrid/QQE/Waddah、SuperTrend、fpemehd 及 3 份 FMZ 模板，不能解释为 8 个独立策略家族。

只在隔离副本中将 `nz(source=` 改为 `nz(x=`，8 份全部准入，四模式共 32 次运行完成。每份的历史、增量历史和实时历史输出一致；5 份 forming 输出的表格快照 barIndex 不同，相关源码有 `barstate.islastconfirmedhistory` 分支，因此不宣称四模式输出逐字一致。原源码与原库统计均未改动。

建议验收：版本合法命名、位置参数、混合位置/命名参数、重复参数、非法名称，以及数值类型和 na 替换的返回值。还应补 v4 `y=` 的合法用法，不能只修本次命中的 v5 模板。

## 7. 回执与复跑

完整本机证据目录：`E:\projects\pine-interpreter\target\strategy-corpus-retest-20261029`。

- `summary.json`：与冻结基线的集合和分类比较。
- `analysis-receipts.jsonl` / `runtime-receipts.jsonl`：7,255 / 6,153 条新回执。
- `results.json`、`triage.json`：来源、全部错误、候选命中和运行诊断分组。
- `source-integrity.json`、`source-manifest.json`：来源完整性与执行前后哈希。
- `runtime-count-differences.json`：完成集合的四类输出数量差异，为空数组。
- `order-na-results.json` / `order-na/`：56 份最小脚本、14 份原库衍生对照及逐次原始输出。
- `probe-results.json` / `probes/`：nz、UDF、初步订单控制用例及原始输出。
- `pine-compat.exe` / `bars.csv`：固定 CLI 和全库行情输入。

摘要 JSON 保存关键证据文件 SHA-256、候选源码 SHA 和来源定位；第三方完整源码不复制进仓库。完整回执位于本机 `target`，不是远端制品，需另行保留。

全量复跑需要仍可读取的原始来源和规范化源码；输出目录必须全新。在本工作树执行：

```powershell
cargo build --locked --release -p pine-cli
python scripts/retest_saved_strategy_library.py --baseline 'E:\projects\pine-interpreter\target\strategy-gap-final45-20261018\results.json' --bars 'E:\projects\pine-interpreter\target\strategy-corpus-retest-20261029\bars.csv' --cli target/release/pine-compat.exe --out target/strategy-corpus-rerun
python scripts/summarize_saved_strategy_retest.py target/strategy-corpus-rerun/results.json target/strategy-corpus-rerun/triage.json
```

不依赖第三方策略库的最小复现：

```powershell
target/release/pine-compat.exe analyze docs/evidence/strategy-corpus-20261029/entry-bare-na.pine --format json
target/release/pine-compat.exe run docs/evidence/strategy-corpus-20261029/entry-typed-na.pine --bars docs/evidence/strategy-corpus-20261029/bars.csv --chart-symbol V-BTC-SPOT --chart-timeframe 1
target/release/pine-compat.exe run docs/evidence/strategy-corpus-20261029/entry-omitted.pine --bars docs/evidence/strategy-corpus-20261029/bars.csv --chart-symbol V-BTC-SPOT --chart-timeframe 1
target/release/pine-compat.exe analyze docs/evidence/strategy-corpus-20261029/nz-v5-source.pine --format json
target/release/pine-compat.exe analyze docs/evidence/strategy-corpus-20261029/nz-v4-y.pine --format json
```

本轮完成的是准入重测、固定输入运行冒烟和候选缺陷隔离；没有重新测量性能、执行完整发布测试矩阵、提供缺失的宿主输入，或宣称盈亏、逐笔成交与 TradingView 全面等价。
