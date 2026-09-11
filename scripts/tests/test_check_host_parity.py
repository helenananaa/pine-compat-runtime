from pathlib import Path
import sys
import unittest


SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))

import check_host_parity  # noqa: E402


class HostParityGuardTests(unittest.TestCase):
    def test_library_registry_includes_complete_dependency_bundles(self):
        source = '''
const LIBRARIES: &[LibraryFixture] = &[
    ("one.json", "one.pine", &[("user/a/1", "a.pine")]),
    (
        "nested.json", "nested.pine",
        &[
            ("user/a/1", "a.pine",),
            ("user/b/1", "b.pine"),
        ],
    ),
];
// ("comment.json", "fake.pine", &[]),
/* ("block.json", "fake.pine", &[]), */
let fake = r#"("string.json", "fake.pine", &[])"#;
'''
        fixtures = check_host_parity.parse_library_snapshot_fixtures(
            source, Path("fixtures.rs")
        )
        self.assertEqual(
            [(item.snapshot, item.source) for item in fixtures],
            [("one.json", "one.pine"), ("nested.json", "nested.pine")],
        )
        registered = {item.snapshot for item in fixtures}
        self.assertEqual(check_host_parity.parity_errors(
            registered, registered, registered, registered
        ), [])
        self.assertIn("not registered by the CLI: nested.json", "\n".join(
            check_host_parity.parity_errors(
                registered - {"nested.json"}, registered, registered, registered
            )
        ))
        self.assertIn("missing a WASM golden assertion", "\n".join(
            check_host_parity.parity_errors(
                registered, registered, {"one.json"}, registered
            )
        ))

    def test_library_registry_rejects_incomplete_dependency_tuple(self):
        self.assertEqual(check_host_parity.parse_library_snapshot_fixtures(
            '("broken.json", "root.pine", &[("user/a/1", )]),',
            Path("fixtures.rs"),
        ), [])

    def test_parses_rustfmt_multiline_tuples_and_trailing_commas(self):
        fixtures = check_host_parity.parse_runtime_snapshot_fixtures(
            '''
const FIXTURES: &[Fixture] = &[
    ("runtime_inline.json", "tests/fixtures/runtime/inline.pine"),
    (
        "runtime_multiline.json",
        "tests/fixtures/runtime/multiline.pine",
    ),
];
''',
            Path("fixtures.rs"),
        )

        self.assertEqual(
            [(item.snapshot, item.source) for item in fixtures],
            [
                ("runtime_inline.json", "tests/fixtures/runtime/inline.pine"),
                (
                    "runtime_multiline.json",
                    "tests/fixtures/runtime/multiline.pine",
                ),
            ],
        )

    def test_deleting_either_required_host_assertion_fails(self):
        registered = {"required.json", "registered_only.json"}
        required = {"required.json"}
        both_hosts = {"required.json"}

        self.assertEqual(
            check_host_parity.parity_errors(
                registered, required, both_hosts, both_hosts
            ),
            [],
        )
        self.assertIn(
            "missing a WASM golden assertion",
            "\n".join(
                check_host_parity.parity_errors(
                    registered, required, set(), both_hosts
                )
            ),
        )
        self.assertIn(
            "missing a Python golden assertion",
            "\n".join(
                check_host_parity.parity_errors(
                    registered, required, both_hosts, set()
                )
            ),
        )

    def test_python_snapshot_path_counts_only_when_expected_is_asserted(self):
        assignment = '''
def test_contract():
    expected = json.loads(
        (ROOT / "tests/snapshots/required.json").read_text()
    )
    result = run_fixture()
'''

        self.assertEqual(
            check_host_parity.python_snapshot_assertions(
                assignment + "    assert result == expected\n"
            ),
            {"required.json"},
        )
        self.assertEqual(
            check_host_parity.python_snapshot_assertions(assignment),
            set(),
        )

    def test_python_snapshot_path_accepts_registered_assertion_helper(self):
        source = '''
def test_contract():
    expected = json.loads(
        (ROOT / "tests/snapshots/required.json").read_text()
    )
    result = run_fixture()
    assert_json_close(result, expected)
'''

        self.assertEqual(
            check_host_parity.python_snapshot_assertions(source),
            {"required.json"},
        )

    def test_wasm_snapshot_assertion_ignores_comments_and_strings(self):
        source = r'''
// assert_snapshot("commented.json", &output);
/* assert_snapshot("block_comment.json", &output); */
let ordinary = "assert_snapshot(\"ordinary_string.json\", &output);";
let raw = r#"assert_snapshot("raw_string.json", &output);"#;
let lifetime: &'static str = "still code";
assert_snapshot("real.json", &output);
'''

        self.assertEqual(
            check_host_parity.wasm_snapshot_assertions(source),
            {"real.json"},
        )

    def test_analysis_snapshot_helpers_count_as_real_host_assertions(self):
        python_source = '''
def test_contract():
    assert_analysis_snapshot(
        "tests/fixtures/legacy/v2/runtime/core_legacy.pine",
        "tests/snapshots/analysis_legacy_v2_core.json",
    )
'''
        wasm_source = '''
assert_analysis_snapshot("analysis_legacy_v2_core.json", &output);
'''

        self.assertEqual(
            check_host_parity.python_snapshot_assertions(python_source),
            {"analysis_legacy_v2_core.json"},
        )
        self.assertEqual(
            check_host_parity.wasm_snapshot_assertions(wasm_source),
            {"analysis_legacy_v2_core.json"},
        )

    def test_new_paired_assertion_must_be_added_to_manifest(self):
        errors = check_host_parity.parity_errors(
            {"required.json", "new.json"},
            {"required.json"},
            {"required.json", "new.json"},
            {"required.json", "new.json"},
        )

        self.assertEqual(
            errors,
            ["paired host snapshot is not recorded in the required manifest: new.json"],
        )

    def test_registered_single_host_assertions_require_pair_or_reasoned_allowlist(self):
        registered = {"wasm_only.json", "python_only.json"}

        errors = check_host_parity.parity_errors(
            registered,
            set(),
            {"wasm_only.json"},
            {"python_only.json"},
        )

        self.assertEqual(
            errors,
            [
                "registered snapshot python_only.json has only a Python golden assertion",
                "registered snapshot wasm_only.json has only a WASM golden assertion",
            ],
        )
        self.assertEqual(
            check_host_parity.parity_errors(
                registered,
                set(),
                {"wasm_only.json"},
                {"python_only.json"},
                {
                    "python_only.json": "Python-only API boundary",
                    "wasm_only.json": "WASM-only API boundary",
                },
            ),
            [],
        )

    def test_unpaired_allowlist_requires_a_live_reasoned_exception(self):
        errors = check_host_parity.parity_errors(
            {"paired.json", "wasm_only.json"},
            {"paired.json"},
            {"paired.json", "wasm_only.json"},
            {"paired.json"},
            {
                "missing.json": "not registered",
                "paired.json": "already required",
                "wasm_only.json": "",
            },
        )

        self.assertIn(
            "unpaired snapshot allowlist entry wasm_only.json must include a reason",
            errors,
        )
        self.assertIn(
            "unpaired snapshot allowlist entry is not registered by the CLI: missing.json",
            errors,
        )
        self.assertIn(
            "required snapshot cannot be exempted from host parity: paired.json",
            errors,
        )


if __name__ == "__main__":
    unittest.main()
