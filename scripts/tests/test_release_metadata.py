"""Release index aggregation never mixes sources or admits altered host assets."""

import hashlib
import json
from pathlib import Path
import shutil
import sys
import tempfile
import unittest

from entry_fixture import EntryFixture
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from release_metadata import merge


class MetadataTests(unittest.TestCase):
    def test_exact_index_and_mixed_or_tampered_delivery_refusal(self):
        with tempfile.TemporaryDirectory() as name:
            base = Path(name)
            fixture = EntryFixture(base / 'fixture')
            assets = base / 'hosts/first'
            shutil.copytree(fixture.assets, assets)
            (assets / ('craftguild-release-v7.8.9-' + fixture.machine['target'] + '.json')).write_text(json.dumps(fixture.release))
            result = merge(base / 'hosts', base / 'index')
            self.assertEqual(result, fixture.release)
            index = json.loads((base / 'index/craftguild-release-v7.8.9.json').read_text())
            self.assertEqual(index, result)
            fixture.binary.write_bytes(b'tampered')
            shutil.copy2(fixture.binary, assets / fixture.binary.name)
            with self.assertRaisesRegex(ValueError, 'bytes differ'):
                merge(base / 'hosts', base / 'tampered')
            self.assertFalse((base / 'tampered').exists())
            shutil.rmtree(assets)
            shutil.copytree(fixture.assets, assets)
            # Restore asset authority for the next independent identity check.
            fixture.release['hosts'][fixture.machine['target']]['binary']['sha256'] = hashlib.sha256(fixture.binary.read_bytes()).hexdigest()
            metadata = assets / ('craftguild-release-v7.8.9-' + fixture.machine['target'] + '.json')
            metadata.write_text(json.dumps(fixture.release))
            second = base / 'hosts/second'
            shutil.copytree(assets, second)
            other = json.loads(metadata.read_text())
            other['identity']['revision'] = 'b' * 40
            (second / metadata.name).write_text(json.dumps(other))
            with self.assertRaisesRegex(ValueError, 'source/version/compatibility'):
                merge(base / 'hosts', base / 'mixed')
            self.assertFalse((base / 'mixed').exists())

    def test_host_index_combines_only_matching_unique_host_deliveries(self):
        with tempfile.TemporaryDirectory() as name:
            base = Path(name).resolve()
            first = EntryFixture(base / 'first')
            machine = json.loads(json.dumps(first.machine))
            machine['target'] = ('x86_64-unknown-linux-gnu' if machine['target'] != 'x86_64-unknown-linux-gnu'
                                 else 'aarch64-apple-darwin')
            second = EntryFixture(base / 'second', machine=machine)
            for fixture in [first, second]:
                directory = base / 'hosts' / fixture.machine['target']
                shutil.copytree(fixture.assets, directory)
                (directory / ('craftguild-release-v7.8.9-' + fixture.machine['target'] + '.json')).write_text(json.dumps(fixture.release))
            result = merge(base / 'hosts', base / 'combined')
            self.assertEqual(set(result['hosts']), {first.machine['target'], second.machine['target']})
            shutil.copytree(base / 'hosts' / first.machine['target'], base / 'hosts/duplicate')
            with self.assertRaisesRegex(ValueError, 'Duplicate host'):
                merge(base / 'hosts', base / 'duplicate-result')
            self.assertFalse((base / 'duplicate-result').exists())
