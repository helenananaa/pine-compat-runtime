from __future__ import annotations

import csv
import json
from pathlib import Path
import sys
import tempfile
import unittest


SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))

import import_legacy_corpus  # noqa: E402
import merge_legacy_corpus_manifests  # noqa: E402


class MergeLegacyCorpusManifestTests(unittest.TestCase):
    def write_manifest(
        self,
        root: Path,
        *,
        item_id: str,
        version: int = 4,
        scope: str = "legacy_indicator",
    ) -> Path:
        root.mkdir()
        source = root / f"{item_id}.pine"
        source.write_text(
            f"//@version={version}\nstudy(\"{item_id}\")\nplot(close)\n",
            encoding="utf-8",
        )
        bars = root / "bars.csv"
        bars.write_text(
            "time,open,high,low,close,volume\n0,1,1,1,1,1\n",
            encoding="utf-8",
        )
        manifest = root / "corpus.tsv"
        import_legacy_corpus.write_tsv(
            manifest,
            import_legacy_corpus.REQUIRED_COLUMNS,
            [
                {
                    "id": item_id,
                    "source_path": source.name,
                    "declared_or_expected_version": str(version),
                    "chart_bars_path": bars.name,
                    "chart_symbol": "TEST",
                    "chart_timeframe": "1",
                    "execution_times_path": "",
                    "request_data_manifest": "",
                    "reference_output_path": "",
                    "license_class": "original",
                    "expected_scope": scope,
                    "notes": "merge test",
                }
            ],
        )
        return manifest

    def test_merge_sorts_rows_and_absolutizes_file_inputs(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            manifest_b = self.write_manifest(root / "b", item_id="b")
            manifest_a = self.write_manifest(root / "a", item_id="a")

            rows = merge_legacy_corpus_manifests.merge_manifests(
                [manifest_b, manifest_a]
            )

            self.assertEqual([row["id"] for row in rows], ["a", "b"])
            self.assertEqual(
                Path(rows[0]["source_path"]), (root / "a" / "a.pine").resolve()
            )
            self.assertEqual(
                Path(rows[1]["chart_bars_path"]),
                (root / "b" / "bars.csv").resolve(),
            )

    def test_merge_rebases_nested_request_data_paths_into_sidecars(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            corpus_root = root / "corpus"
            manifest = self.write_manifest(corpus_root, item_id="requests")
            request_bars = corpus_root / "request-bars.csv"
            request_bars.write_text(
                "time,open,high,low,close,volume\n0,2,2,2,2,2\n",
                encoding="utf-8",
            )
            request_manifest = corpus_root / "request.json"
            request_manifest.write_text(
                json.dumps({"TEST:60": request_bars.name}) + "\n",
                encoding="utf-8",
            )
            with manifest.open(newline="", encoding="utf-8") as handle:
                rows = list(csv.DictReader(handle, delimiter="\t"))
            rows[0]["request_data_manifest"] = request_manifest.name
            import_legacy_corpus.write_tsv(
                manifest, import_legacy_corpus.REQUIRED_COLUMNS, rows
            )
            sidecar = root / "merged.tsv.request-data"

            merged = merge_legacy_corpus_manifests.merge_manifests(
                [manifest], request_manifest_dir=sidecar
            )

            rebased = Path(merged[0]["request_data_manifest"])
            self.assertEqual(rebased, (sidecar / "requests.json").resolve())
            self.assertEqual(
                json.loads(rebased.read_text(encoding="utf-8")),
                {"TEST:60": str(request_bars.resolve())},
            )

    def test_merge_refuses_to_overwrite_request_data_sidecars(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            corpus_root = root / "corpus"
            manifest = self.write_manifest(corpus_root, item_id="requests")
            (corpus_root / "request-bars.csv").write_text(
                "time,open,high,low,close,volume\n0,2,2,2,2,2\n",
                encoding="utf-8",
            )
            (corpus_root / "request.json").write_text(
                json.dumps({"TEST:60": "request-bars.csv"}) + "\n",
                encoding="utf-8",
            )
            with manifest.open(newline="", encoding="utf-8") as handle:
                rows = list(csv.DictReader(handle, delimiter="\t"))
            rows[0]["request_data_manifest"] = "request.json"
            import_legacy_corpus.write_tsv(
                manifest, import_legacy_corpus.REQUIRED_COLUMNS, rows
            )
            sidecar = root / "merged.tsv.request-data"
            sidecar.mkdir()

            with self.assertRaisesRegex(
                merge_legacy_corpus_manifests.ManifestMergeError,
                "refusing to overwrite request data sidecar directory",
            ):
                merge_legacy_corpus_manifests.merge_manifests(
                    [manifest], request_manifest_dir=sidecar
                )

    def test_request_data_sidecars_reject_unsafe_ids(self) -> None:
        for item_id in ("../escape", ".", "id with spaces"):
            with self.subTest(item_id=item_id), self.assertRaisesRegex(
                merge_legacy_corpus_manifests.ManifestMergeError,
                "not safe for a request data sidecar filename",
            ):
                merge_legacy_corpus_manifests.validate_sidecar_id(item_id)

    def test_merge_filters_version_and_scope(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            v4 = self.write_manifest(root / "v4", item_id="v4")
            v5 = self.write_manifest(
                root / "v5",
                item_id="v5",
                version=5,
                scope="modern_indicator_control",
            )

            rows = merge_legacy_corpus_manifests.merge_manifests(
                [v5, v4],
                expected_version=4,
                expected_scope="legacy_indicator",
            )

            self.assertEqual([row["id"] for row in rows], ["v4"])

    def test_explicit_roots_are_paired_with_manifests(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            manifest = self.write_manifest(root / "manifest", item_id="rooted")
            external_root = root / "external"
            external_root.mkdir()
            (external_root / "rooted.pine").write_text(
                "//@version=4\nstudy(\"rooted\")\nplot(close)\n",
                encoding="utf-8",
            )

            rows = merge_legacy_corpus_manifests.merge_manifests(
                [manifest], roots=[external_root]
            )

            self.assertEqual(
                Path(rows[0]["source_path"]),
                (external_root / "rooted.pine").resolve(),
            )

    def test_explicit_root_count_must_match_manifest_count(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            manifest = self.write_manifest(root / "manifest", item_id="rooted")

            with self.assertRaisesRegex(
                merge_legacy_corpus_manifests.ManifestMergeError,
                "once per --manifest",
            ):
                merge_legacy_corpus_manifests.merge_manifests(
                    [manifest], roots=[]
                )

    def test_duplicate_ids_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            first = self.write_manifest(root / "first", item_id="same")
            second = self.write_manifest(root / "second", item_id="same")

            with self.assertRaisesRegex(
                merge_legacy_corpus_manifests.ManifestMergeError,
                "duplicate id",
            ):
                merge_legacy_corpus_manifests.merge_manifests([first, second])

    def test_explicit_exclusions_are_applied(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            kept = self.write_manifest(root / "kept", item_id="kept")
            excluded = self.write_manifest(root / "excluded", item_id="excluded")

            rows = merge_legacy_corpus_manifests.merge_manifests(
                [excluded, kept], exclude_ids={"excluded"}
            )

            self.assertEqual([row["id"] for row in rows], ["kept"])

    def test_unknown_exclusion_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            manifest = self.write_manifest(root / "kept", item_id="kept")

            with self.assertRaisesRegex(
                merge_legacy_corpus_manifests.ManifestMergeError,
                "excluded ids were not found: typo",
            ):
                merge_legacy_corpus_manifests.merge_manifests(
                    [manifest], exclude_ids={"typo"}
                )


if __name__ == "__main__":
    unittest.main()
