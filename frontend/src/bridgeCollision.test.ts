import test from 'node:test';
import assert from 'node:assert/strict';
import { BRIDGE, FOOTPRINT } from './mapGeometry.ts';
import { isBlocked, findRoute } from './navigation.ts';
import { moveGround } from './grounding.ts';

test('Bridge clearance is measured at planted feet and excludes both rails', () => {
  const top = BRIDGE.deckTop + FOOTPRINT.halfHeight,
    bottom = BRIDGE.deckBottom - FOOTPRINT.halfHeight;
  for (const x of [614, 640, 690, 740, 766]) {
    assert.equal(isBlocked(x, top, true), false);
    assert.equal(isBlocked(x, bottom, true), false);
    assert.equal(isBlocked(x, top - 0.1, true), true);
    assert.equal(isBlocked(x, bottom + 0.1, true), true);
    assert.equal(isBlocked(x, BRIDGE.y, false), true);
  }
  // A tall character's torso is over the river; the feet are safely on deck.
  const actor = { x: 690, y: 352, art: { x: 0, y: -9, displayHeight: 112 } };
  assert.equal(isBlocked(actor.x, actor.y, true), false);
  assert.equal(isBlocked(actor.x, actor.y - actor.art.displayHeight / 2, true), true);
  assert.equal(moveGround(actor, 1, 0, 8, (x, y) => isBlocked(x, y, true)).distance, 8);
});

test('Keyboard crossings connect both banks without feet touching water or rails', () => {
  for (const direction of [1, -1]) {
    let point = { x: direction === 1 ? 590 : 790, y: 352 };
    for (let i = 0; i < 25; i++) {
      const next = moveGround(point, direction, 0, 8, (x, y) => isBlocked(x, y, true));
      assert.equal(next.distance, 8);
      point = next;
    }
    assert.equal(point.x, direction === 1 ? 790 : 590);
    // Walking toward either rail stops while the footprint remains on deck.
    for (const vertical of [-1, 1]) {
      let feet = { x: 690, y: 352 };
      for (let i = 0; i < 30; i++)
        feet = moveGround(feet, 0, vertical, 1, (x, y) => isBlocked(x, y, true));
      assert.ok(feet.y - FOOTPRINT.halfHeight >= BRIDGE.deckTop);
      assert.ok(feet.y + FOOTPRINT.halfHeight <= BRIDGE.deckBottom);
    }
  }
});

test('Tap routes cross the measured deck with continuous footprint clearance', () => {
  for (const [start, target] of [
    [
      { x: 580, y: 450 },
      { x: 800, y: 300 },
    ],
    [
      { x: 800, y: 300 },
      { x: 580, y: 450 },
    ],
  ]) {
    assert.deepEqual(findRoute(start, target, false), []);
    const path = findRoute(start, target, true);
    assert.ok(path.length);
    assert.ok(path.some((p) => p.x === 688 && (p.y === 352 || p.y === 368)));
    for (let i = 1; i < path.length; i++)
      for (let n = 0; n <= 16; n++) {
        const a = path[i - 1],
          b = path[i];
        assert.equal(
          isBlocked(a.x + ((b.x - a.x) * n) / 16, a.y + ((b.y - a.y) * n) / 16, true),
          false,
        );
      }
  }
});
