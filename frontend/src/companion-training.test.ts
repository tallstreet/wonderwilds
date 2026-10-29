import { test } from 'node:test';
import assert from 'node:assert/strict';
import type { CharacterProgression } from '../../contracts/generated/contracts.ts';
import { trainingDetails, trainingLevelUp, trainingProgress } from './companion-training.ts';

const pip = (level = 1): CharacterProgression => ({
  creatureId: 'pip',
  xp: level === 1 ? 0 : 10,
  level,
  nextLevelXp: 20,
  completedTrainingIds: level === 1 ? [] : ['training-pip-1'],
  availableTraining: [
    { trainingId: 'training-pip-1', requiredLevel: 1, xp: 10, reactionId: 'pip-celebration' },
  ],
});
test('Training presents confirmed server progression and cannot recruit or grant XP', () => {
  const state = { ownedCreatures: ['pip'], characterProgression: [pip()] };
  const before = structuredClone(state);
  assert.match(trainingDetails('pip', state), /Level 1 · 0 character XP/);
  assert.match(trainingDetails('pip', state), /data-server-training="training-pip-1"/);
  assert.match(trainingDetails('pip', state), /Train with Pip · 10 XP/);
  assert.equal(trainingProgress('tansy', state), undefined);
  assert.equal(trainingDetails('pip', { ...state, ownedCreatures: [] }), '');
  assert.equal(trainingDetails('pip', { ownedCreatures: ['pip'] }), '');
  assert.deepEqual(state, before);
});
test('Rewarded milestones become practice without repeated XP claims', () => {
  const state = { ownedCreatures: ['pip'], characterProgression: [pip(2)] };
  const html = trainingDetails('pip', state);
  assert.match(html, /Practise with Pip · no extra XP/);
  assert.match(html, /Level 2 · 10 character XP/);
  assert.doesNotMatch(html, /Train with Pip · 10 XP/);
});
test('Celebration requires an authoritative level increase for the bound owned friend', () => {
  const before = { ownedCreatures: ['pip'], characterProgression: [pip()] };
  const after = { ...before, characterProgression: [pip(2)] };
  assert.equal(trainingLevelUp(before, after, 'pip')?.level, 2);
  assert.equal(trainingLevelUp(after, after, 'pip'), undefined);
  assert.equal(trainingLevelUp(before, after, 'tansy'), undefined);
  assert.equal(trainingLevelUp(before, { ...after, ownedCreatures: [] }, 'pip'), undefined);
});
test('Training identifiers render as text and escaped attributes', () => {
  const progress = pip();
  progress.availableTraining[0].trainingId = '"><img src=x onerror=alert(1)>';
  const html = trainingDetails('pip', {
    ownedCreatures: ['pip'],
    characterProgression: [progress],
  });
  assert.doesNotMatch(html, /<img/);
  assert.match(html, /&quot;&gt;&lt;img/);
});
