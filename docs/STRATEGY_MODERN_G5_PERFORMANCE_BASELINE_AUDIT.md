# Strategy Modern G5 Performance Baseline Audit

阶段与切片 ID：阶段 5 / 5.1–5.2 基线；5.3–5.4 无优化合入  
状态：历史初步测量（原“基线完成”表述过宽，已由 2026-09-07 复核更正）
实际基线 HEAD：`d7e048960`  
本轮目标：对已正确样本分开测量编译、历史运行、增量、形成中 bar 与序列化。  
非目标：不通过跳过风险检查或减弱保留限制换速度；本轮不合入优化。

后续：[阶段 5 基线复核审计](STRATEGY_MODERN_G5_BASELINE_REVIEW_AUDIT.md)。
旧记录保留原始运行事实，不再作为真实 append、独立 forming replacement 或完整资源测量验收。

冻结集合：

- `tests/fixtures/runtime/strategy_trade_counts.pine`
- `tests/fixtures/runtime/strategy_entry.pine`
- `tests/fixtures/runtime/generic_input.pine`

规模：16、64 与 256 根合成 bars，seed=1。证据跑 warmup=1，iters=3；工具默认 warmup=2，iters=10。

两次运行结果 hash 一致。`optimizationCommitted=false`。状态字符串：`基线完成`。

相位均单独计时：compile、historicalRun、incrementalAppend（标注为 prefix rerun，不是 VM append_bar）、formingReplace（realtime seed + forming/confirm）、outputSerialization。

环境见 `.local/five-stage-evidence/stage5/bench-1.json` `environment`。

验证：`python3 -m unittest scripts/tests/test_benchmark_modern_strategy.py` 3 tests, exit 0。
`scripts/verify.sh` 接入该单测，完整门禁 exit 0（`.local/five-stage-evidence/stage5/verify.log`）。

未实施热点优化，因为先完成可复现基线；没有 A/B 收益，故不宣称提速。
