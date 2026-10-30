import type { StoryAction, StorySnapshot } from '../../../contracts/generated/contracts.ts';
import type { Point, WorldEncounter, StoryShape, WorldDecoration } from '../types.ts';

export const storyPlaces = {
  bridge_bank: { region: 'meadow', name: 'Pip’s bridge · near bank', spawn: { x: 160, y: 490 } },
  workshop: { region: 'orchard', name: 'The woodcutter’s workshop', spawn: { x: 850, y: 490 } },
  far_bank: { region: 'meadow', name: 'Pip’s bridge · far bank', spawn: { x: 850, y: 359 } },
} as const;
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
    });
  for (const character of snapshot.sceneInput?.here.characters ?? []) {
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
    if (action.type === 'collect') {
      const source = resourcePlaces[action.sourceId];
      if (source && source.location === snapshot.locationId) add(id, source.name, source, source);
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
      const point =
        snapshot.locationId === 'workshop'
          ? { x: 960, y: 453 }
          : action.destinationId === 'workshop'
            ? { x: 100, y: 475 }
            : snapshot.locationId === 'far_bank'
              ? { x: 810, y: 359 }
              : { x: 570, y: 359 };
      add(
        id,
        action.destinationId === 'workshop'
          ? 'Path to the workshop'
          : action.destinationId === 'far_bank'
            ? 'Cross the bridge'
            : 'Return to the bridge',
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
  return decorations;
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
