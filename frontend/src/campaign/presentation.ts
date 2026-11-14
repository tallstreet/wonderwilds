import type { CampaignEncounterDefinition } from '../../../contracts/generated/contracts.ts';
export const beaconFlags: Record<string, string> = {
  bridge_bank: 'world_bridge_restored',
  flower_garden: 'world_garden_restored',
  old_watchtower: 'world_tower_restored',
  old_mill: 'world_mill_restored',
  courier_pond: 'world_river_restored',
  shell_beach: 'world_refuge_restored',
  harbour_lookout: 'world_lighthouse_restored',
  observatory: 'world_moonlight_returned',
};
export const companionAffinities: Record<string, string> = {
  pip: 'gleam',
  luna: 'gleam',
  glimmer: 'gleam',
  tansy: 'growth',
  cleo: 'growth',
  nib: 'craft',
  bramble: 'craft',
  flutter: 'rhythm',
  twizzle: 'rhythm',
  sail: 'rhythm',
  patter: 'tide',
  pebble: 'tide',
};
export function signatureCost(affinity: string) {
  return affinity === 'growth' || affinity === 'tide' ? 2 : 1;
}
export function encounterPhase(board: CampaignEncounterDefinition, progress: number) {
  const passed = board.phases.filter((p) => p.threshold <= progress);
  return {
    affinity: passed.at(-1)?.affinity ?? board.affinity,
    next: board.phases.find((p) => p.threshold > progress),
  };
}
export function savedPuzzles(flags: string[]) {
  return [
    { id: 'garden_water', title: 'Garden water channels' },
    { id: 'tower_chimes', title: 'Watchtower chimes' },
    { id: 'pump_valves', title: 'River pump valves' },
  ].flatMap((puzzle) => {
    const values = flags
      .filter((f) => f.startsWith(`puzzle:${puzzle.id}:`))
      .map((f) => f.split(':').slice(2));
    if (!values.length) return [];
    return [
      {
        ...puzzle,
        settings: values
          .filter((v) => v.length === 2)
          .map((v) => ({ setting: v[0]!, value: v[1]! })),
        status: values.some((v) => v[0] === 'solved')
          ? 'solved'
          : values.some((v) => v[0] === 'blocked')
            ? 'blocked'
            : 'adjusting',
      },
    ];
  });
}
const history: Record<string, string> = {
  starter_chosen: 'Your first companion joined the trail.',
  world_bridge_planked: 'You laid the bridge crossing.',
  world_bridge_restored: 'Bridge beacon restored: a safe path opened.',
  world_bridge_alcove_open: 'You found the bridge supply alcove.',
  world_garden_restored: 'Bloom beacon restored: water reaches the seedlings.',
  world_tower_restored: 'Echo beacon restored: the watchtower sings again.',
  world_mill_restored: 'Harvest beacon restored: the mill turns again.',
  world_river_restored: 'Current beacon restored: river routes are safe.',
  world_refuge_restored: 'Tide beacon restored: the beach refuge is sheltered.',
  world_lighthouse_restored: 'Beacon light restored: the lighthouse guides the way.',
  world_moonlight_returned: 'Accord beacon restored: moonlight returns to every region.',
  collectors_changed: 'Vell’s collectors now return light to the living world.',
  finale_complete: 'You celebrated the constellation with your friends.',
  glimmer_recruited: 'Glimmer found a home with your friends.',
};
export function journalHistory(flags: string[]) {
  return Object.entries(history)
    .filter(([flag]) => flags.includes(flag))
    .map(([, text]) => text);
}
export function adventureUrl(current: string, earlier: boolean) {
  const url = new URL(current);
  if (earlier) url.searchParams.set('adventure', 'earlier');
  else url.searchParams.delete('adventure');
  return url.href;
}
export function earlierAdventureRequested(current: string) {
  return new URL(current).searchParams.get('adventure') === 'earlier';
}
export function canVisitEarlier(
  pending: boolean,
  learningPending: boolean,
  activeActivity: boolean,
) {
  return !pending && !learningPending && !activeActivity;
}

export function nextObjective(
  actions: import('../../../contracts/generated/contracts.ts').CampaignAction[],
  grants: string[],
) {
  const uncompleted = actions.filter(
    (action) =>
      !grants.includes(`action:${action.id}`) &&
      !(action.encounter && grants.includes(`win:${action.encounter}`)),
  );
  return (
    uncompleted.find(
      (action) =>
        /^(welcome|bridge-|bloom-|echo-|harvest-|current-|tide-|beacon-|accord-|constellation-ending)/.test(
          action.mission,
        ) && action.kind !== 'practice',
    ) ??
    uncompleted.find(
      (action) =>
        action.kind !== 'practice' &&
        !/^(side-|growth-|friendship-|habitat-|exchange-|field-|postgame)/.test(action.mission),
    )
  );
}

/** Compare confirmed public projections only; absence of a baseline never replays a milestone. */
export function newlyEarnedWayfinders(
  before: import('../../../contracts/generated/contracts.ts').CampaignCompanion[] | undefined,
  after: import('../../../contracts/generated/contracts.ts').CampaignCompanion[],
) {
  if (!before) return [];
  return after.filter(
    (friend) =>
      friend.level >= 5 &&
      before.some((previous) => previous.id === friend.id && previous.level < 5),
  );
}
export function changedCompanionAppearance(
  before: import('../../../contracts/generated/contracts.ts').CampaignCompanion[],
  after: import('../../../contracts/generated/contracts.ts').CampaignCompanion[],
  id: string,
) {
  const previous = before.find((friend) => friend.id === id),
    current = after.find((friend) => friend.id === id);
  if (!previous || !current || (previous.appearance ?? 'base') === (current.appearance ?? 'base'))
    return undefined;
  return {
    id,
    before: (previous.appearance ?? 'base') === 'wayfinder' ? id + '-wayfinder' : id,
    after: (current.appearance ?? 'base') === 'wayfinder' ? id + '-wayfinder' : id,
  };
}
