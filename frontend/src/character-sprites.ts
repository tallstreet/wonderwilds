import type Phaser from 'phaser';
import roster from '../../content/creatures/friends.v1.json';
import type { Rect } from './types.ts';
import { characterSheetColumnSplit, characterSheetRowSplit } from './character-art.ts';

export const characterSheets = ['meadow', 'orchard', 'cove'] as const;
const campaignSheets: {
  name: string;
  ids: string[];
  species: string[];
  frames: Rect[];
  width?: number;
  height?: number;
}[] = [
  {
    name: 'cast-v1',
    ids: ['rowan', 'hazel', 'moss', 'collector-cart'],
    species: ['fox', 'owl', 'mossback', 'prop'],
    frames: [
      [0, 0, 627, 627],
      [627, 0, 627, 627],
      [0, 627, 627, 627],
      [627, 627, 627, 627],
    ],
  },
  {
    name: 'props-v1',
    ids: ['campaign-camp', 'beacon-dim', 'beacon-restored', 'garden-irrigation'],
    species: ['prop', 'prop', 'prop', 'prop'],
    frames: [
      [0, 0, 627, 627],
      [627, 0, 627, 627],
      [0, 627, 627, 627],
      [627, 627, 627, 627],
    ],
  },
  {
    name: 'wayfinders-meadow-v1',
    ids: ['pip-wayfinder', 'tansy-wayfinder', 'nib-wayfinder', 'moonburst'],
    species: ['kitten', 'hedgehog', 'field mouse', 'prop'],
    frames: [
      [0, 0, 627, 627],
      [627, 0, 627, 627],
      [0, 627, 627, 627],
      [627, 627, 627, 627],
    ],
  },
  {
    name: 'finale-v1',
    ids: ['vell', 'lighthouse-lens', 'observatory-machine', 'sanctuary'],
    species: ['badger', 'prop', 'prop', 'prop'],
    frames: [
      [0, 0, 627, 627],
      [627, 0, 627, 627],
      [0, 627, 627, 627],
      [627, 627, 627, 627],
    ],
  },
  {
    name: 'orchard-v1',
    ids: ['orchard-mill', 'mill-automaton', 'ferry-raft', 'reed'],
    species: ['prop', 'prop', 'prop', 'bird'],
    frames: [
      [0, 0, 627, 627],
      [627, 0, 627, 627],
      [0, 627, 627, 627],
      [627, 627, 627, 627],
    ],
  },
  {
    name: 'wayfinders-orchard-v1',
    ids: ['flutter-wayfinder', 'cleo-wayfinder', 'bramble-wayfinder', 'patter-wayfinder'],
    species: ['moth', 'kitten', 'bunny', 'duckling'],
    frames: [
      [0, 0, 627, 590],
      [627, 0, 627, 590],
      [0, 590, 627, 664],
      [627, 590, 627, 664],
    ],
  },
  {
    name: 'wayfinders-cove-v1',
    ids: ['twizzle-wayfinder', 'luna-wayfinder', 'pebble-wayfinder', 'glimmer-wayfinder'],
    species: ['squirrel', 'sea kitten', 'otter', 'firefly'],
    frames: [
      [0, 0, 600, 627],
      [600, 0, 654, 627],
      [0, 627, 600, 627],
      [600, 627, 654, 627],
    ],
  },
  {
    name: 'wayfinders-sail-v1',
    ids: ['sail-wayfinder', 'bridge-seal', 'bloom-seal', 'accord-seal'],
    species: ['puffin', 'prop', 'prop', 'prop'],
    frames: [
      [0, 0, 627, 627],
      [627, 0, 627, 627],
      [0, 627, 627, 627],
      [627, 627, 627, 627],
    ],
  },
  {
    name: 'effects-v1',
    width: 1536,
    height: 1024,
    ids: [
      'effect-gleam',
      'effect-growth',
      'effect-craft',
      'effect-rhythm',
      'effect-tide',
      'effect-celebration',
    ],
    species: Array(6).fill('prop'),
    frames: [
      [0, 0, 512, 512],
      [512, 0, 512, 512],
      [1024, 0, 512, 512],
      [0, 512, 512, 512],
      [512, 512, 512, 512],
      [1024, 512, 512, 512],
    ],
  },
  ...[
    { name: 'seals-v1', ids: ['echo-seal', 'harvest-seal', 'current-seal', 'tide-seal'] },
    {
      name: 'affinities-a-v1',
      ids: ['beacon-seal', 'affinity-gleam', 'affinity-growth', 'affinity-craft'],
    },
    {
      name: 'affinities-b-v1',
      ids: ['affinity-rhythm', 'affinity-tide', 'fossil-keepsake', 'clue-rubbing'],
    },
    {
      name: 'landmarks-v1',
      ids: ['chime-tower', 'tide-refuge', 'habitat-cache-closed', 'habitat-cache-open'],
    },
  ].map((sheet) => ({
    ...sheet,
    species: ['prop', 'prop', 'prop', 'prop'],
    frames: [
      [0, 0, 627, 627],
      [627, 0, 627, 627],
      [0, 627, 627, 627],
      [627, 627, 627, 627],
    ] as Rect[],
  })),
];
export function characterSprite(id: string) {
  for (const sheet of campaignSheets) {
    const index = sheet.ids.indexOf(id);
    if (index >= 0)
      return { texture: `campaign-${sheet.name}`, frame: id, index, species: sheet.species[index] };
  }
  const friend = roster.creatures.find((creature) => creature.id === id);
  if (!friend) return undefined;
  const regional = roster.creatures.filter((creature) => creature.regionId === friend.regionId);
  return {
    texture: `friends-${friend.regionId}`,
    frame: id,
    index: regional.findIndex((creature) => creature.id === id),
    species: friend.species,
  };
}

