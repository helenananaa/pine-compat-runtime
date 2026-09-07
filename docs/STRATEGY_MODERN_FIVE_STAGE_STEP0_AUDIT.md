# Strategy Modern Five-Stage Step 0 Audit

阶段与切片 ID：批次 0 / 步骤 0.1–0.3  
状态：closed  
实际基线 HEAD：`9a68e4f02cf2186e1cc880df43ce1cd9743552c4`  
已有工作区变更：规划文档指针已更新；未跟踪 `AGENTS.md` 与
`docs/MODERN_STRATEGY_FIVE_STAGE_EXECUTION_PLAN.md`。不覆盖、删除或暂存
`AGENTS.md`。  
本轮目标：锁定环境、工作区边界、证据位置和基线门禁。  
非目标：不修改解释器接受范围或 broker 行为。  
文件白名单：

- `docs/MODERN_STRATEGY_FIVE_STAGE_EXECUTION_PLAN.md`
- `docs/STRATEGY_MODERN_FIVE_STAGE_STEP0_AUDIT.md`
- `docs/NEXT_INTERNAL_CAPABILITY_PLAN.md`
- `docs/PURE_INTERNAL_ROADMAP.md`
- `docs/README.md`
- `docs/STRATEGY_ACCURACY_NEXT_EXECUTION_PLAN.md`

语料 revision / manifest hash / 样本 ID：本步骤不冻结语料。  
问题、预期结果及证据等级：G0 要求工作区已分类、工具可用、基线验证通过。

## Workspace classification

Captured at `.local/five-stage-evidence/workspace.txt` after
`git check-ignore -v` confirmed `.local/` is ignored by `.git/info/exclude`.

| Item | Value |
| --- | --- |
| HEAD | `9a68e4f02cf2186e1cc880df43ce1cd9743552c4` |
| Recent commits | `9a68e4f02` session-window identity; `24af9a3a6` host session tests; `6a46c3d86` legal corpus inventory; `8120777b6` ordinary-chart gaps; `ff84b09aa` session window keys |
| Dirty tracked files | `docs/NEXT_INTERNAL_CAPABILITY_PLAN.md`, `docs/PURE_INTERNAL_ROADMAP.md`, `docs/README.md`, `docs/STRATEGY_ACCURACY_NEXT_EXECUTION_PLAN.md` |
| Untracked | `AGENTS.md` (leave unstaged), `docs/MODERN_STRATEGY_FIVE_STAGE_EXECUTION_PLAN.md` (this plan) |
| Cached diff | empty |
| `git diff --check` | clean |
| rustc | 1.95.0 (59807616e 2026-04-14) |
| cargo | 1.95.0 (f2d3ce0bd 2026-03-21) |
| python3 | 3.10.12 |
| node | v24.18.0 |
| maturin | `/home/helenanana/.local/bin/maturin` |
| rustup targets | `wasm32-unknown-unknown`, `x86_64-unknown-linux-gnu` |

`AGENTS.md` applies only at the repository root. No nested `AGENTS.md` files
were found under crates or scripts.

## Persistent evidence directory

- Persistent path: `pine-compat-runtime/.local/five-stage-evidence`
- Ignore proof: `git check-ignore -v .local/five-stage-evidence` matches
  `.git/info/exclude:9:.local/`
- Scratch copies: `/tmp/grok-goal-f03b0656af3b/implementer/step0/`
- Temporary collector used for the gate: `/tmp/pine-five-stage.NbIweR`

Do not commit `.local/` contents. Public audits cite hashes, counts, and
commands only.

## Baseline gate

Command:

```bash
set -o pipefail
scripts/verify.sh 2>&1 | tee "$PINE_FIVE_STAGE_EVIDENCE_DIR/baseline-verify.log"
```

| Observation | Result |
| --- | --- |
| Log path | `.local/five-stage-evidence/baseline-verify.log` (547942 bytes, 6729 lines) |
| Exit code | `0` |
| Classification | pass; not a tool-env, pre-existing, or goal-gap failure |
| Notable steps | `cargo fmt --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; structure/host-parity/legacy script unittests; WASM/Node smoke; isolated maturin wheel; `pytest python/tests` 647 passed |

`scripts/verify.sh` and `scripts/check_wasm_node.sh` require Rust workspace
tools, Node, the `wasm32-unknown-unknown` target, maturin, and Python venv.
Those tools were present and the gate completed.

## Support-range lock (read-only)

Current executable claims remain
`tests/fixtures/conformance.tsv`, CLI/Python/WASM goldens, and the closed
A/B/C, Stage 18g, and Stage 23 audits. Stage D1 still lacks a frozen
per-sample manifest and five independent metrics
(`docs/STRATEGY_MODERN_CORPUS_BEHAVIOR_AUDIT.md`). Independently validated
strategy result samples remain 0. This step does not change those claims.

Observable G0 acceptance, frozen before later stages:

1. Workspace files classified; `AGENTS.md` left unstaged.
2. Tools recorded; persistent ignored evidence directory exists.
3. `scripts/verify.sh` log exists with recorded exit code. Non-zero would be
   classified, never rewritten as pass. Actual exit code is 0.

剩余限制、下一步及前置条件：进入阶段 1.1。阶段 1 不得修改解释器接受范围或
broker 行为。独立 Tester 参考仍然缺失，不阻塞语料冻结与测量。
