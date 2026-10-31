// Test-only private fixtures. Never bundled into the browser or exposed by the API.
import { readFile } from 'node:fs/promises';
const bank=JSON.parse(await readFile(new URL('../../content/activities/skill-exercises-v1.json',import.meta.url)));
export function fixtureAnswer(activity){
 const skill=bank.skills.find(s=>s.skillId===activity.skillId);
 const exercise=skill.exercises.find(e=>e.id===activity.task.exerciseId);
 const seed=Number(activity.variantId.split('-').at(-1));
 const item=exercise.items[Math.floor(seed/skill.exercises.length)%exercise.items.length];
 const answer=structuredClone(item.answer);const interaction=activity.task.interaction;
 if(answer?.rule==='equal')return {kind:'assign',assignments:interaction.items.map((o,i)=>({objectId:o.id,targetId:interaction.targets[i%interaction.targets.length].id}))};
 const remap=id=>interaction.options.find(o=>o.label===item.interaction.options.find(o=>o.id===id).label).id;
 if(answer?.kind==='order'){const used=new Set();answer.itemIds=answer.itemIds.map(id=>{const label=item.interaction.items.find(i=>i.id===id).label;const result=interaction.items.find(i=>i.label===label&&!used.has(i.id)).id;used.add(result);return result;});}
 if(answer?.kind==='choice')answer.choiceId=remap(answer.choiceId);
 if(answer?.kind==='choices')answer.choiceIds=answer.choiceIds.map(remap);
 return answer;
}
export async function enterAnswer(page,activity,answer=fixtureAnswer(activity)){
 const root=page.locator('.ww-question');const interaction=activity.task.interaction;
 if(answer.kind==='numbers'){for(const value of answer.values)await root.getByLabel(interaction.fields.find(f=>f.id===value.fieldId).label,{exact:false}).fill(value.value);}
 else if(answer.kind==='text')await root.getByLabel(interaction.label).fill(answer.value);
 else if(answer.kind==='choice')await root.locator(`input[value="${answer.choiceId}"]`).check();
 else if(answer.kind==='choices'){for(const id of answer.choiceIds)await root.locator(`input[value="${id}"]`).check();}
 else if(answer.kind==='assign'){for(const {objectId,targetId} of answer.assignments){await root.locator('.ww-question-piece').nth(interaction.items.findIndex(i=>i.id===objectId)).click();await root.locator('.ww-question-place').nth(interaction.targets.findIndex(i=>i.id===targetId)).click();}}
 else if(answer.kind==='order'){for(const [index,id]of answer.itemIds.entries()){await root.locator('.ww-question-piece').nth(interaction.items.findIndex(i=>i.id===id)).click();await root.locator('.ww-question-place').nth(index).click();}}
}
