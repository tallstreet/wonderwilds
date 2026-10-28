import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import inventory from '../../content/creatures/art/library.v1.json';
import { characterArtwork } from './character-art.ts';
import { characterFrame } from './character-sprites.ts';

test('All twelve library portraits and world frames match the inspected PNG inventory', () => {
  let count = 0;
  for (const sheet of inventory.sheets) {
    const png = readFileSync(new URL(`../../${sheet.path}`, import.meta.url));
    assert.equal(createHash('sha256').update(png).digest('hex'), sheet.sha256);
    const width = png.readUInt32BE(16),
      height = png.readUInt32BE(20);
    assert.deepEqual([width, height], sheet.dimensions);
    for (const entry of sheet.characters) {
      const art = characterArtwork(entry.creatureId)!;
      assert.ok(art);
      assert.deepEqual([art.x, art.y, art.width, art.height], entry.frame);
      assert.deepEqual(characterFrame(entry.cell, width, height, sheet.regionId), entry.frame);
      count++;
    }
  }
  assert.equal(count, 12);
});
