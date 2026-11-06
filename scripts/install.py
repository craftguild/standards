#!/usr/bin/env python3
"""Dispatch copy-only installation through an already-built verified engine."""

import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys


def install(root, bindir, destdir, target_dir, target, profile=None):
    root = root.resolve()
    record = root / '.craftguild/craftguild-standards-engine/receipt.json'
    if not record.is_file():
        raise ValueError('No verified engine found. Run make build without privileges first.')
    receipt = json.loads(record.read_text())
    binary = Path(receipt['context']['output'])
    if not binary.is_file() or binary.is_symlink():
        raise ValueError('Verified engine missing. Run make build without privileges first.')
    digest = hashlib.sha256(binary.read_bytes()).hexdigest()
    if receipt.get('protocol') != 'craftguild-execution-1' or receipt['outputs'] != {binary.name: digest}:
        raise ValueError('Engine integrity check failed. Run make build without privileges first.')
    argv = [str(binary), 'install', '--artifact', 'standards', '--bindir', bindir, '--destdir', destdir]
    for flag, value in [('target-dir', target_dir), ('profile', profile), ('target', target)]:
        if value:
            argv += ['--' + flag, value]
    subprocess.run(argv, cwd=root, check=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bindir', default='/usr/local/bin')
    parser.add_argument('--destdir', default='')
    parser.add_argument('--target-dir')
    parser.add_argument('--target', default='')
    parser.add_argument('--profile')
    args = parser.parse_args()
    try:
        install(Path(__file__).resolve().parent.parent, **vars(args))
    except (ValueError, OSError, KeyError, subprocess.SubprocessError) as error:
        print(f'Install failed: {error}', file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())
