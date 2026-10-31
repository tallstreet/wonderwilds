// Synthetic sprites and sessions only; no paid provider calls or progress writes.
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { chromium } from 'playwright';
const fixtures = JSON.parse(await readFile(new URL('../../contracts/fixtures/cases.json', import.meta.url), 'utf8'));
const snapshot = fixtures.find(item => item.contract === 'StorySnapshot' && item.valid && item.value.scene).value;
const origin = process.env.WONDERWILDS_TEST_URL ?? 'http://127.0.0.1:5194';
const profile = '00000000-0000-4000-8000-000000000001';
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH });
try {
  for (const width of [1280, 390]) {
    const page = await browser.newPage({ viewport: { width, height: 844 } });
    const errors = [];
    page.on('pageerror', error => errors.push(error.message));
    let writes = 0, calls = 0, responseMode = 'ok', release;
    let png;
    await page.route('**/__demo/bootstrap', route => route.fulfill({ json: { profileId: profile, csrf: 'synthetic-csrf', characterGeneration: true } }));
    await page.route('**/api/v1/mission-learning**', route => route.fulfill({ status: 503, json: {} }));
    await page.route('**/api/v1/story**', route => {
      if (route.request().method() !== 'GET') writes++;
      return route.fulfill({ json: snapshot });
    });
    await page.route('**/__demo/character', async route => {
      calls++;
      assert.equal(route.request().headers()['x-csrf-token'], 'synthetic-csrf');
      assert.equal(route.request().headers()['x-wonderwilds-profile'], profile);
      assert.equal(route.request().postDataJSON().description, 'A blue fox');
      if (responseMode === 'hold') await new Promise(resolve => { release = resolve; });
      if (responseMode === 'fail') return route.fulfill({ status: 503 });
      return route.fulfill({ contentType: 'image/png', body: png });
    });
    await page.goto(origin);
    const button = page.getByRole('button', { name: 'Customize my character', exact: true });
    await page.waitForFunction(() => !document.querySelector('#character').disabled);
    assert.equal(await page.title(), 'Wonderwilds • A maths adventure');
    assert.equal(await page.locator('.location-badge small').textContent(), 'WONDERWILDS');
    assert.equal(await page.locator('.location-badge #character').count(), 1);
    assert.equal(await page.locator('.game-hud nav #character').count(), 0);
    const tile = await page.locator('.location-badge').boundingBox();
    const target = await button.boundingBox();
    assert.ok(target.width >= tile.width - 6 && target.height >= tile.height - 6, 'Whole title tile is clickable');
    const encoded = await page.evaluate(() => {
      const canvas = document.createElement('canvas'); canvas.width = 384; canvas.height = 512;
      const ctx = canvas.getContext('2d'); ctx.fillStyle = '#ff00ff'; ctx.fillRect(0, 0, 384, 512);
      for (let i = 0; i < 12; i++) {
        const x = i % 3 * 128, y = Math.floor(i / 3) * 128;
        ctx.fillStyle = '#527ac6'; ctx.fillRect(x + 40, y + 20, 48, 88);
        ctx.fillStyle = '#fee87b'; ctx.fillRect(x + 40, y + 60, 48, 8);
      }
      return canvas.toDataURL('image/png').split(',')[1];
    });
    png = Buffer.from(encoded, 'base64');
    assert.equal(await page.evaluate(async () => {
      const { prepareCharacter } = await import('/src/custom-character.ts');
      for (const colour of ['#ff00ff', '#222222']) {
        const canvas = document.createElement('canvas'); canvas.width = 384; canvas.height = 512;
        const ctx = canvas.getContext('2d'); ctx.fillStyle = colour; ctx.fillRect(0, 0, 384, 512);
        try { await prepareCharacter(canvas.toDataURL('image/png')); return false; } catch {}
      }
      return true;
    }), true, 'Empty frames and opaque backgrounds are rejected');
    await button.focus(); await page.keyboard.press('Enter');
    await page.getByLabel('Describe your character', { exact: true }).fill('A blue fox');
    await page.getByRole('button', { name: 'Generate character', exact: true }).click();
    await page.getByRole('button', { name: 'Use this character', exact: true }).waitFor();
    assert.equal(await page.evaluate(() => localStorage.getItem('wonderwilds.character.v1:00000000-0000-4000-8000-000000000001')), null, 'Preview does not apply/save');
    await page.getByRole('button', { name: 'Back', exact: true }).click();
    assert.equal(await page.getByRole('button', { name: 'Back', exact: true }).getAttribute('aria-pressed'), 'true');
    await page.emulateMedia({ reducedMotion: 'reduce' });
    assert.equal(await page.locator('.character-walk-preview').evaluate(el => getComputedStyle(el).animationName), 'none');
    await page.emulateMedia({ reducedMotion: 'no-preference' });
    const bounds = await page.locator('dialog').boundingBox();
    assert.ok(bounds.x >= 0 && bounds.x + bounds.width <= width);
    assert.equal(await page.locator('dialog').evaluate(el => el.scrollWidth <= el.clientWidth), true);
    await page.screenshot({ path: `/tmp/wonderwilds-character-${width}.png` });
    await page.getByRole('button', { name: 'Use this character', exact: true }).click();
    await page.waitForFunction(() => !document.querySelector('dialog').open);
    assert.equal(await button.evaluate(el => el === document.activeElement), true);
    const player = () => page.evaluate(() => {
      const data = document.querySelector('#game').dataset;
      return { x: Number(data.storyPlayerX), y: Number(data.storyPlayerY), texture: data.playerTexture, frame: Number(data.playerFrame) };
    });
    assert.equal((await player()).texture, 'custom-character-v1');
    const before = await player();
    await page.keyboard.down('ArrowRight'); await page.waitForTimeout(180);
    const moving = await player();
    await page.keyboard.up('ArrowRight');
    assert.ok(moving.x > before.x, 'Custom character can walk');
    assert.ok([7, 8].includes(moving.frame), 'Right-facing walking frames');
    await page.reload();
    await page.waitForFunction(() => !document.querySelector('#character').disabled);
    await page.waitForFunction(() => document.querySelector('#game').dataset.playerTexture === 'custom-character-v1');
    assert.equal((await player()).texture, 'custom-character-v1', 'Appearance restores on reload');
    await button.click();
    await page.getByRole('button', { name: 'Use original character', exact: true }).click();
    assert.notEqual((await player()).texture, 'custom-character-v1');
    await page.reload(); await page.waitForFunction(() => !document.querySelector('#character').disabled);
    await page.waitForFunction(() => document.querySelector('#game').dataset.playerTexture && document.querySelector('#game').dataset.playerTexture !== 'custom-character-v1');
    assert.notEqual((await player()).texture, 'custom-character-v1', 'Reset persists');
    const validPng = png;
    png = Buffer.from(await page.evaluate(() => {
      const canvas = document.createElement('canvas'); canvas.width = 384; canvas.height = 512;
      const ctx = canvas.getContext('2d'); ctx.fillStyle = '#222222'; ctx.fillRect(0, 0, 384, 512);
      return canvas.toDataURL('image/png').split(',')[1];
    }), 'base64');
    await button.click(); await page.getByLabel('Describe your character', { exact: true }).fill('A blue fox');
    await page.getByRole('button', { name: 'Generate character', exact: true }).click();
    await page.waitForFunction(() => document.querySelector('#character-debug').open);
    const report = JSON.parse(await page.locator('[data-character-report]').textContent());
    assert.equal(report.width, 384); assert.equal(report.height, 512);
    assert.equal(report.cells.length, 12);
    assert.match(report.cells[0].issues.join(' '), /background still fills/);
    assert.equal(await page.getByRole('button', { name: 'Use this character', exact: true }).isVisible(), false);
    assert.equal(await page.locator('[data-character-keyed]').isVisible(), true);
    assert.equal(await page.locator('[data-character-raw]').evaluate(img => img.naturalWidth), 384);
    assert.equal(await page.locator('dialog').evaluate(el => el.scrollWidth <= el.clientWidth), true);
    const originalBytes = await page.locator('[data-character-download]').evaluate(async link => Array.from(new Uint8Array(await (await fetch(link.href)).arrayBuffer())));
    assert.deepEqual(Buffer.from(originalBytes), png, 'Download preserves exact provider PNG');
    await page.screenshot({ path: `/tmp/wonderwilds-character-debug-${width}.png` });
    await page.keyboard.press('Escape'); await button.click();
    await page.getByText('Inspect returned image', { exact: true }).click();
    assert.deepEqual(JSON.parse(await page.locator('[data-character-report]').textContent()), report, 'Diagnostics survive closing/reopening without another provider call');
    await page.keyboard.press('Escape');
    png = validPng;
    responseMode = 'fail';
    await button.click(); await page.getByLabel('Describe your character', { exact: true }).fill('A blue fox');
    await page.getByRole('button', { name: 'Generate character', exact: true }).click();
    await page.getByText('Character generation is unavailable. Try again shortly.', { exact: true }).waitFor();
    await page.keyboard.press('Escape');
    responseMode = 'hold';
    await button.click(); await page.getByLabel('Describe your character', { exact: true }).fill('A blue fox');
    await page.getByRole('button', { name: 'Generate character', exact: true }).click();
    await page.waitForFunction(() => document.querySelector('#character-form button').disabled);
    while (!release) await page.waitForTimeout(20);
    await page.keyboard.press('Escape'); release(); await page.waitForTimeout(100);
    await button.click();
    assert.equal(await page.locator('#character-preview').isVisible(), false, 'Cancelled output never appears in next dialog');
    assert.equal(calls, 4);
    await page.keyboard.press('Escape');
    // Untrusted or unavailable saved art never blocks the original character.
    await page.evaluate(() => localStorage.setItem('wonderwilds.character.v1:00000000-0000-4000-8000-000000000001', 'data:image/png;base64,AAAA'));
    await page.reload(); await page.waitForFunction(() => !document.querySelector('#character').disabled);
    await page.route('**/__demo/bootstrap', route => route.fulfill({ json: { profileId: profile, csrf: 'synthetic-csrf', characterGeneration: false } }));
    await page.reload(); await page.waitForFunction(() => !document.querySelector('#character').disabled);
    await button.click();
    assert.equal(await page.getByRole('button', { name: 'Generate character', exact: true }).isDisabled(), true);
    assert.equal(writes, 0);
    assert.deepEqual(errors, []);
    await page.close();
  }
  console.log('PASS: character preview/apply/walk/reload/reset, phone layout, focus, provider failure and cancellation; no progress writes');
} finally { await browser.close(); }
