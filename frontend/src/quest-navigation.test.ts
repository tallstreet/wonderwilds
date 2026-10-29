import { test } from 'node:test';
import assert from 'node:assert/strict';
import { nextQuestExit, questDirection, questWaypoint } from './quest-navigation.ts';
import { maps } from './maps.ts';

test('Quest guidance uses connected exits in both directions, never a destination spawn', () => {
  assert.equal(nextQuestExit('meadow', 'orchard'), 'east');
  assert.equal(nextQuestExit('orchard', 'meadow'), 'west');
  assert.equal(nextQuestExit('meadow', 'meadow'), undefined);
  assert.equal(nextQuestExit('__proto__', 'meadow'), undefined);
  assert.match(questDirection('meadow', 'orchard'), /east path to Starlight Cove/);
  assert.deepEqual(questWaypoint('meadow', { region: 'orchard', x: 920, y: 323 }), {
    point: { x: 1072, y: 345 },
    arrived: false,
  });
  assert.deepEqual(questWaypoint('orchard', { region: 'orchard', x: 920, y: 323 }), {
    point: { x: 890, y: 348 },
    arrived: true,
  });
});
test('Guidance reaches every destination through adjacent maps without loops', () => {
  for (const destination of Object.keys(maps)) {
    let current = 'meadow';
    const visited = new Set<string>();
    while (current !== destination) {
      assert.ok(!visited.has(current));
      visited.add(current);
      const side = nextQuestExit(current, destination);
      assert.ok(side);
      current = maps[current][side];
    }
  }
});
