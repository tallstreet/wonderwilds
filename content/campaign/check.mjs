import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import Ajv2020 from '../../contracts/node_modules/ajv/dist/2020.js';

const read = (path) => JSON.parse(readFileSync(new URL(path, import.meta.url), 'utf8'));
const checkSchema = new Ajv2020({ allErrors: true, strict: true }).compile(read('./schema.v1.json'));
const creatures = new Set(read('../creatures/friends.v1.json').creatures.map((c) => c.id));
const pack = read('./moonlight-trail.v1.json');

function validate(pack) {
  assert.ok(checkSchema(pack), JSON.stringify(checkSchema.errors));
  const unique = (ids) => assert.equal(new Set(ids).size, ids.length, 'duplicate IDs');
  unique(pack.missions.map((m) => m.id));
  unique(pack.training.map((t) => t.id));
  unique(pack.collectionIds);
  unique(pack.starterChoice.creatureIds);
  assert.equal(pack.starterChoice.creatureIds.length, 3);
  assert.equal(pack.starterChoice.count, 1);
  for (const id of pack.collectionIds) assert.ok(creatures.has(id), `unknown creature ${id}`);
  for (const id of pack.starterChoice.creatureIds) assert.ok(pack.collectionIds.includes(id));
  const reached = new Set();
  for (const mission of pack.missions) {
    assert.ok(mission.requires.every((id) => reached.has(id)), `unreachable ${mission.id}`);
    assert.ok(mission.steps.length > 0);
    unique(mission.steps.map((s) => s.id));
    for (const step of mission.steps) {
      assert.ok(pack.locations.includes(step.locationId), `unknown location ${step.locationId}`);
      if (['choose_companion', 'choose_design', 'care', 'travel', 'talk', 'celebrate'].includes(step.kind)) {
        assert.equal(step.learning, false, 'ordinary choices must not be assessed');
      }
    }
    reached.add(mission.id);
  }
  const thresholds = pack.progression.characterLevelThresholds;
  assert.equal(thresholds[0], 0);
  assert.equal(pack.training.length, thresholds.length - 1);
  let xp = 0;
  for (const [index, training] of pack.training.entries()) {
    assert.ok(reached.has(training.unlockedBy));
    assert.ok(pack.locations.includes(training.locationId));
    assert.equal(training.requiresLevel, index + 1);
    assert.equal(xp, thresholds[index], 'training must be reachable without replay grinding');
    assert.equal(training.grantScope, 'profile_companion_milestone');
    assert.equal(training.replayXp, 0);
    xp += training.xp;
    assert.equal(xp, thresholds[index + 1]);
  }
  for (const slot of pack.progression.teamSlots) assert.ok(reached.has(slot.after));
  assert.equal(pack.progression.supportedRewardMultiplier, 1);
  assert.equal(pack.progression.storyRequiresAcademicMastery, false);
  assert.equal(pack.progression.storyRequiresCharacterLevel, false);
  assert.equal(pack.care.neglectPenalty, false);
  assert.equal(pack.care.blocksAdventures, false);
  assert.equal(pack.sourcePolicy.generationRequired, false);
  // Simulate recruitment for every initial choice, including both second-friend choices.
  for (const starter of pack.starterChoice.creatureIds) {
    for (const second of pack.starterChoice.creatureIds.filter((id) => id !== starter)) {
      const owned = new Set();
      for (const mission of pack.missions) for (const step of mission.steps) {
        if (step.kind === 'choose_companion') owned.add(starter);
        if (step.kind !== 'recruit') continue;
        const id = step.targetId === 'unchosen_starter' ? second
          : step.targetId === 'remaining_starter'
            ? pack.starterChoice.creatureIds.find((id) => !owned.has(id)) : step.targetId;
        assert.ok(pack.collectionIds.includes(id), 'recruitment must resolve to an available friend');
        assert.ok(!owned.has(id), 'new-player recruitment must not duplicate ownership');
        owned.add(id);
      }
      assert.deepEqual([...owned].sort(), [...pack.collectionIds].sort());
    }
  }
}
validate(pack);
const invalid = [
  (p) => { p.releaseEnabled = true; },
  (p) => { p.missions[0].requires = ['gathering']; },
  (p) => { p.missions[1].steps[0].locationId = 'missing'; },
  (p) => { p.training[0].xp = 1; },
  (p) => { p.starterChoice.creatureIds[0] = 'missing'; },
  (p) => { p.missions[0].steps[1].learning = true; },
  (p) => { p.missions[4].steps[3].targetId = 'pip'; },
  (p) => { p.training[0].unexpected = true; },
];
for (const mutate of invalid) {
  const copy = structuredClone(pack);
  mutate(copy);
  assert.throws(() => validate(copy));
}
console.log(`Campaign schema, mission order, XP reachability and six starter/recruitment paths passed; ${invalid.length} invalid packs rejected. Runtime playability is not established.`);
