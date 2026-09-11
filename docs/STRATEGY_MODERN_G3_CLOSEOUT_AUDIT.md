# G3 独立参考扩展与修复验收

状态：本轮 g3-chrome-r2 已 closed；全量 r1 参考和 B1 内部顺序仍未验证。

日期：2026-09-07。起始 HEAD：`45cbd6586a9e0b6d7e9f137945bacb2eaf9ab346`。
本轮使用 Windows 已登录 Chrome、TradingView 标准 K 线与原生成交导出。
范围是下表冻结的新原创场景，不把它们计入旧 r1 的参考分母。

## 独立参考

| 场景 | 内容 | 平仓笔数 | 逐笔比较 | 逐 bar 输出 |
| --- | --- | ---: | --- | --- |
| price-orders | 多空 limit、stop、stop-limit、bracket、trailing、OCA cancel/reduce | 13 | passed | 5 序列 × 511 bars passed |
| cash-fees | v6 多空、部分平仓、反转、固定每单费用 | 4 | passed | 6 序列 × 511 bars passed |
| cash-v5 | 相同费用场景的 v5 控制 | 4 | passed | 6 序列 × 511 bars passed |
| cash-recalc | 固定手续费反转与成交后重算交互 | 4 | passed | 6 序列 × 511 bars passed |
| b1-observable | 两种声明顺序，4 个固定同价碰撞窗口 | 8 | passed | 6 序列 × 511 bars passed |
| b1-recalc | 相同窗口开启成交后重算，增加佣金越界控制 | 8 | passed | 8 序列 × 511 bars passed |

合计 41 笔非空平仓交易、37 条序列的 18,907 个 bar 值。
成交 9 字段沿用已有比较器：id、entry/exitBarIndex、entry/exitTime、qty、
entry/exitPrice、profit；身份字段精确一致，数值绝对/相对容差仍为 1e-9。
各序列包含持仓、权益、净利润、平仓计数与首笔佣金；费用场景增加 mintick，
B1 增加佣金的 na/负索引/越界控制。公开成交 JSON 未添加 commission 字段。

原始文件和复现工具在 `.local/g3-closeout-20260907/`；本地 Git exclude 已明确
忽略 `.local/`。源码与数据不公开提交，参考从原始 Tester CSV/图表 CSV 生成，
不使用 runtime 输出制作预期。前轮 4 笔基准另在 `.local/g3-chrome-20260907/`。

图表为 OKX:BTCUSDT 15 分钟，UTC+8 显示，导出为 UTC epoch 秒；运行时转毫秒。
从图表加载的真实历史截取到 `1788785100000`，去除形成中 bar。
原始 CSV 数量结合“多头/空头进场”转换成既有有符号 qty，原交易编号顺序保留。
所有价格/时间/设置在比较前写入 Pine 与 `planned-cases.json`；每组 settings
在源码中明确。无 request 或 Magnifier 输入，指标不需要额外预热。
这里的标准历史参考不能证明真实 tick、Magnifier 或全市场账户行为。

## 已定位并修复的差异

1. **价格步长缺输入**：TradingView 输出 `mintick=0.1`，核心原来固定 0.01。
   两笔 50 tick trailing 各差 4.5 USDT，持仓相同但权益/利润随后持续偏离。
   `ChartContext` 现在接受正整数 `minMove/priceScale`，作用于 syminfo、tick
   止盈止损/trailing、滑点、limit verification、round_to_mintick 和 mintick
   字符串格式；同品种 request 继承网格，不向其他品种传播。
2. **每单固定佣金反转重复计费**：旧入口先 close 再 open，各收全额费用。
   现在该非零 cash-per-order 反转复用现有原子净额转换，按平仓/开仓数量
   分摊一次费用，保留 entry 方向准入与目标仓位限制。拒绝超额反转不产生
   半笔成交或额外费用。零费用及其他佣金模式的入口行为保持原有路径。
3. **无有效平仓索引的佣金**：TV v5/v6 的 commission(0)、负索引和越界控制
   返回 0；旧实现返回 na。修正仅作用于 commission，其他平仓字段仍返回 na。
