// Renderer-only synthetic visual fixture: this does not establish progression.
import { chromium } from 'playwright';
const ids = (process.argv[2] ?? 'pip,tansy,nib').split(',');
if (ids.length !== 3 || ids.some(id => !/^[a-z]+$/.test(id))) throw new Error('Provide three comma-separated companion IDs');
const outputPrefix = process.argv[3] ?? '/tmp/campaign-wayfinder-phaser-fixture';
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH ?? '/usr/bin/chromium' });
const page = await browser.newPage({ viewport: { width: 1280, height: 800 } });
await page.route('**/campaign-art-fixture', route => route.fulfill({ contentType: 'text/html', body: '<html><body style="margin:0"><div id="game" style="width:100vw;height:100vh"></div></body></html>' }));
try {
  await page.goto('http://127.0.0.1:5185/campaign-art-fixture');
  await page.evaluate(async (ids) => {
    const { createWorld } = await import('/src/world.ts');
    const state = { version: 1, region: 'meadow', hero: 'clover', companionName: '', done: [], evidence: [], position: { x: 540, y: 530 } };
    window.fixture = createWorld('game', () => state, () => {}, () => {}, undefined, undefined, {
      encounters: () => ids.map((id, index) => ({
        id, region: 'meadow', x: [445, 540, 635][index], y: [385, 485, 385][index],
        short: `${id} Wayfinder`, character: id, sprite: `${id}-wayfinder`,
      })), companions: () => [], allowExits: false,
    });
  }, ids);
  await page.locator('#game canvas').waitFor();
  await page.waitForTimeout(1800);
  await page.evaluate(() => {
    // Hide the synthetic avatar so it cannot obscure the inspected art.
    const scene = window.fixture.scene.getScenes(true)[0];
    scene.player.setVisible(false);
    for (const child of scene.children.list) if (child.text === 'YOU') child.setVisible(false);
  });
  await page.screenshot({ path: `${outputPrefix}.png` });
  await page.setViewportSize({ width: 390, height: 844 });
  await page.waitForTimeout(500);
  await page.screenshot({ path: `${outputPrefix}-phone.png` });
} finally { await browser.close(); }
