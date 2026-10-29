import { test } from 'node:test';
import assert from 'node:assert/strict';
import type { GameSnapshot } from '../../contracts/generated/contracts.ts';
import { meadowBeats, worldQuests } from './backend-world-model.ts';
import {
  encounterDialogue,
  nextTrailEncounter,
  parseQuestTrailState,
  trailChecklistView,
  trailReadyToReturn,
  trailSteps,
} from './exploration-quest.ts';

const blank = (): GameSnapshot => ({
  schemaVersion: '1.0',
  revision: 0,
  contentVersion: 'test',
  rulesVersion: 'test',
  storyFlags: [],
  ownedCreatures: [],
  inventory: [],
  pendingEncounters: [],
  learnerSummaries: [],
});
const accepted = { accepted: true, returned: false };
const finishBeat = (snapshot: GameSnapshot, id: string) => {
  snapshot.storyFlags.push(...meadowBeats.find((beat) => beat.id === id)!.completionFlags);
};

test('Stored trail conversation state rejects malformed fields and cannot confer activity progress', () => {
  for (const value of [
    null,
    [],
    'true',
    { accepted: 'true', returned: true },
    { accepted: true },
    1,
  ]) {
    assert.deepEqual(parseQuestTrailState(value), { accepted: false, returned: false });
  }
  assert.deepEqual(parseQuestTrailState({ accepted: false, returned: true }), {
    accepted: false,
    returned: false,
  });
  const forged = parseQuestTrailState({ accepted: true, returned: true, done: ['meadow-bridge'] });
  assert.equal(nextTrailEncounter(blank(), forged), 'meadow-welcome');
  assert.equal(trailSteps(blank(), forged).at(-1)!.complete, false);
  assert.equal(trailReadyToReturn(blank()), false);
});

test('Trail follows server confirmations across three maps and returns to Pip only after all tasks', () => {
  const snapshot = blank();
  assert.equal(
    nextTrailEncounter(snapshot, { accepted: false, returned: false }),
    'meadow-welcome',
  );
  for (const id of ['meadow-welcome', 'meadow-bridge', 'meadow-picnic']) {
    assert.equal(nextTrailEncounter(snapshot, accepted), id);
    finishBeat(snapshot, id);
  }
  assert.equal(nextTrailEncounter(snapshot, accepted), 'befriend-tansy');
  snapshot.ownedCreatures.push('tansy');
  assert.equal(nextTrailEncounter(snapshot, accepted), 'meadow-garden');
  finishBeat(snapshot, 'meadow-garden');
  assert.equal(trailReadyToReturn(snapshot), true);
  assert.equal(nextTrailEncounter(snapshot, accepted), 'meadow-welcome');
  assert.equal(nextTrailEncounter(snapshot, { accepted: true, returned: true }), undefined);
  assert.deepEqual(
    [...new Set(trailSteps(snapshot, accepted).map((step) => step.region))],
    ['meadow', 'cove', 'orchard'],
  );
});

test('Previously completed activities remain checked without requiring replay or local acceptance', () => {
  const snapshot = blank();
  finishBeat(snapshot, 'meadow-bridge');
  const steps = trailSteps(snapshot, { accepted: false, returned: false });
  assert.equal(steps.find((step) => step.id === 'meadow-bridge')!.complete, true);
  assert.equal(steps[0].complete, false);
  assert.equal(
    nextTrailEncounter(snapshot, { accepted: false, returned: false }),
    'meadow-welcome',
  );
});

test('Checklist exposes one current step, completion and locked states without mutating progress', () => {
  const snapshot = blank();
  const before = structuredClone(snapshot);
  const html = trailChecklistView(snapshot, accepted);
  assert.equal(html.match(/aria-current="step"/g)?.length, 1);
  assert.match(html, /Complete/);
  assert.match(html, /Current step/);
  assert.match(html, /Locked/);
  assert.match(html, /Helping together counts too/);
  assert.deepEqual(snapshot, before);
});

test('Dialogue covers every existing encounter and friendship help uses the authored problem', () => {
  for (const quest of worldQuests) {
    const dialogue = encounterDialogue(quest.id, blank(), accepted);
    assert.ok(dialogue.speaker && dialogue.text && dialogue.actionLabel, quest.id);
  }
  assert.match(encounterDialogue('befriend-tansy', blank(), accepted).text, /numbered seed pots/);
  assert.equal(
    encounterDialogue('meadow-welcome', blank(), { accepted: false, returned: false }).actionLabel,
    'Accept the trail',
  );
  assert.equal(
    encounterDialogue('meadow-training-clearing', blank(), accepted).actionLabel,
    'Train together',
  );
});
