"""Real initializer/engine transitions against isolated, transport-stubbed releases."""
import copy
import json
from pathlib import Path
import shutil

from entry_fixture import EntryFixture, digest
from verify_execution import run


def snapshot(root):
    return {str(p.relative_to(root)): p.read_bytes() for p in root.rglob('*') if p.is_file()}


def seed(base, template, machine):
    fixture = EntryFixture(base, machine=machine)
    reference = json.loads((template.entry / 'engine.json').read_bytes())
    fixture.pin['managed'] = reference['managed']
    for name in ['AGENTS.md', '.agents/skills/craftguild-standardize/SKILL.md']:
        target = fixture.root / name
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(template.root / name, target)
    fixture.save()
    # Prove the previous engine is usable before attempting replacement.
    run(fixture.root, ['make', 'setup'], fixture.env)
    run(fixture.root, ['make', 'prime'], fixture.env)
    old_cache = snapshot(fixture.cache)
    for path in template.assets.iterdir():
        shutil.copy2(path, fixture.assets / path.name)
    for name in ['Cargo.toml', 'Cargo.lock', 'package.json', 'package-lock.json', 'craftguild.toml']:
        (fixture.root / name).write_text('consumer-owned inputs\n')
    with (fixture.root / 'AGENTS.md').open('a') as output:
        output.write('\nProduct contracts\n')
    return fixture, old_cache


