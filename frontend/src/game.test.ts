import test from 'node:test';
import assert from 'node:assert/strict';
import { answerFor, isCorrect, validateEncounter } from './content.ts';
import {
  activeEncounters,
  freshState,
  restoreProgress,
  completeEncounter,
  writeProgress,
  readProgress,
  validEvidence,
  skillSummary,
} from './progress.ts';
import { findRoute, isBlocked } from './navigation.ts';
const [bridge, picnic, garden] = activeEncounters;
const session = { attempts: 2, hints: 0, assisted: false };
test('All active encounters are valid and maths is checked deterministically', () => {
  activeEncounters.forEach((e) => assert.equal(validateEncounter(e), true));
  assert.equal(answerFor(bridge), 4);
  assert.equal(isCorrect(bridge, 4), true);
  assert.equal(isCorrect(bridge, 5), false);
  assert.equal(isCorrect(picnic, [3, 3]), true);
  assert.equal(isCorrect(picnic, [2, 4]), false);
  assert.equal(isCorrect(picnic, [3, 3, 0]), false);
  assert.equal(isCorrect(garden, 5), true);
});
test('Story stars are unique while repeated practice remains in evidence', () => {
  let s = completeEncounter(freshState(), bridge, session);
  s = completeEncounter(s, bridge, session);
  assert.deepEqual(s.done, [bridge.id]);
  assert.equal(s.evidence.length, 2);
  assert.equal(skillSummary(s)[0].independent, 2);
});
test('Hints or assisted completion always mark evidence as supported', () => {
  for (const override of [{ hints: 1 }, { assisted: true }, { hints: 3, assisted: true }]) {
    const s = completeEncounter(freshState(), bridge, { ...session, ...override });
    assert.equal(s.evidence[0].supported, true);
    assert.equal(s.done.length, 1);
    assert.equal(skillSummary(s)[0].independent, 0);
  }
});
test('Restore rejects malformed or contradictory evidence and unknown rewards', () => {
  const s = completeEncounter(freshState(), bridge, session),
    e = s.evidence[0];
  s.done.push('other-map');
  s.evidence.push(
    { ...e, hints: 1, supported: false },
    { ...e, attempts: -1 },
    { ...e, representation: 'equal-plates' },
    { ...e, skillId: garden.skillId },
  );
  const restored = restoreProgress(JSON.stringify(s));
  assert.equal(restored.evidence.length, 1);
  assert.deepEqual(restored.done, [bridge.id]);
  assert.equal(validEvidence({ ...e, completedAt: 'invalid' }), false);
});
test('Malformed saves and unavailable storage recover safely', () => {
  assert.deepEqual(restoreProgress('{bad'), freshState());
  assert.deepEqual(restoreProgress('{"version":9}'), freshState());
  const storage = {
    getItem() {
      throw new Error('blocked');
    },
    setItem() {
      throw new Error('full');
    },
  };
  assert.equal(readProgress(storage).available, false);
  assert.equal(writeProgress(freshState(), storage), false);
});
test('Saved progress restores the current exploration map and details', () => {
  const s = completeEncounter(freshState(), picnic, { ...session, hints: 2 });
  s.region = 'cove';
  s.companionName = 'Sparkle';
  s.hero = 'sky';
  const restored = restoreProgress(JSON.stringify(s));
  assert.equal(restored.region, 'cove');
  assert.equal(restored.companionName, 'Sparkle');
  assert.equal(restored.hero, 'sky');
  assert.deepEqual(restored.evidence, s.evidence);
});
test('Invalid and blocked saved positions recover to spawn', () => {
  for (const p of [
    { x: 150, y: 150 },
    { x: 680, y: 350 },
  ]) {
    const s = freshState();
    s.position = p;
    assert.deepEqual(restoreProgress(JSON.stringify(s)).position, freshState().position);
  }
});
test('Routes stay outside obstacles and cross only a repaired bridge', () => {
  const start = { x: 160, y: 490 },
    east = { x: 890, y: 310 };
  assert.equal(findRoute(start, east, false).length, 0);
  const path = findRoute(start, east, true);
  assert.ok(path.length > 10);
  path.forEach((p) => assert.equal(isBlocked(p.x, p.y, true), false));
  for (let i = 1; i < path.length; i++) {
    const a = path[i - 1],
      b = path[i];
    for (let n = 0; n <= 4; n++)
      assert.equal(
        isBlocked(a.x + ((b.x - a.x) * n) / 4, a.y + ((b.y - a.y) * n) / 4, true),
        false,
      );
  }
  assert.ok(path.some((p) => p.x > 640 && p.x < 720 && p.y >= 342 && p.y <= 374));
});
test('Every initial quest approach on west bank is reachable', () => {
  for (const target of [
    { x: 555, y: 425 },
    { x: 305, y: 325 },
  ])
    assert.ok(findRoute({ x: 160, y: 490 }, target, false).length > 0);
});

test('New bird and bunny heroes survive save restoration while unknown heroes fall back', () => {
  for (const hero of ['poppy', 'wren', 'clover', 'mochi']) {
    const state = freshState();
    state.hero = hero;
    assert.equal(restoreProgress(JSON.stringify(state)).hero, hero);
  }
  for (const hero of ['unknown', 'constructor', '__proto__']) {
    assert.equal(restoreProgress(JSON.stringify({ ...freshState(), hero })).hero, 'rose');
  }
});

test('Generated meadow collision matches clubhouse and open lawns', () => {
  assert.equal(isBlocked(180, 150, true), true);
  assert.equal(isBlocked(432, 200, true), false);
  for (const y of [100, 200, 300, 450, 550]) assert.equal(isBlocked(690, y, true), true);
  assert.equal(isBlocked(690, 352, false), true);
  assert.equal(isBlocked(690, 352, true), false);
  assert.ok(findRoute({ x: 895, y: 310 }, { x: 305, y: 325 }, true).length > 0);
});

test('Untrusted save shapes cannot enter typed progress', () => {
  for (const raw of [null, 'null', '[]', 'true', '42', '"save"']) {
    assert.deepEqual(restoreProgress(raw), freshState());
  }
  const valid = completeEncounter(freshState(), bridge, session);
  const restored = restoreProgress(
    JSON.stringify({
      ...valid,
      done: [null, {}, 42, bridge.id],
      evidence: [null, [], false, {}, valid.evidence[0]],
      position: { x: '160', y: 490 },
    }),
  );
  assert.deepEqual(restored.done, [bridge.id]);
  assert.deepEqual(restored.evidence, valid.evidence);
  assert.deepEqual(restored.position, freshState().position);
  assert.equal(readProgress(undefined).available, false);
  assert.equal(writeProgress(freshState(), undefined), false);
});
