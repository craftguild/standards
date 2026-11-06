#!/usr/bin/env python3
"""Dispatch read-only packaging through an existing integrity-checked engine."""

import hashlib
import json
from pathlib import Path
import subprocess
import sys


def verified_engine(root):
    record = root / '.craftguild/craftguild-standards/receipt.json'
    if not record.is_file():
        raise ValueError('No verified engine found. Run make build without privileges first.')
    receipt = json.loads(record.read_text())
    binary = Path(receipt['context']['output'])
    if not binary.is_file() or binary.is_symlink():
        raise ValueError('Verified engine missing. Run make build without privileges first.')
    digest = hashlib.sha256(binary.read_bytes()).hexdigest()
    if receipt.get('protocol') != 'craftguild-execution-1' or receipt['outputs'] != {binary.name: digest}:
        raise ValueError('Engine integrity check failed. Run make build without privileges first.')
    return binary


if __name__ == '__main__':
    root = Path(__file__).resolve().parent.parent
    if len(sys.argv) < 2 or sys.argv[1] != 'package':
        sys.exit('Usage: engine.py package [--group ID] --output DIR')
    subprocess.run([str(verified_engine(root)), *sys.argv[1:]], cwd=root, check=True)
