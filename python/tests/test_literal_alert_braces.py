from pathlib import Path

import pine_compat
import pytest

EVIDENCE = Path(__file__).resolve().parents[2] / "docs/evidence/strategy-library-20261010"


@pytest.mark.parametrize("version", [4, 5, 6])
def test_plain_alert_braces_preserve_text_and_message_template_renders(version):
    source = (EVIDENCE / f"literal-alert-braces-v{version}-oracle-25.pine").read_text(encoding="utf-8")
    result = pine_compat.compile_script(source).run([
        dict(time=0, open=1.0, high=1.0, low=1.0, close=1.0, volume=1.0)
    ])
    assert [a["message"] for a in result["alerts"]] == [
        "{{ticker}}|{{close}}|{{unknown}}|{{",
        "{{ticker}}|{{close}}|{{unknown}}|{{x",
        "C=1",
    ]
    assert result["alerts"][2]["source"] == "{{ticker}}|{{unknown}}|{{"


def test_alertcondition_unknown_message_placeholder_is_still_explicit():
    with pytest.raises(ValueError, match="unknown"):
        pine_compat.compile_script('//@version=5\nindicator("negative")\nalertcondition(true,"title","{{unknown}}")\nplot(1)\n')
