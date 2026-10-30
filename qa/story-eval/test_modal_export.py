"""Offline validation of BF16 export prerequisites."""
import json
from pathlib import Path
import sys
from tempfile import TemporaryDirectory
import unittest

sys.path.insert(0, str(Path(__file__).parent / 'modal'))
from merge_worker import check_config, check_source, MODEL, REVISION


class ExportTests(unittest.TestCase):
    def test_nested_quantization_is_rejected(self):
        check_config({'text_config': {'dtype': 'bfloat16'}})
        with self.assertRaises(ValueError):
            check_config({'text_config': {'quantization_config': {'quant_method': 'bitsandbytes'}}})

    def test_source_requires_completed_pinned_run_and_adapter(self):
        with TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'status.json').write_text(json.dumps({'status': 'running'}))
            (root / 'environment.json').write_text(json.dumps({'baseModel': MODEL, 'revision': REVISION}))
            with self.assertRaises(ValueError):
                check_source(root)
            (root / 'status.json').write_text(json.dumps({'status': 'completed'}))
            with self.assertRaises(ValueError):
                check_source(root)
            (root / 'adapter').mkdir()
            for name in ('adapter_config.json', 'adapter_model.safetensors'):
                (root / 'adapter' / name).write_text('fixture')
            self.assertEqual(check_source(root), root / 'adapter')
            (root / 'environment.json').write_text(json.dumps({'baseModel': MODEL, 'revision': 'wrong'}))
            with self.assertRaises(ValueError):
                check_source(root)
