#!/usr/bin/env python3
"""Verify a pinned local entry bundle and dispatch through its matching engine."""

import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import shutil
import subprocess
import sys
import tempfile

PROTOCOL = 'craftguild-execution-1'
INSTRUCTION = 'Run make setup and make build without privileges first.'


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def owned(root, name):
    relative = PurePosixPath(name)
    if (not name or relative.is_absolute() or '..' in relative.parts
            or str(relative) != name or '\\' in name or ':' in name):
        raise ValueError('Unsafe entry path: ' + name)
    path = root
    for part in relative.parts:
        path = path / part
        if path.is_symlink():
            raise ValueError('Symlink in entry bundle: ' + name)
    return path


def manifest(directory, expected):
    path = owned(directory, 'bundle.json')
    if digest(path) != expected:
        raise ValueError('Entry bundle manifest digest differs from the fixed descriptor')
    data = json.loads(path.read_text())
    if (data.get('schema_version') != 1 or data.get('protocol') != PROTOCOL
            or data.get('engine') != 'bin/craftguild-standards'
            or not data.get('files') or data.get('source', {}).get('kind') != 'source-export'):
        raise ValueError('Incompatible entry bundle or unsupported source provenance')
    actual = {path.relative_to(directory).as_posix() for path in directory.rglob('*')
              if path.is_file() or path.is_symlink()}
    if actual != set(data['files']) | {'bundle.json'}:
        raise ValueError('Entry bundle inventory differs from its manifest')
    for name, expected_hash in data['files'].items():
        path = owned(directory, name)
        if not path.is_file() or digest(path) != expected_hash:
            raise ValueError('Entry bundle file digest differs: ' + name)
    source = json.loads(owned(directory, 'source.json').read_text())
    if (set(source) != {'schema_version', 'kind', 'inventory_sha256'}
            or source['schema_version'] != 2 or source['kind'] != 'source-export'
            or len(source['inventory_sha256']) != 64
            or any(c not in '0123456789abcdef' for c in source['inventory_sha256'])):
        raise ValueError('Unsupported source-export provenance; do not bundle a source ledger')
    source_digest = hashlib.sha256(json.dumps(source, sort_keys=True,
                                             separators=(',', ':')).encode()).hexdigest()
    if data['source'].get('sha256') != source_digest:
        raise ValueError('Source-export provenance digest differs')
    return data


def engine(root, entry, operation):
    descriptor = json.loads(owned(entry, 'engine.json').read_text())
    if descriptor.get('schema_version') != 1 or descriptor.get('protocol') != PROTOCOL:
        raise ValueError('Incompatible entry descriptor')
    expected = descriptor.get('bundle_sha256', '')
    if len(expected) != 64 or any(c not in '0123456789abcdef' for c in expected):
        raise ValueError('Entry descriptor requires an exact SHA-256 bundle digest')
    cache = owned(root, '.craftguild/engine/' + expected)
    if not cache.exists():
        if operation == 'install':
            raise ValueError('Matching engine cache is missing. ' + INSTRUCTION)
        source = owned(entry, descriptor['bundle'])
        manifest(source, expected)
        cache.parent.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix='entry-', dir=cache.parent) as temporary:
            candidate = Path(temporary) / 'bundle'
            shutil.copytree(source, candidate, symlinks=False)
            manifest(candidate, expected)
            try:
                candidate.rename(cache)
            except FileExistsError:
                manifest(cache, expected)
    data = manifest(cache, expected)
    for name in ['launcher.py', 'make/common.mk']:
        if digest(owned(entry, name)) != data['files'][name]:
            raise ValueError('Entry files and engine bundle are from different exports')
    binary = owned(cache, data['engine'])
    result = subprocess.run([str(binary), 'protocol'], cwd=root, capture_output=True,
                            check=True, timeout=10)
    if result.stdout != (PROTOCOL + '\n').encode() or result.stderr:
        raise ValueError('Engine protocol differs from the entry bundle')
    return binary


def main():
    if len(sys.argv) < 2:
        raise ValueError('Usage: launcher.py setup|build|prepare|install|test|check|acceptance|release')
    operation = sys.argv[1]
    if operation not in {'setup', 'build', 'prepare', 'install', 'test', 'check', 'acceptance', 'release',
                         'banner', 'assess', 'verify', 'verify-frontend', 'verify-receipt', 'validate-sbom',
                         'check-release-config', 'measure', 'measure-conditional'}:
        raise ValueError('Unknown entry operation: ' + operation)
    root = Path.cwd().resolve()
    entry = Path(__file__).resolve().parent
    binary = engine(root, entry, operation)
    if operation == 'setup':
        if len(sys.argv) != 2:
            raise ValueError('Usage: launcher.py setup')
        print(binary)
    else:
        os.execv(binary, [str(binary), *sys.argv[1:]])


if __name__ == '__main__':
    try:
        main()
    except (ValueError, OSError, KeyError, subprocess.SubprocessError) as error:
        print(f'Craftguild entry failed: {error}. {INSTRUCTION}', file=sys.stderr)
        sys.exit(1)
