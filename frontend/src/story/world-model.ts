import type { StoryAction, StorySnapshot } from '../../../contracts/generated/contracts.ts';
import type { Point, WorldEncounter } from '../types.ts';

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
      return `object:${action.objectId}`;
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
    art: { sprite?: string; prop?: string; shape?: 'stones' | 'lantern' },
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
    // Existing Bramble art stands in for the authored woodcutter; no new character asset.
    if (character.id === 'woodcutter')
      add('npc:woodcutter', 'Woodcutter', { x: 470, y: 380 }, { sprite: 'bramble' });
  }
  const actions = snapshot.sceneInput?.availableActions ?? [];
  for (const { action } of actions) {
    const id = actionTarget(action);
    if (markers.some((marker) => marker.id === id)) continue;
    if (action.type === 'collect') {
      const source = resourcePlaces[action.sourceId];
      if (source && source.location === snapshot.locationId) add(id, source.name, source, source);
    }
    if (action.type === 'assemble') {
      const object = assemblyPlaces[action.objectId];
      if (object && object.location === snapshot.locationId) add(id, object.name, object, object);
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
  location: StorySnapshot['locationId'];
  name: string;
  prop?: string;
  shape?: 'stones' | 'lantern';
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
export function storyDecorations(
  snapshot: StorySnapshot,
): { id: string; kind: 'picnic' | 'lantern'; x: number; y: number }[] {
  if (snapshot.locationId === 'workshop' || !snapshot.sceneInput) return [];
  const facts = snapshot.sceneInput.context.memory.facts;
  const decorations: { id: string; kind: 'picnic' | 'lantern'; x: number; y: number }[] = [];
  if (facts.includes('The picnic table is ready.'))
    decorations.push({ id: 'picnic_table', kind: 'picnic', x: 359, y: 272 });
  if (facts.includes('Two lanterns light the near bank.'))
    decorations.push(
      { id: 'near_lantern_1', kind: 'lantern', x: 400, y: 530 },
      { id: 'near_lantern_2', kind: 'lantern', x: 500, y: 530 },
    );
  if (facts.includes('One lantern lights the far bank.'))
    decorations.push({ id: 'far_lantern', kind: 'lantern', x: 940, y: 500 });
  return decorations;
}
export function crossingRoute(from: StorySnapshot['locationId'], to: StorySnapshot['locationId']) {
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
