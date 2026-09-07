# Strategy Modern G2 v5 UDF box.new Audit

阶段与切片 ID：阶段 2 / 切片 `udf_v5_box_new` / `function_side_effect`  
状态：closed  
实际基线 HEAD：`a962497b5`（G2 `strategy_commission_value_default_percent` 之后）  
已有工作区变更：未跟踪 `AGENTS.md` 保持不暂存。  
本轮目标：v5/v6 UDF 体内允许 `box.new` 构造，对应 captain_backtest_model 的
`rr_boxes` 中 `box.new(...)` 形式。  
非目标：不把 `array.unshift`、`label.new`、`box.set_*`、plot/strategy 扩进 UDF；
不伪造 G3 Tester 参考。  
文件白名单：`crates/pine-sema`、`pine-runtime`、`pine-cli`、`pine-wasm`、
`python/tests`、`scripts/host_parity_required.txt`、相关 fixture/snapshot、
`tests/fixtures/conformance.tsv`、`docs/BUILTIN_SIGNATURES.md`、
`docs/LANGUAGE_SCOPE.md`、本 audit、执行计划状态行。

语料 revision：`modern-strategy-r1`  
manifest SHA-256：`cd41cf7fd95a307ee8dacc983bf9ffd0bda37027dc8397ea78abc8c9b68446bb`

## 问题与证据

G3 仍 blocked：combined-manifest 全部 `reference.status=none`（607 行）。
`.local/` 无 Tester/fills 包。本切片是文档允许的独立 G2 工作。

合并排名可实施语言根因：`E_UNSUPPORTED_FEATURE` `function_side_effect`
（`captain_backtest_model_[tfo]`）。首个诊断为 UDF 内 `box.new`（drawing calls），
随后是 `array.unshift`。本切片只取 `box.new`。

项目依据：v4 UDF 已允许 `label.new`/`line.new`；`box.new` 运行时顶层已实现。
官方 v5 允许函数内创建 drawing。v4 UDF `box.new` 继续拒绝。

## 设计

接受：v5/v6 `function_depth > 0` 时 `box.new`。运行时与顶层 `box.new` 相同，
含 forming-bar 回滚。

继续拒绝：v4 UDF `box.new`；v5 UDF `label.new`、`array.unshift`、plot/strategy。

## 实际修改

- `allows_udf_output_or_declaration_side_effect`：v4 既有子集 + v5/v6 `box.new`。
- 正向：`supported_udf_box_new.pine`、`udf_box_new.pine`、
  `strategy_udf_box_new.pine`、realtime rollback。
- 负向：`unsupported_udf_box_new_v4.pine`；既有
  `unsupported_drawing_function_side_effect.pine`（label.new）仍拒绝。
- CLI/Python/WASM 快照与 host_parity。
- conformance 新行 `udf.v5.box_new`。

## 验证

| 命令 | 退出码 | 有效测试 |
| --- | --- | --- |
| `cargo test -p pine-sema --test fixtures accepts_supported_udf_box_new_fixture` | 0 | 1 |
| `cargo test -p pine-sema --test fixtures reports_unsupported_udf_box_new_v4_fixture` | 0 | 1 |
| `cargo test -p pine-sema --test fixtures reports_unsupported_drawing_function_side_effect_fixture` | 0 | 1 |
| `cargo test -p pine-runtime --lib udf_box_new_creates_boxes_on_each_historical_bar` | 0 | 1 |
| `cargo test -p pine-runtime --test realtime udf_box_new_fixture_rolls_back_forming_boxes` | 0 | 1 |
| `cargo test -p pine-runtime --test incremental runtime_fixtures_match_incremental` | 0 | 1 |
| `cargo test -p pine-cli runtime_outputs_match_golden_snapshots` | 0 | 1 |
| `python3 scripts/check_host_parity.py` | 0 | 870 CLI / 574 required |
| `scripts/verify.sh` | 0 | workspace tests + 661 pytest；日志 `.local/five-stage-evidence/stage2i/verify.log` |

## 语料前后

Combined freeze（含本地 permissive，不入库）：

| Metric | Before | After |
| --- | --- | --- |
| parse | 510/510 | 510/510 |
| sema | 505/510 | 505/510 |
| run | 507/512 | 507/512 |
| comparability | 0/512 | 0/512 |
| consistency | N/A | N/A |

阶段转换：无脚本升到 run。

首个诊断转换：

- `permissive.captain_backtest_model_[tfo].default`：UDF `box.new` drawing
  calls → `array.unshift` 集合变更。sema 仍 failed。

公开 min-repro `udf_box_new.pine` parse/sema/run 通过。下一语言形式是 v5 UDF
`array.unshift`。`E_IMPORT_MISSING_LIBRARY` 仍是 host 库源，不是核心语言切片。
G3 仍 blocked。

剩余限制：UDF 内 `array.unshift`、`label.new`、`box.set_*`、plot/strategy 仍拒绝。
