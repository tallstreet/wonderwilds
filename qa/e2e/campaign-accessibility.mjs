import { finishLearningSetup } from './campaign-ui.mjs';
// Technical M6 checks against the real saved campaign. Not a human accessibility audit.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { chromium } from 'playwright';
const origin = process.env.WW_DEMO_TEST_URL ?? 'http://127.0.0.1:5180';
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH ?? '/usr/bin/chromium' });
try {
  for (const mode of ['unavailable', 'service_error', 'throws', 'live_reduction']) {
    const page = await browser.newPage({ viewport: { width: 1280, height: 800 }, reducedMotion: mode === 'live_reduction' ? 'no-preference' : 'reduce' });
    const errors = [];
    page.on('pageerror', error => errors.push(error.message));
    await page.addInitScript(mode => {
      window.speechCalls = [];
      Object.defineProperty(window, 'speechSynthesis', { configurable: true, value: ['unavailable', 'live_reduction'].includes(mode) ? undefined : {
        cancel() { window.speechCalls.push('cancel'); },
        speak(utterance) {
          window.speechCalls.push('speak');
          if (mode === 'throws') throw new DOMException('Synthetic speech permission rejection', 'NotAllowedError');
          queueMicrotask(() => utterance.onerror?.(new Event('error')));
        },
      } });
    }, mode);
    await page.goto(origin);
    await page.locator('#campaign-start, #campaign-missions, #campaign-age').first().waitFor();
    if (await page.locator('#campaign-start').count()) await page.locator('#campaign-start').click();
    await finishLearningSetup(page);
    await page.locator('#campaign-missions').waitFor();
    if (await page.getByRole('button', { name: 'Choose Pip', exact: true }).count()) {
      await page.getByRole('button', { name: 'Choose Pip', exact: true }).click();
      await page.locator('[data-campaign-command="welcome_meet"]').waitFor();
    }
    await page.getByRole('button', { name: 'Close dialog', exact: true }).click();
    await page.locator('#missions').focus();
    await page.keyboard.press('Enter');
    await page.locator('[data-campaign-command="welcome_meet"]').focus();
    await page.keyboard.press('Enter');
    assert.equal(await page.evaluate(() => document.activeElement?.id), 'dialog-title', 'Dialogue heading receives focus');
    assert.ok((await page.locator('.campaign-dialogue p').innerText()).length > 10, 'Essential instruction available as text');
    const controlChecks = await page.locator('.campaign-actions button:enabled').evaluateAll(buttons => buttons.map(button => {
      const style = getComputedStyle(button), bounds = button.getBoundingClientRect();
      const luminance = color => {
        const values = color.match(/[\d.]+/g).slice(0, 3).map(Number).map(value => { value /= 255; return value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4; });
        return values[0] * 0.2126 + values[1] * 0.7152 + values[2] * 0.0722;
      };
      const a = luminance(style.color), b = luminance(style.backgroundColor);
      return { name: button.textContent, contrast: (Math.max(a,b) + 0.05) / (Math.min(a,b) + 0.05), width: bounds.width, height: bounds.height };
    }));
    assert.ok(controlChecks.length > 0, 'Audited visible actionable dialogue controls');
    for (const control of controlChecks) {
      assert.ok(control.contrast >= 4.5, `${control.name} text contrast`);
      assert.ok(control.width >= 24 && control.height >= 24, `${control.name} target size`);
    }
    if (['unavailable', 'live_reduction'].includes(mode)) {
      assert.equal(await page.locator('#campaign-read').count(), 0);
      await page.getByText('Read aloud is unavailable in this browser. All dialogue is shown as text.').waitFor();
    } else {
      const speaker = page.getByRole('button', { name: 'Read this aloud', exact: true });
      assert.equal((await speaker.innerText()).trim(), '', 'Read-aloud is an icon with an accessible name');
      const speakerBounds = await speaker.boundingBox();
      assert.ok(speakerBounds.width >= 44 && speakerBounds.width <= 48 && speakerBounds.height >= 44, 'Compact speaker keeps a touch-sized target');
      assert.equal(await page.locator('.campaign-dialogue-text #campaign-read').count(), 1, 'Speaker sits beside the dialogue text');
      await speaker.click();
      await page.locator('#campaign-read-status').filter({ hasText: 'Read aloud is unavailable' }).waitFor();
      assert.ok((await page.locator('.campaign-dialogue p').innerText()).length > 10);
    }
    const next = page.locator('[data-campaign-page]').filter({ hasText: 'Next' });
    if (await next.count()) {
      await next.focus(); await page.keyboard.press('Enter');
      assert.equal(await page.evaluate(() => document.activeElement?.id), 'dialog-title', 'Next page restores heading focus');
      const background = locator => locator.evaluate(node => getComputedStyle(node).backgroundColor);
      const primary = page.locator('.campaign-actions .campaign-primary');
      assert.equal(await primary.count(), 1, 'Exactly one primary dialogue action');
      const previous = page.getByRole('button', { name: 'Previous', exact: true });
      const back = page.getByRole('button', { name: 'Back to adventure', exact: true });
      assert.notEqual(await background(primary), await background(previous), 'Primary action stands out from Previous');
      assert.equal(await background(previous), await background(back), 'Back and Previous share secondary styling');
      if (mode === 'service_error') {
        await page.screenshot({ path: '/tmp/wonderwilds-dialogue-buttons-desktop.png', fullPage: true });
        await page.setViewportSize({ width: 390, height: 844 });
        assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth));
        await page.screenshot({ path: '/tmp/wonderwilds-dialogue-buttons-phone.png', fullPage: true });
        await page.setViewportSize({ width: 1280, height: 800 });
      }
    }
    await page.keyboard.press('Escape');
    await page.locator('dialog[open]').waitFor({ state: 'hidden' });
    assert.equal(await page.evaluate(() => document.activeElement?.id), 'missions', 'Escape restores launcher focus');
    if (!['unavailable', 'live_reduction'].includes(mode)) assert.ok((await page.evaluate(() => window.speechCalls)).filter(x => x === 'cancel').length >= 2, 'Page change and dismissal cancel narration');
    if (mode === 'live_reduction') await page.emulateMedia({ reducedMotion: 'reduce' });
    assert.equal(await page.evaluate(() => matchMedia('(prefers-reduced-motion: reduce)').matches), true);
    if (['unavailable', 'live_reduction'].includes(mode)) {
      await page.waitForFunction(() => !document.querySelector('#toast')?.classList.contains('show'));
      await page.waitForTimeout(500);
      const first = await page.locator('#game canvas').screenshot();
      await page.waitForTimeout(400);
      const second = await page.locator('#game canvas').screenshot();
      assert.equal(createHash('sha256').update(first).digest('hex'), createHash('sha256').update(second).digest('hex'), 'Reduced-motion idle world is visually static');
      await page.screenshot({ path: `/tmp/campaign-accessibility-${mode}.png` });
      const startX = Number(await page.locator('#game').getAttribute('data-campaign-player-x'));
      await page.keyboard.down('ArrowRight'); await page.waitForTimeout(400); await page.keyboard.up('ArrowRight');
      assert.ok(Number(await page.locator('#game').getAttribute('data-campaign-player-x')) > startX + 10, 'Reduced-motion world still supports keyboard movement');
    }
    assert.deepEqual(errors, [], `No uncaught errors with speech ${mode}`);
    console.log(`PASS M6 ${mode}: text fallback, keyboard dialogue, focus restoration and narration cancellation`);
    await page.close();
  }
} finally { await browser.close(); }
