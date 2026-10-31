// Current seven controls and semantic diagrams, using private synthetic fixtures only in the test runner.
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { createRequire } from 'node:module';
const require = createRequire(new URL('../../../qa/e2e/package.json', import.meta.url));
const { chromium } = require('playwright');
const bank = JSON.parse(
  await readFile(new URL('../../../content/activities/skill-exercises-v1.json', import.meta.url)),
);
const examples = [
  ['M05', 'one_more'],
  ['M10', 'features'],
  ['MY304', 'remainder'],
  ['EY106', 'cvc'],
  ['M13', 'order_events'],
  ['M01', 'one_for_each'],
  ['E13', 'observed'],
];
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH });
const origin = process.env.WW_ACTIVITY_TEST_URL ?? 'http://127.0.0.1:5173';
try {
  for (const width of [1280, 390, 320]) {
    const context = await browser.newContext({ viewport: { width, height: 1000 }, hasTouch: true });
    const page = await context.newPage();
    const errors = [];
    page.on('pageerror', (e) => errors.push(e.message));
    await page.route('**/__questions_test', (r) =>
      r.fulfill({
        contentType: 'text/html',
        body: '<!doctype html><html lang="en"><meta name="viewport" content="width=device-width,initial-scale=1"><style>body{font:18px Trebuchet MS;margin:12px}main{max-width:680px;margin:auto}button{font:inherit}</style><main></main></html>',
      }),
    );
    await page.goto(`${origin}/__questions_test`);
    await page.evaluate(async () => {
      const { mountMissionActivity } = await import('/src/mission-learning/activity.ts');
      window.mount = (activity, replacement, recorder = false) => {
        window.control?.destroy();
        window.answers = [];
        window.help = [];
        window.finished = 0;
        window.replaced = 0;
        const snapshot = { revision: 1, activeActivity: activity, skills: [] };
        window.control = mountMissionActivity(document.querySelector('main'), activity, snapshot, {
          recorder: recorder
            ? {
                start: async () => {},
                finish: async () => window.transcript ?? 'seven',
                cancel: () => {},
              }
            : undefined,
          onAttempt: async (answer) => {
            window.answers.push(answer);
            return replacement
              ? {
                  ...snapshot,
                  activeActivity: { ...activity, variantId: 'replacement' },
                  lastFeedback: {
                    result: 'incorrect',
                    message: 'Try another.',
                    completed: false,
                    explanation: 'Count forward one step.',
                  },
                }
              : snapshot;
          },
          onSupport: async (kind) => {
            window.help.push(kind);
            return {
              ...snapshot,
              activeActivity: {
                ...activity,
                revealedSupport: { kind, text: 'Look carefully at the actual question.' },
              },
            };
          },
          onRetry: async () => snapshot,
          onRefresh: async () => snapshot,
          hasPending: () => false,
          onAbandon: async () => {},
          onContinue: () => window.finished++,
          onReplace: () => window.replaced++,
          onFinishTogether: async () => ({
            ...snapshot,
            activeActivity: undefined,
            skills: [{ skillId: activity.skillId, activityCompleted: true }],
            lastFeedback: { result: 'assisted', message: 'Shared.', completed: true },
          }),
        });
      };
    });
    for (const [skillId, suffix] of examples) {
      const exercise = bank.skills
        .find((s) => s.skillId === skillId)
        .exercises.find((e) => e.id === `${skillId}.${suffix}`);
      const item = exercise.items[0];
      const activity = {
        schemaVersion: '1.0',
        id: 'synthetic-question',
        skillId,
        exerciseId: exercise.id,
        variantId: 'question-0',
        contentVersion: 'test',
        contextId: 'test',
        representation: item.representation,
        presentation: {
          title: item.title,
          objective: 'Draft objective',
          lessonText: 'Help is optional.',
          prompt: item.prompt,
          context: 'Pip has a little challenge for you.',
        },
        support: { hintCount: 0, workedExample: false, readAloud: false },
        task: {
          kind: 'question',
          exerciseId: exercise.id,
          interaction: item.interaction,
          stimuli: item.stimuli,
        },
      };
      await page.evaluate((a) => window.mount(a), activity);
      const root = page.locator('.ww-question-shell');
      const check = root.getByRole('button', { name: 'Check answer', exact: false });
      const interaction = item.interaction;
      if (interaction.kind === 'single_choice') {
        await root.getByRole('radio').first().focus();
        await page.keyboard.press('Space');
        await check.click();
        assert.equal(await page.evaluate(() => window.answers[0].kind), 'choice');
      }
      if (interaction.kind === 'multi_choice') {
        await root.getByRole('checkbox').first().check();
        await root.getByRole('checkbox').last().check();
        await check.click();
        assert.equal(await page.evaluate(() => window.answers[0].choiceIds.length), 2);
      }
      if (interaction.kind === 'number_entry') {
        for (const value of item.answer.values)
          await root
            .getByLabel(interaction.fields.find((f) => f.id === value.fieldId).label, {
              exact: false,
            })
            .fill(value.value);
        await root.getByRole('textbox').last().press('Enter');
        assert.equal(await page.evaluate(() => window.answers[0].kind), 'numbers');
      }
      if (interaction.kind === 'text_entry') {
        await root.getByLabel(interaction.label).fill('hat');
        await check.click();
        assert.deepEqual(await page.evaluate(() => window.answers[0]), {
          kind: 'text',
          value: 'hat',
        });
      }
      if (interaction.kind === 'order' || interaction.kind === 'assign') {
        const count = interaction.items.length;
        for (let i = 0; i < count; i++) {
          await root.locator('.ww-question-piece').nth(i).tap();
          await root.locator('.ww-question-place').nth(i).tap();
        }
        await check.click();
        assert.equal(await page.evaluate(() => window.answers[0].kind), interaction.kind);
        await root.getByRole('button', { name: 'Undo', exact: false }).click();
        assert.equal(await check.isEnabled(), false);
      }
      if (interaction.kind === 'workshop') {
        assert.equal(await check.count(), 0);
        await root.getByRole('textbox').fill('Dear Pip, meet me for a picnic.');
        await root.getByRole('button', { name: 'Ready to share', exact: false }).click();
        assert.equal(await page.evaluate(() => window.finished), 1);
        assert.deepEqual(await page.evaluate(() => window.answers), []);
      } else {
        await root.getByRole('button', { name: 'Hint', exact: false }).click();
        assert.equal(await root.locator('.ww-mission-support-text').isVisible(), true);
        assert.deepEqual(await page.evaluate(() => window.help), ['hint']);
      }
      assert.equal(
        await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth),
        true,
        `${exercise.id} at ${width}`,
      );
      if (interaction.kind !== 'workshop')
        await page.screenshot({
          path: `/tmp/wonderwilds-live-${interaction.kind}-${width}.png`,
          fullPage: true,
        });
      if (interaction.kind === 'single_choice') {
        await page.evaluate((a) => window.mount(a, true), activity);
        await root.getByRole('radio').first().check();
        await check.click();
        assert.match(await root.locator('.ww-mission-status').textContent(), /Count forward/);
        assert.equal(await page.evaluate(() => window.replaced), 0);
        await root.getByRole('button', { name: 'Try another', exact: true }).click();
        assert.equal(await page.evaluate(() => window.replaced), 1);
      }
    }
    // Every semantic diagram receives a responsive render check using real authored examples.
    const seen = new Set();
    for (const skill of bank.skills)
      for (const exercise of skill.exercises)
        for (const item of exercise.items) {
          for (const stimulus of item.stimuli) {
            if (seen.has(stimulus.kind)) continue;
            seen.add(stimulus.kind);
            await page.evaluate(async (stimulus) => {
              window.control?.destroy();
              const { renderStimulus } = await import('/src/mission-learning/stimulus.ts');
              document.querySelector('main').replaceChildren(renderStimulus(document, stimulus));
            }, stimulus);
            assert.equal(
              await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth),
              true,
              `${stimulus.kind} at ${width}`,
            );
          }
        }
    const numericItem = bank.skills
      .find((s) => s.skillId === 'M07')
      .exercises.find((e) => e.id === 'M07.join').items[0];
    const numericActivity = {
      schemaVersion: '1.0',
      id: 'voice-test',
      skillId: 'M07',
      variantId: 'M07-test-0',
      contentVersion: 'test',
      contextId: 'test',
      representation: numericItem.representation,
      presentation: {
        title: numericItem.title,
        objective: 'Add objects.',
        lessonText: 'Try it.',
        prompt: numericItem.prompt,
        context: 'Pip has a challenge.',
      },
      support: { hintCount: 0, workedExample: false, readAloud: false },
      task: {
        kind: 'question',
        exerciseId: 'M07.join',
        interaction: numericItem.interaction,
        stimuli: numericItem.stimuli,
      },
    };
    await page.evaluate((a) => window.mount(a, false, true), numericActivity);
    const numberField = page.getByLabel('Your answer', { exact: true });
    await numberField.fill('5');
    for (const transcript of ['seven', 'maybe seven or eight']) {
      await page.evaluate((value) => (window.transcript = value), transcript);
      await page.getByRole('button', { name: 'Say my answer', exact: true }).click();
      await page.getByRole('button', { name: 'Finish recording', exact: true }).click();
      await page.getByRole('button', { name: 'Use this answer', exact: true }).click();
      assert.equal(await numberField.inputValue(), '7');
      assert.deepEqual(await page.evaluate(() => window.answers), []);
    }
    await page.getByRole('button', { name: 'Check answer', exact: false }).click();
    assert.equal(await page.evaluate(() => window.answers[0].values[0].value), '7');
    await page.evaluate(async () => {
      window.control?.destroy();
      const { renderStimulus } = await import('/src/mission-learning/stimulus.ts');
      document
        .querySelector('main')
        .replaceChildren(renderStimulus(document, { kind: 'audio', assetId: 'ww-audio-missing' }));
    });
    assert.equal(await page.locator('audio').count(), 0);
    assert.match(await page.locator('main').textContent(), /grown-up/);
    assert.deepEqual(errors, []);
    await context.close();
    console.log(`Current mission controls and ${seen.size} stimulus types passed at ${width}px`);
  }
} finally {
  await browser.close();
}
