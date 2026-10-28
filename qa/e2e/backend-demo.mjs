// Use a fresh isolated QA database, never the user's demo database.
import assert from 'node:assert/strict';
import { chromium } from 'playwright';
const origin = process.env.WW_DEMO_TEST_URL ?? 'http://127.0.0.1:5180';
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH });
const page = await browser.newPage();
const errors = [];
page.on('pageerror', e => errors.push(e.message));
try {
  await page.goto(`${origin}/demo.html`);
  await page.getByRole('status').filter({ hasText: 'Connected' }).waitFor();
  const snapshot = async () => JSON.parse(await page.locator('pre').textContent());
  assert.equal((await snapshot()).revision, 0, 'Use a fresh QA database');
  const click = name => page.getByRole('button', { name, exact: true }).click();
  const activity = () => page.getByRole('heading', { name: 'Help with an adventure' }).waitFor();
  const menu = () => page.getByRole('heading', { name: 'Clover Meadow story' }).waitFor();
  const check = async () => {
    const response = page.waitForResponse(r => r.url().endsWith('/api/v1/commands') && r.request().method() === 'POST');
    await click('Check my answer'); assert.equal((await response).status(), 200);
  };
  await page.getByRole('button', { name: 'Start activity' }).and(page.locator(':enabled')).first().click();
  await activity();
  await click('Choose: Put the cup inside the box.'); await check();
  await page.getByText('Not quite yet. Change your answer and try again.', { exact: true }).waitFor();
  await click('Show a hint'); await page.getByText(/This support has been recorded/).waitFor();
  await page.reload(); await activity();
  assert.equal((await snapshot()).pendingEncounters[0].support.hintCount, 1);
  await click('Choose: Put the cup beside the tree.'); await check(); await menu();
  assert.ok((await snapshot()).storyFlags.includes('meadow_welcome_complete'));
  await page.getByRole('button', { name: 'Start activity' }).and(page.locator(':enabled')).first().click();
  await activity();
  await page.getByRole('button', { name: /Leave|Return|abandon/i }).click(); await menu();
  assert.equal((await snapshot()).pendingEncounters.length, 0);
  for (let iteration = 0; iteration < 12; iteration++) {
    const available = page.getByRole('button', { name: /^(Start activity|Help this friend)$/ }).and(page.locator(':enabled'));
    if (!await available.count()) break;
    await available.first().click(); await activity();
    const encounter = (await snapshot()).pendingEncounters[0], p = encounter.parameters;
    if (encounter.templateId === 'missing_quantity') {
      for (let i = 0; i < p.total - (p.given ?? 0); i++) await click('Add one object');
    } else if (encounter.templateId === 'equal_sharing') {
      for (let friend = 1; friend <= p.recipients; friend++) for (let i = 0; i < p.total / p.recipients; i++) await click(`Give one object to friend ${friend}`);
    } else if (p.instructionId === 'count-next') await click(`Choose: ${Number(p.objects[p.prefixLength - 1].label) + 1}`);
    else if (p.instructionId === 'pattern-next') await click(`Choose: ${p.objects[p.prefixLength % p.unitLength].label}`);
    else if (p.instructionId === 'sort-round') {
      for (const object of p.objects.filter(o => o.attribute === 'round')) await click(`Choose: ${object.label}`);
    } else if (p.promptId === 'sound-s') await click('Choose tile 1: s');
    else if (p.promptId === 'seed-order') {
      for (const label of ['Pip finds a seed.', 'Pip plants the seed.', 'Pip waters the seed.']) await click(`Choose: ${label}`);
    } else throw new Error(`Unexpected task: ${JSON.stringify(p)}`);
    await check(); await menu();
  }
  const final = await snapshot();
  assert.ok(final.storyFlags.includes('meadow_light'));
  assert.deepEqual([...final.ownedCreatures].sort(), ['flutter', 'nib', 'pip', 'tansy']);
  assert.ok(final.learnerSummaries.length >= 6);
  await page.getByRole('button', { name: 'Choose as companion' }).first().click();
  await page.getByRole('status').filter({ hasText: 'selected as your server-side companion' }).waitFor();
  const revision = (await snapshot()).revision;
  await page.reload(); await page.getByRole('status').filter({ hasText: 'Connected' }).waitFor();
  assert.equal((await snapshot()).revision, revision);
  await page.setViewportSize({ width: 390, height: 844 });
  assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth));
  await page.screenshot({ path: '../../.local-demo/backend-demo-mobile.png', fullPage: true });
  assert.deepEqual(errors, []);
  assert.equal((await page.request.get(`${origin}/__demo/bootstrap`, { headers: { Origin: 'https://evil.example' } })).status(), 403);
  assert.equal((await page.request.post(`${origin}/api/v1/commands`, { data: {} })).status(), 403);
  assert.equal((await page.request.get(`${origin}/@fs${process.cwd().replace('/qa/e2e', '')}/.local-demo/session.json`)).status(), 403);
  assert.equal((await page.request.get('http://127.0.0.1:3002/api/v1/game')).status(), 401);
  console.log('PASS: full Meadow story, four friendships, incorrect answer, support, abandonment, companion, persistence, mobile layout, local access boundaries');
} finally { await browser.close(); }
