#!/usr/bin/env python3
"""Verify a self-contained skill archive without using its source checkout."""

import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import subprocess
import tarfile
import tempfile


def verify(bundle):
    with tempfile.TemporaryDirectory(prefix='craftguild-skill-check-') as temporary:
        root = Path(temporary)
        with tarfile.open(bundle) as archive:
            for item in archive.getmembers():
                relative = PurePosixPath(item.name)
                if relative.is_absolute() or '..' in relative.parts or not relative.parts:
                    raise ValueError('Unsafe skill archive path')
                if relative.parts[0] != 'craftguild-standardize':
                    raise ValueError('Unexpected skill archive root')
                destination = root / item.name
                if item.isdir():
                    destination.mkdir(parents=True, exist_ok=True)
                elif item.isfile():
                    destination.parent.mkdir(parents=True, exist_ok=True)
                    destination.write_bytes(archive.extractfile(item).read())
                    destination.chmod(item.mode & 0o777)
                else:
                    raise ValueError('Nonregular skill archive member')
        skill = root / 'craftguild-standardize'
        manifest = json.loads((skill / 'bundle.json').read_text())
        actual = {p.relative_to(skill).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest()
                  for p in skill.rglob('*') if p.is_file() and p != skill / 'bundle.json'}
        assert actual == manifest['files']
        assert (skill / 'references/standards/quality.md').is_file()
        assert (skill / 'references/orchestration.md').is_file()
        for path in skill.rglob('*.md'):
            text = re.sub(r'```.*?```', '', path.read_text(), flags=re.S)
            for target in re.findall(r'\[[^\]]*\]\(([^)]+)\)', text):
                if '://' in target or target.startswith(('mailto:', '#')):
                    continue
                target = target.split('#')[0]
                assert not target or (path.parent / target).is_file(), (path, target)
        binary = skill / 'assets/bin/craftguild-standards'
        env = dict(os.environ, PATH='')
        for args in [['--help'], ['--version'], ['sbom']]:
            result = subprocess.run([str(binary), *args], cwd=root, env=env,
                                    capture_output=True, check=True)
            assert not result.stderr
            if args == ['sbom']:
                assert json.loads(result.stdout)['specVersion'] == '1.5'
        entry = skill / 'assets/execution'
        result = subprocess.run([os.sys.executable, str(entry / 'launcher.py'), 'setup'],
                                cwd=root, capture_output=True, check=True)
        assert result.stdout.strip()
    print('PASS: standalone skill manifest, local rules, links and offline executable')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('bundle', type=Path)
    verify(parser.parse_args().bundle)
