#!/usr/bin/env python3
"""Observe actual rebuilds in disposable application Git roots and linked worktrees."""

import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile


def git(root, *args):
    return subprocess.check_output(['git', '-c', 'commit.gpgsign=false',
                                    '-c', 'user.name=Conformance',
                                    '-c', 'user.email=conformance@example.invalid',
                                    *args], cwd=root, text=True).strip()


def verify(root, tool, env):
    with tempfile.TemporaryDirectory(prefix='craftguild-identity-') as temporary:
        parent = Path(temporary)
        source = parent / 'application'
        shutil.copytree(root / 'examples/rust-only', source,
                        ignore=shutil.ignore_patterns('target', '.craftguild', '.git'))
        with (source / '.gitignore').open('a') as stream:
            stream.write('ignored-scratch\n')
        git(source, 'init', '-b', 'main')
        git(source, 'add', '.')
        git(source, 'commit', '-m', 'initial')
        target = json.loads((root / '.craftguild/craftguild-standards/prepared.json').read_text())['build']['target']
        missing = subprocess.run(['cargo', 'build', '--locked', '--release', '--target', target],
                                 cwd=source, env=env, capture_output=True)
        assert missing.returncode and b'Run `make build`' in missing.stderr

        def build(directory):
            subprocess.run([str(tool), 'build'], cwd=directory, env=env, check=True)
            prepared = json.loads((directory / '.craftguild/sample/prepared.json').read_text())
            binary = Path(env['CARGO_TARGET_DIR']) / prepared['build']['target'] / 'release/sample'
            facts = json.loads(subprocess.check_output([str(binary), '--machine-version']))
            assert facts == prepared['build']
            assert facts['command'] == 'sample' and facts['version'] == '0.1.0'
            assert facts['compiler'].startswith('rustc ')
            assert facts['profile'] == 'release'
            return facts

        clean = build(source)
        assert clean['commit'] == git(source, 'rev-parse', 'HEAD')
        assert clean['dirty'] == 'clean'
        (source / 'ignored-scratch').write_text('ignored')
        assert build(source) == clean
        (source / 'untracked').write_text('dirty')
        prepared = json.loads((source / '.craftguild/sample/prepared.json').read_text())
        direct = subprocess.run(['cargo', 'build', '--locked', '--release', '--target',
                                 prepared['build']['target']], cwd=source, env=env,
                                capture_output=True)
        assert direct.returncode and b'Run `make build`' in direct.stderr
        dirty = build(source)
        assert dirty['commit'] == clean['commit'] and dirty['dirty'] == 'dirty'
        git(source, 'add', 'untracked')
        git(source, 'commit', '-m', 'second')
        next_commit = build(source)
        assert next_commit['commit'] != clean['commit'] and next_commit['dirty'] == 'clean'
        linked = parent / 'linked'
        git(source, 'worktree', 'add', '-b', 'linked', str(linked))
        assert build(linked) == next_commit
        archive = source / 'archive'
        shutil.copytree(root / 'examples/rust-only', archive,
                        ignore=shutil.ignore_patterns('target', '.craftguild', '.git'))
        archived = build(archive)
        assert archived['commit'] == 'unknown' and archived['dirty'] == 'unknown'
    print('PASS: actual clean/dirty/commit rebuilds, stale direct build refusal, worktree and archive identity')
