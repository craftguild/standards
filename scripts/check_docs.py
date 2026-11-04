#!/usr/bin/env python3
"""Check local Markdown targets and pinned third-party input bytes."""

import hashlib
import json
from pathlib import Path
import re
import sys

from public_files import inventory


def check(root):
    errors = []
    files = inventory(root)['files']
    for record in files:
        path = root / record['path']
        if path.suffix != '.md':
            continue
        source = path.read_text()
        # Code examples may contain intentionally hypothetical document names.
        source = re.sub(r'```.*?```', '', source, flags=re.S)
        for target in re.findall(r'\[[^\]]*\]\(([^)]+)\)', source):
            if '://' in target or target.startswith(('mailto:', '#')):
                continue
            target = target.split('#')[0]
            if target and not (path.parent / target).is_file():
                errors.append(f'{record["path"]}: missing local link {target}')
    provenance = json.loads((root / 'conformance/third-party-inputs.json').read_text())
    for item in provenance:
        digest = hashlib.sha256((root / item['path']).read_bytes()).hexdigest()
        if digest != item['sha256']:
            errors.append('Changed vendored third-party bytes: ' + item['path'])
    if errors:
        raise ValueError('\n'.join(errors))
    print(f'PASS: local document links and {len(provenance)} pinned third-party inputs')


if __name__ == '__main__':
    try:
        check(Path(__file__).resolve().parent.parent)
    except (ValueError, OSError) as error:
        print(error, file=sys.stderr)
        sys.exit(1)
