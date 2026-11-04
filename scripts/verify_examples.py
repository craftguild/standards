#!/usr/bin/env python3
"""Build and inspect all three distribution examples through fixed HTTPS revisions."""

import json
import os
from pathlib import Path
import subprocess
import tempfile

from verify_identity import verify as verify_identity


def verify(root):
    own = json.loads((root / '.craftguild/craftguild-standards/prepared.json').read_text())
    own_target = Path(os.environ.get('CARGO_TARGET_DIR', 'target'))
    if not own_target.is_absolute():
        own_target = root / own_target
    tool = own_target / own['build']['target'] / 'release/craftguild-standards'
    if not tool.is_file():
        raise ValueError('Run make build before verifying examples.')
    with tempfile.TemporaryDirectory(prefix='craftguild-example-build-') as temporary:
        env = dict(os.environ, CARGO_TARGET_DIR=temporary)
        for name, command in [('rust-only', 'sample'), ('embedded-web', 'embedded-demo'),
                              ('independent', 'backend-demo')]:
            sample = root / 'examples' / name
            subprocess.run(['make', 'build', 'STANDARDS=' + str(tool)], cwd=sample,
                           env=env, check=True)
            subprocess.run(['make', 'acceptance'], cwd=sample, env=env, check=True)
            subprocess.run(['make', 'check', 'STANDARDS=' + str(tool)], cwd=sample,
                           env=env, check=True)
            prepared = json.loads((sample / f'.craftguild/{command}/prepared.json').read_text())
            bom = json.loads((sample / prepared['sbom']).read_text())
            included = {c['name'] for c in bom['components']}
            assert 'craftguild-cli' in included
            assert 'craftguild-cli-build' not in included
            assert 'craftguild-core' not in included
            assert ('is-number' in included) == (name == 'embedded-web')
            assert not included & {'esbuild', 'typescript', 'picocolors'}
            if name == 'independent':
                frontend = json.loads((sample / '.craftguild/independent-ui/bom.json').read_text())
                assert {c['name'] for c in frontend['components']} == {'is-number'}
                assert frontend['metadata']['component']['name'] == 'independent-ui'
                assert not any(p.startswith('frontend/') for p in prepared['inputs'])
        verify_identity(root, tool, env)
    print('PASS: real Rust-only, embedded npm and independent frontend/backend distributions')


if __name__ == '__main__':
    verify(Path(__file__).resolve().parent.parent)
