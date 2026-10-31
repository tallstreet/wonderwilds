import { items } from '../items.ts';
import type { StoryAction, StorySnapshot } from '../../../contracts/generated/contracts.ts';
import { exitPoint } from '../maps.ts';
import type { Point, WorldEncounter, StoryShape, WorldDecoration } from '../types.ts';

export const storyPlaces = {
  bridge_bank: { region: 'meadow', name: 'Pip’s bridge · near bank', spawn: { x: 160, y: 490 } },
  workshop: { region: 'orchard', name: 'The woodcutter’s workshop', spawn: { x: 850, y: 490 } },
  far_bank: { region: 'meadow', name: 'Pip’s bridge · far bank', spawn: { x: 850, y: 359 } },
  cove: { region: 'cove', name: 'Starlight Cove', spawn: { x: 160, y: 330 } },
  fern_hollow: { region: 'explore-0-0', name: 'Fern Hollow', spawn: { x: 160, y: 345 } },
  mossy_clearing: { region: 'explore-0-1', name: 'Mossy Clearing', spawn: { x: 160, y: 345 } },
  owl_grove: { region: 'explore-0-2', name: 'Owl Grove', spawn: { x: 160, y: 345 } },
  acorn_trail: { region: 'explore-0-3', name: 'Acorn Trail', spawn: { x: 160, y: 345 } },
} as const;
/** Mission illustrations stay visible even when the supplies are in another area. */
export function missionSupplyIds(snapshot: StorySnapshot): string[] {
  const authored: Record<string, string[]> = {
    pips_bridge: ['wood', 'planks'],
    pips_picnic: ['berries', 'picnic_basket'],
    welcome_lights: ['glow_pebbles', 'lanterns'],
    humming_hollow: ['wind_instrument'],
  };
  const mission = snapshot.sceneInput?.mission;
  const ids = new Set<string>(
    mission && mission.status !== 'completed' ? (authored[mission.missionId ?? ''] ?? []) : [],
  );
  for (const { action } of snapshot.sceneInput?.availableActions ?? []) {
    if (action.type === 'collect' && !action.sourceId.startsWith('find_')) ids.add(action.itemId);
    if (action.type === 'exchange') {
      const outputs: Record<string, string> = {
        wood_to_planks: 'planks',
        berries_to_picnic: 'picnic_basket',
        glow_to_lanterns: 'lanterns',
      };
      if (outputs[action.recipeId]) ids.add(outputs[action.recipeId]);
    }
  }
  const projectItems: Record<string, string[]> = {
    gather_wood: ['wood'],
    craft_planks: ['wood', 'planks'],
    build_seat: ['planks'],
    deliver_planks: ['planks'],
    gather_berries: ['berries'],
    pack_basket: ['berries', 'picnic_basket'],
    deliver_basket: ['picnic_basket'],
    gather_glow_pebbles: ['glow_pebbles'],
    craft_lantern: ['glow_pebbles', 'lanterns'],
    place_lantern: ['lanterns'],
  };
  for (const q of snapshot.sceneInput?.neighbourMissions ?? []) {
    if (q.status !== 'open') continue;
    for (const step of q.generatedStory?.steps ?? [])
      for (const item of projectItems[step.mechanic ?? 'inspect'] ?? []) ids.add(item);
  }
  return [...ids];
}
export const neighbours: Record<string, { name: string; sprite: string }> = {
  marina: { name: 'Marina', sprite: 'sail' },
  fern: { name: 'Fern', sprite: 'flutter' },
  moss: { name: 'Moss', sprite: 'nib' },
  orla: { name: 'Orla', sprite: 'luna' },
  hazel: { name: 'Hazel', sprite: 'patter' },
};
const trail = ['workshop', 'cove', 'fern_hollow', 'mossy_clearing', 'owl_grove', 'acorn_trail'];
export function storyRoutePoint(from: StorySnapshot['locationId'], to: string): Point {
  if (from === 'bridge_bank' && to === 'workshop') return exitPoint('meadow', 'west');
  if (from === 'workshop' && to === 'bridge_bank') return exitPoint('orchard', 'east');
  if (trail.includes(from) && trail.includes(to)) {
    const forward = trail.indexOf(to) > trail.indexOf(from);
    // The workshop's eastern path already returns to Pip.
    return exitPoint(storyPlaces[from].region, from === 'workshop' || !forward ? 'west' : 'east');
  }
  return from === 'far_bank' ? { x: 810, y: 359 } : { x: 570, y: 359 };
}
/** Walking only commits a currently offered route; never invents an exit. */
export function walkingStoryExit(snapshot: StorySnapshot, point: Point) {
  return snapshot.sceneInput?.availableActions.find(({ action }) => {
    if (action.type !== 'travel') return false;
    const target = storyRoutePoint(snapshot.locationId, action.destinationId);
    return (
      (target.x < 65 || target.x > 1055) && Math.hypot(target.x - point.x, target.y - point.y) < 18
    );
  });
}
export function storyBounds(location: StorySnapshot['locationId'], crossing = false) {
  return (x: number, _y: number) =>
    !crossing && (location === 'bridge_bank' ? x > 600 : location === 'far_bank' ? x < 780 : false);
}
export function bridgeRepaired(snapshot: StorySnapshot) {
  return snapshot.sceneInput?.context.memory.facts.includes('The bridge is repaired.') ?? false;
}
export function actionTarget(action: StoryAction): string {
  switch (action.type) {
    case 'accept_mission':
    case 'dismiss':
    case 'cancel_mission':
      return 'npc:pip';
    case 'collect':
      return `resource:${action.sourceId}`;
    case 'exchange':
      return `npc:${action.characterId}`;
    case 'assemble':
    case 'inspect':
      return `object:${action.objectId}`;
    case 'choose':
      return `object:${action.choiceId}`;
    case 'talk':
    case 'invite':
    case 'accompany':
      return `npc:${action.characterId}`;
    case 'travel':
      return `route:${action.destinationId}`;
  }
}
/** Resolve only the saved mission's current action target, including its generation. */
export function neighbourForAction(snapshot: StorySnapshot, action: StoryAction) {
  const target = actionTarget(action).split(':').slice(1).join(':');
  return snapshot.sceneInput?.neighbourMissions?.find((q) => {
    if (q.status !== 'open' || q.inspected || neighbourDestination(q) !== snapshot.locationId)
      return false;
    if (!q.generatedStory)
      return action.type === 'inspect' && target === `neighbour_${q.characterId}`;
    const project = q.generatedStory.steps.some(
      (step) => step.mechanic && step.mechanic !== 'inspect',
    );
    const id = project
      ? `neighbour_${q.characterId}_g${q.generation}_${q.stepIndex ?? 0}`
      : `neighbour_${q.characterId}_${q.stepIndex ?? 0}`;
    return target === id;
  });
}
/** Authored display placement, never an alternative source of legal actions or progress. */
export function storyMarkers(snapshot: StorySnapshot): WorldEncounter[] {
  const place = storyPlaces[snapshot.locationId];
  const markers: WorldEncounter[] = [];
  const add = (
    id: string,
    name: string,
    point: Point,
    art: { sprite?: string; prop?: string; shape?: StoryShape },
  ) =>
    markers.push({
      id,
      short: name,
      character: name,
      region: place.region,
      x: point.x,
      y: point.y,
      sprite: art.sprite,
      prop: art.prop,
      shape: art.shape,
      itemId:
        (
          {
            stones: 'glow_pebbles',
            lantern: 'lanterns',
            instrument: 'wind_instrument',
            leaf: 'fern_leaf',
          } as Record<string, string>
        )[art.shape ?? ''] ??
        (art.prop === 'plank' ? 'planks' : art.prop === 'berry' ? 'berries' : undefined),
    });
  for (const character of snapshot.sceneInput?.here.characters ?? []) {
    const generated = snapshot.sceneInput?.neighbourMissions?.find(
      (q) => q.characterId === character.id,
    )?.generatedStory;
    const neighbour = generated
      ? { name: generated.name, sprite: generated.sprite }
      : neighbours[character.id];
    if (neighbour) add(`npc:${character.id}`, neighbour.name, { x: 530, y: 400 }, neighbour);
    if (character.id === 'pip')
      add(
        'npc:pip',
        'Pip',
        snapshot.locationId === 'far_bank' ? { x: 890, y: 420 } : { x: 530, y: 405 },
        { sprite: 'pip' },
      );
    // Existing art is reused, but the woodcutter and Bramble must remain visually distinct.
    if (character.id === 'woodcutter')
      add('npc:woodcutter', 'Woodcutter', { x: 470, y: 380 }, { sprite: 'cleo' });
    if (character.id === 'bramble')
      add(
        'npc:bramble',
        'Bramble',
        snapshot.locationId === 'workshop'
          ? { x: 650, y: 350 }
          : snapshot.locationId === 'far_bank'
            ? { x: 1010, y: 420 }
            : { x: 430, y: 405 },
        { sprite: 'bramble' },
      );
  }
  const actions = snapshot.sceneInput?.availableActions ?? [];
  for (const { action } of actions) {
    const id = actionTarget(action);
    if (markers.some((marker) => marker.id === id)) continue;
    const neighbour = neighbourForAction(snapshot, action);
    if (neighbour?.generatedStory) {
      const index = neighbour.stepIndex ?? 0;
      const step = neighbour.generatedStory.steps[index];
      const points = [
        { x: 780, y: 460 },
        { x: 420, y: 540 },
        { x: 900, y: 300 },
        { x: 730, y: 530 },
        { x: 350, y: 330 },
        { x: 880, y: 550 },
      ];
      const output: Record<string, string> = {
        gather_wood: 'wood',
        gather_berries: 'berries',
        gather_glow_pebbles: 'glow_pebbles',
        craft_planks: 'planks',
        pack_basket: 'picnic_basket',
        craft_lantern: 'lanterns',
        deliver_basket: 'picnic_basket',
        deliver_planks: 'planks',
        build_seat: 'planks',
        place_lantern: 'lanterns',
      };
      const item = output[step.mechanic ?? 'inspect'];
      const art = item
        ? { prop: items[item].prop }
        : { shape: step.mechanic === 'choose_style' ? ('corner-leaves' as const) : step.shape };
      add(id, step.label, points[index % points.length], art);
      if (item) markers[markers.length - 1].itemId = item;
      continue;
    }
    if (action.type === 'collect') {
      const source = resourcePlaces[action.sourceId];
      if (source && source.location === snapshot.locationId) {
        add(id, source.name, source, source);
        markers[markers.length - 1].itemId = action.itemId;
      }
    }
    if (action.type === 'assemble' || action.type === 'inspect' || action.type === 'choose') {
      const object = assemblyPlaces[action.type === 'choose' ? action.choiceId : action.objectId];
      const selectedPlace = gatheringLocation(snapshot);
      const point = gatheringPoint(snapshot);
      const placed =
        object?.location === 'gathering'
          ? {
              ...object,
              x:
                point.x +
                (action.type === 'assemble' && action.objectId === 'gathering_music' ? 60 : -40),
              y: point.y,
              location: selectedPlace,
            }
          : object?.location === 'instrument'
            ? { ...object, ...instrumentPoint(snapshot), location: instrumentLocation(snapshot) }
            : object;
      const cornerBuild =
        action.type === 'assemble' &&
        (action.objectId === 'corner_frame' || action.objectId === 'corner_cushions');
      const chosenShape: StoryShape = snapshot.sceneInput!.context.memory.facts.includes(
        'The chosen cosy corner design is star_canopy.',
      )
        ? 'corner-stars'
        : 'corner-leaves';
      if (placed && placed.location === snapshot.locationId)
        add(id, placed.name, placed, cornerBuild ? { ...placed, shape: chosenShape } : placed);
    }
    if (action.type === 'travel') {
      const edge = storyRoutePoint(snapshot.locationId, action.destinationId);
      // Keep the accessible signpost approach inside the map. Walking beyond it
      // onto the edge takes the route without a second confirmation dialog.
      const point = { ...edge, x: edge.x < 65 ? 100 : edge.x > 1055 ? 1020 : edge.x };
      add(
        id,
        `Path to ${storyPlaces[action.destinationId as keyof typeof storyPlaces]?.name ?? action.destinationId}`,
        point,
        { prop: 'arch' },
      );
    }
  }
  return markers;
}

