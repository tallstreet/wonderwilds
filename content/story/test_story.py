import unittest
from validate import discover, load, validate


class StoryTests(unittest.TestCase):
    def test_authored_content(self):
        self.assertEqual(validate(), [])

    def test_cycles_and_dangling_dependencies_rejected(self):
        for dependency in ['cove-finale', 'invented-beat']:
            data = load('content/story/adventure.v1.json')
            data['beats'][0]['requiresBeatIds'] = [dependency]
            self.assertTrue(validate(adventure=data))

    def test_rare_friend_and_mastery_gates_rejected(self):
        for key in ['requiredCreatureIds', 'requiredMasteryIds']:
            data = load('content/story/adventure.v1.json')
            data['beats'][1][key] = ['invented-gate']
            self.assertTrue(validate(adventure=data))

    def test_missing_or_unverified_assets_block_release(self):
        data = load('content/manifests/story-assets.json')
        data['releaseReady'] = True
        self.assertTrue(validate(assets=data))
        data = load('content/creatures/friends.v1.json')
        data['creatures'][0]['assetId'] = 'invented-art'
        self.assertTrue(validate(friends=data))

    def test_bounded_discovery_and_reproducibility(self):
        for region in ['meadow', 'orchard', 'cove']:
            for initial_seed in range(512):
                seed, owned = initial_seed, set()
                history = []
                for _ in range(4):
                    previous_seed = seed
                    seed, friend = discover(seed, region, owned)
                    self.assertEqual((seed,friend), discover(previous_seed,region,owned))
                    self.assertNotIn(friend, owned)
                    history.append(friend)
                    owned.add(friend)
                self.assertEqual(len(owned),4)
                if region == 'meadow':
                    self.assertEqual(history[0], 'pip')

    def test_declining_preserves_offer_and_seed(self):
        seed, friend = discover(12,'orchard',set())
        for _ in range(20):
            self.assertEqual(discover(seed,'orchard',set(),pending=friend), (seed,friend))

    def test_all_collected_revisits_avoid_immediate_repeat(self):
        roster = {c['id'] for c in load('content/creatures/friends.v1.json')['creatures'] if c['regionId']=='cove'}
        for seed in range(100):
            _, friend = discover(seed,'cove',roster,last='luna')
            self.assertNotEqual(friend,'luna')

    def test_supported_complete_journey_reaches_ending(self):
        data = load('content/story/adventure.v1.json')
        completed, flags, rewards = set(), set(), set()
        while len(completed) < len(data['beats']):
            available = [b for b in data['beats'] if b['id'] not in completed and set(b['requiresBeatIds']) <= completed]
            self.assertTrue(available)
            for beat in available:
                self.assertTrue(beat['supportedCompletion']['allowed'])
                completed.add(beat['id'])
                flags.update(beat['completionFlags'])
                for reward in beat['rewards']:
                    self.assertNotIn(reward['id'], rewards)
                    rewards.add(reward['id'])
        self.assertIn(data['endingBeatId'],completed)
        self.assertTrue(set(data['ending']['requiredFlags']) <= flags)


if __name__ == '__main__':
    unittest.main()
