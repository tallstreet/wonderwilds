// Requires a fresh isolated world QA database behind Vite port 5180.
import assert from 'node:assert/strict';
import { chromium } from 'playwright';
const origin = process.env.WW_DEMO_TEST_URL ?? 'http://127.0.0.1:5180';
const browser = await chromium.launch({executablePath:process.env.CHROMIUM_PATH});
const page = await browser.newPage({viewport:{width:1280,height:800}});
const errors=[]; page.on('pageerror',e=>errors.push(e.message));
const guestSave='{"keep":"original guest bytes"}';
await page.addInitScript(value => { if(!localStorage.getItem('starwhisker.meadow.v1')) localStorage.setItem('starwhisker.meadow.v1',value); },guestSave);
const click=name=>page.getByRole('button',{name,exact:true}).click();
const snapshot=()=>page.evaluate(async()=>{
  const session=await (await fetch('/__demo/bootstrap')).json();
  return (await fetch('/api/v1/game',{headers:{'X-Wonderwilds-Profile':session.profileId}})).json();
});
const activity=()=>page.getByRole('heading',{name:'Help with an adventure'}).waitFor();
const start=async id=>{
  await page.locator('#missions').click();
  await page.locator(`[data-server-visit="${id}"]`).click();
  await activity();
};
const check=async()=>{await click('Check my answer'); await page.locator('#server-return').waitFor(); await page.locator('#server-return').click();};
try {
  await page.goto(origin);
  await page.locator('#game canvas').waitFor();
  await page.getByText(/Local server · progress saved/).waitFor();
  assert.equal((await snapshot()).revision,0);
  await page.locator('#missions').click();
  assert.equal(await page.locator('[data-server-visit]').count(),10);
  assert.equal(await page.locator('[data-server-visit="meadow-bridge"]').isDisabled(),true);
  await click('Close dialog');
  // Simulate uncertain transport: retry must preserve the exact command ID/body.
  let failedBody;
  await page.route('**/api/v1/commands',async route=>{ failedBody=route.request().postData(); await route.abort(); });
  await page.locator('#missions').click(); await page.locator('[data-server-visit="meadow-welcome"]').click();
  await click('Retry saving').catch(()=>{});
  await page.unroute('**/api/v1/commands');
  const retryRequest=page.waitForRequest(r=>r.url().endsWith('/api/v1/commands'));
  await click('Retry saving');
  assert.equal((await retryRequest).postData(),failedBody); await activity();
  await page.keyboard.press('Escape');
  await page.locator('dialog').waitFor({state:'hidden'});
  assert.equal((await snapshot()).pendingEncounters.length,0);
  await start('meadow-welcome');
  await click('Choose: Put the cup inside the box.'); await click('Check my answer');
  await page.getByText('Not quite yet. Try another idea, or ask for a little help.',{exact:true}).waitFor();
  await click('Show a hint'); await page.getByText(/This help is recorded as support/).waitFor();
  await page.reload(); await page.locator('#game canvas').waitFor();
  await page.locator('#missions').click(); await click('Continue your activity'); await activity();
  assert.equal((await snapshot()).pendingEncounters[0].support.hintCount,1);
  await click('Choose: Put the cup beside the tree.'); await check();
  for(const id of ['meadow-bridge','meadow-picnic','meadow-garden','meadow-letter','meadow-finale','befriend-pip','befriend-tansy','befriend-nib','befriend-flutter']) {
    await start(id);
    const encounter=(await snapshot()).pendingEncounters[0],p=encounter.parameters;
    if(encounter.templateId==='missing_quantity') for(let i=0;i<p.total-(p.given??0);i++) await click('Add one object');
    else if(encounter.templateId==='equal_sharing') { for(let f=1;f<=p.recipients;f++) for(let i=0;i<p.total/p.recipients;i++) await click(`Give one object to friend ${f}`); }
    else if(p.instructionId==='count-next') await click(`Choose: ${Number(p.objects[p.prefixLength-1].label)+1}`);
    else if(p.instructionId==='pattern-next') await click(`Choose: ${p.objects[p.prefixLength%p.unitLength].label}`);
    else if(p.instructionId==='sort-round') {for(const o of p.objects.filter(o=>o.attribute==='round')) await click(`Choose: ${o.label}`);}
    else if(p.promptId==='sound-s') await click('Choose tile 1: s');
    else if(p.promptId==='seed-order') {for(const label of ['Pip finds a seed.','Pip plants the seed.','Pip waters the seed.']) await click(`Choose: ${label}`);}
    else throw new Error(JSON.stringify(p));
    await check();
    console.log(`World path and completion: ${id}`);
  }
  await page.locator('#journal').click(); await click('Walk with Flutter');
  await page.getByRole('button',{name:'Your companion',exact:true}).waitFor();
  assert.deepEqual((await snapshot()).selectedCompanions,['flutter']);
  await click('Close dialog');
  await page.keyboard.down('ArrowLeft'); await page.waitForTimeout(800); await page.keyboard.up('ArrowLeft');
  await page.reload(); await page.locator('#game canvas').waitFor();
  await page.locator('#journal').click(); await page.getByRole('button',{name:'Your companion',exact:true}).waitFor();
  await click('Close dialog');
  await page.screenshot({path:new URL('../../.local-demo/world-complete.png',import.meta.url).pathname});
  await page.locator('#grownup').click();
  assert.match(await page.locator('dialog').innerText(),/server|Server/);
  assert.equal(await page.evaluate(()=>localStorage.getItem('starwhisker.meadow.v1')),guestSave);
  await click('Close dialog');
  await page.setViewportSize({width:768,height:1024});
  await page.locator('#missions').click();
  assert.ok(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth));
  await page.screenshot({path:new URL('../../.local-demo/world-tablet.png',import.meta.url).pathname});
  assert.deepEqual(errors,[]);
  console.log('PASS: actual walking to all ten activities, bridge crossing, error retry, Escape abandonment, reload resume, selected companion persistence, guest save isolation and tablet layout');
} finally {await browser.close();}
