"""Compare frozen corpus runs, including broker diagnostics hidden by pass counts."""
import argparse
from collections import Counter
import json
from pathlib import Path

from retest_saved_strategy_library import category, digest, write_json


def unique(data):
    return {r["normalized_sha256"]: r for r in data["rows"] if r.get("normalized_sha256")}


def codes(row):
    runtime = row.get("runtime", {})
    return sorted({d["code"] for d in runtime.get("diagnostics", [])
                   + runtime.get("strategy_diagnostics", []) if d.get("code", "").startswith("E_")})


def stats(rows):
    completed = [r for r in rows.values() if category(r) == "runtime_passed"]
    return {"status_counts": dict(Counter(category(r) for r in rows.values())),
            "compile_accepted": sum(r["analysis"]["executable"] for r in rows.values()),
            "completed_with_e_diagnostics": sum(bool(codes(r)) for r in completed),
            "completed_without_e_diagnostics": sum(not codes(r) for r in completed),
            "completed_diagnostic_source_counts": dict(Counter(c for r in completed for c in codes(r)))}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("before", type=Path)
    parser.add_argument("after", type=Path)
    parser.add_argument("--before-triage", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    before_data, after_data = [json.loads(p.read_text(encoding="utf-8")) for p in (args.before, args.after)]
    before, after = unique(before_data), unique(after_data)
    assert set(before) == set(after), "Source inventory changed"
    assert before_data["metadata"]["bars_sha256"] == after_data["metadata"]["bars_sha256"]
    triage = json.loads(args.before_triage.read_text(encoding="utf-8"))
    counts_changed, diagnostics_changed = [], []
    fields = ("orders", "trades", "plots", "alerts")
    for sha, old in before.items():
        new = after[sha]
        if category(old) != "runtime_passed" or category(new) != "runtime_passed":
            continue
        old_counts = {k: old["runtime"][k] for k in fields}
        new_counts = {k: new["runtime"][k] for k in fields}
        info = {"sha256": sha, "path": old.get("path"), "normalized_path": old["normalized_path"]}
        if old_counts != new_counts:
            counts_changed.append({**info, "before": old_counts, "after": new_counts,
                                   "before_diagnostics": codes(old), "after_diagnostics": codes(new)})
        if codes(old) != codes(new):
            diagnostics_changed.append({**info, "before": codes(old), "after": codes(new)})
    groups = {}
    for name in ("entry_order_explicit_na", "nz_named_source", "udf_side_effect:alert", "standard_ohlc_fill_contract"):
        shas = [r["sha256"] for r in triage["candidate_groups"].get(name, [])]
        groups[name] = {"sources": len(shas), "after": dict(Counter(category(after[s]) for s in shas)),
                        "rows": [{"sha256": s, "status": category(after[s]),
                                  "errors": [d for d in after[s]["analysis"]["diagnostics"]
                                             if d.get("severity", "").lower() == "error"]} for s in shas]}
    result = {"before": {"path": str(args.before), "sha256": digest(args.before),
                          "metadata": before_data["metadata"], **stats(before)},
              "after": {"path": str(args.after), "sha256": digest(args.after),
                         "metadata": after_data["metadata"], **stats(after)},
              "candidate_outcomes": groups, "counts_changed": counts_changed,
              "diagnostics_changed": diagnostics_changed,
              "newly_accepted": [s for s in before if after[s]["analysis"]["executable"] and not before[s]["analysis"]["executable"]],
              "newly_rejected": [s for s in before if before[s]["analysis"]["executable"] and not after[s]["analysis"]["executable"]],
              "runtime_pass_lost": [s for s in before if category(before[s]) == "runtime_passed" and category(after[s]) != "runtime_passed"],
              "limitations": "Default-input synthetic-data smoke, not native equivalence. runtime_passed retains frozen exit/severity definition. Candidate groups overlap; count changes do not compare every output value."}
    write_json(args.out, result)
    print(json.dumps({"before": stats(before), "after": stats(after),
                      "counts_changed": len(counts_changed), "diagnostics_changed": len(diagnostics_changed),
                      "newly_rejected": len(result["newly_rejected"]), "runtime_pass_lost": len(result["runtime_pass_lost"])}, ensure_ascii=False))


if __name__ == "__main__":
    main()
