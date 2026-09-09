from __future__ import annotations

import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest
import zipfile


SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))

import build_wheel_manifest  # noqa: E402


def write_wheel(
    directory: Path,
    platform_tag: str,
    *,
    version: str = "0.1.0",
    distribution: str = "pine-compat-runtime",
) -> Path:
    filename = (
        f"pine_compat_runtime-{version}-cp310-abi3-{platform_tag}.whl"
    )
    path = directory / filename
    metadata_dir = f"pine_compat_runtime-{version}.dist-info"
    metadata = (
        "Metadata-Version: 2.4\n"
        f"Name: {distribution}\n"
        f"Version: {version}\n"
        "Requires-Python: >=3.10\n"
        "\n"
    )
    with zipfile.ZipFile(path, "w") as archive:
        archive.writestr(f"{metadata_dir}/METADATA", metadata)
    return path


class BuildWheelManifestTests(unittest.TestCase):
    def test_builds_deterministic_manifest_and_checksums(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            dist = Path(temp_dir)
            linux = write_wheel(dist, "manylinux_2_17_x86_64")
            windows = write_wheel(dist, "win_amd64")

            manifest = build_wheel_manifest.build_manifest(
                dist,
                tag="v0.1.0",
                commit="abc123",
                expected_wheel_count=2,
            )
            manifest_path, checksums_path = build_wheel_manifest.write_release_files(
                dist, manifest
            )

            rendered = json.loads(manifest_path.read_text())
            self.assertEqual(rendered["version"], "0.1.0")
            self.assertEqual(rendered["channel"], "stable")
            self.assertEqual(rendered["python_requires"], ">=3.10")
            self.assertEqual(
                [asset["platform_tag"] for asset in rendered["assets"]],
                ["manylinux_2_17_x86_64", "win_amd64"],
            )
            self.assertEqual(
                rendered["assets"][0]["sha256"],
                hashlib.sha256(linux.read_bytes()).hexdigest(),
            )
            self.assertIn(windows.name, checksums_path.read_text())
            self.assertIn("manifest.json", checksums_path.read_text())

    def test_rejects_tag_version_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            dist = Path(temp_dir)
            write_wheel(dist, "win_amd64")

            with self.assertRaisesRegex(
                build_wheel_manifest.ManifestError, "does not match tag"
            ):
                build_wheel_manifest.build_manifest(
                    dist,
                    tag="v0.2.0",
                    commit="abc123",
                )

    def test_rejects_wrong_wheel_count(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            dist = Path(temp_dir)
            write_wheel(dist, "win_amd64")

            with self.assertRaisesRegex(
                build_wheel_manifest.ManifestError, "expected 2 wheels"
            ):
                build_wheel_manifest.build_manifest(
                    dist,
                    tag="v0.1.0",
                    commit="abc123",
                    expected_wheel_count=2,
                )

    def test_candidate_and_development_versions_are_not_stable(self) -> None:
        for version in ("0.3.0a1", "0.3.0b1", "0.3.0rc1", "0.3.0.dev1"):
            with self.subTest(version=version), tempfile.TemporaryDirectory() as temp_dir:
                dist = Path(temp_dir)
                write_wheel(dist, "win_amd64", version=version)
                manifest = build_wheel_manifest.build_manifest(dist, tag=f"v{version}", commit="abc123")
                self.assertEqual(manifest["channel"], "prerelease")
                self.assertEqual(manifest["version"], version)

    def test_semver_candidate_tag_matches_normalized_wheel_version(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            dist = Path(temp_dir)
            write_wheel(dist, "win_amd64", version="0.3.0rc1")
            manifest = build_wheel_manifest.build_manifest(dist, tag="v0.3.0-rc.1", commit="abc123")
            self.assertEqual(manifest["version"], "0.3.0rc1")
            self.assertEqual(manifest["tag"], "v0.3.0-rc.1")
            self.assertEqual(manifest["channel"], "prerelease")

    def test_malformed_tag_or_wheel_version_is_rejected(self) -> None:
        for tag, version, reason in (("vnot-a-version", "0.1.0", "invalid release version"),
                                     ("v0.1.0", "broken", "invalid wheel version")):
            with self.subTest(tag=tag, version=version), tempfile.TemporaryDirectory() as temp_dir:
                dist = Path(temp_dir)
                write_wheel(dist, "win_amd64", version=version)
                with self.assertRaisesRegex(build_wheel_manifest.ManifestError, reason):
                    build_wheel_manifest.build_manifest(dist, tag=tag, commit="abc123")


if __name__ == "__main__":
    unittest.main()
