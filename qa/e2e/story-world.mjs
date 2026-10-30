// Actual Phaser journey against a fresh disposable API/PostgreSQL profile.
// Run with WW_QA_BROWSER_TEST=qa/e2e/story-world.mjs via run-training-qa.mjs.
import assert from 'node:assert/strict';
import { chromium } from 'playwright';
const origin = process.env.WW_DEMO_TEST_URL ?? 'http://127.0.0.1:5180';
const source = process.env.WW_STORY_SOURCE ?? 'bank';
assert.ok(['bank', 'workshop'].includes(source));
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH });
const context = await browser.newContext({ viewport: { width: 1280, height: 800 } });
const page = await context.newPage();
const errors = [];
page.on('pageerror', error => errors.push(error.message));
const mockLine = 'Pip says: follow the silver leaves to our bridge. <img src=x onerror="window.storyInjected=true">';
const mockChoice = 'Let’s help Pip cross together';
const guestSave = '{"keep":"untouched guest story bytes"}';
await page.addInitScript(value => {
  if (!localStorage.getItem('starwhisker.meadow.v1')) localStorage.setItem('starwhisker.meadow.v1', value);
}, guestSave);
try {
  // Change presentation only, keeping the real server's IDs, choices and mechanics.
  // This catches a client that quietly hardcodes its own dialogue instead of rendering output.
  await page.route('**/api/v1/story', async route => {
    const response = await route.fetch();
    assert.equal(response.status(), 200);
    const value = await response.json();
    assert.equal(value.revision, 0, 'Must use a fresh disposable QA profile');
    value.scene.presentation[0].text = mockLine;
    value.scene.choices.find(choice => value.sceneInput.availableActions.some(action => action.id === choice.actionId && action.action.type === 'accept_mission')).label = mockChoice;
    await route.fulfill({ response, json: value });
  }, { times: 1 });
  await page.goto(origin);
  await page.locator('#game canvas').waitFor();
  await page.getByText(/Local server · progress saved/).waitFor();
  await page.locator('#missions').click();
  await page.locator('#story-adventure-link').click();
  await page.waitForURL('**/?story=1');
  await page.locator('#game canvas').waitFor();
  await page.waitForFunction(() => document.querySelector('#game')?.dataset.storyRevision === '0');
  const bootstrap = await (await page.request.get(`${origin}/__demo/bootstrap`)).json();
  const get = async path => {
    const response = await page.request.get(`${origin}${path}`, { headers: { 'X-Wonderwilds-Profile': bootstrap.profileId } });
    assert.equal(response.status(), 200);
    return response.json();
  };
  const snapshot = () => get('/api/v1/story');
  const originalGame = await get('/api/v1/game');
  const position = () => page.locator('#game').evaluate(node => ({ x: Number(node.dataset.storyPlayerX), y: Number(node.dataset.storyPlayerY) }));
  const close = async () => {
    if (await page.locator('dialog[open]').count()) await page.getByRole('button', { name: 'Close dialog', exact: true }).click();
  };
  const ready = async () => {
    const state = await snapshot();
    await page.waitForFunction(revision => document.querySelector('#game')?.dataset.storyRevision === String(revision), state.revision);
    return state;
  };
  const visit = async id => {
    await close();
    await page.locator('#missions').click();
    await page.locator(`[data-story-visit="${id}"]`).click();
    await page.locator('dialog[open] [data-story-action], dialog[open] [data-story-continue]').first().waitFor({ timeout: 20000 });
  };
  const choose = async type => {
    const previous = (await snapshot()).revision;
    await page.locator(`[data-story-action="${type}"]`).click();
    await page.waitForFunction(revision => Number(document.querySelector('#game')?.dataset.storyRevision) > revision, previous, { timeout: 20000 });
    return ready();
  };
  const initialPosition = await position();
  await visit('npc:pip');
  assert.ok(Math.hypot((await position()).x - initialPosition.x, (await position()).y - initialPosition.y) > 30, 'Visiting Pip walks the real player');
  await page.getByText(mockLine, { exact: true }).waitFor();
  await page.getByRole('button', { name: mockChoice, exact: true }).waitFor();
  assert.equal(await page.locator('dialog img[src="x"]').count(), 0);
  assert.equal(await page.evaluate(() => window.storyInjected), undefined);
  await choose('accept_mission');
  await page.screenshot({ path: '/tmp/wonderwilds-story-world-desktop.png', fullPage: true });
  if (source === 'bank') { await visit('resource:fallen_wood'); await choose('collect'); }
  await visit('route:workshop'); await choose('travel');
  assert.equal((await snapshot()).locationId, 'workshop');
  await visit('npc:woodcutter');
  const beforeAck = (await snapshot()).revision;
  await page.locator('[data-story-continue]').waitFor();
  await page.reload(); await page.locator('#game canvas').waitFor(); await ready();
  assert.equal((await snapshot()).revision, beforeAck, 'Rendering/reloading never acknowledges presentation');
  await visit('npc:woodcutter');
  await page.locator('[data-story-continue]').click();
  await page.waitForFunction(revision => Number(document.querySelector('#game')?.dataset.storyRevision) > revision, beforeAck);
  if (source === 'workshop') { await visit('resource:spare_wood'); await choose('collect'); }
  assert.deepEqual((await snapshot()).sceneInput.context.inventory, [{ itemId: 'wood', quantity: 3 }]);
  await visit('npc:woodcutter');
  let originalBody;
  await page.route('**/api/v1/story/commands', async route => {
    originalBody = route.request().postData();
    const response = await route.fetch(); assert.equal(response.status(), 200);
    await route.abort('failed');
  }, { times: 1 });
  await page.locator('[data-story-action="exchange"]').click();
  await page.getByRole('button', { name: 'Retry the same action', exact: true }).waitFor();
  const committedRevision = (await snapshot()).revision;
  // A second tab advances the real server via UI, while this tab retains the lost exchange.
  const other = await page.context().newPage();
  other.on('pageerror', error => errors.push(error.message));
  await other.goto(`${origin}/?story=1`);
  await other.locator('#game canvas').waitFor();
  await other.waitForFunction(revision => document.querySelector('#game')?.dataset.storyRevision === String(revision), committedRevision);
  assert.equal(await other.evaluate(() => Object.keys(sessionStorage).some(key => key.startsWith('wonderwilds.story.pending.'))), false);
  await other.locator('#missions').click();
  await other.locator('[data-story-visit="route:bridge_bank"]').click();
  await other.locator('dialog[open] [data-story-action="travel"]').click();
  await other.waitForFunction(revision => Number(document.querySelector('#game')?.dataset.storyRevision) > revision, committedRevision);
  const advancedRevision = (await snapshot()).revision;
  assert.equal((await snapshot()).locationId, 'bridge_bank');
  await other.close();
  await page.reload(); await page.locator('#game canvas').waitFor(); await ready();
  await page.locator('#missions').click();
  await page.getByRole('button', { name: 'Retry the same action', exact: true }).waitFor();
  assert.equal(await page.locator('#game').getAttribute('data-story-location'), 'bridge_bank');
  assert.equal(await page.locator('#game').getAttribute('data-story-revision'), String(advancedRevision));
  // The old receipt is valid, but its following reconciliation GET fails once.
  await page.route('**/api/v1/story', route => route.fulfill({ status: 503, json: { code: 'UNAVAILABLE' } }), { times: 1 });
  const retry = page.waitForRequest(request => request.url().endsWith('/story/commands'));
  await page.getByRole('button', { name: 'Retry the same action', exact: true }).click();
  assert.equal((await retry).postData(), originalBody, 'Reload retries the identical prepared body');
  await page.locator('#story-world-status').filter({ hasText: 'Progress could not be refreshed' }).waitFor();
  assert.equal(await page.locator('#game').getAttribute('data-story-location'), 'bridge_bank', 'An older receipt never rolls the rendered world back');
  assert.equal(await page.locator('#game').getAttribute('data-story-revision'), String(advancedRevision));
  assert.equal(await page.locator('[data-story-action]:enabled, [data-story-visit]:enabled').count(), 0);
  await close();
  const blockedPosition = await position();
  await page.keyboard.down('ArrowRight'); await page.waitForTimeout(300); await page.keyboard.up('ArrowRight');
  assert.deepEqual(await position(), blockedPosition, 'Movement stays blocked until authoritative refresh succeeds');
  await page.locator('#missions').click();
  await page.getByRole('button', { name: 'Refresh progress', exact: true }).click();
  await page.locator('dialog').waitFor({ state: 'hidden' });
  await ready();
  assert.equal((await snapshot()).revision, advancedRevision);
  assert.deepEqual((await snapshot()).sceneInput.context.inventory, [{ itemId: 'planks', quantity: 3 }]);
  if ((await snapshot()).locationId !== 'bridge_bank') { await visit('route:bridge_bank'); await choose('travel'); }
  await page.setViewportSize({ width: 390, height: 844 });
  await visit('object:bridge');
  assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth));
  assert.equal(await page.locator('#game').getAttribute('data-story-bridge'), 'broken');
  const attemptUncommittedCrossing = async () => {
    await close();
    const revision = (await snapshot()).revision;
    if ((await position()).y > 360) {
      await page.keyboard.down('ArrowUp');
      await page.waitForFunction(() => Number(document.querySelector('#game')?.dataset.storyPlayerY) <= 360);
      await page.keyboard.up('ArrowUp');
    }
    assert.ok((await position()).y >= 350, 'Walking attempt aligns with the bridge deck');
    await page.keyboard.down('ArrowRight');
    await page.waitForTimeout(1200);
    await page.keyboard.up('ArrowRight');
    assert.ok((await position()).x <= 601, 'Free walking cannot change the authoritative bank');
    assert.equal((await snapshot()).revision, revision);
    assert.equal((await snapshot()).locationId, 'bridge_bank');
  };
  await attemptUncommittedCrossing();
  await visit('object:bridge');
  await choose('assemble');
  assert.equal(await page.locator('#game').getAttribute('data-story-bridge'), 'repaired');
  assert.notEqual((await snapshot()).sceneInput.mission.status, 'completed');
  await attemptUncommittedCrossing();
  await page.screenshot({ path: '/tmp/wonderwilds-story-world-mobile.png', fullPage: true });
  await visit('npc:pip');
  const crossingEvidence = page.waitForFunction(() => {
    const data = document.querySelector('#game')?.dataset;
    const followers = JSON.parse(data?.storyFollowers || '[]');
    const x = Number(data?.storyPlayerX);
    return data?.storyCrossing === 'true' && x > 620 && x < 760 && followers.length === 1
      ? { x, followers, markers: JSON.parse(data.storyMarkers || '[]') } : false;
  }, null, { timeout: 15000 });
  await choose('accompany');
  const duringCrossing = await (await crossingEvidence).jsonValue();
  assert.equal(duringCrossing.followers[0].sprite, 'pip');
  assert.equal(duringCrossing.markers.some(marker => marker.id === 'npc:pip'), false, 'Pip is a follower during crossing, not a duplicate NPC');
  await page.screenshot({ path: '/tmp/wonderwilds-story-world-crossing.png', fullPage: true });
  await page.waitForFunction(() => document.querySelector('#game')?.dataset.storyCrossing === 'false' && Number(document.querySelector('#game')?.dataset.storyPlayerX) > 770, null, { timeout: 15000 });
  const final = await ready();
  assert.equal(final.locationId, 'far_bank');
  assert.equal(final.sceneInput.mission.status, 'completed');
  assert.deepEqual(final.sceneInput.context.inventory, []);
  assert.equal(final.sceneInput.mission.remaining.length, 0);
  await close();
  assert.ok((await position()).x > 770, 'Shared crossing puts the real player on the far bank');
  const farMarkers = JSON.parse(await page.locator('#game').getAttribute('data-story-markers'));
  assert.equal(farMarkers.filter(marker => marker.id === 'npc:pip').length, 1);
  assert.ok(farMarkers.find(marker => marker.id === 'npc:pip').x > 770);
  assert.deepEqual(JSON.parse(await page.locator('#game').getAttribute('data-story-followers')), []);
  await page.reload(); await page.locator('#game canvas').waitFor(); await ready();
  assert.equal((await snapshot()).revision, final.revision);
  assert.ok((await position()).x > 770, 'Reload restores the authoritative bank');
  assert.deepEqual(await get('/api/v1/game'), originalGame, 'Story never mutates assessed progress');
  assert.equal(await page.evaluate(() => localStorage.getItem('starwhisker.meadow.v1')), guestSave);
  // Presentation-only cap response: never manufacture 256 server commands or mutate saves.
  await page.route('**/api/v1/story', route => route.fulfill({ json: {
    schemaVersion: '1.0', contentVersion: final.contentVersion, revision: 256,
    locationId: final.locationId, commandBudgetExhausted: true, acknowledgeableEntryIds: [],
  } }), { times: 1 });
  await page.reload();
  await page.getByRole('heading', { name: 'This development story has reached its save limit', exact: true }).waitFor();
  assert.equal(await page.locator('#game canvas').count(), 0);
  assert.equal(await page.locator('[data-story-action], [data-story-visit], [data-story-continue]').count(), 0);
  assert.equal((await snapshot()).revision, final.revision, 'Terminal rendering does not alter saved progress');
  assert.deepEqual(errors, []);
  console.log(`PASS: Phaser ${source} route, walked NPC approach, response-driven inert text/choices, explicit ack, stale receipt/reload and failed-refresh recovery, terminal view, repaired bridge/shared crossing, phone layout, separate education/guest data`);
} catch (error) {
  console.error('Story QA state:', await page.locator('#game').evaluate(node => ({ ...node.dataset })).catch(() => ({})));
  console.error('Visible dialog:', await page.locator('dialog[open]').innerText().catch(() => 'none'));
  await page.screenshot({ path: '/tmp/wonderwilds-story-world-failure.png', fullPage: true }).catch(() => {});
  throw error;
} finally { await browser.close(); }
