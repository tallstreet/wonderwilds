// Presentation-only fixtures: no demo database or provider requests.
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { chromium } from 'playwright';
const fixtures = JSON.parse(await readFile(new URL('../../contracts/fixtures/cases.json', import.meta.url), 'utf8'));
const base = fixtures.find(item => item.contract === 'StorySnapshot' && item.valid && item.value.scene).value;
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH });
const origin = process.env.WONDERWILDS_TEST_URL ?? 'http://127.0.0.1:5189';
try {
  for (const width of [1280, 390]) {
    const page = await browser.newPage({ viewport: { width, height: 844 } });
    const errors = [];
    page.on('pageerror', error => errors.push(error.message));
    let snapshot = structuredClone(base);
    let writes = 0;
    await page.route('**/__demo/bootstrap', route => route.fulfill({ json: {
      profileId: '00000000-0000-4000-8000-000000000001', csrf: 'synthetic-token-for-browser-testing-only',
    } }));
    await page.route('**/api/v1/mission-learning**', route => route.fulfill({ status: 503, json: {} }));
    await page.route('**/api/v1/story**', route => {
      if (route.request().method() !== 'GET') writes++;
      return route.fulfill({ json: snapshot });
    });
    const load = async () => {
      await page.goto(`${origin}/?story=1`);
      await page.locator('#game canvas').waitFor();
      await page.waitForFunction(() => document.querySelector('#game').dataset.storyRevision === '3');
    };
    const visit = async () => {
      await page.locator('#missions').click();
      await page.locator('[data-story-visit="npc:woodcutter"]').click();
    };
    await load();
    await page.locator('#missions').click();
    await page.screenshot({ path: `/tmp/wonderwilds-missions-${width}.png` });
    const dialog = await page.locator('dialog').boundingBox();
    assert.ok(dialog.x >= 0 && dialog.x + dialog.width <= width);
    await page.getByRole('button', { name: 'Close dialog', exact: true }).click();
    await visit();
    await page.locator('[data-story-continue]').waitFor({ timeout: 20000 });
    assert.equal(await page.locator('.world-speech:visible').count(), 0, 'Required acknowledgement stays explicit');
    await page.screenshot({ path: `/tmp/wonderwilds-choice-${width}.png` });
    snapshot.acknowledgeableEntryIds = [];
    snapshot.sceneInput.availableActions = snapshot.sceneInput.availableActions.filter(item => item.action.type === 'travel');
    snapshot.scene.choices = [{ actionId: snapshot.sceneInput.availableActions[0].id, label: 'Return to the bridge' }];
    const text = 'The planks are ready. <img src=x onerror=alert(1)> Come back any time!';
    snapshot.scene.presentation[0].text = text;
    snapshot.revision = 4;
    await page.locator('[data-story-continue]').click();
    const bubble = page.locator('.world-speech:visible');
    await bubble.waitFor({ timeout: 20000 });
    assert.equal(await page.locator('dialog[open]').count(), 0);
    assert.equal(await bubble.locator('img').count(), 0);
    assert.ok((await bubble.innerText()).includes(text));
    await page.screenshot({ path: `/tmp/wonderwilds-speech-${width}.png` });
    const bounds = await bubble.boundingBox();
    assert.ok(bounds.x >= 0 && bounds.x + bounds.width <= width);
    await page.keyboard.press('Escape');
    await bubble.waitFor({ state: 'hidden' });
    await page.keyboard.press('Space');
    await bubble.waitFor();
    await page.getByRole('button', { name: 'Dismiss speech' }).click();
    await bubble.waitFor({ state: 'hidden' });
    await page.keyboard.press('Space');
    await bubble.waitFor();
    await page.keyboard.down('ArrowRight');
    await bubble.waitFor({ state: 'hidden' });
    await page.keyboard.up('ArrowRight');
    // Revisit after movement and ensure Missions clears the non-modal overlay.
    await visit();
    await bubble.waitFor({ timeout: 20000 });
    await page.locator('#missions').click();
    await bubble.waitFor({ state: 'hidden' });
    assert.equal(writes, 1, 'Only the explicit acknowledgement may commit progress; reading/dismissing never does');
    assert.deepEqual(errors, []);
    await page.close();
  }
  console.log('PASS: desktop/phone mission layout, explicit acknowledgements, safe non-modal speech, keyboard/walking dismissal and no implicit writes');
} finally { await browser.close(); }
