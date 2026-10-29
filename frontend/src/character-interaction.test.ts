import { test } from 'node:test';
import assert from 'node:assert/strict';
import { nearestCharacter } from './navigation.ts';

test('Space selects the nearest character in chat range, regardless of encounter order', () => {
  const farther = { id: 'farther', x: 90, y: 0 };
  const closer = { id: 'closer', x: 20, y: 0 };
  assert.equal(nearestCharacter({ x: 0, y: 0 }, [farther, closer]), closer);
  assert.equal(nearestCharacter({ x: 0, y: 0 }, [closer, farther]), closer);
  assert.equal(nearestCharacter({ x: 0, y: 0 }, [{ x: 111, y: 0 }]), undefined);
  assert.deepEqual(nearestCharacter({ x: 0, y: 0 }, [{ x: 110, y: 0 }]), { x: 110, y: 0 });
  assert.equal(nearestCharacter({ x: 0, y: 0 }, []), undefined);
});
