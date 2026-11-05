"""Adapt only an explicit isolated export for the optional orchestration pilots."""

import argparse
import json
from pathlib import Path
import re
import shutil
import sys
import os
import tomllib

ROOT = Path(__file__).resolve().parents[2]


def dump(data):
    lines = []

    def key(parts):
        return '.'.join(json.dumps(part) for part in parts)

    def section(mapping, path=(), array=False):
        if path:
            lines.append(('[[{}]]' if array else '[{}]').format(key(path)))
        for name, value in mapping.items():
            if not isinstance(value, dict) and not (isinstance(value, list) and value and isinstance(value[0], dict)):
                lines.append(json.dumps(name) + ' = ' + json.dumps(value))
        lines.append('')
        for name, value in mapping.items():
            if isinstance(value, dict):
                section(value, (*path, name))
            elif isinstance(value, list) and value and isinstance(value[0], dict):
                for entry in value:
                    section(entry, (*path, name), True)

    section(data)
    text = '\n'.join(lines)
    assert tomllib.loads(text) == data
    return text


def common_source(pilot, entry):
    if not pilot.resolve().is_relative_to((ROOT / '.craftguild/pilots').resolve()):
        raise ValueError('Pilot writes must remain in the standards repository isolated pilot tree')
    destination = pilot / 'standards/execution'
    shutil.copytree(entry, destination)
    source = pilot / 'standards/source'
    source.mkdir()
    for name in ['craftguild-cli', 'craftguild-cli-build', 'craftguild-core']:
        shutil.copytree(ROOT / 'crates' / name, source / 'crates' / name)
    shutil.copytree(ROOT / 'schemas', source / 'schemas')
    workspace = tomllib.loads((ROOT / 'Cargo.toml').read_text())
    workspace['workspace']['members'] = ['crates/craftguild-cli', 'crates/craftguild-cli-build', 'crates/craftguild-core']
    (source / 'Cargo.toml').write_text(dump(workspace))
    manifest = tomllib.loads((pilot / 'Cargo.toml').read_text())
    for name in ['craftguild-cli', 'craftguild-cli-build', 'craftguild-core']:
        if name in manifest['workspace']['dependencies']:
            manifest['workspace']['dependencies'][name] = {'path': 'standards/source/crates/' + name}
    (pilot / 'Cargo.toml').write_text(dump(manifest))
    isolate_source_workspace(pilot)
    (pilot / 'tools/standards.sh').write_text('''#!/bin/sh
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd -P)
cd "$root"
exec python3 standards/execution/launcher.py "$@"
''')
    config = tomllib.loads((pilot / 'craftguild.toml').read_text())
    config['schema_version'] = 2
    config['sources'].extend([
        {'pattern': 'standards/source/**', 'role': 'vendored',
         'reason': 'Isolated unreleased consumer-library source export; original implementation is measured in standards'},
        {'pattern': 'standards/execution/**', 'role': 'vendored',
         'reason': 'Pinned engine/entry source export verified by its exact bundle hash manifest'},
    ])
    for artifact in config['artifacts']:
        if artifact['kind'] == 'rust':
            artifact.setdefault('input_dirs', []).append('standards/source/schemas')
    return config


def isolate_source_workspace(pilot):
    manifest = tomllib.loads((pilot / 'Cargo.toml').read_text())
    excluded = manifest['workspace'].setdefault('exclude', [])
    for name in ['craftguild-cli', 'craftguild-cli-build', 'craftguild-core']:
        directory = 'standards/source/crates/' + name
        if directory not in excluded:
            excluded.append(directory)
        path = pilot / directory / 'Cargo.toml'
        package = tomllib.loads(path.read_text())
        package['package']['workspace'] = '../..'
        path.write_text(dump(package))
    (pilot / 'Cargo.toml').write_text(dump(manifest))


