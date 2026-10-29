// Real UI + assessor journey. Run only via run-training-qa.mjs with a disposable cluster.
import assert from 'node:assert/strict';
import { chromium } from 'playwright';

if (!process.env.WW_QA_CLUSTER_DIR) throw new Error('Exploration QA requires the disposable PostgreSQL runner');
const origin = process.env.WW_DEMO_TEST_URL ?? 'http://127.0.0.1:5180';
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH || '/usr/bin/chromium', headless: true, args: ['--no-sandbox'] });
const page = await browser.newPage({ viewport: { width: 1120, height: 690 }, reducedMotion: 'reduce' });
page.setDefaultTimeout(20000);
const errors = [];
const commands = [];
page.on('pageerror', error => errors.push(error.message));
page.on('request', request => {
  if (request.url().endsWith('/api/v1/commands') && request.method() === 'POST') commands.push(request.postDataJSON().payload);
});
const click = name => page.getByRole('button', { name, exact: true }).click();
const close = async () => {
  await click('Close dialog');
  await page.locator('#dialog').waitFor({ state: 'hidden' });
};
const location = () => page.locator('.location-badge h1').textContent();
const arrive = name => page.waitForFunction(expected => document.querySelector('.location-badge h1')?.textContent === expected, name);
const guide = () => page.locator('#quest-guide').click();
const dialogue = () => page.locator('#encounter-help').waitFor();
const activity = () => page.getByRole('button', { name: 'Check my answer', exact: true }).waitFor();
try {
  await page.goto(origin);
  await page.locator('#game canvas').waitFor();
  const bootstrap = await page.evaluate(async () => (await fetch('/__demo/bootstrap')).json());
  const snapshot = async () => {
    const response = await page.request.get(`${origin}/api/v1/game`, { headers: { 'X-Wonderwilds-Profile': bootstrap.profileId } });
    assert.equal(response.status(), 200);
    return response.json();
  };
  assert.equal((await snapshot()).revision, 0, 'Fresh synthetic QA profile required');
  const start = async expectedId => {
    await guide();
    await dialogue();
    const before = commands.length;
    await page.locator('#encounter-help').click();
    await activity();
    assert.equal(commands[before].type, 'start_encounter');
    assert.equal(commands[before].beatId, expectedId);
    return (await snapshot()).pendingEncounters[0];
  };
  const solve = async () => {
    const encounter = (await snapshot()).pendingEncounters[0];
    const p = encounter.parameters;
    if (encounter.templateId === 'missing_quantity') {
      for (let i = 0; i < p.total - p.given; i++) await click('Add one object');
    } else if (encounter.templateId === 'equal_sharing') {
      for (let friend = 1; friend <= p.recipients; friend++) {
        for (let item = 0; item < p.total / p.recipients; item++) await click(`Give one object to friend ${friend}`);
      }
    } else if (p.instructionId === 'count-next') {
      await click(`Choose: ${Number(p.objects[p.prefixLength - 1].label) + 1}`);
    } else if (p.instructionId === 'pattern-next') {
      await click(`Choose: ${p.objects[p.prefixLength % p.unitLength].label}`);
    } else if (p.promptId === 'listen-direction') {
      await click('Choose: Put the cup beside the tree.');
    } else throw new Error(`Unhandled exploration activity ${encounter.templateId}`);
    await click('Check my answer');
    await page.locator('#server-return').waitFor();
    assert.equal((await snapshot()).pendingEncounters.length, 0);
    await page.locator('#server-return').click();
    await page.locator('#dialog').waitFor({ state: 'hidden' });
  };
  const followTo = async name => {
    const before = await location();
    await guide();
    // Guiding starts a walk: it must not synchronously teleport to the destination.
    assert.equal(await location(), before);
    await arrive(name);
  };

  // At the initial 1:1 canvas scale, Pip's body is at (205, 450).
  // Exercise direct character input, independently of the guidance button.
  await page.locator('#game canvas').click({ position: { x: 205, y: 450 } });
  await page.locator('#trail-accept').waitFor();
  assert.equal(commands.length, 0, 'Talking must not issue an activity or award progress');
  await page.locator('#encounter-later').click();
  assert.equal((await snapshot()).revision, 0, 'Declining a conversation is not a server action');
  await page.keyboard.press('Space');
  await page.locator('#trail-accept').waitFor();
  await page.locator('#trail-accept').click();
  await dialogue();
  assert.equal(commands.length, 0, 'Accepting only records the local conversation');
  await page.locator('#encounter-help').click();
  await activity();
  const first = (await snapshot()).pendingEncounters[0];
  await click('Choose: Put the cup inside the box.');
  await click('Check my answer');
  await page.getByText(/Not quite yet/).waitFor();
  assert.deepEqual((await snapshot()).storyFlags, []);
  await click('Show a hint');
  await page.getByText(/This help is recorded as support/).waitFor();
  await page.reload();
  await page.locator('#game canvas').waitFor();
  await page.locator('#missions').click();
  assert.match(await page.locator('.quest-trail').innerText(), /1 of 7 steps complete/);
  await click('Continue your activity');
  await activity();
  const resumed = (await snapshot()).pendingEncounters[0];
  assert.equal(resumed.id, first.id);
  assert.equal(resumed.support.hintCount, 1);
  await click('Return to exploring');
  await page.locator('#dialog').waitFor({ state: 'hidden' });
  assert.equal((await snapshot()).pendingEncounters.length, 0);
  assert.deepEqual((await snapshot()).storyFlags, []);
  await start('meadow-welcome');
  await solve();
  await start('meadow-bridge');
  await solve();
  assert.ok((await snapshot()).storyFlags.includes('meadow_bridge_complete'));
  console.log('Dialogue acceptance, wrong answer, support, reload/resume, abandonment and bridge repair passed');

  await page.locator('#missions').click();
  assert.match(await page.locator('.quest-trail').innerText(), /3 of 7 steps complete/);
  const beforeTrack = await location();
  await page.locator('#trail-track').click();
  assert.equal(await location(), beforeTrack, 'Tracking a quest must not teleport');
  await followTo('Starlight Cove');
  await start('meadow-picnic');
  await solve();
  await followTo('Peachy Orchard');
  await start('befriend-tansy');
  await click('Show a hint');
  await page.getByText(/This help is recorded as support/).waitFor();
  await solve();
  assert.ok((await snapshot()).ownedCreatures.includes('tansy'), 'Supported friendship activity still recruits Tansy');
  await start('meadow-garden');
  await solve();
  assert.ok((await snapshot()).storyFlags.includes('meadow_garden_complete'));
  await page.locator('#missions').click();
  assert.match(await page.locator('.quest-trail').innerText(), /6 of 7 steps complete/);
  assert.match(await page.locator('.quest-trail-step.current').innerText(), /Return to Pip/);
  await close();
  console.log('Walked Meadow → Cove → Orchard; shared picnic, befriended Tansy and grew garden through real assessment');

  await followTo('Starlight Cove');
  await followTo('Clover Meadow');
  await guide();
  await page.locator('#trail-finish').waitFor();
  const beforeFinish = (await snapshot()).revision;
  await page.locator('#trail-finish').click();
  assert.match(await page.locator('.quest-trail').innerText(), /7 of 7 steps complete/);
  assert.equal((await snapshot()).revision, beforeFinish, 'Return conversation must not invent an extra reward');
  await page.locator('#trail-done').click();
  await page.reload();
  await page.locator('#game canvas').waitFor();
  await page.setViewportSize({ width: 390, height: 844 });
  await page.locator('#missions').click();
  assert.match(await page.locator('.quest-trail').innerText(), /7 of 7 steps complete/);
  assert.equal(await page.locator('.quest-trail-step.current').count(), 0);
  assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), 'Mobile page must not overflow');
  assert.ok(await page.locator('#dialog').evaluate(node => node.scrollWidth <= node.clientWidth + 1), 'Mobile quest dialog must not overflow');
  await page.screenshot({ path: '/tmp/wonderwilds-exploration-quest-mobile.png', fullPage: true });
  assert.deepEqual(errors, []);
  console.log('PASS: complete quest via dialogue, real walking, server assessment, supported friendship, return conversation, saved checklist and mobile layout');
} catch (error) {
  console.error('Browser errors:', errors);
  console.error('Location:', await location().catch(() => 'unavailable'));
  console.error('Current dialog:', await page.locator('#dialog').innerText().catch(() => 'unavailable'));
  await page.screenshot({ path: '/tmp/wonderwilds-exploration-quest-failure.png', fullPage: true }).catch(() => {});
  throw error;
} finally {
  await browser.close();
}
