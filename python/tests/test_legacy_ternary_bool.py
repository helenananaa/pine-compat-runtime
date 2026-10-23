from pathlib import Path

import pytest
import pine_compat

EVIDENCE = Path(__file__).resolve().parents[2] / "docs/evidence/strategy-library-20261010"
BARS = [dict(time=1700000000000 + i * 60000, open=1.0, high=1.0, low=1.0, close=1.0, volume=1.0) for i in range(4)]


@pytest.mark.parametrize("version", [4, 5])
def test_legacy_ternary_matches_saved_native_both_condition_directions(version):
    source = (EVIDENCE / f"bool-numeric-ternary-expanded-v{version}-oracle-24.pine").read_text(encoding="utf-8")
    result = pine_compat.compile_script(source).run(BARS)
    assert [p["values"] for p in result["plots"]] == [[x] * 4 for x in [1, 1, 1, 0, 0, 1, 1, 1, 0, 0, 0, 0]]


def test_legacy_ternary_keeps_native_bool_result_and_modern_rejection():
    for filename, message in [
        ("bool-int-ternary-v6-negative-24.pine", "incompatible types"),
        ("bool-numeric-ternary-type-negative-v5-24.pine", "bool"),
    ]:
        with pytest.raises(ValueError, match=message):
            pine_compat.compile_script((EVIDENCE / filename).read_text(encoding="utf-8"))