def oriha(pilot, entry):
    config = common_source(pilot, entry)
    manifest = tomllib.loads((pilot / 'Cargo.toml').read_text())
    manifest['workspace']['members'].remove('crates/artifact-tools')
    manifest['workspace']['dependencies'].pop('craftguild-standards')
    (pilot / 'Cargo.toml').write_text(dump(manifest))
    # Keep the old adapter's font-reader regression as a real ordinary test.
    test = pilot / 'crates/core/tests/font_archive_compatibility.rs'
    test.parent.mkdir(exist_ok=True)
    shutil.copy2(pilot / 'crates/artifact-tools/tests/font_archive_compatibility.rs', test)
    core = tomllib.loads((pilot / 'crates/core/Cargo.toml').read_text())
    core.setdefault('dev-dependencies', {})['figlet-rs'] = '=1.0.0'
    (pilot / 'crates/core/Cargo.toml').write_text(dump(core))
    shutil.rmtree(pilot / 'crates/artifact-tools')
    for policy in config['policies']:
        if policy['kind'] == 'dependencies':
            policy['layers'].pop('oriha-artifact-tools')
    config['releases'][0]['cargo'][0]['packages'].remove('oriha-artifact-tools')
    libraries = json.loads((pilot / 'assets/library-artifacts.json').read_text())['libraries']
    for library in libraries:
        config['artifacts'].append({
            'id': library['id'], 'command': library['package'], 'project': 'oriha.rs',
            'kind': 'rust', 'manifest': 'Cargo.toml', 'lockfile': 'Cargo.lock',
            'package': library['package'], 'supplemental': library['supplemental'],
            'inputs': ['assets/library-artifacts.json', *library.get('inputs', [])],
            'input_dirs': ['standards/source/schemas'],
            'build': {'library': {'name': library['target_name'], 'formats':
                                 ['cdylib', 'rlib'] if library['id'] == 'wasm' else ['cdylib', 'staticlib', 'rlib']},
                      'dependencies': ['native-bridge'] if library['id'] == 'windows-bridge' else []},
        })
    for name in ['test', 'check']:
        config.setdefault('operations', []).append({'name': name, 'artifact': 'oriha-cli',
                                                   'prepare': ['native-bridge', 'wasm', 'windows-bridge', 'oriha-cli']})
    for command in config['commands']:
        if command['id'] == 'release-version-ownership':
            command['argv'] = ['sh', 'tools/standards.sh', 'check-release-config']
    finish_oriha(pilot, config)
    return {'removed_adapter': 'oriha-artifact-tools', 'retained_font_gate': str(test.relative_to(pilot)),
            'library_artifacts': [library['id'] for library in libraries]}


