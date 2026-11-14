// Real API and journal checks; run only through the disposable training QA runner.
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {chromium} from 'playwright';
const bank=JSON.parse(await readFile(new URL('../../content/curriculum/nsw/exercises-v1.json',import.meta.url)));
const origin=process.env.WW_DEMO_TEST_URL??'http://127.0.0.1:5180';
const browser=await chromium.launch({executablePath:process.env.CHROMIUM_PATH});
try{
 const page=await browser.newPage({viewport:{width:390,height:844}});
 await page.goto(origin);await page.locator('#game canvas').waitFor();
 const bootstrap=await(await page.request.get(origin+'/__demo/bootstrap')).json();
 const headers={'X-Wonderwilds-Profile':bootstrap.profileId,'X-CSRF-Token':bootstrap.csrf,Origin:origin};
 const get=async path=>{const r=await page.request.get(origin+path,{headers});assert.equal(r.status(),200);return r.json();};
 let counter=0;
 const send=async(path,state,payload)=>{
  const data={schemaVersion:'1.0',commandId:`nsw-qa-${++counter}`,expectedRevision:state.revision,payload};
  const r=await page.request.post(origin+path,{headers,data});assert.equal(r.status(),200,await r.text());
  const response=await r.json();
  const retry=await page.request.post(origin+path,{headers,data});assert.deepEqual(await retry.json(),response);
  return response;
 };
 let story=await get('/api/v1/story');assert.equal(story.revision,0,'Fresh synthetic database only');
 const accept=story.sceneInput.availableActions.find(a=>a.action.type==='accept_mission');
 story=await send('/api/v1/story/commands',story,{type:'choose',actionId:accept.id});
 let learning=await get('/api/v1/mission-learning');
 assert.equal(learning.skills.length,736);assert.ok(learning.skills.every(s=>s.learningResources.length>0));
 // Exercise the real bounded browser transport with the full replacement snapshot.
 assert.equal(await page.evaluate(async b=>{
  const {MissionLearningClient}=await import('/src/mission-learning/client.ts');
  const client=new MissionLearningClient(b.profileId,b.csrf);const s=await client.snapshot();client.destroy();return s.skills.length;
 },bootstrap),736);
 const seen=new Set();
 for(let n=0;n<10;n++){
  learning=await send('/api/v1/mission-learning/commands',learning,{type:'begin'});
  const activity=learning.activeActivity;assert.equal(activity.skillId,'M-K-001');assert.equal(seen.has(activity.task.exerciseId),false);seen.add(activity.task.exerciseId);
  const skill=bank.skills.find(s=>s.skillId===activity.skillId);const item=skill.exercises.find(e=>e.id===activity.task.exerciseId).items[0];
  const answer=structuredClone(item.answer);
  for(const key of ['sourceScoring','sourceInput','acceptedSequences','acceptedMappings'])delete answer[key];
  if(answer.kind==='choice'){
   const original=item.interaction.options.find(o=>o.id===answer.choiceId);
   answer.choiceId=activity.task.interaction.options.find(o=>o.label===original.label&&o.diagram===original.diagram).id;
  }
  learning=await send('/api/v1/mission-learning/commands',learning,{type:'answer',encounterId:activity.id,variantId:activity.variantId,answer});
  assert.equal(learning.lastFeedback.result,'correct');
  if(n<9)assert.equal(learning.skills.find(s=>s.skillId==='M-K-001').activityCompleted,false);
 }
 assert.equal(learning.skills.find(s=>s.skillId==='M-K-001').eligibleChecks,10);
 assert.equal(learning.skills.find(s=>s.skillId==='M-K-001').status,'ready_for_next');
 assert.notEqual(learning.frontierSkillId,'M-K-001');
 assert.equal((await get('/api/v1/story')).revision,story.revision,'Practice does not invent story commands');
 await page.reload();await page.locator('#game canvas').waitFor();
 await page.locator('#missions').click();
 await page.getByText('See learning progress',{exact:true}).click();
 await page.getByText('Learning resources',{exact:true}).first().click();
 const link=page.getByRole('link',{name:/Khan Academy/}).first();assert.match(await link.getAttribute('href'),/^https:\/\//);
 assert.equal(await link.getAttribute('rel'),'noopener noreferrer');
 await page.screenshot({path:'/tmp/nsw-live-journal-phone.png',fullPage:true});
 console.log('Live 736-skill transport, ten independent items, progression, exact retries, reload and resource links passed');
}finally{await browser.close();}
