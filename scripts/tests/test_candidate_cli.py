"""Drive the shipped CLI binary for candidate identity and a public frozen fixture."""
from __future__ import annotations

import json
import subprocess
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


class CandidateCliTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        build = subprocess.run(
            ['cargo', 'build', '-p', 'pine-cli', '--message-format=json'],
            cwd=ROOT, text=True, capture_output=True, check=True,
        )
        artifacts = [json.loads(line) for line in build.stdout.splitlines() if line.startswith('{')]
        cls.binary = Path(next(
            a['executable'] for a in artifacts
            if a.get('executable') and a.get('target', {}).get('name') == 'pine-compat'
        ))

    def test_version_and_public_macd_fixture_are_stable_across_launches(self):
        version = subprocess.run(
            [str(self.binary), '--version'], cwd=ROOT, text=True, capture_output=True, check=True,
        )
        self.assertEqual(version.stdout.strip(), 'pine-compat 0.3.0-rc.1')
        args = [
            str(self.binary), 'run',
            str(ROOT / 'tests/fixtures/runtime/macd.pine'),
            '--bars', str(ROOT / 'tests/fixtures/runtime/bars.csv'),
        ]
        first = subprocess.run(args, cwd=ROOT, text=True, capture_output=True, check=True)
        second = subprocess.run(args, cwd=ROOT, text=True, capture_output=True, check=True)
        self.assertEqual(first.stdout, second.stdout)
        payload = json.loads(first.stdout)
        expected = json.loads((ROOT / 'tests/snapshots/runtime_macd.json').read_text(encoding='utf-8'))
        self.assertEqual(payload['schemaVersion'], 8)
        self.assertEqual(payload['plots'][0]['values'], expected['plots'][0]['values'])
        incremental = subprocess.run(
            [str(self.binary), 'run-incremental',
             str(ROOT / 'tests/fixtures/runtime/macd.pine'),
             '--bars', str(ROOT / 'tests/fixtures/runtime/bars.csv')],
            cwd=ROOT, text=True, capture_output=True, check=True,
        )
        self.assertEqual(json.loads(incremental.stdout)['plots'][0]['values'], payload['plots'][0]['values'])


if __name__ == '__main__':
    unittest.main()
