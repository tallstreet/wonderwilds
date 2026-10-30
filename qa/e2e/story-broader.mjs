// Seven stories, including clues, lasting choices, a new escort and parallel preparations.
// Run only through run-training-qa.mjs with its fresh disposable database.
import assert from 'node:assert/strict';
import { chromium } from 'playwright';
const origin = process.env.WW_DEMO_TEST_URL ?? 'http://127.0.0.1:5180';
const alternate = process.env.WW_STORY_BRANCHES === 'alternate';
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH });
const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
const errors = [];
page.on('pageerror', error => errors.push(error.message));
try {
  await page.goto(`${origin}/?story=1`);
  await page.locator('#game canvas').waitFor();
  const bootstrap = await (await page.request.get(`${origin}/__demo/bootstrap`)).json();
  const get = async path => {
    const response = await page.request.get(`${origin}${path}`, { headers: { 'X-Wonderwilds-Profile': bootstrap.profileId } });
    assert.equal(response.status(), 200); return response.json();
  };
  const snapshot = () => get('/api/v1/story');
  const originalGame = await get('/api/v1/game');
  assert.equal((await snapshot()).revision, 0, 'Use a fresh disposable QA profile');
  const close = async () => { if (await page.locator('dialog[open]').count()) await page.getByRole('button', { name: 'Close dialog', exact: true }).click(); };
  const ready = async () => {
    const state = await snapshot();
    await page.waitForFunction(revision => document.querySelector('#game')?.dataset.storyRevision === String(revision), state.revision);
    await page.waitForFunction(() => document.querySelector('#game')?.dataset.storyCrossing === 'false');
    return state;
  };
  const findAction = (state, type, fields) => {
    const matches = state.sceneInput.availableActions.filter(({ action }) => action.type === type && Object.entries(fields).every(([key, value]) => action[key] === value));
    assert.equal(matches.length, 1, `${type} ${JSON.stringify(fields)} must bind one legal action: ${JSON.stringify(state.sceneInput.availableActions)}`);
    return matches[0];
  };
  const markerFor = action => {
    if (['accept_mission', 'cancel_mission'].includes(action.type)) return 'npc:pip';
    if (['talk', 'invite', 'exchange', 'accompany'].includes(action.type)) return `npc:${action.characterId}`;
    if (action.type === 'travel') return `route:${action.destinationId}`;
    if (action.type === 'collect') return `resource:${action.sourceId}`;
    return `object:${action.type === 'choose' ? action.choiceId : action.objectId}`;
  };
  const visit = async id => {
    await close(); await page.locator('#missions').click();
    await page.locator(`[data-story-visit="${id}"]`).click();
    await page.locator('dialog[open] [data-story-action], dialog[open] [data-story-continue]').first().waitFor({ timeout: 20000 });
    const marker = JSON.parse(await page.locator('#game').getAttribute('data-story-markers')).find(item => item.id === id);
    const point = await page.locator('#game').evaluate(node => ({ x: Number(node.dataset.storyPlayerX), y: Number(node.dataset.storyPlayerY) }));
    assert.ok(Math.hypot(point.x - marker.x, point.y - marker.y) < 110, `${id} requires actual walking`);
  };
  const act = async (type, fields = {}) => {
    let state = await ready();
    const target = markerFor(findAction(state, type, fields).action);
    await visit(target);
    if (await page.locator('[data-story-continue]').count()) {
      await page.locator('[data-story-continue]').click();
      await page.waitForFunction(previous => Number(document.querySelector('#game')?.dataset.storyRevision) > previous, state.revision);
      state = await ready();
      await visit(target);
    }
    const selected = findAction(state, type, fields);
    await page.locator(`[data-story-handle="${selected.id}"]`).click();
    if (type === 'accompany') {
      await page.waitForFunction(character => {
        const surface = document.querySelector('#game');
        return surface?.dataset.storyCrossing === 'true' && JSON.parse(surface.dataset.storyFollowers ?? '[]').some(item => item.sprite === character);
      }, fields.characterId);
      assert.ok(!JSON.parse(await page.locator('#game').getAttribute('data-story-markers')).some(item => item.id === `npc:${fields.characterId}`), 'No stationary duplicate during escort');
    }
    await page.waitForFunction(previous => Number(document.querySelector('#game')?.dataset.storyRevision) > previous, state.revision, { timeout: 20000 });
    return ready();
  };
  const travel = destinationId => act('travel', { destinationId });
  const talk = (characterId, topicId) => act('talk', { characterId, topicId });
  const inspect = objectId => act('inspect', { objectId });
  const assemble = objectId => act('assemble', { objectId });
  const reload = async () => { await close(); await page.reload(); await page.locator('#game canvas').waitFor(); return ready(); };
  const mission = async (id, status = 'completed') => {
    const state = await ready();
    assert.equal(state.sceneInput.mission.missionId, id); assert.equal(state.sceneInput.mission.status, status);
    return state;
  };
  const visuals = async () => JSON.parse(await page.locator('#game').getAttribute('data-story-visuals'));
  const fact = async text => assert.ok((await snapshot()).sceneInput.context.memory.facts.includes(text), text);

  // Original trilogy establishes persistent world state without touching education.
  await act('accept_mission'); await act('collect', { sourceId: 'fallen_wood' });
  await travel('workshop'); await act('exchange', { characterId: 'woodcutter' });
  await travel('bridge_bank'); await assemble('bridge');
  await act('accompany', { characterId: 'pip', destinationId: 'far_bank' });
  await mission('pips_bridge');
  await act('accept_mission'); await act('collect', { sourceId: 'berry_bush' });
  await travel('bridge_bank'); await travel('workshop'); await act('exchange', { characterId: 'woodcutter' });
  await travel('bridge_bank'); await assemble('picnic_table'); await travel('far_bank');
  await act('accompany', { characterId: 'pip', destinationId: 'bridge_bank' }); await mission('pips_picnic');
  await act('accept_mission'); await travel('far_bank'); await act('collect', { sourceId: 'glow_stones' });
  await travel('bridge_bank'); await travel('workshop'); await act('exchange', { characterId: 'woodcutter' });
  await travel('bridge_bank'); await assemble('near_lanterns'); await travel('far_bank'); await assemble('far_lantern');
  await mission('welcome_lights'); await travel('bridge_bank');

  // Clues and a lasting choice. An optional incorrect lead never blocks discovery.
  await act('accept_mission'); await mission('humming_hollow', 'active');
  await talk('pip', 'humming_hollow'); await travel('far_bank');
  await inspect('humming_hollow'); await inspect('trembling_leaf');
  if (alternate) await inspect('mossy_stump');
  await inspect('hollow_branch');
  assert.match(await page.locator('[data-story-dialogue]').innerText(), /wind|breeze/i, 'Discovery is shown immediately after inspection');
  assert.match(await page.locator('[data-story-dialogue]').innerText(), /branch|hum/i);
  const instrumentHome = alternate ? 'gathering' : 'woods';
  await act('choose', { choiceId: 'instrument_home', optionId: instrumentHome });
  assert.ok(!(await visuals()).some(item => item.id === 'wind_instrument'), 'Choosing a home does not teleport the instrument');
  if (alternate) await travel('bridge_bank');
  await assemble('wind_instrument');
  await reload();
  const instrument = (await visuals()).find(item => item.id === 'wind_instrument');
  assert.equal(instrument.x > 780, !alternate);
  if (!alternate) await travel('bridge_bank');
  await talk('pip', 'share_discovery'); await mission('humming_hollow');

  // Choice-dependent design and a reload while paused after one preparation.
  await act('accept_mission'); await talk('pip', 'cosy_preferences');
  assert.match(await page.locator('[data-story-dialogue]').innerText(), /stream/i, 'Pip’s preference reply remains visible after talking');
  await inspect('corner_site');
  const cornerStyle = alternate ? 'star_canopy' : 'leafy_nook';
  await act('choose', { choiceId: 'corner_style', optionId: cornerStyle });
  assert.ok(!(await visuals()).some(item => item.id === 'cosy_corner'), 'Choosing a style is not a completed corner');
  assert.equal(JSON.parse(await page.locator('#game').getAttribute('data-story-markers')).find(item => item.id === 'object:corner_frame').shape, alternate ? 'corner-stars' : 'corner-leaves');
  await assemble('corner_frame'); await act('cancel_mission');
  await reload(); await mission('cosy_corner', 'abandoned');
  assert.ok(!(await snapshot()).sceneInput.availableActions.some(item => item.action.type === 'assemble'));
  await act('accept_mission');
  assert.equal(JSON.parse(await page.locator('#game').getAttribute('data-story-markers')).find(item => item.id === 'object:corner_cushions').shape, alternate ? 'corner-stars' : 'corner-leaves');
  await assemble('corner_cushions');
  await talk('pip', 'enjoy_corner'); await mission('cosy_corner');
  await fact(`Pip's cosy corner uses ${cornerStyle}.`);
  assert.equal((await visuals()).find(item => item.id === 'cosy_corner').kind, alternate ? 'corner-stars' : 'corner-leaves');

  // Bramble is a separate present NPC; readiness and a cleared path precede escort.
  await act('accept_mission'); await travel('workshop');
  const workshopMarkers = JSON.parse(await page.locator('#game').getAttribute('data-story-markers'));
  assert.notEqual(workshopMarkers.find(item => item.id === 'npc:bramble').sprite, workshopMarkers.find(item => item.id === 'npc:woodcutter').sprite);
  await talk('bramble', 'visit_wish'); await inspect('workshop_path');
  assert.ok(!(await snapshot()).sceneInput.availableActions.some(item => item.action.type === 'accompany'));
  await assemble('workshop_path'); await fact('The workshop path is clear.');
  assert.ok((await visuals()).some(item => item.id === 'workshop_path'));
  await talk('bramble', 'ready_to_visit');
  await act('accompany', { characterId: 'bramble', destinationId: 'bridge_bank' });
  await fact('Bramble is at bridge_bank.'); await talk('bramble', 'visit_arrival'); await mission('brambles_visit');
  await reload();
  assert.equal(JSON.parse(await page.locator('#game').getAttribute('data-story-markers')).filter(item => item.id === 'npc:bramble').length, 1);

  // Both locations and opposite preparation orders across the two QA runs.
  await act('accept_mission'); await act('choose', { choiceId: 'gathering_place', optionId: alternate ? 'far_bank' : 'near_bank' });
  if (alternate) {
    await act('invite', { characterId: 'pip' }); await act('invite', { characterId: 'bramble' }); await travel('far_bank');
    await assemble('gathering_seats'); await reload(); await assemble('gathering_music');
  } else {
    await assemble('gathering_music'); await act('invite', { characterId: 'bramble' });
    await assemble('gathering_seats'); await reload(); await act('invite', { characterId: 'pip' });
  }
  await talk('pip', 'welcome_friends');
  const final = await mission('moonlight_gathering');
  await fact('The moonlight gathering has begun.');
  assert.equal(final.sceneInput.context.memory.recentMissions.length, 7);
  assert.ok(!final.sceneInput.availableActions.some(item => item.action.type === 'accept_mission'));
  const before = await visuals();
  for (const id of ['picnic_table', 'near_lantern_1', 'near_lantern_2', 'far_lantern', 'wind_instrument', 'cosy_corner', 'gathering_seats', 'gathering_music', 'moonlight_gathering'])
    assert.ok(before.some(item => item.id === id), `${id} remains in the world`);
  assert.ok(before.filter(item => item.id.startsWith('gathering_') || item.id === 'moonlight_gathering').every(item => (item.x > 780) === alternate));
  assert.deepEqual(final.sceneInput.here.characters.map(item => item.id).sort(), ['bramble', 'pip']);
  await close();
  await page.screenshot({ path: `/tmp/wonderwilds-story-broader-${alternate ? 'alternate' : 'default'}-desktop.png`, fullPage: true });
  await reload(); assert.deepEqual(await visuals(), before);
  await page.setViewportSize({ width: 390, height: 844 }); await page.locator('#missions').click();
  assert.equal(await page.locator('[data-story-history] li').count(), 7);
  assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth));
  await page.screenshot({ path: `/tmp/wonderwilds-story-broader-${alternate ? 'alternate' : 'default'}-mobile.png`, fullPage: true });
  assert.deepEqual(await get('/api/v1/game'), originalGame, 'Education state remains separate');
  assert.deepEqual(errors, []);
  console.log(`PASS: seven playable stories (${alternate ? 'alternate' : 'default'} choices), clues, lasting designs, Bramble escort, parallel gathering, pause/reload, visible outcomes, mobile history and separate education`);
} catch (error) {
  await page.screenshot({ path: '/tmp/wonderwilds-story-broader-failure.png', fullPage: true }).catch(() => {});
  throw error;
} finally { await browser.close(); }
