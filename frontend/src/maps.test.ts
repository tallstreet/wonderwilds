import test from 'node:test';
import assert from 'node:assert/strict';
import { maps, mapExit, exitPoint, type MapId } from './maps.ts';
import { findRoute, isBlocked } from './navigation.ts';
import { freshState, restoreProgress } from './progress.ts';

test('All paths connect to the opposite destination path, preserving the walking offset', () => {
  for (const region of Object.keys(maps) as MapId[]) {
    for (const side of ['west', 'east'] as const) {
      const point = exitPoint(region, side);
      const arrivalSide = side === 'west' ? 'east' : 'west';
      for (const offset of [-12, 0, 12]) {
        const exit = mapExit(region, { x: point.x, y: point.y + offset });
        assert.ok(exit);
        assert.equal(maps[exit.region][arrivalSide], region);
        assert.equal(exit.position.y, maps[exit.region].pathY[arrivalSide] + offset);
        assert.equal(isBlocked(exit.position.x, exit.position.y, false, exit.region), false);
        assert.equal(mapExit(exit.region, exit.position), null);
        // An arrival must survive a reload, including on the meadow's east bank.
        const saved = { ...freshState(), region: exit.region, position: exit.position };
        assert.deepEqual(restoreProgress(JSON.stringify(saved)), saved);
      }
      assert.equal(mapExit(region, { x: point.x, y: point.y + 45 }), null);
      assert.equal(mapExit(region, { x: 560, y: point.y }), null);
    }
  }
});
test('Tap routes reach every exit and all new-map paths stay walkable', () => {
  for (const region of Object.keys(maps) as MapId[]) {
    for (const side of ['west', 'east'] as const) {
      const point = exitPoint(region, side);
      const start = { x: side === 'west' ? 160 : 950, y: 490 };
      const path = findRoute(start, point, false, region);
      assert.ok(path.length > 0);
      path.forEach((p) => assert.equal(isBlocked(p.x, p.y, false, region), false));
      assert.ok(mapExit(region, path[path.length - 1]));
    }
    if (region !== 'meadow') {
      assert.ok(
        findRoute(exitPoint(region, 'west'), exitPoint(region, 'east'), false, region).length > 40,
      );
      assert.equal(isBlocked(180, 150, false, region), true);
    }
  }
  assert.equal(
    restoreProgress(JSON.stringify({ ...freshState(), region: '__proto__' })).region,
    'meadow',
  );
});

test('The exploration library has 75 unique destinations in one walkable loop', () => {
  assert.equal(Object.keys(maps).length, 75);
  assert.equal(new Set(Object.values(maps).map((m) => m.name)).size, 75);
  const visited = new Set<string>();
  let region = 'meadow';
  while (!visited.has(region)) {
    visited.add(region);
    if (maps[region].biome !== undefined) {
      const start = { x: 160, y: 490 };
      for (const side of ['west', 'east'] as const) {
        const path = findRoute(start, exitPoint(region, side), false, region);
        assert.ok(path.length > 0, `${region} ${side} is reachable from the travel spawn`);
      }
      assert.deepEqual(
        restoreProgress(JSON.stringify({ ...freshState(), region, position: { x: 160, y: 490 } }))
          .region,
        region,
      );
    }
    region = maps[region].east;
  }
  assert.equal(region, 'meadow');
  assert.equal(visited.size, 75);
});
