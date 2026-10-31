#!/usr/bin/env python3
"""Build the internal deterministic exercise bank from original authoring modules."""
import json,sys,hashlib
from pathlib import Path
from activity_content import number,operations,fractions,measurement,data,english,refine
from activity_content.common import SKILLS
ROOT=Path(__file__).resolve().parents[1]
for module in [number,operations,fractions,measurement,data,english,refine]:module.build()
nodes=[n for p in ['content/curriculum/seed-v1.json','content/curriculum/years/seed-years1to6-v1.json'] for n in json.loads((ROOT/p).read_text())['nodes']]
assert set(SKILLS)=={n['id'] for n in nodes},(set(n['id'] for n in nodes)-set(SKILLS),set(SKILLS)-set(n['id'] for n in nodes))
# Attach permitted curriculum representation and preserve pending review gates.
for n in nodes:
 for exercise in SKILLS[n['id']]:
  exercise['difficultyRungs']=['introductory','standard','stretch'] if len(exercise['items'])>=12 else ['introductory','standard']
  exercise['assessmentMode']='human_review' if all(i['answer'] is None for i in exercise['items']) else 'automatic'
  exercise['limitations']=['Adult observation required; not automatically assessed.'] if exercise['assessmentMode']=='human_review' else []
  if n['id'] in ['E01','E03','E04','E05','E11','EY107']:exercise['limitations'].append('Reviewed audio unavailable; use the human-supported route.')
  for index,item in enumerate(exercise['items']):
   if item['representation'] not in n['allowedRepresentations']:item['representation']=n['allowedRepresentations'][0]
   item['title']={'M01':'A gift for every friend','M03':'Count a little collection','M05':'One little step','M07':'A few more berries','MY110':'Time to explore','EY106':'A word for this','E13':'You’re invited, Pip!'}.get(n['id'],exercise['facet'].capitalize())
# Remove repeated source items. Smaller finite domains remain explicit in coverage.
for exercises in SKILLS.values():
 for exercise in exercises:
  for item in exercise['items']:item['semanticId']=hashlib.sha256(json.dumps([item['prompt'],item['interaction'],item['answer'],item['stimuli']],sort_keys=True).encode()).hexdigest()[:20]
  seen=set();exercise['items']=[i for i in exercise['items'] if not (i['semanticId'] in seen or seen.add(i['semanticId']))]
value={'schemaVersion':'1.0','reviewStatus':'pending','releaseEnabled':False,'skills':[{'skillId':n['id'],'exercises':SKILLS[n['id']]} for n in nodes]}
path=ROOT/'content/activities/skill-exercises-v1.json'
# One authored item per line keeps generated banks reviewable without excessive whitespace.
lines=['{', '  "schemaVersion": "1.0",', '  "reviewStatus": "pending",', '  "releaseEnabled": false,', '  "skills": [']
for skill_index,skill in enumerate(value['skills']):
 lines.append('    {"skillId": '+json.dumps(skill['skillId'])+', "exercises": [')
 for exercise_index,exercise in enumerate(skill['exercises']):
  metadata={k:v for k,v in exercise.items() if k!='items'}
  lines.append('      '+json.dumps(metadata,ensure_ascii=False)[:-1]+', "items": [')
  lines.extend('        '+json.dumps(item,ensure_ascii=False,separators=(',', ':'))+(',' if i<len(exercise['items'])-1 else '') for i,item in enumerate(exercise['items']))
  lines.append('      ]}'+(',' if exercise_index<len(skill['exercises'])-1 else ''))
 lines.append('    ]}'+(',' if skill_index<len(value['skills'])-1 else ''))
lines.extend(['  ]','}']);body='\n'.join(lines)+'\n' 
if '--check' in sys.argv:
 assert path.read_text()==body,'Exercise bank drift: run python3 scripts/build-activities.py'
else:path.write_text(body)
print(f'{len(SKILLS)} skills, {sum(len(v) for v in SKILLS.values())} exercise families, {sum(len(e["items"]) for v in SKILLS.values() for e in v)} authored questions')