// Coordinates/art are authored presentation data; the offered actions decide visibility.
type Placement = Point & {
  location: StorySnapshot['locationId'] | 'gathering' | 'instrument';
  name: string;
  prop?: string;
  shape?: StoryShape;
};
const resourcePlaces: Record<string, Placement> = {
  ...Object.fromEntries(
    [
      ['find_clover', 'lucky_clover', 'bridge_bank'],
      ['find_ribbon', 'ribbon', 'workshop'],
      ['find_crystal', 'river_crystal', 'far_bank'],
      ['find_shell', 'seashell', 'cove'],
      ['find_fern', 'fern_leaf', 'fern_hollow'],
      ['find_mushroom', 'moon_mushroom', 'mossy_clearing'],
      ['find_feather', 'feather', 'owl_grove'],
      ['find_acorn', 'acorn', 'acorn_trail'],
    ].map(([source, item, location]) => [
      source,
      {
        location,
        name: `Pick up ${items[item].name}`,
        x: location === 'far_bank' ? 880 : 310,
        y: 490,
        prop: items[item].prop,
      },
    ]),
  ),
  fallen_wood: { location: 'bridge_bank', name: 'Gather wood', x: 350, y: 490, prop: 'plank' },
  spare_wood: { location: 'workshop', name: 'Gather spare wood', x: 700, y: 480, prop: 'plank' },
  berry_bush: { location: 'far_bank', name: 'Gather berries', x: 930, y: 540, prop: 'berry' },
  glow_stones: {
    location: 'far_bank',
    name: 'Gather glow pebbles',
    x: 930,
    y: 480,
    shape: 'stones',
  },
};
const assemblyPlaces: Record<string, Placement> = {
  humming_hollow: {
    location: 'far_bank',
    name: 'Listen at the hollow',
    x: 1020,
    y: 290,
    shape: 'hollow',
  },
  trembling_leaf: {
    location: 'far_bank',
    name: 'Look at the trembling leaf',
    x: 920,
    y: 300,
    shape: 'leaf',
  },
  mossy_stump: {
    location: 'far_bank',
    name: 'Look beside the mossy stump',
    x: 1020,
    y: 550,
    shape: 'hollow',
  },
  hollow_branch: {
    location: 'far_bank',
    name: 'Inspect the hollow branch',
    x: 960,
    y: 540,
    shape: 'hollow',
  },
  instrument_home: {
    location: 'far_bank',
    name: 'Choose a home for the sound',
    x: 960,
    y: 540,
    shape: 'instrument',
  },
  wind_instrument: {
    location: 'instrument',
    name: 'Place the wind instrument',
    x: 0,
    y: 0,
    shape: 'instrument',
  },
  corner_site: {
    location: 'bridge_bank',
    name: 'Look at the cosy corner',
    x: 340,
    y: 580,
    shape: 'corner-leaves',
  },
  corner_style: {
    location: 'bridge_bank',
    name: 'Choose Pip’s cosy corner',
    x: 340,
    y: 580,
    shape: 'corner-leaves',
  },
  corner_frame: {
    location: 'bridge_bank',
    name: 'Set up the cosy corner',
    x: 340,
    y: 580,
    shape: 'corner-leaves',
  },
  corner_cushions: {
    location: 'bridge_bank',
    name: 'Arrange the cushions',
    x: 340,
    y: 580,
    shape: 'corner-stars',
  },
  workshop_path: {
    location: 'workshop',
    name: 'Check the path for Bramble',
    x: 870,
    y: 420,
    shape: 'trail',
  },
  gathering_place: {
    location: 'bridge_bank',
    name: 'Choose a gathering place',
    x: 370,
    y: 450,
    shape: 'bunting',
  },
  gathering_seats: {
    location: 'gathering',
    name: 'Arrange seats for friends',
    x: 0,
    y: 0,
    shape: 'seating',
  },
  gathering_music: {
    location: 'gathering',
    name: 'Prepare the gathering music',
    x: 0,
    y: 0,
    shape: 'music',
  },
  bridge: { location: 'bridge_bank', name: 'Repair the bridge', x: 570, y: 359, prop: 'plank' },
  picnic_table: {
    location: 'bridge_bank',
    name: 'Lay out the picnic',
    x: 359,
    y: 272,
    prop: 'berry',
  },
  near_lanterns: {
    location: 'bridge_bank',
    name: 'Light the near bank',
    x: 440,
    y: 540,
    shape: 'lantern',
  },
  far_lantern: {
    location: 'far_bank',
    name: 'Light the far bank',
    x: 940,
    y: 500,
    shape: 'lantern',
  },
};
export function storyDecorations(snapshot: StorySnapshot): WorldDecoration[] {
  if (!snapshot.sceneInput) return [];
  const projects: WorldDecoration[] = [];
  for (const object of snapshot.sceneInput.here.objects) {
    if (!object.id.startsWith(`project_${snapshot.locationId}_`)) continue;
    const kind = object.id.slice(`project_${snapshot.locationId}_`.length);
    if (kind === 'seat')
      projects.push({
        id: object.id,
        kind: object.state === 'starry' ? 'corner-stars' : 'corner-leaves',
        x: 340,
        y: 580,
      });
    if (kind === 'picnic') projects.push({ id: object.id, kind: 'picnic', x: 350, y: 280 });
    if (kind === 'supplies') projects.push({ id: object.id, kind: 'supplies', x: 880, y: 530 });
    if (kind === 'lantern') {
      projects.push({ id: object.id, kind: 'lantern', x: 660, y: 310 });
      projects.push({
        id: `${object.id}-design`,
        kind: object.state === 'starry' ? 'bunting' : 'leaf',
        x: 660,
        y: 365,
      });
    }
  }
  if (!['bridge_bank', 'far_bank', 'workshop'].includes(snapshot.locationId)) return projects;
  const facts = snapshot.sceneInput.context.memory.facts;
  if (snapshot.locationId === 'workshop')
    return facts.includes('The workshop path is clear.')
      ? [{ id: 'workshop_path', kind: 'trail', x: 870, y: 420 }]
      : [];
  const decorations: WorldDecoration[] = [];
  if (facts.includes('The picnic table is ready.'))
    decorations.push({ id: 'picnic_table', kind: 'picnic', x: 359, y: 272 });
  if (facts.includes('Two lanterns light the near bank.'))
    decorations.push(
      { id: 'near_lantern_1', kind: 'lantern', x: 400, y: 530 },
      { id: 'near_lantern_2', kind: 'lantern', x: 500, y: 530 },
    );
  if (facts.includes('One lantern lights the far bank.'))
    decorations.push({ id: 'far_lantern', kind: 'lantern', x: 940, y: 500 });
  for (const [location, point] of [
    ['bridge_bank', { x: 300, y: 380 }],
    ['far_bank', { x: 960, y: 540 }],
  ] as const)
    if (facts.includes(`The wind instrument is at ${location}.`))
      decorations.push({ id: 'wind_instrument', kind: 'instrument', ...point });
  if (facts.includes("Pip's cosy corner uses leafy_nook."))
    decorations.push({ id: 'cosy_corner', kind: 'corner-leaves', x: 340, y: 580 });
  if (facts.includes("Pip's cosy corner uses star_canopy."))
    decorations.push({ id: 'cosy_corner', kind: 'corner-stars', x: 340, y: 580 });
  const gathering = gatheringPoint(snapshot);
  if (facts.includes('Gathering seats are ready.'))
    decorations.push({
      id: 'gathering_seats',
      kind: 'seating',
      x: gathering.x - 40,
      y: gathering.y,
    });
  if (facts.includes('Gathering music is ready.'))
    decorations.push({ id: 'gathering_music', kind: 'music', x: gathering.x + 60, y: gathering.y });
  if (facts.includes('The moonlight gathering has begun.'))
    decorations.push({
      id: 'moonlight_gathering',
      kind: 'bunting',
      x: gathering.x,
      y: gathering.y - 30,
    });
  return [...decorations, ...projects];
}
export function gatheringLocation(snapshot: StorySnapshot): StorySnapshot['locationId'] {
  return snapshot.sceneInput?.context.memory.facts.includes('The gathering is at far_bank.')
    ? 'far_bank'
    : 'bridge_bank';
}
function instrumentLocation(snapshot: StorySnapshot) {
  const facts = snapshot.sceneInput?.context.memory.facts ?? [];
  if (facts.includes('The chosen wind instrument home is far_bank.')) return 'far_bank';
  if (facts.includes('The chosen wind instrument home is bridge_bank.')) return 'bridge_bank';
  return undefined;
}
function instrumentPoint(snapshot: StorySnapshot): Point {
  return instrumentLocation(snapshot) === 'far_bank' ? { x: 960, y: 540 } : { x: 300, y: 380 };
}
function gatheringPoint(snapshot: StorySnapshot): Point {
  return gatheringLocation(snapshot) === 'far_bank' ? { x: 950, y: 570 } : { x: 370, y: 450 };
}
export function crossingRoute(from: StorySnapshot['locationId'], to: StorySnapshot['locationId']) {
  if (from === 'workshop' && to === 'bridge_bank')
    return { start: { x: 100, y: 475 }, end: { x: 380, y: 420 }, followerOffset: -40 };
  if (from === 'bridge_bank' && to === 'far_bank')
    return { start: { x: 560, y: 359 }, end: { x: 850, y: 359 }, followerOffset: -40 };
  if (from === 'far_bank' && to === 'bridge_bank')
    return { start: { x: 850, y: 359 }, end: { x: 540, y: 359 }, followerOffset: 40 };
  return undefined;
}
export function nextStoryLabel(snapshot: StorySnapshot) {
  if (snapshot.sceneInput?.mission?.status !== 'completed') return undefined;
  const accept = snapshot.sceneInput.availableActions.find(
    (item) => item.action.type === 'accept_mission',
  );
  return (
    accept &&
    (snapshot.scene?.choices.find((choice) => choice.actionId === accept.id)?.label ??
      accept.description)
  );
}

