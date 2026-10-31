// Real local bridge, synthetic session and mocked Meta endpoint; no paid requests.
import assert from 'node:assert/strict';
import { createServer as httpServer, request } from 'node:http';
import { mkdtemp, writeFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createServer } from 'vite';
const directory = await mkdtemp(join(tmpdir(), 'ww-character-'));
const originalFetch = globalThis.fetch;
let authenticated = true,
  providerCalls = 0;
const api = httpServer((req, res) => {
  assert.equal(req.headers.cookie, 'wonderwilds_session=synthetic-session');
  assert.equal(req.headers['x-wonderwilds-profile'], 'synthetic-profile');
  res.statusCode = authenticated ? 200 : 401;
  res.end('{}');
});
await new Promise((resolve) => api.listen(0, '127.0.0.1', resolve));
process.env.WW_DEMO_API_PORT = String(api.address().port);
process.env.WW_DEMO_PORT = '5195';
process.env.WW_LOCAL_DEMO_CONFIG = join(directory, 'session.json');
process.env.MODEL_API_KEY = 'synthetic-meta-key';
await writeFile(
  process.env.WW_LOCAL_DEMO_CONFIG,
  JSON.stringify({
    profileId: 'synthetic-profile',
    browserToken: 'synthetic-csrf',
    csrf: 'synthetic-api-csrf',
    session: 'synthetic-session',
  }),
);
const png = Buffer.alloc(24);
Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]).copy(png);
png.writeUInt32BE(768, 16);
png.writeUInt32BE(1024, 20);
globalThis.fetch = async (url, options) => {
  if (String(url) === 'https://api.meta.ai/v1/images/edits') {
    providerCalls++;
    assert.equal(options.headers.Authorization, 'Bearer synthetic-meta-key');
    const payload = JSON.parse(options.body);
    assert.equal(payload.model, 'muse-image-1.0');
    assert.match(payload.images[0].image_url, /^data:image\/png;base64,iVBOR/);
    assert.match(payload.prompt, /rendering-style reference/);
    if (payload.size === '1024x1024') assert.match(payload.prompt, /ONE isolated inventory item/);
    assert.equal(JSON.stringify(payload).includes('synthetic-profile'), false);
    return Response.json({ data: [{ b64_json: png.toString('base64') }] });
  }
  return originalFetch(url, options);
};
const vite = await createServer({
  root: 'frontend',
  optimizeDeps: { noDiscovery: true, include: [] },
  server: { host: '127.0.0.1', port: 5195, strictPort: true },
});
const origin = 'http://127.0.0.1:5195';
const headers = {
  Origin: origin,
  'Content-Type': 'application/json',
  'X-CSRF-Token': 'synthetic-csrf',
  'X-Wonderwilds-Profile': 'synthetic-profile',
};
const post = (overrides = {}, body = { description: 'A blue fox' }) =>
  originalFetch(`${origin}/__demo/character`, {
    method: 'POST',
    headers: { ...headers, ...overrides },
    body: JSON.stringify(body),
  });
try {
  await vite.listen();
  const bootstrap = await originalFetch(`${origin}/__demo/bootstrap`).then((r) => r.json());
  assert.equal(bootstrap.characterGeneration, true);
  assert.equal(JSON.stringify(bootstrap).includes('meta-key'), false);
  assert.equal((await originalFetch(`${origin}/__demo/character`)).status, 405);
  for (const override of [
    { Origin: 'https://evil.example' },
    { 'X-CSRF-Token': 'wrong' },
    { 'X-Wonderwilds-Profile': 'wrong' },
    { 'Sec-Fetch-Site': 'cross-site' },
  ])
    assert.equal((await post(override)).status, 403);
  // Host is tested through raw HTTP so the transport cannot rewrite it.
  const hostStatus = await new Promise((resolve) => {
    const req = request(
      `${origin}/__demo/character`,
      { method: 'POST', headers: { ...headers, Host: 'evil.example' } },
      (res) => {
        res.resume();
        resolve(res.statusCode);
      },
    );
    req.end('{}');
  });
  assert.equal(hostStatus, 403);
  assert.equal((await post({ 'Content-Type': 'text/plain' })).status, 415);
  for (const description of ['', ' ', 'a'.repeat(401)])
    assert.equal((await post({}, { description })).status, 400);
  assert.equal((await post({}, { description: 'x', profile: 'forged' })).status, 400);
  assert.equal((await post({}, { description: 'a'.repeat(5000) })).status, 413);
  authenticated = false;
  assert.equal((await post()).status, 401);
  assert.equal(providerCalls, 0);
  authenticated = true;
  const result = await post();
  assert.equal(result.status, 200);
  assert.equal(result.headers.get('cache-control'), 'no-store');
  assert.equal(result.headers.get('content-type'), 'image/png');
  assert.deepEqual(Buffer.from(await result.arrayBuffer()), png);
  assert.equal(providerCalls, 1);
  assert.equal((await post()).status, 429);
  assert.equal(providerCalls, 1);
  const itemPost = (body = { itemId: 'seashell' }, overrides = {}) =>
    originalFetch(`${origin}/__demo/item`, {
      method: 'POST',
      headers: { ...headers, ...overrides },
      body: JSON.stringify(body),
    });
  assert.equal((await itemPost({}, { 'X-CSRF-Token': 'wrong' })).status, 403);
  assert.equal((await itemPost({ itemId: '__proto__' })).status, 400);
  assert.equal((await itemPost({ description: 'invent rewards' })).status, 400);
  authenticated = false;
  assert.equal((await itemPost()).status, 401);
  authenticated = true;
  assert.equal((await itemPost()).status, 200);
  assert.equal(providerCalls, 2);
  assert.equal((await itemPost()).status, 429);
  console.log(
    'PASS: item and character bridge Host/Origin/CSRF/profile/session, bounded input, fixed Meta model, no secret exposure and request throttling',
  );
} finally {
  globalThis.fetch = originalFetch;
  await vite.close();
  await new Promise((resolve) => api.close(resolve));
  await rm(directory, { recursive: true, force: true });
}
