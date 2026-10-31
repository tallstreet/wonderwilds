"""Original Wonderwilds exercise authoring helpers; answer keys never enter public assets."""
from fractions import Fraction
import hashlib,json
SKILLS={}
def text(value):return {'kind':'text','text':value}
def table(headings,rows):return {'kind':'table','headings':headings,'rows':[[str(v) for v in r] for r in rows]}
def coll(counts,labels=None,layout='rows'):return {'kind':'collection','counts':counts,'labels':labels or [f'Group {i+1}' for i in range(len(counts))],'layout':layout}
def pic(name,description=None):return {'kind':'picture','name':name,'description':description or name.capitalize()}
def item(prompt, interaction, answer, stimuli=(), hint='Look carefully at the information in the question.', explanation=None, representation='text', title='A little discovery'):
 result={'title':title,'prompt':prompt,'representation':representation,'stimuli':list(stimuli),'interaction':interaction,'answer':answer,'hints':[hint, 'Work through one part at a time. '+hint], 'explanation':explanation or hint}
 result['semanticId']=hashlib.sha256(json.dumps([prompt,interaction,answer,stimuli],sort_keys=True).encode()).hexdigest()[:20]
 return result
def choice(prompt, correct, wrong, stimuli=(),hint='Compare each option with the information shown.',explanation=None,representation='text',pictures=False):
 labels=list(dict.fromkeys(str(v) for v in [correct,*wrong]))[:4]
 assert len(labels)>=2,(prompt,labels)
 options=[{'id':f'option-{i}','label':v,**({'picture':v} if pictures else {})} for i,v in enumerate(labels)]
 return item(prompt,{'kind':'single_choice','options':options},{'kind':'choice','choiceId':'option-0'},stimuli,hint,explanation or f'The answer is {correct}. {hint}',representation)
def multi(prompt, labels, correct, stimuli=(),hint='Check each option separately.',representation='text',pictures=False):
 options=[{'id':f'option-{i}','label':v,**({'picture':v} if pictures else {})} for i,v in enumerate(labels)]
 return item(prompt,{'kind':'multi_choice','options':options},{'kind':'choices','choiceIds':[f'option-{i}' for i in correct]},stimuli,hint,'The matching options are '+', '.join(labels[i] for i in correct)+'. '+hint,representation)
def number(prompt,value,stimuli=(),hint='Use the model to work out the missing value.',representation='text',unit='',format='integer',fields=None):
 values=[str(value)] if not isinstance(value,list) else [str(v) for v in value]
 fields=fields or [{'id':f'value-{i}','label':'Your answer' if len(values)==1 else ['Whole number','Numerator','Denominator'][i],'format':format,**({'unit':unit} if unit else {})} for i in range(len(values))]
 return item(prompt,{'kind':'number_entry','fields':fields},{'kind':'numbers','values':[{'fieldId':f['id'],'value':v} for f,v in zip(fields,values)]},stimuli,hint,'We find '+', '.join(values)+f'{" "+unit if unit else ""}. '+hint,representation)
def typed(prompt,answer,stimuli=(),hint='Say the sounds and think about the taught spelling pattern.',representation='text'):
 return item(prompt,{'kind':'text_entry','label':'Your word','maxLength':120},{'kind':'text','value':answer},stimuli,hint,f'The word is {answer}. {hint}',representation)
def order(prompt,values,stimuli=(),hint='Choose what comes first, then continue in order.',representation='text',pictures=False):
 cards=[{'id':f'card-{i}','label':str(v),**({'picture':v} if pictures else {})} for i,v in enumerate(values)]
 return item(prompt,{'kind':'order','items':cards[1:]+cards[:1]},{'kind':'order','itemIds':[c['id'] for c in cards]},stimuli,hint,'The order is '+', '.join(str(v) for v in values)+'. '+hint,representation)
def assign(prompt,objects,targets,expected=None,stimuli=(),hint='Move each object to a place, then check every place.',representation='text',capacity=1):
 objects=[{'id':f'object-{i}','label':str(v)} for i,v in enumerate(objects)]
 targets=[{'id':f'target-{i}','label':str(v)} for i,v in enumerate(targets)]
 answer={'rule':'equal'} if expected is None else {'kind':'assign','assignments':[{'objectId':o['id'],'targetId':targets[expected[i]]['id']} for i,o in enumerate(objects)]}
 explanation=(f'Give {len(objects)//len(targets)} objects to each place. ' if expected is None else '; '.join(t['label']+': '+', '.join(o['label'] for i,o in enumerate(objects) if expected[i]==j) for j,t in enumerate(targets)))
 return item(prompt,{'kind':'assign','items':objects,'targets':targets,'capacity':capacity},answer,stimuli,hint,explanation+' '+hint,representation)
def workshop(prompt,scaffold,rubric,stimuli=(),representation='text',hint='Share your thinking with a grown-up.'):
 return item(prompt,{'kind':'workshop','scaffold':scaffold,'rubric':rubric},None,stimuli,hint,'Read or demonstrate your work with a grown-up. '+scaffold,representation)
def add(skill,suffix,items,facet=None):
 assert items,(skill,suffix)
 SKILLS.setdefault(skill,[]).append({'id':f'{skill}.{suffix}','facet':facet or suffix.replace('_',' '),'items':items})
def nums(skill,suffix,fn,count=12):add(skill,suffix,[fn(i) for i in range(count)])
