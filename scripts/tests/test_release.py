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

    def set_version(self, version):
        for name in ['Cargo.toml', 'Cargo.lock', 'package.json', 'package-lock.json']:
            self.write(name, (self.root / name).read_text().replace('0.1.0', version))
        self.write('craftguild.toml', (self.root / 'craftguild.toml').read_text().replace('initial_version="0.1.0"', 'initial_version="' + version + '"'))
        self.commit('chore: configure fixture version')

    def test_explicit_major_tagged_empty_fix_nonbumping_and_breaking_ranges(self):
        self.baseline()
        for message in [None, 'fix: adjust behavior', 'docs: describe behavior',
                        'feat!: remove behavior']:
            self.git(self.root, 'reset', '--hard', 'v0.1.0')
            if message:
                self.change(message)
            plan = self.release('--major')
            self.assertEqual(plan['next'], '1.0.0')
            self.assertEqual(plan['bump'], 'major')
            self.assertEqual(plan['mode'], 'explicit-major')
        self.release('--major', '--apply')
        before = {p.name: p.read_bytes() for p in (self.root / 'docs/releases').iterdir()}
        self.assertIn('mode conflict', self.release('--apply', success=False))
        self.assertTrue(self.release('--major', '--apply')['prepared'])
        self.commit('chore(release): prepare explicit major')
        self.assertIn('mode conflict', self.release(success=False))
        self.assertEqual(self.release('--major', '--apply')['next'], '1.0.0')
        self.assertEqual(before, {p.name: p.read_bytes() for p in (self.root / 'docs/releases').iterdir()})
        for name in ['package.json', 'package-lock.json']:
            self.assertEqual(json.loads((self.root / name).read_text())['version'], '1.0.0')
        self.assertRegex((self.root / 'Cargo.lock').read_text(), r'version\s*=\s*"1.0.0"')

    def test_normal_preparation_rejects_explicit_mode_before_and_after_commit(self):
        self.baseline()
        self.change('fix: adjust behavior')
        self.release('--apply')
        for committed in [False, True]:
            if committed:
                self.commit('chore(release): prepare patch')
            before = self.git(self.root, 'status', '--porcelain')
            self.assertIn('mode conflict', self.release('--major', '--apply', success=False))
            self.assertEqual(before, self.git(self.root, 'status', '--porcelain'))
            self.assertEqual(self.release('--apply')['next'], '0.1.1')

    def test_mode_conflict_precedes_product_preparation_hooks(self):
        self.write('hook.py', 'from pathlib import Path\nPath(".craftguild/hook.json").write_text("{}")\n')
        config = (self.root / 'craftguild.toml').read_text().replace('schema_version=1', 'schema_version=2')
        self.write('craftguild.toml', config + '''
[[operations]]
name="release"
artifact="demo"
[[operations.hooks]]
id="observe"
stage="prepare"
argv=["python3", "hook.py"]
inputs=["hook.py"]
outputs=[".craftguild/hook.json"]
''')
        self.commit('chore: configure release hook')
        self.release('--major', '--apply')
        marker = self.root / '.craftguild/hook.json'
        self.assertTrue(marker.is_file())
        marker.unlink()
        before = {str(p.relative_to(self.root)): p.read_bytes()
                  for p in self.root.rglob('*') if p.is_file() and '.git' not in p.parts}
        self.assertIn('mode conflict', self.release('--apply', success=False))
        self.assertEqual(before, {str(p.relative_to(self.root)): p.read_bytes()
                                 for p in self.root.rglob('*') if p.is_file() and '.git' not in p.parts})

    def test_explicit_major_initial_and_later_major_releases(self):
        self.set_version('1.2.3')
        self.assertEqual(self.release()['next'], '1.2.3')
        self.assertEqual(self.release('--major')['next'], '2.0.0')
        self.release('--major', '--apply')
        self.commit('chore(release): prepare initial explicit major')
        self.git(self.root, 'tag', 'v2.0.0')
        self.change('feat!: replace operation')
        self.assertEqual(self.release()['next'], '3.0.0')
        self.assertEqual(self.release('--major')['next'], '3.0.0')
        self.release('--major', '--apply')
        self.assertEqual(self.release('--major')['next'], '3.0.0')

    def test_explicit_major_from_a_tagged_one_x_group(self):
        self.set_version('1.2.3')
        self.git(self.root, 'tag', 'v1.2.3')
        self.change('fix: repair stable operation')
        self.assertEqual(self.release('--major')['next'], '2.0.0')
        self.release('--major', '--apply')
        self.assertEqual(self.release('--major', '--apply')['next'], '2.0.0')
        self.assertIn('mode conflict', self.release('--apply', success=False))

    def test_explicit_major_initial_zero_and_legacy_receipt_is_not_authorization(self):
        self.assertEqual(self.release('--major')['next'], '1.0.0')
        self.release('--major', '--apply')
        path = self.root / 'docs/releases/v1.0.0.json'
        receipt = json.loads(path.read_text())
        del receipt['mode']
        path.write_text(json.dumps(receipt))
        self.assertIn('missing preparation intent', self.release('--apply', success=False))
        self.assertIn('missing preparation intent', self.release('--major', '--apply', success=False))
        receipt['schema_version'] = 1
        path.write_text(json.dumps(receipt))
        self.assertIn('no explicit MAJOR authorization', self.release('--apply', success=False))
        self.assertIn('mode conflict', self.release('--major', '--apply', success=False))

    def test_major_keeps_signature_message_dirty_and_stale_guards(self):
        self.baseline()
        self.change('not conventional')
        self.assertIn('Conventional', self.release('--major', '--apply', success=False))
        self.git(self.root, 'reset', '--hard', 'v0.1.0')
        self.change('fix: adjust')
        # Deliberately unsigned negative fixture; never touches real history.
        subprocess.run(['git', '-c', 'user.name=Conformance',
                        '-c', 'user.email=conformance@example.invalid', 'commit',
                        '--amend', '--no-gpg-sign', '-qm', 'fix: unsigned'],
                       cwd=self.root, env=self.git.environment, check=True)
        self.assertIn('lacks an OpenPGP signature', self.release('--major', '--apply', success=False))
        self.git(self.root, 'reset', '--hard', 'v0.1.0')
        self.write('unrelated', 'preserve')
        self.assertIn('clean working tree', self.release('--major', '--apply', success=False))
        (self.root / 'unrelated').unlink()
        self.release('--major', '--apply')
        self.write('package.json', '{}')
        self.assertIn('drift', self.release('--major', '--apply', success=False))

    def test_signed_zero_patch_feature_and_breaking_forms_at_high_minor(self):
        self.set_version('0.99.8')
        self.git(self.root, 'tag', 'v0.99.8')
        for message, expected in [
            ('fix: repair', '0.99.9'),
            ('feat: extend', '0.100.0'),
            ('refactor!: replace', '0.100.0'),
            ('refactor(api)!: replace again', '0.100.0'),
            ('fix: adapt\n\nBREAKING CHANGE: migrate callers', '0.100.0'),
            ('fix: adapt\n\nBREAKING-CHANGE: migrate again', '0.100.0'),
        ]:
            self.change(message)
            plan = self.release()
            self.assertEqual(plan['next'], expected)
            self.assertIn(self.git(self.root, 'rev-parse', 'HEAD'), plan['notes'])
        self.assertIn('migrate callers', plan['notes'])
        self.assertIn('migrate again', plan['notes'])

    def test_signed_zero_zero_patch_and_one_x_normal_semver(self):
        self.set_version('0.0.8')
        self.git(self.root, 'tag', 'v0.0.8')
        self.change('fix: repair')
        self.assertEqual(self.release()['next'], '0.0.9')
        self.change('feat!: replace')
        self.assertEqual(self.release()['next'], '0.1.0')
        self.release('--major', '--apply')
        self.commit('chore(release): prepare major')
        self.git(self.root, 'tag', 'v1.0.0')
        for message, expected in [('fix: repair', '1.0.1'), ('feat: add', '1.1.0'),
                                  ('feat!: replace', '2.0.0')]:
            self.change(message)
            self.assertEqual(self.release()['next'], expected)

    def test_bumps_and_maximum_once(self):
        self.baseline()
        cases = [('fix(parser): correct parsing', '0.1.1'),
                 ('feat(api): add operation', '0.2.0'),
                 ('refactor(api)!: remove operation', '0.2.0')]
        for message, version in cases:
            self.change(message)
            plan = self.release()
            self.assertEqual(plan['next'], version)
            self.assertIn(message, plan['notes'])
        plan = self.release('--apply')
        self.assertEqual(plan['next'], '0.2.0')
        self.assertEqual(plan['bump'], 'minor')
        self.assertEqual(json.loads((self.root / 'package.json').read_text())['version'], '0.2.0')
        lock = json.loads((self.root / 'package-lock.json').read_text())
        self.assertEqual(lock['version'], '0.2.0')
        self.assertEqual(lock['packages']['']['version'], '0.2.0')
        self.assertEqual(lock['packages']['node_modules/other']['version'], '9.0.0')
        metadata = json.loads(subprocess.check_output(
            ['cargo', 'metadata', '--locked', '--offline', '--no-deps', '--format-version=1'],
            cwd=self.root, env=self.git.environment))
        self.assertEqual(metadata['packages'][0]['version'], '0.2.0')
        self.assertTrue(self.release()['prepared'])

    def test_apply_build_derived_evidence_and_reapply_preserve_plan(self):
        self.write('.gitignore', '.craftguild/\ntarget/\n')
        self.write('build.rs', '''fn main() {
    let prepared = std::env::var("CRAFTGUILD_PREPARED").unwrap();
    let bom = std::path::Path::new(&prepared).parent().unwrap().join("bom.json");
    println!("cargo:rustc-env=FIXTURE_SBOM={}/{}", std::env::var("CARGO_MANIFEST_DIR").unwrap(), bom.display());
    println!("cargo:rerun-if-env-changed=CRAFTGUILD_PREPARED");
}
''')
        self.write('src/main.rs', '''use std::io::Write;
fn main() { std::io::stdout().write_all(include_bytes!(env!("FIXTURE_SBOM"))).unwrap(); }
''')
        self.write('evidence.py', '''import hashlib,json,pathlib
data = {"manifest": hashlib.sha256(pathlib.Path("Cargo.toml").read_bytes()).hexdigest()}
pathlib.Path(".craftguild/derived/release.json").write_text(json.dumps(data))
''')
        config = (self.root / 'craftguild.toml').read_text().replace('schema_version=1', 'schema_version=2', 1)
        config += '''
[artifacts.build]
[[artifacts.build.hooks]]
id="derived-ownership"
stage="evidence"
argv=["python3","evidence.py"]
inputs=["Cargo.toml","evidence.py"]
outputs=[".craftguild/derived/release.json"]
'''
        self.write('craftguild.toml', config)
        self.commit('chore: configure build provenance fixture')
        self.baseline()
        self.change('feat: add distribution capability')
        plan = self.release('--apply')
        self.assertEqual(plan['next'], '0.2.0')
        before = self.git(self.root, 'status', '--porcelain')
        notes = {p: p.read_bytes() for p in (self.root / 'docs/releases').iterdir()}
        build = subprocess.run([str(self.tool), 'build', '--profile', 'debug'],
                               cwd=self.root, env=self.git.environment,
                               capture_output=True, text=True)
        self.assertEqual(build.returncode, 0, build.stderr)
        receipt = json.loads((self.root / '.craftguild/demo/receipt.json').read_text())
        self.assertIn('.craftguild/derived/release.json', receipt['prepared']['inputs'])
        self.assertEqual(receipt['prepared']['build']['version'], '0.2.0')
        self.assertEqual(before, self.git(self.root, 'status', '--porcelain'))
        replay = self.release('--apply')
        self.assertTrue(replay['prepared'])
        self.assertEqual(replay['next'], '0.2.0')
        self.assertEqual(notes, {p: p.read_bytes() for p in (self.root / 'docs/releases').iterdir()})
        self.assertEqual(before, self.git(self.root, 'status', '--porcelain'))

    def test_breaking_footers_and_invalid_message(self):
        self.baseline()
        for token in ['BREAKING CHANGE', 'BREAKING-CHANGE']:
            self.change('fix: adapt contract\n\n' + token + ': callers must migrate')
            plan = self.release()
            self.assertEqual(plan['next'], '0.2.0')
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
        self.commit('chore(release): prepare UI feature')
        self.git(self.root, 'tag', 'ui-v2.1.0')
        self.write('ui/main.js', 'export const replacement = true;\n')
        self.change('feat!: replace shared contract')
        self.assertEqual(self.release('--group', 'demo')['next'], '0.2.0')
        self.assertEqual(self.release('--group', 'ui')['next'], '3.0.0')
        self.assertEqual(self.release('--group', 'ui', '--major')['next'], '3.0.0')


if __name__ == '__main__':
    unittest.main()
