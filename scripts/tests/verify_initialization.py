"""Qualify flat delivery, real init/prime and standalone consumers on this host."""

import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

from entry_fixture import EntryFixture, ROOT
from signing import signed_git
from verify_execution import run, verify as execution
from verify_installers import verify as installers
from verify_packaging import verify as packaging
from verify_updates import verify as updates
from verify_legacy_update import verify as legacy_update
sys.path.insert(0, str(ROOT / 'scripts'))
from public_files import inventory, export


def snapshot(root):
    return {str(p.relative_to(root)): p.read_bytes() for p in root.rglob('*') if p.is_file()}


def prime(fixture, tool):
    denied = fixture.base / 'offline-tools'
    denied.mkdir(exist_ok=True)
    (denied / 'python3').symlink_to(sys.executable)
    marker = fixture.base / 'forbidden-tool'
    for name in ['curl', 'cargo', 'rustc', 'rustup', 'npm', 'node', 'git']:
        path = denied / name
        path.write_text('#!' + sys.executable + '\nfrom pathlib import Path\n'
                        'Path(' + repr(str(marker)) + ').touch()\nraise SystemExit(97)\n')
        path.chmod(0o755)
    environment = dict(fixture.env, PATH=str(denied))
    for contents in [None, 'invalid [', 'schema_version=2\nstandards_version="0.1.0"\nartifacts=[]\n']:
        config = fixture.root / 'craftguild.toml'
        if contents is None:
            config.unlink(missing_ok=True)
        else:
            config.write_text(contents)
        before = snapshot(fixture.root)
        result = run(fixture.root, [tool, 'prime'], environment)
        assert b'Not ready' in result.stdout and b'conformance success' in result.stdout
        assert fixture.machine['engine']['revision'].encode() in result.stdout
        assert b'\x1b' not in result.stdout
        assert before == snapshot(fixture.root)
    result = run(fixture.root, [tool, 'prime', '--topic', 'configuration'], environment)
    assert b'# Configuration' in result.stdout
    assert b'blob/' + fixture.machine['engine']['revision'].encode() in result.stdout
    result = run(fixture.root, [tool, 'prime', '--topic', 'unknown'], environment, success=False)
    assert b'Available:' in result.stderr
    assert not marker.exists(), 'Prime invoked network/compiler/product code'


def change_fixture_version(root, before, after):
    manifest = root / 'Cargo.toml'
    manifest.write_text(manifest.read_text().replace('version = "' + before + '"', 'version = "' + after + '"', 1))
    lockfile = root / 'Cargo.lock'
    blocks = lockfile.read_text().split('[[package]]')
    owned = ['craftguild-cli', 'craftguild-cli-build', 'craftguild-core', 'craftguild-standards', 'craftguild-initializer']
    for index, block in enumerate(blocks):
        if any(block.lstrip().startswith('name = "' + name + '"\n') for name in owned):
            blocks[index] = block.replace('version = "' + before + '"', 'version = "' + after + '"', 1)
    lockfile.write_text('[[package]]'.join(blocks))
    for name in ['package.json', 'package-lock.json']:
        path = root / name
        data = json.loads(path.read_text())
        data['version'] = after
        if name == 'package-lock.json':
            data['packages']['']['version'] = after
        path.write_text(json.dumps(data, indent=2) + '\n')


