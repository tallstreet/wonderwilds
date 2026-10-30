// Real world, API, and disposable PostgreSQL regression. Never run against demo data.
import assert from 'node:assert/strict';
import { finishInlineStep } from './inline-step.mjs';
import { chromium } from 'playwright';
const generated = process.env.WW_GENERATED_STORIES === '1';
const origin = process.env.WW_DEMO_TEST_URL ?? 'http://127.0.0.1:5180';
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH });
const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
const errors = [];
page.on('pageerror', error => errors.push(error.message));
try {
  await page.goto(`${origin}/?story=1`);
  await page.locator('#game canvas').waitFor();
  const bootstrap = await (await page.request.get(`${origin}/__demo/bootstrap`)).json();
  const snapshot = async () => {
    const response = await page.request.get(`${origin}/api/v1/story`, { headers: { 'X-Wonderwilds-Profile': bootstrap.profileId } });
    assert.equal(response.status(), 200);
    return response.json();
  };
  assert.equal((await snapshot()).revision, 0, 'fresh synthetic profile only');
  const ready = async () => {
    const state = await snapshot();
    await page.waitForFunction(revision => document.querySelector('#game')?.dataset.storyRevision === String(revision), state.revision);
    return state;
  };
  const close = async () => {
    if (await page.locator('dialog[open]').count()) await page.getByRole('button', { name: 'Close dialog', exact: true }).click();
  };
  const visit = async id => {
    console.log('Visit', id);
    await close();
    await page.locator('#missions').click();
    await page.locator(`[data-story-visit="${id}"]`).click();
    const dialogue = page.locator('dialog[open] [data-story-action]').first().waitFor({ timeout: 20000 });
    if (id.startsWith('route:')) {
      await Promise.any([dialogue, page.waitForFunction(destination => document.querySelector('#game')?.dataset.storyLocation === destination, id.slice(6), { timeout: 20000 })]);
    } else await dialogue;
  };
  const choose = async type => {
    const previous = (await snapshot()).revision;
    await page.locator(`[data-story-action="${type}"]`).click();
    await finishInlineStep(page, type);
    await page.waitForFunction(revision => Number(document.querySelector('#game')?.dataset.storyRevision) > revision, previous, { timeout: 20000 });
    return ready();
  };
  const travel = async destination => {
    await visit(`route:${destination}`);
    if ((await snapshot()).locationId !== destination) await choose('travel');
    await ready();
  };
  await visit('npc:pip'); await choose('accept_mission'); await close();
  // Walk onto the actual map edge using keyboard input, with no travel button.
  await page.keyboard.down('ArrowDown');
  await page.waitForFunction(() => Number(document.querySelector('#game')?.dataset.storyPlayerY) >= 475);
  await page.keyboard.up('ArrowDown');
  // Pip approach leaves us at y430; align exactly with the edge path.
  let y = Number(await page.locator('#game').getAttribute('data-story-player-y'));
  if (y > 483) {
    await page.keyboard.down('ArrowUp');
    await page.waitForFunction(() => Number(document.querySelector('#game')?.dataset.storyPlayerY) <= 480);
    await page.keyboard.up('ArrowUp');
  }
  await page.keyboard.down('ArrowLeft');
  await page.waitForFunction(() => document.querySelector('#game')?.dataset.storyLocation === 'workshop', null, { timeout: 20000 });
  await page.keyboard.up('ArrowLeft');
  assert.equal((await ready()).locationId, 'workshop');
  const places = ['cove', 'fern_hollow', 'mossy_clearing', 'owl_grove', 'acorn_trail'];
  const friends = ['marina', 'fern', 'moss', 'orla', 'hazel'];
  let lostBody;
  for (let i = 0; i < places.length; i++) {
    await travel(places[i]);
    assert.equal((await snapshot()).locationId, places[i]);
    if (generated) {
      const state = await snapshot();
      const mission = state.sceneInput.neighbourMissions.find(q => q.characterId === friends[i]);
      assert.equal(mission.status, 'offered');
      assert.equal(mission.storySource, i === 4 ? 'authored_fallback' : 'llm');
      if (i !== 4) assert.equal(mission.generatedStory.name, `Visitor ${i + 1}`);
    }
    await visit(`npc:${friends[i]}`);
    if (i === 0) {
      await page.route('**/api/v1/story/commands', async route => {
        lostBody = route.request().postData();
        const response = await route.fetch(); assert.equal(response.status(), 200);
        await route.abort('failed');
      }, { times: 1 });
      await page.locator('[data-story-action="talk"]').click();
      await page.getByRole('button', { name: 'Retry the same action', exact: true }).waitFor();
      await page.reload(); await page.locator('#game canvas').waitFor();
      await page.locator('#missions').click();
      const retry = page.waitForRequest(request => request.url().endsWith('/api/v1/story/commands'));
      await page.getByRole('button', { name: 'Retry the same action', exact: true }).click();
      assert.equal((await retry).postData(), lostBody);
      await page.waitForFunction(() => !document.querySelector('dialog[open]'));
      await ready();
    } else await choose('talk');
    const taskCount = generated ? (await snapshot()).sceneInput.neighbourMissions.find(q => q.characterId === friends[i]).generatedStory.steps.length : 1;
    for (let task = 0; task < taskCount; task++) {
      await visit(`object:neighbour_${friends[i]}${generated ? `_${task}` : ''}`); await choose('inspect');
    }
    const state = await snapshot();
    assert.ok(state.sceneInput.neighbourMissions.filter(q => q.status === 'open').length <= 3);
    assert.equal(state.sceneInput.mission.status, 'active');
  }
  let state = await snapshot();
  assert.equal(state.sceneInput.neighbourMissions[0].status, 'archived');
  assert.equal(state.sceneInput.neighbourMissions[1].status, 'archived');
  await page.reload(); await page.locator('#game canvas').waitFor(); await ready();
  await page.locator('#missions').click();
  await page.getByText(/Neighbourhood missions · 4 \/ 4 open/).waitFor();
  await page.screenshot({ path: '/tmp/wonderwilds-exploration-desktop.png', fullPage: true });
  for (const place of ['owl_grove', 'mossy_clearing', 'fern_hollow', 'cove']) {
    await travel(place);
  }
  await visit('npc:marina'); await choose('talk');
  state = await snapshot();
  assert.equal(state.sceneInput.neighbourMissions.at(-1).inspected, true);
  await visit('npc:marina'); await choose('talk');
  assert.equal((await snapshot()).sceneInput.neighbourMissions.at(-1).status, 'completed');
  await page.setViewportSize({ width: 390, height: 844 });
  await page.locator('#missions').click();
  await page.screenshot({ path: '/tmp/wonderwilds-exploration-phone.png', fullPage: true });
  assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth));
  assert.ok(await page.locator('dialog').evaluate(node => node.scrollWidth <= node.clientWidth), 'mission text must wrap inside the phone dialog');
  await travel('workshop');
  if (generated) {
    await travel('cove');
    const mission = (await snapshot()).sceneInput.neighbourMissions.find(q => q.characterId === 'marina');
    assert.equal(mission.generation, 2);
    assert.equal(mission.generatedStory.name, 'Visitor 6');
    assert.equal(mission.status, 'offered');
    const content = mission.generatedStory;
    await page.reload(); await page.locator('#game canvas').waitFor(); await ready();
    assert.deepEqual((await snapshot()).sceneInput.neighbourMissions.find(q => q.characterId === 'marina').generatedStory, content);
    await travel('workshop');
  }
  await travel('bridge_bank');
  assert.equal((await snapshot()).sceneInput.mission.status, 'active');
  assert.deepEqual(errors, []);
  console.log('Story exploration: walking exit, five friends, cap/archive/resume, completion, exact retry, reload and mobile passed.');
} catch (error) {
  await page.screenshot({ path: '/tmp/wonderwilds-exploration-failure.png', fullPage: true });
  console.log(await page.locator('#game').evaluate(node => ({ ...node.dataset })));
  throw error;
} finally { await browser.close(); }
