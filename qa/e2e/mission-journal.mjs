// Read-only presentation regression; synthetic snapshots, no demo data or provider calls.
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { chromium } from 'playwright';
const fixtures = JSON.parse(await readFile(new URL('../../contracts/fixtures/cases.json', import.meta.url), 'utf8'));
const base = fixtures.find(item => item.contract === 'StorySnapshot' && item.valid && item.value.scene).value;
const origin = process.env.WONDERWILDS_TEST_URL ?? 'http://127.0.0.1:5189';
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH });
try {
  for (const width of [1280, 390]) {
    const page = await browser.newPage({ viewport: { width, height: 844 } });
    const errors = [];
    page.on('pageerror', error => errors.push(error.message));
    const snapshot = structuredClone(base);
    snapshot.sceneInput.mission.status = 'completed';
    snapshot.sceneInput.neighbourMissions = [{
      characterId: 'marina', locationId: 'cove', title: 'The travelling tune & echoes',
      status: 'open', inspected: false, task: 'Listen in Fern Hollow', payoff: 'A shared tune', stepIndex: 0,
      generatedStory: {
        name: 'Wren', personality: 'Curious', sprite: 'patter', greeting: 'Hello',
        title: 'The travelling tune & echoes', premise: 'Follow a tune through the trees.', payoff: 'A shared tune',
        steps: [{ label: 'Listen in Fern Hollow', shape: 'instrument', locationId: 'fern_hollow', discovery: 'A hollow stem makes music.' }],
      },
    }];
    let writes = 0;
    await page.route('**/__demo/bootstrap', route => route.fulfill({ json: {
      profileId: '00000000-0000-4000-8000-000000000001', csrf: 'synthetic-token-for-browser-testing-only',
    } }));
    await page.route('**/api/v1/mission-learning**', route => route.fulfill({ status: 503, json: {} }));
    await page.route('**/api/v1/story**', route => {
      if (route.request().method() !== 'GET') writes++;
      return route.fulfill({ json: snapshot });
    });
    const open = async () => {
      await page.goto(origin);
      await page.waitForFunction(() => document.querySelector('#game').dataset.storyRevision === '3', { timeout: 15000 });
      await page.locator('#missions').click();
    };
    await open();
    assert.equal(await page.locator('#dialog-title').textContent(), 'The travelling tune & echoes');
    assert.match(await page.locator('[data-story-mission-status="open"]').textContent(), /Listen in Fern Hollow.*Fern Hollow/);
    assert.equal(await page.locator('[data-story-pip-summary]').getAttribute('open'), null);
    assert.equal(await page.locator('[data-story-mission-status="completed"]').isVisible(), false);
    assert.equal(await page.getByText('A hollow stem makes music.', { exact: true }).count(), 0);
    assert.equal(await page.locator('#dialog').evaluate(dialog => dialog.scrollTop), 0);
    assert.equal(await page.locator('#dialog-title').evaluate(title => title === document.activeElement), true);
    const titleBounds = await page.locator('#dialog-title').boundingBox();
    assert.ok(titleBounds.y >= 0 && titleBounds.y + titleBounds.height <= 844);
    await page.screenshot({ path: `/tmp/wonderwilds-current-mission-${width}.png` });
    await page.locator('[data-story-pip-summary] summary').click();
    assert.equal(await page.locator('[data-story-mission-status="completed"]').isVisible(), true);
    snapshot.sceneInput.neighbourMissions[0].inspected = true;
    snapshot.sceneInput.neighbourMissions[0].stepIndex = 1;
    await open();
    assert.match(await page.locator('[data-story-mission-status="open"]').textContent(), /Return to Wren.*Starlight Cove/);
    assert.match(await page.locator('#dialog > .story-checklist').textContent(), /Listen in Fern Hollow/);
    snapshot.sceneInput.mission.status = 'active';
    await open();
    assert.equal(await page.locator('#dialog-title').textContent(), base.sceneInput.mission.goal);
    assert.equal(await page.locator('[data-story-pip-summary]').count(), 0);
    assert.equal(writes, 0);
    assert.deepEqual(errors, []);
    await page.close();
  }
  console.log('PASS: desktop/phone current mission heading, objective, return destination, collapsed Pip history, reload and active Pip priority');
} finally { await browser.close(); }