def finish_oriha(pilot, config):
    install_forwarder(pilot, 'scripts/install.py')
    if not any(c['id'] == 'cli-standalone' for c in config['commands']):
        config['commands'].append({'id': 'cli-standalone', 'phase': 'acceptance',
                                   'purpose': 'Retain offline CLI, renderer JSON and staged install acceptance',
                                   'argv': ['python3', 'scripts/acceptance.py']})
    for artifact in config['artifacts']:
        if artifact['id'] in ['native-bridge', 'windows-bridge']:
            artifact['build']['library']['formats'] = ['cdylib', 'staticlib', 'rlib']
    for directory in ['native-bridge', 'wasm', 'windows-bridge']:
        source = pilot / 'crates' / directory / 'src'
        symbol = 'oriha_' + directory.replace('-', '_')
        (source / 'distribution.rs').write_text('''//! Verified distribution metadata; no UniFFI or CXX contract changes.
const SBOM: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/craftguild-sbom.json"));
#[no_mangle]
pub extern "C" fn SYMBOL_sbom_bytes() -> *const u8 {
    SBOM.as_ptr()
}
#[no_mangle]
pub extern "C" fn SYMBOL_sbom_size() -> usize {
    SBOM.len()
}
'''.replace('SYMBOL', symbol))
        library = source / 'lib.rs'
        text = library.read_text()
        if 'mod distribution;' not in text:
            library.write_text('mod distribution;\n' + text)
    (pilot / 'craftguild.toml').write_text(dump(config))
    (pilot / 'Makefile').write_text('''ARTIFACT ?= oriha-cli
RELEASE_ARGS ?= --group oriha --apply
ACCEPTANCE_ARGS ?= --gate cli-standalone
include standards/execution/make/common.mk
.PHONY: all build-cli build-expo build-font-registry prepare-libraries acceptance-font-registry
all build-cli: build
build-expo:
	node packages/expo-oriha/scripts/build-package.mjs
build-font-registry:
	$(CRAFTGUILD_LAUNCH) prepare --artifact font-registry
prepare: prepare-libraries
prepare-libraries:
	$(CRAFTGUILD_LAUNCH) prepare --artifact native-bridge
	$(CRAFTGUILD_LAUNCH) prepare --artifact wasm
	$(CRAFTGUILD_LAUNCH) prepare --artifact windows-bridge
acceptance-font-registry:
	$(CRAFTGUILD_LAUNCH) acceptance --gate font-registry-runtime
''')
    launcher = pilot / 'tools/standards.sh'
    text = launcher.read_text()
    if 'CRAFTGUILD_TOKEI' not in text:
        text = text.replace('cd "$root"', 'export CRAFTGUILD_TOKEI="$root/tools/tokei.py"\ncd "$root"')
    launcher.write_text(text)
    (pilot / 'tools/artifacts.sh').write_text('''#!/bin/sh
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd -P)
cd "$root"
exec python3 tools/library-entry.py "$@"
''')
    (pilot / 'tools/library-entry.py').write_text('''import os, sys
args = ['--artifact' if arg == '--library' else arg for arg in sys.argv[1:]]
os.execv(sys.executable, [sys.executable, 'standards/execution/launcher.py', *args])
''')
    if not any(s['pattern'] == 'tools/library-entry.py' for s in config['sources']):
        config['sources'].append({'pattern': 'tools/library-entry.py', 'role': 'implementation',
                                  'reason': 'Public library selector forwarding alias to common engine'})
    (pilot / 'craftguild.toml').write_text(dump(config))


