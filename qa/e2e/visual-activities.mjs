import assert from 'node:assert/strict';
import { chromium } from 'playwright';
const browser=await chromium.launch({executablePath:process.env.CHROMIUM_PATH});
const page=await browser.newPage({viewport:{width:1280,height:900},hasTouch:true});
const errors=[]; page.on('pageerror',e=>errors.push(e.message));
try {
 await page.route('http://127.0.0.1:5173/visual-test',route=>route.fulfill({contentType:'text/html',body:'<!doctype html><html><head><title>Visual activity QA</title></head><body></body></html>'}));
 await page.goto('http://127.0.0.1:5173/visual-test');
 await page.evaluate(async()=>{
  await import('/src/style.css');await import('/src/redesign.css');await import('/src/fullscreen.css');
  const {mountActivity}=await import('/src/learning/activity.ts');
  const {authoredPresentation}=await import('/src/learning/presentation.ts');
  document.body.style.background='url(/assets/clover-meadow-v2.png) center / cover fixed';
  window.mountVisual=(templateId,parameters)=>{
   window.activity?.destroy();document.body.replaceChildren(); window.answers=[];
   const dialog=document.createElement('dialog');dialog.id='dialog';document.body.append(dialog);dialog.showModal();
   const encounter={schemaVersion:'1.0',id:'visual-qa',templateId,parameters,skillId:'M06',contentVersion:'draft',permittedActions:['attempt','hint','worked_example','abandon'],support:{hintCount:0,workedExample:false,readAloud:false}};
   window.activity=mountActivity(dialog,encounter,{onAttempt:async answer=>{window.answers.push(answer);return {message:'Checked by the server.'};},onSupport:async()=>({message:'Try one piece at a time.'}),onAbandon:async()=>{}},authoredPresentation(encounter));
  };
 });
 const mount=(...args)=>page.evaluate(args=>window.mountVisual(...args),args);
 const click=name=>page.getByRole('button',{name,exact:true}).click();
 const answer=async()=>{await click('Check my answer');return page.evaluate(()=>window.answers.at(-1));};
 const drag=async(source,target,touch=false)=>{
  const a=await source.boundingBox(), b=await target.boundingBox();assert.ok(a&&b);
  const x=a.x+a.width/2,y=a.y+a.height/2,tx=b.x+b.width/2,ty=b.y+b.height/2;
  if(touch) {
   const cdp=await page.context().newCDPSession(page);
   await cdp.send('Input.dispatchTouchEvent',{type:'touchStart',touchPoints:[{x,y}]});
   for(let i=1;i<=8;i++)await cdp.send('Input.dispatchTouchEvent',{type:'touchMove',touchPoints:[{x:x+(tx-x)*i/8,y:y+(ty-y)*i/8}]});
   await cdp.send('Input.dispatchTouchEvent',{type:'touchEnd',touchPoints:[]});await cdp.detach();
  } else {await page.mouse.move(x,y);await page.mouse.down();await page.mouse.move(tx,ty,{steps:12});await page.mouse.up();}
 };
 await mount('missing_quantity',{total:8,given:3});
 await drag(page.getByRole('button',{name:'Place plank 1',exact:true}),page.locator('.ww-river'));
 assert.deepEqual(await answer(),{kind:'quantity',value:1});
 await drag(page.getByRole('button',{name:'Return plank 1',exact:true}),page.locator('.ww-plank-supply'));
 assert.deepEqual(await answer(),{kind:'quantity',value:0});
 await drag(page.getByRole('button',{name:'Place plank 2',exact:true}),page.locator('.ww-river'),true);
 assert.deepEqual(await answer(),{kind:'quantity',value:1});
 await click('Place plank 3');assert.deepEqual(await answer(),{kind:'quantity',value:2});
 await page.screenshot({path:new URL('../../.local-demo/visual-bridge.png',import.meta.url).pathname});
 await mount('equal_sharing',{total:6,recipients:2});
 await drag(page.getByRole('button',{name:'Berry 1',exact:true}),page.locator('.ww-friend-plate').nth(0),true);
 await click('Berry 2');await page.locator('.ww-plate-objects').nth(1).click();
 assert.deepEqual(await answer(),{kind:'groups',groups:[1,1]});
 await drag(page.getByRole('button',{name:'Berry 1',exact:true}),page.locator('.ww-friend-plate').nth(1));
 assert.deepEqual(await answer(),{kind:'groups',groups:[0,2]});
 await click('Berry 1');assert.deepEqual(await answer(),{kind:'groups',groups:[0,1]});
 await page.screenshot({path:new URL('../../.local-demo/visual-picnic.png',import.meta.url).pathname});
 await mount('story_message',{cards:[{id:'plant_seed',text:'Pip plants the seed.'},{id:'water_seed',text:'Pip waters the seed.'},{id:'find_seed',text:'Pip finds a seed.'}],promptId:'seed-order'});
 await drag(page.getByRole('button',{name:'Choose: Pip waters the seed.',exact:true}),page.locator('.ww-answer-slot').nth(0));
 await drag(page.getByRole('button',{name:'Choose: Pip plants the seed.',exact:true}),page.locator('.ww-answer-slot').nth(0));
 assert.deepEqual(await answer(),{kind:'choices',choiceIds:['plant_seed','water_seed']});
 await drag(page.locator('.ww-answer-slot').nth(0),page.locator('.ww-answer-slot').nth(1));
 assert.deepEqual(await answer(),{kind:'choices',choiceIds:['water_seed','plant_seed']});
 await page.screenshot({path:new URL('../../.local-demo/visual-story.png',import.meta.url).pathname});
 await page.setViewportSize({width:390,height:844});
 await mount('equal_sharing',{total:6,recipients:2});
 assert.ok(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth));
 await page.screenshot({path:new URL('../../.local-demo/visual-mobile.png',import.meta.url).pathname});
 assert.deepEqual(errors,[]);
 console.log('PASS: real mouse and touch drags, tap placement, return to supply, bowl-to-bowl transfer, ordered card insertion/reordering, mobile width, and exact answer payloads.');
}finally{await browser.close();}
