// A light woodcutter hint and real placement mechanics from the reported state.
// Always run through run-training-qa.mjs against its fresh disposable database.
import assert from 'node:assert/strict';
import { randomUUID } from 'node:crypto';
import { chromium } from 'playwright';
import { finishInlineStep } from './inline-step.mjs';
import { waitForStoryVisit } from './story-visit.mjs';
const origin = process.env.WW_DEMO_TEST_URL ?? 'http://127.0.0.1:5180';
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH });
const page = await browser.newPage({ viewport: { width: 390, height: 844 } });
const errors = [];
page.on('pageerror', error => errors.push(error.message));
try {
  await page.goto(origin); await page.locator('#game canvas').waitFor();
  const b = await (await page.request.get(`${origin}/__demo/bootstrap`)).json();
  const headers = { 'X-Wonderwilds-Profile': b.profileId, 'X-CSRF-Token': b.csrf, Origin: origin };
  const snapshot = async () => (await page.request.get(`${origin}/api/v1/story`, { headers })).json();
  assert.equal((await snapshot()).revision, 0, 'fresh synthetic profile only');
  const seed = async (type, field, expected) => {
    const s = await snapshot();
    const a = s.sceneInput.availableActions.find(a => a.action.type === type && (!field || a.action[field] === expected));
    assert.ok(a, `${type} ${field} ${expected}`);
    const response = await page.request.post(`${origin}/api/v1/story/commands`, { headers, data: { schemaVersion: '1.0', commandId: randomUUID(), expectedRevision: s.revision, payload: { type: 'choose', actionId: a.id } } });
    assert.equal(response.status(), 200);
  };
  for (const args of [
    ['accept_mission'], ['collect','sourceId','fallen_wood'], ['travel','destinationId','workshop'], ['exchange','recipeId','wood_to_planks'], ['travel','destinationId','bridge_bank'], ['assemble','objectId','bridge'], ['accompany','destinationId','far_bank'],
    ['accept_mission'], ['collect','sourceId','berry_bush'], ['travel','destinationId','bridge_bank'], ['travel','destinationId','workshop'], ['exchange','recipeId','berries_to_picnic'], ['travel','destinationId','bridge_bank'], ['assemble','objectId','picnic_table'], ['travel','destinationId','far_bank'], ['accompany','destinationId','bridge_bank'],
    ['accept_mission'], ['travel','destinationId','far_bank'], ['collect','sourceId','glow_stones'],
  ]) await seed(...args);
  await page.reload(); await page.locator('#game canvas').waitFor();
  const sync = async () => {
    const s = await snapshot();
    await page.waitForFunction(r => document.querySelector('#game')?.dataset.storyRevision === String(r), s.revision);
    return s;
  };
  await sync();
  assert.match(await page.locator('#mission-status').textContent(), /Find the woodcutter/);
  const close = async () => { if (await page.locator('dialog[open]').count()) await page.getByRole('button', { name: 'Close dialog', exact: true }).click(); };
  const action = async type => {
    const before = (await snapshot()).revision;
    await page.locator(`[data-story-action="${type}"]`).click();
    await finishInlineStep(page, type);
    await page.waitForFunction(r => Number(document.querySelector('#game')?.dataset.storyRevision) > r, before);
    return sync();
  };
  const follow = async target => {
    await close(); await page.locator('#missions').click();
    await page.locator(`[data-story-visit="${target}"]`).click();
    await waitForStoryVisit(page, target);
    if (target.startsWith('route:') && (await snapshot()).locationId !== target.slice(6)) await action('travel');
    await sync();
  };
  await page.locator('#missions').click();
  assert.match(await page.locator('[data-story-hint]').textContent(), /Find the woodcutter/);
  assert.doesNotMatch(await page.locator('[data-story-hint]').textContent(), /workshop|cross|placement/i);
  assert.equal(await page.locator('[data-story-guide]').count(), 0);
  await page.screenshot({ path: '/tmp/wonderwilds-lantern-hint-phone.png' });
  await follow('route:bridge_bank');
  await follow('route:workshop');
  await follow('npc:woodcutter');
  if (await page.locator('[data-story-continue]').count()) {
    await page.locator('[data-story-continue]').click();
    await page.locator('[data-story-action="exchange"]').waitFor();
  }
  await action('exchange');
  assert.ok((await snapshot()).sceneInput.context.inventory.some(i => i.itemId === 'lanterns' && i.quantity === 3));
  await follow('route:bridge_bank');
  await follow('object:near_lanterns'); await action('assemble');
  assert.ok((await snapshot()).sceneInput.context.inventory.some(i => i.itemId === 'lanterns' && i.quantity === 1));
  await follow('route:far_bank');
  await follow('object:far_lantern'); await action('assemble');
  assert.equal((await snapshot()).sceneInput.mission.status, 'completed');
  await page.reload(); await page.locator('#game canvas').waitFor(); await sync();
  assert.equal((await snapshot()).sceneInput.mission.status, 'completed');
  assert.deepEqual(errors, []);
  console.log('PASS: phone woodcutter hint leaves route/placement discovery open; real crafting, both placements, supported activities and reload complete successfully');
} finally { await browser.close(); }
