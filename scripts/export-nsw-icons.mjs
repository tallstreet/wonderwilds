// Build-time allowlist only: no unrestricted icon names enter the learner payload.
import * as icons from '../frontend/node_modules/@fortawesome/free-solid-svg-icons/index.mjs';
import fs from 'node:fs';
const root = new URL('../', import.meta.url);
const source = JSON.parse(fs.readFileSync(new URL('content/curriculum/nsw/source-v1.json', root)));
const names = new Set(['circle', 'apple-whole', 'star', 'fish', 'leaf', 'cube', 'seedling']);
for (const skill of source.skills) for (const item of skill.practice_items)
  for (const name of item.visual_enhancement?.icon_names ?? []) names.add(name);
const result = {};
for (const name of names) {
  const icon = Object.values(icons).find(x => x?.iconName === name);
  if (icon) result[name] = {width:icon.icon[0],height:icon.icon[1],paths:typeof icon.icon[4]==='string'?[icon.icon[4]]:icon.icon[4]};
}
const path = new URL('content/curriculum/nsw/icons-v1.json', root);
const text = JSON.stringify(result,null,2)+'\n';
if (process.argv.includes('--check')) {
  if(fs.readFileSync(path,'utf8')!==text) throw new Error('Font Awesome allowlist drift');
} else fs.writeFileSync(path,text);
console.log(`${Object.keys(result).length} verified Font Awesome Free icons; unavailable suggestions omitted`);
