import { openCampaignPage } from './campaign-ui.mjs';
// Two real synthetic household profiles; only actual authored rewards are offered.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { writeFile } from 'node:fs/promises';
import { chromium } from 'playwright';
const origin = process.env.WW_DEMO_TEST_URL ?? 'http://127.0.0.1:5196';
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH ?? '/usr/bin/chromium' });
const page = await browser.newPage({ viewport: { width: 1280, height: 800 }, reducedMotion: 'reduce' });
const errors = []; page.on('pageerror', error => errors.push(error.message));
const hash = value => createHash('sha256').update(JSON.stringify(value)).digest('hex');
try {
  const bootstrap = await (await page.request.get(origin + '/__demo/bootstrap')).json();
  const headers = profile => ({ 'X-Wonderwilds-Profile': profile, 'X-CSRF-Token': bootstrap.csrf, Origin: origin });
  const get = async (path, profile = bootstrap.profileId) => {
    const response = await page.request.get(origin + '/api/v1/' + path, { headers: headers(profile) });
    assert.equal(response.status(), 200, `${path}: ${await response.text()}`); return response.json();
  };
  const post = async (path, profile, revision, payload, expected = 200) => {
    const response = await page.request.post(origin + '/api/v1/' + path + '/commands', { headers: headers(profile), data: { schemaVersion: '1.0', commandId: crypto.randomUUID(), expectedRevision: revision, payload } });
    assert.equal(response.status(), expected, `${path}/${payload.type}: ${await response.text()}`); return response.json();
  };
  const household = await get('household/profiles'); assert.equal(household.profiles.length, 2, 'Runner must provision exactly two synthetic profiles');
  const first = bootstrap.profileId, second = household.profiles.find(profile => profile.id !== first).id;
  const owned = { [first]: 'pip', [second]: 'tansy' };
  const before = {};
  for (const profile of [first, second]) {
    console.log(`Preparing authored Bridge/Rowan reward for ${owned[profile]}`);
    const setup = await get('mission-learning', profile);
    await post('mission-learning', profile, setup.revision, { type: 'set_age', age: 8 });
    let state = await get('campaign', profile); assert.equal(state.adopted, false);
    state = await post('campaign', profile, state.revision, { type: 'adopt' });
    state = await post('campaign', profile, state.revision, { type: 'choose_starter', companion_id: owned[profile] });
    for (const id of ['welcome_meet','practice_pip','practice_tansy','practice_nib','welcome_supplies','welcome_depart','bridge_mounts','bridge_record','bridge_gather','bridge_plan','bridge_planks','bridge_build','bridge_trial','bridge_celebrate']) {
      const action = state.campaign.legal_actions.find(a => a.id === id); assert.ok(action, id);
      if (action.learning) {
        const learning = await get('mission-learning', profile);
        const begun = await post('mission-learning', profile, learning.revision, { type: 'begin_campaign_step', actionId: id, campaignRevision: state.revision });
        await post('mission-learning', profile, begun.revision, { type: 'finish_together', encounterId: begun.activeActivity.id, variantId: begun.activeActivity.variantId });
        state = await get('campaign', profile);
      } else state = await post('campaign', profile, state.revision, { type: 'action', action_id: id });
      while (state.campaign.state.encounter) state = await post('campaign', profile, state.revision, { type: 'encounter_turn', companion_id: state.campaign.state.team[0], movement: 'guided' });
    }
    state = await post('campaign', profile, state.revision, { type: 'travel', location_id: 'picnic_clearing' });
    state = await post('campaign', profile, state.revision, { type: 'action', action_id: 'rowan_first' });
    while (state.campaign.state.encounter) state = await post('campaign', profile, state.revision, { type: 'encounter_turn', companion_id: owned[profile], movement: 'guided' });
    state = await post('campaign', profile, state.revision, { type: 'action', action_id: 'rowan_first_ribbon' });
    assert.equal(state.campaign.state.items.rowan_first_ribbon, 1, 'Real authored source reward');
    before[profile] = { contentVersion: state.campaign.state.content_version, campaign: hash(state), game: hash(await get('game', profile)), learning: hash(await get('mission-learning', profile)) };
  }
  const forbidden = await page.request.get(origin + '/api/v1/paired', { headers: headers('00000000-0000-4000-8000-000000000099') });
  assert.equal(forbidden.status(), 403, 'Bridge rejects a profile not owned by this household');
  let actor = first;
  const open = async () => {
    await openCampaignPage(page, 'paired');
    if (await page.locator('#paired-refresh').count()) {
      const response = page.waitForResponse(r => r.url().endsWith('/api/v1/paired') && r.request().method() === 'GET');
      await page.locator('#paired-refresh').click(); assert.equal((await response).status(), 200);
    }
    await page.getByText('Who is holding the device?', { exact: true }).waitFor();
  };
  const switchTo = async profile => {
    if (actor === profile) return;
    await page.locator(`[data-select-profile="${profile}"]`).click();
    await page.waitForURL(url => url.searchParams.get('player') === profile);
    actor = profile; await page.locator('#campaign-more').waitFor(); await open();
  };
  const choose = async payload => {
    const write = page.waitForResponse(r => r.url().endsWith('/paired/commands') && JSON.stringify(r.request().postDataJSON()?.payload) === JSON.stringify(payload));
    const refreshed = page.waitForResponse(r => r.url().endsWith('/api/v1/paired') && r.request().method() === 'GET');
    await page.locator(`[data-paired-payload='${JSON.stringify(payload)}']`).click();
    assert.equal((await write).status(), 200); assert.equal((await refreshed).status(), 200);
    await page.locator('.campaign-status').filter({ hasText: 'Shared expedition saved.' }).waitFor();
    return get('paired', actor);
  };
  await page.goto(origin); await page.locator('#campaign-more').waitFor(); await open();
  let pair = await choose({ type: 'start', partnerProfileId: second });
  assert.equal(pair.navigatorProfileId, first); assert.equal(pair.keeperProfileId, second);
  assert.deepEqual(pair.availableCompanions.map(friend => friend.id), ['pip']);
  await post('paired', first, pair.revision, { type: 'select_companion', companionId: 'tansy' }, 403);
  pair = await choose({ type: 'select_companion', companionId: 'pip' });
  await switchTo(second); pair = await choose({ type: 'select_companion', companionId: 'tansy' });
  await switchTo(first); pair = await choose({ type: 'start_practice' });
  pair = await choose({ type: 'practice_turn', movement: 'scout' });
  await post('paired', first, pair.revision, { type: 'practice_turn', movement: 'brace' }, 403);
  await page.reload(); await page.locator('#campaign-more').waitFor(); await open();
  assert.equal((await get('paired', first)).practice.history.length, 1, 'Reload preserves the credited tactical turn');
  for (const [profile, movement] of [[second,'brace'],[first,'scout'],[second,'brace']]) {
    await switchTo(profile); pair = await choose({ type: 'practice_turn', movement });
  }
  assert.equal(pair.practice.outcome, 'won'); assert.equal(pair.completedPractices, 1);
  for (const turn of pair.practice.history) assert.equal(turn.companionId, owned[turn.profileId], 'Every turn uses the selected learner’s own companion');
  const creditedHistory = structuredClone(pair.practice.history);
  assert.equal(pair.navigatorProfileId, second, 'Successful boundary swaps roles without moving ownership');
  await page.screenshot({ path: '/tmp/campaign-paired-practice-desktop.png', fullPage: true });
  pair = await choose({ type: 'swap_roles' }); assert.equal(pair.navigatorProfileId, first);
  pair = await choose({ type: 'offer_keepsake', keepsakeId: 'rowan_first_ribbon' });
  await switchTo(first); pair = await choose({ type: 'offer_keepsake', keepsakeId: 'rowan_first_ribbon' });
  const offers = page.locator('[aria-label="Both keepsake offers"]');
  assert.equal(await offers.locator('article').count(), 2);
  for (const [index, profile] of [first, second].entries()) {
    const code = profile.replaceAll('-', '').slice(-8).toUpperCase();
    const card = offers.locator('article').filter({ hasText: `Player ${index + 1} · ${code}` });
    assert.equal(await card.count(), 1, 'Offer identifies its exact selected player');
    assert.ok((await card.innerText()).includes('Rowan First Ribbon'), 'Both exact authored offered items visible before consent');
  }
  const confirm = page.locator(`[data-paired-payload='${JSON.stringify({ type: 'confirm_exchange' })}']`);
  assert.equal(await confirm.innerText(), 'Confirm these two copied keepsakes');
  await confirm.scrollIntoViewIfNeeded();
  await page.screenshot({ path: '/tmp/campaign-paired-exact-offers-desktop.png', fullPage: true });
  await page.setViewportSize({ width: 390, height: 844 });
  await confirm.scrollIntoViewIfNeeded();
  assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth));
  await page.screenshot({ path: '/tmp/campaign-paired-exact-offers-phone.png', fullPage: true });
  await page.setViewportSize({ width: 1280, height: 800 });
  pair = await choose({ type: 'confirm_exchange' }); assert.equal(pair.exchangeComplete, false);
  assert.equal((await get('paired', first)).keepsakes.length, 0); assert.equal((await get('paired', second)).keepsakes.length, 0);
  await switchTo(second);
  let originalBody;
  await page.route('**/api/v1/paired/commands', async route => {
    originalBody = route.request().postData(); const response = await route.fetch(); assert.equal(response.status(), 200); await route.abort('failed');
  }, { times: 1 });
  await page.locator(`[data-paired-payload='${JSON.stringify({ type: 'confirm_exchange' })}']`).click();
  await page.locator('#paired-retry').waitFor();
  assert.equal(await page.locator(`[data-select-profile="${first}"]`).isDisabled(), true, 'Uncertain actor command blocks player switches');
  await page.reload(); await page.locator('#campaign-more').waitFor(); await openCampaignPage(page, 'paired');
  await page.locator('#paired-retry').waitFor();
  const retry = page.waitForRequest(r => r.url().endsWith('/paired/commands'));
  await page.locator('#paired-retry').click(); assert.equal((await retry).postData(), originalBody, 'Reload preserves exact shared-command actor/body/ID');
  await page.getByText('Both players confirmed. Your shared keepsakes are saved independently.').waitFor();
  for (const profile of [first, second]) {
    const saved = await get('paired', profile); assert.equal(saved.keepsakes.length, 1); assert.equal(saved.keepsakeCount, 1);
    assert.equal(saved.keepsakes[0].sourceProfileId, profile === first ? second : first);
    assert.equal(hash(await get('campaign', profile)), before[profile].campaign, 'Exchange retains original resources and campaign progress');
    assert.equal(hash(await get('game', profile)), before[profile].game, 'Shared tactics does not grant XP or transfer companions');
    assert.equal(hash(await get('mission-learning', profile)), before[profile].learning, 'Role changes do not move academic evidence');
  }
  await page.setViewportSize({ width: 390, height: 844 });
  assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth));
  await page.screenshot({ path: '/tmp/campaign-paired-exchange-phone.png', fullPage: true });
  await choose({ type: 'continue_solo' });
  for (const profile of [first, second]) assert.equal((await get('paired', profile)).mode, 'solo');
  await page.reload(); await page.locator('#campaign-missions').waitFor();
  assert.equal((await get('paired', second)).keepsakes.length, 1, 'Solo continuation preserves the received memory');
  assert.deepEqual(errors, []);
  await writeFile('/tmp/campaign-paired-evidence.json', JSON.stringify({ result: 'passed', contentVersion: before[first].contentVersion, sourceRewards: 'Actual Bridge and Rowan first ribbon for each profile', profiles: { [first]: 'pip', [second]: 'tansy' }, history: creditedHistory, assertions: ['owned household selection','own companions only','explicit actor turn attribution','reload','automatic and manual role swaps','no XP or academic evidence changes','two-sided consent','exact uncertain confirmation retry after reload','single copies with originals retained','phone layout','solo continuation'] }, null, 2));
  console.log('PASS M7 actual two-player practice, role/learner ownership, consented authored keepsakes, exact retry/reload and solo continuation');
} catch (error) {
  await page.screenshot({ path: '/tmp/campaign-paired-failure.png', fullPage: true }).catch(() => {});
  console.error((await page.locator('body').innerText()).slice(-5000)); throw error;
} finally { await browser.close(); }
