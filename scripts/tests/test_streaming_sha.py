import hashlib
import sys
import tempfile
import tracemalloc
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from requalify_core_scripts import sha


class StreamingShaTests(unittest.TestCase):
    def test_empty_multi_chunk_and_exact_eof_match_sha256_without_read_bytes(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'output.json'
            for data in (b'', b'x' * (2 * 1024 * 1024) + b'last-byte',
                         b'x' * (2 * 1024 * 1024) + b'last-byte\x00'):
                path.write_bytes(data)
                expected = hashlib.sha256(data).hexdigest()
                with patch.object(Path, 'read_bytes', side_effect=AssertionError('unbounded read')):
                    self.assertEqual(sha(path), expected)

    def test_large_file_hash_memory_is_bounded_by_chunks(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'large-output.json'
            block = b'0123456789abcdef' * 65536
            expected = hashlib.sha256()
            with path.open('wb') as stream:
                for _ in range(32):
                    stream.write(block)
                    expected.update(block)
            tracemalloc.start()
            try:
                actual = sha(path)
                _, peak = tracemalloc.get_traced_memory()
            finally:
                tracemalloc.stop()
            self.assertEqual(actual, expected.hexdigest())
            self.assertLess(peak, 3 * 1024 * 1024,
                            '32 MiB input must not become a whole-file hash buffer')


if __name__ == '__main__':
    unittest.main()
