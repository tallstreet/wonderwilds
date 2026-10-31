import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
  customFacing,
  customFrame,
  validSavedCharacter,
  inspectCharacterCells,
} from './custom-character.ts';
import { generateCharacter, validDescription } from '../server/character.ts';

test('custom characters face movement and alternate walking feet, preserving idle direction', () => {
  for (const [direction, dx, dy, row] of [
    ['down', 0, 1, 0],
    ['left', -1, 0, 1],
    ['right', 1, 0, 2],
    ['up', 0, -1, 3],
  ] as const) {
    assert.equal(customFacing(dx, dy, 'down'), direction);
    assert.equal(customFacing(0, 0, direction), direction);
    assert.equal(customFrame(direction, 0, false, false), row * 3);
    assert.equal(customFrame(direction, 0.1, true, false), row * 3 + 1);
    assert.equal(customFrame(direction, 0.6, true, false), row * 3 + 2);
    assert.equal(customFrame(direction, 0.6, true, true), row * 3);
  }
});
test('untrusted descriptions and saved image sources are bounded', () => {
  for (const value of [null, {}, '', '  ', 'a'.repeat(401)])
    assert.equal(validDescription(value), false);
  assert.equal(validDescription('a blue fox'), true);
  for (const value of [
    null,
    {},
    'https://example.com/x.png',
    'data:image/svg+xml;base64,AAAA',
    'data:image/png;base64,<script>',
  ])
    assert.equal(validSavedCharacter(value), false);
  assert.equal(validSavedCharacter('data:image/png;base64,AAAA'), true);
});
test('Muse request uses fixed model, layout and server credentials; invalid images fail', async () => {
  const png = Buffer.alloc(24);
  Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]).copy(png);
  png.writeUInt32BE(768, 16);
  png.writeUInt32BE(1024, 20);
  const fetcher: typeof fetch = async (url, options) => {
    assert.equal(url, 'https://api.meta.ai/v1/images/edits');
    assert.equal((options!.headers as Record<string, string>).Authorization, 'Bearer test-key');
    const body = JSON.parse(options!.body as string);
    assert.equal(body.model, 'muse-image-1.0');
    assert.equal(body.n, 1);
    assert.equal(body.images.length, 1);
    assert.match(body.images[0].image_url, /^data:image\/png;base64,iVBOR/);
    assert.equal(body.reasoning_strength, 'high');
    assert.match(body.prompt, /rounded 3D animated toy-like forms/);
    assert.match(body.prompt, /20% blank magenta margin on ALL four sides of EVERY cell/);
    assert.match(body.prompt, /central 60% of BOTH cell width and cell height/);
    assert.match(body.prompt, /feet at 78% of cell height/);
    assert.match(body.prompt, /row 2, column 3 the nose points to the LEFT edge/);
    assert.match(body.prompt, /row 3, column 3 the nose points to the RIGHT edge/);
    assert.match(body.prompt, /Changing the foot phase must NEVER turn or mirror the character/);
    assert.equal(body.output_format, 'png');
    assert.equal(body.tool_enablement.enable_web_search, false);
    assert.match(body.prompt, /3 equal columns and 4 equal rows/);
    assert.match(body.prompt, /blue fox/);
    return Response.json({ data: [{ b64_json: png.toString('base64') }] });
  };
  assert.deepEqual(
    await generateCharacter('blue fox', 'test-key', new AbortController().signal, fetcher),
    png,
  );
  for (const body of [
    { data: [{ url: 'https://example.com/evil' }] },
    { data: [{ b64_json: 'AAAA' }] },
  ]) {
    await assert.rejects(
      generateCharacter('fox', 'key', new AbortController().signal, async () =>
        Response.json(body),
      ),
    );
  }
});

test('sheet diagnostics distinguish empty, background-filled and edge-touching cells', () => {
  const width = 96,
    height = 128;
  const pixels = new Uint8ClampedArray(width * height * 4);
  let cells = inspectCharacterCells(pixels, width, height);
  assert.equal(cells.length, 12);
  assert.deepEqual(cells[0].issues, ['too little visible artwork']);
  assert.equal(cells[0].bounds, null);
  for (let y = 0; y < 32; y++) for (let x = 0; x < 32; x++) pixels[(y * width + x) * 4 + 3] = 255;
  for (let y = 36; y < 56; y++) for (let x = 36; x < 56; x++) pixels[(y * width + x) * 4 + 3] = 255;
  cells = inspectCharacterCells(pixels, width, height);
  assert.equal(cells[0].occupiedPercent, 100);
  assert.match(cells[0].issues.join(' '), /background still fills/);
  assert.match(cells[0].issues.join(' '), /left, top, right, bottom/);
  assert.deepEqual(cells[4].issues, []);
  assert.deepEqual(cells[4].bounds, { x: 36, y: 36, width: 20, height: 20 });
  assert.equal(cells[4].facing, 'left');
  assert.equal(cells[4].pose, 'left step');
  assert.deepEqual(cells[11].cell, { x: 64, y: 96, width: 32, height: 32 });
});
