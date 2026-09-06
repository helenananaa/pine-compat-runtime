# Modern strategy corpus revision r1

Frozen v5/v6 strategy inventory used by
`scripts/analyze_modern_strategy_corpus.py`.

- `r1/manifest.jsonl` is the public original-fixture freeze. Paths are in-repo
  fixtures only.
- Local permissive mirrors, if measured, stay under `.local/` and are not
  public fixtures.
- Default `tests/fixtures/runtime/bars.csv` coverage is `synthetic_smoke`. It
  is not a real-symbol backtest and not an independent correctness oracle.
- There are no independent Tester reference outputs in this revision.

This directory does not change interpreter acceptance or broker behavior.
