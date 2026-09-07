from __future__ import annotations

import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))

import analyze_modern_strategy_corpus as corpus  # noqa: E402


def write_manifest(root: Path, rows: list[dict]) -> Path:
    path = root / "manifest.jsonl"
    path.write_text(
        "\n".join(json.dumps(row, sort_keys=True) for row in rows) + "\n",
        encoding="utf-8",
    )
    return path


def sample(
    sample_id: str,
    source_path: str,
    *,
    category: str = "original_regression",
    version: int = 6,
    bars: str = "bars.csv",
    source_sha: str = "",
    bars_sha: str = "",
    reference_status: str = "none",
    reference_path: str = "",
    reference_fields: list[str] | None = None,
    reference_reason: str = "",
    timeout: float = 30,
    scenario_id: str = "default",
    script_id: str | None = None,
    magnifier_path: str = "",
) -> dict:
    source = Path(source_path)
    payload = {
        "schemaVersion": 1,
        "sampleId": sample_id,
        "scriptId": script_id or sample_id.rsplit(".", 1)[0],
        "scenarioId": scenario_id,
        "sampleCategory": category,
        "pineVersion": version,
        "source": {
            "kind": "test",
            "identifier": source_path,
            "revision": "test",
            "licenseClass": "original",
            "licenseId": "test",
            "retainAllowed": True,
            "publishAllowed": True,
            "sourceSha256": source_sha,
            "sourceModified": False,
        },
        "script": {"sourcePath": source_path, "imports": [], "sourceModified": False},
        "settings": {"inputOverrides": []},
        "data": {
            "ohlcvPath": bars,
            "ohlcvSha256": bars_sha,
            "coverage": {"status": "synthetic_smoke", "reason": "test"},
        },
        "hostInputs": {
            "requestData": {"status": "not_supplied", "reason": "test"},
            "magnifierBars": {
                "status": "present" if magnifier_path else "not_supplied",
                "reason": "test",
                "path": magnifier_path,
            },
            "sessionWindows": {"status": "not_supplied", "reason": "test"},
            "executionTimes": {"status": "not_supplied", "reason": "test"},
        },
        "reference": {
            "status": reference_status,
            "path": reference_path,
            "fieldCoverage": reference_fields or [],
            "incomparableReason": reference_reason,
            "reason": reference_reason,
        },
        "execution": {
            "mode": "historical",
            "timeoutSeconds": timeout,
            "expectedSupportBoundary": "test",
        },
    }
    return payload


class FakeRunner:
    def __init__(self, handler=None) -> None:
        self.commands: list[list[str]] = []
        self.handler = handler

    def __call__(
        self, command: list[str] | tuple[str, ...], root: Path, timeout_seconds: float
    ) -> corpus.CommandResult:
        del timeout_seconds
        argv = [str(part) for part in command]
        self.commands.append(argv)
        if self.handler is not None:
            return self.handler(argv, root)
        return self._default(argv, root)

    def _default(self, argv: list[str], root: Path) -> corpus.CommandResult:
        verb = argv[1]
        source = Path(argv[2]).read_text(encoding="utf-8")
        if verb == "fmt-ast":
            if "PARSE_FAIL" in source:
                return corpus.CommandResult(
                    argv,
                    0,
                    "Program { statements: [] }\nE_PARSE_EXPECTED:Error:1:1: expected expression\n",
                    "",
                    False,
                    1,
                )
            return corpus.CommandResult(argv, 0, "Program { statements: [] }\n", "", False, 1)
        if verb == "analyze":
            if "SEMA_FAIL" in source:
                return corpus.CommandResult(
                    argv,
                    1,
                    "diagnostics: 1\nE_UNKNOWN_FUNCTION:Error:2:1: unknown function `mystery`\n",
                    "analysis failed\n",
                    False,
                    1,
                )
            return corpus.CommandResult(
                argv, 0, "diagnostics: 0\nsupported: 1, unsupported: 0\n", "", False, 1
            )
        if "RUN_FAIL" in source:
            return corpus.CommandResult(
                argv, 1, "", "runtime failed: broker exploded\n", False, 1
            )
        if "NO_TRADES" in source:
            return corpus.CommandResult(
                argv,
                0,
                json.dumps({"schemaVersion": 8, "strategy": {"trades": []}}) + "\n",
                "",
                False,
                1,
            )
        return corpus.CommandResult(
            argv,
            0,
            json.dumps(
                {
                    "schemaVersion": 8,
                    "strategy": {
                        "trades": [
                            {
                                "id": "L",
                                "exitId": "X",
                                "qty": 1,
                                "entryPrice": 2,
                                "exitPrice": 3,
                                "entryTime": 1,
                                "exitTime": 2,
                                "entryBarIndex": 0,
                                "exitBarIndex": 1,
                                "profit": 1,
                            }
                        ]
                    },
                }
            )
            + "\n",
            "",
            False,
            1,
        )