/** Display-only projection into the existing scenery renderer; never saved as assessed progress. */
export function storySceneryFlags(snapshot: StorySnapshot): string[] {
  const flags = bridgeRepaired(snapshot) ? ['meadow-bridge'] : [];
  if (snapshot.sceneInput?.context.memory.facts.includes('The picnic table is ready.'))
    flags.push('meadow-picnic');
  return flags;
}

/** Old saves without step destinations keep their original neighbourhood. */
export function neighbourDestination(
  q: NonNullable<NonNullable<StorySnapshot['sceneInput']>['neighbourMissions']>[number],
) {
  return (!q.inspected && q.generatedStory?.steps[q.stepIndex ?? 0]?.locationId) || q.locationId;
}
/** Share the current open mission between the HUD tracker and Missions dialog. */
export function currentNeighbourMission(snapshot: StorySnapshot) {
  const input = snapshot.sceneInput;
  const open = input?.neighbourMissions?.filter((q) => q.status === 'open') ?? [];
  const local = [...open].reverse().find((q) => neighbourDestination(q) === snapshot.locationId);
  const pip = input?.mission;
  const pipHere =
    pip?.status === 'active' &&
    ['bridge_bank', 'far_bank', 'workshop'].includes(snapshot.locationId);
  return local ?? (pipHere ? undefined : open.at(-1));
}
export function missionTracker(snapshot: StorySnapshot): string {
  const input = snapshot.sceneInput;
  const pip = input?.mission;
  const neighbour = currentNeighbourMission(snapshot);
  if (neighbour) {
    const task = neighbour.inspected
      ? `Return to ${neighbour.generatedStory?.name ?? neighbours[neighbour.characterId]?.name ?? neighbour.characterId}`
      : neighbour.task;
    return `${task} · ${storyPlaces[neighbourDestination(neighbour)].name}`;
  }
  if (pip?.status === 'active') return pip.current ?? pip.goal;
  const offered = input?.neighbourMissions
    ?.slice()
    .reverse()
    .find((q) => q.status === 'offered');
  if (offered)
    return `Meet ${offered.generatedStory?.name ?? neighbours[offered.characterId]?.name ?? offered.characterId} · ${storyPlaces[offered.locationId].name}`;
  return nextStoryLabel(snapshot) ?? 'Explore and talk to a friend for a new adventure';
}
export function missionDiscovery(snapshot: StorySnapshot, objectId: string): string | undefined {
  const q = snapshot.sceneInput?.neighbourMissions?.find((q) =>
    objectId.startsWith(`neighbour_${q.characterId}_`),
  );
  if (!q) return;
  const suffix = objectId.slice(`neighbour_${q.characterId}_`.length);
  if (suffix.startsWith('g') && !suffix.startsWith(`g${q.generation}_`)) return;
  const index = Number(suffix.split('_').at(-1));
  if (!Number.isInteger(index) || index < 0 || index >= (q.stepIndex ?? 0)) return;
  return q.status === 'completed' && q.generatedStory?.completeOnLastStep
    ? q.payoff
    : q.generatedStory?.steps[index]?.discovery;
}
