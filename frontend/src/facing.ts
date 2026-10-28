import type Phaser from 'phaser';
import type { Rect, Facing } from './types.ts';
import { heroFrames, frameRect } from './sprites.ts';

export const SIDE_ATLAS = '/assets/hero-side-atlas-v3.png';
export const BACK_ATLAS = '/assets/hero-back-atlas-v3.png';
export const facingDirections = ['down', 'right', 'up', 'left'];

// Inspected source rectangles: the generated sheets are not a perfect grid.
// Rosie reaches below row 1, and rear Wren's wing crosses column 1.
export const directionalRects: Record<string, Rect[]> = {
  side: [
    [0, 0, 444, 453],
    [444, 0, 443, 443],
    [887, 0, 444, 443],
    [1331, 0, 443, 443],
    [0, 453, 444, 434],
    [444, 443, 443, 444],
    [887, 443, 444, 444],
    [1331, 443, 443, 444],
  ],
  back: [
    [0, 0, 444, 439],
    [444, 0, 443, 431],
    [887, 0, 444, 432],
    [1331, 0, 443, 439],
    [0, 440, 470, 447],
    [490, 430, 397, 457],
    [887, 430, 444, 457],
    [1331, 440, 443, 447],
  ],
};
// Full-atlas coordinates of the midpoint between planted feet, measured from
// the art rather than the frame centre (tails make these very different).
const feet = {
  down: [
    [248, 430],
    [696, 430],
    [1135, 430],
    [1556, 421],
    [254, 863],
    [703, 874],
    [1135, 874],
    [1561, 875],
  ],
  side: [
    [302, 444],
    [735, 442],
    [1170, 442],
    [1596, 436],
    [300, 870],
    [754, 875],
    [1189, 875],
    [1637, 875],
  ],
  back: [
    [268, 430],
    [698, 430],
    [1110, 430],
    [1538, 430],
    [259, 855],
    [699, 855],
    [1118, 855],
    [1545, 855],
  ],
};
const visualHeight = {
  down: [425, 420, 425, 420, 410, 430, 430, 435],
  side: [440, 440, 430, 430, 415, 430, 430, 430],
  back: [405, 405, 410, 425, 400, 425, 425, 410],
};

export function facingGeometry(heroId: string, facing: string) {
  const index = Math.max(0, heroFrames.indexOf(heroId));
  const kind = facing === 'up' ? 'back' : facing === 'left' || facing === 'right' ? 'side' : 'down';
  const [x, y, w, h] = kind === 'down' ? frameRect(index, 'hero') : directionalRects[kind][index];
  const [footX, footY] = feet[kind][index],
    ratio = visualHeight.down[index] / visualHeight[kind][index];
  const originX = (footX - x) / w;
  return {
    originX: facing === 'left' ? 1 - originX : originX,
    originY: (footY - y) / h,
    scaleX: (w / 444) * ratio,
    scaleY: (h / 444) * ratio,
  };
}

/** Load once from Scene.preload. Frames are registered after loading completes. */
export function preloadFacing(scene: Phaser.Scene) {
  scene.load.image('hero-side-atlas', SIDE_ATLAS);
  scene.load.image('hero-back-atlas', BACK_ATLAS);
}

/** Regular-grid utility retained for tools; rendered frames use inspected rects. */
export function directionalFrameRect(index: number, width: number, height: number): Rect {
  const col = index % 4,
    row = Math.floor(index / 4);
  const left = Math.round((col * width) / 4),
    top = Math.round((row * height) / 2);
  return [
    left,
    top,
    Math.round(((col + 1) * width) / 4) - left,
    Math.round(((row + 1) * height) / 2) - top,
  ];
}
export function registerFacing(scene: Phaser.Scene) {
  for (const key of ['hero-side-atlas', 'hero-back-atlas']) {
    if (!scene.textures.exists(key)) continue;
    const texture = scene.textures.get(key),
      kind = key === 'hero-side-atlas' ? 'side' : 'back';
    heroFrames.forEach((id, index) => {
      if (!texture.has(id)) texture.add(id, 0, ...directionalRects[kind][index]);
    });
  }
}

/**
 * Choose heading from actual movement, not an unfulfilled path target. Idle keeps
 * the last heading. A small diagonal dead band avoids jitter at equal x/y speed.
 * `dx`/`dy` are world-space displacement since the previous update.
 */
export function resolveFacing(heroId: string, dx: number, dy: number, previousFacing = 'down') {
  const hero = heroFrames.includes(heroId) ? heroId : 'rose';
  const previous = facingDirections.includes(previousFacing) ? previousFacing : 'down';
  let facing = previous as Facing;
  if (Number.isFinite(dx) && Number.isFinite(dy) && Math.hypot(dx, dy) > 0.01) {
    const ax = Math.abs(dx),
      ay = Math.abs(dy),
      horizontal = previous === 'left' || previous === 'right';
    if (ax > ay * 1.15 || (horizontal && ax >= ay / 1.15 && ax > 0))
      facing = dx < 0 ? 'left' : 'right';
    else facing = dy < 0 ? 'up' : 'down';
  }
  return {
    texture:
      facing === 'up' ? 'hero-back-atlas' : facing === 'down' ? 'hero-atlas' : 'hero-side-atlas',
    frame: hero,
    flipX: facing === 'left',
    facing,
    ...facingGeometry(hero, facing),
  };
}