/** Frame the original transparent PNG without editing its pixels. */
export function characterFrame(index: number, width: number, height: number, region = ''): Rect {
  if (!Number.isInteger(index) || index < 0 || index > 3 || width < 2 || height < 2)
    throw new Error('Invalid character sheet frame');
  const col = index % 2,
    row = Math.floor(index / 2),
    columnSplit = Math.round(width * characterSheetColumnSplit(region)),
    x = col === 0 ? 0 : columnSplit,
    split = Math.round(height * characterSheetRowSplit(region)),
    y = row === 0 ? 0 : split;
  return [x, y, (col === 0 ? columnSplit : width) - x, (row === 0 ? split : height) - y];
}

export function preloadCharacters(scene: Phaser.Scene) {
  for (const sheet of campaignSheets)
    scene.load.image(`campaign-${sheet.name}`, `/assets/campaign/${sheet.name}.png`);
  for (const region of characterSheets)
    scene.load.image(`friends-${region}`, `/assets/characters/${region}.png`);
}

export function registerCharacters(scene: Phaser.Scene) {
  for (const sheet of campaignSheets) {
    const key = `campaign-${sheet.name}`;
    if (!scene.textures.exists(key)) continue;
    const texture = scene.textures.get(key);
    sheet.ids.forEach((id, index) => {
      if (!texture.has(id)) texture.add(id, 0, ...sheet.frames[index]);
    });
  }
  for (const region of characterSheets) {
    const key = `friends-${region}`;
    if (!scene.textures.exists(key)) continue;
    const texture = scene.textures.get(key);
    const { width, height } = texture.source[0];
    roster.creatures
      .filter((creature) => creature.regionId === region)
      .forEach((creature, index) => {
        if (!texture.has(creature.id))
          texture.add(creature.id, 0, ...characterFrame(index, width, height, region));
      });
  }
}

/** Missing generated sheets keep the known Pip art rather than an invalid frame. */
export function availableCharacter(
  id: string,
  textures: { exists(key: string): boolean; get(key: string): { has(frame: string): boolean } },
) {
  const sprite = characterSprite(id);
  return sprite && textures.exists(sprite.texture) && textures.get(sprite.texture).has(sprite.frame)
    ? sprite
    : undefined;
}

/** Exact inspected atlas crop for campaign portraits, shared with Phaser registration. */
export function campaignCharacterArt(id: string) {
  for (const sheet of campaignSheets) {
    const index = sheet.ids.indexOf(id);
    if (index >= 0)
      return {
        url: `/assets/campaign/${sheet.name}.png`,
        width: sheet.width ?? 1254,
        height: sheet.height ?? 1254,
        frame: sheet.frames[index]!,
      };
  }
  return undefined;
}
