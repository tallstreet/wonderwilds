// Portable runner: always uses a newly-created disposable synthetic cluster.
import { spawn, spawnSync } from 'node:child_process';
import { mkdtemp, readFile, writeFile, rm } from 'node:fs/promises';
import { randomBytes, createHash } from 'node:crypto';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';
import { join } from 'node:path';
import { once } from 'node:events';
import { createServer } from 'node:net';

const root = fileURLToPath(new URL('../../', import.meta.url));
// The parent wrapper owns a new cluster and stops it on success, failure or signals.
if (!process.argv.includes('--qa-child')) {
  const result = spawnSync('python3', ['scripts/with-qa-postgres.py', process.execPath, fileURLToPath(import.meta.url), '--qa-child'], { cwd: root, stdio: 'inherit', env: process.env });
  if (result.error) throw result.error;
  process.exit(result.status ?? 1);
}
const pgBin = process.env.WONDERWILDS_PG_BIN;
const env = { ...process.env };
const cargo = 'cargo';
const apiPort = env.WW_QA_API_PORT || '3002';
const vitePort = env.WW_QA_VITE_PORT || '5180';
if (env.WW_STORY_COMBINED === '1' && (!env.MODAL_PROXY_TOKEN_ID || !env.MODAL_PROXY_TOKEN_SECRET || env.WW_STORY_EXPECT_PLANS !== '1')) {
  throw new Error('Combined verification requires both Modal providers and saved-plan assertions');
}
const database = process.env.WW_QA_DATABASE_URL;
if (!env.WW_QA_CLUSTER_DIR || database !== `host=${env.WW_QA_CLUSTER_DIR}/socket port=55432 user=ww_qa_admin dbname=ww_qa`) {
  throw new Error('Training QA requires its generated disposable cluster');
}
const temp = await mkdtemp(join(tmpdir(), 'wonderwilds-training-'));
const children = [];
const run = (bin, args, options = {}) => {
  const result = spawnSync(bin, args, { cwd: root, env, stdio: ['pipe', 'inherit', 'inherit'], ...options });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`${bin} exited ${result.status}`);
};
const available = async port => {
  const server = createServer();
  await new Promise((resolve, reject) => { server.once('error', reject); server.listen(port, '127.0.0.1', resolve); });
  await new Promise(resolve => server.close(resolve));
};
const waitReady = async (url, child) => {
  for (let attempt = 0; attempt < 100; attempt++) {
    if (child.exitCode !== null) throw new Error(`Server exited before ready: ${url}`);
    try { if ((await fetch(url)).ok) return; } catch { /* Wait for local process startup. */ }
    await new Promise(resolve => setTimeout(resolve, 200));
  }
  throw new Error(`Timed out waiting for ${url}`);
};
try {
  await Promise.all([available(Number(apiPort)), available(Number(vitePort))]);
  run('sh', ['scripts/migrate-postgres.sh'], { env: { ...env, WONDERWILDS_DATABASE_URL: database } });
  const session = randomBytes(32).toString('hex'), csrf = randomBytes(32).toString('hex');
  const profileId = '10000000-0000-0000-0000-000000000002', parentId = '10000000-0000-0000-0000-000000000001';
  const digest = value => createHash('sha256').update(value).digest('hex');
  const sql = `DO $$ BEGIN IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname='ww_demo_runtime') THEN CREATE ROLE ww_demo_runtime LOGIN NOSUPERUSER NOBYPASSRLS; END IF; END $$;
GRANT USAGE ON SCHEMA wonderwilds TO ww_demo_runtime;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA wonderwilds TO ww_demo_runtime;
GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA wonderwilds TO ww_demo_runtime;
INSERT INTO wonderwilds.parents(id,identity_issuer,identity_subject) VALUES('${parentId}','synthetic-training-qa','training-qa');
INSERT INTO wonderwilds.profiles(id,parent_id,ability_band) VALUES('${profileId}','${parentId}','beginning');
INSERT INTO wonderwilds.sessions(token_digest,csrf_digest,parent_id,expires_at,reauthenticated_at) VALUES(decode('${digest(session)}','hex'),decode('${digest(csrf)}','hex'),'${parentId}',now()+interval '1 hour',now());`;
  run(join(pgBin, 'psql'), ['--dbname', database, '-X', '-q', '-v', 'ON_ERROR_STOP=1'], { input: sql });
  const configPath = join(temp, 'session.json');
  await writeFile(configPath, JSON.stringify({ session, csrf, profileId, browserToken: randomBytes(32).toString('hex') }), { mode: 0o600 });
  if (process.env.WW_QA_SKIP_BUILD !== '1') run(cargo, ['build', '-p', 'wonderwilds-api', '--locked']);
  const apiLog = [];
  const sessionLogs = [];
  const startApi = () => {
    const sessionLog = [];
    sessionLogs.push(sessionLog);
    const child = spawn(join(root, 'target/debug/wonderwilds-api'), [], {
      cwd: root, env: { ...env, WONDERWILDS_DATABASE_URL: database.replace('user=ww_qa_admin', 'user=ww_demo_runtime'), WONDERWILDS_ORIGIN: 'https://wonderwilds.local', WONDERWILDS_PORT: apiPort }, stdio: ['ignore', 'inherit', 'pipe'],
    });
    child.stderr.on('data', chunk => { apiLog.push(chunk.toString()); sessionLog.push(chunk.toString()); process.stderr.write(chunk); });
    children.push(child);
    return child;
  };
  let api = startApi();
  await waitReady(`http://127.0.0.1:${apiPort}/readyz`, api);
  const vite = spawn(process.execPath, ['frontend/node_modules/vite/bin/vite.js', 'frontend', '--host', '127.0.0.1', '--port', vitePort, '--strictPort'], {
    cwd: root, env: { ...env, WW_LOCAL_DEMO_CONFIG: configPath, WW_DEMO_PORT: vitePort, WW_DEMO_API_PORT: apiPort }, stdio: 'inherit',
  });
  children.push(vite);
  await waitReady(`http://127.0.0.1:${vitePort}/`, vite);
  // Async child lets this process continue draining the local servers' output.
  const test = spawn(process.execPath, [process.env.WW_QA_BROWSER_TEST || 'qa/e2e/story-world.mjs'], { cwd: root, env: { ...env, WW_DEMO_TEST_URL: `http://127.0.0.1:${vitePort}` }, stdio: ['inherit', 'inherit', 'inherit', 'ipc'] });
  children.push(test);
  test.on('message', async message => {
    if (message?.type !== 'restart-api' || !(env.WW_STORY_COMBINED === '1' || env.WW_STORY_RETURNING === '1' || env.WW_LONG_PLAY === '1')) return;
    try {
      const stopped = once(api, 'exit');
      api.kill('SIGTERM');
      const timer = setTimeout(() => api.kill('SIGKILL'), 5000);
      await stopped;
      clearTimeout(timer);
      api = startApi();
      await waitReady(`http://127.0.0.1:${apiPort}/readyz`, api);
      test.send({ type: 'api-restarted' });
    } catch { test.send({ type: 'api-restart-failed' }); }
  });
  const [code] = await once(test, 'exit');
  if (env.WW_STORY_COMBINED === '1') {
    const log = apiLog.join('');
    const summary = {
      branch: env.WW_STORY_BRANCHES || 'default',
      model: 'wonderwilds',
      modelRevision: 'Endpoint alias; immutable weight revision unavailable',
      apiTimeoutSeconds: 6,
      sourceSha256: Object.fromEntries(await Promise.all([
        'contracts/schemas/v1.json', 'backend/crates/api/src/story/modal.rs',
        'backend/crates/api/src/story/modal_prompt.txt',
        'backend/crates/director/src/story/prompts/mission_baseline.txt',
        'backend/crates/director/src/story/prompts/mission_dependencies.txt',
        'qa/e2e/story-broader.mjs', 'qa/e2e/run-training-qa.mjs',
      ].map(async path => [path, createHash('sha256').update(await readFile(join(root, path))).digest('hex')]))),
      acceptedSceneResponses: (log.match(/Modal story: validated scene accepted/g) || []).length,
      fallbackSceneResponses: (log.match(/Modal story: unavailable or invalid/g) || []).length,
      staleSceneResponses: (log.match(/Modal story: state changed/g) || []).length,
      acceptedPlans: (log.match(/Modal planner: validated plan/g) || []).length,
      fallbackPlans: (log.match(/Modal planner: .*authored fallback/g) || []).length,
      browserExitCode: code,
      acceptedScenesBySession: sessionLogs.map(parts => (parts.join('').match(/Modal story: validated scene accepted/g) || []).length),
      note: 'Scene counts include cache hits and QA reads; they are not unique inference counts.'
    };
    if (env.WW_STORY_REPORT) await writeFile(env.WW_STORY_REPORT, JSON.stringify(summary, null, 2));
    console.log('Combined provider evidence:', JSON.stringify(summary));
    if (summary.acceptedScenesBySession.length !== 4 || summary.acceptedScenesBySession.some(count => count === 0) || summary.acceptedPlans !== 7) throw new Error('Combined run requires accepted scenes in all four sessions and seven generated plans');
  }
  if (code !== 0) throw new Error(`Training browser journey exited ${code}`);
} finally {
  for (const child of children.reverse()) {
    if (child.exitCode !== null || child.signalCode !== null) continue;
    const stopped = once(child, 'exit');
    child.kill('SIGTERM');
    const timer = setTimeout(() => child.kill('SIGKILL'), 5000);
    await stopped;
    clearTimeout(timer);
  }
  await rm(temp, { recursive: true, force: true });
}
