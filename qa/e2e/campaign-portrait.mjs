// Real campaign dialogue renderer with a deliberately delayed portrait response.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { chromium } from 'playwright';

const origin = process.env.WW_DEMO_TEST_URL ?? 'http://127.0.0.1:5173';
const runtime = JSON.parse(readFileSync(new URL('../../content/campaign/runtime.v1.json', import.meta.url), 'utf8'));
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH });
try {
  const page = await browser.newPage();
  const errors = [];
  page.on('pageerror', error => errors.push(error.message));
  await page.route('**/__portrait_test', route => route.fulfill({
    contentType: 'text/html',
    body: '<!doctype html><html lang="en"><title>Campaign portrait</title><dialog id="dialog"></dialog></html>',
  }));
  let release;
  const held = new Promise(resolve => { release = resolve; });
  let sheetRequests = 0;
  const request = page.waitForRequest('**/assets/characters/meadow.png');
  await page.route('**/assets/characters/meadow.png', async route => {
    sheetRequests++;
    await held;
    await route.continue();
  });
  await page.goto(`${origin}/__portrait_test`);
  await page.evaluate(async runtime => {
    const { CampaignWorld } = await import('/src/campaign/world.ts');
    const world = Object.create(CampaignWorld.prototype);
    Object.defineProperty(world, 'campaign', { value: {
      legal_actions: runtime.actions,
      state: { flags: [] },
    } });
    world.learning = { snapshot: { learnerAge: 8 } };
    world.hooks = { open(html) {
      const dialog = document.querySelector('dialog');
      dialog.innerHTML = html;
      dialog.showModal();
    } };
    window.show = (id = 'practice_pip') => world.showAction(id);
    window.showSpeaker = speaker => {
      world.campaign.legal_actions.find(action => action.id === 'practice_pip').dialogue[0].speaker = speaker;
      window.show();
    };
    window.show();
  }, runtime);
  await request;
  assert.match(await page.getByRole('status').first().innerText(), /Loading Pip.*portrait/,
    'A delayed sheet must show loading feedback, not an empty portrait');
  assert.equal(await page.getByRole('img', { name: 'Pip', exact: true }).count(), 0);
  release();
  await page.getByRole('img', { name: 'Pip', exact: true }).waitFor();
  const portrait = await page.getByRole('img', { name: 'Pip', exact: true }).screenshot();
  const colors = await page.evaluate(async data => {
    const image = new Image();
    image.src = `data:image/png;base64,${data}`;
    await image.decode();
    const canvas = document.createElement('canvas');
    canvas.width = image.width;
    canvas.height = image.height;
    const context = canvas.getContext('2d');
    context.drawImage(image, 0, 0);
    const pixels = context.getImageData(0, 0, image.width, image.height).data;
    const colors = new Set();
    for (let i = 0; i < pixels.length; i += 4) colors.add(`${pixels[i]},${pixels[i + 1]},${pixels[i + 2]}`);
    return colors.size;
  }, portrait.toString('base64'));
  assert.ok(colors > 100, 'The decoded portrait must actually paint, not just reserve space');
  assert.equal(await page.locator('.campaign-portrait img').evaluate(image => image.complete && image.naturalWidth > 0), true);
  assert.equal(sheetRequests, 1, 'The decoded image is reused without a second CSS-background fetch');
  await page.evaluate(() => window.show());
  await page.getByRole('img', { name: 'Pip', exact: true }).waitFor();
  await page.evaluate(() => window.showSpeaker('Unknown'));
  assert.equal(await page.locator('#campaign-dialogue-portrait').innerText(), '');
  assert.match(await page.locator('.campaign-dialogue').innerText(), /Unknown/);
  await page.route('**/assets/campaign/cast-v1.png', route => route.abort());
  await page.evaluate(() => window.showSpeaker('Hazel'));
  await page.getByText('Hazel’s portrait is unavailable.', { exact: true }).waitFor();
  assert.match(await page.locator('.campaign-dialogue').innerText(), /Pip tips a practice lantern/);
  assert.deepEqual(errors, []);
  console.log('Campaign portrait: delayed, cached, unknown-speaker and failed-image cases passed.');
} finally {
  await browser.close();
}
