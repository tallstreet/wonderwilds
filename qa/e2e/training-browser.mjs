// Real API + PostgreSQL journey. Run through run-training-qa.mjs on a disposable database.
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { chromium } from 'playwright';

const origin = process.env.WW_DEMO_TEST_URL ?? 'http://127.0.0.1:5180';
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH || '/usr/bin/chromium', headless: true, args: ['--no-sandbox'] });
const page = await browser.newPage({ viewport: { width: 1280, height: 800 }, reducedMotion: 'reduce' });
const errors = [];
page.setDefaultTimeout(15000);
page.on('pageerror', error => errors.push(error.message));
page.on('response', response => {
  if (response.url().includes('/api/') && response.status() >= 400)
    console.error('API failure', response.status(), response.url());
});
try {
  await page.goto(origin);
  await page.waitForSelector('#game canvas');
  const bootstrap = await page.evaluate(async () => (await fetch('/__demo/bootstrap')).json());
  const headers = { 'X-Wonderwilds-Profile': bootstrap.profileId, 'X-CSRF-Token': bootstrap.csrf, Origin: origin };
  const snapshot = async () => {
    const response = await page.request.get(`${origin}/api/v1/game`, { headers });
    assert.equal(response.status(), 200);
    return response.json();
  };
  const command = async payload => {
    const state = await snapshot();
    const body = { schemaVersion: '1.0', commandId: crypto.randomUUID(), expectedRevision: state.revision, payload };
    const response = await page.request.post(`${origin}/api/v1/commands`, { headers, data: body });
    assert.equal(response.status(), 200, `Command ${payload.type}: ${await response.text()}`);
    return { state: await response.json(), body };
  };
  const progression = (state, creatureId) => state.characterProgression.find(row => row.creatureId === creatureId);
  assert.equal((await snapshot()).revision, 0, 'This test requires a fresh disposable QA database');
  // Arrange the existing story through the real HTTP assessor; no database progress injection.
  const story = JSON.parse(await readFile(new URL('../../content/story/adventure.v1.json', import.meta.url)));
  const friends = JSON.parse(await readFile(new URL('../../content/story/help-quests.v1.json', import.meta.url)));
  const answerFor = encounter => {
    const p = encounter.parameters;
    if (encounter.templateId === 'missing_quantity') return { kind: 'quantity', value: p.total - p.given };
    if (encounter.templateId === 'equal_sharing') return { kind: 'groups', groups: Array(p.recipients).fill(p.total / p.recipients) };
    let ids;
    if (p.instructionId === 'count-next') ids = [p.objects.find(object => object.attribute === 'choice' && Number(object.label) === Number(p.objects[p.prefixLength - 1].label) + 1).id];
    else if (p.instructionId === 'pattern-next') ids = [p.objects.find(object => object.attribute === 'choice' && object.label === p.objects[p.prefixLength % p.unitLength].label).id];
    else if (p.instructionId === 'sort-round') ids = p.objects.filter(object => object.attribute === 'round').map(object => object.id);
    else if (p.promptId === 'sound-s') ids = [String(p.tiles.indexOf('s'))];
    else if (p.promptId === 'seed-order') ids = ['find_seed', 'plant_seed', 'water_seed'];
    else if (p.promptId === 'listen-direction') ids = [p.cards.find(card => card.text === 'Put the cup beside the tree.').id];
    else throw new Error(`Unsupported test task ${JSON.stringify(p)}`);
    return { kind: 'choices', choiceIds: ids };
  };
  for (const quest of [...story.beats.filter(beat => beat.chapterId === 'meadow'), ...friends.quests.filter(quest => ['pip', 'tansy', 'nib', 'flutter'].includes(quest.creatureId))]) {
    const started = await command({ type: 'start_encounter', beatId: quest.id });
    const encounter = started.state.pendingEncounters[0];
    const completed = await command({ type: 'attempt', encounterId: encounter.id, answer: answerFor(encounter) });
    assert.equal(completed.state.pendingEncounters.length, 0, `${quest.id} completed through the assessor`);
  }
  assert.deepEqual([...(await snapshot()).ownedCreatures].sort(), ['flutter', 'nib', 'pip', 'tansy']);
  console.log('Recruitment through real assessment passed');
  await page.reload();
  await page.waitForSelector('#game canvas');
  const click = name => page.getByRole('button', { name, exact: true }).click();
  const journal = async () => { await click('My journal'); await page.getByRole('heading', { name: 'My adventure journal' }).waitFor(); };
  await journal();
  await page.getByRole('button', { name: 'Walk with Pip', exact: true }).focus();
  await page.keyboard.press('Enter');
  await page.getByRole('button', { name: 'Rest Pip', exact: true }).waitFor();
  console.log('Keyboard companion selection passed');
  assert.ok((await snapshot()).selectedCompanions.includes('pip'));
  const pipBefore = progression(await snapshot(), 'pip');
  assert.equal(pipBefore.level, 1);
  assert.equal(pipBefore.xp, 0);
  await page.locator('[data-server-training="training-pip-1"]').focus();
  await page.keyboard.press('Enter');
  await page.getByRole('heading', { name: 'Training with Pip', exact: true }).waitFor();
  const firstEncounter = (await snapshot()).pendingEncounters[0];
  assert.equal(firstEncounter.training.creatureId, 'pip');
  // A wrong answer must leave XP at zero and keep the encounter open.
  await click('Check my answer');
  await page.getByText(/Not quite yet/).waitFor();
  assert.equal(progression(await snapshot(), 'pip').xp, 0);
  await click('Show a hint');
  await page.getByText(/This help is recorded as support/).waitFor();
  await page.reload();
  await page.waitForSelector('#game canvas');
  await page.locator('#missions').click();
  await click('Continue your activity');
  await page.getByRole('heading', { name: 'Training with Pip', exact: true }).waitFor();
  const resumed = (await snapshot()).pendingEncounters[0];
  assert.equal(resumed.id, firstEncounter.id);
  assert.equal(resumed.support.hintCount, 1);
  await click('Return to exploring');
  // Closing follows the server acknowledgement; a click alone does not await the command.
  await page.locator('#dialog').waitFor({ state: 'hidden' });
  assert.equal((await snapshot()).pendingEncounters.length, 0);
  assert.equal(progression(await snapshot(), 'pip').xp, 0);
  await journal();
  await page.locator('[data-server-training="training-pip-1"]').click();
  await page.getByRole('heading', { name: 'Training with Pip', exact: true }).waitFor();
  await click('Show a hint');
  await page.getByText(/This help is recorded as support/).waitFor();
  const training = (await snapshot()).pendingEncounters[0];
  assert.equal(training.templateId, 'missing_quantity');
  for (let i = 0; i < training.parameters.total - training.parameters.given; i++) await click('Add one object');
  // Capture the exact completion request, then replay it after reload.
  const requestPromise = page.waitForRequest(request => request.url().endsWith('/api/v1/commands') && request.method() === 'POST' && request.postDataJSON().payload.type === 'attempt');
  await click('Check my answer');
  const completedBody = (await requestPromise).postDataJSON();
  await page.getByRole('heading', { name: 'Pip reached level 2!', exact: true }).waitFor();
  assert.ok(await page.locator('.training-celebration').isVisible());
  const earned = progression(await snapshot(), 'pip');
  assert.equal(earned.level, 2);
  assert.ok(earned.xp > 0);
  assert.ok(earned.reactionId);
  await page.reload();
  await page.waitForSelector('#game canvas');
  await journal();
  assert.match(await page.locator('[data-character-level="pip"]').textContent(), /Level 2/);
  assert.deepEqual(progression(await snapshot(), 'pip'), earned);
  const retry = await page.request.post(`${origin}/api/v1/commands`, { headers, data: completedBody });
  assert.equal(retry.status(), 200);
  assert.deepEqual(progression(await snapshot(), 'pip'), earned, 'Exact command retry grants no extra XP');
  // A fresh command may practise the same milestone, but earns no additional XP.
  const practice = await command({ type: 'start_encounter', beatId: 'training-pip-1' });
  await command({ type: 'attempt', encounterId: practice.state.pendingEncounters[0].id, answer: answerFor(practice.state.pendingEncounters[0]) });
  assert.deepEqual(progression(await snapshot(), 'pip'), earned, 'Repeated training cannot farm XP');
  await page.reload();
  await page.waitForSelector('#game canvas');
  await page.setViewportSize({ width: 390, height: 844 });
  await journal();
  assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth));
  assert.ok(await page.locator('[data-character-level="pip"]').isVisible());
  await page.locator('[data-server-training="training-tansy-1"]').click();
  await page.getByRole('heading', { name: 'Training with Tansy', exact: true }).waitFor();
  const tansyEncounter = (await snapshot()).pendingEncounters[0];
  // Complete through HTTP after proving the mobile training UI mounts. Authority is identical.
  await command({ type: 'attempt', encounterId: tansyEncounter.id, answer: answerFor(tansyEncounter) });
  const tansy = progression(await snapshot(), 'tansy');
  assert.equal(tansy.level, 2);
  assert.equal(tansy.xp, earned.xp, 'Independent and supported first training earn the same XP');
  assert.deepEqual(errors, []);
  console.log('PASS: real-API recruitment, keyboard companion/training, wrong answer, recorded support, reload/resume, abandonment, supported level-two reaction, persisted XP, exact retry, repeat-practice reward prevention and mobile training');
} catch (error) {
  console.error('Browser errors:', errors);
  console.error('Current dialog:', await page.locator('#dialog').innerText().catch(() => 'unavailable'));
  await page.screenshot({ path: '/tmp/wonderwilds-training-failure.png', fullPage: true }).catch(() => {});
  throw error;
} finally { await browser.close(); }
