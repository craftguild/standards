"""Explicit packaging acceptance against the real, already-built host engine."""

import copy
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'scripts'))
from package_entry import package as entry
from package_skill import package as skill
from public_files import inventory


def verify():
    original = json.loads((ROOT / '.craftguild/craftguild-standards/receipt.json').read_text())
    with tempfile.TemporaryDirectory(prefix='packaging-acceptance-', dir=ROOT / '.craftguild') as name:
        base = Path(name)
        root = base / 'source'
        for record in inventory(ROOT)['files']:
            destination = root / record['path']
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(ROOT / record['path'], destination)
        subprocess.run(['git', 'init', '--quiet', str(root)], check=True)
        receipt = copy.deepcopy(original)
        context = receipt['context']
        context['target_dir'] = str(root / 'target')
        binary = root / 'target' / context['target'] / context['profile'] / Path(context['output']).name
        binary.parent.mkdir(parents=True)
        shutil.copy2(original['context']['output'], binary)
        context['output'] = str(binary)
        context['outputs'] = [str(binary)]
        prepared = receipt['prepared']
        bom = root / prepared['sbom']
        bom.parent.mkdir(parents=True)
        shutil.copy2(ROOT / prepared['sbom'], bom)
        receipt_file = root / '.craftguild/craftguild-standards/receipt.json'
        receipt_file.write_text(json.dumps(receipt))
        entry(root, base / 'control-entry')
        skill(root, base / 'control-skill.tar.gz')
        assert {p['name'] for p in json.loads(bom.read_bytes())['metadata']['properties']} == {
            'craftguild:build', 'craftguild:features'}
        source = root / 'crates/craftguild-standards/src/execution_process.rs'
        addition = source.with_name('unsealed_packaging_addition.rs')
        serial = 0

        def rejected(label):
            nonlocal serial
            for package, suffix in [(entry, 'entry'), (skill, 'skill.tar.gz')]:
                serial += 1
                destination = base / f'{serial}-{suffix}'
                try:
                    package(root, destination)
                except (ValueError, OSError, subprocess.CalledProcessError):
                    pass
                else:
                    raise AssertionError('Packaging accepted ' + label)
                assert not destination.exists(), 'Rejected packaging published a delivery artifact'

        addition.write_text('pub fn unsealed() {}\n')
        rejected('added sealed-directory file')
        # Matching the directory list alone must not authorize an unsealed file.
        altered = copy.deepcopy(receipt)
        directory = 'crates/craftguild-standards/src'
        assert directory in altered['prepared']['inventories']
        altered['prepared']['inventories'][directory].append(str(addition.relative_to(root)))
        receipt_file.write_text(json.dumps(altered))
        rejected('inventory entry without a sealed hash')
        receipt_file.write_text(json.dumps(receipt))
        addition.unlink()
        before = source.read_bytes()
        source.write_bytes(before + b'// changed without rebuilding\n')
        rejected('changed source')
        source.unlink()
        rejected('missing source')
        source.write_bytes(before)
        before = bom.read_bytes()
        bom.write_bytes(b'{}')
        rejected('altered SBOM')
        bom.write_bytes(before)
        before = binary.read_bytes()
        binary.write_bytes(b'tampered engine')
        rejected('altered binary')
        binary.write_bytes(before)
        entry(root, base / 'restored-entry')
    print('PASS: real entry/skill packaging rejects added, unsealed, changed, missing, SBOM and binary inputs; unchanged control succeeds')


if __name__ == '__main__':
    verify()
