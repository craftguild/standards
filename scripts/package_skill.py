#!/usr/bin/env python3
"""Package the skill with its exact rules and an already-built host executable."""

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import tarfile
import tempfile


def package(root, destination):
    record = json.loads((root / '.craftguild/craftguild-standards/prepared.json').read_text())
    # Accept only the configured built target, never a bootstrap executable.
    binary = root / 'target' / record['build']['target'] / 'release/craftguild-standards'
    if not binary.is_file():
        raise ValueError('Run make build before packaging the skill.')
    with tempfile.TemporaryDirectory(prefix='craftguild-skill-') as temporary:
        skill = Path(temporary) / 'craftguild-standardize'
        shutil.copytree(root / 'skills/craftguild-standardize', skill)
        shutil.copytree(root / 'spec', skill / 'references/standards')
        shutil.copytree(root / 'schemas', skill / 'schemas')
        shutil.copytree(root / 'assets/fonts', skill / 'assets/fonts')
        shutil.copytree(root / 'conformance', skill / 'conformance')
        bundled = skill / 'assets/bin/craftguild-standards'
        bundled.parent.mkdir(parents=True)
        shutil.copy2(binary, bundled)
        for source in ['LICENSE-MIT', 'LICENSE-APACHE']:
            shutil.copy2(root / source, skill / source)
        workflow = skill / 'references/workflow.md'
        workflow.write_text(workflow.read_text().replace('../../../spec/README.md',
                                                       'standards/README.md'))
        entries = {p.relative_to(skill).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest()
                   for p in sorted(skill.rglob('*')) if p.is_file()}
        manifest = {'schema_version': 1, 'standards_version': '0.1.0',
                    'target': record['build']['target'], 'build': record['build'],
                    'files': entries}
        (skill / 'bundle.json').write_text(json.dumps(manifest, indent=2) + '\n')
        destination.parent.mkdir(parents=True, exist_ok=True)
        with tarfile.open(destination, 'w:gz') as archive:
            archive.add(skill, arcname=skill.name)
    print(destination)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    package(Path(__file__).resolve().parent.parent, args.output)
