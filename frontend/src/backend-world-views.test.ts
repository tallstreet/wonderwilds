import { test } from 'node:test';
import assert from 'node:assert/strict';
import type { GameSnapshot } from '../../contracts/generated/contracts.ts';
import { journalView, missionsView, trainingView } from './backend-world-views.ts';

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
test('Journal uses stable creature IDs and escapes server inventory text without changing state', () => {
  const state = blank();
  state.ownedCreatures = ['pip', 'tansy'];
  state.selectedCompanions = ['pip'];
  state.teamCapacity = 1;
  state.inventory = [{ itemId: '<img src=x>', quantity: 1 }];
  const before = structuredClone(state);
  const html = journalView(state, { hero: 'rose' });
  assert.match(html, /data-companion-id="pip"/);
  assert.match(html, /data-companion-id="tansy"/);
  assert.match(html, /data-server-companion="tansy" disabled/);
  assert.match(html, /Rest Pip/);
  assert.match(html, /&lt;img src=x&gt;/);
  assert.doesNotMatch(html, /<img src=x>/);
  assert.deepEqual(state, before);
});
test('View extraction preserves locked missions and unowned training guidance', () => {
  const state = blank();
  const missions = missionsView(state);
  assert.match(missions, /data-server-visit="meadow-bridge" disabled/);
  assert.doesNotMatch(missions, /data-server-visit="meadow-welcome" disabled/);
  assert.match(trainingView(state), /Befriend a Meadow character/);
  assert.doesNotMatch(trainingView(state), /data-server-training=/);
});

test('Missions identify the destination of activities and training', () => {
  const html = missionsView(blank());
  for (const name of [
    'Clover Meadow',
    'Starlight Cove',
    'Peachy Orchard',
    'Fern Hollow',
    'Mossy Clearing',
    'Owl Grove',
  ]) {
    assert.ok(html.includes(name), name);
  }
});
