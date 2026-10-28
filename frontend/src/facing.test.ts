import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, existsSync } from 'node:fs';
import { resolveFacing, directionalRects, facingGeometry } from './facing.ts';
import { heroFrames } from './sprites.ts';

test('Directional poses follow travel and preserve idle heading', () => {
  for (const [dx, dy, direction, texture, flip] of [
    [1, 0, 'right', 'hero-side-atlas', false],
    [-1, 0, 'left', 'hero-side-atlas', true],
    [0, -1, 'up', 'hero-back-atlas', false],
    [0, 1, 'down', 'hero-atlas', false],
  ] as const) {
    const pose = resolveFacing('clover', dx, dy);
    assert.equal(pose.facing, direction);
    assert.equal(pose.texture, texture);
    assert.equal(pose.flipX, flip);
    assert.deepEqual(resolveFacing('clover', 0, 0, direction), pose);
  }
});
test('Diagonal facing resists axis chatter and rejects invalid identity/heading', () => {
  assert.equal(resolveFacing('poppy', 1, 0.99, 'up').facing, 'down');
  assert.equal(resolveFacing('poppy', 1, 0.99, 'right').facing, 'right');
  assert.equal(resolveFacing('poppy', 0.2, -1, 'right').facing, 'up');
  assert.equal(resolveFacing('unknown', NaN, 0, 'invalid').frame, 'rose');
  assert.equal(resolveFacing('unknown', NaN, 0, 'invalid').facing, 'down');
});
test('Inspected directional rectangles fit the real PNGs without overflow', () => {
  for (const [kind, asset] of [
    ['side', 'hero-side-atlas-v3.png'],
    ['back', 'hero-back-atlas-v3.png'],
  ]) {
    const file = new URL(`../public/assets/${asset}`, import.meta.url);
    assert.ok(existsSync(file), `${asset} must ship with the helper`);
    const png = readFileSync(file),
      width = png.readUInt32BE(16),
      height = png.readUInt32BE(20);
    assert.equal(directionalRects[kind].length, heroFrames.length);
    for (const [x, y, w, h] of directionalRects[kind])
      assert.ok(x >= 0 && y >= 0 && x + w <= width && y + h <= height);
  }
});
test('All heroes have bounded foot anchors and consistent mirrored pivots', () => {
  for (const hero of heroFrames) {
    for (const direction of ['down', 'up', 'left', 'right']) {
      const pose = facingGeometry(hero, direction);
      assert.ok(pose.originX > 0 && pose.originX < 1);
      assert.ok(pose.originY > 0 && pose.originY <= 1);
      assert.ok(pose.scaleX > 0.8 && pose.scaleX < 1.2);
      assert.ok(pose.scaleY > 0.8 && pose.scaleY < 1.2);
    }
    const l = facingGeometry(hero, 'left'),
      r = facingGeometry(hero, 'right');
    assert.equal(l.originX, 1 - r.originX);
    assert.equal(l.originY, r.originY);
  }
});