def verify():
    control = ROOT / '.craftguild'
    control.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='init-qualification-', dir=control) as name:
        base = Path(name).resolve()
        source = export(ROOT, base / 'source', inventory(ROOT))
        with signed_git() as git:
            git(source, 'init', '-q', '-b', 'main')
            git(source, 'add', '.')
            git(source, 'commit', '-S', '-qm', 'feat: isolated initialization qualification')
            environment = dict(git.environment)
            home = base / 'build-home'
            home.mkdir()
            environment.update(HOME=str(home),
                               RUSTUP_HOME=os.environ.get('RUSTUP_HOME', str(Path.home() / '.rustup')),
                               CARGO_HOME=os.environ.get('CARGO_HOME', str(Path.home() / '.cargo')))
            run(source, ['make', 'build'], environment)
            record = json.loads((source / '.craftguild/craftguild-standards-engine/receipt.json').read_bytes())
            tool = Path(record['context']['output'])
            machine = json.loads(run(source, [tool, 'engine-identity'], environment).stdout)
            assert machine['clean'] is True
            initializer_record = json.loads((source / '.craftguild/craftguild-standards/receipt.json').read_bytes())
            initializer = Path(initializer_record['context']['output'])
            distribution = base / 'deliveries'
            distribution.mkdir()
            index = packaging(source, distribution)
            fixture = EntryFixture(base / 'initialization', binary=tool,
                                   sbom=(source / record['prepared']['sbom']).read_bytes(), machine=machine)
            shutil.copy2(index, fixture.assets / index.name)
            shutil.rmtree(fixture.root)
            fixture.root.mkdir()
            # A parent checkout is not discovered or initialized as the consumer.
            (fixture.base / '.git').mkdir()
            result = run(fixture.root, [initializer, 'init'], fixture.env)
            assert b'Initialized' in result.stdout
            assert not (fixture.root / '.git').exists()
            assert (fixture.root / '.agents/skills/craftguild-standardize/SKILL.md').is_file()
            assert (fixture.root / '.gitignore').read_text().count('/.craftguild/') == 1
            assert len(fixture.calls()) == 3  # index plus binary/SBOM; no mutable lookup.
            before = snapshot(fixture.root)
            times = {str(p): p.stat().st_mtime_ns for p in fixture.root.rglob('*') if p.is_file()}
            run(fixture.root, [initializer, 'init'], fixture.env)
            assert before == snapshot(fixture.root)
            assert times == {str(p): p.stat().st_mtime_ns for p in fixture.root.rglob('*') if p.is_file()}
            assert len(fixture.calls()) == 3
            # Clone equivalent: committed entry/config only, no global initializer/cache.
            clone = base / 'clone'
            shutil.copytree(fixture.root, clone, ignore=shutil.ignore_patterns('.craftguild'))
            calls = len(fixture.calls())
            run(clone, ['make', 'prime'], fixture.env, success=False)
            assert len(fixture.calls()) == calls
            run(clone, ['make', 'setup'], fixture.env)
            run(clone, ['make', 'prime'], fixture.env)
            assert len(fixture.calls()) == calls + 2
            prime(fixture, tool)
            run(fixture.root, ['make', 'check'], fixture.env, success=False)
            assert len(fixture.calls()) == 5
            # Standalone consumers commit only a pin and thin entry, with no rules tree.
            assert not (fixture.root / 'spec').exists()
            assert not list(fixture.root.rglob('*.tar.gz'))
            execution(fixture.entry, fixture.env)
            installers(fixture.entry, fixture.env)
            requests = len(fixture.calls())
            updates(base, fixture, initializer, tool)
            legacy_update(base, fixture, initializer)
            # Dirty unpublished builds may not replace even an existing pin.
            change_fixture_version(source, machine['engine']['version'], '9.9.9')
            run(source, ['make', 'build'], environment)
            other = json.loads(run(source, [tool, 'engine-identity'], environment).stdout)
            assert other['engine']['version'] == '9.9.9' and other['clean'] is False
            before = snapshot(fixture.root)
            run(fixture.root, [initializer, 'init'], fixture.env, success=False)
            assert before == snapshot(fixture.root)
            uninitialized = base / 'unpublished'
            uninitialized.mkdir()
            result = run(uninitialized, [initializer, 'init'], fixture.env, success=False)
            assert b'clean published engine' in result.stderr and not list(uninitialized.iterdir())
            requests = len(fixture.calls())
            cached = fixture.cache / 'craftguild-standards-engine'
            before = cached.read_bytes()
            cached.write_bytes(b'tampered')
            run(fixture.root, [tool, 'prime'], fixture.env, success=False)
            cached.write_bytes(before)
            cached.rename(cached.with_suffix('.missing'))
            run(fixture.root, [tool, 'prime'], fixture.env, success=False)
            assert len(fixture.calls()) == requests
            print(json.dumps({'host': machine['target'], 'fixture_engine': machine['engine'],
                              'other_installed_version': other['engine']['version'],
                              'transport_requests': len(fixture.calls())}), flush=True)
    print('PASS: signed source fixture, flat delivery, real init/retry/prime, differing installed tool, offline/no-write/no-product-code, standalone consumers and temporary installer acceptance')


if __name__ == '__main__':
    verify()
