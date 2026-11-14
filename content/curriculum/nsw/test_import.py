"""Integrity and delivery-boundary checks for the replacement, not curriculum approval."""
import hashlib
import importlib.util
import json
import math
import pathlib
import unittest
from unittest.mock import patch

ROOT = pathlib.Path(__file__).resolve().parents[3]
spec = importlib.util.spec_from_file_location('nsw_import', ROOT/'scripts/import-nsw-curriculum.py')
compiler = importlib.util.module_from_spec(spec)
spec.loader.exec_module(compiler)


class ReplacementTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.source = json.loads((compiler.PACK/'source-v1.json').read_text())
        cls.bank = json.loads((compiler.PACK/'exercises-v1.json').read_text())

    def test_inventory_and_fingerprints(self):
        self.assertEqual(len(self.source['skills']), 736)
        self.assertEqual(sum(len(s['practice_items']) for s in self.source['skills']), 8125)
        for skill in self.source['skills']:
            self.assertGreaterEqual(len({i['item_fingerprint'] for i in skill['practice_items']}), 10)
            self.assertTrue(skill['learning_resources'])
        report = json.loads((compiler.PACK/'delivery-report-v1.json').read_text())
        self.assertEqual(report['sourceSha256'], hashlib.sha256((compiler.PACK/'source-v1.json').read_bytes()).hexdigest())

    def test_graph_is_acyclic_and_links_are_consistent(self):
        nodes = {s['id']: s for s in self.source['skills']}
        done = set()
        while len(done) < len(nodes):
            ready = {i for i,s in nodes.items() if i not in done and set(s['prerequisites']) <= done}
            self.assertTrue(ready, 'Dangling dependency or cycle')
            done.update(ready)
        for skill in nodes.values():
            for child in skill['children']:
                self.assertIn(skill['id'], nodes[child]['prerequisites'])
            for resource in skill['learning_resources']:
                self.assertTrue(resource['url'].startswith('https://'))

    def test_supported_routes_cannot_acquire_answer_keys(self):
        self.assertFalse(self.bank['releaseEnabled'])
        self.assertEqual(self.bank['reviewStatus'], 'pending')
        for skill in self.bank['skills']:
            for exercise in skill['exercises']:
                for item in exercise['items']:
                    if exercise['assessmentMode'] == 'human_review':
                        self.assertIsNone(item['answer'])
                        self.assertEqual(item['interaction']['kind'], 'workshop')
                        self.assertTrue(exercise['limitations'])
                    for media in item['stimuli']:
                        if media['kind'] == 'diagram':
                            path = compiler.ASSETS/(media['assetId']+'.svg')
                            self.assertEqual(hashlib.sha256(path.read_bytes()).hexdigest(),media['assetId'])

    def test_svg_and_unknown_media_fail_closed(self):
        for svg in ['<svg><script>alert(1)</script></svg>', '<svg onload="run()"/>', '<svg><image href="https://example.com"/></svg>', '<svg><rect fill="url(https://example.com)"/></svg>']:
            with self.assertRaises(compiler.Unsupported): compiler.asset(svg)
        with self.assertRaises(compiler.Unsupported): compiler.visual({'kind':'unknown'})
        source_item = self.source['skills'][0]['practice_items'][0]
        with self.assertRaises(compiler.Unsupported):
            compiler.convert(self.source['skills'][0], {**source_item,'stimulus':{'audio':{'script':'private answer'}}})

    def test_clock_coordinates_ignore_platform_trig_roundoff(self):
        def clocks(value):
            if isinstance(value, dict):
                if value.get('kind') == 'analogue_clock':
                    yield value
                for child in value.values():
                    yield from clocks(child)
            elif isinstance(value, list):
                for child in value:
                    yield from clocks(child)

        visuals = list(clocks(self.source))
        self.assertTrue(visuals)
        sine, cosine = math.sin, math.cos
        # Simulate the last-bit differences observed between host libm builds.
        # The complete generated SVG (and therefore its hash) must remain equal.
        with patch.object(compiler, 'asset', side_effect=lambda svg: svg):
            expected = [compiler.visual(v) for v in visuals]
            for direction in (math.inf, -math.inf):
                with patch.object(math, 'sin', side_effect=lambda a: math.nextafter(sine(a), direction)), \
                     patch.object(math, 'cos', side_effect=lambda a: math.nextafter(cosine(a), direction)):
                    self.assertEqual([compiler.visual(v) for v in visuals], expected)
        self.assertEqual(compiler.clock_coordinate(-0.0), '0')
        self.assertEqual(compiler.clock_coordinate(-0.000000001), '0')
        self.assertEqual(compiler.clock_coordinate(50.00000000000001), '50')
        self.assertEqual(compiler.clock_coordinate(12.3456789), '12.345679')


if __name__ == '__main__': unittest.main()
