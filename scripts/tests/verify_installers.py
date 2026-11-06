"""Real host installer/package acceptance through a matching distributed entry."""

import hashlib
import json
import platform
from pathlib import Path
import shutil
import sys
import tempfile

from installer_fixture import InstallerFixture
from signing import signed_git
import verify_execution
from verify_execution import consumer, offline, run


def files(directory):
    return {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in directory.iterdir()}


def verify(entry, environment=None):
    verify_execution.TRANSPORT.clear()
    verify_execution.TRANSPORT.update(environment or {})
    with tempfile.TemporaryDirectory(prefix='standards-real-installer-') as name:
        base = Path(name).resolve()
        root = base / 'consumer'
        consumer(root, 'rust-only', entry)
        config = (root / 'craftguild.toml').read_text().replace(
            'schema_version = 2', 'schema_version = 2\nrust_output_layout = "standard"')
        config += '''
[[installers]]
artifact="backend"
repository="owner/repo"
release_group="sample"
output="scripts/download.sh"
'''
        (root / 'craftguild.toml').write_text(config)
        (root / '.gitignore').write_text('.craftguild/\ntarget/\noffline/\n')
        # Package requires explicit setup; it cannot acquire the pinned engine.
        refused = run(root, ['make', 'package', 'PACKAGE_ARGS=--output ' + str(base / 'absent')], success=False)
        assert b'engine cache is missing' in refused.stderr
        assert not (root / '.craftguild/engine').exists()
        run(root, ['make', 'setup'])
        run(root, ['make', 'installer'])
        generated = (root / 'scripts/download.sh').read_bytes()
        run(root, ['make', 'installer', 'INSTALLER_ARGS=--artifact backend --check'])
        run(root, [sys.executable, 'standards/launcher.py', 'installer', '--check'])
        engine = next((root / '.craftguild/engine').glob('*/*/craftguild-standards'))
        run(root, [engine, 'installer', '--artifact', 'backend'])
        assert (root / 'scripts/download.sh').read_bytes() == generated
        (root / 'scripts/download.sh').write_bytes(generated + b'# drift\n')
        run(root, [engine, 'installer', '--check'], success=False)
        (root / 'scripts/download.sh').write_bytes(generated)
        with signed_git() as git:
            git(root, 'init', '-q')
            git(root, 'add', '.')
            git(root, 'commit', '-S', '-qm', 'feat: isolated installer acceptance')
            run(root, ['make', 'build'])
            record = offline(root, 'sample')
            target = record['context']['target']
            version = record['prepared']['build']['version']
            revision = record['prepared']['build']['commit']
            assert record['prepared']['build']['dirty'] == 'clean'
            assert Path(record['context']['output']) == root / 'target' / target / 'release/sample'
            for operation, destination in [
                ([engine, 'package'], base / 'direct'),
                ([sys.executable, 'standards/launcher.py', 'package'], base / 'launcher'),
                (['make', 'package'], base / 'make'),
            ]:
                if operation == ['make', 'package']:
                    argv = [*operation, 'PACKAGE_ARGS=--group sample --output ' + str(destination)]
                else:
                    argv = [*operation, '--group', 'sample', '--output', destination]
                run(root, argv)
            assert files(base / 'direct') == files(base / 'launcher') == files(base / 'make')
            asset = f'sample-v{version}-{target}'
            fixture = InstallerFixture(base / 'runtime', command='sample', version=version)
            shutil.rmtree(fixture.assets)
            shutil.copytree(base / 'direct', fixture.assets)
            fixture.env.update(OS=platform.system(), ARCH=platform.machine())
            script = root / 'scripts/download.sh'
            fixture.run('--version', version, '--bindir', str(fixture.bindir), script=script)
            installed = fixture.bindir / 'sample'
            assert installed.read_bytes() == (base / 'direct' / asset).read_bytes()
            isolated = base / 'offline'
            isolated.mkdir()
            for args in [[], ['--help'], ['--version'], ['--machine-version'], ['sbom']]:
                actual = run(isolated, [installed, *args], {'PATH': ''}).stdout
                expected = run(isolated, [record['context']['output'], *args], {'PATH': ''}).stdout
                assert actual == expected
            assert run(isolated, [installed, 'sbom'], {'PATH': ''}).stdout == (root / record['prepared']['sbom']).read_bytes()
            settings = fixture.home / 'settings'
            settings.write_text('keep settings')
            other = fixture.bindir / 'other'
            other.write_text('keep command')
            for tool in ['curl', 'uname', 'sha256sum', 'shasum']:
                (fixture.tools / tool).unlink(missing_ok=True)
            fixture.run('--uninstall', '--bindir', str(fixture.bindir), script=script)
            assert not installed.exists() and settings.read_text() == 'keep settings' and other.read_text() == 'keep command'
            fixture.clean()
            print(json.dumps({'host': platform.system(), 'architecture': platform.machine(),
                              'target': target, 'version': version, 'revision': revision,
                              'assets': files(base / 'direct'),
                              'sudo': [event for event in fixture.log() if event[0] == 'sudo']}), flush=True)
    print('PASS: real distributed consumer, standard output, Make/CLI/launcher installer/package, temporary install/uninstall, offline identity and exact embedded SBOM', flush=True)


if __name__ == '__main__':
    verify(Path(sys.argv[1]).resolve())