def chigiri(pilot, entry):
    config = common_source(pilot, entry)
    artifact = config['artifacts'][0]
    artifact['inputs'] = [p for p in artifact['inputs'] if not p.startswith('tools/standards-adapter/')]
    artifact['input_dirs'] = [p for p in artifact['input_dirs'] if not p.startswith('tools/standards-adapter/')]
    sdk_inputs = ['scripts/check-build-tools.sh', 'scripts/build-tool-versions.sh',
                  'scripts/pilot-platform.py', 'scripts/pilot-xwin.sh', 'scripts/run-without-network.sh']
    sdk_inputs += [str(p.relative_to(pilot)) for p in (pilot / 'scripts').glob('xwin*.py')]
    artifact['inputs'].extend(sdk_inputs + ['rust-toolchain.toml'])
    variants = []
    targets = []
    for alias, target, compiler in [
        ('macos-arm64', 'aarch64-apple-darwin', ['cargo']),
        ('macos-x86_64', 'x86_64-apple-darwin', ['cargo']),
        ('linux-x86_64', 'x86_64-unknown-linux-gnu', ['cross']),
        ('windows-x86_64', 'x86_64-pc-windows-msvc', ['sh', 'scripts/pilot-xwin.sh']),
    ]:
        data = {'target': target, 'output': 'target/dist/{target}/{file}',
                'output_root': 'repository', 'compiler': compiler}
        variants.append({'id': alias, **data})
        targets.append({'cross_only': True, **data})
    artifact['build'] = {
        'toolchain': '1.95.0', 'output': '{target_path}{profile}/{file}',
        'environment': {'CARGO_ENCODED_RUSTFLAGS': '', 'CARGO_ENCODED_RUSTDOCFLAGS': ''},
        'targets': targets, 'variants': variants,
        'hooks': [
            {'id': 'platform-sdk', 'stage': 'prerequisite',
             'argv': ['python3', 'scripts/pilot-platform.py', 'sdk'], 'inputs': sdk_inputs,
             'outputs': ['.craftguild/derived/platform.json'],
             'environment_output': '.craftguild/derived/platform.json',
             'unset_env': ['CARGO_TARGET_DIR', 'CARGO_ENCODED_RUSTFLAGS', 'CARGO_ENCODED_RUSTDOCFLAGS']},
            {'id': 'distribution-format', 'stage': 'verify',
             'argv': ['python3', 'scripts/pilot-platform.py', 'verify'],
             'inputs': ['scripts/pilot-platform.py', 'scripts/verify-dist.py']},
        ],
    }
    (pilot / 'scripts/pilot-platform.py').write_text('''import json, os, pathlib, subprocess, sys
root = pathlib.Path(__file__).resolve().parent.parent
target = os.environ['CRAFTGUILD_TARGET']
host = os.environ['CRAFTGUILD_HOST']
distribution = bool(os.environ['CRAFTGUILD_VARIANT']) or target != host
if sys.argv[1] == 'sdk':
    if distribution and os.environ['CRAFTGUILD_PROFILE'] != 'release':
        raise SystemExit('Cross/distribution builds require release profile')
    subprocess.run(['sh', 'scripts/check-build-tools.sh', target if distribution else 'native'], check=True)
    environment = {}
    if distribution and target.endswith('apple-darwin'):
        environment['SDKROOT'] = subprocess.check_output(['xcrun','--sdk','macosx','--show-sdk-path'], text=True).strip()
        linker = subprocess.check_output(['xcrun','--sdk','macosx','--find','clang'], text=True).strip()
        environment['CARGO_TARGET_' + target.upper().replace('-', '_') + '_LINKER'] = linker
    if distribution and target == 'x86_64-unknown-linux-gnu':
        environment.update(CROSS_CONFIG=str(root / 'Cross.toml'), CROSS_CONTAINER_ENGINE='docker')
    pathlib.Path('.craftguild/derived/platform.json').write_text(json.dumps(environment))
elif sys.argv[1] == 'verify':
    if distribution:
        subprocess.run([sys.executable, 'scripts/verify-dist.py', '--artifact', target,
                        os.environ['CRAFTGUILD_CANDIDATE']], check=True)
else:
    raise SystemExit('Expected sdk or verify')
''')
    (pilot / 'scripts/pilot-xwin.sh').write_text('''#!/bin/sh
set -eu
. ./scripts/build-tool-versions.sh
source_cache=${CHIGIRI_XWIN_CACHE_DIR:-"$PWD/target/tool-cache/cargo-xwin-$CHIGIRI_CARGO_XWIN_VERSION-sdk-$CHIGIRI_WINDOWS_SDK_VERSION-crt-$CHIGIRI_WINDOWS_CRT_VERSION-vs-$CHIGIRI_XWIN_VERSION-$CHIGIRI_XWIN_VARIANT"}
view=$(mktemp -d .craftguild/xwin-view.XXXXXX)
cleanup() { python3 scripts/xwin-cache.py unseal-view "$view" >/dev/null 2>&1 || true; rm -rf "$view"; }
trap cleanup EXIT HUP INT TERM
python3 scripts/xwin-cache.py stage "$source_cache" "$view" "$CHIGIRI_CARGO_XWIN_VERSION" "$CHIGIRI_XWIN_VERSION" "$CHIGIRI_WINDOWS_SDK_VERSION" "$CHIGIRI_WINDOWS_CRT_VERSION" x86_64 "$CHIGIRI_XWIN_VARIANT"
python3 scripts/xwin-cache.py verify-view "$view"
case $(uname -s) in Darwin) guard=sandbox-exec; option=--deny-network;; Linux) guard=unshare; option=--user-network;; *) exit 1;; esac
export CARGO_NET_OFFLINE=true XWIN_CACHE_DIR="$view" XWIN_ARCH=x86_64 XWIN_VERSION="$CHIGIRI_XWIN_VERSION" XWIN_SDK_VERSION="$CHIGIRI_WINDOWS_SDK_VERSION" XWIN_CRT_VERSION="$CHIGIRI_WINDOWS_CRT_VERSION" XWIN_VARIANT="$CHIGIRI_XWIN_VARIANT" XWIN_CROSS_COMPILER=clang-cl XWIN_INCLUDE_ATL=false XWIN_INCLUDE_DEBUG_LIBS=false XWIN_INCLUDE_DEBUG_SYMBOLS=false XWIN_HTTP_RETRIES=0
scripts/run-without-network.sh "$guard" "$option" cargo xwin "$@"
python3 scripts/xwin-cache.py verify-view "$view"
''')
    for name in ['test', 'check']:
        config.setdefault('operations', []).append({
            'name': name, 'artifact': 'chigiri-cli', 'prepare': ['chigiri-cli'],
            'hooks': [{'id': 'ownership', 'stage': 'evidence',
                       'argv': ['bash', 'scripts/python-with-toml.sh', 'scripts/refresh-standards-ownership.py'],
                       'inputs': ['scripts/refresh-standards-ownership.py'],
                       'output_dirs': ['.craftguild/ownership']}],
        })
    config['sources'].append({'pattern': 'scripts/pilot-*', 'role': 'implementation',
                              'reason': 'Bounded platform SDK and binary-format gates in the isolated pilot'})
    shutil.rmtree(pilot / 'tools/standards-adapter')
    language = pilot / 'scripts/standards-language.py'
    text = language.read_text()
    start = text.index('        manifest = "tools/standards-adapter/Cargo.toml"')
    end = text.index('        run(["sh", "tools/standards.sh", "banner"', start)
    text = text[:start] + '        run(["sh", "tools/standards.sh", "check-release-config"])\n' + text[end:]
    language.write_text(text)
    finish_chigiri(pilot, config)
    return {'removed_adapter': 'chigiri-standards-adapter',
            'retained_asset_gate': artifact['bundle']['command'], 'sdk_hook': 'platform-sdk',
            'preserved_distribution_aliases': [v['id'] for v in variants]}


