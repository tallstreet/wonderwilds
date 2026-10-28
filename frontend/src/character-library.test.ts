import assert from 'node:assert/strict';
import { test } from 'node:test';
import roster from '../../content/creatures/friends.v1.json';
import { characterArtwork, characterPortrait, characterSheets } from './character-art.ts';
import {
  characterCard,
  charactersForRegion,
  escapeCharacterText,
} from './character-library-model.ts';

test('all twelve friends resolve to a distinct sheet cell in their authored region', () => {
  const cells = new Set<string>();
  for (const friend of roster.creatures) {
    const art = characterArtwork(friend.id);
    assert.ok(art);
    assert.equal(art.path, `/assets/characters/${friend.regionId}.png`);
    assert.ok([0, 1].includes(art.column));
    assert.ok([0, 1].includes(art.row));
    cells.add(`${art.path}:${art.column}:${art.row}`);
  }
  assert.equal(cells.size, 12);
  assert.equal(Object.values(characterSheets).flatMap((sheet) => sheet.ids).length, 12);
  assert.equal(characterArtwork('rose'), undefined, 'avatars are separate from friends');
  assert.equal(characterPortrait('invented'), '');
});

test('corrected Orchard sheet uses a clean equal row split', () => {
  const cleo = characterArtwork('cleo')!;
  const patter = characterArtwork('patter')!;
  assert.equal(cleo.height, 627);
  assert.equal(patter.y, 627);
  assert.equal(patter.height, 627);
  assert.equal(cleo.height + patter.height, cleo.sheetHeight);
  assert.ok(characterPortrait('cleo').includes('aspect-ratio:627/627'));
});

test('Cove column gutter preserves the wide wing silhouettes', () => {
  const glimmer = characterArtwork('glimmer')!;
  const sail = characterArtwork('sail')!;
  assert.equal(glimmer.width, 660);
  assert.equal(sail.x, 660);
  assert.equal(sail.width, 594);
  assert.equal(glimmer.width + sail.width, glimmer.sheetWidth);
});

test('region filtering covers the entire roster once and cards contain character development', () => {
  assert.equal(charactersForRegion('all').length, 12);
  const grouped = ['meadow', 'orchard', 'cove'] as const;
  const ids = grouped.flatMap((region) => {
    const friends = charactersForRegion(region);
    assert.equal(friends.length, 4);
    return friends.map((friend) => friend.id);
  });
  assert.equal(new Set(ids).size, 12);
  for (const friend of roster.creatures) {
    const card = characterCard(friend.id);
    assert.ok(card.includes(`id="friend-${friend.id}"`));
    assert.ok(card.includes(friend.name));
    assert.ok(card.includes(escapeCharacterText(friend.habitatClue)));
    assert.ok(card.includes('How they speak'));
    assert.ok(card.includes('<details>'));
    assert.ok(!card.includes('friendship points'));
  }
  assert.equal(characterCard('invented'), '');
});

test('authored text is escaped and artwork markup rejects untrusted IDs and sizes', () => {
  assert.equal(escapeCharacterText('<script>"&\''), '&lt;script&gt;&quot;&amp;&#39;');
  assert.equal(characterPortrait('" onerror="alert(1)'), '');
  assert.ok(characterPortrait('pip', Number.NaN).includes('--portrait-size:160px'));
  assert.ok(characterPortrait('pip', 100000).includes('--portrait-size:600px'));
  assert.ok(characterPortrait('pip', -1).includes('--portrait-size:24px'));
});
