import { test } from 'node:test';
import assert from 'node:assert/strict';
import { placeWorldEncounters, meadowPlacements } from './world-placements.ts';
import { worldQuests, trainingClearing } from './backend-world-model.ts';
import { findRoute, isBlocked } from './navigation.ts';

const definition = { id: 'test', region: 'meadow', short: 'Test', character: 'Pip' };
const placement = { region: 'meadow', x: 205, y: 490 };
test('Placement validation identifies missing, obsolete and duplicate encounter IDs', () => {
  assert.throws(() => placeWorldEncounters([definition], {}), /Missing world placement: test/);
  assert.throws(
    () => placeWorldEncounters([], { test: placement }),
    /Obsolete world placement: test/,
  );
  assert.throws(
    () => placeWorldEncounters([definition, definition], { test: placement }),
    /Duplicate/,
  );
});
test('Placement validation rejects unknown or mismatched maps and non-finite/outside coordinates', () => {
  for (const region of ['missing-map', 'orchard']) {
    assert.throws(
      () => placeWorldEncounters([definition], { test: { ...placement, region } }),
      /Invalid map/,
    );
  }
  for (const coordinates of [
    { x: NaN },
    { y: Infinity },
    { x: -1 },
    { x: 1121 },
    { y: -1 },
    { y: 691 },
  ]) {
    assert.throws(
      () => placeWorldEncounters([definition], { test: { ...placement, ...coordinates } }),
      /Invalid coordinates/,
    );
  }
});
test('Every authored placement including training has a reachable approach after bridge repair', () => {
  const encounters = [...worldQuests, trainingClearing];
  assert.equal(encounters.length, Object.keys(meadowPlacements).length);
  for (const encounter of encounters) {
    const target = { x: encounter.x - 30, y: encounter.y + 25 };
    assert.equal(isBlocked(target.x, target.y, true, encounter.region), false, encounter.id);
    const route = findRoute({ x: 160, y: 490 }, target, true, encounter.region);
    assert.ok(route.length > 0, encounter.id);
    assert.ok(
      Math.hypot(route.at(-1)!.x - target.x, route.at(-1)!.y - target.y) < 24,
      encounter.id,
    );
  }
});
