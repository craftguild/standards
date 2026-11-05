#!/usr/bin/env python3
"""Package the skill with its exact rules and an already-built host executable."""

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import tarfile
import tempfile
from package_entry import package as package_entry


def package(root, destination):
    receipt = json.loads((root / '.craftguild/craftguild-standards/receipt.json').read_text())
    record = receipt['prepared']
    with tempfile.TemporaryDirectory(prefix='craftguild-skill-') as temporary:
        skill = Path(temporary) / 'craftguild-standardize'
        shutil.copytree(root / 'skills/craftguild-standardize', skill)
        shutil.copytree(root / 'spec', skill / 'references/standards')
        shutil.copytree(root / 'schemas', skill / 'schemas')
        shutil.copytree(root / 'assets/fonts', skill / 'assets/fonts')
        shutil.copytree(root / 'conformance', skill / 'conformance')
        shutil.copy2(root / 'docs/design/orchestration.md', skill / 'references/orchestration.md')
        entry = skill / 'assets/execution'
        package_entry(root, entry)
        binary = entry / 'bundle/bin/craftguild-standards'
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
