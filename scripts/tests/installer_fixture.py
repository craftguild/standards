"""Process-boundary network/privilege stubs for the real generated POSIX shell."""

import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / 'scripts/install.sh'
TARGETS = ['x86_64-apple-darwin', 'aarch64-apple-darwin',
           'x86_64-unknown-linux-gnu', 'aarch64-unknown-linux-gnu']

STUB = r'''
import json,os,pathlib,shutil,subprocess,sys
name=pathlib.Path(sys.argv[0]).name
args=sys.argv[1:]
with open(os.environ['EVENTS'],'a') as log: log.write(json.dumps([name,*args])+'\n')
if name=='uname':
    print(os.environ['OS'] if args==['-s'] else os.environ['ARCH'])
elif name=='id': print(os.environ.get('UID_STUB','501'))
elif name=='curl':
    url=next(a for a in args if a.startswith('https://'))
    if os.environ.get('NETWORK_FAIL','') and os.environ['NETWORK_FAIL'] in url: sys.exit(22)
    if url.endswith('/latest'): print(os.environ['LATEST'],end='')
    else:
        source=pathlib.Path(os.environ['ASSETS'])/url.rsplit('/',1)[1]
        if not source.is_file(): sys.exit(22)
        shutil.copyfile(source,args[args.index('-o')+1])
elif name=='sudo':
    if os.environ.get('SUDO_FAIL','')==args[0]: sys.exit(1)
    # Unlock only this fixture's directory; never use real sudo.
    pathlib.Path(os.environ['LOCKED']).chmod(0o755)
    if os.environ.get('SIGNAL_AT','')==args[0]:
        import signal
        os.kill(os.getppid(),signal.SIGTERM)
        sys.exit(1)
    sys.exit(subprocess.call(args))
else:
    sys.exit(subprocess.call([os.environ['REAL_HASH'],*args]))
'''


class InstallerFixture:
    def __init__(self, root, payload=b'#!/bin/sh\nexit 42\n', command='craftguild-standards', version='1.2.3'):
        self.command, self.version = command, version
        self.root = root.resolve()
        self.tools = self.root / 'tools'
        self.assets = self.root / 'assets'
        self.home = self.root / 'home'
        self.temp = self.root / 'tmp'
        self.bindir = self.root / 'destination with spaces'
        for path in [self.tools, self.assets, self.home, self.temp]:
            path.mkdir(parents=True)
        self.events = self.root / 'events.jsonl'
        self.env = dict(os.environ, PATH=str(self.tools), HOME=str(self.home),
                        TMPDIR=str(self.temp), ASSETS=str(self.assets),
                        EVENTS=str(self.events), OS='Darwin', ARCH='arm64',
                        LATEST='https://github.com/craftguild/standards/releases/tag/v1.2.3')
        for key in ['BINDIR', 'NETWORK_FAIL', 'SUDO_FAIL', 'SIGNAL_AT', 'UID_STUB']:
            self.env.pop(key, None)
        for name in ['awk', 'dirname', 'mktemp', 'rm', 'mkdir', 'cp', 'chmod', 'mv']:
            (self.tools / name).symlink_to(shutil.which(name))
        for name in ['curl', 'uname', 'id', 'sudo']:
            self.stub(name)
        self.hash('sha256sum' if shutil.which('sha256sum') else 'shasum')
        self.publish(payload)

    def stub(self, name):
        path = self.tools / name
        path.write_text('#!' + sys.executable + '\n' + STUB)
        path.chmod(0o755)

    def hash(self, name):
        for existing in ['sha256sum', 'shasum']:
            (self.tools / existing).unlink(missing_ok=True)
        self.env['REAL_HASH'] = shutil.which(name)
        self.stub(name)

    def publish(self, payload):
        for target in TARGETS:
            name = self.command + '-v' + self.version + '-' + target
            (self.assets / name).write_bytes(payload)
            (self.assets / ('SHA256SUMS-' + target + '.txt')).write_text(
                hashlib.sha256(payload).hexdigest() + '  ' + name + '\n')

    def run(self, *args, success=True, script=SCRIPT):
        result = subprocess.run(['/bin/sh', str(script), *args], env=self.env,
                                capture_output=True, timeout=30)
        if (result.returncode == 0) != success:
            raise AssertionError(f'{args}: {result.returncode}\n{result.stderr.decode()}')
        return result

    def log(self):
        if not self.events.exists():
            return []
        return [json.loads(line) for line in self.events.read_text().splitlines()]

    def clear(self):
        self.events.unlink(missing_ok=True)

    def clean(self):
        assert not list(self.temp.iterdir()), 'download temporary files leaked'
        if self.bindir.exists():
            assert not list(self.bindir.glob('.' + self.command + '.*')), 'destination staging leaked'
