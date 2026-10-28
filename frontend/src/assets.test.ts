import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { heroFrames, propFrames, frameRect } from './sprites.ts';
import { heroes } from './art.ts';

test('Shared sprite frames are inside real PNG assets and cover every selectable hero', () => {
  for (const [kind, names] of [
    ['hero', heroFrames],
    ['prop', propFrames],
  ] as const) {
    const png = readFileSync(new URL(`../public/assets/${kind}-atlas-v2.png`, import.meta.url));
    const width = png.readUInt32BE(16),
      height = png.readUInt32BE(20);
    assert.equal(png.toString('ascii', 1, 4), 'PNG');
    for (let i = 0; i < names.length; i++) {
      const [x, y, w, h] = frameRect(i, kind);
      assert.ok(
        w > 0 && h > 0 && x >= 0 && y >= 0 && x + w <= width && y + h <= height,
        `${names[i]} exceeds image bounds`,
      );
    }
  }
  for (const id of Object.keys(heroes)) assert.ok(heroFrames.includes(id));
});