def write_source(root: Path, name: str, body: str) -> str:
    path = root / name
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(body, encoding="utf-8")
    return name


def write_bars(root: Path) -> str:
    (root / "bars.csv").write_text(
        "time,open,high,low,close,volume\n1,1,1,1,1,1\n",
        encoding="utf-8",
    )
    return "bars.csv"


def hashed_sample(root: Path, name: str, body: str, **kwargs) -> dict:
    rel = write_source(root, name, body)
    bars = kwargs.pop("bars", write_bars(root))
    source_sha = corpus.sha256_file(root / rel)
    bars_sha = corpus.sha256_file(root / bars) if (root / bars).is_file() else ""
    sample_id = kwargs.pop("sample_id", f"original.test.{Path(name).stem}.default")
    return sample(
        sample_id,
        rel,
        source_sha=source_sha,
        bars=bars,
        bars_sha=bars_sha,
        **kwargs,
    )


def report_for(root: Path, rows: list[dict], runner=None, compare_fn=None) -> dict:
    rows = sorted(rows, key=lambda row: row["sampleId"])
    manifest = write_manifest(root, rows)
    return corpus.build_report(
        corpus.parse_manifest(manifest),
        root=root,
        manifest_path=manifest,
        pine_compat=root / "pine-compat",
        build_revision="test-revision",
        command_runner=runner or FakeRunner(),
        compare_fn=compare_fn or corpus.missing_comparator_result,
    )


