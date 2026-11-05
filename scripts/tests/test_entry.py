"""Pinned entry integrity and missing-engine refusal, without any build tool."""

import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
PROTOCOL = 'craftguild-execution-1'


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def fixture(root):
    entry = root / 'standards'
    bundle = entry / 'bundle'
    (bundle / 'bin').mkdir(parents=True)
    (bundle / 'make').mkdir()
    shutil.copyfile(ROOT / 'templates/launcher.py', bundle / 'launcher.py')
    shutil.copyfile(ROOT / 'templates/make/common.mk', bundle / 'make/common.mk')
    binary = bundle / 'bin/craftguild-standards'
    binary.write_text('#!' + sys.executable + '\nimport pathlib,sys\n'
                      'if sys.argv[1] == "protocol": print("' + PROTOCOL + '")\n'
                      'else: pathlib.Path("dispatched").write_text(sys.argv[1])\n')
    binary.chmod(0o755)
    source = {'schema_version': 2, 'kind': 'source-export', 'inventory_sha256': 'a' * 64}
    (bundle / 'source.json').write_text(json.dumps(source))
    source_digest = hashlib.sha256(json.dumps(source, sort_keys=True,
                                             separators=(',', ':')).encode()).hexdigest()
    metadata = {'schema_version': 1, 'protocol': PROTOCOL,
                'engine': 'bin/craftguild-standards',
                'source': {'kind': 'source-export', 'sha256': source_digest},
                'files': {p.relative_to(bundle).as_posix(): digest(p)
                          for p in bundle.rglob('*') if p.is_file()}}
    (bundle / 'bundle.json').write_text(json.dumps(metadata))
    (entry / 'make').mkdir()
    shutil.copyfile(bundle / 'launcher.py', entry / 'launcher.py')
    shutil.copyfile(bundle / 'make/common.mk', entry / 'make/common.mk')
    expected = digest(bundle / 'bundle.json')
    (entry / 'engine.json').write_text(json.dumps({'schema_version': 1,
                                                'protocol': PROTOCOL,
                                                'bundle': 'bundle',
                                                'bundle_sha256': expected}))
    (root / 'Makefile').write_text('include standards/make/common.mk\n')
    return entry, root / '.craftguild/engine' / expected


def launch(root, operation):
    return subprocess.run([sys.executable, str(root / 'standards/launcher.py'), operation],
                          cwd=root, capture_output=True)


class EntryTests(unittest.TestCase):
    def test_install_does_not_acquire_even_vendored_engine(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            _, cache = fixture(root)
            result = launch(root, 'install')
            self.assertNotEqual(result.returncode, 0)
            self.assertIn(b'without privileges', result.stderr)
            self.assertFalse(cache.exists())
            self.assertFalse((root / 'dispatched').exists())

    def test_first_use_cache_reuse_and_tamper_refusal(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            entry, cache = fixture(root)
            first = launch(root, 'build')
            self.assertEqual(first.returncode, 0, first.stderr)
            self.assertEqual((root / 'dispatched').read_text(), 'build')
            self.assertTrue(cache.is_dir())
            shutil.rmtree(entry / 'bundle')
            repeated = launch(root, 'install')
            self.assertEqual(repeated.returncode, 0, repeated.stderr)
            self.assertEqual((root / 'dispatched').read_text(), 'install')
            (cache / 'bin/craftguild-standards').write_text('tampered engine')
            refused = launch(root, 'build')
            self.assertNotEqual(refused.returncode, 0)
            self.assertIn(b'digest differs', refused.stderr)
            self.assertEqual((root / 'dispatched').read_text(), 'install')

    def test_mixed_entry_and_engine_refused_before_dispatch(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            entry, _ = fixture(root)
            (entry / 'make/common.mk').write_text('incompatible entry')
            refused = launch(root, 'build')
            self.assertNotEqual(refused.returncode, 0)
            self.assertIn(b'different exports', refused.stderr)
            self.assertFalse((root / 'dispatched').exists())


if __name__ == '__main__':
    unittest.main()
