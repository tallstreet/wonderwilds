import { defineConfig } from 'vite';
import { museHandler } from './server/muse.ts';
import { characterHandler } from './server/character.ts';
import { readFileSync } from 'node:fs';
import { request } from 'node:http';
import { fileURLToPath } from 'node:url';

// Opt-in local operator bridge. Production builds have no bridge or credentials.
export default defineConfig(({ command }) => {
  const configPath = command === 'serve' ? process.env.WW_LOCAL_DEMO_CONFIG : undefined;
  const demo = configPath ? JSON.parse(readFileSync(configPath, 'utf8')) : undefined;
  const port = Number(process.env.WW_DEMO_PORT ?? 5173);
  const apiPort = Number(process.env.WW_DEMO_API_PORT ?? 3001);
  const host = `127.0.0.1:${port}`;
  const origin = `http://${host}`;
  const museKey = process.env.BASETEN_API_KEY;
  const authorize = async () => {
    const response = await fetch(`http://127.0.0.1:${apiPort}/api/v1/game`, {
      headers: {
        Cookie: `wonderwilds_session=${demo.session}`,
        'X-Wonderwilds-Profile': demo.profileId,
      },
      signal: AbortSignal.timeout(5000),
    });
    await response.body?.cancel();
    return response.ok;
  };
  const transcribe = museHandler(museKey, authorize);
  const characterKey = process.env.MODEL_API_KEY;
  const character = characterHandler(characterKey, authorize);
  const item = characterHandler(characterKey, authorize, fetch, 'item');
  return {
    build: {
      rollupOptions: {
        input: {
          adventure: fileURLToPath(new URL('./index.html', import.meta.url)),
          characters: fileURLToPath(new URL('./characters.html', import.meta.url)),
          story: fileURLToPath(new URL('./story.html', import.meta.url)),
          demo: fileURLToPath(new URL('./demo.html', import.meta.url)),
        },
      },
    },
    server: { fs: { deny: ['.env', '.env.*', '*.{crt,pem}', '**/.git/**', '**/.local-demo/**'] } },
    plugins: demo
      ? [
          {
            name: 'local-synthetic-demo',
            configureServer(server) {
              server.middlewares.use((req, res, next) => {
                const path = req.url ?? '';
                if (!path.startsWith('/api/') && !path.startsWith('/__demo/')) return next();
                const reject = (status: number) => {
                  res.statusCode = status;
                  res.end();
                };
                if (
                  req.headers.host !== host ||
                  (req.headers.origin && req.headers.origin !== origin) ||
                  (req.headers['sec-fetch-site'] && req.headers['sec-fetch-site'] !== 'same-origin')
                )
                  return reject(403);
                res.setHeader('Cache-Control', 'no-store');
                if (path === '/__demo/bootstrap' && req.method === 'GET') {
                  res.setHeader('Content-Type', 'application/json');
                  return res.end(
                    JSON.stringify({
                      profileId: demo.profileId,
                      csrf: demo.browserToken,
                      voiceInput: Boolean(museKey),
                      characterGeneration: Boolean(characterKey),
                    }),
                  );
                }
                if (
                  path === '/__demo/transcribe' ||
                  path === '/__demo/character' ||
                  path === '/__demo/item'
                ) {
                  if (req.method !== 'POST') return reject(405);
                  if (
                    req.headers.origin !== origin ||
                    req.headers['x-csrf-token'] !== demo.browserToken ||
                    req.headers['x-wonderwilds-profile'] !== demo.profileId
                  )
                    return reject(403);
                  void (
                    path === '/__demo/item'
                      ? item
                      : path === '/__demo/character'
                        ? character
                        : transcribe
                  )(req, res);
                  return;
                }
                if (
                  !/^\/api\/v1\/(game|commands|story|story\/commands|mission-learning|mission-learning\/commands|encounters\/[a-zA-Z0-9-]+)$/.test(
                    path,
                  )
                )
                  return reject(404);
                if (req.method !== 'GET' && req.method !== 'POST') return reject(405);
                if (
                  req.method === 'POST' &&
                  (req.headers.origin !== origin ||
                    req.headers['x-csrf-token'] !== demo.browserToken)
                )
                  return reject(403);
                if (req.headers['x-wonderwilds-profile'] !== demo.profileId) return reject(403);
                const upstream = request(
                  {
                    hostname: '127.0.0.1',
                    port: apiPort,
                    path,
                    method: req.method,
                    headers: {
                      'Content-Type': 'application/json',
                      Accept: 'application/json',
                      Cookie: `wonderwilds_session=${demo.session}`,
                      'X-CSRF-Token': demo.csrf,
                      'X-Wonderwilds-Profile': demo.profileId,
                      Origin: 'https://wonderwilds.local',
                    },
                  },
                  (response) => {
                    res.statusCode = response.statusCode ?? 502;
                    res.setHeader('Content-Type', 'application/json');
                    response.pipe(res);
                  },
                );
                upstream.setTimeout(10000, () => upstream.destroy());
                upstream.on('error', () => {
                  if (!res.headersSent) res.statusCode = 503;
                  res.end();
                });
                req.pipe(upstream);
              });
            },
          },
        ]
      : [],
  };
});
