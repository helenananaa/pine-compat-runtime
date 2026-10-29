"""Rerun a saved source inventory without reusing execution receipts.

The inventory and source files remain local; no third-party Pine is copied into
the repository. This measures admission and a bounded default-input smoke run,
not Pine equivalence or profitability. Output directories must be new.
"""

from __future__ import annotations

import argparse
from collections import Counter
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import platform
import shutil
import subprocess
import time


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def write_json(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2), encoding="utf-8")


def errors(row):
    return [d for d in row.get("diagnostics", []) if d.get("severity", "").lower() == "error"]


def category(row):
    analysis = row["analysis"]
    if analysis.get("tool_status") != "completed":
        return "analysis_" + analysis.get("tool_status", "unknown")
    if not analysis.get("executable"):
        return "analysis_rejected"
    run = row.get("runtime", {})
    if run.get("status") == "passed":
        return "runtime_passed"
    stderr = run.get("stderr", "")
    for label, needle in [
        ("missing_request_input", "missing request data"),
        ("missing_currency_input", "missing currency rate data"),
        ("missing_execution_clock", "timenow requires"),
        ("currency_configuration_or_conversion", "account currency"),
    ]:
        if needle in stderr:
            return label
    if stderr.startswith("runtime failed:"):
        return "runtime_rejected_other"
    return run.get("status", "not_run")


