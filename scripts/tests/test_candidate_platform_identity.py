"""Candidate Linux wheel tag in docs must match the shipped artifact filename."""
from __future__ import annotations

import re
import unittest
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SURFACES = ROOT / "docs" / "DELIVERY_SURFACES.md"
CANDIDATE_LINUX = ROOT / ".local" / "candidate-0.3.0-rc.1" / "linux"
WHEEL_NAME = re.compile(
    r"pine_compat_runtime-0\.3\.0rc1-cp310-abi3-(?P<tag>manylinux_2_\d+_x86_64)(?:\.manylinux2014_x86_64)?\.whl"
)


class CandidatePlatformIdentityTests(unittest.TestCase):
    def test_surface_doc_names_the_shipped_linux_wheel_tag(self):
        surfaces = SURFACES.read_text(encoding="utf-8")
        qualified = self._qualified_targets_paragraph(surfaces)
        wheels = sorted(CANDIDATE_LINUX.rglob("pine_compat_runtime-0.3.0rc1-*.whl"))
        if not wheels:
            self.skipTest(f"candidate Linux wheel not retained under {CANDIDATE_LINUX}")
        for wheel in wheels:
            with self.subTest(wheel=wheel.name):
                match = WHEEL_NAME.fullmatch(wheel.name)
                self.assertIsNotNone(match, wheel.name)
                tag = match.group("tag")
                self.assertIn(tag, qualified)
                with zipfile.ZipFile(wheel) as archive:
                    metadata = [n for n in archive.namelist() if n.endswith('.dist-info/WHEEL')]
                    self.assertEqual(len(metadata), 1)
                    self.assertIn('Tag: cp310-abi3-' + tag,
                                  archive.read(metadata[0]).decode('utf-8'))

    @staticmethod
    def _qualified_targets_paragraph(text: str) -> str:
        start = text.index("Qualified desktop targets")
        return text[start : text.index("Default chart context", start)]


if __name__ == "__main__":
    unittest.main()
