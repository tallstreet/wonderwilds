// Real pointer/keyboard interaction and mocked transcription; no paid provider requests.
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
const require = createRequire(new URL('../../../qa/e2e/package.json', import.meta.url));
const { chromium } = require('playwright');
const origin = process.env.WW_ACTIVITY_TEST_URL ?? 'http://127.0.0.1:5173';
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH });
const base = {
  schemaVersion: '1.0',
  id: 'browser-practice',
  skillId: 'M01',
  contentVersion: 'draft',
  variantId: 'draft-1',
  contextId: 'picnic',
  representation: 'object_pairing',
  presentation: {
    title: 'Pack a picnic with Pip',
    objective: 'Give each friend a seed.',
    lessonText: 'One for each friend.',
    prompt: 'Give one seed to each friend.',
    context: 'Pip is ready for a picnic.',
  },
  support: { hintCount: 0, workedExample: false, readAloud: false },
};
const pairing = (layout) => ({
  ...base,
  task: {
    kind: 'pairing',
    layout,
    objects: [
      { id: 'seed-1', label: 'Seed one' },
      { id: 'seed-2', label: 'Seed two' },
    ],
    recipients: [
      { id: 'pip', label: 'Pip' },
      { id: 'fern', label: 'Fern' },
    ],
  },
});
const cards = (review = false) => ({
  ...base,
  skillId: review ? 'E13' : 'MY104',
  task: {
    kind: 'practice_cards',
    stimulus: review ? 'Invite Pip to a picnic.' : 'Pip has 3 shells. Fern brings 2 more.',
    choices: review
      ? []
      : [
          { id: 'a', label: '5' },
          { id: 'b', label: '4' },
          { id: 'c', label: '6' },
        ],
    adultReview: review,
  },
});
const errors = [];
try {
  for (const width of [1280, 390]) {
    const context = await browser.newContext({ viewport: { width, height: 844 }, hasTouch: true });
    const page = await context.newPage();
    page.on('pageerror', (error) => errors.push(error.message));
    await page.route('**/__mission_play_test', (route) =>
      route.fulfill({
        contentType: 'text/html',
        body: `<!doctype html><html lang="en"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Practice</title><style>body{font:18px system-ui;margin:20px}main{max-width:700px;margin:auto}button{font:inherit}button[hidden]{display:none}</style><main></main></html>`,
      }),
    );
    await page.goto(`${origin}/__mission_play_test`);
    await page.evaluate(async () => {
      const { mountMissionActivity } = await import('/src/mission-learning/activity.ts');
      window.mount = (activity, voice = false) => {
        window.component?.destroy();
        window.calls = [];
        window.cancelCount = 0;
        window.pending = false;
        const snapshot = { revision: 1, activeActivity: activity };
        window.component = mountMissionActivity(
          document.querySelector('main'),
          activity,
          snapshot,
          {
            hasPending: () => window.pending,
            onAttempt: async (answer) => {
              window.calls.push(answer);
              return snapshot;
            },
            onSupport: async (kind) => ({
              ...snapshot,
              activeActivity: {
                ...activity,
                revealedSupport: { kind, text: 'Try one at a time.' },
              },
            }),
            onAbandon: async () => {},
            onRetry: async () => snapshot,
            onRefresh: async () => snapshot,
            onContinue: () => {},
            onFinishTogether: async () => {
              window.calls.push('together');
              return {
                lastFeedback: {
                  result: 'assisted',
                  message: 'We explored together.',
                  completed: true,
                },
              };
            },
            recorder: voice
              ? {
                  start: async () => {},
                  finish: () =>
                    new Promise((resolve) => {
                      window.resolveTranscript = resolve;
                    }),
                  cancel: () => {
                    window.cancelCount++;
                  },
                }
              : undefined,
          },
        );
        window.component.focus();
      };
    });
    const mount = (activity, voice = false) =>
      page.evaluate(([a, v]) => window.mount(a, v), [activity, voice]);
    const drag = async (source, target) => {
      const a = await source.boundingBox(),
        b = await target.boundingBox();
      await page.mouse.move(a.x + a.width / 2, a.y + a.height / 2);
      await page.mouse.down();
      await page.mouse.move(b.x + b.width / 2, b.y + b.height / 2, { steps: 12 });
      await page.mouse.up();
    };
    const check = () => page.getByRole('button', { name: 'Check my idea', exact: true });
    for (const layout of ['object_cards', 'recipient_slots']) {
      await mount(pairing(layout));
      assert.equal(await page.locator('select').count(), 0);
      if (layout === 'recipient_slots') {
        assert.equal(
          await page
            .locator('.ww-mission-recipient-slots')
            .evaluate((node) => getComputedStyle(node).gridTemplateColumns.split(' ').length),
          2,
        );
      }
      assert.equal(await check().isDisabled(), true);
      await drag(
        page.locator('.ww-mission-piece').nth(0),
        page.locator('.ww-mission-pair-row').nth(0),
      );
      assert.equal(
        await page.locator('.ww-mission-pair-row').nth(0).textContent(),
        'Pip: Seed one',
      );
      await page.locator('.ww-mission-piece').nth(1).focus();
      await page.keyboard.press('Space');
      await page.locator('.ww-mission-pair-row').nth(1).focus();
      await page.keyboard.press('Enter');
      assert.equal(await page.evaluate(() => window.calls.length), 0, 'Dragging does not submit');
      await check().click();
      assert.deepEqual(await page.evaluate(() => window.calls[0]), {
        kind: 'pairing',
        assignments: [
          { objectId: 'seed-1', recipientId: 'pip' },
          { objectId: 'seed-2', recipientId: 'fern' },
        ],
      });
      // Move an already placed piece: its old slot becomes empty.
      await page.locator('.ww-mission-piece').nth(0).tap();
      await page.locator('.ww-mission-pair-row').nth(1).tap();
      assert.equal(await check().isDisabled(), true);
    }
    await mount(cards());
    await drag(page.locator('.ww-mission-choice').nth(0), page.locator('.ww-mission-answer-tray'));
    assert.equal(await page.locator('input[value="a"]').isChecked(), true);
    const unused = await page.locator('.ww-mission-choice').nth(1).boundingBox();
    await page.mouse.move(unused.x + 20, unused.y + 20);
    await page.mouse.down();
    await page.mouse.move(2, 2, { steps: 12 });
    await page.mouse.up();
    assert.equal(
      await page.locator('input[value="a"]').isChecked(),
      true,
      'An invalid drop preserves the answer',
    );
    await check().click();
    assert.deepEqual(await page.evaluate(() => window.calls), [{ kind: 'choice', choiceId: 'a' }]);
    await page.screenshot({ path: `/tmp/wonderwilds-practice-cards-${width}.png`, fullPage: true });
    assert.equal(
      await page.evaluate(() => document.documentElement.scrollWidth > innerWidth),
      false,
    );
    const touchSource = await page.locator('.ww-mission-choice').nth(1).boundingBox();
    const touchTarget = await page.locator('.ww-mission-answer-tray').boundingBox();
    const cdp = await context.newCDPSession(page);
    await cdp.send('Input.dispatchTouchEvent', {
      type: 'touchStart',
      touchPoints: [{ x: touchSource.x + 20, y: touchSource.y + 20 }],
    });
    await cdp.send('Input.dispatchTouchEvent', {
      type: 'touchMove',
      touchPoints: [{ x: touchTarget.x + 20, y: touchTarget.y + 20 }],
    });
    await cdp.send('Input.dispatchTouchEvent', { type: 'touchEnd', touchPoints: [] });
    assert.equal(
      await page.locator('input[value="b"]').isChecked(),
      true,
      'Touch dragging selects a new idea',
    );
    await mount(cards(), true);
    await page.getByRole('button', { name: 'Say my answer', exact: true }).click();
    assert.equal(await check().isDisabled(), true);
    await page.getByRole('button', { name: 'Finish recording', exact: true }).click();
    await page.evaluate(() => window.resolveTranscript('five'));
    await page.waitForFunction(
      () => document.querySelector('.ww-mission-voice input').value === 'five',
    );
    assert.equal(await page.evaluate(() => window.calls.length), 0, 'Transcription never submits');
    await page.getByRole('button', { name: 'Use this answer', exact: true }).click();
    assert.equal(await page.locator('input[value="a"]').isChecked(), true);
    await page.locator('.ww-mission-voice input').fill('something ambiguous');
    await page.getByRole('button', { name: 'Use this answer', exact: true }).click();
    assert.equal(
      await page.locator('input[value="a"]').isChecked(),
      true,
      'Unknown transcript preserves the answer',
    );
    await check().click();
    assert.deepEqual(await page.evaluate(() => window.calls), [{ kind: 'choice', choiceId: 'a' }]);
    await page.getByRole('button', { name: 'Say my answer', exact: true }).click();
    await page.getByRole('button', { name: 'Finish recording', exact: true }).click();
    await page.evaluate(() => {
      window.component.destroy();
      window.resolveTranscript('six');
    });
    assert.equal(await page.locator('.ww-mission-activity').count(), 0);
    assert.ok(await page.evaluate(() => window.cancelCount > 0));
    await mount(cards(true), true);
    assert.equal(await check().isVisible(), false, 'Writing is never automatically marked');
    await page.getByRole('button', { name: 'Say my answer', exact: true }).click();
    await page.getByRole('button', { name: 'Finish recording', exact: true }).click();
    await page.evaluate(() => window.resolveTranscript('Dear Pip, come to our picnic.'));
    await page.getByRole('button', { name: 'Use this answer', exact: true }).click();
    assert.equal(await page.locator('textarea').inputValue(), 'Dear Pip, come to our picnic.');
    assert.equal(await page.evaluate(() => window.calls.length), 0);
    await page.screenshot({
      path: `/tmp/wonderwilds-practice-workshop-${width}.png`,
      fullPage: true,
    });
    await page.getByRole('button', { name: 'Let’s do it together', exact: true }).click();
    assert.deepEqual(await page.evaluate(() => window.calls), ['together']);
    assert.equal(
      await page.getByRole('button', { name: 'Continue the story', exact: true }).isVisible(),
      true,
    );
    await mount({ ...cards(), skillId: 'E07' }, true);
    assert.equal(
      await page.getByRole('button', { name: 'Say my answer', exact: true }).count(),
      0,
      'Transcription cannot substitute for spelling',
    );
    const unsafe = cards();
    unsafe.task.stimulus = '<img src=x onerror="window.injected=true">';
    unsafe.task.choices[0].label = 'x'.repeat(120);
    await mount(unsafe);
    assert.equal(await page.locator('.ww-mission-stimulus img').count(), 0);
    assert.equal(await page.locator('.ww-mission-stimulus').textContent(), unsafe.task.stimulus);
    assert.equal(
      await page.evaluate(() => document.documentElement.scrollWidth > innerWidth),
      false,
      'Long untrusted labels wrap',
    );
    await context.close();
  }
  assert.deepEqual(errors, []);
  console.log(
    'Mission activity mouse/touch/keyboard, phone layout, voice review and cancellation passed.',
  );
} finally {
  await browser.close();
}