def finish_chigiri(pilot, config):
    install_forwarder(pilot, 'scripts/install-cli.py')
    build = config['artifacts'][0]['build']
    for settings in [*build['variants'], *build['targets']]:
        settings.update(output='target/dist/{target}/{file}', output_root='repository')
    shutil.copy2(ROOT / 'scripts/tests/pilot_acceptance.py', pilot / 'scripts/pilot_acceptance.py')
    if not any(s['pattern'] == 'scripts/pilot_acceptance.py' for s in config['sources']):
        config['sources'].append({'pattern': 'scripts/pilot_acceptance.py', 'role': 'test-helper',
                                  'reason': 'Isolated common-receipt install and retained public CLI acceptance'})
    for command in config['commands']:
        if command['phase'] == 'acceptance':
            command['argv'] = ['python3', 'scripts/pilot_acceptance.py']
    if not any(op['name'] == 'acceptance' for op in config['operations']):
        config['operations'].append({'name': 'acceptance', 'artifact': 'chigiri-cli', 'build': ['chigiri-cli']})
    size = pilot / 'scripts/test-standards-size.py'
    text = size.read_text().replace('TOOL = ROOT / ".craftguild/tool-build/debug/chigiri-standards-adapter"',
                                  'PIN = json.loads((ROOT / "standards/execution/engine.json").read_text())\n'
                                  'TOOL = ROOT / ".craftguild/engine" / PIN["bundle_sha256"] / "bin/craftguild-standards"')
    text = text.replace('report["measured"]["src/mixed.rs"], 2)',
                        'report["measured"]["src/mixed.rs"], 3)')
    size.write_text(text)
    (pilot / 'scripts/build-native.sh').write_text('''#!/bin/sh
set -eu
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd -P)
cd "$root"
exec python3 standards/execution/launcher.py build --profile "${1:-release}"
''')
    (pilot / 'scripts/build-distribution.sh').write_text('''#!/bin/sh
set -eu
case ${1:-} in
 aarch64-apple-darwin) variant=macos-arm64;;
 x86_64-apple-darwin) variant=macos-x86_64;;
 x86_64-unknown-linux-gnu) variant=linux-x86_64;;
 x86_64-pc-windows-msvc) variant=windows-x86_64;;
 *) echo 'Expected a supported distribution target' >&2; exit 2;;
esac
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd -P)
cd "$root"
exec python3 standards/execution/launcher.py build --variant "$variant" --profile release
''')
    (pilot / 'craftguild.toml').write_text(dump(config))
    (pilot / 'Makefile').write_text('''ARTIFACT ?= chigiri-cli
RELEASE_ARGS ?= --group chigiri --apply
include standards/execution/make/common.mk
.PHONY: debug macos-arm64 macos-x86_64 linux-x86_64 windows-x86_64 check-build-tools verify-dist cross
debug:
	$(CRAFTGUILD_LAUNCH) build --profile debug
macos-arm64 macos-x86_64 linux-x86_64 windows-x86_64:
	$(CRAFTGUILD_LAUNCH) build --variant $@
check-build-tools:
	./scripts/check-build-tools.sh all
verify-dist:
	./scripts/verify-dist.py
cross: check-build-tools macos-arm64 macos-x86_64 linux-x86_64 windows-x86_64 verify-dist
''')


