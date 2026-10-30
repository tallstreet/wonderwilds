// Integrated story practice against the runner's fresh disposable database.
// Run with WW_QA_BROWSER_TEST=qa/e2e/mission-learning.mjs via run-training-qa.mjs.
import assert from 'node:assert/strict';
import { chromium } from 'playwright';

const origin = process.env.WW_DEMO_TEST_URL ?? 'http://127.0.0.1:5180';
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH });
const context = await browser.newContext({ viewport: { width: 1280, height: 800 } });
const page = await context.newPage();
const errors = [];
page.on('pageerror', (error) => errors.push(error.message));
const guestSave = '{"keep":"untouched guest story bytes"}';
await page.addInitScript((value) => {
  if (!localStorage.getItem('starwhisker.meadow.v1'))
    localStorage.setItem('starwhisker.meadow.v1', value);
}, guestSave);

try {
  await page.goto(`${origin}/?story=1`);
  await page.locator('#game canvas').waitFor();
  const bootstrap = await (await page.request.get(`${origin}/__demo/bootstrap`)).json();
  const profileHeaders = { 'X-Wonderwilds-Profile': bootstrap.profileId };
  const commandHeaders = {
    ...profileHeaders,
    'X-CSRF-Token': bootstrap.csrf,
    Origin: origin,
  };
  const get = async (path) => {
    const response = await page.request.get(`${origin}${path}`, { headers: profileHeaders });
    assert.equal(response.status(), 200, `${path} responds for the selected synthetic profile`);
    return response.json();
  };
  const storySnapshot = () => get('/api/v1/story');
  const learningSnapshot = () => get('/api/v1/mission-learning');
  const originalGame = await get('/api/v1/game');
  assert.equal((await storySnapshot()).revision, 0, 'Use the fresh disposable QA profile');
  assert.equal(await page.evaluate(() => localStorage.getItem('starwhisker.meadow.v1')), guestSave);

  const visualPage = await context.newPage();
  await visualPage.route(`${origin}/__mission_learning_components`, (route) =>
    route.fulfill({
      contentType: 'text/html',
      body: '<!doctype html><html lang="en"><body></body></html>',
    }),
  );
  await visualPage.goto(`${origin}/__mission_learning_components`);
  await visualPage.evaluate(async () => {
    const { mountMissionActivity } = await import('/src/mission-learning/activity.ts');
    window.mountMissionFixture = (activity) => {
      window.missionComponent?.destroy();
      window.missionCalls = [];
      const snapshot = {
        schemaVersion: '1.0',
        revision: 1,
        available: true,
        activeActivity: activity,
        skills: [],
      };
      window.missionComponent = mountMissionActivity(document.body, activity, snapshot, {
        onAttempt: async (answer) => {
          window.missionCalls.push(['answer', answer]);
          return {
            ...snapshot,
            lastFeedback: { result: 'incorrect', message: 'Try again.', completed: false },
          };
        },
        onSupport: async (kind) => {
          window.missionCalls.push(['support', kind]);
          const activeActivity = {
            ...activity,
            support: { hintCount: 1, workedExample: kind === 'worked_example', readAloud: false },
            revealedSupport: { kind, text: 'Try counting one object at a time.' },
          };
          return { ...snapshot, activeActivity };
        },
        onRetry: async () => snapshot,
        onRefresh: async () => snapshot,
        hasPending: () => false,
        onAbandon: async () => {},
        onContinue: () => {},
      });
    };
  });
  const activityBase = {
    schemaVersion: '1.0',
    id: 'visual-task',
    skillId: 'M01',
    contentVersion: 'draft',
    variantId: 'visual-v1',
    contextId: 'bridge',
    representation: 'object_pairing',
    presentation: {
      title: 'A small practice',
      objective: 'Share one object with each friend.',
      lessonText: 'Every object has one place.',
      prompt: 'Find a matching place for each object.',
      context: 'Help Pip get ready.',
    },
    support: { hintCount: 0, workedExample: false, readAloud: false },
    task: null,
  };
  const mountVisual = async (activity) =>
    visualPage.evaluate((value) => window.mountMissionFixture(value), activity);
  const componentError = (expression) => visualPage.locator(expression);
  const pairing = (layout) => ({
    ...activityBase,
    representation: layout === 'object_cards' ? 'object_pairing' : 'recipient_slots',
    task: {
      kind: 'pairing',
      layout,
      objects: [
        { id: 'object-a', label: 'Red seed' },
        { id: 'object-b', label: 'Blue seed' },
      ],
      recipients: [
        { id: 'recipient-a', label: 'Pip' },
        { id: 'recipient-b', label: 'Bramble' },
      ],
    },
  });
  await mountVisual(pairing('object_cards'));
  assert.match(await componentError('.ww-mission-objective').textContent(), /Share one object/);
  let visualSelects = componentError('.ww-mission-pair-row select');
  await visualSelects.nth(0).selectOption('recipient-a');
  await visualSelects.nth(1).selectOption('recipient-a');
  assert.equal(
    await componentError('button').filter({ hasText: 'Check my idea' }).isEnabled(),
    true,
    'A non-bijective pairing reaches the server for assessment',
  );
  await componentError('button').filter({ hasText: 'Check my idea' }).click();
  assert.deepEqual(await visualPage.evaluate(() => window.missionCalls[0]), [
    'answer',
    {
      kind: 'pairing',
      assignments: [
        { objectId: 'object-a', recipientId: 'recipient-a' },
        { objectId: 'object-b', recipientId: 'recipient-a' },
      ],
    },
  ]);
  await mountVisual(pairing('recipient_slots'));
  visualSelects = componentError('.ww-mission-pair-row select');
  await visualSelects.nth(0).selectOption('object-a');
  await visualSelects.nth(1).selectOption('object-a');
  await componentError('button').filter({ hasText: 'Check my idea' }).click();
  assert.equal(await visualPage.evaluate(() => window.missionCalls[0][1].assignments.length), 2);

  const countFixture = (representation) => ({
    ...activityBase,
    skillId: 'M03',
    representation,
    task: {
      kind: 'count_collection',
      layout: representation,
      objects: [
        { id: 'one', label: 'Plank one' },
        { id: 'two', label: 'Plank two' },
        { id: 'three', label: 'Plank three' },
      ],
      choices: [
        { id: 'n2', label: 'Two', value: 2 },
        { id: 'n3', label: 'Three', value: 3 },
      ],
    },
  });
  await mountVisual(countFixture('ten_frame'));
  assert.equal(await componentError('.ww-mission-ten-cell').count(), 10);
  const frameGeometry = await componentError('.ww-mission-ten-row').evaluate((element) => {
    const style = getComputedStyle(element);
    return {
      columns: style.gridTemplateColumns.split(' ').length,
      rows: style.gridTemplateRows.split(' ').length,
    };
  });
  assert.deepEqual(frameGeometry, { columns: 5, rows: 2 });
  assert.equal(await componentError('.ww-mission-ten-cell.is-filled').count(), 3);
  await mountVisual(countFixture('movable_objects'));
  await componentError('.ww-mission-movable-object').first().click();
  assert.equal(
    await componentError('.ww-mission-movable-count .ww-mission-movable-object').count(),
    1,
  );

  const compare = (representation) => ({
    ...activityBase,
    skillId: 'M04',
    representation,
    task: {
      kind: 'compare_collections',
      promptMode: 'more',
      collections: [
        {
          id: 'left',
          label: 'Picnic plates',
          items: [
            { id: 'a', label: 'Berry' },
            { id: 'b', label: 'Berry' },
          ],
        },
        {
          id: 'right',
          label: 'Basket',
          items: [
            { id: 'c', label: 'Berry' },
            { id: 'd', label: 'Berry' },
            { id: 'e', label: 'Berry' },
          ],
        },
      ],
      choices: [
        { id: 'left', label: 'Picnic plates' },
        { id: 'right', label: 'Basket' },
      ],
    },
  });
  await mountVisual(compare('paired_collections'));
  assert.equal(await componentError('.ww-mission-collections').count(), 1);
  await mountVisual(compare('aligned_rows'));
  assert.equal(await componentError('.ww-mission-aligned-items').count(), 2);

  const path = (representation) => ({
    ...activityBase,
    skillId: 'M05',
    representation,
    task: {
      kind: 'number_path',
      shown: [2, 3, 4],
      direction: 'forward',
      choices: [
        { id: 'n5', label: 'Five', value: 5 },
        { id: 'n6', label: 'Six', value: 6 },
      ],
    },
  });
  await mountVisual(path('number_path'));
  assert.equal(await componentError('.ww-mission-number-path .ww-mission-path-number').count(), 4);
  await mountVisual(path('countable_flower_sequence'));
  assert.equal(await componentError('.ww-mission-flower-group').count(), 4);
  assert.equal(await componentError('.ww-mission-flower').count(), 9);

  const partWhole = (representation) => ({
    ...activityBase,
    skillId: 'M06',
    representation,
    task: {
      kind: 'part_whole',
      whole: 5,
      shownPart: 3,
      choices: [
        { id: 'n1', label: 'One', value: 1 },
        { id: 'n2', label: 'Two', value: 2 },
      ],
    },
  });
  await mountVisual(partWhole('countable_planks'));
  assert.equal(await componentError('.ww-mission-plank').count(), 5);
  await mountVisual(partWhole('part_whole_objects'));
  assert.equal(await componentError('.ww-mission-whole-group').count(), 2);
  assert.equal(await componentError('.ww-mission-part-object').count(), 8);

  await mountVisual(countFixture('ten_frame'));
  assert.equal(await componentError('.ww-mission-support-text').isVisible(), false);
  await componentError('button').filter({ hasText: 'Ask for a hint' }).click();
  await componentError('.ww-mission-support-text')
    .filter({ hasText: 'Try counting one object at a time.' })
    .waitFor();
  await visualPage.setViewportSize({ width: 390, height: 844 });
  assert.ok(await visualPage.evaluate(() => document.documentElement.scrollWidth <= innerWidth));
  await visualPage.close();

  // Accept Pip’s real story mission through its authoritative API command.
  const initialStory = await storySnapshot();
  const accept = initialStory.sceneInput.availableActions.find(
    (entry) => entry.action.type === 'accept_mission',
  );
  assert.ok(accept, 'Pip offers the first story mission');
  const acceptedResponse = await page.request.post(`${origin}/api/v1/story/commands`, {
    headers: commandHeaders,
    data: {
      schemaVersion: '1.0',
      commandId: 'mission-learning-accept-story',
      expectedRevision: initialStory.revision,
      payload: { type: 'choose', actionId: accept.id },
    },
  });
  assert.equal(acceptedResponse.status(), 200);
  await page.reload();
  await page.locator('#game canvas').waitFor();
  await page.locator('#missions').click();
  assert.equal(await page.getByRole('button', { name: 'Practice this story skill' }).count(), 0);
  const openCollection = async () => {
    if (await page.locator('dialog[open]').count()) await page.keyboard.press('Escape');
    await page.locator('#missions').click();
    await page.locator('[data-story-visit="resource:fallen_wood"]').click();
    await page.locator('dialog[open] [data-story-action="collect"]').click();
    await page.locator('.ww-mission-activity').waitFor();
  };
  await openCollection();
  const started = await learningSnapshot();
  const activity = started.activeActivity;
  assert.ok(activity.storyStep, 'The task is bound to the selected mission action');
  assert.equal(activity.skillId, 'M01');
  assert.equal(activity.task.kind, 'pairing');
  assert.equal(await page.locator('.ww-mission-context').textContent(), activity.storyStep.introduction);
  assert.equal((await storySnapshot()).revision, acceptedResponse.ok() ? initialStory.revision + 1 : -1);
  assert.equal(await page.locator('.ww-mission-support-text').isVisible(), false);
  await page.screenshot({ path: '/tmp/wonderwilds-mission-learning-desktop.png', fullPage: true });

  // The first authored pairing has one object; the server completes it, not the client.
  let selects = page.locator('.ww-mission-pair-row select');
  assert.equal(await selects.count(), 1);
  const oneValidChoice = await selects.first().locator('option').nth(1).getAttribute('value');
  await selects.first().selectOption(oneValidChoice);
  await page.getByRole('button', { name: 'Check my idea' }).click();
  await page.getByRole('status').filter({ hasText: 'That works!' }).waitFor();
  const firstCorrect = await learningSnapshot();
  await page.waitForFunction((revision) => Number(document.querySelector('#game')?.dataset.storyRevision) === revision, (await storySnapshot()).revision);
  assert.equal(firstCorrect.lastFeedback.result, 'correct');
  assert.equal(firstCorrect.lastFeedback.completed, true);
  assert.equal(firstCorrect.skills.find((skill) => skill.skillId === 'M01').correctChecks, 1);
  await page.getByRole('button', { name: 'Continue the story' }).click();
  await page.locator('dialog').waitFor({ state: 'hidden' });
  assert.ok((await storySnapshot()).sceneInput.context.inventory.some((item) => item.itemId === 'wood'));
  // Move to the workshop through its normal authoritative travel action.
  let currentStory = await storySnapshot();
  const travel = currentStory.sceneInput.availableActions.find((a) => a.action.type === 'travel' && a.action.destinationId === 'workshop');
  const sendStory = async (snapshot, payload, id) => {
    const result = await page.request.post(`${origin}/api/v1/story/commands`, {
      headers: commandHeaders, data: { schemaVersion: '1.0', commandId: id, expectedRevision: snapshot.revision, payload },
    });
    assert.equal(result.status(), 200);
    return result.json();
  };
  currentStory = await sendStory(currentStory, { type: 'choose', actionId: travel.id }, 'inline-travel-workshop');
  if (currentStory.acknowledgeableEntryIds.length) {
    currentStory = await sendStory(currentStory, { type: 'acknowledge', sceneId: currentStory.scene.sceneId, entryId: currentStory.acknowledgeableEntryIds[0] }, 'inline-greet-woodcutter');
  }
  await page.reload();
  await page.locator('#game canvas').waitFor();
  await page.locator('#missions').click();
  await page.locator('[data-story-visit="npc:woodcutter"]').click();
  await page.locator('dialog[open] [data-story-action="exchange"]').click();
  await page.locator('.ww-mission-activity').waitFor();
  const secondTask = (await learningSnapshot()).activeActivity;
  assert.equal(secondTask.task.kind, 'pairing');
  assert.ok(secondTask.storyStep);
  const beforeSecond = (await storySnapshot()).revision;

  // Intentionally send a non-bijective answer. Only the server decides its result.
  selects = page.locator('.ww-mission-pair-row select');
  const rowCount = await selects.count();
  assert.ok(rowCount >= 2);
  const repeatedOption = await selects.nth(0).locator('option').nth(1).getAttribute('value');
  for (let index = 0; index < rowCount; index++)
    await selects.nth(index).selectOption(repeatedOption);
  await page.getByRole('button', { name: 'Check my idea' }).click();
  await page
    .getByRole('status')
    .filter({ hasText: 'Try another way, or ask for a hint.' })
    .waitFor();
  const incorrect = await learningSnapshot();
  assert.equal(incorrect.lastFeedback.result, 'incorrect');
  assert.equal(incorrect.lastFeedback.completed, false);
  assert.equal(incorrect.activeActivity.id, secondTask.id);
  const m01 = incorrect.skills.find((skill) => skill.skillId === 'M01');
  assert.equal(m01.correctChecks, 1, 'A wrong independent check does not add correct progress');
  assert.equal(m01.status, 'practising');

  // Simulate a lost support response after the server commits it; reload must retry exact bytes.
  let originalSupportBody;
  await page.route(
    '**/api/v1/mission-learning/commands',
    async (route) => {
      originalSupportBody = route.request().postData();
      const response = await route.fetch();
      assert.equal(response.status(), 200);
      await route.abort('failed');
    },
    { times: 1 },
  );
  await page.getByRole('button', { name: 'Ask for a hint' }).click();
  await page.getByRole('button', { name: 'Retry the same saved action' }).waitFor();
  assert.equal(await page.locator('.ww-mission-support-text').isVisible(), false);
  await page.reload();
  await page.locator('#game canvas').waitFor();
  await page.locator('#missions').click();
  await page.getByRole('button', { name: 'Retry the same practice save' }).waitFor();
  const retryRequest = page.waitForRequest((request) =>
    request.url().endsWith('/api/v1/mission-learning/commands'),
  );
  await page.getByRole('button', { name: 'Retry the same practice save' }).click();
  assert.equal(
    (await retryRequest).postData(),
    originalSupportBody,
    'Reload retries the identical command body',
  );
  await page.locator('.ww-mission-activity').waitFor();
  const supported = await learningSnapshot();
  assert.equal(supported.activeActivity.support.hintCount, 1);
  assert.equal(supported.activeActivity.revealedSupport.kind, 'hint');
  assert.equal(supported.skills.find((skill) => skill.skillId === 'M01').correctChecks, 1);
  await page.getByText(supported.activeActivity.revealedSupport.text, { exact: true }).waitFor();
  await page.setViewportSize({ width: 390, height: 844 });
  assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth));
  await page.screenshot({ path: '/tmp/wonderwilds-mission-learning-mobile.png', fullPage: true });

  // The supported route commits this mission step without independent success.
  await page.getByRole('button', { name: 'Let’s do it together', exact: true }).click();
  await page.getByRole('button', { name: 'Continue the story' }).waitFor();
  const assisted = await learningSnapshot();
  assert.equal(assisted.lastFeedback.result, 'assisted');
  assert.equal(assisted.activeActivity, undefined);
  assert.equal(assisted.skills.find((skill) => skill.skillId === 'M01').correctChecks, 1);
  assert.equal((await storySnapshot()).revision, beforeSecond + 1);
  await page.getByRole('button', { name: 'Continue the story' }).click();
  await page.locator('dialog').waitFor({ state: 'hidden' });
  const afterStory = await storySnapshot();
  assert.ok(afterStory.sceneInput.context.inventory.some((item) => item.itemId === 'planks'));
  // Lose a response that atomically completed both the learning task and bridge.
  currentStory = await storySnapshot();
  const home = currentStory.sceneInput.availableActions.find((a) => a.action.type === 'travel' && a.action.destinationId === 'bridge_bank');
  await sendStory(currentStory, { type: 'choose', actionId: home.id }, 'inline-return-to-bridge');
  await page.reload();
  await page.locator('#game canvas').waitFor();
  await page.locator('#missions').click();
  await page.locator('[data-story-visit="object:bridge"]').click();
  await page.locator('[data-story-action="assemble"]').click();
  await page.locator('.ww-mission-activity').waitFor();
  let completionBody;
  await page.route('**/api/v1/mission-learning/commands', async (route) => {
    completionBody = route.request().postData();
    const result = await route.fetch();
    assert.equal(result.status(), 200);
    await route.abort('failed');
  }, { times: 1 });
  await page.getByRole('button', { name: 'Let’s do it together', exact: true }).click();
  await page.getByRole('button', { name: 'Retry the same saved action' }).waitFor();
  const completedRevision = (await storySnapshot()).revision;
  await page.reload();
  await page.locator('#game canvas').waitFor();
  await page.locator('#missions').click();
  const completionRetry = page.waitForRequest((request) => request.url().endsWith('/mission-learning/commands'));
  await page.getByRole('button', { name: 'Retry the same practice save' }).click();
  assert.equal((await completionRetry).postData(), completionBody);
  await page.getByRole('button', { name: 'Retry the same practice save' }).waitFor({ state: 'hidden' });
  assert.equal((await storySnapshot()).revision, completedRevision);
  assert.equal(await page.locator('#game').getAttribute('data-story-bridge'), 'repaired');
  assert.equal((await learningSnapshot()).skills.find((skill) => skill.skillId === 'M01').correctChecks, 1);

  assert.deepEqual(
    await get('/api/v1/game'),
    originalGame,
    'Practice never mutates educational adventure progress',
  );
  assert.equal(await page.evaluate(() => localStorage.getItem('starwhisker.meadow.v1')), guestSave);
  assert.deepEqual(errors, []);
  console.log(
    'PASS: inline mission tasks, atomic story progression, M01 assessment, support, exact reload retry, assisted completion and isolated guest/game saves',
  );
} catch (error) {
  console.error(
    'Mission-learning story:',
    await page
      .locator('#game')
      .evaluate((node) => ({ ...node.dataset }))
      .catch(() => ({})),
  );
  console.error(
    'Visible dialog:',
    await page
      .locator('dialog[open]')
      .innerText()
      .catch(() => 'none'),
  );
  await page
    .screenshot({ path: '/tmp/wonderwilds-mission-learning-failure.png', fullPage: true })
    .catch(() => {});
  throw error;
} finally {
  await browser.close();
}
