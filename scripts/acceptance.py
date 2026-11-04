#!/usr/bin/env python3
"""Exercise an already built binary and staged install in isolated temporary roots."""

import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile


def run(binary, cwd, args, env):
    result = subprocess.run([str(binary), *args], cwd=cwd, env=env,
                            capture_output=True)
    if result.returncode or result.stderr:
        raise ValueError(f'{args}: exit {result.returncode}: {result.stderr!r}')
    return result.stdout


def verify(root):
    prepared = json.loads((root / '.craftguild/craftguild-standards/prepared.json').read_text())
    target_dir = Path(os.environ.get('CARGO_TARGET_DIR', 'target'))
    if not target_dir.is_absolute():
        target_dir = root / target_dir
    binary = target_dir / prepared['build']['target'] / 'release/craftguild-standards'
    if not binary.is_file():
        raise ValueError('Run make build before acceptance.')
    expected_sbom = (root / prepared['sbom']).read_bytes()
    contract = json.loads((root / 'conformance/contracts.json').read_text())['presentation']
    with tempfile.TemporaryDirectory(prefix='craftguild-acceptance-') as temporary:
        temp = Path(temporary)
        isolated = temp / 'only-binary'
        isolated.mkdir()
        copied = isolated / binary.name
        shutil.copy2(binary, copied)
        (isolated / 'craftguild.toml').write_text('deliberately invalid TOML [')
        before = {p.name: p.read_bytes() for p in isolated.iterdir()}
        env = dict(os.environ, PATH='')
        variants = [run(copied, isolated, args, env) for args in contract['help_arguments']]
        bare = variants[0]
        assert all(value == bare for value in variants)
        versions = [run(copied, isolated, args, env) for args in contract['version_arguments']]
        assert all(value == versions[0] for value in versions)
        assert bare.count(b'craftguild/standards') == 1
        lines = bare.decode('ascii').splitlines()
        affiliation = next(i for i, line in enumerate(lines) if line.strip() == 'craftguild/standards')
        width = max(len(line) for line in lines[:affiliation])
        assert lines[affiliation] == ' ' * max(0, (width - len('craftguild/standards')) // 2) + 'craftguild/standards'
        assert b'\x1b' not in bare
        assert expected_sbom == run(copied, isolated, ['sbom'], env)
        assert json.loads(expected_sbom)['specVersion'] == '1.5'
        for command in ['banner', 'build', 'prepare', 'check']:
            help_text = run(copied, isolated, [command, '--help'], env)
            assert help_text.startswith(b'Usage:') and b'craftguild/standards' not in help_text
        assert before == {p.name: p.read_bytes() for p in isolated.iterdir()}
        stage = temp / 'stage'
        subprocess.run(['make', 'install', f'DESTDIR={stage}', 'PREFIX=/opt/craftguild',
                        f'CARGO_TARGET_DIR={target_dir}'], cwd=root, check=True)
        installed = stage / 'opt/craftguild/bin/craftguild-standards'
        assert installed.read_bytes() == binary.read_bytes()
        empty = temp / 'empty-target'
        missing = subprocess.run(['make', 'install', f'DESTDIR={stage}',
                                  f'CARGO_TARGET_DIR={empty}'], cwd=root,
                                 capture_output=True)
        assert missing.returncode and b'Run make build' in missing.stderr
    print('PASS: isolated offline presentation, exact SBOM, subcommand help and staged install')


if __name__ == '__main__':
    verify(Path(__file__).resolve().parent.parent)
