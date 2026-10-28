import type Phaser from 'phaser';
import roster from '../../content/creatures/friends.v1.json';
import type { Rect } from './types.ts';
import { characterSheetColumnSplit, characterSheetRowSplit } from './character-art.ts';

export const characterSheets = ['meadow', 'orchard', 'cove'] as const;
export function characterSprite(id: string) {
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
  for (const region of characterSheets)
    scene.load.image(`friends-${region}`, `/assets/characters/${region}.png`);
}

export function registerCharacters(scene: Phaser.Scene) {
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
