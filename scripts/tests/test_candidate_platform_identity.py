"""Candidate Linux wheel tag in docs must match the shipped artifact filename."""
from __future__ import annotations

import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SURFACES = ROOT / "docs" / "DELIVERY_SURFACES.md"
CANDIDATE_LINUX = ROOT / ".local" / "candidate-0.3.0-rc.1" / "linux"
WHEEL_NAME = re.compile(
    r"pine_compat_runtime-0\.3\.0rc1-cp310-abi3-(?P<tag>manylinux_2_\d+_x86_64)\.whl"
)


class CandidatePlatformIdentityTests(unittest.TestCase):
    def test_surface_doc_names_the_shipped_linux_wheel_tag(self):
        surfaces = SURFACES.read_text(encoding="utf-8")
        qualified = self._qualified_targets_paragraph(surfaces)
        self.assertRegex(qualified.lower(), r"not rebuilt|unverif|do not treat")
        wheels = sorted(CANDIDATE_LINUX.glob("pine_compat_runtime-0.3.0rc1-*.whl"))
        if not wheels:
            self.skipTest(f"candidate Linux wheel not retained under {CANDIDATE_LINUX}")
        self.assertEqual(len(wheels), 1, wheels)
        match = WHEEL_NAME.fullmatch(wheels[0].name)
        self.assertIsNotNone(match, wheels[0].name)
        tag = match.group("tag")
        first = re.search(r"manylinux_2_\d+_x86_64", qualified)
        self.assertIsNotNone(first)
        self.assertEqual(first.group(0), tag)
        self.assertNotEqual(tag, "manylinux_2_17_x86_64")

    @staticmethod
    def _qualified_targets_paragraph(text: str) -> str:
        start = text.index("Qualified desktop targets")
        return text[start : text.index("Default chart context", start)]


if __name__ == "__main__":
    unittest.main()
