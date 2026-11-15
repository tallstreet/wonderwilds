// Real disposable Bridge trial state, for semantic board/art inspection.
import assert from 'node:assert/strict';
import { chromium } from 'playwright';
const origin = process.env.WW_DEMO_TEST_URL ?? 'http://127.0.0.1:5194';
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH ?? '/usr/bin/chromium' });
const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
try {
  const bootstrap = await (await page.request.get(origin + '/__demo/bootstrap')).json();
  const headers = { 'X-Wonderwilds-Profile': bootstrap.profileId, 'X-CSRF-Token': bootstrap.csrf, Origin: origin };
  const get = async path => {
    const response = await page.request.get(origin + '/api/v1/' + path, { headers });
    assert.equal(response.status(), 200); return response.json();
  };
  const send = async (path, revision, payload) => {
    const response = await page.request.post(origin + '/api/v1/' + path + '/commands', { headers, data: { schemaVersion: '1.0', commandId: crypto.randomUUID(), expectedRevision: revision, payload } });
    assert.equal(response.status(), 200, await response.text()); return response.json();
  };
  const setup = await get('mission-learning');
  await send('mission-learning', setup.revision, { type: 'set_age', age: 8 });
  let state = await get('campaign');
  state = await send('campaign', state.revision, { type: 'adopt' });
  state = await send('campaign', state.revision, { type: 'choose_starter', companion_id: 'pip' });
  for (const id of ['welcome_meet', 'practice_pip', 'practice_tansy', 'practice_nib', 'welcome_supplies', 'welcome_depart', 'bridge_mounts', 'bridge_record', 'bridge_gather', 'bridge_plan', 'bridge_planks', 'bridge_build', 'bridge_trial']) {
    const action = state.campaign.legal_actions.find(a => a.id === id); assert.ok(action, id);
    if (action.learning) {
      const learning = await get('mission-learning');
      const begun = await send('mission-learning', learning.revision, { type: 'begin_campaign_step', actionId: id, campaignRevision: state.revision });
      await send('mission-learning', begun.revision, { type: 'finish_together', encounterId: begun.activeActivity.id, variantId: begun.activeActivity.variantId });
      state = await get('campaign');
    } else state = await send('campaign', state.revision, { type: 'action', action_id: id });
    if (id.startsWith('practice_')) while (state.campaign.state.encounter) state = await send('campaign', state.revision, { type: 'encounter_turn', companion_id: state.campaign.state.team[0], movement: 'guided' });
  }
  await page.goto(origin); await page.locator('[data-campaign-move="signature"]').waitFor();
  assert.equal(await page.getByRole('progressbar', { name: 'Light restored' }).getAttribute('value'), '0');
  await page.getByRole('heading', { name: '1. Who will help?' }).waitFor();
  await page.getByRole('heading', { name: '2. Choose a move for Pip' }).waitFor();
  assert.ok(await page.locator('[data-campaign-move="signature"]').getAttribute('aria-describedby'));
  assert.match(await page.locator('#move-snack-help').innerText(), /None in your bag/);
  assert.match(await page.locator('[data-campaign-move="guided"]').getAttribute('class'), /campaign-secondary/);
  await page.locator('#campaign-guided').click();
  await page.getByText('Plan your next move', { exact: true }).waitFor();
  assert.equal(await page.getByRole('progressbar', { name: 'Light restored' }).getAttribute('value'), '0');
  await page.locator('#campaign-guided').click();
  await page.screenshot({ path: '/tmp/campaign-trial-board-desktop.png', fullPage: true });
  await page.setViewportSize({ width: 390, height: 844 });
  assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth));
  await page.screenshot({ path: '/tmp/campaign-trial-board-phone.png', fullPage: true });
  await page.setViewportSize({ width: 1280, height: 800 });
  const turn = async movement => {
    const response = page.waitForResponse(r => r.url().endsWith('/campaign/commands') && r.request().postDataJSON()?.payload?.movement === movement);
    await page.locator(`[data-campaign-move="${movement}"]`).click();
    assert.equal((await response).status(), 200);
    await page.locator('.campaign-status').filter({ hasText: 'Saved' }).waitFor();
  };
  await turn('signature');
  await page.locator('.campaign-effect-bitmap').waitFor();
  await page.getByText(/Pip used their Gleam signature/).waitFor();
  assert.equal(await page.getByRole('progressbar', { name: 'Light restored' }).getAttribute('value'), '2');
  await page.screenshot({ path: '/tmp/campaign-trial-signature-desktop.png', fullPage: true });
  await page.emulateMedia({ reducedMotion: 'reduce' });
  await page.screenshot({ path: '/tmp/campaign-trial-signature-reduced.png', fullPage: true });
  await page.setViewportSize({ width: 390, height: 844 });
  await page.screenshot({ path: '/tmp/campaign-trial-signature-phone.png', fullPage: true });
  await page.setViewportSize({ width: 1280, height: 800 });
  await turn('tool');
  await page.locator('.campaign-effect-bitmap').waitFor({ state: 'hidden' });
  await page.screenshot({ path: '/tmp/campaign-trial-effect-cleared.png', fullPage: true });
  assert.equal(await page.locator('.campaign-effect-bitmap').count(), 0, 'Confirmed signature effect clears before next saved move');
  await turn('tool');
  await page.locator('.campaign-confirmed-effect').filter({ hasText: 'keeper trial is complete' }).waitFor();
  await page.screenshot({ path: '/tmp/campaign-trial-celebration-desktop.png', fullPage: true });
  await page.setViewportSize({ width: 390, height: 844 });
  await page.screenshot({ path: '/tmp/campaign-trial-celebration-phone.png', fullPage: true });
  console.log('Actual authoritative Bridge trial board/signature/cleared/celebration screenshots captured');
} finally { await browser.close(); }
