# G3 Chrome 独立参考闭环

同日后续：新 g3-chrome-r2 六组参考（41 笔）和四项修复已完成本轮验收；
[最终记录](STRATEGY_MODERN_G3_CLOSEOUT_AUDIT.md)。下文保留此前阶段的历史状态。


日期：2026-09-07。状态：首个独立场景 passed；G3 整体仍 partial。

本次连接已登录的 Windows Chrome，在 TradingView 实际执行原创 v6 策略
`G3 Chrome reference 20260907`。原始源码、策略属性、Tester CSV、同图表
OHLCV 导出、时间映射、重放脚本、比较结果及截图保存在
`.local/g3-chrome-20260907/`，校验清单为 `manifest.json`。
这些原始数据保持为本地证据，不加入公共语料或改变冻结 r1 分母。

## 输入与独立性

- 图表：OKX:BTCUSDT，15 分钟，标准 K 线；显示时区 UTC+8。
- 策略：无导入库、无参数、无 request、无 Magnifier、无外部执行时钟。
- 资金 1,000,000 USDT；固定数量；pyramiding=1；佣金与滑点 0；
  多空 margin=100；收盘计算，下一 tick 执行，不在成交后重算。
- 信号使用固定 epoch 毫秒，2026-09-04 00:00–02:30 UTC；
  最后平仓为 02:45 UTC。此前没有订单或需要预热的指标。
- Tester 显示范围 2026-02-01–2026-09-07；本地仅重放已导出的 511 根连续
  已收盘 K 线，覆盖所有信号和成交及其前置无交易区间。并不声称重放了完整
  Tester 图表历史。最后一根形成中的 K 线明确剔除。
- 第一次导出只覆盖较新的已加载 K 线，不能用于此交易场景。通过图表“前往到”
  加载 9 月 3 日附近历史后重新导出 `chart-history-raw.csv`，共 512 行。

参考成交仅来自 TradingView CSV，不从本解释器输出或手算利润生成。
交易编号按 CSV 原顺序 1–4 映射，不按运行时结果重排。
CSV 时间按已观察到的 UTC+8 转 epoch 毫秒，bar 索引按冻结 CSV 从零映射；
不是 TradingView 全历史绝对 bar_index。

CSV 数量是绝对值，方向由“多头进场/空头进场”表示；runtime 的平仓 qty
带方向符号（`close_orders.rs` 的 `signed_quantity`）。首次未转换方向的比较
在两笔空头 qty 上失败，原报告保留。随后仅依据 CSV 方向字段映射符号，
不修改 runtime、比较器或数值容差。

## 结果

| 场景 | TradingView 平仓笔数 | 对照结果 |
| --- | ---: | --- |
| 多头市价进场后 close | 1 | passed |
| 空头部分 close | 1 | passed |
| 空头剩余仓位被反向 entry 平掉 | 1 | passed |
| 反向建立的多头 close_all | 1 | passed |

4/4 笔，9 字段：id、entry/exitBarIndex、entry/exitTime、qty、
entry/exitPrice、profit；0 差异，沿用绝对/相对 1e-9 容差。
额外核对 TradingView 导出的 `G3 position`：511/511 根持仓值一致。
比较器原有 12 个注入错误测试通过。

运行基线 HEAD `45cbd6586a9e0b6d7e9f137945bacb2eaf9ab346`，Windows 原生
debug CLI；工作区已有另一轮暂存修改，因此不是 clean-HEAD qualification，
也不是 WSL 本轮执行证据。manifest 记录工作区差异与二进制 hash。

复现（从仓库根目录；WSL 可传入其原生 CLI 的路径）：

```powershell
python .local/g3-chrome-20260907/reproduce.py
```

## 状态边界

“无法连接已登录 Chrome，完全没有可比独立场景”的阻塞已解除；可以继续
按同一采集流程扩展 G3。这个新增原创最小场景不替代 modern-strategy-r1
语料准确性验收；不证明限价、止损、同价内部顺序、费用、逐 bar 权益、风险、
实时或 Magnifier 已对齐。此次没有修改解释器成交语义。

旧 Stage 18g B1 的内部顺序问题仍未验证，本次四笔交易不用于推断它。
