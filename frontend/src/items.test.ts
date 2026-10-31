import assert from 'node:assert/strict';
import test from 'node:test';
import { itemArt, items } from './items.ts';
import { generateCharacter } from '../server/character.ts';

test('item generation uses repository props and a single isolated item rather than a character sheet', async () => {
  const png = Buffer.alloc(24);
  Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]).copy(png);
  png.writeUInt32BE(1024, 16);
  png.writeUInt32BE(1024, 20);
  let called = false;
  const fetcher: typeof fetch = async (_url, init) => {
    called = true;
    const body = JSON.parse(String(init?.body));
    assert.equal(body.size, '1024x1024');
    assert.match(body.images[0].image_url, /^data:image\/png;base64,/);
    assert.match(body.prompt, /ONE isolated inventory item/);
    assert.match(body.prompt, /repository prop atlas/);
    assert.match(body.prompt, /seashell/);
    assert.equal(body.tool_enablement.enable_web_search, false);
    return Response.json({ data: [{ b64_json: png.toString('base64') }] });
  };
  assert.deepEqual(
    await generateCharacter(
      items.seashell.description,
      'synthetic',
      new AbortController().signal,
      fetcher,
      'item',
    ),
    png,
  );
  assert.ok(called);
});

test('unknown and malicious item IDs have safe bundled art', () => {
  assert.match(itemArt('wood'), /sprite-art/);
  assert.match(itemArt('__proto__'), /sprite-art/);
  assert.equal(itemArt('<img src=x onerror=alert(1)>').includes('onerror'), false);
});
