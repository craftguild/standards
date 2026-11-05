"""Test-only acceptance for the common receipt and retained Chigiri CLI grammar."""

import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def verify(root):
    receipt = json.loads((root / '.craftguild/chigiri/receipt.json').read_text())
    context = receipt['context']
    binary = Path(context['output'])
    prepared = receipt['prepared']
    bom = (root / prepared['sbom']).read_bytes()
    assert receipt['outputs'] == {binary.name: digest(binary)}
    assert hashlib.sha256(bom).hexdigest() == prepared['sbom_sha256']
    assert {p['name'] for p in json.loads(bom)['metadata']['properties']} == {
        'craftguild:build', 'craftguild:features'}
    spec = importlib.util.spec_from_file_location('retained_acceptance', root / 'scripts/standards-acceptance.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    with tempfile.TemporaryDirectory(prefix='craftguild-pilot-acceptance-') as name:
        base = Path(name)
        module.offline(binary, base, {'build': prepared['build']}, bom)
        tools = base / 'tools'
        tools.mkdir()
        (tools / 'python3').symlink_to(sys.executable)
        marker = base / 'forbidden-tool'
        for tool in ['cargo', 'rustc', 'rustup', 'npm', 'node', 'git']:
            denied = tools / tool
            denied.write_text('#!' + sys.executable + '\nfrom pathlib import Path\n'
                              + 'Path(' + repr(str(marker)) + ').touch()\nraise SystemExit(97)\n')
            denied.chmod(0o755)
        env = {k: v for k, v in os.environ.items() if k not in {
            'TARGET', 'PROFILE', 'ARTIFACT', 'VARIANT', 'TOOLCHAIN', 'FEATURES',
            'NO_DEFAULT_FEATURES', 'CARGO_TARGET_DIR', 'BINDIR', 'DESTDIR', 'PREFIX'}}
        env.update(PATH=str(tools), HOME=str(base / 'runtime/home'))
        make = shutil.which('make')
        stage = base / 'stage'

        def install(*selectors, success=True):
            result = subprocess.run([make, 'install', f'DESTDIR={stage}', 'PREFIX=/opt/chigiri',
                                     *selectors], cwd=root, env=env, capture_output=True, timeout=30)
            assert (result.returncode == 0) == success, result.stderr.decode()
            assert not marker.exists(), 'Install invoked a build or Git tool'

        install()
        installed = stage / 'opt/chigiri/bin/chigiri'
        expected = binary.read_bytes()
        assert installed.read_bytes() == expected
        install('BINDIR=/opt/chigiri-custom/sbin')
        assert (stage / 'opt/chigiri-custom/sbin/chigiri').read_bytes() == expected
        install('PROFILE=debug', success=False)
        install(f'CARGO_TARGET_DIR={base / "missing"}', success=False)
        try:
            binary.write_bytes(b'tampered output')
            install(success=False)
            assert installed.read_bytes() == expected
        finally:
            binary.write_bytes(expected)
        descriptor = json.loads((root / 'standards/execution/engine.json').read_text())
        cache = root / '.craftguild/engine' / descriptor['bundle_sha256']
        backup = cache.with_name(cache.name + '.acceptance-backup')
        cache.rename(backup)
        try:
            install(success=False)
            assert not cache.exists(), 'Install acquired a missing engine cache'
        finally:
            backup.rename(cache)
    print('PASS: retained offline Chigiri grammar/machine JSON/output failure, exact SBOM; copy-only install, selectors, tamper and missing cache')


if __name__ == '__main__':
    verify(Path(__file__).resolve().parents[1])
