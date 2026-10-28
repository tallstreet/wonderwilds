import test from 'node:test';
import assert from 'node:assert/strict';
import { moveGround, advanceFollower } from './grounding.ts';

test('Blocked motion reports no travel, while sliding records safe ground segments', () => {
  const still = moveGround({ x: 0, y: 0 }, 1, 0, 4, (x) => x > 0);
  assert.equal(still.distance, 0);
  assert.deepEqual(still.path, []);
  const slide = moveGround({ x: 0, y: 0 }, 1, 1, 4, (x) => x > 0);
  assert.equal(slide.x, 0);
  assert.ok(slide.y > 0);
  assert.ok(slide.distance > 0);
  assert.deepEqual(slide.path, [{ x: 0, y: slide.y }]);
});
test('Companion follows a turn and stops with a fixed trail gap instead of cutting a corner', () => {
  const trail = [
    { x: 10, y: 0 },
    { x: 10, y: 10 },
    { x: 20, y: 10 },
  ];
  const a = advanceFollower({ x: 0, y: 0 }, trail, 15, 5, () => false);
  assert.deepEqual({ x: a.x, y: a.y }, { x: 10, y: 5 });
  const b = advanceFollower(a, trail, 100, 5, () => false);
  assert.deepEqual({ x: b.x, y: b.y }, { x: 15, y: 10 });
  assert.equal(advanceFollower(b, trail, 100, 5, () => false).distance, 0);
});
test('Companion refuses a blocked trail segment', () => {
  const trail = [{ x: 10, y: 0 }];
  assert.equal(advanceFollower({ x: 0, y: 0 }, trail, 10, 0, (x) => x > 4).distance, 0);
  assert.equal(trail.length, 1);
});

import { findRoute, isBlocked } from './navigation.ts';
test('Foot-anchored NPC approaches stay reachable and garden remains gated', () => {
  const spawn = { x: 160, y: 490 };
  for (const target of [
    { x: 555, y: 463 },
    { x: 305, y: 363 },
  ]) {
    const route = findRoute(spawn, target, false);
    assert.ok(route.length > 0);
    for (const point of route) assert.equal(isBlocked(point.x, point.y, false), false);
  }
  const garden = { x: 890, y: 348 };
  assert.equal(findRoute(spawn, garden, false).length, 0);
  const route = findRoute(spawn, garden, true);
  assert.ok(route.length > 0);
  for (const point of route) assert.equal(isBlocked(point.x, point.y, true), false);
});
