"""Exercise release planning and writes against signed, isolated Git histories."""

import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

from signing import signed_git


class Release(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='craftguild-release-test-')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        signing = signed_git()
        self.git = signing.__enter__()
        self.addCleanup(signing.__exit__, None, None, None)
        self.git(self.root, 'init', '-b', 'main')
        self.write('.gitignore', '.craftguild/\n')
        self.write('package.json', json.dumps({'name': 'demo', 'version': '0.1.0'}))
        self.write('package-lock.json', json.dumps({
            'name': 'demo', 'version': '0.1.0', 'lockfileVersion': 3,
            'packages': {'': {'name': 'demo', 'version': '0.1.0'},
                         'node_modules/other': {'version': '9.0.0'}}}))
        self.write('Cargo.toml', '[package]\nname="demo"\nversion="0.1.0"\nedition="2021"\n')
        self.write('src/main.rs', 'fn main() {}\n')
        self.write('Cargo.lock', 'version = 3\n[[package]]\nname="demo"\nversion="0.1.0"\n')
        self.write('craftguild.toml', '''schema_version=1
standards_version="0.1.0"
[[artifacts]]
id="demo"
command="demo"
project="demo"
kind="rust"
manifest="Cargo.toml"
lockfile="Cargo.lock"
package="demo"
[[releases]]
id="demo"
initial_version="0.1.0"
tag_prefix="v"
notes="docs/releases"
paths=["src", "Cargo.toml", "Cargo.lock", "package.json", "package-lock.json", "craftguild.toml"]
[[releases.cargo]]
manifest="Cargo.toml"
lockfile="Cargo.lock"
packages=["demo"]
[[releases.npm]]
manifest="package.json"
lockfile="package-lock.json"
''')
        self.commit('feat: initialize fixture')
        project = Path(__file__).resolve().parents[2]
        target = Path(os.environ.get('CARGO_TARGET_DIR', project / 'target'))
        if not target.is_absolute():
            target = project / target
        self.tool = target / 'debug/craftguild-standards-bootstrap'

    def write(self, name, text):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)

    def commit(self, message):
        self.git(self.root, 'add', '.')
        self.git(self.root, 'commit', '-S', '-qm', message)

    def release(self, *args, success=True):
        result = subprocess.run([str(self.tool), 'release', '--json', *args],
                                cwd=self.root, env=self.git.environment,
                                capture_output=True, text=True)
        if success:
            self.assertEqual(result.returncode, 0, result.stderr)
            return json.loads(result.stdout)
        self.assertNotEqual(result.returncode, 0)
        return result.stderr

    def baseline(self):
        self.git(self.root, 'tag', 'v0.1.0')

    def change(self, message):
        self.write('src/main.rs', (self.root / 'src/main.rs').read_text() + '// change\n')
        self.commit(message)

    def test_initial_preview_apply_replay_and_committed_replay(self):
        before = self.git(self.root, 'status', '--porcelain')
        plan = self.release()
        self.assertEqual(plan['next'], '0.1.0')
        self.assertIsNone(plan['base'])
        self.assertEqual(before, self.git(self.root, 'status', '--porcelain'))
        self.assertEqual(plan, self.release())
        self.release('--apply')
        self.assertTrue(self.release('--apply')['prepared'])
        self.commit('chore(release): prepare v0.1.0')
        self.assertTrue(self.release('--apply')['prepared'])
        self.git(self.root, 'tag', 'v0.1.0')
        self.assertIsNone(self.release()['next'])

    def test_bumps_and_maximum_once(self):
        self.baseline()
        cases = [('fix(parser): correct parsing', '0.1.1'),
                 ('feat(api): add operation', '0.2.0'),
                 ('refactor(api)!: remove operation', '1.0.0')]
        for message, version in cases:
            self.change(message)
            plan = self.release()
            self.assertEqual(plan['next'], version)
            self.assertIn(message, plan['notes'])
        plan = self.release('--apply')
        self.assertEqual(plan['next'], '1.0.0')
        self.assertEqual(json.loads((self.root / 'package.json').read_text())['version'], '1.0.0')
        lock = json.loads((self.root / 'package-lock.json').read_text())
        self.assertEqual(lock['version'], '1.0.0')
        self.assertEqual(lock['packages']['']['version'], '1.0.0')
        self.assertEqual(lock['packages']['node_modules/other']['version'], '9.0.0')
        metadata = json.loads(subprocess.check_output(
            ['cargo', 'metadata', '--locked', '--offline', '--no-deps', '--format-version=1'],
            cwd=self.root, env=self.git.environment))
        self.assertEqual(metadata['packages'][0]['version'], '1.0.0')
        self.assertTrue(self.release()['prepared'])

    def test_breaking_footers_and_invalid_message(self):
        self.baseline()
        for token in ['BREAKING CHANGE', 'BREAKING-CHANGE']:
            self.change('fix: adapt contract\n\n' + token + ': callers must migrate')
            plan = self.release()
            self.assertEqual(plan['next'], '1.0.0')
            self.assertIn('callers must migrate', plan['notes'])
        self.change('not a conventional message')
        self.assertIn('Conventional', self.release(success=False))

    def test_no_bump_scope_and_dirty_refusal(self):
        self.baseline()
        self.change('docs: explain operation')
        self.assertIsNone(self.release('--apply')['next'])
        self.write('unrelated', 'do not include')
        self.assertIn('clean working tree', self.release('--apply', success=False))
        self.commit('feat: unrelated path')
        self.assertIsNone(self.release()['next'])

    def test_lock_drift_and_preparation_drift(self):
        self.baseline()
        self.change('fix: correct behavior')
        self.release('--apply')
        self.write('src/main.rs', 'fn main() { println!("new work"); }\n')
        self.assertIn('unrelated work', self.release('--apply', success=False))
        self.git(self.root, 'restore', 'src/main.rs')
        self.write('package-lock.json', '{}')
        self.assertIn('drift', self.release('--apply', success=False))

    def test_reject_inconsistent_lock_versions(self):
        self.baseline()
        self.change('fix: correct behavior')
        self.write('package-lock.json', (self.root / 'package-lock.json').read_text().replace('0.1.0', '0.1.8'))
        self.commit('test: record inconsistent lock')
        self.assertIn('mismatch', self.release(success=False))

    def test_simulated_gpg_failure_cannot_write_or_publish(self):
        # Every real fixture commit is signed. Only the verifier is simulated.
        self.baseline()
        self.change('fix: correct behavior')
        fake = self.root / '.craftguild/failure-bin'
        fake.mkdir(parents=True)
        (fake / 'gpg').write_text('#!/bin/sh\nexit 1\n')
        (fake / 'gpg').chmod(0o700)
        refs = self.git(self.root, 'show-ref')
        self.git.environment['PATH'] = str(fake) + os.pathsep + os.environ['PATH']
        self.assertIn('Required command failed', self.release('--apply', success=False))
        self.assertEqual(refs, self.git(self.root, 'show-ref'))
        self.assertFalse((self.root / 'docs/releases').exists())
        self.assertEqual(self.git(self.root, 'status', '--porcelain'), '')

    def test_foreign_target_tag_is_never_overwritten(self):
        self.baseline()
        self.git(self.root, 'checkout', '--orphan', 'foreign')
        self.commit('feat: independent history')
        self.git(self.root, 'tag', 'v0.1.1')
        self.git(self.root, 'checkout', 'main')
        self.change('fix: correct behavior')
        self.assertIn('already exists', self.release('--apply', success=False))

    def test_independent_group_keeps_other_versions_unchanged(self):
        self.write('ui/package.json', json.dumps({'name': 'ui', 'version': '2.0.0'}))
        self.write('ui/package-lock.json', json.dumps({
            'name': 'ui', 'version': '2.0.0', 'lockfileVersion': 3,
            'packages': {'': {'name': 'ui', 'version': '2.0.0'}}}))
        config = (self.root / 'craftguild.toml').read_text() + '''
[[releases]]
id="ui"
initial_version="2.0.0"
tag_prefix="ui-v"
notes="docs/ui-releases"
paths=["ui"]
[[releases.npm]]
manifest="ui/package.json"
lockfile="ui/package-lock.json"
'''
        self.write('craftguild.toml', config)
        self.commit('chore: configure independent UI')
        self.baseline()
        self.git(self.root, 'tag', 'ui-v2.0.0')
        self.write('ui/main.js', 'export const feature = true;\n')
        self.commit('feat(ui): add UI feature')
        self.assertIsNone(self.release('--group', 'demo')['next'])
        self.assertEqual(self.release('--group', 'ui', '--apply')['next'], '2.1.0')
        self.assertEqual(json.loads((self.root / 'ui/package.json').read_text())['version'], '2.1.0')
        self.assertEqual(json.loads((self.root / 'package.json').read_text())['version'], '0.1.0')


if __name__ == '__main__':
    unittest.main()
