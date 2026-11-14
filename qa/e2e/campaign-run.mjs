// Each starter gets an isolated synthetic database. Never uses persistent demo data.
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../../', import.meta.url));
const starters = process.env.WW_CAMPAIGN_STARTERS?.split(',') ?? ['pip', 'tansy', 'nib'];
for (const starter of starters) {
  const env = { ...process.env };
  for (const name of ['MODAL_PROXY_TOKEN_ID', 'MODAL_PROXY_TOKEN_SECRET', 'BASETEN_API_KEY', 'MODEL_API_KEY']) delete env[name];
  Object.assign(env, {
    WW_QA_BROWSER_TEST: process.env.WW_QA_BROWSER_TEST ?? 'qa/e2e/campaign-journey.mjs',
    WW_CAMPAIGN_STARTER: starter,
    WW_LONG_PLAY: '1',
    WW_QA_VITE_CONFIG: fileURLToPath(new URL('./campaign-vite.config.mjs', import.meta.url)),
    CHROMIUM_PATH: process.env.CHROMIUM_PATH ?? '/usr/bin/chromium',
  });
  console.log(`Campaign isolated no-provider journey: ${starter}`);
  const result = spawnSync(process.execPath, ['qa/e2e/run-training-qa.mjs'], {
    cwd: root, env, stdio: 'inherit',
  });
  if (result.error) throw result.error;
  if (result.status !== 0) process.exit(result.status ?? 1);
}
