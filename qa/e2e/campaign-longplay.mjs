import { openCampaignPage } from './campaign-ui.mjs';
// Disposable campaign API progression with real supported activity controls in Chromium.
import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';
import { chromium } from 'playwright';
const puzzleGoals = { garden_water: { near: 'garden', far: 'seedlings' }, tower_chimes: { first: 'one_ring', second: 'two_rings', third: 'three_rings' }, pump_valves: { first: 'inlet', second: 'bypass', third: 'landing' } };
const definition = JSON.parse(await readFile(new URL('../../content/campaign/runtime.v1.json', import.meta.url)));
const origin = process.env.WW_DEMO_TEST_URL ?? 'http://127.0.0.1:5180';
const stage = process.env.WW_CAMPAIGN_STAGE ?? '';
const artifactPrefix = stage ? `campaign-${stage}` : 'campaign';
const starter = process.env.WW_CAMPAIGN_STARTER ?? 'pip';
const order = process.env.WW_CAMPAIGN_ORDER ?? 'harvest';
const useLearningUi = process.env.WW_CAMPAIGN_ALL_API !== '1' && starter === 'pip' && order === 'harvest';
const hasRelay = definition.encounters.some(board => board.relay);
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH ?? '/usr/bin/chromium' });
const page = await browser.newPage({ viewport: { width: 960, height: 640 }, reducedMotion: 'reduce' });
const errors = [];
page.on('pageerror', e => errors.push(e.message));
const completed = [];
const guidedTimings = [];
const guidedRetreats = [];
try {
  let lastCombat;
  const bootstrap = await (await page.request.get(`${origin}/__demo/bootstrap`)).json();
  const headers = { 'X-Wonderwilds-Profile': bootstrap.profileId, 'X-CSRF-Token': bootstrap.csrf, Origin: origin };
  const get = async path => {
    const response = await page.request.get(`${origin}/api/v1/${path}`, { headers });
    assert.equal(response.status(), 200, `${path} ${await response.text()}`);
    return response.json();
  };
  const post = async (path, revision, payload) => {
    const body = { schemaVersion: '1.0', commandId: crypto.randomUUID(), expectedRevision: revision, payload };
    const requestStarted = performance.now();
    const response = await page.request.post(`${origin}/api/v1/${path}/commands`, { headers, data: body });
    if (payload.type === 'encounter_turn' && payload.movement === 'guided') {
      const elapsedMs = performance.now() - requestStarted;
      guidedTimings.push({ encounter: lastCombat?.encounter?.id, team: lastCombat?.team, elapsedMs });
      if (elapsedMs >= 8000) await writeFile(`/tmp/${artifactPrefix}-longplay-${starter}-${order}-combat-error.json`, JSON.stringify({ ...lastCombat, elapsedMs }, null, 2));
      assert.ok(elapsedMs < 8000, `Guided HTTP move exceeds 8s margin below10s transport budget: ${elapsedMs.toFixed(0)}ms`);
    }
    if (response.status() !== 200 && lastCombat) {
      await writeFile(`/tmp/${artifactPrefix}-longplay-${starter}-${order}-combat-error.json`, JSON.stringify(lastCombat, null, 2));
      console.error('Combat request diagnostic:', JSON.stringify(lastCombat));
    }
    assert.equal(response.status(), 200, `${path} ${JSON.stringify(payload)} ${await response.text()}`);
    return response.json();
  };
  const setup = await get('mission-learning');
  await post('mission-learning', setup.revision, { type: 'set_age', age: 8 });
  let state = await get('campaign');
  assert.equal(state.adopted, false, 'Fresh disposable profile');
  state = await post('campaign', state.revision, { type: 'adopt' });
  state = await post('campaign', state.revision, { type: 'choose_starter', companion_id: starter });
  if (useLearningUi) {
    await page.goto(origin);
    await page.locator('#game canvas').waitFor();
    await page.locator('#campaign-missions').waitFor();
  }
  const useful = (action, c) => {
    const s = c.state, flags = s.flags;
    if (action.encounter && s.grants.includes(`win:${action.encounter}`)) return false;
    // A supported retreat must let exploration continue. Retry an unbeaten
    // board only after the team or its available abilities have improved.
    if (action.encounter && attemptedBoards.has(boardAttemptKey(action, c))) return false;
    if (action.puzzle) {
      const puzzle = action.puzzle;
      if (flags.includes(`puzzle:${puzzle.id}:solved`)) return false;
      assert.ok(puzzle.goal == null, 'Public puzzle response does not expose solution');
      const goal = puzzleGoals[puzzle.id];
      assert.ok(goal, `Expected authored clue configuration for ${puzzle.id}`);
      if (puzzle.operation.startsWith('set_')) {
        const key = puzzle.operation.slice(4);
        return puzzle.value === goal[key] && !flags.includes(`puzzle:${puzzle.id}:${key}:${puzzle.value}`);
      }
      return puzzle.operation === 'check' && Object.entries(goal).every(([key,value]) => flags.includes(`puzzle:${puzzle.id}:${key}:${value}`));
    }
    if (action.recipient && !c.companions.some(friend => friend.id === action.recipient)) return false;
    if (action.recruit && c.companions.some(friend => friend.id === action.recruit)) return false;
    return true;
  };
  const attemptedBoards = new Set();
  const boardAttemptKey = (action, c) => JSON.stringify([
    action.encounter,
    c.state.team,
    c.companions.map(friend => [friend.id, friend.level]),
    c.state.flags.filter(flag => /^(technique:|free_signature|relay:)/.test(flag)),
  ]);
  const visited = new Set();
  for (let step = 0; step < 3000; step++) {
    const c = state.campaign, s = c.state;
    lastCombat = { encounter: s.encounter, team: s.team, energy: s.energy, companions: c.companions, flags: s.flags.filter(flag => /^(technique:|free_signature|relay:)/.test(flag)), flagCount: s.flags.length, grantCount: s.grants.length };
    const allGrown = c.companions.length === 12 && c.companions.every(friend => friend.level === 5);
    if (s.flags.includes('finale_complete') && allGrown && (!hasRelay || s.flags.includes('rowan_final_won'))) break;
    if (!s.encounter && s.team.length === 1 && state.locations.find(place => place.id === s.location)?.camp) {
      const actorAffinity = definition.companions.find(friend => friend.id === s.team[0]).affinity;
      const partner = c.companions.find(friend => definition.companions.find(item => item.id === friend.id)?.affinity !== actorAffinity);
      if (partner && s.flags.includes('bridge_seal')) {
        state = await post('campaign', state.revision, { type: 'set_team', companion_ids: [s.team[0], partner.id] });
        continue;
      }
    }
    if (s.encounter && c.encounter_definition?.relay && useLearningUi) {
      if (s.encounter.turns === 0) {
        await page.reload(); await page.locator('[data-campaign-move="guided"]').waitFor();
        await page.screenshot({ path: `/tmp/${artifactPrefix}-relay-${starter}-${order}-start.png`, fullPage: true });
      }
      const requestStarted = performance.now();
      const written = page.waitForResponse(r => r.url().endsWith('/campaign/commands') && r.request().postDataJSON()?.payload?.movement === 'guided');
      const refreshed = page.waitForResponse(r => r.url().endsWith('/api/v1/campaign') && r.request().method() === 'GET');
      await page.locator('[data-campaign-move="guided"]').click();
      assert.equal((await written).status(), 200);
      const elapsedMs = performance.now() - requestStarted;
      guidedTimings.push({ encounter: s.encounter.id, team: s.team, elapsedMs, controls: 'browser' });
      assert.ok(elapsedMs < 8000, `Relay guided move exceeds8s margin: ${elapsedMs.toFixed(0)}ms`);
      assert.equal((await refreshed).status(), 200);
      state = await get('campaign');
      if (!state.campaign.state.encounter) await page.locator('.campaign-confirmed-effect').waitFor();
      await page.screenshot({ path: `/tmp/${artifactPrefix}-relay-${starter}-${order}-turn-${s.encounter.turns + 1}.png`, fullPage: true });
      continue;
    }
    if (s.encounter) {
      state = await post('campaign', state.revision, { type: 'encounter_turn', companion_id: s.team[0], movement: 'guided' });
      if (!state.campaign.state.encounter && !state.campaign.state.grants.includes(`win:${s.encounter.id}`)) {
        guidedRetreats.push({ encounter: s.encounter.id, team: s.team, step });
      }
      continue;
    }
    const action = c.legal_actions.find(a => useful(a, c));
    if (action) {
      if (action.encounter) attemptedBoards.add(boardAttemptKey(action, c));
      console.log(`${starter}/${order} ${step} ${s.location} ${action.id}`);
      completed.push(action.id);
      if (action.learning) {
        const learning = await get('mission-learning');
        const begun = await post('mission-learning', learning.revision, { type: 'begin_campaign_step', actionId: action.id, campaignRevision: state.revision });
        const before = state.revision;
        if (useLearningUi) {
          await page.reload();
          await page.locator('.ww-mission-activity').waitFor();
          const save = page.waitForResponse(response => response.url().endsWith('/mission-learning/commands') && response.request().postDataJSON()?.payload?.type === 'finish_together');
          await page.getByRole('button', { name: /^(Let’s do it together|Finish activity)$/ }).click();
          assert.equal((await save).status(), 200, 'Supported completion committed');
          await page.locator('.ww-mission-activity').waitFor({ state: 'hidden', timeout: 30000 });
        } else {
          const activity = begun.activeActivity;
          assert.ok(activity, 'Real curriculum activity resolved');
          await post('mission-learning', begun.revision, { type: 'finish_together', encounterId: activity.id, variantId: activity.variantId });
        }
        state = await get('campaign');
        assert.ok(state.revision > before, `Supported learning commits ${action.id}`);
      } else state = await post('campaign', state.revision, { type: 'action', action_id: action.id });
      continue;
    }
    // Explore only legal routes. Revisit a location when progress changes.
    const fingerprint = JSON.stringify([s.flags.filter(flag => !flag.startsWith('last_camp:')), s.grants, c.companions]);
    visited.add(`${fingerprint}:${s.location}`);
    const queue = [[s.location, []]], seen = new Set([s.location]);
    let route;
    while (queue.length && !route) {
      const [location, path] = queue.shift();
      let routes = definition.locations.find(l => l.id === location).routes.filter(r => r.requires_flags.every(f => s.flags.includes(f)));
      if (location === 'orchard_hub') {
        const first = order === 'harvest' ? 'old_mill' : 'courier_pond';
        routes = [...routes].sort((a, b) => Number(b.to === first) - Number(a.to === first));
      }
      for (const next of routes) {
        if (seen.has(next.to)) continue;
        seen.add(next.to);
        const candidate = [...path, next.to];
        if (!visited.has(`${fingerprint}:${next.to}`)) { route = candidate; break; }
        queue.push([next.to, candidate]);
      }
    }
    assert.ok(route, `No progress at ${s.location}: ${JSON.stringify({ flags: s.flags, legal: c.legal_actions.map(a => a.id), companions: c.companions })}`);
    console.log(`${starter}/${order} ${step} travel ${s.location} -> ${route[0]}`);
    state = await post('campaign', state.revision, { type: 'travel', location_id: route[0] });
  }
  const harvestIndex = completed.indexOf('harvest_trial');
  const currentIndex = completed.indexOf('current_trial');
  assert.ok(harvestIndex >= 0 && currentIndex >= 0, 'Both Orchard arcs played');
  assert.equal(harvestIndex < currentIndex, order === 'harvest', 'Actual Orchard order matches requested case');
  const c = state.campaign;
  assert.ok(c.state.flags.includes('finale_complete'), 'Full ending');
  for (const seal of ['bridge', 'bloom', 'echo', 'harvest', 'current', 'tide', 'beacon', 'accord']) assert.ok(c.state.flags.includes(`${seal}_seal`), seal);
  assert.equal(c.companions.length, 12);
  assert.ok(c.companions.every(friend => friend.level === 5), JSON.stringify(c.companions));
  assert.equal(c.state.flags.filter(f => f.startsWith('milestone:')).length, 48);
  if (hasRelay) {
    assert.ok(c.state.flags.includes('rowan_final_won'));
    assert.equal(c.state.flags.filter(flag => flag.startsWith('relay:rowan_final_board:round:')).length, 3, 'Three relay rounds completed');
    assert.ok(c.state.flags.filter(flag => flag.startsWith('relay:rowan_final_board:affinity:')).length >= 2, 'At least two distinct affinities contributed signatures');
  }
  // Return through real legal routes to the nearest camp for appearance controls.
  const campQueue = [[c.state.location, []]], campSeen = new Set();
  let campPath;
  while (campQueue.length && !campPath) {
    const [location, path] = campQueue.shift();
    if (campSeen.has(location)) continue;
    campSeen.add(location);
    const place = definition.locations.find(l => l.id === location);
    if (place.camp) { campPath = path; break; }
    for (const route of place.routes.filter(r => r.requires_flags.every(f => c.state.flags.includes(f)))) campQueue.push([route.to, [...path, route.to]]);
  }
  assert.ok(campPath, 'Ending retains a legal path to camp');
  for (const location_id of campPath) state = await post('campaign', state.revision, { type: 'travel', location_id });
  const beforeRestart = state;
  if (process.send) {
    const restarted = new Promise((resolve, reject) => {
      const timer = setTimeout(() => reject(new Error('API restart timeout')), 30000);
      process.once('message', message => { clearTimeout(timer); message.type === 'api-restarted' ? resolve() : reject(new Error('API restart failed')); });
    });
    process.send({ type: 'restart-api' });
    await restarted;
    assert.deepEqual(await get('campaign'), beforeRestart, 'API restart preserves full save');
  }
  if (useLearningUi) await page.reload(); else await page.goto(origin);
  await page.locator('#game canvas').waitFor();
  assert.deepEqual(await get('campaign'), beforeRestart, 'Browser reload preserves full save');
  await page.locator('#campaign-missions').waitFor();
  await openCampaignPage(page, 'team');
  assert.equal(await page.locator('.campaign-card').count(), 12, 'Collection renders all twelve friends');
  const form = { type: 'care', companion_id: starter, kind: 'appearance_wayfinder' };
  const formResponse = page.waitForResponse(r => r.url().endsWith('/campaign/commands') && r.request().postDataJSON()?.payload?.kind === 'appearance_wayfinder');
  await page.locator(`[data-campaign-payload='${JSON.stringify(form)}']`).click();
  assert.equal((await formResponse).status(), 200);
  await page.locator('.campaign-status').filter({ hasText: /Saved/ }).waitFor();
  assert.equal((await get('campaign')).campaign.companions.find(f => f.id === starter).appearance, 'wayfinder');
  await page.reload(); await page.locator('#campaign-missions').waitFor();
  assert.equal((await get('campaign')).campaign.companions.find(f => f.id === starter).appearance, 'wayfinder', 'Canonical form survives reload');
  await openCampaignPage(page, 'team');
  await page.setViewportSize({ width: 1280, height: 800 });
  const overflowingControls = await page.locator('.campaign-card').evaluateAll(cards => cards.flatMap(card => {
    const box = card.getBoundingClientRect();
    return [...card.querySelectorAll('button')].filter(button => { const rect = button.getBoundingClientRect(); return rect.left < box.left - 1 || rect.right > box.right + 1; }).map(button => button.textContent);
  }));
  assert.deepEqual(overflowingControls, [], 'Collection controls stay inside their cards');
  const prefix = `/tmp/${artifactPrefix}-ending-${starter}-${order}`;
  await page.screenshot({ path: `${prefix}-team.png`, fullPage: true });
  const glimmer = page.locator('.campaign-card').filter({ has: page.getByRole('heading', { name: 'Glimmer', exact: true }) });
  await glimmer.scrollIntoViewIfNeeded();
  await glimmer.screenshot({ path: `${prefix}-glimmer.png` });
  await openCampaignPage(page, 'journal');
  await page.screenshot({ path: `${prefix}-journal.png`, fullPage: true });
  await openCampaignPage(page, 'guide');
  await page.screenshot({ path: `${prefix}-guide.png`, fullPage: true });
  await page.setViewportSize({ width: 390, height: 844 });
  assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), 'Phone has no horizontal overflow');
  await page.screenshot({ path: `${prefix}-guide-phone.png`, fullPage: true });
  await openCampaignPage(page, 'journal');
  await page.screenshot({ path: `${prefix}-journal-phone.png`, fullPage: true });
  await openCampaignPage(page, 'team');
  await page.screenshot({ path: `${prefix}-team-phone.png`, fullPage: true });
  state = await get('campaign');
  assert.deepEqual(errors, []);
  await writeFile(`/tmp/${artifactPrefix}-longplay-${starter}-${order}.json`, JSON.stringify({ stage, guidedPerformance: { budgetMs: 8000, maxMs: Math.max(...guidedTimings.map(timing => timing.elapsedMs)), samples: guidedTimings }, guidedRetreats, contentVersion: definition.version, starter, order, relay: hasRelay ? { rounds: 3, affinities: c.state.flags.filter(flag => flag.startsWith('relay:rowan_final_board:affinity:')), renderedTurns: useLearningUi } : undefined, learningControls: useLearningUi ? 'actual browser supported controls' : 'authenticated API begin/finish_together', actions: completed, snapshot: state }, null, 2));
  console.log(`Full campaign ${starter}/${order}: eight seals, twelve level-five friends, 48 milestones, ending and restart passed`);
} catch (error) {
  await page.screenshot({ path: `/tmp/${artifactPrefix}-longplay-${starter}-${order}-failure.png`, fullPage: true });
  console.error((await page.locator('body').innerText()).slice(-4000));
  throw error;
} finally { await browser.close(); }
