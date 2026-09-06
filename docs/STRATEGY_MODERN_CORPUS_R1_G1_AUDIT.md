# Strategy Modern Corpus R1 G1 Audit

阶段与切片 ID：阶段 1 / 1.1–1.6 / corpus revision `modern-strategy-r1`  
状态：closed  
实际基线 HEAD：`2d0c6e80f8e948c694580420878a26c1fd8fd98e`（步骤 0 提交）  
结束工作区：阶段 1 工具、公共 freeze 与文档；解释器 crate 无改动。  
已有工作区变更：未跟踪 `AGENTS.md` 保持不暂存。  
本轮目标：冻结合法 v5/v6 策略清单、去重分母、分阶段测量器和五项独立指标。  
非目标：不修改解释器接受范围或 broker 行为；不伪造 Tester 参考。  
文件白名单：

- `scripts/analyze_modern_strategy_corpus.py`
- `scripts/tests/test_analyze_modern_strategy_corpus.py`
- `scripts/verify.sh`
- `tests/fixtures/modern-strategy-corpus/README.md`
- `tests/fixtures/modern-strategy-corpus/r1/*`
- `docs/STRATEGY_MODERN_CORPUS_R1_G1_AUDIT.md`
- `docs/STRATEGY_MODERN_CORPUS_BEHAVIOR_AUDIT.md`
- `docs/STRATEGY_ACCURACY_NEXT_EXECUTION_PLAN.md`
- `docs/MODERN_STRATEGY_FIVE_STAGE_EXECUTION_PLAN.md`

语料 revision / manifest hash / 样本 ID：

| Item | Value |
| --- | --- |
| corpus revision | `modern-strategy-r1` |
| public manifest | `tests/fixtures/modern-strategy-corpus/r1/manifest.jsonl` |
| manifest SHA-256 | `cd41cf7fd95a307ee8dacc983bf9ffd0bda37027dc8397ea78abc8c9b68446bb` |
| scripts N | 480 |
| scenarios M | 482 |
| negative controls | 93 |
| excluded | 2 (v4 strategy fixtures) |
| local permissive overlay | `.local/five-stage-evidence/stage1/permissive-candidates.jsonl` (not git) |

问题、预期结果及证据等级：G1 要求可重跑清单、hash、去重、五项指标、原始结果和根因排名。没有独立参考时一致性为 `N/A`，不得缺报。

## Inventory

Legal sources inspected:

1. In-repo original fixtures (`licenseClass=original`, publish allowed). v5/v6
   `strategy()` files under `tests/fixtures/{runtime,sema,regressions,profile,realtime,syntax}`.
   `unsupported_*` names are negative controls. v1–v4 strategy fixtures are excluded.
2. Local MIT GitHub mirrors under `.local/upstream-pine-candidates` (permissive,
   not copied into git). 30 v5/v6 strategy scripts were measured locally as
   `modern_strategy`; 47 other strategy files were version-excluded. Official
   TradingView copies inside those mirrors stay unpublished.
3. `.local/legacy-corpus-r2` / `r3`: previously inspected, zero `strategy(`
   files, not ingested.

Same-source SHA-256 duplicates were merged to one `scriptId`. The only
near-duplicate group is three scenarios of
`strategy_session_overnight_filled_orders.pine` (same source, different session
windows). Missing OHLCV for eligible public samples: 0. Default
`tests/fixtures/runtime/bars.csv` is recorded as `synthetic_smoke`.

## Measurement tool

`scripts/analyze_modern_strategy_corpus.py` runs `fmt-ast`, `analyze`, and
`run` through subprocess argv arrays. Parse pass is diagnostic-based;
`fmt-ast` exit 0 is not treated as parse success. Comparability uses a
registered strategy-reference comparator; the comparator is not implemented in
this stage, so reference-backed samples are `not_run` / `tool_unavailable`.
No reference is `not_run` / `no_reference`, never consistency-pass.

In-repo tests (`python3 -m unittest scripts/tests/test_analyze_modern_strategy_corpus.py`):
16 tests, exit 0, covering parse fail with `fmt-ast` exit 0, sema fail, run
fail, missing bars, timeout, duplicate sample id, no/partial/mismatched
reference, missing comparator, path with spaces, and hash invalidation.
Wired into `scripts/verify.sh`.

