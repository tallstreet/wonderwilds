// Run through run-training-qa.mjs with synthetic Modal credentials and an unavailable
// loopback endpoint. Exercises saved offline journeys, never paid calls or demo data.
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
    await page.locator(`[data-story-action="${type}"]`).click();
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
  await travel('workshop');
  await travel('cove');
  await visit('npc:marina'); await choose('talk');
  let quest = (await snapshot()).sceneInput.neighbourMissions.find(q => q.characterId === 'marina');
  assert.equal(quest.storySource, 'authored_fallback');
  assert.equal(quest.generatedStory.steps.length, 2);
  assert.match(await page.locator('#mission-status').textContent(), /Listen for the little tune/);
  await visit('object:neighbour_marina_0'); await choose('inspect');
  await page.getByText(quest.generatedStory.steps[0].discovery, { exact: true }).waitFor();
  assert.match(await page.locator('#mission-status').textContent(), /Fern Hollow/);
  await page.setViewportSize({ width: 390, height: 844 });
  await travel('fern_hollow');
  await page.reload(); await page.locator('#game canvas').waitFor(); await sync();
  assert.match(await page.locator('#mission-status').textContent(), /Find where the breeze makes music.*Fern Hollow/);
  await page.locator('#missions').click();
  await page.locator('dialog').getByText(/Find where the breeze makes music.*Fern Hollow/).waitFor();
  await close();
  await visit('object:neighbour_marina_1'); await choose('inspect');
  await page.getByText(quest.generatedStory.steps[1].discovery, { exact: true }).waitFor();
  assert.match(await page.locator('#mission-status').textContent(), /Return to Marina.*Starlight Cove/);
  await page.screenshot({ path: '/tmp/wonderwilds-mission-journey-phone.png' });
  await travel('cove'); await visit('npc:marina'); await choose('talk');
  quest = (await snapshot()).sceneInput.neighbourMissions.find(q => q.characterId === 'marina');
  assert.equal(quest.status, 'completed');
  assert.doesNotMatch(await page.locator('#mission-status').textContent(), /Return to Marina/);
  await travel('fern_hollow'); await travel('cove');
  quest = (await snapshot()).sceneInput.neighbourMissions.find(q => q.characterId === 'marina');
  assert.equal(quest.status, 'offered');
  assert.equal(quest.generatedStory.steps.length, 1);
  assert.equal(quest.generatedStory.steps[0].shape, 'leaf');
  assert.deepEqual(errors, []);
  console.log('PASS: cross-area learning steps, clue replies, tracker, phone journal, reload, return/payoff and varied repeat story');
} finally { await browser.close(); }
