// Synthetic renderer fixture only: actual Phaser residency, not earned progression.
import assert from 'node:assert/strict';
import { readFile, writeFile } from 'node:fs/promises';
import { chromium } from 'playwright';
const registry = JSON.parse(await readFile(new URL('../../content/campaign/art/motion-registry.v1.json', import.meta.url)));
assert.equal(registry.sheets.length, 24, 'Full native motion residency sweep requires all24 registered sheets');
const origin = process.env.WW_MEMORY_TEST_URL ?? 'http://127.0.0.1:5185';
const pairs = [['pip','tansy'],['nib','flutter'],['cleo','bramble'],['patter','twizzle'],['luna','pebble'],['sail','glimmer']];
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH ?? '/usr/bin/chromium' });
const evidence = { scope: 'Actual Phaser manager in synthetic team/form renderer fixture; no progression claim', estimate: 'sum(source.width * source.height * 4), not real GPU allocation; excludes driver copies/mipmaps/decoded duplicates', viewports: [] };
try {
  for (const viewport of [{ width: 1280, height: 800 }, { width: 390, height: 844 }]) {
    const page = await browser.newPage({ viewport });
    const errors = []; page.on('pageerror', error => errors.push(error.message));
    await page.route('**/campaign-memory-fixture', route => route.fulfill({ contentType: 'text/html', body: '<html><head><meta name="viewport" content="width=device-width,initial-scale=1"></head><body style="margin:0"><div id="game" style="width:100vw;height:100vh"></div></body></html>' }));
    await page.goto(origin + '/campaign-memory-fixture');
    await page.evaluate(async initial => {
      const { createWorld } = await import('/src/world.ts');
      const state = { version: 1, region: 'meadow', hero: 'clover', companionName: '', done: [], evidence: [], position: { x: 540, y: 500 } };
      let team = initial;
      const game = createWorld('game', () => state, () => {}, () => {}, undefined, undefined, { encounters: () => [], companions: () => team.map(id => ({ sprite: id, name: id })), allowExits: false });
      window.memoryFixture = game;
      window.switchMemoryTeam = next => { team = next; game.events.emit('authoritative-position'); };
    }, pairs[0]);
    await page.locator('#game canvas').waitFor();
    const read = () => page.locator('#game').evaluate(node => JSON.parse(node.dataset.textureResidency));
    const samples = [];
    let initialResources;
    const sequence = pairs.flatMap(pair => [pair, pair.map(id => id + '-wayfinder')]).concat([pairs[0], pairs[0].map(id => id + '-wayfinder'), pairs[0], pairs[0].map(id => id + '-wayfinder'), pairs[0]]);
    for (const team of sequence) {
      await page.evaluate(team => window.switchMemoryTeam(team), team);
      await page.waitForFunction(expected => {
        const value = document.querySelector('#game')?.dataset.textureResidency;
        if (!value) return false;
        const stats = JSON.parse(value);
        return expected.every(id => stats.motion.some(texture => texture.key.includes(id.replace('-wayfinder','')) && (id.endsWith('-wayfinder') ? texture.key.includes('wayfinder') : !texture.key.includes('wayfinder'))));
      }, team, { timeout: 20000 });
      await page.waitForTimeout(600);
      const sample = await read();
      sample.sources = await page.evaluate(() => { const scene = window.memoryFixture.scene.getScenes(true)[0]; return scene.textures.getTextureKeys().map(key => ({ key, sources: scene.textures.get(key).source.map(source => [source.width, source.height]) })); });
      assert.ok(sample.motionCount <= 4, `Bounded motion residency for ${team}: ${sample.motionCount}`);
      assert.ok(sample.motionCount >= 2, `Both selected friends have native motion textures for ${team}`);
      samples.push({ team, ...sample });
      if (!initialResources) initialResources = await page.evaluate(() => performance.getEntriesByType('resource').filter(entry => new URL(entry.name).pathname.startsWith('/assets/')).map(entry => ({ path: new URL(entry.name).pathname, transferBytes: entry.transferSize, encodedBytes: entry.encodedBodySize })));
    }
    assert.equal(samples.at(-1).motionRgbaBytes, samples.at(-3).motionRgbaBytes, 'Repeated warmed return cycles do not retain cumulative motion texture memory');
    assert.equal(samples.at(-1).motionCount, samples.at(-3).motionCount, 'Repeated warmed return cycles keep bounded motion texture count');
    assert.equal(samples.at(-1).textureCount, samples.at(-3).textureCount, 'Repeated warmed return cycles leave no orphan texture registrations');
    assert.equal(samples.at(-1).rgbaBytes, samples.at(-3).rgbaBytes, 'Repeated warmed return cycles retain no extra canvas/atlas sources');
    await page.evaluate(() => { window.memorySceneRestarted = false; window.memoryFixture.events.once('world-ready', () => window.memorySceneRestarted = true); window.memoryFixture.scene.stop('island'); window.memoryFixture.scene.start('island'); });
    await page.waitForFunction(() => window.memorySceneRestarted === true);
    await page.waitForTimeout(800);
    const restarted = await read();
    assert.ok(restarted.motionCount <= 4, 'Scene restart retains bounded native textures');
    assert.equal(restarted.textureCount, samples.at(-1).textureCount, 'Scene restart does not leak old label textures');
    const resources = await page.evaluate(() => performance.getEntriesByType('resource').filter(entry => new URL(entry.name).pathname.startsWith('/assets/')).map(entry => ({ path: new URL(entry.name).pathname, transferBytes: entry.transferSize, encodedBytes: entry.encodedBodySize })));
    await page.screenshot({ path: `/tmp/campaign-texture-residency-${viewport.width}.png` });
    evidence.viewports.push({ viewport, afterSceneRestart: restarted, initialAssetTransferBytes: initialResources.reduce((sum,entry) => sum + entry.transferBytes, 0), initialAssetEncodedBytes: initialResources.reduce((sum,entry) => sum + entry.encodedBytes, 0), initialResources, samples, assetTransferBytes: resources.reduce((sum,entry) => sum + entry.transferBytes, 0), assetEncodedBytes: resources.reduce((sum,entry) => sum + entry.encodedBytes, 0), resources });
    assert.deepEqual(errors, []);
    await page.close();
  }
  await writeFile('/tmp/campaign-texture-residency.json', JSON.stringify(evidence, null, 2));
  console.log('PASS actual Phaser texture residency across all twelve friends/base+forms and return cycles, desktop+phone');
} finally { await browser.close(); }
