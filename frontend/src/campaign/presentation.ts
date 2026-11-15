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
  flags: string[] = [],
  starter?: string | null,
) {
  const uncompleted = actions.filter(
    (action) =>
      !(action.puzzle && flags.includes(`puzzle:${action.puzzle.id}:solved`)) &&
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
    (!flags.some((flag) => /^practised_(pip|tansy|nib)$/.test(flag))
      ? (uncompleted.find((action) => action.id === `practice_${starter}`) ??
        uncompleted.find((action) => action.mission === 'welcome' && action.kind === 'practice'))
      : undefined) ??
    uncompleted.find((action) =>
      ['field-bloom', 'field-resonate', 'field-illuminate'].includes(action.mission),
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

/** Child-facing context, separate from immutable saved campaign definitions. */
export function missionBrief(flags: string[], mission = '') {
  const chapters = [
    {
      key: 'welcome',
      done: 'welcome_complete',
      title: 'Follow the missing moonlight',
      story:
        'The lights that guide our neighbours home are fading. Hazel needs your help to find out why.',
      goal: 'Meet Hazel, choose a travelling friend and get ready to investigate the bridge.',
    },
    {
      key: 'bridge',
      done: 'bridge_seal',
      title: 'Make the bridge safe',
      story:
        'The bridge is broken and its guiding light has gone out. A brass cart has left clues beside the empty moonstone mounts.',
      goal: 'Look for clues, gather wood and rebuild the crossing. Then steady the bridge and light its beacon.',
    },
    {
      key: 'bloom',
      done: 'bloom_seal',
      title: 'Bring water back to the garden',
      story:
        'The garden is drying out. Something has changed the water channels near the moonlight collector.',
      goal: 'Follow the water, reconnect the channels and restore the garden’s beacon.',
    },
    {
      key: 'echo',
      done: 'echo_seal',
      title: 'Find the watchtower’s warning',
      story:
        'Strange echoes lead towards the old watchtower. Its chimes may explain what the collectors are doing.',
      goal: 'Follow the clues with Flutter, arrange the chimes and restore the watchtower light.',
    },
    {
      key: 'harvest',
      done: 'harvest_seal',
      title: 'Get the mill working again',
      story:
        'The orchard’s deliveries have stopped. Cleo needs help finding the missing gear and getting the mill turning.',
      goal: 'Recover the gear, repair the mill and restore its beacon.',
    },
    {
      key: 'current',
      done: 'current_seal',
      title: 'Make the river safe',
      story:
        'Water is flowing the wrong way. Rowan and his partner need help before anyone can use the river route.',
      goal: 'Help the stranded friends, open the bypass and restore the landing light.',
    },
    {
      key: 'tide',
      done: 'tide_seal',
      title: 'Protect the beach refuge',
      story:
        'A mysterious glow has appeared near the refuge. Tracks and pipes can help you discover what is really happening.',
      goal: 'Investigate the glow and steady the refuge so its animals have a safe home.',
    },
    {
      key: 'beacon',
      done: 'beacon_seal',
      title: 'Light the way to the Observatory',
      story:
        'The lighthouse needs its missing lens. Sail can help you find a way to bring its guiding beam back.',
      goal: 'Recover and align the lens, then follow the light towards the Observatory.',
    },
    {
      key: 'accord',
      done: 'accord_seal',
      title: 'Bring the moonlight home',
      story:
        'The collectors have been storing light instead of returning it. Your clues can help Vell change the machines.',
      goal: 'Reconnect the beacon lines and return the stored light to every region.',
    },
  ];
  return (
    chapters.find((chapter) => mission === chapter.key || mission.startsWith(chapter.key + '-')) ??
    chapters.find((chapter) => !flags.includes(chapter.done)) ?? {
      title: 'Celebrate with your friends',
      story: 'The eight beacons can shine together again.',
      goal: flags.includes('finale_complete')
        ? 'Explore optional discoveries and your friends’ invitations.'
        : 'Join your friends for the Moonlight Gathering.',
    }
  );
}

export function campaignPages(hasStarter: boolean, inTrial: boolean) {
  if (!hasStarter || inTrial) return [['missions', inTrial ? 'Current trial' : 'Your adventure']];
  return [
    ['missions', 'Your mission'],
    ['more', 'Friends & more'],
  ];
}

export function openingDialogue(id: string) {
  const dialogue: Record<string, { speaker: string; text: string }[]> = {
    bridge_planks: [
      {
        speaker: 'Woodcutter',
        text: 'You found all three fallen branches. I’ll cut two short planks and one long plank. The long one will span the wide middle gap.',
      },
      {
        speaker: 'Woodcutter',
        text: 'Take these to the crossing. Fit the lengths carefully, then brace both banks before anyone walks over.',
      },
    ],
    bridge_build: [
      {
        speaker: 'Hazel',
        text: 'Let’s carry the cut planks and my braces to the crossing. We have the materials; now we need to fit them together.',
      },
    ],
    bridge_trial: [
      {
        speaker: 'Hazel',
        text: 'The traveller is waiting on this bank. Choose a plank, place it in a matching gap, and brace both ends. When the bridge is secure, invite the traveller across.',
      },
    ],
    welcome_meet: [
      {
        speaker: 'Hazel',
        text: 'Our beacons are the lights that guide everyone home. The bridge light is fading, and I need your help to find out why.',
      },
      {
        speaker: 'Hazel',
        text: 'First choose one friend: Pip finds hidden clues, Tansy helps plants grow, and Nib builds and steadies things. Any of them can help you finish this adventure.',
      },
      {
        speaker: 'Hazel',
        text: 'We will pack for the trip and try one short teamwork practice here. The other practices are optional. Then we will look for clues at the bridge, repair the crossing and bring its light back.',
      },
    ],
    welcome_supplies: [
      {
        speaker: 'Hazel',
        text: 'We’re getting ready to investigate the dark bridge. Help me with a short activity before we set off.',
      },
      {
        speaker: 'Hazel',
        text: 'You can ask for a hint or choose “Let’s do it together”. We will still finish packing and move on with the adventure.',
      },
    ],
    welcome_depart: [
      {
        speaker: 'Hazel',
        text: 'The bridge light has gone out! A brass cart has just left the bank, but we need clues before we decide what happened.',
      },
      {
        speaker: 'Rowan',
        text: 'Moss and I saw the cart go east. We will meet you along the trail.',
      },
      {
        speaker: 'Hazel',
        text: 'First inspect the empty moonstone mounts beside the broken bridge. Then we will collect wood, rebuild the crossing and light the beacon.',
      },
    ],
  };
  return dialogue[id];
}

/** A direction hint only. The server's current travel list authorizes the move. */
export function nextTrail(flags: string[], location: string, travel: string[]) {
  const target = !flags.includes('bridge_seal')
    ? 'bridge_bank'
    : !flags.includes('bloom_seal')
      ? 'flower_garden'
      : !flags.includes('echo_seal')
        ? flags.includes('echo_clue_complete')
          ? 'old_watchtower'
          : 'echo_hollow'
        : !flags.includes('current_seal') &&
            (location === 'courier_pond' || flags.includes('harvest_seal'))
          ? 'courier_pond'
          : !flags.includes('harvest_seal')
            ? 'old_mill'
            : !flags.includes('tide_seal')
              ? 'shell_beach'
              : !flags.includes('beacon_seal')
                ? 'harbour_lookout'
                : !flags.includes('finale_complete')
                  ? 'observatory'
                  : undefined;
  if (!target || target === location) return undefined;
  const links: Record<string, string[]> = {
    bridge_bank: ['picnic_clearing', 'flower_garden', 'bridge_island'],
    picnic_clearing: ['bridge_bank', 'flower_garden'],
    flower_garden: ['bridge_bank', 'picnic_clearing', 'echo_hollow'],
    echo_hollow: ['flower_garden', 'old_watchtower'],
    old_watchtower: ['echo_hollow', 'orchard_hub'],
    orchard_hub: ['old_watchtower', 'old_mill', 'courier_pond'],
    old_mill: ['orchard_hub', 'garland_ridge'],
    garland_ridge: ['old_mill', 'courier_pond'],
    courier_pond: ['orchard_hub', 'garland_ridge', 'cove_hub'],
    bridge_island: ['bridge_bank'],
    cove_hub: ['courier_pond', 'shell_beach', 'harbour_lookout'],
    shell_beach: ['cove_hub', 'hidden_tide_garden'],
    harbour_lookout: ['cove_hub', 'observatory'],
    observatory: ['harbour_lookout'],
    hidden_tide_garden: ['shell_beach', 'star_island'],
    star_island: ['hidden_tide_garden'],
  };
  const queue = travel.map((step) => ({ step, at: step }));
  const visited = new Set([location]);
  for (const route of queue) {
    if (route.at === target) return { step: route.step, target };
    if (visited.has(route.at)) continue;
    visited.add(route.at);
    for (const at of links[route.at] ?? []) queue.push({ step: route.step, at });
  }
  return undefined;
}
