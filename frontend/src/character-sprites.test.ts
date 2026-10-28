import test from 'node:test';
import assert from 'node:assert/strict';
import roster from '../../content/creatures/friends.v1.json';
import { characterFrame, characterSprite, availableCharacter } from './character-sprites.ts';
import { friendSprites, worldQuests } from './backend-world-model.ts';

test('Every authored friend has its own regional texture/frame', () => {
  for (const creature of roster.creatures) {
    const sprite = characterSprite(creature.id);
    assert.ok(sprite);
    assert.equal(sprite.frame, creature.id);
    assert.equal(sprite.texture, `friends-${creature.regionId}`);
    assert.equal(sprite.species, creature.species);
  }
  for (const id of ['pip', 'tansy', 'nib', 'flutter']) {
    assert.equal(friendSprites[id], id);
    assert.equal(worldQuests.find((q) => q.id === `befriend-${id}`)?.sprite, id);
  }
  assert.equal(characterSprite('rose'), undefined);
});

test('Four sheet frames tile even and odd PNG dimensions without gaps or overflow', () => {
  for (const [width, height] of [
    [2048, 2048],
    [1536, 1024],
    [1775, 887],
  ]) {
    const frames = [0, 1, 2, 3].map((index) => characterFrame(index, width, height));
    assert.equal(
      frames.reduce((sum, frame) => sum + frame[2] * frame[3], 0),
      width * height,
    );
    for (const [x, y, w, h] of frames) {
      assert.ok(w > 0 && h > 0 && x >= 0 && y >= 0 && x + w <= width && y + h <= height);
    }
    assert.equal(frames[0][0] + frames[0][2], frames[1][0]);
    assert.equal(frames[0][1] + frames[0][3], frames[2][1]);
  }
  assert.throws(() => characterFrame(4, 1024, 1024));
});

test('Missing textures and frames safely decline character art', () => {
  assert.equal(
    availableCharacter('tansy', {
      exists: () => false,
      get: () => {
        throw new Error('must not read missing texture');
      },
    }),
    undefined,
  );
  assert.equal(
    availableCharacter('tansy', { exists: () => true, get: () => ({ has: () => false }) }),
    undefined,
  );
  assert.equal(
    availableCharacter('unknown', { exists: () => true, get: () => ({ has: () => true }) }),
    undefined,
  );
  assert.equal(
    availableCharacter('tansy', { exists: () => true, get: () => ({ has: () => true }) })?.frame,
    'tansy',
  );
});

test('Corrected Orchard sheet uses an equal grid with clear gutters', () => {
  assert.deepEqual(characterFrame(0, 1254, 1254, 'orchard'), [0, 0, 627, 627]);
  assert.deepEqual(characterFrame(2, 1254, 1254, 'orchard'), [0, 627, 627, 627]);
  assert.deepEqual(characterFrame(0, 1254, 1254, 'meadow'), [0, 0, 627, 627]);
  assert.equal(
    [0, 1, 2, 3]
      .map((i) => characterFrame(i, 1254, 1254, 'orchard'))
      .reduce((sum, f) => sum + f[2] * f[3], 0),
    1254 * 1254,
  );
});

test('Inspected Cove column preserves Luna cape and Glimmer wings', () => {
  assert.deepEqual(characterFrame(0, 1254, 1254, 'cove'), [0, 0, 660, 627]);
  assert.deepEqual(characterFrame(1, 1254, 1254, 'cove'), [660, 0, 594, 627]);
  assert.deepEqual(characterFrame(2, 1254, 1254, 'cove'), [0, 627, 660, 627]);
});
