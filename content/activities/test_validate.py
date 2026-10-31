import copy,json,unittest
from pathlib import Path
from validate import errors_for
BANK=json.loads((Path(__file__).parent/'skill-exercises-v1.json').read_text())
class ActivityValidation(unittest.TestCase):
 def test_complete_inventory(self):self.assertEqual(errors_for(BANK),[])
 def test_missing_skill(self):
  bank=copy.deepcopy(BANK);bank['skills'].pop();self.assertTrue(errors_for(bank))
 def test_release_gate(self):
  bank=copy.deepcopy(BANK);bank['releaseEnabled']=True;self.assertTrue(errors_for(bank))
 def test_duplicate_semantics(self):
  bank=copy.deepcopy(BANK);items=bank['skills'][0]['exercises'][0]['items'];items.append(copy.deepcopy(items[0]));self.assertTrue(errors_for(bank))
 def test_unassessed_workshop(self):
  bank=copy.deepcopy(BANK);item=next(i for s in bank['skills'] for e in s['exercises'] for i in e['items'] if i['interaction']['kind']=='workshop');item['answer']={'kind':'text','value':'approved'};self.assertTrue(errors_for(bank))
 def test_duplicate_choice(self):
  bank=copy.deepcopy(BANK);item=next(i for s in bank['skills'] for e in s['exercises'] for i in e['items'] if i['interaction']['kind']=='single_choice');item['interaction']['options'][1]['label']=item['interaction']['options'][0]['label'];self.assertTrue(errors_for(bank))
 def test_arithmetic_keys_against_prompt(self):
  import re
  from fractions import Fraction
  count=0
  for skill in BANK['skills']:
   for exercise in skill['exercises']:
    for item in exercise['items']:
     match=re.fullmatch(r'(\d+(?:/\d+)?)\s*([+−×÷])\s*(\d+(?:/\d+)?)\s*=\s*\?',item['prompt'])
     if not match or item['interaction']['kind']!='number_entry' or len(item['answer']['values'])!=1:continue
     a,operator,b=match.groups();a,b=Fraction(a),Fraction(b)
     expected={'+':lambda:a+b,'−':lambda:a-b,'×':lambda:a*b,'÷':lambda:a/b}[operator]()
     self.assertEqual(Fraction(item['answer']['values'][0]['value']),expected,exercise['id']);count+=1
  self.assertGreater(count,200)
 def test_one_more_nine(self):
  skill=next(s for s in BANK['skills'] if s['skillId']=='M05');exercise=next(e for e in skill['exercises'] if e['id']=='M05.one_more');item=next(i for i in exercise['items'] if i['prompt']=='What number is one more than 9?')
  self.assertEqual(next(o['label'] for o in item['interaction']['options'] if o['id']==item['answer']['choiceId']),'10')
 def test_all_clock_keys_match_hands(self):
  for skill in BANK['skills']:
   if skill['skillId'] not in ['MY110','MY207']:continue
   for exercise in skill['exercises']:
    if not exercise['id'].endswith('.read_clock'):continue
    for item in exercise['items']:
     clock=item['stimuli'][0];correct=next(o for o in item['interaction']['options'] if o['id']==item['answer']['choiceId'])
     self.assertEqual(correct['label'],f"{clock['hour']}:{clock['minute']:02}")
 def test_addition_bounds(self):
  import re
  skill=next(s for s in BANK['skills'] if s['skillId']=='MY104')
  for exercise in skill['exercises']:
   for item in exercise['items']:
    equation=item['stimuli'][0]['text'];a,b=map(int,re.findall(r'\d+',equation)[:2]);self.assertLessEqual(a+b,20)
 def test_unreviewed_audio_is_not_assessed(self):
  bank=copy.deepcopy(BANK);item=bank['skills'][0]['exercises'][0]['items'][0];item['stimuli'].append({'kind':'audio','assetId':'ww-audio-missing'})
  self.assertTrue(errors_for(bank))
