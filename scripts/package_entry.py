#!/usr/bin/env python3
"""Export a source-development entry bundle around an already verified engine."""

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

from public_files import inventory

PROTOCOL = 'craftguild-execution-1'


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def package(root, destination):
    if destination.exists():
        raise ValueError('Entry destination must not already exist')
    receipt = json.loads((root / '.craftguild/craftguild-standards/receipt.json').read_text())
    binary = Path(receipt['context']['output'])
    if (receipt['protocol'] != PROTOCOL
            or receipt['outputs'] != {binary.name: digest(binary)}):
        raise ValueError('Run make build before packaging the entry')
    # The hash-verified engine owns the complete seal contract, including exact
    # directory inventories and their linkage to the sealed file hashes.
    subprocess.run([str(binary), 'verify-receipt', '--artifact', receipt['context']['artifact']],
                   cwd=root, check=True)
    expected_bom = (root / receipt['prepared']['sbom']).read_bytes()
    actual_bom = subprocess.check_output([str(binary), 'sbom'])
    if actual_bom != expected_bom or hashlib.sha256(actual_bom).hexdigest() != receipt['prepared']['sbom_sha256']:
        raise ValueError('Engine SBOM differs from the verified receipt')
    if subprocess.check_output([str(binary), 'protocol']) != (PROTOCOL + '\n').encode():
        raise ValueError('Engine does not implement the entry protocol')
    source_inventory = inventory(root)
    inventory_digest = hashlib.sha256(json.dumps(source_inventory, sort_keys=True,
                                             separators=(',', ':')).encode()).hexdigest()
    local_inventory = root / '.craftguild/source-exports' / (inventory_digest + '.json')
    local_inventory.parent.mkdir(parents=True, exist_ok=True)
    local_inventory.write_text(json.dumps(source_inventory, indent=2) + '\n')
    source = {'schema_version': 2, 'kind': 'source-export',
              'inventory_sha256': inventory_digest}
    source_digest = hashlib.sha256(json.dumps(source, sort_keys=True,
                                             separators=(',', ':')).encode()).hexdigest()
    bundle = destination / 'bundle'
    (bundle / 'bin').mkdir(parents=True)
    (bundle / 'make').mkdir()
    shutil.copy2(binary, bundle / 'bin/craftguild-standards')
    shutil.copy2(root / 'templates/launcher.py', bundle / 'launcher.py')
    shutil.copy2(root / 'templates/make/common.mk', bundle / 'make/common.mk')
    (bundle / 'source.json').write_text(json.dumps(source, indent=2) + '\n')
    files = {path.relative_to(bundle).as_posix(): digest(path)
             for path in sorted(bundle.rglob('*')) if path.is_file()}
    metadata = {'schema_version': 1, 'protocol': PROTOCOL, 'engine': 'bin/craftguild-standards',
                'source': {'kind': 'source-export', 'sha256': source_digest},
                'build': receipt['prepared']['build'], 'files': files}
    (bundle / 'bundle.json').write_text(json.dumps(metadata, indent=2) + '\n')
    (destination / 'make').mkdir()
    shutil.copy2(bundle / 'launcher.py', destination / 'launcher.py')
    shutil.copy2(bundle / 'make/common.mk', destination / 'make/common.mk')
    descriptor = {'schema_version': 1, 'protocol': PROTOCOL, 'bundle': 'bundle',
                  'bundle_sha256': digest(bundle / 'bundle.json')}
    (destination / 'engine.json').write_text(json.dumps(descriptor, indent=2) + '\n')
    return descriptor


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    print(json.dumps(package(Path(__file__).resolve().parent.parent, args.output), indent=2))