def install_forwarder(pilot, name):
    (pilot / name).write_text('''#!/usr/bin/env python3
import os, pathlib, sys
root = pathlib.Path(__file__).resolve().parent.parent
os.chdir(root)
os.execv(sys.executable, [sys.executable, 'standards/execution/launcher.py', 'install', *sys.argv[1:]])
''')


def npm_audit_only_tools(pilot):
    """Allow only the user-authorized audit lookup; npm installation stays offline."""
    real = shutil.which('npm')
    tools = pilot / '.craftguild/pilot-tools'
    tools.mkdir(parents=True, exist_ok=True)
    script = '#!' + sys.executable + '\nimport os,sys\n'
    script += 'args=sys.argv[1:]\n'
    script += 'if args and args[0]=="ci": args.insert(1,"--offline")\n'
    script += 'if args and args[0]=="audit": args[1:1]=["--offline=false","--registry=https://registry.npmjs.org/"]\n'
    script += 'os.execv(' + repr(real) + ', [' + repr(real) + ',*args])\n'
    (tools / 'npm').write_text(script)
    (tools / 'npm').chmod(0o755)
    python = tools / 'python3'
    if not python.exists():
        python.symlink_to(sys.executable)
    (tools / 'empty-npmrc').write_text('')
    return {'PATH': str(tools) + os.pathsep + os.environ['PATH'],
            'NPM_CONFIG_USERCONFIG': str(tools / 'empty-npmrc'),
            'NPM_CONFIG_LOGS_DIR': str(tools / 'logs')}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pilot', choices=['oriha', 'chigiri'], required=True)
    parser.add_argument('--directory', type=Path, required=True)
    parser.add_argument('--entry', type=Path, required=True)
    args = parser.parse_args()
    prepare = oriha if args.pilot == 'oriha' else chigiri
    print(json.dumps(prepare(args.directory.resolve(), args.entry.resolve()), indent=2))
