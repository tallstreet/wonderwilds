import assert from 'node:assert/strict';
import test from 'node:test';
import pack from '../../../content/campaign/runtime.v1.json';
import { challengeStory, storySoFar } from './story.ts';
import { nextObjective } from './presentation.ts';
import type { CampaignAction } from '../../../contracts/generated/contracts.ts';

test('every authored challenge has a specific objective, progress label and aftermath', () => {
  const titles = new Set<string>();
  for (const encounter of pack.encounters) {
    const story = challengeStory(encounter.id);
    assert.notEqual(story.title, 'A challenge together', encounter.id);
    assert.ok(story.goal.length > 40);
    assert.ok(story.outcome.length > 40);
    assert.ok(!titles.has(story.title), encounter.id);
    titles.add(story.title);
  }
  assert.equal(challengeStory('trial_bridge').progress, 'Crossing secured');
  assert.equal(challengeStory('trial_bloom').progress, 'Seedlings sheltered');
});
test('story recap uses saved discoveries and does not reveal later chapters', () => {
  assert.equal(storySoFar([]), undefined);
  assert.match(storySoFar(['bridge_seal'])!, /garden/);
  assert.doesNotMatch(storySoFar(['bridge_seal'])!, /Vell|Observatory/);
  assert.match(storySoFar(['echo_seal', 'current_seal'])!, /Rowan/);
  assert.match(storySoFar(['echo_seal', 'harvest_seal'])!, /ledger/);
  assert.match(storySoFar(['beacon_seal', 'finale_complete'])!, /constellation/);
});
test('only one tutorial is promoted; remaining introductions are optional', () => {
  const actions = pack.actions.filter(
    (action) => action.kind === 'practice',
  ) as unknown as CampaignAction[];
  assert.equal(nextObjective(actions, [], [], 'tansy')?.id, 'practice_tansy');
  assert.equal(nextObjective(actions, [], ['practised_tansy'], 'tansy'), undefined);
  const departure = pack.actions.find(
    (action) => action.id === 'welcome_depart',
  ) as unknown as CampaignAction;
  assert.equal(
    nextObjective([...actions, departure], [], ['practised_tansy'], 'tansy')?.id,
    'welcome_depart',
  );
});
