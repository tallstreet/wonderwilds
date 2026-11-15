// Actual authored campaign browser journey; run with campaign-run.mjs.
import assert from 'node:assert/strict';
import { chromium } from 'playwright';

const origin = process.env.WW_DEMO_TEST_URL ?? 'http://127.0.0.1:5180';
const starter = process.env.WW_CAMPAIGN_STARTER ?? 'pip';
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH ?? '/usr/bin/chromium' });
const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
const errors = [];
page.on('pageerror', error => errors.push(error.message));
page.on('console', message => { if (message.type() === 'error') console.error('Browser console:', message.text()); });
page.on('response', async response => {
  if (new URL(response.url()).pathname.startsWith('/api/v1/campaign')) {
    try { console.log('Campaign HTTP', new URL(response.url()).pathname, response.status(), (await response.body()).length); } catch {}
  }
});
try {
  await page.goto(origin);
  await page.locator('#campaign-start').click();
  await page.locator('#game canvas').waitFor();
  const bootstrap = await (await page.request.get(`${origin}/__demo/bootstrap`)).json();
  const snapshot = async () => {
    const response = await page.request.get(`${origin}/api/v1/campaign`, {
      headers: { 'X-Wonderwilds-Profile': bootstrap.profileId },
    });
    assert.equal(response.status(), 200);
    return response.json();
  };
  const clickPayload = async payload => {
    const button = page.locator(`[data-campaign-payload='${JSON.stringify(payload)}']`);
    await button.waitFor();
    const before = (await snapshot()).revision;
    const saved = page.waitForResponse(r => r.url().endsWith('/campaign/commands') && JSON.stringify(r.request().postDataJSON()?.payload) === JSON.stringify(payload));
    await button.click();
    assert.equal((await saved).status(), 200);
    await page.locator('.campaign-status').filter({ hasText: /Saved|safely saved/ }).waitFor();
    assert.ok((await snapshot()).revision > before, 'Command committed');
    return snapshot();
  };
  await page.locator('#campaign-age').waitFor();
  await page.setViewportSize({ width: 390, height: 844 });
  assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth));
  await page.screenshot({ path: '/tmp/wonderwilds-age-setup-phone.png', fullPage: true });
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.locator('#campaign-age').selectOption('8');
  const ageSaved = page.waitForResponse(r => r.url().endsWith('/mission-learning/commands') && r.request().postDataJSON()?.payload?.type === 'set_age');
  await page.locator('#campaign-save-age').click();
  assert.equal((await ageSaved).status(), 200);
  await page.locator('#campaign-age').waitFor({ state: 'hidden' });
  assert.match(await page.locator('#dialog').innerText(), /lights.*fading/s);
  assert.equal(await page.locator('.campaign-tabs button').count(), 1, 'No unavailable pages in onboarding');
  await page.locator('[data-campaign-command="welcome_meet"]').click();
  while (await page.locator('[data-campaign-page]').filter({ hasText: 'Next' }).count()) {
    await page.locator('[data-campaign-page]').filter({ hasText: 'Next' }).click();
  }
  await clickPayload({ type: 'action', action_id: 'welcome_meet' });
  await page.locator('[data-campaign-payload]').first().waitFor();
  const state = await clickPayload({ type: 'choose_starter', companion_id: starter });
  assert.equal(state.campaign.state.starter, starter);
  await page.locator('.campaign-objective').waitFor();
  await page.screenshot({ path: '/tmp/wonderwilds-mission-clarity-desktop.png', fullPage: true });
  await page.getByRole('button', { name: 'Close dialog', exact: true }).click();
  await page.screenshot({ path: `/tmp/wonderwilds-campaign-${starter}-world.png`, fullPage: true });
  const startX = Number(await page.locator('#game').getAttribute('data-campaign-player-x'));
  await page.keyboard.down('ArrowRight');
  await page.waitForTimeout(400);
  await page.keyboard.up('ArrowRight');
  assert.ok(Number(await page.locator('#game').getAttribute('data-campaign-player-x')) > startX + 10, 'Keyboard moves the player in Phaser');
  await page.reload();
  await page.locator('#game canvas').waitFor();
  assert.equal((await snapshot()).campaign.state.starter, starter, 'Starter survives reload');
  assert.equal(await page.locator('#campaign-age').count(), 0, 'Saved age survives reload');
  await page.locator('#campaign-missions').waitFor();
  await page.locator('#campaign-missions').click();
  const action = async id => {
    console.log(`Campaign ${starter} action ${id}`);
    const initial = await snapshot();
    const before = initial.revision;
    const learning = initial.campaign.legal_actions.find(a => a.id === id)?.learning;
    const actionButton = page.locator(`[data-campaign-command="${id}"]`);
    if (!(await actionButton.isVisible())) await page.locator('.campaign-optional summary').click();
    await actionButton.click();
    while (await page.locator('[data-campaign-page]').filter({ hasText: 'Next' }).count()) {
      await page.locator('[data-campaign-page]').filter({ hasText: 'Next' }).click();
    }
    const control = page.locator(`[data-campaign-payload='${JSON.stringify({ type: 'action', action_id: id })}']`);
    const saved = learning ? undefined : page.waitForResponse(r => r.url().endsWith('/campaign/commands') && r.request().postDataJSON()?.payload?.action_id === id);
    await control.click();
    if (learning) {
      await page.locator('.ww-mission-activity').waitFor();
      const finished = page.waitForResponse(r => r.url().endsWith('/mission-learning/commands') && r.request().postDataJSON()?.payload?.type === 'finish_together');
      await page.getByRole('button', { name: /^(Let’s do it together|Finish activity)$/ }).click();
      assert.equal((await finished).status(), 200);
      await page.locator('.ww-mission-activity').waitFor({ state: 'hidden', timeout: 30000 });
    } else {
      assert.equal((await saved).status(), 200);
      await page.locator('.campaign-status').filter({ hasText: /Saved/ }).waitFor();
    }
    await assertEventually(async () => (await snapshot()).revision > before, `action ${id}`);
    await page.locator('#campaign-missions').waitFor();
    if (id === 'water_check' && initial.campaign.state.flags.includes('puzzle:garden_water:near:bypass')) {
      const flags = (await snapshot()).campaign.state.flags;
      assert.ok(flags.includes('puzzle:garden_water:blocked'));
      assert.ok(!flags.includes('puzzle:garden_water:solved'), 'Wrong irrigation configuration does not solve puzzle');
    }
    if (id.startsWith('practice_')) {
      for (let count = 0; count < 5; count++) {
        const current = (await snapshot()).campaign.state;
        if (!current.encounter) break;
        await clickPayload({ type: 'encounter_turn', companion_id: current.team[0], movement: 'tool' });
      }
      assert.equal((await snapshot()).campaign.state.encounter, null, 'Starter practice completes');
    }
  };
  for (const id of ['practice_pip', 'practice_tansy', 'practice_nib', 'welcome_supplies', 'welcome_depart', 'bridge_mounts', 'bridge_record', 'bridge_gather', 'bridge_plan', 'bridge_planks', 'bridge_build', 'bridge_trial']) await action(id);
  assert.ok((await snapshot()).campaign.companions.find(friend => friend.id === starter).level >= 2, 'First-help training reaches level two');
  const turn = movement => clickPayload({ type: 'encounter_turn', companion_id: starter, movement });
  // Deliberately fail once: recovery must preserve the trial invitation.
  for (let n = 0; n < 10 && (await snapshot()).campaign.state.encounter; n++) await turn('rest');
  assert.equal((await snapshot()).campaign.state.encounter, null);
  assert.ok((await snapshot()).campaign.legal_actions.some(a => a.id === 'bridge_trial'));
  await action('bridge_trial');
  await turn('tool');
  const paused = (await snapshot()).campaign.state.encounter;
  await page.reload();
  await page.locator('#game canvas').waitFor();
  await page.locator('#campaign-missions').waitFor();
  await page.locator('#campaign-missions').click();
  assert.deepEqual((await snapshot()).campaign.state.encounter, paused, 'Trial reload preserves exact state');
  let uncertainBody;
  await page.route('**/api/v1/campaign/commands', async route => {
    uncertainBody = route.request().postData();
    const response = await route.fetch();
    assert.equal(response.status(), 200);
    await route.abort('failed');
  }, { times: 1 });
  await page.locator('[data-campaign-move="tool"]').click();
  await page.locator('#campaign-retry').waitFor();
  const committed = await snapshot();
  const retry = page.waitForRequest(request => request.url().endsWith('/campaign/commands'));
  await page.locator('#campaign-retry').click();
  assert.equal((await retry).postData(), uncertainBody, 'Uncertain save retries identical command');
  await page.locator('.campaign-status').filter({ hasText: /safely saved/ }).waitFor();
  assert.deepEqual(await snapshot(), committed, 'Retry does not repeat turn or spend energy');
  await turn('tool');
  await action('bridge_celebrate');
  await page.getByRole('button', { name: 'Close dialog', exact: true }).click();
  await page.screenshot({ path: `/tmp/wonderwilds-campaign-${starter}-bridge-restored.png`, fullPage: true });
  const canvas = page.locator('#game canvas');
  // The second server-legal route is an actual sign in Phaser at (900,490).
  const bounds = await canvas.boundingBox();
  assert.ok(bounds);
  await canvas.click({ position: { x: bounds.width * 900 / 1120, y: bounds.height * 490 / 690 } });
  await assertEventually(async () => Number(await page.locator('#game').getAttribute('data-campaign-player-x')) > 800, 'walk across repaired bridge');
  await page.keyboard.press('Space');
  await assertEventually(async () => (await snapshot()).campaign.state.location === 'flower_garden', 'interact with garden trail sign');
  await page.locator('#campaign-missions').waitFor();
  assert.equal(await page.locator('#game').getAttribute('data-campaign-location'), 'flower_garden');
  for (const id of ['bloom_beds', 'bloom_record', 'water_near_bypass', 'water_far_collector', 'water_check', 'water_near_garden', 'water_far_seedlings', 'water_check', 'garden_wild', 'bloom_shelter', 'bloom_prepare', 'bloom_trial']) await action(id);
  await turn('tool'); await turn('tool'); await turn('tool'); await turn('focus');
  await action('bloom_celebrate');
  const friend = starter === 'pip' ? 'tansy' : 'pip';
  if (friend === 'pip') await clickPayload({ type: 'travel', location_id: 'bridge_bank' });
  await action(`observe_${friend}`);
  await action(`approach_${friend}_${friend === 'pip' ? 'wait' : 'help'}`);
  if (friend === 'pip') await clickPayload({ type: 'travel', location_id: 'flower_garden' });
  await action(`help_${friend}`); await action(`invite_${friend}`);
  await clickPayload({ type: 'travel', location_id: 'bridge_bank' });
  await page.locator('#campaign-more').click();
  await page.locator('#campaign-team').click();
  await clickPayload({ type: 'set_team', companion_ids: [starter, friend] });
  await page.locator('#campaign-more').click();
  await page.locator('#campaign-care').click();
  await clickPayload({ type: 'camp' });
  await page.locator('#campaign-missions').click();
  await action('bridge_alcove');
  const grantsBeforeReplay = (await snapshot()).campaign.state.grants;
  await action('bridge_trial');
  await clickPayload({ type: 'encounter_turn', companion_id: starter, movement: 'rally' });
  await page.locator(`[data-campaign-select="${friend}"]`).click();
  await clickPayload({ type: 'encounter_turn', companion_id: friend, movement: 'signature' });
  assert.ok((await snapshot()).campaign.state.encounter.progress >= 4, 'Partner signature uses the starter’s rally');
  await clickPayload({ type: 'encounter_turn', companion_id: friend, movement: 'tool' });
  assert.equal((await snapshot()).campaign.state.encounter, null, 'Paired practice wins');
  assert.deepEqual((await snapshot()).campaign.state.grants, grantsBeforeReplay, 'Practice replay grants no repeated reward');
  await page.locator('#campaign-more').click();
  await page.locator('#campaign-care').click();
  const beforeRecovery = (await snapshot()).revision;
  await page.route('**/api/v1/campaign', route => route.fulfill({ status: 503, json: { code: 'UNAVAILABLE', schemaVersion: '1.0', correlationId: 'synthetic-refresh-failure' } }), { times: 2 });
  const savedRest = page.waitForResponse(r => r.url().endsWith('/campaign/commands') && r.request().postDataJSON()?.payload?.type === 'camp');
  await page.locator(`[data-campaign-payload='${JSON.stringify({ type: 'camp' })}']`).click();
  assert.equal((await savedRest).status(), 200);
  await page.locator('#campaign-refresh').waitFor();
  assert.equal(await page.locator('[data-campaign-payload]').count(), 0, 'Failed reconciliation blocks new commands');
  await page.locator('#campaign-refresh').click();
  await page.locator('#campaign-refresh').waitFor({ state: 'hidden' });
  assert.equal((await snapshot()).revision, beforeRecovery + 1, 'Refresh never repeats committed rest');
  const complete = (await snapshot()).campaign.state;
  for (const flag of ['bridge_seal', 'bloom_seal', 'world_bridge_restored', 'world_garden_restored', 'world_bridge_alcove_open']) assert.ok(complete.flags.includes(flag), flag);
  assert.equal(complete.team.length, 2);
  const phone = await browser.newPage({ viewport: { width: 390, height: 844 }, hasTouch: true, isMobile: true, reducedMotion: 'reduce' });
  phone.on('pageerror', error => errors.push(error.message));
  await phone.goto(origin);
  await phone.locator('#campaign-missions').waitFor();
  await phone.locator('#campaign-more').tap();
  await phone.locator('#campaign-journal').tap();
  assert.match(await phone.locator('#dialog-title').innerText(), /Your story so far/);
  assert.ok(await phone.evaluate(() => document.documentElement.scrollWidth <= innerWidth), 'Mobile page has no horizontal overflow');
  await phone.screenshot({ path: `/tmp/wonderwilds-campaign-${starter}-mobile.png`, fullPage: true });
  await phone.close();
  assert.deepEqual(errors, [], 'No uncaught browser errors');
  console.log(`Campaign ${starter}: Bridge/Bloom, support, loss recovery, trial reload, care, pair, revisit and mobile passed`);
} catch (error) {
  await page.screenshot({ path: `/tmp/wonderwilds-campaign-${starter}-failure.png`, fullPage: true });
  console.error('Browser state:', (await page.locator('body').innerText()).slice(-6000));
  console.error('Page errors:', errors);
  throw error;
} finally {
  await browser.close();
}

async function assertEventually(check, label) {
  const deadline = Date.now() + 20000;
  while (Date.now() < deadline) {
    if (await check()) return;
    await new Promise(resolve => setTimeout(resolve, 100));
  }
  assert.fail(`Timed out: ${label}`);
}
