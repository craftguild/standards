"""Install must copy the selected artifact and fail without invoking a compiler."""

import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

MODULE = Path(__file__).resolve().parents[1] / 'install.py'
SPEC = importlib.util.spec_from_file_location('craftguild_install', MODULE)
INSTALL = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(INSTALL)


class InstallTests(unittest.TestCase):
    def test_custom_target_directory_destdir_and_missing_binary(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            record = root / '.craftguild/craftguild-standards/prepared.json'
            record.parent.mkdir(parents=True)
            record.write_text(json.dumps({'build': {'target': 'test-target'}}))
            stage = root / 'stage'
            with self.assertRaisesRegex(ValueError, 'Run make build'):
                INSTALL.install(root, '/opt/test/bin', str(stage), 'custom', '')
            binary = root / 'custom/test-target/release/craftguild-standards'
            binary.parent.mkdir(parents=True)
            binary.write_bytes(b'prebuilt executable fixture')
            INSTALL.install(root, '/opt/test/bin', str(stage), 'custom', 'test-target')
            self.assertEqual(binary.read_bytes(),
                             (stage / 'opt/test/bin/craftguild-standards').read_bytes())
            with self.assertRaisesRegex(ValueError, 'TARGET differs'):
                INSTALL.install(root, '/opt/test/bin', str(stage), 'custom', 'other-target')


if __name__ == '__main__':
    unittest.main()
