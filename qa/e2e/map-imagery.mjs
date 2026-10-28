import { chromium } from 'playwright';
import assert from 'node:assert/strict';
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH || '/usr/bin/chromium', args: ['--no-sandbox'] });
try {
  const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
  const errors = [];
  page.on('pageerror', e => errors.push(e.message));
  await page.goto('http://localhost:5173/?guest=1');
  await page.locator('#explore').click();
  assert.equal(await page.locator('.place-card img').count(), 75);
  for (let biome = 0; biome < 12; biome++) {
    for (let variant = 0; variant < 6; variant++) {
      const response = await page.request.get(`http://localhost:5173/assets/places/explore-${biome}-${variant}.webp`);
      assert.equal(response.status(), 200);
      assert.match(response.headers()['content-type'], /image\/webp/);
    }
    const id = `explore-${biome}-0`;
    await page.locator(`[data-place="${id}"]`).click();
    await page.waitForFunction(() => [...document.images].every(i => !i.src.includes('/places/') || !i.complete || i.naturalWidth > 0));
    await page.waitForTimeout(150);
    await page.locator('#explore').click();
  }
  await page.locator('[data-place="explore-8-0"]').click();
  await page.waitForTimeout(600);
  await page.screenshot({ path: '/tmp/wonderwilds-map-imagery.png' });
  assert.deepEqual(errors, []);
  console.log('All 72 map images served; 75 thumbnails and travel through 12 themes passed.');
} finally { await browser.close(); }
