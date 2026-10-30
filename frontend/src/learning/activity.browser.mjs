import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
const require = createRequire(new URL('../../../qa/e2e/package.json', import.meta.url));
const { chromium } = require('playwright');
const browser = await chromium.launch({
  executablePath: process.env.CHROMIUM_PATH ?? '/usr/bin/chromium',
  headless: true,
  args: ['--no-sandbox', '--use-fake-device-for-media-stream', '--use-fake-ui-for-media-stream'],
});
const page = await browser.newPage({
  viewport: { width: 768, height: 1024 },
  reducedMotion: 'reduce',
});
const errors = [];
page.on('pageerror', (error) => errors.push(error.message));
try {
  const testUrl = process.env.WW_ACTIVITY_TEST_URL ?? 'http://127.0.0.1:5173/';
  await page.route(testUrl, (route) =>
    route.fulfill({
      contentType: 'text/html',
      body: '<!doctype html><html lang="en"><head><title>Activity test</title></head><body></body></html>',
    }),
  );
  await page.goto(testUrl);
  await page.evaluate(async () => {
    const { mountActivity } = await import('/src/learning/activity.ts');
    const { authoredPresentation } = await import('/src/learning/presentation.ts');
    window.presentationTest = authoredPresentation;
    window.mountTest = (
      templateId,
      parameters,
      presentation = {},
      skillId = 'M06',
      permittedActions = ['attempt', 'hint', 'worked_example', 'abandon'],
    ) => {
      window.activity?.destroy();
      document.body.replaceChildren();
      window.calls = [];
      window.rejectSupport = false;
      window.delayAttempt = false;
      window.delaySupport = false;
      window.activity = mountActivity(
        document.body,
        {
          schemaVersion: '1.0',
          id: 'test',
          templateId,
          parameters,
          skillId,
          contentVersion: 'draft',
          permittedActions,
          support: { hintCount: 0, workedExample: false, readAloud: false },
        },
        {
          recorder: {
            start: async () => {
              window.calls.push(['record']);
            },
            finish: async () => {
              if (window.delayTranscript)
                return new Promise((resolve) => {
                  window.resolveTranscript = resolve;
                });
              return window.transcript ?? 'three';
            },
            cancel: () => {
              window.calls.push(['cancel-record']);
            },
          },
          onAttempt: async (answer) => {
            window.calls.push(['attempt', answer]);
            if (window.delayAttempt)
              await new Promise((resolve) => {
                window.resolveAttempt = resolve;
              });
            return {
              message: 'The server checked this answer.',
              result: 'not_assessed',
            };
          },
          onSupport: async (kind) => {
            window.calls.push(['support', kind]);
            if (window.rejectSupport) throw Error('offline');
            if (window.delaySupport)
              await new Promise((resolve) => {
                window.resolveSupport = resolve;
              });
            return {
              message: 'Help recorded.',
              narrationText: 'Reviewed local narration.',
            };
          },
          narrate: async (text) => {
            window.calls.push(['narrate', text]);
          },
          stopNarration: () => {
            window.calls.push(['stop']);
          },
          onAbandon: async () => {
            window.calls.push(['abandon']);
          },
        },
        { prompt: 'An approved activity instruction.', ...presentation },
      );
      window.activity.focus();
    };
  });
  const mount = async (...args) => page.evaluate((args) => window.mountTest(...args), args);
  const click = async (name) => page.getByRole('button', { name, exact: true }).click();
  const lastAnswer = async () =>
    page.evaluate(() => window.calls.filter((call) => call[0] === 'attempt').at(-1)?.[1]);

  await mount('missing_quantity', { total: 6, given: 2 });
  assert.equal(await page.locator('h2').evaluate((node) => node === document.activeElement), true);
  // Real Tab/Enter navigation exercises the same buttons exposed to touch.
  await page.keyboard.press('Tab');
  assert.equal(
    await page.evaluate(() => document.activeElement.getAttribute('aria-label')),
    'Place plank 1',
  );
  await page.keyboard.press('Enter');
  await click('Add one object');
  await click('Remove one object');
  await click('Check my answer');
  assert.deepEqual(await lastAnswer(), { kind: 'quantity', value: 1 });
  assert.match(await page.getByRole('status').innerText(), /server checked/);

  await mount('equal_sharing', { total: 2, recipients: 2 });
  await click('Give one object to friend 1');
  await click('Give one object to friend 2');
  assert.equal(
    await page
      .getByRole('button', { name: 'Give one object to friend 1', exact: true })
      .isDisabled(),
    true,
  );
  await click('Check my answer');
  assert.deepEqual(await lastAnswer(), { kind: 'groups', groups: [1, 1] });

  await mount('pattern_sort', {
    objects: [
      { id: 'leaf', label: 'Leaf', attribute: 'leaf' },
      { id: 'star', label: 'Star', attribute: 'star' },
    ],
    unitLength: 2,
    prefixLength: 3,
    instructionId: 'extend',
  });
  assert.match(await page.locator('section').textContent(), /Pattern so far: Leaf, Star, Leaf/);
  await click('Choose: Star');
  await click('Check my answer');
  assert.deepEqual(await lastAnswer(), {
    kind: 'choices',
    choiceIds: ['star'],
  });

  await mount(
    'pattern_sort',
    {
      objects: [
        { id: 'a', label: 'Shell', attribute: 'round' },
        { id: 'b', label: 'Stone', attribute: 'round' },
      ],
      instructionId: 'sort',
    },
    { choiceMode: 'multiple' },
  );
  await click('Choose: Shell');
  await click('Choose: Stone');
  await click('Choose: Shell');
  await click('Check my answer');
  assert.deepEqual(await lastAnswer(), { kind: 'choices', choiceIds: ['b'] });

  await mount('spatial_measurement', {
    objects: [
      { id: 'short', label: 'Short rope', length: 2 },
      { id: 'long', label: 'Long rope', length: 5 },
    ],
    instructionId: 'longest',
  });
  await click('Choose: Long rope, length 5 units');
  await click('Check my answer');
  assert.deepEqual(await lastAnswer(), {
    kind: 'choices',
    choiceIds: ['long'],
  });

  await mount('pattern_sort', {
    objects: [
      { id: 'shown-0', label: 'Leaf', attribute: 'shown' },
      { id: 'choice-0', label: 'Star', attribute: 'choice' },
    ],
    prefixLength: 1,
    instructionId: 'pattern-next',
  });
  assert.equal(await page.getByRole('button', { name: 'Choose: Leaf', exact: true }).count(), 0);
  assert.match(await page.locator('section').textContent(), /Pattern so far: Leaf/);
  assert.equal(
    await page.evaluate(() =>
      window.presentationTest({
        templateId: 'pattern_sort',
        parameters: { instructionId: 'invented' },
      }),
    ),
    null,
  );
  assert.equal(
    await page.evaluate(
      () =>
        window.presentationTest({
          templateId: 'pattern_sort',
          parameters: { instructionId: 'sort-round' },
        }).choiceMode,
    ),
    'multiple',
  );

  // Pick-one questions are answered by tapping the picture itself: no abstract answer slot.
  const countParameters = {
    objects: [
      { id: 'shown-0', label: '1', attribute: 'shown' },
      { id: 'shown-1', label: '2', attribute: 'shown' },
      { id: 'number-2', label: '2', attribute: 'choice' },
      { id: 'number-3', label: '3', attribute: 'choice' },
      { id: 'number-4', label: '4', attribute: 'choice' },
    ],
    prefixLength: 2,
    instructionId: 'count-next',
  };
  assert.equal(
    await page.evaluate(
      (parameters) => window.presentationTest({ templateId: 'pattern_sort', parameters }).prompt,
      countParameters,
    ),
    'Count along: 1, 2. Which number comes next?',
  );
  await mount('pattern_sort', countParameters, { choiceMode: 'single' });
  assert.equal(await page.locator('.ww-answer-slot').count(), 0);
  assert.equal(await page.locator('.ww-next-tile').innerText(), '?');
  await click('Choose: 3');
  await click('Choose: 4');
  assert.equal(
    await page.getByRole('button', { name: 'Choose: 3', exact: true }).getAttribute('aria-pressed'),
    'false',
  );
  assert.equal(
    await page.getByRole('button', { name: 'Choose: 4', exact: true }).getAttribute('aria-pressed'),
    'true',
  );
  assert.match(await page.locator('.ww-next-tile').innerText(), /4/);
  await click('Check my answer');
  assert.deepEqual(await lastAnswer(), {
    kind: 'choices',
    choiceIds: ['number-4'],
  });

  await mount(
    'pattern_sort',
    {
      objects: [
        { id: 'a', label: 'round object 1', attribute: 'round' },
        { id: 'b', label: 'square object 2', attribute: 'square' },
        { id: 'c', label: 'round object 3', attribute: 'round' },
      ],
      instructionId: 'sort-round',
    },
    { choiceMode: 'multiple' },
  );
  assert.equal(await page.locator('.ww-answer-slot').count(), 0);
  await click('Choose: round object 1');
  await click('Choose: round object 3');
  await click('Choose: round object 1');
  await click('Check my answer');
  assert.deepEqual(await lastAnswer(), { kind: 'choices', choiceIds: ['c'] });

  await mount(
    'phonics_word_building',
    { tiles: ['t', 'a', 't'], promptId: 'build' },
    { narrationAvailable: true },
    'E07',
  );
  await click('Choose tile 3: t');
  await click('Choose tile 2: a');
  await click('Choose tile 1: t');
  await click('Undo last choice');
  await click('Choose tile 1: t');
  await click('Check my answer');
  assert.deepEqual(await lastAnswer(), {
    kind: 'choices',
    choiceIds: ['2', '1', '0'],
  });
  await click('Read the activity aloud (with support)');
  assert.deepEqual(
    await page.evaluate(() => window.calls.filter((c) => ['support', 'narrate'].includes(c[0]))),
    [
      ['support', 'read_aloud'],
      ['narrate', 'Reviewed local narration.'],
    ],
  );
  await page.evaluate(() => {
    window.rejectSupport = true;
  });
  await click('Show a hint');
  assert.match(await page.getByRole('status').innerText(), /did not reach/);
  assert.equal(await page.evaluate(() => window.calls.filter((c) => c[0] === 'narrate').length), 1);

  await mount(
    'story_message',
    {
      cards: [
        { id: 'water', text: 'Water the seed' },
        { id: 'plant', text: 'Plant the seed' },
      ],
      promptId: 'order',
    },
    {},
    'E11',
  );
  await click('Choose: Plant the seed');
  await click('Choose: Water the seed');
  await click('Check my answer');
  assert.deepEqual(await lastAnswer(), {
    kind: 'choices',
    choiceIds: ['plant', 'water'],
  });
  await mount(
    'story_message',
    {
      cards: [{ id: 'kind', text: '<img src=x onerror=alert(1)>' }],
      promptId: 'reason',
    },
    {},
    'E14',
  );
  assert.equal(await page.locator('img').count(), 0);
  assert.match(
    await page.locator('section').textContent(),
    /does not certify writing or reasoning/,
  );

  await mount('missing_quantity', { total: 6, given: 2 });
  await page.evaluate(() => {
    window.delayAttempt = true;
  });
  await click('Check my answer');
  assert.equal(
    await page.getByRole('button', { name: 'Check my answer', exact: true }).isDisabled(),
    true,
  );
  await page.evaluate(() => {
    window.resolveAttempt();
  });
  await page.waitForFunction(
    () => document.querySelector('section').getAttribute('aria-busy') === 'false',
  );
  assert.equal(await page.evaluate(() => window.calls.filter((c) => c[0] === 'attempt').length), 1);

  await mount('missing_quantity', { total: 999999 });
  assert.equal(await page.getByRole('button', { name: 'Check my answer', exact: true }).count(), 0);
  assert.match(await page.locator('section').innerText(), /updated content adapter/);
  await mount('missing_quantity', { total: 6, given: 2 }, {}, 'M06', ['hint']);
  assert.equal(
    await page.getByRole('button', { name: 'Check my answer', exact: true }).isDisabled(),
    true,
  );
  const dimensions = await page
    .getByRole('button', { name: 'Show a hint', exact: true })
    .boundingBox();
  assert.ok(dimensions.width >= 44 && dimensions.height >= 44);
  await mount(
    'phonics_word_building',
    { tiles: ['c', 'a', 't'], promptId: 'word-cat' },
    { narrationAvailable: true },
    'E06',
  );
  await page.evaluate(() => {
    window.delaySupport = true;
  });
  await click('Read the activity aloud (with support)');
  await page.evaluate(async () => {
    window.activity.destroy();
    window.resolveSupport();
    await Promise.resolve();
    await Promise.resolve();
  });
  assert.equal(await page.locator('section').count(), 0);
  assert.equal(await page.evaluate(() => window.calls.filter((c) => c[0] === 'narrate').length), 0);
  await mount('missing_quantity', { total: 6, given: 2 });
  await page.evaluate(() => {
    window.delayAttempt = true;
  });
  await click('Check my answer');
  await page.evaluate(async () => {
    window.activity.destroy();
    window.resolveAttempt();
    await Promise.resolve();
    await Promise.resolve();
  });
  assert.equal(await page.getByRole('status').count(), 0);
  // Speech is an editable draft, never an automatic attempt or correctness judgement.
  await mount('missing_quantity', { total: 6, given: 2 });
  await click('Say my answer');
  assert.equal(
    await page.getByRole('button', { name: 'Check my answer', exact: true }).isDisabled(),
    true,
  );
  await click('Finish recording');
  await page.waitForFunction(() => document.querySelector('input')?.value === 'three');
  assert.equal(await lastAnswer(), undefined);
  await click('Use this answer');
  await click('Check my answer');
  assert.deepEqual(await lastAnswer(), { kind: 'quantity', value: 3 });
  await page.getByRole('textbox', { name: 'Your answer', exact: true }).fill('two');
  await click('Use this answer');
  await click('Check my answer');
  assert.deepEqual(await lastAnswer(), { kind: 'quantity', value: 2 });
  await page.getByRole('textbox', { name: 'Your answer', exact: true }).fill('two or three');
  await click('Use this answer');
  await click('Check my answer');
  assert.deepEqual(await lastAnswer(), { kind: 'quantity', value: 2 });
  await page.evaluate(() => {
    window.delayTranscript = true;
  });
  await click('Say my answer');
  await click('Finish recording');
  await click('Cancel recording');
  await page.evaluate(() => window.resolveTranscript('six'));
  assert.equal(
    await page.getByRole('textbox', { name: 'Your answer', exact: true }).inputValue(),
    'two or three',
  );
  await click('Say my answer');
  await click('Finish recording');
  await page.evaluate(() => {
    window.activity.destroy();
    window.resolveTranscript('six');
  });
  assert.equal(await page.locator('section').count(), 0);
  await page.evaluate(() => {
    window.delayTranscript = false;
  });
  await mount('phonics_word_building', {
    tiles: ['c', 'a', 't'],
    promptId: 'word-cat',
  });
  assert.equal(await page.getByRole('button', { name: 'Say my answer' }).count(), 0);
  await mount('missing_quantity', { total: 6, given: 2 }, {}, 'M06', ['hint']);
  assert.equal(await page.getByRole('button', { name: 'Say my answer' }).isDisabled(), true);
  assert.equal(
    await page.getByRole('textbox', { name: 'Your answer', exact: true }).isDisabled(),
    true,
  );

  await page.setViewportSize({ width: 390, height: 844 });
  await mount('missing_quantity', { total: 6, given: 2 });
  assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), true);
  await mount(
    'story_message',
    {
      cards: [
        { id: 'red', text: 'Red cup' },
        { id: 'blue', text: 'Blue cup' },
      ],
      promptId: 'read-red-cup',
    },
    { choiceMode: 'single' },
    'E06',
  );
  await page.getByRole('textbox', { name: 'Your answer', exact: true }).fill('Blue cup.');
  await click('Use this answer');
  await click('Check my answer');
  assert.deepEqual(await lastAnswer(), {
    kind: 'choices',
    choiceIds: ['blue'],
  });

  // Exercise real browser recording and WAV conversion with a synthetic microphone.
  let upload;
  await page.route('**/__demo/transcribe', async (route) => {
    upload = route.request().postDataBuffer();
    assert.equal(route.request().headers()['x-csrf-token'], 'test-token');
    await route.fulfill({ contentType: 'text/plain', body: 'four' });
  });
  const transcript = await page.evaluate(async () => {
    const { createRecorder } = await import('/src/learning/voice.ts');
    const recorder = createRecorder('test-token', 'test-profile');
    await recorder.start();
    await new Promise((resolve) => setTimeout(resolve, 300));
    const text = await recorder.finish();
    recorder.cancel();
    return text;
  });
  assert.equal(transcript, 'four');
  assert.equal(upload.toString('ascii', 0, 4), 'RIFF');
  assert.equal(upload.readUInt32LE(24), 16000);
  assert.equal(upload.readUInt16LE(22), 1);
  assert.ok(upload.length > 44 && upload.length <= 640044);
  assert.deepEqual(errors, []);
  console.log(
    'Activity browser checks PASS: six families, sort, keyboard, editable answers, support-before-audio, failed support, duplicate guard, safe text, permission guards, invalid content, tablet targets.',
  );
} finally {
  await browser.close();
}
