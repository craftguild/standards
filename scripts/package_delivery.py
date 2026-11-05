#!/usr/bin/env python3
"""Package verified release-host assets without compiling or publishing."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile

from package_entry import package as package_entry
from package_skill import package as package_skill


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def verify_checkout(root, revision):
    if not (root / '.git').exists() or (root / '.git').is_symlink():
        raise ValueError('Delivery requires Git at its explicit source root')
    environment = {k: v for k, v in os.environ.items() if k not in {
        'GIT_DIR', 'GIT_WORK_TREE', 'GIT_COMMON_DIR', 'GIT_INDEX_FILE',
        'GIT_OBJECT_DIRECTORY', 'GIT_ALTERNATE_OBJECT_DIRECTORIES'}}
    command = ['git', '--no-optional-locks', '-c', 'core.fsmonitor=false',
               '--git-dir', str(root / '.git'), '--work-tree', str(root)]
    head = subprocess.check_output(command + ['rev-parse', 'HEAD'], env=environment).decode().strip()
    dirty = subprocess.check_output(command + ['status', '--porcelain=v1', '-z'], env=environment)
    if head != revision or dirty:
        raise ValueError('Delivery checkout must match the clean verified build revision')


def package(root, destination):
    receipt = json.loads((root / '.craftguild/craftguild-standards/receipt.json').read_text())
    build = receipt['prepared']['build']
    revision = build['commit']
    if (build['dirty'] != 'clean' or build['profile'] != 'release'
            or len(revision) != 40 or any(c not in '0123456789abcdef' for c in revision)
            or receipt['context']['host'] != build['target']):
        raise ValueError('Delivery requires a clean full-revision release build for this host')
    verify_checkout(root, revision)
    if destination.exists() or destination.is_symlink():
        raise ValueError('Delivery destination must not already exist')
    destination.parent.mkdir(parents=True, exist_ok=True)
    version, target = build['version'], build['target']
    suffix = f'v{version}-{target}'
    with tempfile.TemporaryDirectory(prefix='delivery-', dir=destination.parent) as name:
        temporary = Path(name)
        artifacts = temporary / 'artifacts'
        artifacts.mkdir()
        entry = temporary / 'craftguild-execution'
        descriptor = package_entry(root, entry)
        with tarfile.open(artifacts / f'craftguild-execution-{suffix}.tar.gz', 'w:gz') as archive:
            archive.add(entry, arcname='craftguild-execution')
        package_skill(root, artifacts / f'craftguild-standardize-{suffix}.tar.gz')
        shutil.copy2(receipt['context']['output'], artifacts / f'craftguild-standards-{suffix}')
        shutil.copy2(root / receipt['prepared']['sbom'], artifacts / f'craftguild-standards-{suffix}.bom.json')
        record = {
            'schema_version': 1, 'version': version, 'target': target, 'revision': revision,
            'protocol': receipt['protocol'], 'entry_bundle_sha256': descriptor['bundle_sha256'],
            'assets': {p.name: digest(p) for p in sorted(artifacts.iterdir())},
        }
        (artifacts / f'craftguild-release-{suffix}.json').write_text(json.dumps(record, indent=2) + '\n')
        checksums = ''.join(f'{digest(p)}  {p.name}\n' for p in sorted(artifacts.iterdir()))
        (artifacts / f'SHA256SUMS-{target}.txt').write_text(checksums)
        artifacts.rename(destination)
    print(destination)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    package(Path(__file__).resolve().parent.parent, args.output.resolve())
