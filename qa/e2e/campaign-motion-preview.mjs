// Technical renderer fixture only: no canonical ownership, XP or reward is fabricated.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { chromium } from 'playwright';
const base = process.env.WW_CAMPAIGN_ART_URL ?? 'http://127.0.0.1:5185';
const prefix = process.argv[2] ?? '/tmp/campaign-pip-motion';
const character = process.argv[3] ?? 'pip';
const registry = JSON.parse(readFileSync(new URL('../../content/campaign/art/motion-registry.v1.json',import.meta.url),'utf8'));
const selectedSheet = registry.sheets.find(sheet => sheet.character === character);
assert.ok(selectedSheet, 'Measured atlas exists for '+character);
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH ?? '/usr/bin/chromium' });
const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
const errors = [];
page.on('pageerror', error => errors.push(error.message));
await page.route('**/campaign-motion-fixture', route => route.fulfill({ contentType: 'text/html', body: '<html><body style="margin:0"><div id="game" style="width:100vw;height:100vh"></div></body></html>' }));
try {
  await page.goto(`${base}/campaign-motion-fixture`);
  await page.evaluate(async (character) => {
    const { createWorld } = await import('/src/world.ts');
    const state = { version: 1, region: 'meadow', hero: 'clover', companionName: '', done: [], evidence: [], position: { x: 540, y: 530 } };
    window.fixture = createWorld('game', () => state, () => {}, () => {}, undefined, undefined, {
      encounters: () => [], companions: () => [{sprite:character,name:'Motion fixture'}], allowExits: false,
    });
  }, character);
  await page.locator('#game canvas').waitFor();
  await page.waitForFunction(() => window.fixture.scene.getScenes(true)[0]?.player);
  const inspected = await page.evaluate(async (campaignAnimationSheet) => {
    const { campaignAnimationFrame, applyCampaignAnimation, startCampaignPose } = await import('/src/campaign/animation.ts');
    const scene = window.fixture.scene.getScenes(true)[0];
    scene.player.setVisible(false);
    scene.player.shadow.setVisible(false);
    for(const follower of scene.followers){follower.actor.setVisible(false);follower.actor.shadow.setVisible(false);follower.label?.setVisible(false);}
    for (const child of scene.children.list) if (child.text === 'YOU') child.setVisible(false);
    const frames = campaignAnimationSheet.frames;
    const labels = [];
    const actors = frames.map((frame, index) => {
      const x = 220 + (index % 5) * 145, y = 190 + Math.floor(index / 5) * 145;
      const actor = scene.makeActor(x, y, campaignAnimationSheet.character, 84, 94);
      labels.push(scene.add.text(x, y + 12, frame.id, { fontFamily: 'sans-serif', fontSize: '13px', color: '#183d35', backgroundColor: '#fff8de' }).setOrigin(0.5, 0).setDepth(y + 101));
      return actor;
    });
    const render = () => {
      frames.forEach((frame, index) => {
        const actor = actors[index];
        const walking = frame.id.startsWith('walk-');
        const facing = frame.id.includes('-right') ? 'right' : frame.id.includes('-left') ? 'left' : frame.id.includes('-up') ? 'up' : 'down';
        actor.facing = facing;
        actor.phase = frame.id.endsWith('-b') ? 0.55 : 0.05;
        const pose = frame.id.startsWith('signature') ? 'signature' : frame.id.startsWith('cheer') ? 'cheer' : frame.id.startsWith('rest') ? 'rest' : undefined;
        const elapsed = { 'signature-anticipate': 0, 'signature-release': 300, 'signature-settle': 600, 'cheer-prepare': 0, 'cheer-lift': 400, 'cheer-settle': 700, 'rest-settle': 0, 'rest-asleep': 1200 }[frame.id] ?? 0;
        if (pose) startCampaignPose(actor, pose, scene.time.now - elapsed);
        const dx = facing === 'left' ? -1 : facing === 'right' ? 1 : 0;
        const dy = facing === 'up' ? -1 : facing === 'down' ? 1 : 0;
        const applied = applyCampaignAnimation(actor, { dx, dy, distance: walking ? 0.1 : 0, now: scene.time.now, reducedMotion: false });
        if (!applied || actor.art.frame.name !== frame.id || actor.art.flipX) throw new Error(`Native frame mismatch: ${frame.id}/${actor.art.frame.name}`);
        actor.setDepth(actor.y);
      });
    };
    render();
    scene.events.on('postupdate', render);
    window.previewActors = actors;
    window.previewLabels = labels;
    window.previewRender = render;
    return { frames: frames.length, texture: actors[0].art.texture.key,
      reduced: ['down','right','up','left'].map(f => campaignAnimationFrame(f, 0.6, true, true)) };
  }, selectedSheet);
  assert.equal(inspected.frames, 20);
  assert.equal(inspected.texture, selectedSheet.texture);
  assert.deepEqual(inspected.reduced, ['idle-down','idle-right','idle-up','idle-left']);
  await page.waitForTimeout(250);
  await page.screenshot({ path: `${prefix}-phaser.png` });
  await page.setViewportSize({ width: 390, height: 844 });
  await page.waitForTimeout(300);
  for (let group = 0; group < 4; group++) {
    await page.evaluate((group) => {
      window.previewActors.forEach((actor, index) => {
        const shown = Math.floor(index / 5) === group;
        actor.setVisible(shown); actor.shadow.setVisible(shown);
        window.previewLabels[index].setVisible(shown);
        if (shown) {
          const slot = index % 5, x = 450 + (slot % 2) * 140, y = 240 + Math.floor(slot / 2) * 170;
          actor.setPosition(x,y);actor.shadow.setPosition(x,y+1);
          window.previewLabels[index].setPosition(x,y+12).setDepth(y+101);
        }
      });
    }, group);
    await page.waitForTimeout(100);
    await page.screenshot({ path: `${prefix}-phone-group${group+1}.png` });
  }
  const animation = await page.evaluate(async () => {
    const { startCampaignPose } = await import('/src/campaign/animation.ts');
    const scene = window.fixture.scene.getScenes(true)[0];
    scene.events.off('postupdate', window.previewRender);
    const actor = scene.followers[0].actor;
    actor.facing = 'down';
    const seen = new Set();
    scene.game.events.emit('campaign-pose', actor.heroId.replace(/-wayfinder$/, ''), 'signature');
    const tick = () => { scene.poseActor(actor, 0, 0, 0); seen.add(String(actor.art.frame.name)); };
    scene.events.on('postupdate', tick);
    await new Promise(resolve => window.setTimeout(resolve, 850));
    scene.events.off('postupdate', tick);
    return [...seen];
  });
  for (const frame of ['signature-anticipate','signature-release','signature-settle','idle-down']) assert.ok(animation.includes(frame), `Timed actual-renderer frame ${frame}`);
  assert.deepEqual(errors, []);
  console.log(character+' actual Phaser motion: twenty native frame/anchor registrations, four distinct directions, walk/signature/cheer/rest poses and reduced-motion still selection passed. Synthetic visual fixture only.');
} finally { await browser.close(); }
