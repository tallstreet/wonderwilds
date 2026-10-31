// Real disposable API/database, mocked artwork provider response. No paid calls.
import assert from 'node:assert/strict';
import { chromium } from 'playwright';
const origin = process.env.WW_DEMO_TEST_URL ?? 'http://127.0.0.1:5180';
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH });
try {
  for (const [index, width] of [1280, 390].entries()) {
    const page = await browser.newPage({ viewport: { width, height: 844 } });
    const errors = [];
    page.on('pageerror', error => errors.push(error.message));
    // Enable only the cosmetic bridge in the browser; its responses are mocked.
    await page.route('**/__demo/bootstrap', async route => {
      const response = await route.fetch();
      await route.fulfill({ json: { ...await response.json(), characterGeneration: true } });
    });
    const artRequests = [];
    await page.route('**/__demo/item', async route => {
      artRequests.push(route.request().postDataJSON().itemId);
      const image = await page.evaluate(() => {
        const canvas = document.createElement('canvas'); canvas.width = canvas.height = 256;
        const c = canvas.getContext('2d');
        c.fillStyle = '#ff00ff'; c.fillRect(0, 0, 256, 256);
        c.fillStyle = '#39a987'; c.beginPath(); c.arc(128, 128, 65, 0, 2 * Math.PI); c.fill();
        return canvas.toDataURL('image/png').split(',')[1];
      });
      await route.fulfill({ contentType: 'image/png', body: Buffer.from(image, 'base64') });
    });
    await page.goto(origin);
    await page.locator('#game canvas').waitFor();
    await page.waitForFunction(() => !document.querySelector('#bag').disabled);
    const source = index ? 'find_ribbon' : 'find_clover';
    const id = index ? 'ribbon' : 'lucky_clover';
    const label = index ? 'golden ribbon' : 'lucky clover';
    if (index) {
      await page.evaluate(async () => {
        const b = await fetch('/__demo/bootstrap').then(r => r.json());
        const s = await fetch('/api/v1/story', { headers: { 'X-Wonderwilds-Profile': b.profileId } }).then(r => r.json());
        const a = s.sceneInput.availableActions.find(a => a.action.type === 'travel' && a.action.destinationId === 'workshop');
        const r = await fetch('/api/v1/story/commands', { method: 'POST', headers: { 'Content-Type': 'application/json', 'X-Wonderwilds-Profile': b.profileId, 'X-CSRF-Token': b.csrf }, body: JSON.stringify({ schemaVersion: '1.0', commandId: crypto.randomUUID(), expectedRevision: s.revision, payload: { type: 'choose', actionId: a.id } }) });
        if (!r.ok) throw new Error(`Travel failed: ${r.status}`);
      });
      await page.reload();
      await page.waitForFunction(() => !document.querySelector('#bag').disabled);
    }
    await page.locator('#missions').click();
    await page.locator(`[data-story-visit="resource:${source}"]`).click();
    await page.getByRole('heading', { name: `Pick up ${label}`, exact: true }).waitFor({ timeout: 20000 });
    await page.locator('[data-story-handle][data-story-action="collect"]').click();
    await page.waitForFunction(() => !document.querySelector('dialog[open]'));
    await page.locator('#bag').click();
    const row = page.locator('.story-bag-items li').filter({ hasText: label });
    await row.waitFor();
    assert.equal(await row.locator('.story-bag-quantity').textContent(), '× 1');
    await row.locator('img.item-art').waitFor({ timeout: 20000 });
    assert.ok(artRequests.includes(id));
    await page.screenshot({ path: `/tmp/wonderwilds-items-${width}.png` });
    await page.reload();
    await page.waitForFunction(() => !document.querySelector('#bag').disabled);
    await page.locator('#bag').click();
    await page.locator('.story-bag-items li').filter({ hasText: label }).locator('img.item-art').waitFor();
    await page.keyboard.press('Escape');
    await page.locator('#missions').click();
    assert.equal(await page.locator(`[data-story-visit="resource:${source}"]`).count(), 0);
    assert.deepEqual(errors, []);
    await page.close();
  }
  console.log('PASS: real keepsake pickups, generated icons, persisted bag quantities/art, depletion, desktop and phone');
} finally { await browser.close(); }
