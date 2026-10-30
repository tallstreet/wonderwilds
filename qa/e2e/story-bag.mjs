// Presentation fixtures only: no demo database or provider calls.
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
    let writes = 0;
    await page.route('**/__demo/bootstrap', route => route.fulfill({ json: {
      profileId: '00000000-0000-4000-8000-000000000001', csrf: 'synthetic-token-for-browser-testing-only',
    } }));
    await page.route('**/api/v1/mission-learning**', route => route.fulfill({ status: 503, json: {} }));
    await page.route('**/api/v1/story**', route => {
      if (route.request().method() !== 'GET') writes++;
      return route.fulfill({ json: snapshot });
    });
    const bag = page.getByRole('button', { name: 'Open my bag', exact: true });
    for (const inventory of [[], [{ itemId: 'wood', quantity: 3 }, { itemId: 'glow_pebbles', quantity: 2 }], [{ itemId: 'planks', quantity: 1 }]]) {
      snapshot.sceneInput.context.inventory = inventory;
      await page.goto(origin);
      await page.locator('#game canvas').waitFor();
      await page.waitForFunction(() => !document.querySelector('#bag').disabled);
      assert.equal(await page.locator('#explore, #journal, #grownup').count(), 0);
      await bag.focus();
      await page.keyboard.press('Enter');
      await page.getByRole('heading', { name: 'My bag', exact: true }).waitFor();
      assert.equal(await page.locator('.story-bag-items li').count(), inventory.length);
      for (const [index, item] of inventory.entries()) {
        const row = page.locator('.story-bag-items li').nth(index);
        assert.equal(await row.locator('strong').textContent(), item.itemId.replaceAll('_', ' '));
        assert.equal(await row.locator('.story-bag-quantity').textContent(), `× ${item.quantity}`);
      }
      assert.equal(await page.getByText('Your bag is empty.', { exact: true }).count(), inventory.length ? 0 : 1);
      const bounds = await page.locator('dialog').boundingBox();
      assert.ok(bounds.x >= 0 && bounds.x + bounds.width <= width);
      assert.ok(await page.evaluate(() => document.querySelector('dialog').scrollWidth <= document.querySelector('dialog').clientWidth));
      await page.screenshot({ path: `/tmp/wonderwilds-bag-${width}-${inventory.length}.png` });
      await page.keyboard.press('Escape');
      assert.equal(await bag.evaluate(element => element === document.activeElement), true);
      await bag.click();
      await page.getByRole('button', { name: 'Close dialog', exact: true }).click();
      await page.locator('#missions').click();
      await page.locator('[data-story-missions-panel]').waitFor();
      assert.equal(await page.locator('#story-world-inventory').count(), 0);
      assert.equal(await page.locator('dialog').innerText().then(text => text.includes('Carrying:')), false);
      await page.keyboard.press('Escape');
    }
    assert.equal(writes, 0, 'Opening the bag never changes progress');
    assert.deepEqual(errors, []);
    await page.close();
  }
  console.log('PASS: desktop/phone bag, saved quantities after reload, empty state, keyboard/focus, Missions and no progress writes');
} finally { await browser.close(); }
