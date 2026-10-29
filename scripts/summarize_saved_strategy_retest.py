"""Group fresh saved-library receipts by source, not diagnostic occurrence."""
import argparse
from collections import Counter, defaultdict
import json
from pathlib import Path

from retest_saved_strategy_library import category, digest, errors, write_json


def candidate(diagnostic, source):
    message = diagnostic["message"]
    if "`nz` has no argument named `source`" in message:
        return "nz_named_source"
    if diagnostic["code"] == "E_CALL_ARG_TYPE" and "got const na" in message and any(
        f"`{fn}` argument `{arg}`" in message for fn in ("strategy.entry", "strategy.order")
        for arg in ("qty", "limit", "stop")
    ):
        return "entry_order_explicit_na"
    if "has no argument named `fill_orders_on_standard_ohlc`" in message:
        return "standard_ohlc_fill_contract"
    if "function_side_effect" in message:
        span = diagnostic.get("span", {})
        text = source[span.get("start", 0):span.get("end", 0)].decode("utf-8", errors="replace")
        if text == "max_bars_back":
            return "udf_max_bars_back"
        return "udf_side_effect:" + text[:80]
    if diagnostic["code"] == "E_UNKNOWN_SYMBOL" and any(
        f"`{name}`" in message for name in ("accdist", "obv", "nvi", "pvi", "pvt", "wad", "wvad", "iii")
    ):
        return "legacy_volume_builtin_alias"
    return None


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("result", type=Path)
    parser.add_argument("out", type=Path)
    args = parser.parse_args()
    data = json.loads(args.result.read_text(encoding="utf-8"))
    groups = defaultdict(list)
    for row in data["rows"]:
        if row.get("normalized_sha256"):
            groups[row["normalized_sha256"]].append(row)
    messages, first, codes = defaultdict(list), defaultdict(list), defaultdict(list)
    candidates = defaultdict(list)
    completed_diagnostics = defaultdict(list)
    runtime_groups = defaultdict(list)
    inventory = []
    for sha, aliases in groups.items():
        row = aliases[-1]
        ds = errors(row["analysis"])
        if ds:
            first[(ds[0]["code"], ds[0]["message"])].append(sha)
        for code in set(d["code"] for d in ds):
            codes[code].append(sha)
        for key in set((d["code"], d["message"]) for d in ds):
            messages[key].append(sha)
        source = Path(row["normalized_path"]).read_bytes() if ds else b""
        kinds = [candidate(d, source) for d in ds]
        for kind in set(k for k in kinds if k):
            candidates[kind].append({"sha256": sha, "only_this_diagnostic_family": all(k == kind for k in kinds),
                                     "error_count": len(ds)})
        status = category(row)
        if status == "runtime_passed":
            run = row.get("runtime", {})
            for code in sorted({d.get("code", "") for d in
                                run.get("diagnostics", []) + run.get("strategy_diagnostics", [])
                                if d.get("code", "").startswith("E_")}):
                completed_diagnostics[code].append(sha)
        if status != "runtime_passed" and row["analysis"]["executable"]:
            runtime_groups[(status, row.get("runtime", {}).get("stderr", ""))].append(sha)
        inventory.append({"sha256": sha, "status": status, "mode": row["analysis"].get("mode"),
                          "version": row["analysis"].get("version"), "path": row["normalized_path"],
                          "origins": [{k: a.get(k) for k in ("source_id", "path", "source_url", "candidate_names")}
                                      for a in aliases], "errors": ds})
    def pack(group, first_key="code"):
        return [{first_key: key[0], "message": key[1], "count": len(shas), "sources": shas}
                for key, shas in sorted(group.items(), key=lambda x: (-len(x[1]), x[0]))]
    out = {"input_sha256": digest(args.result), "metadata": data["metadata"],
           "status_counts": dict(Counter(r["status"] for r in inventory)),
           "mode_counts": dict(Counter(r["mode"] for r in inventory)),
           "first_error_groups": pack(first), "all_error_groups": pack(messages),
           "error_code_counts": {k: len(v) for k, v in sorted(codes.items(), key=lambda x: -len(x[1]))},
           "runtime_error_groups": pack(runtime_groups, "status"),
           "completed_with_e_diagnostics": {
               "source_count": len({sha for shas in completed_diagnostics.values() for sha in shas}),
               "code_counts": {code: len(shas) for code, shas in completed_diagnostics.items()},
               "sources_by_code": dict(completed_diagnostics),
               "note": "Supplementary view only. Frozen runtime_passed means exit 0 without an explicitly severity=error diagnostic; E_* broker diagnostics may omit severity. These sources are not all proven runtime defects."
           },
           "candidate_groups": dict(candidates), "sources": inventory,
           "counting_note": "first-error groups are disjoint; all-error and candidate groups overlap; none by itself proves invalid Pine or a runtime defect"}
    write_json(args.out, out)
    print(json.dumps({"status_counts": out["status_counts"],
                      "candidate_groups": {k: {"sources": len(v), "only_family": sum(x["only_this_diagnostic_family"] for x in v)}
                                           for k, v in candidates.items()}}, ensure_ascii=False))


if __name__ == "__main__":
    main()
