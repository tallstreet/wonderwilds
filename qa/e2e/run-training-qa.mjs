// Prepared Linux workspace runner: always uses a newly-created disposable synthetic database.
import { spawn, spawnSync } from 'node:child_process';
import { mkdtemp, writeFile, rm } from 'node:fs/promises';
import { randomBytes, createHash } from 'node:crypto';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';
import { join } from 'node:path';
import { once } from 'node:events';
import { createServer } from 'node:net';

const root = fileURLToPath(new URL('../../', import.meta.url));
const pgBin = process.env.WONDERWILDS_PG_BIN || '/workspace/setup-tools/postgres/usr/lib/postgresql/17/bin';
const env = {
  ...process.env,
  CARGO_HOME: process.env.CARGO_HOME || '/workspace/.cargo',
  RUSTUP_HOME: process.env.RUSTUP_HOME || '/workspace/.rustup',
  LD_LIBRARY_PATH: `/workspace/setup-tools/postgres/usr/lib/x86_64-linux-gnu${process.env.LD_LIBRARY_PATH ? `:${process.env.LD_LIBRARY_PATH}` : ''}`,
};
const cargo = join(env.CARGO_HOME, 'bin/cargo');
const db = `ww_training_${Date.now()}_${process.pid}`;
const database = `host=/workspace/setup-tools/pgsocket port=55432 user=agent dbname=${db}`;
const temp = await mkdtemp(join(tmpdir(), 'wonderwilds-training-'));
const children = [];
let databaseCreated = false;
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
  await Promise.all([available(3002), available(5180)]);
  run('sh', ['scripts/start-postgres.sh']);
  run(join(pgBin, 'createdb'), ['-h', '/workspace/setup-tools/pgsocket', '-p', '55432', db]);
  databaseCreated = true;
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
  const api = spawn(join(root, 'target/debug/wonderwilds-api'), [], {
    cwd: root, env: { ...env, WONDERWILDS_DATABASE_URL: database.replace('user=agent', 'user=ww_demo_runtime'), WONDERWILDS_ORIGIN: 'https://wonderwilds.local', WONDERWILDS_PORT: '3002' }, stdio: 'inherit',
  });
  children.push(api);
  await waitReady('http://127.0.0.1:3002/readyz', api);
  const vite = spawn(process.execPath, ['frontend/node_modules/vite/bin/vite.js', 'frontend', '--host', '127.0.0.1', '--port', '5180', '--strictPort'], {
    cwd: root, env: { ...env, WW_LOCAL_DEMO_CONFIG: configPath, WW_DEMO_PORT: '5180', WW_DEMO_API_PORT: '3002' }, stdio: 'inherit',
  });
  children.push(vite);
  await waitReady('http://127.0.0.1:5180/', vite);
  // Async child lets this process continue draining the local servers' output.
  const test = spawn(process.execPath, ['qa/e2e/training-browser.mjs'], { cwd: root, env: { ...env, WW_DEMO_TEST_URL: 'http://127.0.0.1:5180' }, stdio: 'inherit' });
  children.push(test);
  const [code] = await once(test, 'exit');
  if (code !== 0) throw new Error(`Training browser journey exited ${code}`);
} finally {
  for (const child of children.reverse()) {
    if (child.exitCode !== null) continue;
    const stopped = once(child, 'exit');
    child.kill('SIGTERM');
    const timer = setTimeout(() => child.kill('SIGKILL'), 5000);
    await stopped;
    clearTimeout(timer);
  }
  if (databaseCreated) run(join(pgBin, 'dropdb'), ['-h', '/workspace/setup-tools/pgsocket', '-p', '55432', db]);
  await rm(temp, { recursive: true, force: true });
}
