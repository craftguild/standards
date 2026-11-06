"""Flat delivery retains complete producer seal/inventory/SBOM/output checks."""

import copy
import hashlib
import json
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'scripts'))
from package_delivery import package
from release_metadata import merge


def verify(root, destination):
    receipt_file = root / '.craftguild/craftguild-standards-engine/receipt.json'
    original = json.loads(receipt_file.read_bytes())
    binary = Path(original['context']['output'])
    bom = root / original['prepared']['sbom']
    package(root, destination / 'host')
    files = {p.name: p.read_bytes() for p in (destination / 'host').iterdir()}
    target = original['context']['target']
    version = original['prepared']['build']['version']
    stem = f'craftguild-standards-engine-v{version}-{target}'
    initializer = f'craftguild-standards-v{version}-{target}'
    assert set(files) == {stem, stem + '.bom.json', initializer, initializer + '.bom.json', f'craftguild-release-v{version}-{target}.json',
                          f'SHA256SUMS-{target}.txt'}
    checksums = ''.join(f'{hashlib.sha256(files[name]).hexdigest()}  {name}\n'
                        for name in sorted(files) if not name.startswith('SHA256SUMS'))
    assert files[f'SHA256SUMS-{target}.txt'].decode() == checksums
    assert files[stem] == binary.read_bytes() and files[stem + '.bom.json'] == bom.read_bytes()
    assert {p['name'] for p in json.loads(bom.read_bytes())['metadata']['properties']} == {
        'craftguild:build', 'craftguild:features'}
    merged = merge(destination / 'host', destination / 'index')
    assert merged['identity']['revision'] == original['prepared']['build']['commit']
    assert set(merged['hosts']) == {target}
    source = root / 'crates/craftguild-standards/src/execution_process.rs'
    addition = source.with_name('unsealed_packaging_addition.rs')
    with (root / '.git/info/exclude').open('a') as exclude:
        exclude.write('\nunsealed_packaging_addition.rs\n')
    serial = 0

    def rejected(label):
        nonlocal serial
        serial += 1
        output = destination / ('rejected-' + str(serial))
        try:
            package(root, output)
        except (ValueError, OSError, subprocess.CalledProcessError):
            pass
        else:
            raise AssertionError('Delivery accepted ' + label)
        assert not output.exists(), 'Rejected delivery published artifacts'

    addition.write_text('pub fn unsealed() {}\n')
    rejected('ignored added sealed-directory input')
    altered = copy.deepcopy(original)
    altered['prepared']['inventories']['crates/craftguild-standards/src'].append(str(addition.relative_to(root)))
    receipt_file.write_text(json.dumps(altered))
    rejected('inventory entry without sealed-file hash')
    receipt_file.write_text(json.dumps(original))
    addition.unlink()
    # Keep Git identity clean to exercise sealed input validation independently.
    subprocess.run(['git', 'update-index', '--assume-unchanged', str(source.relative_to(root))], cwd=root, check=True)
    before = source.read_bytes()
    try:
        source.write_bytes(before + b'// changed sealed source\n')
        rejected('changed sealed source')
        source.unlink()
        rejected('missing sealed source')
    finally:
        source.write_bytes(before)
        subprocess.run(['git', 'update-index', '--no-assume-unchanged', str(source.relative_to(root))], cwd=root, check=True)
    for path, label in [(bom, 'altered SBOM'), (binary, 'altered binary')]:
        before = path.read_bytes()
        try:
            path.write_bytes(b'tampered')
            rejected(label)
        finally:
            path.write_bytes(before)
    package(root, destination / 'restored')
    assert files == {p.name: p.read_bytes() for p in (destination / 'restored').iterdir()}
    print('PASS: flat delivery exact inventory/checksums and unchanged control; added/unsealed/changed/missing/SBOM/binary refusal')
    return destination / 'index' / f'craftguild-release-v{version}.json'


if __name__ == '__main__':
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('root', type=Path)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    verify(args.root.resolve(), args.output.resolve())
