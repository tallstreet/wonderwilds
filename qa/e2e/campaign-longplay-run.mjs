// Six isolated no-provider cases, two at a time; every case owns a fresh DB.
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { fileURLToPath } from 'node:url';
import { open } from 'node:fs/promises';
const root = fileURLToPath(new URL('../../', import.meta.url));
const starters = process.env.WW_CAMPAIGN_STARTERS?.split(',') ?? ['pip', 'tansy', 'nib'];
const orders = process.env.WW_CAMPAIGN_ORDERS?.split(',') ?? ['harvest', 'current'];
const cases = starters.flatMap(starter => orders.map(order => ({ starter, order })));
let next = 0;
let failed = false;
const results = await Promise.allSettled([0, 1].map(async worker => {
  while (!failed && next < cases.length) {
    const { starter, order } = cases[next++];
    const log = `/tmp/${process.env.WW_CAMPAIGN_STAGE ? 'campaign-' + process.env.WW_CAMPAIGN_STAGE : 'campaign'}-longplay-${starter}-${order}.log`;
    const file = await open(log, 'w');
    console.log(`Starting ${starter}/${order}: ${log}`);
    const child = spawn(process.execPath, ['qa/e2e/campaign-run.mjs'], {
      cwd: root,
      env: { ...process.env, WW_QA_BROWSER_TEST: 'qa/e2e/campaign-longplay.mjs', WW_CAMPAIGN_STARTERS: starter, WW_CAMPAIGN_ORDER: order, WW_QA_API_PORT: String(3010 + worker), WW_QA_VITE_PORT: String(5190 + worker) },
      stdio: ['ignore', file.fd, file.fd],
    });
    const [code] = await once(child, 'exit');
    await file.close();
    if (code !== 0) { failed = true; throw new Error(`${starter}/${order} failed (${code}); inspect ${log}`); }
    console.log(`Passed ${starter}/${order}`);
  }
}));

for (const result of results) if (result.status === "rejected") console.error(result.reason);
if (failed) process.exitCode = 1;
