// Disposable real-API exploration + construction + irrigation regression.
import assert from 'node:assert/strict';
import { chromium } from 'playwright';
const origin = process.env.WW_DEMO_TEST_URL ?? 'http://127.0.0.1:5180';
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH });
const page = await browser.newPage({ viewport: { width: 1280, height: 900 } });
const errors = [];
page.on('pageerror', e => errors.push(e.message));
try {
  const bootstrap = await (await page.request.get(origin + '/__demo/bootstrap')).json();
  const headers = { 'X-Wonderwilds-Profile': bootstrap.profileId, 'X-CSRF-Token': bootstrap.csrf, Origin: origin };
  const get = async (path = 'campaign') => (await page.request.get(origin + '/api/v1/' + path, { headers })).json();
  const send = async (path, revision, payload) => {
    const r = await page.request.post(origin + '/api/v1/' + path + '/commands', { headers, data: { schemaVersion: '1.0', commandId: crypto.randomUUID(), expectedRevision: revision, payload } });
    assert.equal(r.status(), 200, await r.text()); return r.json();
  };
  const command = async payload => send('campaign', (await get()).revision, payload);
  const action = async id => {
    const state = await get();
    const a = state.campaign.legal_actions.find(a => a.id === id); assert.ok(a, id);
    if (a.learning) {
      const l = await get('mission-learning');
      const begun = await send('mission-learning', l.revision, { type: 'begin_campaign_step', actionId: id, campaignRevision: state.revision });
      await send('mission-learning', begun.revision, { type: 'finish_together', encounterId: begun.activeActivity.id, variantId: begun.activeActivity.variantId });
    } else await command({ type: 'action', action_id: id });
  };
  await send('mission-learning', (await get('mission-learning')).revision, { type: 'set_age', age: 8 });
  await command({ type: 'adopt' });
  await command({ type: 'choose_starter', companion_id: 'pip' });
  await action('welcome_meet'); await action('practice_pip');
  while ((await get()).campaign.state.encounter) await command({ type: 'encounter_turn', companion_id: 'pip', movement: 'guided' });
  for (const id of ['welcome_supplies','welcome_depart','bridge_mounts','bridge_record']) await action(id);
  await page.goto(origin);
  await page.getByRole('heading', { name: 'Search the clearing for fallen branches' }).waitFor();
  const uiCommand = async (locator, match, status = 200) => {
    const response = page.waitForResponse(r => r.url().endsWith('/campaign/commands') && match(r.request().postDataJSON()?.payload));
    await locator.click();
    assert.equal((await response).status(), status);
    await page.waitForFunction(() => !document.querySelector('#dialog')?.open || !document.querySelector('#dialog button[data-campaign-payload]:disabled') || document.querySelector('.campaign-project')); 
  };
  await uiCommand(page.getByRole('button', { name: 'Take the path to Picnic Clearing' }), p => p?.type === 'travel');
  await page.locator('#game[data-campaign-location="picnic_clearing"]').waitFor();
  assert.equal(await page.locator('#dialog').isVisible(), false, 'Walking leaves the menu closed');
  const openMission = async () => {
    if (!(await page.locator('#dialog').isVisible())) await page.locator('#missions').click();
  };
  const visit = async id => {
    console.log('Visit', id);
    await openMission();
    const before = await page.locator('#game').getAttribute('data-campaign-player-x');
    if (id.startsWith('find_branch_')) {
      const beforeRevision = (await get()).revision;
      await uiCommand(page.locator(`[data-campaign-command="${id}"]`), p => p?.action_id === id);
      await page.waitForFunction(rev => Number(document.querySelector('#game')?.getAttribute('data-campaign-revision')) > rev, beforeRevision);
      assert.equal(await page.locator('#dialog').isVisible(), false);
      return;
    }
    await page.locator(`[data-campaign-command="${id}"]`).click();
    await page.locator('#campaign-read').waitFor();
    const after = await page.locator('#game').getAttribute('data-campaign-player-x');
    assert.notEqual(after, before, 'The player walks to the person/object');
    while (await page.getByRole('button', { name: 'Next', exact: true }).count()) await page.getByRole('button', { name: 'Next', exact: true }).click();
    await uiCommand(page.locator(`[data-campaign-payload='${JSON.stringify({ type: 'action', action_id: id })}']`), p => p?.action_id === id);
  };
  for (const id of ['find_branch_3','find_branch_1','find_branch_2']) await visit(id);
  const gathered = await get();
  assert.equal(gathered.campaign.state.items.quest_branch, 3);
  assert.ok(gathered.campaign.state.flags.includes('bridge_gather_done'));
  await page.reload(); await page.locator('#campaign-missions').waitFor();
  await uiCommand(page.locator('[data-campaign-payload]').filter({ hasText: 'Bridge Bank' }).first(), p => p?.type === 'travel');
  await page.locator('#game[data-campaign-location="bridge_bank"]').waitFor();
  await action('bridge_plan'); await page.reload(); await page.locator('#campaign-missions').waitFor();
  await visit('bridge_planks'); await visit('bridge_build'); await visit('bridge_trial');
  await page.getByRole('region', { name: 'Bridge construction' }).waitFor();
  assert.equal(await page.locator('[data-campaign-move="focus"]').count(), 0);
  // Wrong-length attempts are rejected by the server without changing the checkpoint.
  const beforeWrong = await get();
  const bad = await page.request.post(origin + '/api/v1/campaign/commands', { headers, data: { schemaVersion: '1.0', commandId: crypto.randomUUID(), expectedRevision: beforeWrong.revision, payload: { type: 'encounter_turn', companion_id: 'pip', movement: 'bridge_fit:middle:short' } } });
  assert.ok(bad.status() >= 400); assert.equal((await get()).revision, beforeWrong.revision);
  const move = async movement => {
    console.log('Project move', movement);
    await uiCommand(page.locator(`[data-campaign-move="${movement}"]`), p => p?.movement === movement);
    await page.waitForFunction(() => !document.querySelector('#dialog button[data-campaign-move="guided"]:disabled'));
  };
  await move('bridge_fit:near:short');
  await page.locator('[data-project-piece="long"]').click();
  await move('bridge_fit:middle:long');
  await page.reload();
  await page.getByRole('button', { name: 'middle plank fitted ✓' }).waitFor();
  await move('bridge_fit:far:short'); await move('bridge_brace:near'); await move('bridge_brace:far');
  await page.screenshot({ path: '/tmp/wonderwilds-bridge-construction.png', fullPage: true });
  await move('bridge_cross');
  assert.ok((await get()).campaign.state.flags.includes('bridge_seal'));
  await action('bridge_celebrate');
  await command({ type: 'travel', location_id: 'flower_garden' });
  for (const id of ['bloom_beds','bloom_record','water_near_garden','water_far_seedlings','water_check','garden_wild','bloom_shelter','bloom_prepare','bloom_trial']) await action(id);
  await page.reload(); await page.getByRole('region', { name: 'Garden irrigation' }).waitFor();
  assert.equal(await page.locator('[data-project-piece]').count(), 0);
  await move('garden_route:north:collector');
  assert.equal(await page.locator('[data-campaign-move="garden_water:north"]').isDisabled(), true);
  for (const m of ['garden_route:north:beds','garden_water:north','garden_mulch:north','garden_water:south','garden_mulch:south']) await move(m);
  await page.setViewportSize({ width: 390, height: 844 });
  await page.reload(); await page.getByRole('region', { name: 'Garden irrigation' }).waitFor();
  assert.equal(await page.locator('.garden-bed.watered').count(), 2);
  assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth));
  await page.screenshot({ path: '/tmp/wonderwilds-garden-irrigation-phone.png', fullPage: true });
  await move('garden_check');
  assert.ok((await get()).campaign.state.flags.includes('bloom_seal'));
  assert.deepEqual(errors, []);
  console.log('Exploration: walked to three saved finds and woodcutter; bridge placement and garden routing completed; reload and phone layout passed.');
} catch (e) {
  await page.screenshot({ path: '/tmp/wonderwilds-exploration-failure.png', fullPage: true });
  console.error((await page.locator('body').innerText()).slice(-5000)); throw e;
} finally { await browser.close(); }