## Five metrics (public freeze, two identical runs)

Commands (repo root, `target/debug/pine-compat`):

```bash
python3 scripts/analyze_modern_strategy_corpus.py measure \
  --manifest tests/fixtures/modern-strategy-corpus/r1/manifest.jsonl \
  --output .local/five-stage-evidence/stage1/metrics-1.json
python3 scripts/analyze_modern_strategy_corpus.py measure \
  --manifest tests/fixtures/modern-strategy-corpus/r1/manifest.jsonl \
  --output .local/five-stage-evidence/stage1/metrics-2.json
```

Stage results and diagnostic classes matched between run 1 and run 2.
Duration fields are stored separately in the full reports.

| Metric | Rate | passed | failed | missing_input | not_run | excluded | Denominator |
| --- | --- | ---: | ---: | ---: | ---: | ---: | --- |
| parse | 480/480 | 480 | 0 | 0 | 0 | 0 | N scripts |
| sema | 479/480 | 479 | 1 | 0 | 0 | 0 | N scripts (conditional on parse: 479/480) |
| run | 481/482 | 481 | 0 | 0 | 1 | 0 | M scenarios (ready inputs: 481/481) |
| output comparability | 0/482 | 0 | 0 | 0 | 482 | 0 | M scenarios |
| result consistency | N/A | 0 | 0 | 0 | 0 | 0 | comparable scenarios = 0 |

Negative-control expected-reject rate: 89/93. The four historical
`unsupported_strategy_*` fixtures that now parse and analyze are mixed-OCA
and `process_orders_on_close` forms closed in later stages; they remain
labeled negative controls and are not counted as consistency evidence.

The one eligible sema failure is
`supported_imported_strategy_max_bars_back_declaration_udf_length` with
`E_LANGUAGE_VERSION_CONFLICT` (root v6 vs library `user/udt/1` at v5). The
crate test harness rewrites library versions; this measurement does not.

## Combined ranking including local permissive v5/v6 strategies

Local overlay is not a public fixture. Combined N=510, M=512. Parse 510/510,
sema 499/510, run 501/512, comparability 0/512, consistency N/A.

Root-cause ranking by affected scripts (first blocking diagnostic):

| Class | Stage | Code | Scripts | Scenarios |
| --- | --- | --- | ---: | ---: |
| language_type | sema | E_CALL_ARG_TYPE | 3 | 3 |
| host_data_contract | sema | E_IMPORT_MISSING_LIBRARY | 2 | 2 |
| language_type | sema | E_CALL_ARG_VALUE | 2 | 2 |
| builtin | sema | E_UNKNOWN_SYMBOL `TODO` | 1 | 1 |
| language_type | sema | E_CALL_ARG_NAME | 1 | 1 |
| language_type | sema | E_LANGUAGE_VERSION_CONFLICT | 1 | 1 |
| language_type | sema | E_UNSUPPORTED_FEATURE `function_side_effect` | 1 | 1 |

`E_CALL_ARG_TYPE` is the highest-count language blocker among measurable
modern samples. Typical diagnostic: `input` `defval` expects a const scalar
but received `series float`. Host-missing libraries are not language slices.
No independently validated fill samples exist; consistency stays N/A.

## Interpreter impact

`git diff` for this stage does not touch `crates/`, broker code, or
conformance rows. Public JSON schemas are unchanged.

验证命令、退出码、有效测试数量：

- `python3 -m unittest scripts/tests/test_analyze_modern_strategy_corpus.py` → 16 tests, exit 0
- two public measure runs → summaries equal, exit 0
- `git diff --check` → clean

日志位置：`.local/five-stage-evidence/stage1/` and
`/tmp/grok-goal-f03b0656af3b/implementer/stage1/`.

剩余限制：无独立 Tester 参考；本地 permissive 源不入库；synthetic bars 不是真实回测；
四个过时 `unsupported_*` 名称仍通过语义分析。下一步：按排名实施
`E_CALL_ARG_TYPE` 语言切片（阶段 2），比较器与独立参考仍待阶段 3。
