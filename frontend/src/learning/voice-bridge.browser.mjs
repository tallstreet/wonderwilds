// Local HTTP integration; all provider responses and identity data are synthetic.
import assert from 'node:assert/strict';
import { createServer as httpServer, request } from 'node:http';
import { mkdtemp, writeFile, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createServer } from 'vite';
const directory = await mkdtemp(join(tmpdir(), 'ww-voice-'));
const originalFetch = globalThis.fetch;
let authenticated = true;
const api = httpServer((req, res) => {
  assert.equal(req.headers.cookie, 'wonderwilds_session=synthetic-session');
  assert.equal(req.headers['x-wonderwilds-profile'], 'synthetic-profile');
  if (req.url === '/api/v1/game') {
    res.statusCode = authenticated ? 200 : 401;
    return res.end('{}');
  }
  if (req.url === '/api/v1/mission-learning') {
    assert.equal(req.method, 'GET');
    assert.equal(req.headers['x-csrf-token'], 'synthetic-api-csrf');
    res.statusCode = 200;
    return res.end(
      '{"schemaVersion":"1.0","contentVersion":"draft","policyVersion":"frontier-v1","policyExplanation":"Practice is a guide.","revision":0,"available":true,"skills":[]}',
    );
  }
  if (req.url === '/api/v1/mission-learning/commands') {
    assert.equal(req.method, 'POST');
    assert.equal(req.headers['x-csrf-token'], 'synthetic-api-csrf');
    let body = '';
    req.setEncoding('utf8');
    req.on('data', (chunk) => {
      body += chunk;
    });
    req.on('end', () => {
      res.statusCode = 200;
      res.end(body);
    });
    return;
  }
  res.statusCode = 404;
  res.end();
});
await new Promise((resolve) => api.listen(0, '127.0.0.1', resolve));
process.env.WW_DEMO_API_PORT = String(api.address().port);
process.env.WW_DEMO_PORT = '5191';
process.env.WW_LOCAL_DEMO_CONFIG = join(directory, 'session.json');
process.env.BASETEN_API_KEY = 'synthetic-provider-key';
await writeFile(
  process.env.WW_LOCAL_DEMO_CONFIG,
  JSON.stringify({
    profileId: 'synthetic-profile',
    browserToken: 'synthetic-csrf',
    csrf: 'synthetic-api-csrf',
    session: 'synthetic-session',
  }),
);
let providerCalls = 0;
globalThis.fetch = async (url, options) => {
  if (String(url).startsWith('https://inference.baseten.co/')) {
    providerCalls++;
    const payload = JSON.parse(options.body);
    assert.equal(payload.model, 'meta-models/muse-spark-open-1026');
    assert.equal(options.headers.Authorization, 'Bearer synthetic-provider-key');
    return Response.json({ choices: [{ message: { content: 'three' } }] });
  }
  return originalFetch(url, options);
};
const vite = await createServer({
  root: 'frontend',
  optimizeDeps: { noDiscovery: true, include: [] },
  server: { host: '127.0.0.1', port: 5191, strictPort: true },
});
try {
  await vite.listen();
  const origin = 'http://127.0.0.1:5191';
  const bootstrap = await originalFetch(`${origin}/__demo/bootstrap`).then((r) => r.json());
  assert.equal(bootstrap.voiceInput, true);
  assert.equal(JSON.stringify(bootstrap).includes('provider-key'), false);
  const learner = await originalFetch(`${origin}/api/v1/mission-learning`, {
    headers: { 'X-Wonderwilds-Profile': 'synthetic-profile' },
  });
  assert.equal(learner.status, 200, 'development bridge forwards the scoped learning snapshot');
  assert.equal(learner.headers.get('cache-control'), 'no-store');
  assert.equal((await learner.json()).policyVersion, 'frontier-v1');
  const commandBody = JSON.stringify({
    schemaVersion: '1.0',
    commandId: 'synthetic',
    expectedRevision: 0,
    payload: { type: 'begin' },
  });
  const learningCommand = await originalFetch(`${origin}/api/v1/mission-learning/commands`, {
    method: 'POST',
    headers: {
      Origin: origin,
      'Content-Type': 'application/json',
      'X-CSRF-Token': 'synthetic-csrf',
      'X-Wonderwilds-Profile': 'synthetic-profile',
    },
    body: commandBody,
  });
  assert.equal(
    learningCommand.status,
    200,
    'development bridge forwards protected learning commands',
  );
  assert.equal(await learningCommand.text(), commandBody);
  assert.equal(
    (
      await originalFetch(`${origin}/api/v1/mission-learning/commands`, {
        method: 'POST',
        headers: {
          Origin: 'https://other.invalid',
          'X-CSRF-Token': 'synthetic-csrf',
          'X-Wonderwilds-Profile': 'synthetic-profile',
        },
        body: commandBody,
      })
    ).status,
    403,
    'development bridge rejects cross-origin learning commands',
  );
  assert.equal(
    (
      await originalFetch(`${origin}/api/v1/mission-learning`, {
        headers: { 'X-Wonderwilds-Profile': 'other-profile' },
      })
    ).status,
    403,
    'development bridge rejects another profile',
  );
  const headers = {
    Origin: origin,
    'Content-Type': 'audio/wav',
    'X-CSRF-Token': 'synthetic-csrf',
    'X-Wonderwilds-Profile': 'synthetic-profile',
  };
  // Minimal mono 16kHz PCM WAV, containing silence; no real audio.
  const wav = Buffer.alloc(46);
  wav.write('RIFF');
  wav.writeUInt32LE(38, 4);
  wav.write('WAVEfmt ', 8);
  wav.writeUInt32LE(16, 16);
  wav.writeUInt16LE(1, 20);
  wav.writeUInt16LE(1, 22);
  wav.writeUInt32LE(16000, 24);
  wav.writeUInt32LE(32000, 28);
  wav.writeUInt16LE(2, 32);
  wav.writeUInt16LE(16, 34);
  wav.write('data', 36);
  wav.writeUInt32LE(2, 40);
  const post = (overrides = {}, body = wav) =>
    new Promise((resolve, reject) => {
      const req = request(
        `${origin}/__demo/transcribe`,
        { method: 'POST', headers: { ...headers, ...overrides } },
        (res) => {
          const chunks = [];
          res.on('data', (chunk) => chunks.push(chunk));
          res.on('end', () =>
            resolve(
              new Response(Buffer.concat(chunks), { status: res.statusCode, headers: res.headers }),
            ),
          );
        },
      );
      req.on('error', reject);
      req.end(body);
    });
  for (const override of [
    { Origin: 'https://other.invalid' },
    { 'X-CSRF-Token': '' },
    { 'X-Wonderwilds-Profile': 'other' },
    { 'Sec-Fetch-Site': 'cross-site' },
    { Host: 'other.invalid' },
  ])
    assert.equal((await post(override)).status, 403, JSON.stringify(override));
  assert.equal((await originalFetch(`${origin}/__demo/transcribe`)).status, 405);
  assert.equal((await post({ 'Content-Type': 'video/webm' })).status, 415);
  assert.equal(providerCalls, 0);
  const result = await post();
  assert.equal(result.status, 200);
  assert.equal(result.headers.get('cache-control'), 'no-store');
  assert.equal(await result.text(), 'three');
  assert.equal(providerCalls, 1);
  assert.equal((await post()).status, 429);
  await new Promise((resolve) => setTimeout(resolve, 2050));
  authenticated = false;
  assert.equal((await post()).status, 401);
  assert.equal(providerCalls, 1);
  await new Promise((resolve) => setTimeout(resolve, 2050));
  authenticated = true;
  assert.equal((await post({}, Buffer.from('not audio'))).status, 400);
  assert.equal(providerCalls, 1);
  console.log(
    'Voice bridge PASS: protected route, session verification, model alias, transcription, rate limit and malformed audio. No live provider calls.',
  );
} finally {
  await vite.close();
  api.closeAllConnections();
  await new Promise((resolve) => api.close(resolve));
  globalThis.fetch = originalFetch;
  await rm(directory, { recursive: true, force: true });
}
