# Modern strategy remaining blockers review

状态：reviewed；不修改冻结 r1 或核心支持范围。
日期：2026-09-07。基线 HEAD：`31f16bb87`。

## 核对结果

| 冻结样本 | 原首个阻塞 | 本次核对 | 后续处置 |
| --- | --- | --- | --- |
| `original.sema.supported_imported_strategy_max_bars_back_declaration_udf_length.default` | 根 v6 / 库 v5 冲突 | `crates/pine-sema/tests/fixtures.rs` 的 `version_matched_fixture_library` 会为测试匹配版本；测量器原样注入库，因此输入不同。本地派生 v6 库后 analyze exit 0 | 属测试输入配置差异；后续建立注明派生来源/hash 的新 scenario，不静默修改 r1，也不移除版本校验 |
| `permissive.strategy.default` | 未知 `TODO` | 模板含未填交易条件；没有实际用户策略条件 | 保留为模板阻塞，不能用默认信号填充后宣称策略解锁 |
| `permissive.samplestrat.default` | 缺 `algomojo/automation/9` | 现有冻结输入未提供该库，且还有别名及后续未知标识符诊断 | 等准确库与完整场景输入；不让核心查网络，不认为补库必然可运行 |
| `permissive.technical_ratings_strategy.default` | 缺 `TradingView/TechnicalRating/3` | 找到本地镜像的 `TechnicalRating-v3.pine`；显式注入后 analyze exit 1，出现 `E_PARSE_EXPECTED` / `E_PARSE_EXPR` 等前端错误 | 修正“只有缺库”的判断：依赖可本地提供，但仍需独立前端根因分析和原创最小复现；本轮不扩大语法范围 |

TechnicalRating 本地探针与派生版本探针的 argv、退出码、原始诊断保存在
`.local/five-stage-evidence/stage5-review/technical-rating-library-probe.json` 和
`matched-version-probe.json`。原始镜像不复制到公共 fixture。

上述探针使用不同依赖输入，是补充调查，不替代原 r1 的 506/510 sema、508/512 run
结果。后续若把补齐的依赖纳入覆盖指标，必须冻结新 revision，并同时保留旧分母。

## 已发现导出文件，完整可比输入仍缺失

r1 及本地 combined manifest 的 `reference.status` 仍为 `none`。本次派生库、
语法探针、运行快照和性能结果都不是独立成交参考。随后已从用户指定下载目录
接收 Tester CSV 和 K 线，但源码、设置和时间映射尚未确认，不改变 G3 的 blocked 状态。
文件核验与 Chrome 登录阻塞见 [参考输入接收审计](STRATEGY_MODERN_G3_REFERENCE_INTAKE_AUDIT.md)。

准备首个 G3 场景时需要同一份输入包：

1. 授权使用的 v5/v6 策略原文及导入库，分别记录版本与 SHA-256。
2. 参数、strategy 设置、品种、图表周期/类型、时间单位、回测起止及预热范围。
3. 实际 OHLCV；如使用 request、Magnifier、session 或 execution clock，一并保存宿主输入。
4. 来自独立执行来源的真实成交/平仓记录，注明来源和导出日期，保留原始文件。
5. 将参考记录明确映射为比较器支持的平仓交易字段：`id`、`entryBarIndex`、
   `exitBarIndex`、`entryTime`、`exitTime`、`qty`、`entryPrice`、`exitPrice`、`profit`。
   缺字段或顺序不明确时标不可比；独立订单成交记录不能冒充平仓交易列表。
6. 不用本运行时输出生成参考，不以空交易表或最终收益相等代替逐笔验证。

当前比较器不覆盖逐 bar 权益及独立费用字段；首轮报告要明确字段覆盖，不能扩大为
所有账户行为已对齐。恢复 G3 时先检查输入一致性，再运行比较器，最后选择首个有
独立证据的执行根因。
