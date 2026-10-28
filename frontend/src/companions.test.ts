import type { CompanionProgress } from './companions.ts';
import { test } from 'node:test';
import assert from 'node:assert/strict';
import development from '../../content/creatures/development.v1.json';
import { companionDetails, companionProfile, companionProgress } from './companions.ts';
import { meadowFriends } from './backend-world-model.ts';

test('Every Meadow friend has a distinct authored journal profile', () => {
  const descriptions = new Set<string>();
  for (const quest of meadowFriends) {
    const profile = companionProfile(quest.creatureId);
    assert.ok(profile);
    descriptions.add(profile.personality);
    const html = companionDetails(quest.creatureId, false);
    assert.ok(html.includes('<summary>Meet ' + profile.name + '</summary>'));
    assert.ok(html.includes(profile.habitatClue));
    assert.ok(!html.includes('Your friend'));
  }
  assert.equal(descriptions.size, meadowFriends.length);
});

test('Confirmed friends show free support without manufacturing progression', () => {
  const html = companionDetails('pip', true);
  assert.ok(html.includes('free helper tray'));
  assert.ok(html.includes('Help neighbours find their way.'));
  assert.ok(!html.includes('We found the way by looking together.'));
  assert.ok(html.includes('still earn story rewards'));
  assert.ok(!html.includes('Habitat clue'));
  assert.ok(!html.includes('pip-glow'));
  assert.ok(!html.includes('friendship points'));
  assert.equal(companionDetails('unknown', true), '');
});

test('Dialogue and reversible glow use only confirmed ownership, points and flags', () => {
  const state: CompanionProgress = {
    ownedCreatures: ['pip'],
    storyFlags: [],
    friendship: [{ creatureId: 'pip', points: 3 }],
  };
  assert.equal(companionProgress('pip', state)?.canTransform, false);
  assert.equal(
    companionProgress('pip', state)?.dialogue,
    development.characters[0].friendshipMoments[1].dialogue,
  );
  state.storyFlags.push('meadow_light');
  assert.equal(companionProgress('pip', state)?.canTransform, true);
  assert.equal(
    companionProgress('pip', state)?.dialogue,
    development.characters[0].friendshipMoments[2].dialogue,
  );
  assert.match(companionDetails('pip', true, state), /Turn glow on/);
  const glowing = { ...state, transformedCreatures: ['pip-glow'] };
  assert.match(companionDetails('pip', true, glowing), /aria-pressed="true"/);
  assert.match(companionDetails('pip', true, glowing), /Turn glow off/);
  assert.equal(companionProgress('pip', { ...state, ownedCreatures: [] }), undefined);
  assert.equal(companionProgress('missing', state), undefined);
});

test('Absent or invalid friendship fields never fabricate progress or unlock dialogue', () => {
  const state = { ownedCreatures: ['pip'], storyFlags: ['meadow_light'] };
  assert.equal(companionProgress('pip', state)?.points, undefined);
  assert.equal(companionProgress('pip', state)?.dialogue, undefined);
  assert.equal(companionProgress('pip', state)?.canTransform, false);
  assert.doesNotMatch(
    companionDetails('pip', true, state),
    /Turn glow on|Friendship:|Remember together/,
  );
  for (const points of [-1, 1.5, NaN, 7]) {
    assert.equal(
      companionProgress('pip', { ...state, friendship: [{ creatureId: 'pip', points }] })?.points,
      undefined,
    );
  }
});

test('Available memories resolve known authored quests and rendering does not mutate server state', () => {
  const state = {
    ownedCreatures: ['pip'],
    storyFlags: [],
    friendship: [{ creatureId: 'pip', points: 1 }],
    availableHelpingMoments: [
      { creatureId: 'pip', completedQuestId: 'meadow-bridge' },
      { creatureId: 'pip', completedQuestId: '<script>alert(1)</script>' },
      { creatureId: 'tansy', completedQuestId: 'meadow-garden' },
    ],
  };
  const before = structuredClone(state);
  const html = companionDetails('pip', true, state);
  assert.equal(companionProgress('pip', state)?.moments.length, 1);
  assert.match(html, /data-completed-quest="meadow-bridge"/);
  assert.doesNotMatch(html, /<script>|data-completed-quest="meadow-garden"/);
  assert.deepEqual(state, before);
  const profile = companionProfile('pip')!;
  const original = profile.personality;
  try {
    profile.personality = '<img src=x onerror="alert(1)">';
    const escaped = companionDetails('pip', true, state);
    assert.ok(escaped.includes('&lt;img src=x onerror=&quot;alert(1)&quot;&gt;'));
    assert.doesNotMatch(escaped, /<img/);
  } finally {
    profile.personality = original;
  }
});
