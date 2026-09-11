# Strategy Modern G4 Account Demand Deferred Audit

2026-09-07 后续：新 G3 参考已证明价格网格输入和费用处理的具体缺口，
本轮已补 main-chart price grid，并修复费用分配和计入时点。见
[独立参考审计](STRATEGY_MODERN_G3_CLOSEOUT_AUDIT.md)。下文是此前无合格需求
时的历史决策；货币换算、合约乘数等其他 deferred 项未因此实现。


阶段与切片 ID：阶段 4 / 需求清单  
状态：deferred  
实际基线 HEAD：`6b757308e` 之后的阶段 3 工作区  
本轮目标：仅当冻结语料证明需要时扩展账户/精度/报告能力。  
非目标：不为完成计划添加公共字段或账户模型。

## 需求清单

| 候选 | 语料证据 | 分类 | 本轮决定 |
| --- | --- | --- | --- |
| 默认下单数量 | 公共 fixture 已覆盖 fixed/cash/percent_of_equity | 已有能力 | 不重复开发 |
| 品种精度 / 合约乘数 | 无冻结样本携带交易所规格或独立参考 | 无合格需求 | deferred |
| 货币换算 | 仅 1 个本地 permissive 脚本在 sema 拒绝 `currency`≠NONE | 语言阻塞，非已验证成交账本 | deferred |
| 佣金类型扩展 | 同上脚本 `commission_value`/`commission_type` sema 拒绝 | 语言阻塞 | deferred |
| 公共成交 `commission` 字段 | 比较器需要但公共 `StrategyTrade` 无该字段；计划禁止为比较器扩 schema | 验证工具需求 | deferred |
| `strategy(..., format/precision)` | 1 个本地脚本 `E_CALL_ARG_NAME` | 声明参数语言切片，属阶段 2 | deferred（非本阶段账户能力） |

没有“策略已运行且独立证明需要新账户字段”的样本。成交基础未通过独立 Tester 验证（G3 blocked）。
因此本轮合格需求清单为空。

**G4：** 空合格需求清单按计划记 deferred，不宣称账户能力已补齐。阶段 5 可以开始。

验证：无新 ledger 实现，故无新边界测试。不得把 deferred 项写入 supported。
