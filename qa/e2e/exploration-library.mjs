import { chromium } from 'playwright';
import assert from 'node:assert/strict';
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH || '/usr/bin/chromium', args: ['--no-sandbox'] });
try {
  const page = await browser.newPage();
  const errors = [];
  page.on('pageerror', error => errors.push(error.message));
  await page.goto('http://localhost:5173/?guest=1');
  await page.locator('#explore').click();
  await page.waitForSelector('.place-card');
  assert.equal(await page.locator('.place-card').count(), 75);
  await page.locator('#place-search').fill('Crystal Caverns');
  assert.equal(await page.locator('.place-card').count(), 6);
  await page.locator('[data-place="explore-5-0"]').click();
  await page.waitForTimeout(600);
  assert.equal(await page.locator('h1').textContent(), 'Amethyst Grotto');
  await page.reload();
  await page.waitForFunction(() => document.querySelector('h1')?.textContent === 'Amethyst Grotto');
  await page.locator('#explore').click();
  for (let biome = 0; biome < 12; biome++) {
    await page.locator(`[data-place="explore-${biome}-3"]`).click();
    await page.waitForTimeout(80);
    await page.locator('#explore').click();
  }
  assert.deepEqual(errors, []);
  console.log('75 cards, biome search, travel, reload, and all 12 scenery themes passed.');
} finally { await browser.close(); }
