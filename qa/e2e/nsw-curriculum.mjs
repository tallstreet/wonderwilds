import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { chromium } from 'playwright';
const bank = JSON.parse(await readFile(new URL('../../content/curriculum/nsw/exercises-v1.json', import.meta.url)));
const items = bank.skills.flatMap(s => s.exercises.flatMap(e => e.items));
const examples = [
  items.find(i => i.interaction.kind === 'number_entry' && i.stimuli.some(s => s.kind === 'diagram')),
  items.find(i => i.interaction.options?.some(o => o.diagram)),
  items.find(i => i.interaction.kind === 'multi_choice' && i.interaction.options.length > 6),
  items.find(i => i.interaction.kind === 'workshop'),
];
const origin = process.env.WW_ACTIVITY_TEST_URL ?? 'http://127.0.0.1:5189';
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH });
try {
  for (const width of [1280,390,320]) {
    const page = await browser.newPage({ viewport:{width,height:900} });
    const errors=[]; page.on('pageerror',e=>errors.push(e.message));
    await page.route('**/__nsw_test',r=>r.fulfill({contentType:'text/html',body:'<!doctype html><meta name="viewport" content="width=device-width,initial-scale=1"><style>body{font:18px sans-serif;margin:12px}main{max-width:680px;margin:auto}</style><main></main>'}));
    await page.goto(origin+'/__nsw_test');
    for (const item of examples) {
      // Only the public projection is passed into the browser, even in QA.
      await page.evaluate(async ({interaction,stimuli,prompt})=>{
        window.control?.destroy();
        const main=document.querySelector('main');main.replaceChildren();const root=document.createElement('div');main.append(root);
        const heading=document.createElement('h1');heading.textContent=prompt;root.append(heading);
        const {mountQuestionControl}=await import('/src/mission-learning/question-control.ts');
        window.control=mountQuestionControl(root,interaction,stimuli,()=>true,()=>{});
      },{interaction:item.interaction,stimuli:item.stimuli,prompt:item.prompt});
      await page.waitForFunction(()=>[...document.images].every(i=>i.complete&&i.naturalWidth>0));
      assert.equal(await page.evaluate(()=>[...document.images].every(i=>i.getBoundingClientRect().width>=60&&i.getBoundingClientRect().height>=40)),true);
      assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth),false);
      if(item.interaction.kind==='single_choice'){
        await page.locator('input[type=radio]').first().focus();await page.keyboard.press('Space');
        assert.equal(await page.locator('input[type=radio]').first().isChecked(),true);
      }
      if(item.stimuli.some(s=>s.exposureMs)) await page.waitForFunction(()=>[...document.images].some(i=>getComputedStyle(i).visibility==='hidden'));
      if(item===examples[1])await page.screenshot({path:`/tmp/nsw-curriculum-${width}.png`,fullPage:true});
    }
    assert.deepEqual(errors,[]);await page.close();
  }
  console.log('NSW diagrams, choices, workshop and keyboard/layout checks passed at desktop and phone sizes');
} finally { await browser.close(); }
