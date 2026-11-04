"""Actual-host smoke using only a prebuilt sample binary in a temporary directory."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
root = Path(__file__).resolve().parent.parent
prepared = json.loads((root / ".craftguild/sample/prepared.json").read_text())
target = Path(os.environ.get("CARGO_TARGET_DIR", "target"))
if not target.is_absolute():
    target = root / target
binary = target / prepared["build"]["target"] / "release/sample"
with tempfile.TemporaryDirectory() as directory:
    copy = Path(directory) / "sample"
    shutil.copy2(binary, copy)
    output = subprocess.check_output([str(copy), "sbom"], cwd=directory, env=dict(os.environ, PATH=""))
    assert output == (root / prepared["sbom"]).read_bytes()
print("PASS: sample binary emits its exact embedded SBOM without runtime tools")
