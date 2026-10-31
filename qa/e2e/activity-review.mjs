// Standalone UI review: no backend, provider calls or saved learning progress.
import assert from 'node:assert/strict';
import { chromium } from 'playwright';

const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH });
const origin = process.env.WW_ACTIVITY_TEST_URL ?? 'http://127.0.0.1:5173';
try {
  for (const width of [1280, 390, 320]) {
    const context = await browser.newContext({ viewport: { width, height: 960 }, hasTouch: true });
    const page = await context.newPage();
    const errors = [];
    const writes = [];
    page.on('pageerror', (error) => errors.push(error.message));
    page.on('request', (request) => {
      if (request.method() !== 'GET') writes.push(request.url());
    });
    await page.goto(`${origin}/activity-review.html`);
    const current = () => page.locator('.sample-card:not([hidden])');
    const select = async (label) => {
      await page.getByRole('navigation', { name: 'Activity examples' }).getByRole('button', { name: label, exact: false }).click();
      assert.equal(await page.locator('.sample-card:not([hidden])').count(), 1);
      assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), true, `No overflow at ${width}: ${label}`);
    };
    const check = () => current().getByRole('button', { name: 'Check answer', exact: true });
    const feedback = () => current().locator('.sample-feedback');
    const enabled = async (value) => assert.equal(await check().isEnabled(), value);

    await enabled(false);
    await current().getByRole('radio', { name: '10', exact: true }).focus();
    await page.keyboard.press('Space');
    await enabled(true);
    assert.equal(await feedback().textContent(), '', 'Selection never submits');
    await check().click();
    assert.match(await feedback().textContent(), /received: 10/);
    await current().getByRole('radio', { name: '8', exact: true }).check();
    assert.equal(await feedback().textContent(), '');
    await current().getByRole('button', { name: 'A little hint' }).click();
    assert.equal(await current().locator('.hint-content').isVisible(), true);
    await page.screenshot({ path: `/tmp/wonderwilds-review-choice-${width}.png`, fullPage: true });

    await select('Choose a few');
    await enabled(false);
    await current().getByRole('checkbox', { name: 'Shape 1:', exact: false }).check();
    await current().getByRole('checkbox', { name: 'Shape 4:', exact: false }).check();
    await check().click();
    assert.match(await feedback().textContent(), /triangle.*rotated triangle/);
    await current().getByRole('checkbox', { name: 'Shape 1:', exact: false }).uncheck();
    await current().getByRole('checkbox', { name: 'Shape 4:', exact: false }).uncheck();
    await enabled(false);

    await select('Type a number');
    const number = current().getByLabel('Your number');
    await number.fill('x');
    await enabled(false);
    await number.fill('7');
    await number.press('Enter');
    assert.match(await feedback().textContent(), /received: 7/);

    await select('Type a word');
    await current().getByLabel('Your word').fill('hat');
    await check().click();
    assert.match(await feedback().textContent(), /received: hat/);
    await current().getByLabel('Your word').fill('<img src=x>');
    await check().click();
    assert.match(await feedback().textContent(), /<img src=x>/);
    assert.equal(await current().locator('.sample-feedback img').count(), 0);

    await select('Put in order');
    for (const [piece, place] of [['seed', 'First'], ['sprout', 'Next'], ['flower', 'Last']]) {
      await current().getByRole('button', { name: `Choose ${piece}`, exact: true }).focus();
      await page.keyboard.press('Space');
      await current().getByRole('button', { name: new RegExp(`^${place} ·`) }).press('Enter');
    }
    await check().click();
    assert.match(await feedback().textContent(), /Seed → Sprout → Flower/);
    await current().getByRole('button', { name: '↶ Undo' }).click();
    await enabled(false);
    await page.screenshot({ path: `/tmp/wonderwilds-review-order-${width}.png`, fullPage: true });

    await select('Match and move');
    for (const [piece, place] of [['berry', 'Pip'], ['leaf', 'Clover'], ['flower', 'Wren']]) {
      await current().getByRole('button', { name: `Choose ${piece}`, exact: true }).tap();
      await current().getByRole('button', { name: new RegExp(`^${place} ·`) }).tap();
    }
    await check().click();
    assert.match(await feedback().textContent(), /Berry for Pip, Leaf for Clover, Flower for Wren/);
    // Replacing an occupied place returns its previous object to the bank.
    await current().getByRole('button', { name: 'Choose berry', exact: true }).click();
    await current().getByRole('button', { name: /^Clover ·/ }).click();
    await enabled(false);
    assert.match(await current().getByRole('button', { name: /^Pip ·/ }).textContent(), /\+/);
    await current().getByRole('button', { name: '↶ Undo' }).click();
    await enabled(true);
    if (width === 1280) {
      const source = await current().getByRole('button', { name: 'Choose berry', exact: true }).boundingBox();
      const destination = await current().getByRole('button', { name: /^Wren ·/ }).boundingBox();
      await page.mouse.move(source.x + source.width / 2, source.y + source.height / 2);
      await page.mouse.down();
      await page.mouse.move(destination.x + destination.width / 2, destination.y + destination.height / 2, { steps: 12 });
      await page.mouse.up();
      assert.match(await current().getByRole('button', { name: /^Wren ·/ }).textContent(), /Berry/);
      assert.equal(await current().locator('.ww-drag-ghost').count(), 0);
    }

    await select('Make something');
    const share = current().getByRole('button', { name: 'Ready to share' });
    assert.equal(await share.isEnabled(), false);
    await current().getByLabel('Your invitation').fill('Dear Pip, meet me at the bridge for a picnic.');
    await share.click();
    assert.match(await feedback().textContent(), /not been automatically marked/);
    assert.equal(await current().getByRole('button', { name: 'Check answer', exact: true }).count(), 0);
    await select('Choose one');
    await select('Make something');
    assert.match(await current().getByLabel('Your invitation').inputValue(), /Dear Pip/);
    await page.screenshot({ path: `/tmp/wonderwilds-review-workshop-${width}.png`, fullPage: true });
    await page.getByRole('button', { name: '↶ Start again' }).click();
    assert.equal(await current().getByLabel('Your invitation').inputValue(), '');
    await page.getByRole('button', { name: 'Back to the first →' }).click();
    assert.equal(await current().getAttribute('data-control'), 'single');
    assert.deepEqual(errors, []);
    assert.deepEqual(writes, [], 'UI review never writes progress or calls providers');
    await context.close();
    console.log(`Seven activity controls passed at ${width}px`);
  }
} finally {
  await browser.close();
}
