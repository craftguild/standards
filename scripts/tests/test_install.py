"""Install must copy the selected artifact and fail without invoking a compiler."""

import importlib.util
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

MODULE = Path(__file__).resolve().parents[1] / 'install.py'
SPEC = importlib.util.spec_from_file_location('craftguild_install', MODULE)
INSTALL = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(INSTALL)


class InstallTests(unittest.TestCase):
    def test_missing_or_tampered_engine_never_dispatches(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            record = root / '.craftguild/craftguild-standards-engine/receipt.json'
            record.parent.mkdir(parents=True)
            stage = root / 'stage'
            with self.assertRaisesRegex(ValueError, 'Run make build'):
                INSTALL.install(root, '/opt/test/bin', str(stage), 'custom', '')
            binary = root / 'custom/test-target/release/craftguild-standards-engine'
            binary.parent.mkdir(parents=True)
            binary.write_bytes(b'prebuilt executable fixture')
            receipt = {'protocol': 'craftguild-execution-1',
                       'context': {'output': str(binary.resolve())},
                       'outputs': {binary.name: hashlib.sha256(binary.read_bytes()).hexdigest()}}
            record.write_text(json.dumps(receipt))
            with patch.object(INSTALL.subprocess, 'run') as execute:
                INSTALL.install(root, '/opt/test/bin', str(stage), 'custom', 'test-target')
                argv = execute.call_args.args[0]
                self.assertEqual(argv[:2], [str(binary.resolve()), 'install'])
                self.assertIn(str(stage), argv)
                self.assertIn('test-target', argv)
                execute.reset_mock()
                binary.write_bytes(b'tampered')
                with self.assertRaisesRegex(ValueError, 'integrity'):
                    INSTALL.install(root, '/opt/test/bin', str(stage), 'custom', '')
                execute.assert_not_called()


if __name__ == '__main__':
    unittest.main()