def verify(base, template, initializer, engine):
    current = template.machine
    for tool in [initializer, engine]:
        for force in [False, True]:
            fixture = EntryFixture(base / ('fresh-' + Path(tool).name + '-' + str(force)))
            shutil.rmtree(fixture.root)
            fixture.root.mkdir()
            for path in template.assets.iterdir():
                shutil.copy2(path, fixture.assets / path.name)
            result = run(fixture.root, [tool, 'init'] + (['--force'] if force else []), fixture.env)
            assert b'initialize: absent ->' in result.stdout
            assert len(fixture.calls()) == 3
            assert json.loads((fixture.entry / 'engine.json').read_bytes())['release']['identity'] == current['engine']
    cases = [('upgrade', '0.0.1', current['engine']['revision']),
             ('downgrade', '99.0.0', current['engine']['revision']),
             ('same-version replacement', current['engine']['version'], 'b' * 40)]
    serial = 0
    for tool in [initializer, engine]:
        for label, version, revision in cases:
            for force in [False, True]:
                serial += 1
                machine = copy.deepcopy(current)
                machine['engine'].update(version=version, revision=revision)
                fixture, old_cache = seed(base / f'update-{serial}', template, machine)
                old_pin = (fixture.entry / 'engine.json').read_bytes()
                old_cache_path = fixture.cache
                before = snapshot(fixture.root)
                calls = len(fixture.calls())
                argv = [tool, 'init'] + (['--force'] if force else [])
                allowed = label == 'upgrade' or force
                result = run(fixture.root, argv, fixture.env, success=allowed)
                if not allowed:
                    assert b'init --force' in result.stderr
                    assert version.encode() in result.stderr and revision.encode() in result.stderr
                    assert len(fixture.calls()) == calls and snapshot(fixture.root) == before
                else:
                    assert label.encode() in result.stdout
                    assert version.encode() in result.stdout and revision.encode() in result.stdout
                    selected = json.loads((fixture.entry / 'engine.json').read_bytes())
                    assert selected['release']['identity'] == current['engine']
                    assert len(fixture.calls()) == calls + 3
                    run(fixture.root, ['make', 'prime'], fixture.env)
                    for name in ['Cargo.toml', 'Cargo.lock', 'package.json', 'package-lock.json', 'craftguild.toml', 'Makefile']:
                        assert (fixture.root / name).read_bytes() == before[name]
                    assert (fixture.root / 'AGENTS.md').read_text().endswith('Product contracts\n')
                assert snapshot(old_cache_path) == old_cache
                assert old_pin == before['standards/engine.json']

    # Exact identity uses recorded metadata and restores missing cache without index lookup.
    for tool in [initializer, engine]:
        root = base / ('exact-' + Path(tool).name)
        shutil.copytree(template.root, root, ignore=shutil.ignore_patterns('.craftguild'))
        before = snapshot(root)
        calls = len(template.calls())
        run(root, [tool, 'init', '--force'], template.env)
        assert len(template.calls()) == calls + 2
        assert before == {k: v for k, v in snapshot(root).items() if not k.startswith('.craftguild/')}
        times = {str(p): p.stat().st_mtime_ns for p in root.rglob('*') if p.is_file()}
        run(root, [tool, 'init'], template.env)
        assert times == {str(p): p.stat().st_mtime_ns for p in root.rglob('*') if p.is_file()}
        assert len(template.calls()) == calls + 2
        cache = root / '.craftguild/engine'
        shutil.rmtree(cache)
        run(root, [tool, 'init'], template.env)
        assert len(template.calls()) == calls + 4

    # Force cannot bypass transport, integrity, artifact role or ownership checks.
    failures = ['transport', 'hash', 'revision', 'sbom', 'role', 'protocol', 'path', 'edited', 'pin-protocol', 'pin-schema', 'host']
    for failure in failures:
        machine = copy.deepcopy(current)
        machine['engine']['version'] = '99.0.0'
        fixture, old_cache = seed(base / ('failure-' + failure), template, machine)
        old_cache_path = fixture.cache
        index = fixture.assets / f'craftguild-release-v{current["engine"]["version"]}.json'
        metadata = json.loads(index.read_bytes())
        host = metadata['hosts'][current['target']]
        if failure == 'transport':
            fixture.env['STUB_FAIL'] = 'yes'
        elif failure == 'hash':
            (fixture.assets / host['engine']['binary']['url'].rsplit('/', 1)[-1]).write_bytes(b'tampered')
        elif failure in ['revision', 'protocol']:
            metadata['identity'][failure] = 'c' * 40 if failure == 'revision' else 'future'
        elif failure == 'role':
            host['engine'] = host['initializer']
        elif failure == 'sbom':
            sidecar = fixture.assets / host['engine']['sbom']['url'].rsplit('/', 1)[-1]
            sidecar.write_bytes(b'another artifact SBOM')
            host['engine']['sbom']['sha256'] = digest(sidecar)
        elif failure == 'path':
            (fixture.root / '.agents/skills/craftguild-standardize/SKILL.md').unlink()
            (fixture.root / '.agents/skills/craftguild-standardize/SKILL.md').symlink_to(index)
        elif failure in ['pin-protocol', 'pin-schema', 'host']:
            if failure == 'pin-protocol':
                fixture.pin['release']['identity']['protocol'] = 'future'
            elif failure == 'pin-schema':
                fixture.pin['schema_version'] = 99
            else:
                metadata['hosts'] = {}
            fixture.save()
        elif failure == 'edited':
            (fixture.root / '.agents/skills/craftguild-standardize/SKILL.md').write_text('consumer edit')
        index.write_text(json.dumps(metadata))
        before = snapshot(fixture.root)
        if failure == 'edited':
            calls = len(fixture.calls())
            result = run(fixture.root, [initializer, 'init'], fixture.env, success=False)
            assert b'Conflicting managed file' in result.stderr and b'Use init --force' not in result.stderr
            assert len(fixture.calls()) == calls
        run(fixture.root, [initializer, 'init', '--force'], fixture.env, success=False)
        assert before == snapshot(fixture.root)
        assert snapshot(old_cache_path) == old_cache
        if failure not in ['path', 'edited', 'pin-protocol', 'pin-schema']:
            run(fixture.root, ['make', 'prime'], fixture.env)
    print('PASS: both real entry points, normal/force transitions, exact cache restoration/no-op, force integrity/ownership refusals and old usable cache')
