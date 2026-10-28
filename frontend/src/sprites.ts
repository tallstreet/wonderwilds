import type Phaser from 'phaser';
import type { Rect } from './types.ts';
// Shared source rectangles keep HTML portraits and Phaser sprites identical.
// Frame definitions trim transparent gutters at runtime; PNG originals remain intact.
export const HERO_ATLAS = '/assets/hero-atlas-v2.png';
export const PROP_ATLAS = '/assets/prop-atlas-v2.png';
export const heroFrames = ['rose', 'sky', 'gold', 'poppy', 'wren', 'clover', 'mochi', 'pip'];
export const propFrames = [
  'bridge-frame',
  'plank',
  'garden-sleeping',
  'garden-blooming',
  'picnic',
  'arch',
  'berry',
  'flower',
];
const atlasWidth = 1774,
  atlasHeight = 887;
export const propRects: Rect[] = [
  [25, 105, 470, 270],
  [606, 25, 110, 385],
  [879, 95, 415, 284],
  [1344, 93, 410, 286],
  [20, 480, 477, 350],
  [510, 430, 420, 410],
  [993, 540, 275, 247],
  [1380, 510, 344, 309],
];
export function frameRect(index: number, kind: string): Rect {
  if (kind === 'prop') return propRects[index];
  const x = [0, 444, 887, 1331][index % 4],
    right = [444, 887, 1331, 1774][index % 4],
    y = index < 4 ? 0 : 444;
  return [x, y, right - x, index < 4 ? 444 : 443];
}
export function registerAtlas(texture: Phaser.Textures.Texture, names: string[]) {
  names.forEach((name, i) =>
    texture.add(name, 0, ...frameRect(i, names === propFrames ? 'prop' : 'hero')),
  );
}
export function spriteMarkup(index: number, kind: string, size = 100) {
  const [x, y, w, h] = frameRect(index, kind);
  const positionX = (x / (atlasWidth - w)) * 100,
    positionY = (y / (atlasHeight - h)) * 100;
  return `<span class="sprite-art ${kind === 'hero' ? 'character-sprite' : 'prop-art'}" aria-hidden="true" style="--sprite-size:${size}px;background-size:${(atlasWidth / w) * 100}% ${(atlasHeight / h) * 100}%;background-position:${positionX}% ${positionY}%;--art-ratio:${w / h}"></span>`;
}
export function propArt(name: string, size = 40) {
  return spriteMarkup(Math.max(0, propFrames.indexOf(name)), 'prop', size);
}
