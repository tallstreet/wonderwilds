import assert from 'node:assert/strict';
import { chromium } from 'playwright';

const browser = await chromium.launch({
  executablePath: process.env.CHROMIUM_PATH || '/usr/bin/chromium',
  headless: true,
  args: ['--no-sandbox'],
});
try {
  for (const [name, width, height] of [['desktop', 1280, 800], ['tablet', 768, 1024]]) {
    const page = await browser.newPage({ viewport: { width, height } });
    const errors = [];
    page.on('pageerror', error => errors.push(error.message));
    const guestUrl = new URL(process.env.WONDERWILDS_TEST_URL || 'http://127.0.0.1:5173/');
    guestUrl.searchParams.set('guest', '1');
    const response = await page.goto(guestUrl.href);
    assert.equal(response.status(), 200);
    await page.waitForSelector('#game canvas');
    await page.locator('#missions').click();
    await page.waitForSelector('dialog[open] .quest');
    assert.equal(await page.locator('.quest').count(), 3);
    await page.getByRole('button', { name: 'Close dialog' }).click();
    await page.getByRole('button', { name: 'My journal' }).click();
    await page.waitForSelector('#companion-name');
    await page.getByRole('button', { name: 'Close dialog' }).click();
    await page.getByRole('button', { name: 'For grown-ups' }).click();
    await page.waitForSelector('#export');
    assert.match(await page.locator('.evidence-note').innerText(), /not a mastery score/);
    assert.deepEqual(errors, []);
    await page.close();
    console.log(`${name}: rendered canvas, three missions, journal, parent export and honest evidence notice passed`);
  }
} finally {
  await browser.close();
}