def invoke(cli, args, timeout, raw_prefix):
    start = time.perf_counter()
    try:
        p = subprocess.run([str(cli), *args], capture_output=True, timeout=timeout)
    except subprocess.TimeoutExpired as exc:
        raw_prefix.with_suffix(".stdout").write_bytes(exc.stdout or b"")
        raw_prefix.with_suffix(".stderr").write_bytes(exc.stderr or b"")
        return {"tool_status": "timeout", "timeout_seconds": timeout,
                "wall_seconds": time.perf_counter() - start}, {}
    stderr = p.stderr.decode("utf-8", errors="replace")
    result = {"tool_status": "completed", "exit_code": p.returncode,
              "stderr": stderr, "wall_seconds": time.perf_counter() - start,
              "stdout_sha256": hashlib.sha256(p.stdout).hexdigest(),
              "stdout_bytes": len(p.stdout)}
    try:
        output = json.loads(p.stdout)
    except (ValueError, UnicodeDecodeError):
        result["tool_status"] = "invalid_output"
        output = {}
    if p.returncode or result["tool_status"] != "completed":
        raw_prefix.with_suffix(".stdout").write_bytes(p.stdout)
        raw_prefix.with_suffix(".stderr").write_bytes(p.stderr)
    return result, output


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--baseline", type=Path, required=True)
    parser.add_argument("--bars", type=Path, required=True)
    parser.add_argument("--cli", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--workers", type=int, default=4)
    parser.add_argument("--analysis-timeout", type=float, default=15)
    parser.add_argument("--runtime-timeout", type=float, default=8)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    args.out.mkdir(parents=True, exist_ok=False)
    raw = args.out / "failures"
    raw.mkdir()
    baseline = json.loads(args.baseline.read_text(encoding="utf-8"))
    rows = [{k: v for k, v in row.items() if k not in ("analysis", "runtime")}
            for row in baseline["rows"]]
    unique = {r["normalized_sha256"]: r for r in rows if r.get("normalized_sha256")}
    previous = {r["normalized_sha256"]: r for r in baseline["rows"] if r.get("normalized_sha256")}
    expected_count = baseline["metadata"]["unique_standalone_sources"]
    assert len(unique) == expected_count
    bad, source_hashes = [], {}
    originals = {r["local_path"]: r["raw_sha256"] for r in rows
                 if r.get("local_path") and r.get("raw_sha256")}
    for path, expected in originals.items():
        if not Path(path).is_file() or digest(path) != expected:
            bad.append(path)
    for sha, row in unique.items():
        path = Path(row["normalized_path"])
        actual = hashlib.sha256(path.read_text(encoding="utf-8").encode()).hexdigest()
        if actual != sha:
            bad.append(str(path))
        source_hashes[sha] = {"path": str(path), "file_sha256": digest(path)}
    integrity = {"original_paths_checked": len(originals), "normalized_sources_checked": len(unique),
                 "changed_or_missing": bad, "status": "passed" if not bad else "failed"}
    write_json(args.out / "source-integrity.json", integrity)
    write_json(args.out / "source-manifest.json", source_hashes)
    assert not bad, "Saved source hashes changed; see source-integrity.json"
    shutil.copyfile(args.bars, args.out / "bars.csv")
    cli_hash = digest(args.cli)
    shutil.copyfile(args.cli, args.out / args.cli.name)
    cli = args.out / args.cli.name
    assert digest(cli) == cli_hash
    metadata = {"started_at": datetime.now(timezone.utc).isoformat(),
                "commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip(),
                "baseline": str(args.baseline.resolve()), "baseline_sha256": digest(args.baseline),
                "baseline_metadata": baseline["metadata"], "cli_sha256": cli_hash,
                "cli_version": subprocess.check_output([str(cli), "--version"], text=True).strip(),
                "bars_sha256": digest(args.out / "bars.csv"), "chart_bars": 512,
                "chart_symbol": "V-BTC-SPOT", "chart_timeframe": "1",
                "workers": args.workers, "analysis_timeout_seconds": args.analysis_timeout,
                "runtime_timeout_seconds": args.runtime_timeout, "python": platform.python_version(),
                "platform": platform.platform(), "runner_sha256": digest(__file__),
                "cargo_lock_sha256": digest(root / "Cargo.lock"),
                "rustc": subprocess.check_output(["rustc", "--version"], text=True).strip(),
                "input_rows": len(rows), "unique_standalone_sources": len(unique),
                "cached_receipts_reused": False,
                "scope": "all source analysis and default-input 512 synthetic bar smoke; no native equivalence claim"}
    assert metadata["bars_sha256"] == baseline["metadata"]["bars_sha256"]
    assert len((args.out / "bars.csv").read_text().splitlines()) - 1 == 512
    write_json(args.out / "metadata.json", metadata)
    print(json.dumps({"stage": "integrity", **integrity, "commit": metadata["commit"]}), flush=True)

    def analyze(item):
        sha, row = item
        result, output = invoke(cli, ["analyze", row["normalized_path"], "--format", "json"],
                                args.analysis_timeout, raw / (sha + ".analysis.raw"))
        result.update(executable=output.get("executable", False), mode=output.get("scriptMode"),
                      version=output.get("languageVersion"), diagnostics=output.get("diagnostics", []),
                      unsupported=output.get("compatibility", {}).get("unsupported", []))
        return sha, result

    def execute(item):
        sha, row = item
        result, output = invoke(cli, ["run", row["normalized_path"], "--bars", str(args.out / "bars.csv"),
                                     "--chart-symbol", "V-BTC-SPOT", "--chart-timeframe", "1"],
                                args.runtime_timeout, raw / (sha + ".runtime.raw"))
        strategy = output.get("strategy") or {}
        result.update(diagnostics=output.get("diagnostics", []),
                      strategy_diagnostics=strategy.get("diagnostics", []),
                      orders=len(strategy.get("orders", [])), trades=len(strategy.get("trades", [])),
                      plots=len(output.get("plots", [])), alerts=len(output.get("alerts", [])))
        runtime_errors = errors({"diagnostics": result["diagnostics"] + result["strategy_diagnostics"]})
        result["status"] = ("passed" if result["tool_status"] == "completed" and result["exit_code"] == 0
                            and not runtime_errors else "failed" if result["tool_status"] == "completed"
                            else result["tool_status"])
        return sha, result

    def stage(name, function, items):
        results = {}
        with ThreadPoolExecutor(max_workers=args.workers) as pool, (args.out / (name + "-receipts.jsonl")).open("x", encoding="utf-8") as log:
            for count, (sha, result) in enumerate(pool.map(function, items), 1):
                results[sha] = result
                log.write(json.dumps({"sha256": sha, **result}, ensure_ascii=False) + "\n")
                log.flush()
                if count % 100 == 0:
                    print(json.dumps({"stage": name, "completed": count, "total": len(items)}), flush=True)
        return results

    analysis = stage("analysis", analyze, list(unique.items()))
    for row in rows:
        if row.get("normalized_sha256"):
            row["analysis"] = analysis[row["normalized_sha256"]]
    write_json(args.out / "analysis.json", {"metadata": metadata, "rows": rows})
    print(json.dumps({"stage": "analysis_complete", "accepted": sum(r["executable"] for r in analysis.values())}), flush=True)
    runtime = stage("runtime", execute, [(sha, r) for sha, r in unique.items()
                                         if analysis[sha]["executable"] and analysis[sha]["mode"] != "library"])
    for row in rows:
        if row.get("normalized_sha256") in runtime:
            row["runtime"] = runtime[row["normalized_sha256"]]
    metadata["completed_at"] = datetime.now(timezone.utc).isoformat()
    write_json(args.out / "results.json", {"metadata": metadata, "rows": rows})
    changes = [{"sha256": sha, "previous": category(previous[sha]), "current": category(row)}
               for sha, row in unique.items() if category(previous[sha]) != category(row)]
    summary = {"metadata": metadata, "source_integrity": integrity,
               "previous_status": dict(Counter(map(category, previous.values()))),
               "current_status": dict(Counter(map(category, unique.values()))),
               "compile_accepted": sum(r["executable"] for r in analysis.values()),
               "status_changes": changes,
               "newly_accepted": [sha for sha, row in unique.items() if row["analysis"]["executable"] and not previous[sha]["analysis"]["executable"]],
               "newly_rejected": [sha for sha, row in unique.items() if not row["analysis"]["executable"] and previous[sha]["analysis"]["executable"]],
               "runtime_pass_lost": [c for c in changes if c["previous"] == "runtime_passed"],
               "runtime_pass_gained": [c for c in changes if c["current"] == "runtime_passed"]}
    assert digest(cli) == cli_hash
    assert all(digest(info["path"]) == info["file_sha256"] for info in source_hashes.values())
    summary["normalized_sources_unchanged_after_run"] = True
    write_json(args.out / "summary.json", summary)
    print(json.dumps({"stage": "complete", "current_status": summary["current_status"],
                      "status_changes": len(changes)}), flush=True)


if __name__ == "__main__":
    main()
