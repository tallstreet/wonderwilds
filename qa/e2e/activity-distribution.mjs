// Synthetic API: checks cross-map mission travel without touching demo progress.
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { chromium } from 'playwright';
const story = JSON.parse(await readFile(new URL('../../content/story/adventure.v1.json', import.meta.url)));
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH });
try {
  for (const [id, name] of [['befriend-pip', 'Starlight Cove'], ['befriend-tansy', 'Peachy Orchard'], ['befriend-nib', 'Fern Hollow'], ['befriend-flutter', 'Mossy Clearing']]) {
    const page = await browser.newPage();
    const errors = [];
    page.on('pageerror', error => errors.push(error.message));
    await page.route('**/__demo/bootstrap', r => r.fulfill({ json: { profileId: 'synthetic-profile', csrf: 'synthetic-csrf' } }));
    await page.route('**/api/v1/game', r => r.fulfill({ json: { schemaVersion: '1.0', revision: 0, contentVersion: 'test', rulesVersion: 'test', storyFlags: story.beats.filter(b => b.chapterId === 'meadow').flatMap(b => b.completionFlags), ownedCreatures: [], inventory: [], pendingEncounters: [], learnerSummaries: [] } }));
    await page.route('**/api/v1/commands', r => r.abort());
    await page.goto(process.env.WONDERWILDS_TEST_URL || 'http://127.0.0.1:5173/');
    await page.locator('#game canvas').waitFor();
    await page.locator('#explore').click();
    assert.match(await page.locator('[data-place="meadow"]').innerText(), /Activities: 2/);
    assert.match(await page.locator('[data-place="explore-0-2"]').innerText(), /Activities: 1/);
    await page.getByRole('button', { name: 'Close dialog' }).click();
    await page.locator('#missions').click();
    await page.locator(`[data-server-visit="${id}"]`).click();
    assert.equal(await page.locator('h1').textContent(), 'Clover Meadow', 'Tracking does not teleport');
    for (let step = 0; (await page.locator('h1').textContent()) !== name; step++) {
      assert.ok(step < 5, 'Route must follow adjacent exits without looping');
      const before = await page.locator('h1').textContent();
      await page.locator('#quest-guide').click();
      await page.waitForFunction(previous => document.querySelector('h1').textContent !== previous, before);
    }
    await page.locator('#quest-guide').click();
    await page.locator('#encounter-help').waitFor();
    const request = page.waitForRequest(r => r.url().endsWith('/api/v1/commands'));
    await page.locator('#encounter-help').click();
    assert.equal((await request).postDataJSON().payload.beatId, id);
    assert.equal(await page.locator('h1').textContent(), name);
    assert.deepEqual(errors, []);
    await page.close();
    console.log(`${id}: mission travels to ${name} and walks to its encounter`);
  }
} finally { await browser.close(); }
