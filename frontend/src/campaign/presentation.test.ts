import assert from 'node:assert/strict';
import test from 'node:test';
import {
  adventureUrl,
  earlierAdventureRequested,
  canVisitEarlier,
  beaconFlags,
  companionAffinities,
  signatureCost,
  encounterPhase,
  journalHistory,
  savedPuzzles,
} from './presentation.ts';
test('signature costs follow affinity including late friends', () => {
  for (const id of ['luna', 'sail']) assert.equal(signatureCost(companionAffinities[id]!), 1);
  for (const id of ['cleo', 'patter']) assert.equal(signatureCost(companionAffinities[id]!), 2);
});
test('phase display changes at exact authored threshold', () => {
  const board = {
    id: 'test',
    target: 12,
    turn_limit: 12,
    pressure_limit: 12,
    hazards: [1],
    affinity: 'gleam',
    win_flags: [],
    phases: [
      { threshold: 4, affinity: 'growth' },
      { threshold: 8, affinity: 'tide' },
    ],
  };
  assert.equal(encounterPhase(board, 3).affinity, 'gleam');
  assert.equal(encounterPhase(board, 4).affinity, 'growth');
  assert.equal(encounterPhase(board, 4).next?.threshold, 8);
  assert.equal(encounterPhase(board, 8).next, undefined);
});
test('all three puzzles show only saved settings and explicit feedback', () => {
  const puzzles = savedPuzzles([
    'puzzle:garden_water:near:bypass',
    'puzzle:garden_water:blocked',
    'puzzle:tower_chimes:first:two_rings',
    'puzzle:pump_valves:solved',
  ]);
  assert.equal(puzzles.length, 3);
  assert.equal(puzzles[0]?.status, 'blocked');
  assert.deepEqual(puzzles[0]?.settings, [{ setting: 'near', value: 'bypass' }]);
  assert.equal(puzzles[2]?.status, 'solved');
});
test('journal excludes internal flags and only eight authored sites have beacons', () => {
  assert.equal(Object.keys(beaconFlags).length, 8);
  assert.equal(beaconFlags.echo_hollow, undefined);
  assert.equal(beaconFlags.old_watchtower, 'world_tower_restored');
  assert.deepEqual(
    journalHistory(['last_camp:x', 'milestone:x', 'appearance:pip:form', 'world_mill_restored']),
    ['Harvest beacon restored: the mill turns again.'],
  );
});
test('entry route lets adopted profiles visit preserved legacy and return without save mutation', () => {
  const original = 'http://127.0.0.1:5173/?other=kept';
  const earlier = adventureUrl(original, true);
  assert.equal(earlierAdventureRequested(earlier), true);
  assert.equal(adventureUrl(earlier, false), original);
  assert.equal(canVisitEarlier(true, false, false), false);
  assert.equal(canVisitEarlier(false, true, false), false);
  assert.equal(canVisitEarlier(false, false, true), false);
  assert.equal(canVisitEarlier(false, false, false), true);
});

test('tracker prioritises active story over starter replays and offers travel after local story finishes', async () => {
  const { nextObjective } = await import('./presentation.ts');
  const task = (
    id: string,
    mission: string,
    kind: string,
    encounter: string | null = null,
  ): import('../../../contracts/generated/contracts.ts').CampaignAction => ({
    id,
    label: id,
    location: 'bridge_bank',
    requires_flags: [],
    forbids_flags: [],
    costs: {},
    rewards: {},
    grants_flags: [],
    encounter,
    recruit: null,
    dialogue: [],
    mission,
    kind,
    grant_id: '',
    learning_context: null,
    learning: false,
    choices: [],
    puzzle: null,
    recipient: null,
    milestone: null,
    field_ability: null,
    field_method: null,
    discovery_for: null,
  });
  const replay = task('practice_pip', 'welcome', 'practice', 'practice_pip'),
    side = task('bridge_island', 'side-island-preview', 'observe'),
    main = task('bridge_gather', 'bridge-project', 'gather');
  assert.equal(nextObjective([replay, side, main], ['win:practice_pip'])?.id, 'bridge_gather');
  assert.equal(nextObjective([replay, side], ['win:practice_pip']), undefined);
  const puzzle = {
    ...task('water_check', 'bloom-project', 'puzzle'),
    puzzle: { id: 'garden_water', operation: 'check', value: null, goal: {} },
  };
  const path = task('field_bloom_tool', 'field-bloom', 'field');
  assert.equal(nextObjective([puzzle, path], [], ['puzzle:garden_water:solved'])?.id, path.id);
  assert.equal(nextObjective([puzzle, path], [])?.id, puzzle.id);
});

test('Wayfinder milestones are shown only for a confirmed crossing with a known baseline, never on reload', async () => {
  const { newlyEarnedWayfinders, changedCompanionAppearance } = await import('./presentation.ts');
  const before = [{ id: 'pip', level: 4, appearance: 'base' as const }],
    after = [{ id: 'pip', level: 5, appearance: 'base' as const }];
  assert.deepEqual(
    newlyEarnedWayfinders(before, after).map((friend) => friend.id),
    ['pip'],
  );
  assert.deepEqual(newlyEarnedWayfinders(undefined, after), []);
  assert.deepEqual(newlyEarnedWayfinders(after, after), []);
  assert.deepEqual(newlyEarnedWayfinders([], after), []);
  assert.equal(after[0]!.appearance, 'base', 'An earned form never changes canonical appearance');
  assert.equal(changedCompanionAppearance(before, after, 'pip'), undefined);
  const chosen = [{ id: 'pip', level: 5, appearance: 'wayfinder' as const }];
  assert.deepEqual(changedCompanionAppearance(after, chosen, 'pip'), {
    id: 'pip',
    before: 'pip',
    after: 'pip-wayfinder',
  });
  assert.deepEqual(changedCompanionAppearance(chosen, after, 'pip'), {
    id: 'pip',
    before: 'pip-wayfinder',
    after: 'pip',
  });
});

test('mission context explains the problem and goal without exposing future answers', async () => {
  const { missionBrief, campaignPages, openingDialogue } = await import('./presentation.ts');
  assert.match(missionBrief([]).story, /lights.*fading/);
  assert.match(missionBrief(['welcome_complete'], 'bridge-project').goal, /rebuild the crossing/);
  assert.match(missionBrief(['welcome_complete', 'bridge_seal']).title, /garden/);
  assert.match(missionBrief(['welcome_complete'], 'current-clue').title, /river/);
  assert.equal(campaignPages(false, false).length, 1);
  assert.deepEqual(campaignPages(true, true), [['missions', 'Current trial']]);
  assert.equal(campaignPages(true, false).length, 2);
  assert.match(openingDialogue('welcome_depart')!.at(-1)!.text, /First inspect/);
});

test('directions use an available path towards the next story destination', async () => {
  const { nextTrail } = await import('./presentation.ts');
  assert.deepEqual(
    nextTrail(['bridge_seal'], 'bridge_bank', ['picnic_clearing', 'flower_garden']),
    { step: 'flower_garden', target: 'flower_garden' },
  );
  assert.deepEqual(
    nextTrail(['bridge_seal', 'bloom_seal'], 'bridge_bank', ['picnic_clearing', 'flower_garden']),
    { step: 'flower_garden', target: 'echo_hollow' },
  );
  assert.equal(nextTrail([], 'bridge_bank', []), undefined);
});
