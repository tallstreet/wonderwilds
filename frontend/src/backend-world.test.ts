import { test } from 'node:test';
import assert from 'node:assert/strict';
import type { GameSnapshot } from '../../contracts/generated/contracts.ts';
import {
  completedWorldQuests,
  meadowBeats,
  questReady,
  worldQuests,
  trainingClearing,
} from './backend-world-model.ts';
import { findRoute, isBlocked } from './navigation.ts';
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
  selectedCompanions: [],
});
test('Server flags drive scenery; guest progress is never a prerequisite', () => {
  const state = blank();
  assert.equal(questReady(state, 'meadow-welcome'), true);
  assert.equal(questReady(state, 'meadow-bridge'), false);
  assert.deepEqual(completedWorldQuests(state), []);
  state.storyFlags = meadowBeats.find((b) => b.id === 'meadow-welcome')!.completionFlags;
  assert.equal(questReady(state, 'meadow-bridge'), true);
  assert.equal(questReady(state, 'meadow-welcome'), false);
  assert.equal(questReady(state, 'meadow-finale'), false);
  assert.equal(questReady(state, 'not-a-quest'), false);
  state.ownedCreatures = ['pip'];
  assert.ok(completedWorldQuests(state).includes('befriend-pip'));
});
test('World interactions have reachable approaches, including early friendships before bridge repair', () => {
  const state = blank();
  state.storyFlags = meadowBeats.find((b) => b.id === 'meadow-welcome')!.completionFlags;
  for (const quest of [...worldQuests, trainingClearing]) {
    const fixed = !questReady(state, quest.id);
    const approach = { x: quest.x - 30, y: quest.y + 25 };
    assert.equal(isBlocked(approach.x, approach.y, fixed), false, quest.id);
    const path = findRoute({ x: 160, y: 490 }, approach, fixed);
    assert.ok(path.length > 0, quest.id);
    assert.ok(Math.hypot(path.at(-1)!.x - approach.x, path.at(-1)!.y - approach.y) < 24, quest.id);
  }
});

test('Bridge edge remains traversable after floating-point ground movement', () => {
  assert.equal(isBlocked(608, 368.00000000000006, true), false);
  assert.equal(isBlocked(608, 349.99999999999994, true), false);
  assert.equal(isBlocked(608, 368.001, true), true);
  assert.equal(isBlocked(608, 349.999, true), true);
  assert.equal(isBlocked(608, 368, false), true);
});
