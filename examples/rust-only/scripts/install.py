#!/usr/bin/env python3
"""Install an already-built artifact; never execute build tools or infer a compiler."""

import argparse
import json
from pathlib import Path
import shutil
import sys


def install(root, bindir, destdir, target_dir, target):
    record = root / '.craftguild/sample/prepared.json'
    if not record.is_file():
        raise ValueError('No prepared build found. Run make build first.')
    info = json.loads(record.read_text())['build']
    selected = target or info['target']
    if selected != info['target']:
        raise ValueError('TARGET differs from the prepared build. Run make build TARGET=... first.')
    build_dir = Path(target_dir)
    if not build_dir.is_absolute():
        build_dir = root / build_dir
    binary = build_dir / selected / 'release/sample'
    if not binary.is_file():
        raise ValueError(f'Built executable missing: {binary}. Run make build first.')
    directory = Path(bindir)
    if not directory.is_absolute():
        raise ValueError('BINDIR must be absolute (DESTDIR provides a staging root).')
    if destdir:
        directory = Path(destdir) / directory.relative_to('/')
    directory.mkdir(parents=True, exist_ok=True)
    destination = directory / binary.name
    shutil.copyfile(binary, destination)
    destination.chmod(0o755)
    print(destination)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bindir', default='/usr/local/bin')
    parser.add_argument('--destdir', default='')
    parser.add_argument('--target-dir', default='target')
    parser.add_argument('--target', default='')
    args = parser.parse_args()
    try:
        install(Path(__file__).resolve().parent.parent, **vars(args))
    except (ValueError, OSError, KeyError) as error:
        print(f'Install failed: {error}', file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    sys.exit(main())
