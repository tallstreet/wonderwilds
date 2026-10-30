import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { chromium } from 'playwright';

const fixtures = JSON.parse(await readFile(new URL('../../contracts/fixtures/cases.json', import.meta.url), 'utf8'));
const snapshot = fixtures.find(item => item.contract === 'StorySnapshot' && item.valid && item.value.scene).value;
const origin = process.env.WONDERWILDS_TEST_URL ?? 'http://127.0.0.1:5173';
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH });
try {
  for (const path of ['/', '/?guest=1', '/?story=1', '/story.html', '/demo.html']) {
    for (const width of [1280, 390]) {
      const page = await browser.newPage({ viewport: { width, height: 844 } });
      const errors = [];
      page.on('pageerror', error => errors.push(error.message));
      let source = 'authored';
      await page.route('**/__demo/bootstrap', route => route.fulfill({ json: {
        profileId: '00000000-0000-4000-8000-000000000001', csrf: 'synthetic-token-for-browser-testing-only',
      } }));
      await page.route('**/api/v1/mission-learning**', route => route.fulfill({ status: 503, json: {} }));
      await page.route('**/api/v1/story', route => route.fulfill({ json: { ...snapshot, sceneSource: source } }));
      await page.goto(origin + path);
      await page.locator('#game canvas').waitFor();
      assert.equal(await page.locator('#story-adventure-link').count(), 0);
      const status = page.locator('#story-source-status');
      for (const [value, label] of [
        ['authored', 'Preauthored story'],
        ['llm', 'LLM-generated dialogue'],
        ['authored_fallback', 'Preauthored story · LLM fallback'],
        [undefined, 'Story source unavailable · refresh to check'],
      ]) {
        source = value;
        if (value !== 'authored') {
          await page.locator('#missions').click();
          await page.getByRole('button', { name: 'Refresh progress', exact: true }).click();
        }
        await page.waitForFunction(text => document.querySelector('#story-source-status')?.textContent === text, label);
        assert.equal(await status.isVisible(), true);
        const bounds = await status.boundingBox();
        assert.ok(bounds.x >= 0 && bounds.x + bounds.width <= width);
      }
      await page.screenshot({ path: `/tmp/story-source-${path.startsWith('/?') ? 'world' : 'inspector'}-${width}.png` });
      assert.deepEqual(errors, []);
      await page.close();
    }
  }
  console.log('PASS: one story world serves the root and all retired mode URLs; source statuses update at desktop and phone sizes');
} finally {
  await browser.close();
}
