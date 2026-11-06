"""Common entry acquisition, compatibility and no-acquisition failure boundaries."""

import copy
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest
from unittest.mock import patch

from entry_fixture import EntryFixture, ROOT, digest


class EntryTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix='standards-entry-')
        self.addCleanup(temporary.cleanup)
        self.fixture = EntryFixture(Path(temporary.name))

    def launch(self, *args, success=True):
        f = self.fixture
        result = subprocess.run([sys.executable, str(f.entry / 'launcher.py'), *args],
                                cwd=f.root, env=f.env, capture_output=True)
        self.assertEqual(result.returncode == 0, success, result.stderr)
        return result

    def test_all_ordinary_operations_refuse_missing_cache_without_acquisition(self):
        f = self.fixture
        for operation in ['build', 'prepare', 'test', 'check', 'release', 'install',
                          'installer', 'package', 'prime', 'acceptance']:
            self.assertIn(b'make setup', self.launch(operation, success=False).stderr)
        self.assertFalse((f.root / '.craftguild').exists())
        self.assertEqual(f.calls(), [])
        self.assertFalse((f.root / 'dispatched').exists())

    def test_setup_reuse_and_every_dispatch_rechecks_bytes(self):
        f = self.fixture
        self.launch('setup')
        self.assertEqual(len(f.calls()), 2)
        self.assertTrue(f.cache.is_dir())
        self.launch('setup')
        self.launch('build')
        self.assertEqual((f.root / 'dispatched').read_text(), 'build')
        self.assertEqual(len(f.calls()), 2)
        cached = f.cache / 'craftguild-standards'
        cached.write_text('#!' + sys.executable + '\nimport pathlib\npathlib.Path("unverified").touch()\n')
        self.assertIn(b'digest differs', self.launch('build', success=False).stderr)
        self.assertFalse((f.root / 'unverified').exists())
        self.assertEqual((f.root / 'dispatched').read_text(), 'build')
        self.assertEqual(len(f.calls()), 2)

    def test_make_forwards_major_and_prime_without_acquisition(self):
        f = self.fixture
        self.launch('setup')
        result = subprocess.run(['make', '--silent', 'release', 'RELEASE_ARGS=--major --json'],
                                cwd=f.root, env=f.env, capture_output=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual((f.root / 'dispatched').read_text(), 'release --major --json')
        before = {str(p): p.read_bytes() for p in f.root.rglob('*') if p.is_file()}
        self.assertEqual(self.launch('prime').stdout, b'# Matching fixture context\n')
        self.assertEqual(before, {str(p): p.read_bytes() for p in f.root.rglob('*') if p.is_file()})
        self.assertEqual(len(f.calls()), 2)

    def test_entry_hash_and_symlink_conflicts_precede_transport(self):
        f = self.fixture
        makefile = f.entry / 'make/common.mk'
        original = makefile.read_bytes()
        makefile.write_bytes(b'conflicting entry')
        self.assertIn(b'entry hash', self.launch('setup', success=False).stderr)
        makefile.write_bytes(original)
        (f.root / '.craftguild').symlink_to(f.assets, target_is_directory=True)
        self.assertIn(b'Symlink', self.launch('setup', success=False).stderr)
        self.assertEqual(f.calls(), [])

    def test_wrong_hash_transport_failure_and_interruption_publish_no_cache(self):
        f = self.fixture
        original = f.binary.read_bytes()
        f.binary.write_bytes(b'incomplete download')
        self.assertIn(b'digest differs', self.launch('setup', success=False).stderr)
        self.assertFalse(f.cache.exists())
        f.binary.write_bytes(original)
        f.env['STUB_FAIL'] = 'yes'
        self.launch('setup', success=False)
        self.assertFalse(f.cache.exists())
        del f.env['STUB_FAIL']
        marker = f.base / 'blocked'
        f.env['STUB_BLOCK'] = str(marker)
        child = subprocess.Popen([sys.executable, str(f.entry / 'launcher.py'), 'setup'],
                                 cwd=f.root, env=f.env, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        try:
            deadline = time.monotonic() + 10
            while not marker.exists() and child.poll() is None and time.monotonic() < deadline:
                time.sleep(0.02)
            self.assertTrue(marker.exists())
            self.assertIsNone(child.poll())
            child.terminate()
            child.communicate(timeout=10)
        finally:
            if child.poll() is None:
                child.kill()
                child.communicate()
        self.assertFalse(f.cache.exists())
        self.assertFalse((f.root / '.craftguild/setup.lock').exists())
        self.assertEqual(list((f.root / '.craftguild/engine').rglob('setup-*')), [])

    def test_revision_version_compatibility_target_and_embedded_sbom_mismatch(self):
        f = self.fixture
        for field, value in [('revision', 'b' * 40), ('version', '8.0.0'),
                             ('protocol', 'future'), ('entry_protocol', 'future')]:
            machine = copy.deepcopy(f.machine)
            machine['engine'][field] = value
            f.fake_binary(machine)
            f.release['hosts'][f.machine['target']]['binary']['sha256'] = digest(f.binary)
            f.save()
            self.assertIn(b'compatibility differs', self.launch('setup', success=False).stderr)
            self.assertFalse(f.cache.exists())
        for machine in [dict(f.machine, target='foreign'), dict(f.machine, clean=False)]:
            f.fake_binary(machine)
            f.release['hosts'][f.machine['target']]['binary']['sha256'] = digest(f.binary)
            f.save()
            self.launch('setup', success=False)
            self.assertFalse(f.cache.exists())
        f.fake_binary(f.machine, embedded=b'wrong embedded sbom')
        f.release['hosts'][f.machine['target']]['binary']['sha256'] = digest(f.binary)
        f.save()
        self.assertIn(b'embedded SBOM differs', self.launch('setup', success=False).stderr)
        self.assertFalse(f.cache.exists())

    def test_supported_host_aliases_and_unsupported_or_absent_hosts(self):
        spec = importlib.util.spec_from_file_location('entry', ROOT / 'templates/launcher.py')
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        for system, suffix in [('Darwin', 'apple-darwin'), ('Linux', 'unknown-linux-gnu')]:
            for arch, expected in [('arm64', 'aarch64'), ('aarch64', 'aarch64'),
                                   ('amd64', 'x86_64'), ('x86_64', 'x86_64')]:
                with patch.object(module.platform, 'system', return_value=system), patch.object(module.platform, 'machine', return_value=arch):
                    self.assertEqual(module.host(), expected + '-' + suffix)
        with patch.object(module.platform, 'system', return_value='Windows'):
            with self.assertRaisesRegex(ValueError, 'Unsupported'):
                module.host()
        f = self.fixture
        f.release['hosts'] = {}
        f.save()
        self.assertIn(b'No published engine asset', self.launch('setup', success=False).stderr)
        self.assertEqual(f.calls(), [])

    def test_changed_cache_metadata_and_sbom_are_refused_offline(self):
        f = self.fixture
        self.launch('setup')
        receipt = f.cache / 'verified.json'
        original = receipt.read_bytes()
        receipt.write_text('{}')
        self.assertIn(b'metadata differs', self.launch('prime', success=False).stderr)
        receipt.write_bytes(original)
        (f.cache / 'engine.bom.json').write_bytes(b'tampered')
        self.assertIn(b'digest differs', self.launch('prime', success=False).stderr)
        self.assertEqual(len(f.calls()), 2)


class HostAcquisitionTests(unittest.TestCase):
    def test_each_supported_host_acquires_its_own_exact_asset(self):
        spec = importlib.util.spec_from_file_location('host_entry', ROOT / 'templates/launcher.py')
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        for system, suffix in [('Darwin', 'apple-darwin'), ('Linux', 'unknown-linux-gnu')]:
            for arch, architecture in [('arm64', 'aarch64'), ('x86_64', 'x86_64')]:
                with self.subTest(system=system, arch=arch), tempfile.TemporaryDirectory() as name:
                    machine = {'engine': {'version': '7.8.9', 'revision': 'a' * 40,
                                          'baseline': '0.1.0', 'protocol': 'craftguild-execution-1',
                                          'entry_protocol': 'craftguild-entry-1'},
                               'target': architecture + '-' + suffix, 'clean': True}
                    f = EntryFixture(Path(name).resolve(), machine=machine)
                    # The host executable here is a test identity stub, not runtime qualification.
                    with patch.object(module.platform, 'system', return_value=system), patch.object(module.platform, 'machine', return_value=arch), patch.dict(module.os.environ, f.env):
                        binary = module.engine(f.root, f.entry, 'setup')
                        self.assertEqual(binary, f.cache / 'craftguild-standards')
                        self.assertEqual(len(f.calls()), 2)
                        self.assertTrue(all(machine['target'] in url for url in f.calls()))
                        module.engine(f.root, f.entry, 'prime')
                        self.assertEqual(len(f.calls()), 2)


if __name__ == '__main__':
    unittest.main()
