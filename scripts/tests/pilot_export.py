"""Explicit read-only reference export for optional product pilots, never a test prerequisite."""

import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import shutil
import subprocess

PRIVATE = {'.git', '.private', '.craftguild', '.beads', '.agents', '.codex', '.cursor',
           'node_modules', 'target', '__pycache__', '.DS_Store'}


def git(source, *args):
    environment = {key: value for key, value in os.environ.items()
                   if key not in {'GIT_DIR', 'GIT_WORK_TREE', 'GIT_COMMON_DIR', 'GIT_INDEX_FILE',
                                  'GIT_OBJECT_DIRECTORY', 'GIT_ALTERNATE_OBJECT_DIRECTORIES'}}
    return subprocess.check_output(['git', '--no-optional-locks', '-c', 'core.fsmonitor=false',
                                    '--git-dir', str(source / '.git'), '--work-tree', str(source),
                                    *args], cwd=source, env=environment)


def capture(source, output):
    source = source.resolve()
    if output.exists():
        raise ValueError('Export destination must be new')
    output.mkdir(parents=True)
    names = git(source, 'ls-files', '--cached', '--others', '--exclude-standard', '-z')
    files = {}
    omitted = []
    for name in sorted({item.decode() for item in names.split(b'\0') if item}):
        relative = PurePosixPath(name)
        if relative.is_absolute() or '..' in relative.parts or str(relative) != name:
            raise ValueError('Unsafe export path')
        if set(relative.parts) & PRIVATE:
            continue
        path = source / name
        if any(p.is_symlink() for p in [path, *path.parents] if p != source):
            omitted.append(name)
            continue
        if not path.exists():
            continue
        if not path.is_file():
            raise ValueError('Nonregular reference source: ' + name)
        destination = output / name
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(path, destination)
        files[name] = hashlib.sha256(path.read_bytes()).hexdigest()
    status = git(source, 'status', '--porcelain=v1', '-z', '--untracked-files=all')
    return {'schema_version': 1, 'kind': 'source-export',
            'source_revision': git(source, 'rev-parse', 'HEAD').decode().strip(),
            'status_sha256': hashlib.sha256(status).hexdigest(),
            'files': files, 'omitted_symlinks': omitted}


def verify(source, record):
    source = source.resolve()
    status = git(source, 'status', '--porcelain=v1', '-z', '--untracked-files=all')
    if hashlib.sha256(status).hexdigest() != record['status_sha256']:
        raise ValueError('Reference working-tree status changed')
    for name, expected in record['files'].items():
        if hashlib.sha256((source / name).read_bytes()).hexdigest() != expected:
            raise ValueError('Reference source changed: ' + name)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--record', type=Path, required=True)
    parser.add_argument('--verify', action='store_true')
    args = parser.parse_args()
    if args.verify:
        verify(args.source, json.loads(args.record.read_text()))
        print('PASS: source reference bytes and status unchanged')
    else:
        if args.output is None:
            parser.error('--output is required for source export')
        record = capture(args.source, args.output)
        args.record.write_text(json.dumps(record, indent=2) + '\n')
        print(f'Exported {len(record["files"])} source files; omitted {len(record["omitted_symlinks"])} symlinks')
