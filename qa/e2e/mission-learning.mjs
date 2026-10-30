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
  assert.equal(await componentError('select').count(), 0);
  await componentError('.ww-mission-piece').nth(0).click();
  await componentError('.ww-mission-pair-row').nth(0).click();
  await componentError('.ww-mission-piece').nth(0).click();
  await componentError('.ww-mission-pair-row').nth(1).click();
  assert.equal(await componentError('button').filter({ hasText: 'Check my idea' }).isEnabled(), false,
    'Moving a piece frees its previous place');
  await componentError('.ww-mission-piece').nth(1).click();
  await componentError('.ww-mission-pair-row').nth(0).click();
  await componentError('button').filter({ hasText: 'Check my idea' }).click();
  assert.deepEqual(await visualPage.evaluate(() => window.missionCalls[0]), ['answer', {
    kind: 'pairing', assignments: [
      { objectId: 'object-b', recipientId: 'recipient-a' },
      { objectId: 'object-a', recipientId: 'recipient-b' },
    ],
  }]);
  await mountVisual(pairing('recipient_slots'));
  for (let i = 0; i < 2; i++) {
    await componentError('.ww-mission-piece').nth(i).click();
    await componentError('.ww-mission-pair-row').nth(i).click();
  }
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

  assert.ok(activity.task.objects.length >= 3);
  for (let index = 0; index < activity.task.objects.length; index++) {
    await page.locator('.ww-mission-piece').nth(index).click();
    await page.locator('.ww-mission-pair-row').nth(index).click();
  }
  let conflictAttempts = 0;
  await page.route('**/api/v1/mission-learning/commands', async route => {
    conflictAttempts++;
    const saved = await route.fetch();
    assert.equal(saved.status(), 200);
    // Model another writer having already completed the activity when this client sees a 409.
    await route.fulfill({status:409,json:{code:'REVISION_CONFLICT',schemaVersion:'1.0',correlationId:'synthetic-conflict'}});
  }, {times:1});
  await page.getByRole('button', {name:'Check my idea',exact:true}).click();
  await page.locator('dialog').waitFor({state:'hidden'});
  assert.equal(conflictAttempts,1);
  assert.equal((await learningSnapshot()).revision,2,'Read-only conflict recovery must not add another command');
  assert.equal(await page.getByRole('button',{name:'Continue the story',exact:true}).count(),0);
  const firstCorrect = await learningSnapshot();
  assert.equal(firstCorrect.activeActivity,undefined);
  assert.equal(firstCorrect.skills.find(s=>s.skillId==='M01').activityCompleted,true);
  assert.notEqual(firstCorrect.frontierSkillId,'M01');
  const sendStory = async (snapshot,payload,id) => {
    const response = await page.request.post(`${origin}/api/v1/story/commands`,{headers:commandHeaders,data:{schemaVersion:'1.0',commandId:id,expectedRevision:snapshot.revision,payload}});
    assert.equal(response.status(),200); return response.json();
  };
  let currentStory = await storySnapshot();
  const travel = currentStory.sceneInput.availableActions.find(a=>a.action.type==='travel' && a.action.destinationId==='workshop');
  currentStory = await sendStory(currentStory,{type:'choose',actionId:travel.id},'travel-workshop');
  if(currentStory.acknowledgeableEntryIds.length) currentStory = await sendStory(currentStory,{type:'acknowledge',sceneId:currentStory.scene.sceneId,entryId:currentStory.acknowledgeableEntryIds[0]},'greet-woodcutter');
  await page.reload();
  await page.locator('#game canvas').waitFor();
  await page.locator('#missions').click();
  await page.locator('[data-story-visit="npc:woodcutter"]').click();
  await page.locator('dialog[open] [data-story-action="exchange"]').click();
  await page.locator('.ww-mission-activity').waitFor();
  const second = (await learningSnapshot()).activeActivity;
  assert.equal(second.skillId,'M03','The completed matching skill must not return');
  const beforeWrong = (await storySnapshot()).revision;
  const wrong = second.task.choices.find(c=>c.value!==second.task.objects.length);
  await page.locator(`input[value="${wrong.id}"]`).check();
  let wrongBody;
  await page.route('**/api/v1/mission-learning/commands',async route=>{
    wrongBody=route.request().postData(); const response=await route.fetch();
    assert.equal(response.status(),200); await route.abort('failed');
  },{times:1});
  await page.getByRole('button',{name:'Check my idea',exact:true}).click();
  await page.getByRole('button',{name:'Retry the same saved action',exact:true}).waitFor();
  const replaced=(await learningSnapshot()).activeActivity;
  assert.equal(replaced.skillId,second.skillId);
  assert.notEqual(replaced.variantId,second.variantId);
  assert.notDeepEqual(replaced.task,second.task);
  assert.equal((await storySnapshot()).revision,beforeWrong,'Wrong answer cannot advance the story');
  const walkWhileSaving = async () => {
    const start = await page.locator('#game').evaluate(el => [Number(el.dataset.storyPlayerX), Number(el.dataset.storyPlayerY)]);
    let moved = false;
    for (const key of ['ArrowLeft','ArrowDown','ArrowRight','ArrowUp']) {
      await page.keyboard.down(key);
      try {
        await page.waitForFunction(([x,y]) => {
          const el = document.querySelector('#game');
          return Math.hypot(Number(el.dataset.storyPlayerX)-x, Number(el.dataset.storyPlayerY)-y) > 5;
        }, start, {timeout:1000});
        moved = true;
      } catch {} finally { await page.keyboard.up(key); }
      if (moved) break;
    }
    assert.equal(moved,true,'A pending activity save must not freeze local walking');
    assert.equal((await storySnapshot()).revision,beforeWrong,'Walking during recovery cannot commit story actions');
  };
  await page.getByRole('button',{name:'Close dialog',exact:true}).click();
  await walkWhileSaving();
  // Hold the automatic retry response across reload: walking must also work
  // while recovery is in flight, without submitting another story command.
  let releaseRetry;
  const retryGate = new Promise(resolve=>{releaseRetry=resolve;});
  await page.route('**/api/v1/mission-learning/commands',async route=>{
    assert.equal(route.request().postData(),wrongBody);
    const response=await route.fetch();
    await retryGate;
    await route.fulfill({response});
  },{times:1});
  const request=page.waitForRequest(r=>r.url().endsWith('/mission-learning/commands'));
  await page.reload();
  await page.locator('#game canvas').waitFor();
  assert.equal((await request).postData(),wrongBody);
  await walkWhileSaving();
  releaseRetry();
  await page.waitForFunction(profile=>sessionStorage.getItem(`wonderwilds.mission-learning.pending.v1:${profile}`)===null,bootstrap.profileId);
  await page.locator('#missions').click();
  await page.getByRole('button',{name:'Resume the mission task',exact:true}).click();
  await page.locator(`[data-activity-variant="${replaced.variantId}"]`).waitFor();
  assert.deepEqual((await learningSnapshot()).activeActivity,replaced,'Retry must not issue another question');
  assert.equal(await page.locator('.ww-mission-task input:checked').count(),0);
  assert.equal(await page.getByRole('button',{name:'Check my idea',exact:true}).isDisabled(),true);
  await page.setViewportSize({width:390,height:844});
  await page.screenshot({path:'/tmp/wonderwilds-mission-learning-mobile.png',fullPage:true});
  const correct=replaced.task.choices.find(c=>c.value===replaced.task.objects.length);
  await page.locator(`input[value="${correct.id}"]`).check();
  await page.getByRole('button',{name:'Check my idea',exact:true}).click();
  await page.locator('dialog').waitFor({state:'hidden'});
  const done=await learningSnapshot();
  assert.equal(done.skills.find(s=>s.skillId==='M03').activityCompleted,true);
  assert.equal(done.skills.find(s=>s.skillId==='M03').correctChecks,0,'A corrected answer must not become independent evidence');
  assert.notEqual(done.frontierSkillId,'M01'); assert.notEqual(done.frontierSkillId,'M03');
  currentStory=await storySnapshot();
  const home=currentStory.sceneInput.availableActions.find(a=>a.action.type==='travel' && a.action.destinationId==='bridge_bank');
  await sendStory(currentStory,{type:'choose',actionId:home.id},'travel-home');
  await page.reload(); await page.locator('#game canvas').waitFor();
  await page.locator('#missions').click();
  await page.locator('[data-story-visit="object:bridge"]').click();
  await page.locator('[data-story-action="assemble"]').click();
  await page.locator('.ww-mission-activity').waitFor();
  const third=(await learningSnapshot()).activeActivity;
  assert.ok(!['M01','M03'].includes(third.skillId));
  // Travel changes story revisions while the original step remains legal.
  // A correct answer must finish the activity and apply that same action once.
  let moved = await storySnapshot();
  const away = moved.sceneInput.availableActions.find(a=>a.action.type==='travel' && a.action.destinationId==='workshop');
  moved = await sendStory(moved,{type:'choose',actionId:away.id},'stale-task-away');
  const back = moved.sceneInput.availableActions.find(a=>a.action.type==='travel' && a.action.destinationId==='bridge_bank');
  moved = await sendStory(moved,{type:'choose',actionId:back.id},'stale-task-back');
  const target = third.task.promptMode === 'more'
    ? Math.max(...third.task.collections.map(c=>c.items.length))
    : Math.min(...third.task.collections.map(c=>c.items.length));
  const group = third.task.collections.find(c=>c.items.length===target);
  await page.locator(`input[value="${group.id}"]`).check();
  await page.getByRole('button',{name:'Check my idea',exact:true}).click();
  await page.locator('dialog').waitFor({state:'hidden'});
  assert.equal((await learningSnapshot()).skills.find(s=>s.skillId===third.skillId).activityCompleted,true);
  assert.equal((await storySnapshot()).revision,moved.revision+1);
  assert.equal(await page.locator('#game').getAttribute('data-story-bridge'),'repaired');
  await page.reload(); await page.locator('#game canvas').waitFor();
  const restored=await learningSnapshot();
  assert.equal(restored.activeActivity,undefined);
  assert.ok(!['M01','M03',third.skillId].includes(restored.frontierSkillId));
  assert.deepEqual(await get('/api/v1/game'),originalGame);
  assert.equal(await page.evaluate(()=>localStorage.getItem('starwhisker.meadow.v1')),guestSave);
  assert.deepEqual(errors,[]);
  console.log('PASS: completed activities close automatically and stay completed; incorrect answers get a new same-skill variant; exact retry/reload preserve the issued question and story atomicity.');

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
