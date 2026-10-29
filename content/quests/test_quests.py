import copy
import unittest
from validate import load, validate_catalog, validate_document


class QuestValidationTests(unittest.TestCase):
    def setUp(self):
        self.catalog, self.documents = copy.deepcopy(load())
        self.path = 'content/quests/pip-lantern-trail.v1.json'
        self.document = self.documents[self.path]
        self.slots = {slot['id']: slot for slot in self.catalog['activitySlots']}

    def rejects(self, mutate):
        mutate(self.document)
        with self.assertRaises(ValueError):
            validate_document(self.document, self.slots)

    def test_authored_catalog_and_choice_specific_continuations(self):
        validate_catalog(self.catalog, self.documents)
        for document in self.documents.values():
            reflection = next(n for n in document['nodes'] if n['id'] == 'reflection')
            self.assertEqual(len(set(c['nextNodeId'] for c in reflection['choices'])), 2)

    def test_dangling_reference(self):
        self.rejects(lambda d: d['nodes'][0]['choices'][0].update(nextNodeId='missing'))

    def test_cycle(self):
        self.rejects(lambda d: d['nodes'][1].update(nextNodeId='start'))

    def test_unreachable_node(self):
        self.rejects(lambda d: d['nodes'].append({'id': 'orphan', 'type': 'handoff', 'text': 'Another day.', 'continuationTags': ['future-path']}))

    def test_activity_bypass(self):
        self.rejects(lambda d: d['nodes'][0]['choices'][0].update(nextNodeId='handoff-stars'))

    def test_activity_allowlist(self):
        self.rejects(lambda d: d['nodes'][1].update(activitySlotId='invented-mastery'))

    def test_companion_slot_binding(self):
        self.rejects(lambda d: d['nodes'][1].update(activitySlotId='nib-share'))

    def test_model_cannot_declare_rewards(self):
        self.rejects(lambda d: d.update(rewardXp=999))

    def test_model_cannot_declare_answer_key(self):
        self.rejects(lambda d: d['nodes'][1].update(answer=3))

    def test_duplicate_node_choice_and_text_bounds(self):
        for mutate in [lambda d: d['nodes'][1].update(id='start'), lambda d: d['nodes'][0]['choices'][1].update(id='pond'), lambda d: d['nodes'][0].update(text='x' * 601)]:
            with self.subTest(mutate=mutate):
                candidate = copy.deepcopy(self.document)
                mutate(candidate)
                with self.assertRaises(ValueError):
                    validate_document(candidate, self.slots)

    def test_unsafe_paths_and_duplicate_rewards(self):
        for mutate in [lambda c: c['quests'][0].update(documentPath='../secrets'), lambda c: c['quests'][1].update(rewardItemId=c['quests'][0]['rewardItemId'])]:
            candidate = copy.deepcopy(self.catalog)
            mutate(candidate)
            with self.assertRaises(ValueError):
                validate_catalog(candidate, self.documents)

    def test_review_cannot_be_self_approved(self):
        self.catalog['releaseEnabled'] = True
        with self.assertRaises(ValueError):
            validate_catalog(self.catalog, self.documents)

    def test_new_versions_need_deliberate_reader(self):
        self.rejects(lambda d: d.update(schemaVersion='2.0'))

    def test_followups_require_reachable_same_companion_source_hooks(self):
        for policy in [
            {"tag":"woodland-trails","sourceQuestIds":["pip-woodland-whispers"],"acknowledgement":"Pip remembers a clue."},
            {"tag":"made-up-tag","sourceQuestIds":["pip-lantern-trail"],"acknowledgement":"Pip remembers a clue."}
        ]:
            candidate=copy.deepcopy(self.catalog)
            next(entry for entry in candidate['quests'] if entry['questId']=='pip-woodland-whispers')['continuation']=policy
            with self.assertRaises(ValueError):
                validate_catalog(candidate,self.documents)


if __name__ == '__main__':
    unittest.main()
