"""Validate original internal questions, public projections and honest assessment routes."""
import json,re,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
def errors_for(bank):
 errors=[]
 nodes={n['id']:n for p in ['content/curriculum/seed-v1.json','content/curriculum/years/seed-years1to6-v1.json'] for n in json.loads((ROOT/p).read_text())['nodes']}
 ids=[s['skillId'] for s in bank['skills']]
 if len(ids)!=len(set(ids)) or set(ids)!=set(nodes):errors.append('Skill inventory must match all 120 unique graph nodes')
 audio=json.loads((ROOT/'content/activities/audio-v1.json').read_text())
 approved={a['id'] for a in audio['assets'] if a['reviewStatus']=='reviewed' and (ROOT/'frontend/public'/a['path'].lstrip('/')).is_file()}
 if bank.get('reviewStatus')!='pending' or bank.get('releaseEnabled') is not False:errors.append('Review/release gates changed')
 all_exercises=set()
 for skill in bank['skills']:
  for exercise in skill['exercises']:
   identity=exercise['id'];items=exercise['items']
   if identity in all_exercises or not identity.startswith(skill['skillId']+'.'):errors.append(identity+': invalid exercise identity')
   all_exercises.add(identity)
   if not items:errors.append(identity+': empty bank')
   if len({item['semanticId'] for item in items})!=len(items):errors.append(identity+': repeated semantic item')
   for item in items:
    if item['representation'] not in nodes[skill['skillId']]['allowedRepresentations']:errors.append(identity+': unsupported representation')
    if len(item['hints'])<2 or not item['explanation']:errors.append(identity+': missing teaching')
    interaction=item['interaction'];answer=item['answer'];kind=interaction['kind']
    if (kind=='workshop')!=(answer is None):errors.append(identity+': dishonest assessment mode')
    if kind=='workshop' and (len(interaction['rubric'])<2 or not exercise['limitations']):errors.append(identity+': missing review rubric/limitation')
    if kind in ['single_choice','multi_choice']:
     options=interaction['options'];allowed={o['id'] for o in options}
     if len(allowed)!=len(options) or len({o['label'] for o in options})!=len(options):errors.append(identity+': duplicate choices')
     selected=[answer['choiceId']] if kind=='single_choice' else answer['choiceIds']
     if not set(selected)<=allowed or len(selected)!=len(set(selected)):errors.append(identity+': invalid answer IDs')
    if kind=='assign' and answer.get('rule')=='equal' and len(interaction['items'])%len(interaction['targets']):errors.append(identity+': indivisible sharing')
    for stimulus in item['stimuli']:
     if stimulus['kind']=='audio' and stimulus['assetId'] not in approved and answer is not None:errors.append(identity+': unreviewed audio cannot be independently assessed')
     if stimulus['kind']=='clock' and not (1<=stimulus['hour']<=12 and 0<=stimulus['minute']<60):errors.append(identity+': invalid clock')
     if stimulus['kind']=='graph' and len(stimulus['labels'])!=len(stimulus['values']):errors.append(identity+': graph category mismatch')
     if stimulus['kind']=='grid' and max(stimulus['x'],stimulus['y'])>stimulus['size']:errors.append(identity+': off-grid point')
     if stimulus['kind']=='passage':
      bounds=nodes[skill['skillId']]['parameterBounds'].get('text_words')
      if bounds and not bounds['min']<=len(stimulus['text'].split())<=bounds['max']:errors.append(identity+': passage length outside node bounds')
 return errors
if __name__=='__main__':
 subprocess.run([sys.executable,str(ROOT/'scripts/build-activities.py'),'--check'],check=True)
 bank=json.loads((ROOT/'content/activities/skill-exercises-v1.json').read_text());errors=errors_for(bank)
 if errors:raise SystemExit('\n'.join(errors))
 print('Activity inventory valid: 120 skills; private answers; pending educator review; release disabled')
