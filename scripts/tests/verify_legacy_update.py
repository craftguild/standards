"""Upgrade the frozen flat reference through the real initialization entry point."""
import copy
import json
from pathlib import Path
import shutil

from entry_fixture import EntryFixture, ROOT, digest
from verify_execution import run
from verify_updates import snapshot


def verify(base, template, initializer):
    machine = copy.deepcopy(template.machine)
    machine.pop('command')  # Historical full CLI's identity grammar.
    machine['engine']['version'] = '0.0.1'
    fixture = EntryFixture(base / 'legacy-update', machine=machine)
    host = machine['target']
    stem = f'craftguild-standards-v0.0.1-{host}'
    binary = fixture.assets / stem
    bom = fixture.assets / (stem + '.bom.json')
    shutil.copy2(fixture.binary, binary)
    shutil.copy2(fixture.sbom, bom)
    prefix = 'https://github.com/craftguild/standards/releases/download/v0.0.1/'
    fixture.release['schema_version'] = 2
    fixture.release['hosts'][host] = {
        'binary': {'url': prefix + stem, 'sha256': digest(binary)},
        'sbom': {'url': prefix + stem + '.bom.json', 'sha256': digest(bom)}}
    old = ROOT / 'crates/craftguild-standards/tests/fixtures/initialization-v0.4.0'
    for source, destination in [('launcher.py', 'standards/launcher.py'),
                                ('common.mk', 'standards/make/common.mk'),
                                ('local-skill.md', '.agents/skills/craftguild-standardize/SKILL.md')]:
        path = fixture.root / destination
        path.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(old / source, path)
    agents = (template.root / 'AGENTS.md').read_text().replace(
        'Update explicitly with `craftguild-standards init [--force]`; follow its dependency revision guidance.\n', '')
    (fixture.root / 'AGENTS.md').write_text(agents + '\nLegacy product contract\n')
    fixture.pin['schema_version'] = 2
    fixture.pin.pop('managed')
    fixture.pin['entries'] = {name: digest(fixture.entry / name) for name in ['launcher.py', 'make/common.mk']}
    fixture.save()
    run(fixture.root, ['make', 'setup'], fixture.env)
    run(fixture.root, ['make', 'prime'], fixture.env)
    cache = fixture.cache
    before = snapshot(cache)
    for path in template.assets.iterdir():
        shutil.copy2(path, fixture.assets / path.name)
    result = run(fixture.root, [initializer, 'init'], fixture.env)
    assert b'upgrade: 0.0.1 @' in result.stdout
    selected = json.loads((fixture.entry / 'engine.json').read_bytes())
    assert selected['schema_version'] == 3 and selected['release']['identity'] == template.machine['engine']
    assert (fixture.root / 'AGENTS.md').read_text().endswith('Legacy product contract\n')
    assert snapshot(cache) == before
    run(fixture.root, ['make', 'prime'], fixture.env)
    assert 'require `--force`' in (fixture.root / '.agents/skills/craftguild-standardize/SKILL.md').read_text()
    print('PASS: real initializer upgrades known schema 2 flat pin/entry/Skill/AGENTS, retaining product contract and old valid cache', flush=True)
