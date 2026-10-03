"""Run a frozen, local native-reference matrix on one committed runtime.

Reference capture is deliberately separate. Older receipts are never overwritten.
Artifacts must already be freshly built and installed in ARTIFACTS/venv.
"""
from __future__ import annotations

import argparse
import csv
import hashlib
import importlib.util
import json
import math
import subprocess
import sys
import time
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
MODES = ("run", "run-incremental", "run-realtime-history")


def read(p):
    return json.loads(Path(p).read_text(encoding="utf-8-sig"))


def sha(p):
    digest = hashlib.sha256()
    with Path(p).open("rb") as stream:
        while chunk := stream.read(1024 * 1024):
            digest.update(chunk)
    return digest.hexdigest()


def write(p, obj):
    Path(p).write_text(json.dumps(obj, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def load_bars(p):
    with Path(p).open(encoding="utf-8-sig", newline="") as f:
        return [{k: int(v) if k == "time" else float(v) for k, v in r.items()}
                for r in csv.DictReader(f)]


def differences(a, b, path=""):
    """Compare the whole public output, retaining types, keys, nulls and lengths."""
    if isinstance(a, dict) and isinstance(b, dict):
        if a.keys() != b.keys():
            return [path + ":keys"]
        return [x for k in a for x in differences(a[k], b[k], path + "/" + k)]
    if isinstance(a, list) and isinstance(b, list):
        if len(a) != len(b):
            return [path + ":length"]
        return [x for i, (u, v) in enumerate(zip(a, b)) for x in differences(u, v, path + f"/{i}")]
    if isinstance(a, (int, float)) and not isinstance(a, bool) and isinstance(b, (int, float)) and not isinstance(b, bool):
        return [] if math.isfinite(a) and math.isfinite(b) and math.isclose(a, b, rel_tol=1e-12, abs_tol=1e-9) else [path]
    return [] if type(a) is type(b) and a == b else [path]


def run(command, out, stdin=None):
    start = time.monotonic()
    with Path(out).open("wb") as f:
        p = subprocess.run(command, cwd=REPO, input=stdin, stdout=f, stderr=subprocess.PIPE, timeout=600)
    receipt = {"command": list(map(str, command)), "exitCode": p.returncode,
               "stderr": p.stderr.decode(errors="replace"), "seconds": time.monotonic() - start,
               "outputSha256": sha(out)}
    write(str(out) + ".receipt.json", receipt)
    if p.returncode:
        raise RuntimeError(str(receipt))
    return receipt


def python_case(payload, output):
    import pine_compat
    p = read(payload)
    chart = p["request"]["$chart"]
    request = {k: v for k, v in p["request"].items() if k != "$chart"}
    request["$chart"] = {k: v for k, v in chart.items() if k not in ("symbol", "timeframe")}
    result = pine_compat.run_script(p["source"], p["bars"], request_bars=request,
                                   input_overrides={int(k): v for k, v in p["overrides"].items()},
                                   chart_symbol=chart["symbol"], chart_timeframe=chart["timeframe"])
    write(output, result)
    print(json.dumps({"module": pine_compat.__file__, "resultSha256": sha(output)}))


def payload_from_command(cmd, cli):
    def option(flag, default=None):
        return cmd[cmd.index(flag) + 1] if flag in cmd else default
    source = REPO / cmd[2]
    bars_path = REPO / option("--bars")
    chart = {"symbol": option("--chart-symbol", "SYNTHETIC"), "timeframe": option("--chart-timeframe", "1"),
             "minMove": 1, "priceScale": 100}
    if "--chart-price-grid" in cmd:
        chart["minMove"], chart["priceScale"] = map(int, option("--chart-price-grid").split("/"))
    if "--chart-quantity-precision" in cmd:
        chart["quantityPrecision"] = int(option("--chart-quantity-precision"))
    if "--chart-point-value" in cmd:
        chart["pointValue"] = float(option("--chart-point-value"))
    if option("--chart-currency", "USD") != "USD":
        raise ValueError("frozen matrix requires the common USD binding profile")
    request = {"$chart": chart}
    for i, flag in enumerate(cmd):
        if flag == "--request-bars":
            key, filename = cmd[i + 1].rsplit("=", 1)
            request[key] = load_bars(REPO / filename)
    analysis = json.loads(subprocess.check_output([str(cli), "analyze", str(source), "--format", "json"]))
    inputs = {str(i["callSiteId"]): i for i in analysis["inputs"]}
    overrides = {}
    for i, flag in enumerate(cmd):
        if flag != "--input-override":
            continue
        key, value = cmd[i + 1].split("=", 1)
        item = inputs[key]
        default = item["default"]
        if item["isSource"] or isinstance(default, str):
            overrides[key] = value
        elif isinstance(default, bool):
            if value.lower() not in ("true", "false"):
                raise ValueError(value)
            overrides[key] = value.lower() == "true"
        elif isinstance(default, int):
            overrides[key] = int(value)
        elif isinstance(default, float):
            overrides[key] = float(value)
        else:
            raise ValueError(f"unsupported input {item}")
    return {"source": source.read_text(encoding="utf-8-sig"), "bars": load_bars(bars_path),
            "barsCsv": bars_path.read_text(encoding="utf-8-sig"), "request": request, "overrides": overrides}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--plan", type=Path, default=REPO / "docs/CORE_SCRIPT_ACCEPTANCE_PLAN.json")
    parser.add_argument("--artifacts", type=Path)
    parser.add_argument("--python-case", type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--resume", action="store_true", help="Audit and reuse completed cross-surface rows; rerun native comparators")
    parser.add_argument("--profile", choices=("debug", "release"), default="debug")
    args = parser.parse_args()
    if args.python_case:
        python_case(args.python_case, args.output)
        return
    root = args.artifacts.resolve()
    plan = read(args.plan)
    head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=REPO, text=True).strip()
    if head != plan["coreCommit"]:
        raise RuntimeError("matrix is pinned to a different core commit")
    core_names = subprocess.check_output(["git", "ls-files", "crates", "Cargo.toml", "Cargo.lock"], cwd=REPO, text=True).splitlines()
    core_hashes = {p: sha(REPO / p) for p in core_names}
    if subprocess.check_output(["git", "diff", "HEAD", "--", "crates", "Cargo.toml", "Cargo.lock"], cwd=REPO):
        raise RuntimeError("uncommitted core changes cannot be qualified as HEAD")
    for path, expected in plan["frozenFiles"].items():
        if sha(REPO / path) != expected:
            raise RuntimeError("frozen reference changed: " + path)
    files = [root / "pine-compat.exe", root / "core-script-probe.exe", *sorted((root / "wheels").glob("*.whl")),
             *sorted((root / "wasm").glob("*"))]
    import platform
    provenance = {"sourceCommit": head, "profile": args.profile, "platform": platform.system() + " " + platform.machine(),
                  "coreFiles": core_hashes, "artifacts": [{"path": str(p), "sha256": sha(p)} for p in files if p.is_file()]}
    previous = None
    if args.resume:
        previous = read(root / "matrix-results.json")
        old_provenance = read(root / "build-provenance.json")
        if old_provenance != provenance or previous["coreCommit"] != head or previous["planSha256"] != sha(args.plan):
            raise RuntimeError("cannot resume changed artifacts, source or plan")
        archive = root / f"matrix-results-before-resume-{time.time_ns()}.json"
        write(archive, previous)
    write(root / "build-provenance.json", provenance)
    report = {"coreCommit": head, "planSha256": sha(args.plan), "nativeRecaptured": False,
              "wholeOutputTolerance": {"absolute": 1e-9, "relative": 1e-12}, "cases": [], "nativeGroups": [], "failures": []}
    if previous:
        report["resumedFromReport"] = {"path": archive.name, "sha256": sha(archive)}
    cli = root / "pine-compat.exe"
    py = root / "venv/Scripts/python.exe"
    for case in plan["cases"]:
        dest = root / case["group"]
        dest.mkdir(exist_ok=True)
        for p in case["files"]:
            original = REPO / p
            (dest / original.name).write_bytes(original.read_bytes())
        saved = next((r for r in previous["cases"] if r["id"] == case["id"]), None) if previous else None
        if saved and saved["status"] == "cross-surface-passed":
            pp = dest / (case["id"] + "-payload.json")
            assert sha(pp) == saved["payloadSha256"]
            expected = read(dest / case["outputs"]["run"])
            for surface, filename in [(m, case["outputs"][m]) for m in MODES] + [("installedPython", case["id"] + "-python.json"), ("actualNodeWasm", case["id"] + "-wasm.json"), ("directRust", case["id"] + "-rust.json")]:
                record = saved["surfaces"][surface]
                assert sha(dest / filename) == record.get("outputSha256", record.get("receipt", {}).get("outputSha256"))
                assert not differences(expected, read(dest / filename))
            report["cases"].append(saved)
            print(case["id"], "audited-reused", flush=True)
            continue
        row = {"id": case["id"], "status": "failed", "surfaces": {}}
        report["cases"].append(row)
        try:
            cmd = case["command"]
            payload = payload_from_command(cmd, cli)
            pp = dest / (case["id"] + "-payload.json")
            write(pp, payload)
            bars = payload["bars"]
            row.update({"bars": len(bars), "firstTime": bars[0]["time"], "lastTime": bars[-1]["time"],
                        "chart": payload["request"]["$chart"], "overrides": payload["overrides"],
                        "sourceSha256": hashlib.sha256(payload["source"].encode()).hexdigest(), "payloadSha256": sha(pp)})
            base = None
            for mode in MODES:
                out = dest / case["outputs"][mode]
                command = [str(cli), mode, str(REPO / cmd[2]), *cmd[3:]]
                for i, flag in enumerate(command):
                    if flag == "--bars":
                        command[i + 1] = str(REPO / command[i + 1])
                    elif flag == "--request-bars":
                        key, filename = command[i + 1].rsplit("=", 1)
                        command[i + 1] = key + "=" + str(REPO / filename)
                receipt = run(command, out)
                value = read(out)
                if value["diagnostics"] or (value.get("strategy") or {}).get("diagnostics"):
                    raise RuntimeError("runtime diagnostics")
                if base is None:
                    base = value
                issues = differences(base, value)
                row["surfaces"][mode] = {"receipt": receipt, "mismatches": len(issues), "examples": issues[:10]}
                if issues:
                    raise RuntimeError(mode + " full output differs")
            pyout = dest / (case["id"] + "-python.json")
            run([str(py), str(Path(__file__).resolve()), "--python-case", str(pp), "--output", str(pyout)], dest / (case["id"] + "-python.log"))
            wasmout = dest / (case["id"] + "-wasm.json")
            run(["node", str(REPO / "scripts/core_script_wasm.cjs"), str(root / "wasm/pine_wasm.js"), str(pp)], wasmout)
            rustout = dest / (case["id"] + "-rust.json")
            run([str(root / "core-script-probe.exe")], rustout, pp.read_bytes())
            for surface, out in [("installedPython", pyout), ("actualNodeWasm", wasmout), ("directRust", rustout)]:
                issues = differences(base, read(out))
                row["surfaces"][surface] = {"outputSha256": sha(out), "mismatches": len(issues), "examples": issues[:10]}
                if issues:
                    raise RuntimeError(surface + " full output differs: " + str(issues[:3]))
            row["status"] = "cross-surface-passed"
        except Exception as exc:
            report["failures"].append({"case": case["id"], "error": str(exc)})
        write(root / "matrix-results.json", report)
        print(case["id"], row["status"], flush=True)
    for group in plan["nativeGroups"]:
        dest = root / group["id"]
        try:
            if group.get("kind") == "fees":
                spec = importlib.util.spec_from_file_location("fee_compare", dest / "compare.py")
                module = importlib.util.module_from_spec(spec)
                spec.loader.exec_module(module)
                module.r = dest
                results = [module.compare(c, m) for c in ("percent", "contract") for m in MODES]
                assert all(r["full_output_qualified"] and not any(r["missing_mismatches"].values()) and max(r["max_errors"].values()) < 1e-7 for r in results)
                write(dest / "comparison.json", results)
            elif group["id"].startswith("ssl-"):
                from compare_core_ssl_open_trades import compare
                results = compare(dest, cli)
            else:
                run([str(py), str(dest / "compare.py")], dest / "comparison-stdout.json")
                results = read(dest / ("comparison.json" if (dest / "comparison.json").exists() else "comparison-stdout.json"))
            report["nativeGroups"].append({"id": group["id"], "status": "compared", "results": results})
        except Exception as exc:
            report["failures"].append({"nativeGroup": group["id"], "error": str(exc)})
    base = REPO / ".local/product-completion-20260912"
    for helper, receipt in [("qualify-rsi.py", "qualification.json"), ("compare-rsi-alternate.py", "rsi-bb-divergence-comparison.json"), ("qualify-pivot.py", "pivot-qualification.json")]:
        try:
            run([str(py), str(base / helper), str(root)], root / (helper + ".log"))
            report["nativeGroups"].append({"id": helper, "status": "passed", "results": read(root / receipt)})
        except Exception as exc:
            report["failures"].append({"helper": helper, "error": str(exc)})
        write(root / "matrix-results.json", report)
        print(helper, "completed", flush=True)
    if {p: sha(REPO / p) for p in core_names} != core_hashes:
        raise RuntimeError("core source changed during qualification")
    for p, expected in plan["frozenFiles"].items():
        if sha(REPO / p) != expected:
            raise RuntimeError("reference changed during qualification: " + p)
    report["completed"] = True
    report["runnerSha256"] = sha(Path(__file__))
    report["wasmRunnerSha256"] = sha(REPO / "scripts/core_script_wasm.cjs")
    report["rustProbeSourceSha256"] = sha(REPO / "scripts/core_script_probe.rs")
    report["sslObserverComparatorSha256"] = sha(REPO / "scripts/compare_core_ssl_open_trades.py")
    report["crossSurfaceCasesPassed"] = sum(r["status"] == "cross-surface-passed" for r in report["cases"])
    write(root / "matrix-results.json", report)
    print(json.dumps({"cases": len(report["cases"]), "failures": len(report["failures"])}))
    if report["failures"]:
        sys.exit(1)


if __name__ == "__main__":
    main()
