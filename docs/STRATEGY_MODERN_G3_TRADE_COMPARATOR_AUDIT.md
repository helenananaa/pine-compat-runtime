# Strategy Modern G3 Trade Comparator Audit

阶段与切片 ID：阶段 3 / 3.1–3.2 比较器；3.3–3.5 成交修复 **blocked**  
状态：partial（比较器 closed；独立参考缺失，G3 准确性验收 blocked）  
实际基线 HEAD：`6b757308e`  
已有工作区变更：未跟踪 `AGENTS.md` 保持不暂存。  
本轮目标：实现逐笔交易比较器及注入错误测试。  
非目标：不把参考格式写入 runtime；不扩大公共 StrategyTrade schema；不伪造 Tester 输出。  
文件白名单：

- `scripts/compare_strategy_reference_outputs.py`
- `scripts/tests/test_compare_strategy_reference_outputs.py`
- `scripts/verify.sh`
- `docs/STRATEGY_MODERN_G3_TRADE_COMPARATOR_AUDIT.md`
- `docs/MODERN_STRATEGY_FIVE_STAGE_EXECUTION_PLAN.md`

语料 revision：`modern-strategy-r1`。所有公共样本 `reference.status=none`。

## 比较契约（查看目标结果前冻结）

| 字段 | 规则 |
| --- | --- |
| `id`, `entryBarIndex`, `exitBarIndex`, `entryTime`, `exitTime` | 精确相等 |
| `qty`, `entryPrice`, `exitPrice`, `profit` | \|Δ\| ≤ 1e-9 或相对 1e-9 |
| `commission`, `direction`, `exitId` | 公共 JSON 无覆盖；声明这些字段则 `incomparable` |
| 匹配 | 按运行时成交列表顺序，不按 ID 或总收益重排 |
| 同时间且身份相同 | `duplicate_trade_identity` 或 `ambiguous_same_time_order` |

## 验证

`python3 -m unittest scripts/tests/test_compare_strategy_reference_outputs.py`

12 tests, exit 0。覆盖错价、错量、漏单、重复/额外单、错时间、费用未覆盖、空交易、重复身份、部分参考、未覆盖字段。均不得判 pass。

比较器已登记为语料测量的 `compare_reference` 接口。无参考样本仍为 `not_run`/`no_reference`。

## G3 准确性 blocked

已尝试：审计记录、`.local/` 文件名搜索、upstream mirrors。无 Tester/独立成交包。
证据：`.local/five-stage-evidence/stage3/blocked.txt`。

2026-09-07 再查（HEAD `8737a28ba`，G2 `box_call_result_set_right` 之后）：公共 r1 577
行与 combined 607 行 `reference.status` 仍全部为 `none`；263 个 run-pass 且
`tradeCount>0` 的样本只是本解释器自金；`.local/legacy-corpus-r2` 的
`tv-r2-*.json` 是 request.security CSV 映射，不是 Tester 成交表。未把 runtime
快照或手算账本当作独立参考。

恢复条件：授权的 v5/v6 策略 + 冻结 K 线 + 独立预期成交（TradingView Strategy
Tester 或同等来源）。在此之前不实施成交/状态根因修复。

剩余限制：无独立参考则结果一致性保持 N/A。阶段 4 已 deferred；阶段 5 为测量基线。
整体五阶段目标在 G3 缺独立参考时不得标为完成。


2026-09-07 性能基线复核后的补充：本地依赖与版本探针见
[剩余阻塞复核](STRATEGY_MODERN_REMAINING_BLOCKERS_REVIEW.md)，其中列出首个 G3
独立输入包所需内容。新性能数据和本地派生输入不算独立成交参考，G3 状态不变。

后续接收更新：已从用户指定的 Windows 下载目录保存 8 份 Tester CSV 与 1 份 K 线 CSV。
旧文“无成交包”仅描述此前搜索结果；当前缺的是匹配的源码、设置与时间映射。
见 [参考输入接收审计](STRATEGY_MODERN_G3_REFERENCE_INTAKE_AUDIT.md)。
G5 局部性能优化已经验收，不改变 G3 的 blocked 状态。
