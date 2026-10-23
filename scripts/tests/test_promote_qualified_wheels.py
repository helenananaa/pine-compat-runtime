from __future__ import annotations

from copy import deepcopy
from contextlib import redirect_stdout
import hashlib
import io
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch
import zipfile

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import promote_qualified_wheels as promotion


class QualifiedPromotionTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.repo = Path(self.temporary.name)
        self.git("init", "-q")
        self.git("config", "user.name", "Release test")
        self.git("config", "user.email", "release-test@example.invalid")
        self.git("config", "core.autocrlf", "false")
        (self.repo / ".git/info/exclude").write_text("dist/\n", encoding="utf-8")
        self.write("runtime.rs", "// qualified source\n")
        self.write(".gitattributes", "*.json text eol=lf\n")
        self.git("add", ".")
        self.git("commit", "-qm", "qualified runtime")
        self.source = self.git("rev-parse", "HEAD").strip()
        self.dist = self.repo / "dist"
        self.dist.mkdir()
        wheels = {}
        for index, label in enumerate(sorted(promotion.PLATFORMS)):
            native_name = "pine_compat/core.pyd" if label.startswith("windows") else "pine_compat/core.so"
            native = ("qualified " + label).encode()
            filename = "pine_compat_runtime-0.3.3-cp310-abi3-" + label.replace("-", "_") + ".whl"
            wheel_path = self.dist / filename
            with zipfile.ZipFile(wheel_path, "w") as archive:
                archive.writestr("pine_compat_runtime-0.3.3.dist-info/METADATA",
                                 "Name: pine-compat-runtime\nVersion: 0.3.3\n\n")
                archive.writestr(native_name, native)
            wheels[label] = dict(artifactId=100 + index, artifactName="wheel-" + label,
                                 filename=filename, sha256=promotion.sha(wheel_path),
                                 size=wheel_path.stat().st_size, archiveSha256=str(index) * 64,
                                 native={native_name: hashlib.sha256(native).hexdigest()})
        self.qualification = dict(status="passed", sourceCommit=self.source,
                                  freshTrials=216, crossSurfaceMismatchGroups=0,
                                  budgetAndExecutionFailures=[], unverified=[],
                                  resourcePlatforms=["Windows", "Linux"],
                                  resourceSurfaces=["rust", "python", "wasm"],
                                  macOSBoundary="Installed-wheel compatibility tested in CI; no macOS resource measurement",
                                  pythonWheelHashes={key: wheel["sha256"] for key, wheel in wheels.items()})
        self.qualification_path = "docs/qualifications/v0.3.3.json"
        self.write_json(self.qualification_path, self.qualification)
        self.plan = dict(schemaVersion=1, tag="v0.3.3", repository="owner/runtime", workflowRun=42,
                         sourceCommit=self.source, wheels=wheels,
                         resourceQualification=dict(path=self.qualification_path,
                                                    sha256=promotion.sha(self.repo / self.qualification_path)))
        self.plan_path = "release-inputs/v0.3.3.json"
        self.write_json(self.plan_path, self.plan)
        self.git("add", ".")
        self.git("commit", "-qm", "release proof")

    def git(self, *args):
        return subprocess.check_output(["git", *args], cwd=self.repo, text=True, stderr=subprocess.STDOUT)

    def write(self, name, content):
        path = self.repo / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content.encode("utf-8"))

    def write_json(self, name, value):
        self.write(name, json.dumps(value, indent=2) + "\n")

    def commit(self):
        self.git("add", ".")
        self.git("commit", "-qm", "test release mutation")

    def load(self):
        return promotion.load_plan(self.repo, "v0.3.3", "owner/runtime")

    def test_qualified_snapshot_and_all_four_wheels_pass(self):
        plan = self.load()
        for label in promotion.PLATFORMS:
            self.assertTrue(promotion.verify_wheel(plan, label, self.dist).is_file())
        self.assertEqual(promotion.source_changes(self.repo, self.source),
                         [self.qualification_path, self.plan_path])

    def test_runtime_or_package_input_change_is_rejected(self):
        for name in ("runtime.rs", "Cargo.toml", "README.md", ".cargo/config.toml", "LICENSE"):
            with self.subTest(name=name):
                self.write(name, "changed")
                self.commit()
                with self.assertRaisesRegex(promotion.PromotionError, "runtime or build inputs"):
                    self.load()
                self.git("reset", "--hard", "HEAD~1")

    def test_dirty_tracked_checkout_is_rejected(self):
        self.write("runtime.rs", "dirty")
        with self.assertRaisesRegex(promotion.PromotionError, "uncommitted"):
            self.load()

    def test_proof_hash_survives_git_checkout_line_ending_rules(self):
        self.git("checkout-index", "--force", "--", self.qualification_path, self.plan_path)
        proof = self.repo / self.qualification_path
        self.assertNotIn(b"\r", proof.read_bytes())
        self.assertEqual(promotion.sha(proof), self.plan["resourceQualification"]["sha256"])
        self.load()
        proof.write_bytes(proof.read_bytes().replace(b"\n", b"\r\n"))
        with self.assertRaisesRegex(promotion.PromotionError, "changed resource"):
            self.load()

    def test_incomplete_or_changed_resource_proof_is_rejected(self):
        for key, value in (("freshTrials", 215), ("status", "notPassed"),
                           ("crossSurfaceMismatchGroups", 1), ("unverified", ["Linux/wasm"]),
                           ("budgetAndExecutionFailures", ["update p95"]), ("macOSBoundary", "")):
            with self.subTest(key=key):
                qualification = dict(self.qualification, **{key: value})
                self.write_json(self.qualification_path, qualification)
                plan = deepcopy(self.plan)
                plan["resourceQualification"]["sha256"] = promotion.sha(self.repo / self.qualification_path)
                self.write_json(self.plan_path, plan)
                self.commit()
                with self.assertRaises(promotion.PromotionError):
                    self.load()
                self.git("reset", "--hard", "HEAD~1")
        self.write_json(self.qualification_path, dict(self.qualification, status="notPassed"))
        self.commit()
        with self.assertRaisesRegex(promotion.PromotionError, "changed resource"):
            self.load()

    def test_qualification_wheel_identity_and_paths_are_bound(self):
        for mutate in (
            lambda plan: plan["wheels"]["windows-x86_64"].update(sha256="a" * 64),
            lambda plan: plan["resourceQualification"].update(path="../outside.json"),
            lambda plan: plan["wheels"].pop("macos-arm64"),
            lambda plan: plan["wheels"]["linux-x86_64"].update(filename="C:other.whl"),
        ):
            plan = deepcopy(self.plan)
            mutate(plan)
            self.write_json(self.plan_path, plan)
            self.commit()
            with self.assertRaises(promotion.PromotionError):
                self.load()
            self.git("reset", "--hard", "HEAD~1")

    def test_producer_metadata_binds_run_source_artifact_and_archive(self):
        label = "windows-x86_64"
        wheel = self.plan["wheels"][label]
        run = dict(repository=dict(full_name="owner/runtime"), name="Wheels",
                   path=".github/workflows/wheels.yml", head_sha=self.source,
                   status="completed", conclusion="success")
        artifact = dict(id=wheel["artifactId"], name=wheel["artifactName"], expired=False,
                        digest="sha256:" + wheel["archiveSha256"],
                        workflow_run=dict(id=42, head_sha=self.source))
        def fetch(suffix):
            return run if suffix == "actions/runs/42" else artifact
        promotion.verify_producer(self.plan, label, fetch)
        for target, key, value in ((run, "head_sha", "b" * 40), (run, "conclusion", "failure"),
                                    (run, "path", "other.yml"), (artifact, "expired", True),
                                    (artifact, "digest", "sha256:" + "f" * 64),
                                    (artifact, "workflow_run", dict(id=43, head_sha=self.source))):
            previous = target[key]
            target[key] = value
            with self.subTest(key=key), self.assertRaises(promotion.PromotionError):
                promotion.verify_producer(self.plan, label, fetch)
            target[key] = previous

    def test_changed_wheel_or_native_identity_is_rejected(self):
        label = "windows-x86_64"
        plan = self.load()
        native_name = next(iter(plan["wheels"][label]["native"]))
        plan["wheels"][label]["native"][native_name] = "f" * 64
        with self.assertRaisesRegex(promotion.PromotionError, "native bytes"):
            promotion.verify_wheel(plan, label, self.dist)
        wheel = self.dist / self.plan["wheels"][label]["filename"]
        wheel.write_bytes(wheel.read_bytes() + b"changed")
        with self.assertRaisesRegex(promotion.PromotionError, "wheel bytes"):
            promotion.verify_wheel(self.plan, label, self.dist)

    def test_record_carries_producer_and_release_commit_and_all_checksums(self):
        plan = self.load()
        head = self.git("rev-parse", "HEAD").strip()
        manifest = dict(tag="v0.3.3", commit=head,
                        assets=[dict(filename=wheel["filename"], sha256=wheel["sha256"], size=wheel["size"])
                                for wheel in plan["wheels"].values()])
        self.write_json("dist/manifest.json", manifest)
        promotion.record_release(self.repo, "v0.3.3", plan, self.dist)
        result = promotion.read(self.dist / "manifest.json")
        self.assertNotIn(b"\r", (self.dist / "manifest.json").read_bytes())
        self.assertNotIn(b"\r", (self.dist / "SHA256SUMS").read_bytes())
        self.assertEqual(result["artifactPromotion"]["sourceCommit"], self.source)
        self.assertEqual(result["artifactPromotion"]["releaseCommit"], head)
        checksums = (self.dist / "SHA256SUMS").read_text().splitlines()
        self.assertEqual(len(checksums), 7)
        for entry in checksums:
            digest, filename = entry.split("  ")
            self.assertEqual(digest, promotion.sha(self.dist / filename))
        manifest["assets"][0]["sha256"] = "f" * 64
        self.write_json("dist/manifest.json", manifest)
        with self.assertRaisesRegex(promotion.PromotionError, "manifest assets"):
            promotion.record_release(self.repo, "v0.3.3", plan, self.dist)

    def test_unplanned_version_and_non_tag_runs_keep_normal_build_route(self):
        self.assertIsNone(promotion.load_plan(self.repo, "v0.3.4", "owner/runtime"))
        with patch.object(sys, "argv", ["promotion", "select", "--tag", "18/merge", "--ref", "refs/pull/18/merge"]), \
                patch.object(promotion, "load_plan", side_effect=AssertionError("must not read tag plan")):
            promotion.main()

    def test_tag_selection_pins_exact_artifact_id(self):
        label = "windows-x86_64"
        output = self.dist / "github-output.txt"
        args = ["promotion", "select", "--repo", str(self.repo), "--tag", "v0.3.3",
                "--ref", "refs/tags/v0.3.3", "--repository", "owner/runtime", "--platform", label]
        with patch.object(sys, "argv", args), patch.dict(promotion.os.environ, {"GITHUB_OUTPUT": str(output)}), \
                patch.object(promotion, "verify_producer") as verify, redirect_stdout(io.StringIO()):
            promotion.main()
        verify.assert_called_once()
        fields = dict(line.split("=", 1) for line in output.read_text().splitlines())
        self.assertEqual(fields["enabled"], "true")
        self.assertEqual(fields["run_id"], "42")
        self.assertEqual(fields["artifact_id"], str(self.plan["wheels"][label]["artifactId"]))
        self.assertEqual(fields["source_commit"], self.source)
        self.assertEqual(fields["tag"], "v0.3.3")

    def test_pull_request_exercises_only_new_release_inputs(self):
        event = self.dist / "event.json"
        output = self.dist / "candidate-output.txt"
        self.write_json("dist/event.json", {"pull_request": {"base": {"sha": self.source}}})
        args = ["promotion", "select", "--check-pull-request", "--repo", str(self.repo),
                "--tag", "18/merge", "--ref", "refs/pull/18/merge", "--repository", "owner/runtime",
                "--platform", "windows-x86_64"]
        environment = {"GITHUB_EVENT_NAME": "pull_request", "GITHUB_EVENT_PATH": str(event), "GITHUB_OUTPUT": str(output)}
        with patch.object(sys, "argv", args), patch.dict(promotion.os.environ, environment), \
                patch.object(promotion, "verify_producer") as verify, redirect_stdout(io.StringIO()):
            promotion.main()
        verify.assert_called_once()
        fields = dict(line.split("=", 1) for line in output.read_text().splitlines())
        self.assertEqual(fields["enabled"], "true")
        self.assertEqual(fields["tag"], "v0.3.3")
        self.write_json("dist/event.json", {"pull_request": {"base": {"sha": self.git("rev-parse", "HEAD").strip()}}})
        output.unlink()
        with patch.object(sys, "argv", args), patch.dict(promotion.os.environ, environment), \
                patch.object(promotion, "verify_producer", side_effect=AssertionError("old plans must not be selected")), \
                redirect_stdout(io.StringIO()):
            promotion.main()
        self.assertEqual(output.read_text().strip(), "enabled=false")


if __name__ == "__main__":
    unittest.main()
