// Browser integration with a synthetic API fixture; Rust tests cover authority/persistence.
import assert from 'node:assert/strict';
import { chromium } from 'playwright';
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH || '/usr/bin/chromium', headless: true, args: ['--no-sandbox'] });
try {
  for (const [name, width, height] of [['desktop',1280,800], ['tablet',768,1024]]) {
    const page = await browser.newPage({ viewport: { width,height } });
    const errors = [];
    page.on('pageerror', e => errors.push(e.message));
    const state = { schemaVersion:'1.0',revision:1,contentVersion:'wonderwilds-story-1.0.0',rulesVersion:'test',storyFlags:['meadow_light'],ownedCreatures:['pip'],inventory:[],pendingEncounters:[],learnerSummaries:[],selectedCompanions:['pip'],friendship:[{creatureId:'pip',points:2}],transformedCreatures:[],availableHelpingMoments:[{creatureId:'pip',completedQuestId:'meadow-bridge'}] };
    await page.route('**/__demo/bootstrap', r => r.fulfill({ json:{profileId:'synthetic-profile',csrf:'synthetic-csrf'} }));
    await page.route('**/api/v1/game', r => r.fulfill({json:state}));
    const commands = [];
    await page.route('**/api/v1/commands', async route => {
      const command = route.request().postDataJSON();
      assert.equal(command.expectedRevision,state.revision);
      commands.push(command.payload);
      if(command.payload.type === 'help_friend') {
        state.friendship[0].points = 3;
        state.availableHelpingMoments = [];
      } else if(command.payload.type === 'transform_companion') {
        state.transformedCreatures = command.payload.enabled ? ['pip-glow'] : [];
      } else throw new Error('Unexpected command');
      state.revision++;
      await route.fulfill({json:state});
    });
    await page.goto(process.env.WONDERWILDS_TEST_URL || 'http://127.0.0.1:5174/');
    await page.waitForSelector('#game canvas');
    await page.getByRole('button',{name:'My journal'}).click();
    await page.getByText('Meet Pip',{exact:true}).click();
    await page.getByRole('button',{name:/Remember together:/}).focus();
    await page.keyboard.press('Enter');
    await page.getByText('Friendship: 3 / 6',{exact:true}).waitFor({state:'visible'});
    assert.equal(await page.locator('.companion-details[open]').count(),1);
    assert.equal(await page.evaluate(() => document.activeElement?.textContent),'Meet Pip');
    await page.getByRole('button',{name:'Turn glow on'}).click();
    await page.getByText('Lantern glow is on',{exact:true}).waitFor({state:'visible'});
    assert.equal(await page.evaluate(() => document.activeElement?.textContent),'Turn glow off');
    await page.reload();
    await page.waitForSelector('#game canvas');
    await page.getByRole('button',{name:'My journal'}).click();
    await page.getByText('Meet Pip',{exact:true}).click();
    await page.getByRole('button',{name:'Turn glow off'}).click();
    await page.getByRole('button',{name:'Turn glow on'}).waitFor({state:'visible'});
    assert.deepEqual(commands.map(x=>x.type),['help_friend','transform_companion','transform_companion']);
    assert.deepEqual(errors,[]);
    await page.close();
    console.log(`${name}: shared memory, glow on, reload and glow off passed (synthetic API)`);
  }
} finally { await browser.close(); }
