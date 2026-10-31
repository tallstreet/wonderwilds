// Run through run-training-qa.mjs with synthetic Modal credentials and an unavailable
// loopback endpoint. Uses a disposable profile, never paid calls or demo progress.
import assert from 'node:assert/strict';
import { chromium } from 'playwright';
import { finishInlineStep } from './inline-step.mjs';
const origin = process.env.WW_DEMO_TEST_URL ?? 'http://127.0.0.1:5180';
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH });
const page = await browser.newPage({ viewport: { width: 1280, height: 844 } });
const errors = [];
page.on('pageerror', error => errors.push(error.message));
try {
  await page.goto(origin);
  await page.locator('#game canvas').waitFor();
  const bootstrap = await (await page.request.get(`${origin}/__demo/bootstrap`)).json();
  const snapshot = async () => {
    const response = await page.request.get(`${origin}/api/v1/story`, { headers: { 'X-Wonderwilds-Profile': bootstrap.profileId } });
    assert.equal(response.status(), 200);
    return response.json();
  };
  assert.equal((await snapshot()).revision, 0, 'fresh disposable profile only');
  const sync = async () => {
    const state = await snapshot();
    await page.waitForFunction(revision => document.querySelector('#game')?.dataset.storyRevision === String(revision), state.revision);
    return state;
  };
  const close = async () => {
    if (await page.locator('dialog[open]').count()) await page.getByRole('button', { name: 'Close dialog', exact: true }).click();
  };
  const visit = async id => {
    await close();
    await page.locator('#missions').click();
    await page.locator(`[data-story-visit="${id}"]`).click();
  };
  const choose = async type => {
    const before = (await snapshot()).revision;
    await page.locator(`[data-story-action="${type}"]`).first().click();
    await finishInlineStep(page, type);
    await page.waitForFunction(revision => Number(document.querySelector('#game')?.dataset.storyRevision) > revision, before);
    return sync();
  };
  const travel = async destination => {
    await visit(`route:${destination}`);
    await Promise.any([
      page.locator('[data-story-action="travel"]').waitFor({ timeout: 20000 }),
      page.waitForFunction(destination => document.querySelector('#game')?.dataset.storyLocation === destination, destination, { timeout: 20000 }),
    ]);
    if ((await snapshot()).locationId !== destination) await choose('travel');
    return sync();
  };
  const quest = state => state.sceneInput.neighbourMissions.find(q => q.characterId === 'marina');
  const mechanics = new Set();
  const projects = new Set();
  await travel('workshop');
  await travel('cove');
  for (let cycle = 0; cycle < 8; cycle++) {
    let state = await snapshot();
    if (state.locationId !== 'cove') await travel('cove');
    state = await snapshot();
    if (quest(state)?.status === 'completed') {
      await travel('workshop'); await travel('cove');
      state = await snapshot();
    }
    const offered = quest(state);
    assert.equal(offered.status, 'offered');
    assert.equal(offered.storySource, 'authored_fallback');
    await visit('npc:marina'); await choose('talk');
    for (let index = 0; index < offered.generatedStory.steps.length; index++) {
      const step = offered.generatedStory.steps[index];
      const mechanic = step.mechanic ?? 'inspect';
      mechanics.add(mechanic);
      state = await snapshot();
      if (state.locationId !== step.locationId) await travel(step.locationId);
      const current = await snapshot();
      const action = current.sceneInput.availableActions.find(a =>
        ['sourceId', 'objectId', 'choiceId'].some(field => a.action[field]?.startsWith('neighbour_marina_')));
      assert.ok(action, `missing ${mechanic} action`);
      const target = action.action.sourceId ? `resource:${action.action.sourceId}`
        : `object:${action.action.objectId ?? action.action.choiceId}`;
      await visit(target);
      if (mechanic === 'choose_style') {
        assert.equal(await page.locator('[data-story-action="choose"]').count(), 2);
        await page.locator('[data-story-action="choose"]').filter({ hasText: 'starry' }).click();
        await page.waitForFunction(revision => Number(document.querySelector('#game')?.dataset.storyRevision) > revision, current.revision);
        await sync();
      } else await choose(action.action.type);
      state = await snapshot();
      assert.equal(quest(state).stepIndex, index + 1);
      if (mechanic.startsWith('gather_') || mechanic.startsWith('craft_') || mechanic === 'pack_basket') {
        const item = { gather_wood: 'wood', gather_berries: 'berries', gather_glow_pebbles: 'glow_pebbles', craft_planks: 'planks', craft_lantern: 'lanterns', pack_basket: 'picnic_basket' }[mechanic];
        assert.ok(state.sceneInput.context.inventory.some(i => i.itemId === item && i.quantity > 0));
        await close(); await page.locator('#bag').click();
        await page.locator('.story-bag-items').waitFor();
        await close();
      }
      if (cycle === 0 && index === 1) {
        await page.setViewportSize({ width: 390, height: 844 });
        await page.reload(); await page.locator('#game canvas').waitFor(); await sync();
        assert.equal(quest(await snapshot()).stepIndex, 2, 'craft progress survives reload');
      }
    }
    state = await snapshot();
    if (offered.generatedStory.completeOnLastStep) {
      assert.equal(quest(state).status, 'completed');
      assert.notEqual(state.locationId, 'cove', 'project finishes at destination');
      assert.doesNotMatch(await page.locator('#mission-status').textContent(), /Return to Marina/);
      const objects = state.sceneInput.here.objects.filter(o => o.id.startsWith('project_'));
      assert.ok(objects.length, 'finished project has a world object');
      for (const object of objects) projects.add(object.id);
      const before = objects;
      await close(); await page.reload(); await page.locator('#game canvas').waitFor(); await sync();
      assert.deepEqual((await snapshot()).sceneInput.here.objects.filter(o => o.id.startsWith('project_')), before);
    } else {
      await travel('cove'); await visit('npc:marina'); await choose('talk');
      assert.equal(quest(await snapshot()).status, 'completed');
    }
    console.log(`Completed ${cycle + 1}: ${offered.title}`);
  }
  for (const mechanic of ['gather_wood', 'craft_planks', 'build_seat', 'gather_berries', 'pack_basket', 'deliver_basket', 'gather_glow_pebbles', 'craft_lantern', 'place_lantern', 'choose_style', 'inspect']) {
    assert.ok(mechanics.has(mechanic), `journey did not cover ${mechanic}`);
  }
  assert.ok(projects.size >= 3);
  await page.screenshot({ path: '/tmp/wonderwilds-project-missions-phone.png' });
  assert.deepEqual(errors, []);
  console.log('PASS: real gathering/crafting/delivery/building/design choices, learning joins, bag quantities, persistent projects, destination completion and discovery variety');
} finally { await browser.close(); }
