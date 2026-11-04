"""Disposable OpenPGP test identity, isolated from personal Git/GPG settings."""

from contextlib import contextmanager
import os
from pathlib import Path
import subprocess
import tempfile


@contextmanager
def signed_git():
    with tempfile.TemporaryDirectory(prefix='craftguild-test-gpg-') as temporary:
        Path(temporary).chmod(0o700)
        env = dict(os.environ, GNUPGHOME=temporary, GIT_CONFIG_GLOBAL=os.devnull,
                   GIT_CONFIG_NOSYSTEM='1')
        try:
            subprocess.run(['gpg', '--batch', '--pinentry-mode', 'loopback',
                            '--passphrase', '', '--quick-generate-key',
                            'Conformance <conformance@example.invalid>',
                            'ed25519', 'sign', '0'], env=env, check=True,
                           stdout=subprocess.PIPE, stderr=subprocess.PIPE)
            keys = subprocess.check_output(['gpg', '--batch', '--with-colons',
                                            '--list-secret-keys'], env=env, text=True)
            key = next(line.split(':')[9] for line in keys.splitlines()
                       if line.startswith('fpr:'))

            def git(root, *args):
                command = ['git', '-c', 'gpg.format=openpgp', '-c', 'gpg.program=gpg',
                           '-c', 'user.signingkey=' + key,
                           '-c', 'user.name=Conformance',
                           '-c', 'user.email=conformance@example.invalid']
                result = subprocess.check_output(command + list(args), cwd=root,
                                                 env=env, text=True).strip()
                if args[0] == 'commit':
                    revision = git(root, 'rev-parse', 'HEAD')
                    git(root, 'verify-commit', revision)
                return result

            git.environment = env
            yield git
        finally:
            subprocess.run(['gpgconf', '--homedir', temporary, '--kill', 'gpg-agent'],
                           stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
                           check=False)