4. **开仓手续费在净利润中的时点**：TV netprofit 立即扣除开仓费；旧值只求和
   已平仓利润。现在减去仍附着在未平仓份额上的费用，部分平仓和反转后不重复
   扣除。已平仓交易统计和原有权益快照字段的计算保持各自定义。

修复前报告保留于 `price-orders-before-fix.json`、`cash-fees-before-fix.json`
及首次 plot 比较记录。先修输入契约，再修费用语义；没有放宽比较器。

## 宿主中立契约

- Rust：`ChartContext::with_price_grid(min_move, price_scale)`，零值拒绝。
- CLI：`--chart-price-grid 1/10`，批量/增量/实时模式共用。
- Python：`request_bars={"$chart":{"minMove":1,"priceScale":10}}`；品种/周期
  仍由现有 `chart_symbol`/`chart_timeframe` 提供；网格拒绝布尔、分数、缺字段。
- WASM：现有 request JSON 的 `$chart` 增加 `minMove`、`priceScale`。
- 未提供时保持既有合成默认 1/100；不按 symbol 猜交易所规格。其他品种 request
  仍使用既有默认元数据，不能把这次 main-chart 输入扩展解释为完整品种规格服务。
- 没有引入网络、数据库、CandleScope 依赖、凭据或调度逻辑；输出 schema 不变。

## 验证与快照审查

新增离线测试覆盖网格驱动的 trailing/舍入/标量和数组格式、多空反转费用分配、
拒绝反转不变性、佣金越界及 CLI/Python/WASM 输入校验。
6 组完整历史分别执行 incremental、realtime-history、forming replacement；
另对移动止损和费用反转的 5 个活动 bar 前缀执行替换/确认，共 23 次模式比较一致。
这是宿主回滚验证，不是 TradingView 实时 tick 参考。

6 个允许更新的快照仅涉及：无有效平仓的 commission 由 null 改为 0，及开仓后
netprofit 扣除佣金。它们是 `runtime_strategy_closedtrades_fields*` 和四个
`runtime_strategy_commission_*`。更新过程中出现的 `runtime_math.json` 平台
libm 最末位差异已撤回，未混入策略变更。

完整门禁、真实 Python/WASM 场景比较与最终哈希见本地 `verification-summary.json`。
初次门禁发现字符串模块超过 1500 行；拆分集合格式化子模块后结构检查通过。
最终 `scripts/verify.ps1` 在非 UPDATE_SNAPSHOTS 模式 exit 0：Rust 6,530 项、
Python 新 wheel 672 项、工具 101 项，以及真实 WASM/Node smoke 全部通过。
6 组 TradingView 输入另外通过 12 次 Python/WASM 全输出比较。

## 语料回写与剩余边界

本轮实际重测公共 r1：480 脚本 / 482 场景；解析 480/480，语义 479/480，
运行 481/482，独立参考仍是 **0/482**。剩余版本冲突不由本轮交易语义修改消除。
93 个旧负向标签中 4 个现在被接受（OCA 与 process_orders_on_close/recalc），
保留原始分类并单列，未通过修改分母隐藏旧清单与现有能力的差异。
新增 6 组是独立 `g3-chrome-r2` 测量批次：可比 6/6、一致 6/6；不得将它们
写成旧 r1 的 100% 准确率。

B1 在本轮覆盖的最终交易和逐 bar 输出一致，但这些观察仍不能证明 TradingView
内部 entry/exit 先后，也不能证明其原子性。继续保留 `UNVERIFIED_INTERNAL_ORDER`，
不添加全局家族优先级；creation sequence/stable key 仍只是本运行时确定性契约。
这是一项明确的可观察性边界，不是可以靠勾选清单完成的实现缺口。

本轮未提交、推送、合并或发布。开始时已有 Windows 工具链暂存修改；保留其内容
与暂存范围。验证对象是记录的工作区，不宣称 clean-HEAD 或远端 CI 已验收。