class AnalyzeModernStrategyCorpusTests(unittest.TestCase):
    def test_fmt_ast_exit_zero_with_parse_diagnostics_is_parse_failure(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            row = hashed_sample(
                root, "bad.pine", "//@version=6\nstrategy(\"x\")\nPARSE_FAIL\n"
            )
            result = report_for(root, [row])
            stages = result["items"][0]["stages"]
            self.assertEqual(stages["parse"]["status"], "failed")
            self.assertFalse(stages["parse"]["usedExitCode"])
            self.assertEqual(stages["parse"]["returnCode"], 0)
            self.assertEqual(result["metrics"]["parse"]["failed"], 1)
            self.assertEqual(stages["run"]["status"], "not_run")

    def test_sema_failure_does_not_run_and_is_not_parse_failure(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            row = hashed_sample(
                root, "sema.pine", "//@version=6\nstrategy(\"x\")\nSEMA_FAIL\n"
            )
            result = report_for(root, [row])
            stages = result["items"][0]["stages"]
            self.assertEqual(stages["parse"]["status"], "passed")
            self.assertEqual(stages["sema"]["status"], "failed")
            self.assertEqual(stages["run"]["status"], "not_run")
            self.assertEqual(
                result["items"][0]["firstBlocking"]["rootCauseClass"], "builtin"
            )

    def test_run_failure_is_independent_of_sema_pass(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            row = hashed_sample(
                root, "run.pine", "//@version=6\nstrategy(\"x\")\nRUN_FAIL\n"
            )
            result = report_for(root, [row])
            stages = result["items"][0]["stages"]
            self.assertEqual(stages["parse"]["status"], "passed")
            self.assertEqual(stages["sema"]["status"], "passed")
            self.assertEqual(stages["run"]["status"], "failed")
            self.assertEqual(stages["resultConsistency"]["status"], "not_run")

    def test_missing_bars_are_missing_input_not_run_failure(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            row = hashed_sample(
                root,
                "ok.pine",
                '//@version=6\nstrategy("ok")\n',
                bars="missing-bars.csv",
            )
            result = report_for(root, [row])
            stages = result["items"][0]["stages"]
            self.assertEqual(stages["sema"]["status"], "passed")
            self.assertEqual(stages["run"]["status"], "missing_input")
            self.assertEqual(
                result["items"][0]["inputAvailability"]["chartBars"], "missing_input"
            )

    def test_timeout_marks_failed_timeout_and_does_not_stop_the_batch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            first = hashed_sample(
                root, "slow.pine", '//@version=6\nstrategy("slow")\nTIMEOUT\n'
            )
            second = hashed_sample(
                root, "ok.pine", '//@version=6\nstrategy("ok")\n'
            )

            def handler(argv: list[str], _root: Path) -> corpus.CommandResult:
                source = Path(argv[2]).read_text(encoding="utf-8")
                if argv[1] == "fmt-ast" and "TIMEOUT" in source:
                    return corpus.CommandResult(argv, -1, "", "", True, 5)
                runner = FakeRunner()
                return runner._default(argv, _root)

            result = report_for(
                root,
                [second, first] if second["sampleId"] < first["sampleId"] else [first, second],
                runner=FakeRunner(handler),
            )
            by_id = {item["sampleId"]: item for item in result["items"]}
            self.assertEqual(by_id[first["sampleId"]]["stages"]["parse"]["status"], "failed")
            self.assertEqual(
                by_id[first["sampleId"]]["stages"]["parse"]["errorKind"], "timeout"
            )
            self.assertEqual(by_id[second["sampleId"]]["stages"]["parse"]["status"], "passed")
            self.assertEqual(len(result["items"]), 2)

    def test_duplicate_sample_id_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            row = hashed_sample(root, "ok.pine", '//@version=6\nstrategy("ok")\n')
            manifest = write_manifest(root, [row, dict(row)])
            with self.assertRaises(corpus.CorpusError) as caught:
                corpus.parse_manifest(manifest)
            self.assertIn("duplicate sampleId", str(caught.exception))

    def test_no_reference_is_not_consistency_pass(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            row = hashed_sample(root, "ok.pine", '//@version=6\nstrategy("ok")\n')
            result = report_for(root, [row])
            stages = result["items"][0]["stages"]
            self.assertEqual(stages["run"]["status"], "passed")
            self.assertEqual(stages["outputComparability"]["status"], "not_run")
            self.assertEqual(stages["outputComparability"]["reason"], "no_reference")
            self.assertEqual(stages["resultConsistency"]["status"], "not_run")
            self.assertEqual(result["metrics"]["resultConsistency"]["rate"], "N/A")
            self.assertEqual(result["metrics"]["resultConsistency"]["percent"], "N/A")

    def test_partial_reference_is_incomparable_not_consistent(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            (root / "ref.json").write_text("{}", encoding="utf-8")
            row = hashed_sample(
                root,
                "ok.pine",
                '//@version=6\nstrategy("ok")\n',
                reference_status="partial",
                reference_path="ref.json",
                reference_fields=["id"],
                reference_reason="partial_reference",
            )
            result = report_for(root, [row])
            stages = result["items"][0]["stages"]
            self.assertEqual(stages["outputComparability"]["status"], "failed")
            self.assertEqual(stages["outputComparability"]["reason"], "partial_reference")
            self.assertEqual(stages["resultConsistency"]["status"], "not_run")

    def test_mismatched_reference_fails_consistency(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            (root / "ref.json").write_text(
                json.dumps({"trades": [{"id": "OTHER"}]}), encoding="utf-8"
            )
            row = hashed_sample(
                root,
                "ok.pine",
                '//@version=6\nstrategy("ok")\n',
                reference_status="present",
                reference_path="ref.json",
                reference_fields=["id"],
            )

            def compare_fn(actual, reference, fields):
                del actual, reference, fields
                return {
                    "status": "failed",
                    "reason": "reference_mismatch",
                    "comparable": True,
                    "coverage": ["id"],
                    "mismatches": 1,
                    "firstDifference": "id",
                }

            result = report_for(root, [row], compare_fn=compare_fn)
            stages = result["items"][0]["stages"]
            self.assertEqual(stages["outputComparability"]["status"], "passed")
            self.assertEqual(stages["resultConsistency"]["status"], "failed")
            self.assertEqual(stages["resultConsistency"]["firstDifference"], "id")

    def test_missing_comparator_is_tool_unavailable_not_consistent(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            (root / "ref.json").write_text("{}", encoding="utf-8")
            row = hashed_sample(
                root,
                "ok.pine",
                '//@version=6\nstrategy("ok")\n',
                reference_status="present",
                reference_path="ref.json",
                reference_fields=["id"],
            )
            result = report_for(
                root, [row], compare_fn=corpus.missing_comparator_result
            )
            stages = result["items"][0]["stages"]
            self.assertEqual(stages["outputComparability"]["status"], "not_run")
            self.assertEqual(stages["outputComparability"]["reason"], "tool_unavailable")
            self.assertEqual(stages["resultConsistency"]["status"], "not_run")
            self.assertNotEqual(stages["resultConsistency"]["status"], "passed")

    def test_path_with_spaces_is_passed_as_argv_element(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            row = hashed_sample(
                root,
                "my script.pine",
                '//@version=6\nstrategy("ok")\n',
            )
            runner = FakeRunner()
            report_for(root, [row], runner=runner)
            fmt = runner.commands[0]
            self.assertEqual(fmt[1], "fmt-ast")
            self.assertTrue(fmt[2].endswith("my script.pine"))
            self.assertEqual(len(fmt), 3)

    def test_hash_invalidation_after_input_change(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            row = hashed_sample(root, "ok.pine", '//@version=6\nstrategy("ok")\n')
            (root / "ok.pine").write_text(
                '//@version=6\nstrategy("changed")\n', encoding="utf-8"
            )
            result = report_for(root, [row])
            stages = result["items"][0]["stages"]
            self.assertEqual(stages["parse"]["status"], "failed")
            self.assertEqual(stages["parse"]["errorKind"], "hash_mismatch")
            self.assertEqual(stages["resultConsistency"]["status"], "not_run")
            self.assertTrue(result["items"][0]["hashErrors"])

    def test_zero_denominator_prints_na_not_100_percent(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            row = hashed_sample(
                root,
                "neg.pine",
                "//@version=6\nstrategy(\"n\")\nSEMA_FAIL\n",
                category="negative_control",
            )
            result = report_for(root, [row])
            self.assertEqual(result["denominators"]["scriptsN"], 0)
            self.assertEqual(result["denominators"]["scenariosM"], 0)
            self.assertEqual(result["metrics"]["parse"]["rate"], "N/A")
            self.assertEqual(result["metrics"]["parse"]["percent"], "N/A")
            self.assertEqual(result["metrics"]["resultConsistency"]["rate"], "N/A")
            self.assertEqual(result["negativeControls"]["expectedRejectPassed"], 1)

    def test_root_cause_ranking_counts_scripts_and_scenarios(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            a = hashed_sample(
                root, "a.pine", "//@version=6\nstrategy(\"a\")\nSEMA_FAIL\n"
            )
            b = hashed_sample(
                root, "b.pine", "//@version=6\nstrategy(\"b\")\nSEMA_FAIL\n"
            )
            result = report_for(root, [a, b] if a["sampleId"] < b["sampleId"] else [b, a])
            ranking = result["rootCauseRanking"]
            self.assertGreaterEqual(len(ranking), 1)
            top = ranking[0]
            self.assertEqual(top["affectedScripts"], 2)
            self.assertEqual(top["affectedScenarios"], 2)
            self.assertEqual(top["rootCauseClass"], "builtin")
            self.assertEqual(top["firstBlockingStage"], "sema")

    def test_subprocess_timeout_uses_argv_and_records_timeout(self) -> None:
        result = corpus.default_command_runner(
            [sys.executable, "-c", "import time; time.sleep(2)"],
            Path("."),
            0.05,
        )
        self.assertTrue(result.timed_out)
        self.assertEqual(result.argv[0], sys.executable)

    def test_rust_bars_map_parser_keeps_or_groups(self) -> None:
        text = '''
        "tests/fixtures/runtime/strategy_a.pine"
        | "tests/fixtures/runtime/strategy_b.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/shared_bars.csv"
        )),
        '''
        mapping = corpus.parse_rust_bars_map(text)
        self.assertEqual(
            mapping["tests/fixtures/runtime/strategy_a.pine"],
            "tests/fixtures/runtime/shared_bars.csv",
        )
        self.assertEqual(
            mapping["tests/fixtures/runtime/strategy_b.pine"],
            "tests/fixtures/runtime/shared_bars.csv",
        )


if __name__ == "__main__":
    unittest.main()
