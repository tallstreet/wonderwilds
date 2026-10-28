import copy
import tempfile
import unittest
from pathlib import Path

from validate import ROOT, check_hashes, load_bundle, validate


class CurriculumValidationTests(unittest.TestCase):
    def setUp(self):
        self.bundle = load_bundle()

    def rejects(self, mutation, expected):
        mutation(self.bundle)
        self.assertTrue(any(expected in e for e in validate(self.bundle)), validate(self.bundle))

    def test_current_pack(self):
        self.assertEqual(validate(self.bundle), [])
        self.assertEqual(check_hashes(), [])

    def test_cycle(self):
        self.rejects(lambda b: b['seed-v1.json']['nodes'][0]['prerequisites'].append('M03'), 'cycle')

    def test_dangling(self):
        self.rejects(lambda b: b['seed-v1.json']['nodes'][0]['prerequisites'].append('M99'), 'dangling')

    def test_duplicate_id(self):
        self.rejects(lambda b: b['seed-v1.json']['nodes'].append(copy.deepcopy(b['seed-v1.json']['nodes'][0])), 'Duplicate skill')

    def test_parameter_outside_bound(self):
        self.rejects(lambda b: b['authoring-v1.json']['skills'][0]['difficultyLadder'][0]['parameters'].update(quantity=999), 'outside bounds')

    def test_boolean_is_not_number(self):
        self.rejects(lambda b: b['seed-v1.json']['nodes'][0]['parameterBounds']['quantity'].update(min=True), 'invalid parameter bound')

    def test_unknown_representation(self):
        self.rejects(lambda b: b['authoring-v1.json']['skills'][0]['difficultyLadder'][0].update(representation='colour_only'), 'unknown context or representation')

    def test_missing_supported_route(self):
        self.rejects(lambda b: b['authoring-v1.json']['skills'][0]['difficultyLadder'][0].update(supportAvailable=False), 'supported completion')

    def test_unreviewed_release(self):
        self.rejects(lambda b: b['integrity-index-v1.json'].update(releaseEnabled=True), 'Release requires')

    def test_fabricated_descriptor(self):
        self.rejects(lambda b: b['seed-v1.json']['nodes'][0]['alignmentRefs'][0].update(descriptorCode='NOT-VERIFIED'), 'requires reviewer')

    def test_coverage_promotion(self):
        self.rejects(lambda b: b.update({'coverage.csv': b['coverage.csv'].replace(',authored,', ',released,', 1)}), 'Coverage promotion')

    def test_unknown_policy(self):
        self.rejects(lambda b: b['seed-v1.json']['nodes'][0].update(evidencePolicyId='unreviewed'), 'unknown evidence policy')

    def test_adult_observation_is_never_automatic(self):
        self.rejects(lambda b: b['evidence-policies-v1.json']['policies'][-1].update(automaticCertification=True), 'automatically certify')

    def test_hash_tampering(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for file in ROOT.iterdir():
                if file.is_file():
                    (root / file.name).write_bytes(file.read_bytes())
            with (root / 'seed-v1.json').open('a') as file:
                file.write(' ')
            self.assertEqual(check_hashes(root), ['Manifest hash mismatch: seed-v1.json'])


if __name__ == '__main__':
    unittest.main()
