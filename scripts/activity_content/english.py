from .common import *
from .passages import passage
# Icons are identifiers from our fixed Lucide allowlist, never arbitrary remote images.
WORDS=['hat','cup','cat','dog','sun','map']
WORD_PAIRS=[('cat','cap'),('hat','hot'),('cup','cut'),('dog','dot'),('sun','sum'),('map','mat')]
WORKSHOPS={
'E01':('Follow directions and take turns','A grown-up gives one or two short spoken directions. Follow them, then swap turns.',['Follow the spoken steps in order.','Wait, listen and take a turn.']),
'E03':('Listen for rhymes and syllables','A grown-up says two words and asks whether they rhyme, then says a word to clap its syllables.',['Hear the ending sounds rather than compare spelling.','Clap and count the spoken syllables.']),
'E04':('Explore spoken sounds','A grown-up says a taught two- or three-sound word. Move a token for each sound, then change its first sound.',['Identify and segment phonemes rather than letter names.','Blend or change one sound and explain the new word.']),
'E05':('Letters and their sounds','Ask a grown-up to say a taught sound. Point to its letter, then say the sound for another taught letter.',['Use only explicitly taught correspondences.','Distinguish a phoneme from a letter name.']),
'E06':('Blend a little word','Read a printed word made from taught sounds to a grown-up. Blend the sounds, then explain its meaning.',['Use taught two- or three-phoneme words.','Observe decoding; do not substitute word recognition.']),
'E11':('Listen to a short story','Ask a grown-up to read the story. Retell it, name the characters and put two to four events in order.',['Recall an explicitly heard character or detail.','Sequence the events and explain the order.']),
'E13':('Write a useful message','Write a short invitation to a picnic. Include who, what and where.',['The message has a clear meaning and audience.','Compose two to six words with scaffolds as needed.']),
'E14':('Talk about a choice','Pip sees someone sitting alone. Discuss what Pip could do and give a reason.',['State a relevant character choice.','Give a reason; different defensible ideas are welcome.']),
'E15':('Add a little detail','Write a short postcard about a garden visit, then add detail and check a taught spelling pattern.',['Compose four to ten meaningful words.','Explain the added detail and spelling revision.']),
'EY101':('Read a new word','Read a short taught decodable text aloud. Tell a grown-up how you blended a new word.',['Use taught grapheme–phoneme correspondences.','Observe actual oral decoding in three to twelve words.']),
'EY104':('Make a sentence','Write a three- to ten-word sentence, then check the capital letter and end mark.',['Compose an independently meaningful grammatical sentence.','Use a capital and appropriate full stop or question mark.']),
'EY107':('Listen and respond','Listen to a grown-up read a short text. Predict, retell and ask a question.',['Respond to the heard text rather than printed clues.','Give a relevant prediction, retell or question.']),
'EY201':('Read with expression','Read the passage to a grown-up. Pause at punctuation and explain how you decoded a longer word.',['Observe accuracy, phrasing and expression.','Fluency is not certified by speed or speech transcription.']),
'EY203':('Tell a short story','Plan a beginning, middle and end. Write three to eight sentences using time connectives.',['Develop a coherent recount or narrative.','Use sequence and time connectives to connect events.']),
'EY302':('Explain something clearly','Write two to four informational paragraphs about helping a garden.',['Include a topic sentence and relevant supporting details.','Organise paragraphs and provide a conclusion.']),
'EY306':('Share a short presentation','Plan two to five topic sentences about a familiar topic, then present to a grown-up.',['Speak clearly for the audience and purpose.','Organise ideas with an opening and ending.']),
'EY402':('Make a persuasive case','Write three to five paragraphs arguing for a useful community improvement.',['State a clear position with reasons and evidence.','Judge the reasoning, not agreement with the opinion.']),
'EY404':('Connect your ideas','Improve three short sentences using conjunctions and subordinate clauses.',['Use grammatically meaningful connections.','Discuss multiple valid revisions and their effects.']),
'EY406':('Improve a draft','Plan a message, draft it, ask for feedback and revise it one to three times.',['Show a draft and a real revision.','Explain improvements to clarity, vocabulary or organisation.']),
'EY502':('Write and refine','Plan and write four to six informational or persuasive paragraphs, then revise.',['Use purposeful organisation and varied sentences.','Select precise vocabulary and improve the draft.']),
'EY602':('Produce an extended text','Plan, draft and polish five to eight paragraphs for a clear audience and purpose.',['Control structure, language and conventions.','Show revisions and explain choices.']),
'EY604':('Investigate a question','Use the supplied sources to investigate a garden question. Organise findings and acknowledge each source.',['Compare two to five sources and note limitations.','Present findings with clear source attribution.']),
'EY605':('Build a discussion','Discuss a community garden idea over two to five turns. Listen, respectfully disagree and build on another idea.',['Observe active listening and respectful turns.','Explain how a contribution responds to another person.']),
}
def build():
 for skill,(title,scaffold,rubric) in WORKSHOPS.items():
  for variant in range(2):
   stimuli=[]
   if skill=='EY201':stimuli=[{'kind':'passage','text':passage(variant,60,150)[1]}]
   if skill in ['E11','EY107']:stimuli=[text('Grown-up reading text: '+passage(variant,30,100 if skill=='EY107' else 80)[1])]
   if skill=='EY604':stimuli=[text('Source A — Wonderwilds authored observation note: a clean bowl in a quiet shaded place received bird visits. Source B — Wonderwilds authored gardener interview: water bowls need cleaning and safe placement. These are fictional supplied sources for practice, not external research evidence.')]
   prompt=title+'.' if variant==0 else title+': try a different idea.'
   add(skill,'observed' if variant==0 else 'second_prompt',[workshop(prompt,scaffold+(' Try writing for another friend or setting.' if variant else ''),rubric,stimuli,representation='adult_reviewed_response')])
 # Picture-supported spelling gives no printed copy of the target word.
 for skill in ['E07','EY106']:
  for suffix in ['cvc','word_building','sound_boxes']:
   records=[]
   for word in WORDS:
    records.append(typed('Spell the word for this picture.',word,[pic(word)],'Say the sounds, then choose the letters.',representation='sound_boxes' if suffix=='sound_boxes' else 'word_building_tiles'))
   add(skill,suffix,records)
 for skill in ['E02','E08','EY102']:
  add(skill,'recognise_word',[choice('Which word matches the picture?',w,[WORDS[(i+1)%6],WORDS[(i+2)%6]],[pic(w)],representation='object_vocabulary' if skill=='E02' else 'word_cards') for i,w in enumerate(WORDS)])
  add(skill,'sentence_context',[choice('Which word completes the sentence?',a,[b,'run'],[text(f'The {"animal" if a in ["cat","dog"] else "object"} in the picture is a ___.'),pic(a)],representation='sentence_context') for a,b in WORD_PAIRS])
  if skill=='EY102':add(skill,'spell_word',[typed('Spell the word for the picture.',w,[pic(w)],representation='word_cards') for w in WORDS])
 add('E02','actions',[choice('Which word names this action?',v,[other,'sleep'],[text(scene)],representation='action_pictures') for v,other,scene in [('run','sit','Pip moves quickly on foot.'),('jump','walk','Pip pushes off the ground into the air.'),('eat','drink','Pip takes bites of food.'),('drink','eat','Pip sips water.'),('sit','run','Pip rests on a chair.'),('wave','jump','Pip moves a hand to say hello.')]])
 for suffix in ['first_last','word_order','boundaries']:
  add('E09',suffix,[order('Put the words in sentence order.',sentence.split(),representation='sentence_strip') for sentence in ['The cat sat.','A dog ran.','Pip has a hat.','The sun is hot.','A cat can nap.','We see a map.']])
 for suffix in ['sentence_picture','written_instruction']:
  add('E10',suffix,[choice('Which picture matches the sentence?',w,[WORDS[(i+1)%6],WORDS[(i+2)%6]],[text(f'I see a {w}.')],representation='controlled_sentence',pictures=True) for i,w in enumerate(WORDS)])
 add('E12','retrieve',[choice('What does Pip have?',w,[WORDS[(i+1)%6],WORDS[(i+2)%6]],[text(f'Pip has a {w}.')],representation='controlled_written_text') for i,w in enumerate(WORDS)])
 for skill in ['EY204','EY303','EY403']:
  wordsets={
   'EY204':[('kitten','cat'),('running','run'),('jumped','jump'),('boats','boat'),('playing','play'),('wishes','wish')],
   'EY303':[('unhappy','happy'),('replay','play'),('careful','care'),('kindness','kind'),('hopeful','hope'),('dislike','like')],
   'EY403':[('their','they own'),('there','a place'),('hear','listen'),('here','this place'),('piece','a part'),('peace','calm')],
  }[skill]
  for suffix in ['pattern','word_family','context']:
   records=[]
   for target,base in wordsets:
    if skill=='EY403':records.append(choice(f'Which word means “{base}”?',target,['hair','hare','where'] if target not in ['hear','here'] else ['hare','hair'],representation='sentence_context'))
    else:records.append(typed(f'Write the word made from {base} plus {"un-" if target=="unhappy" else "re-" if target=="replay" else "-ness" if target=="kindness" else "dis-" if target=="dislike" else "-ful" if target.endswith("ful") else "-ing" if target.endswith("ing") else "-ed" if target.endswith("ed") else "-es" if target.endswith("es") else "-s"}.',target,hint='Check how the base word changes when the affix is added.',representation='word_building_tiles'))
   add(skill,suffix,records)
 nouns=[('The cat sleeps.','cat'),('A bird sings.','bird'),('The child runs.','child'),('My dog jumps.','dog'),('The flower grows.','flower'),('A rabbit hops.','rabbit')]
 for suffix in ['noun','verb','adjective','pronoun']:
  cases=nouns if suffix=='noun' else [('Pip runs quickly.','runs'),('A bird sings softly.','sings'),('The cat sleeps here.','sleeps'),('We walk home.','walk'),('They swim today.','swim'),('You read quietly.','read')] if suffix=='verb' else [('The red hat fell.','red'),('A small cat slept.','small'),('The warm sun shone.','warm'),('A tall tree grew.','tall'),('The soft leaf fell.','soft'),('A quiet bird sang.','quiet')] if suffix=='adjective' else [('She has a hat.','She'),('He likes flowers.','He'),('They play outside.','They'),('We see a cat.','We'),('It is sunny.','It'),('You can help.','You')]
  add('EY105',suffix,[choice(f'Which word is a {suffix}?',answer,[w.strip('.') for w in sentence.split() if w.strip('.')!=answer][:3],[text(sentence)],representation='sentence_context') for sentence,answer in cases])
 for skill in ['EY205','EY304','EY503']:
  cases=[('Where is Pip?','question'),('Please close the gate.','command'),('What a wonderful day!','exclamation'),('The sun is warm.','statement'),('Can you help?','question'),('Bring your hat.','command')]
  if skill=='EY304':
   for suffix in ['sentence_type','punctuation','appropriate_sentence']:add(skill,suffix,[choice('What type of sentence is this?',typ,['statement','question','command','exclamation'],[text(sentence)],representation='sentence_context') for sentence,typ in cases])
  elif skill=='EY205':
   for suffix in ['commas','exclamation','contraction']:
    records=[]
    for i in range(6):
     if suffix=='commas':records.append(choice('Which sentence uses commas correctly?',f'We packed {WORDS[i]}, food and water.',[f'We, packed {WORDS[i]} food and water.',f'We packed {WORDS[i]} food, and, water.'],representation='sentence_context'))
     elif suffix=='exclamation':records.append(choice('Which end mark shows an excited exclamation?','!',['.','?'],[text(['What a bright star','That was amazing','How wonderful','What a surprise','We did it','What a beautiful flower'][i])],representation='sentence_context'))
     else:records.append(choice('Which contraction is correct?',["can't","don't","isn't","we're","they're","it's"][i],["cant'","dont'","isnt'"], [text(['cannot','do not','is not','we are','they are','it is'][i])],representation='sentence_context'))
    add(skill,suffix,records)
  else:
   for suffix in ['tense','voice','modality']:
    add(skill,suffix,[choice('Which word makes the claim less certain?' if suffix=='modality' else 'Which sentence is in the past tense?' if suffix=='tense' else 'Which sentence uses passive voice?', 'might' if suffix=='modality' else f'Pip packed the {WORDS[i]}.' if suffix=='tense' else f'The {WORDS[i]} was found by Pip.', ['must','will'] if suffix=='modality' else [f'Pip packs the {WORDS[i]}.',f'Pip will pack the {WORDS[i]}.'],[text('Compare the grammatical choices.')],representation='sentence_context') for i in range(6)])
 for suffix in ['simile','metaphor','personification']:
  examples={'simile':['as bright as a star','as quiet as a mouse','like a soft blanket','as cold as ice','like a silver ribbon','as light as a feather'],'metaphor':['The classroom was a beehive.','Her smile was sunshine.','The lake was a mirror.','The road was a ribbon.','His words were a warm blanket.','The crowd was a sea.'],'personification':['The wind whispered.','The sun smiled.','The leaves danced.','The gate groaned.','The river sang.','The flowers nodded.']}[suffix]
  add('EY405',suffix,[choice('Which kind of figurative language is used?',suffix,['simile','metaphor','personification'],[text(example)],hint='A simile compares with like/as; a metaphor states a comparison; personification gives human qualities.',representation='text_passage') for example in examples])
 # Substantial source text is shared across skills; questions target different explicitly named facets.
 reading={'EY103':(30,80,['main_idea','detail','evidence']),'EY202':(80,200,['inference','purpose','evidence']),'EY301':(150,400,['character','setting','plot','language_effect']),'EY305':(60,200,['word_meaning','context_clue']),'EY401':(200,500,['compare','agreement','reliability']),'EY501':(300,700,['structure','point_of_view','language_effect','evidence']),'EY504':(300,700,['synthesise','bias','argument']),'EY601':(400,800,['positioning','perspective','structure','evidence'])}
 for skill,(minimum,maximum,facets) in reading.items():
  for facet in facets:
   records=[]
   for j in range(3):
    title,body=passage(j,minimum,maximum)
    stimuli=[{'kind':'passage','text':body}]
    if skill=='EY401':stimuli+=[{'kind':'passage','text':passage((j+1)%3,minimum,maximum)[1]}]
    protagonists=['Mina','Arun','Ella'];details=['a rabbit','the paper cover','a bowl of water']
    prompts=[f'Who is the main character in “{title}”?',f'Which detail belongs to “{title}”?']
    answers=[protagonists[j],details[j]];wrong=[[protagonists[(j+1)%3],protagonists[(j+2)%3]],[details[(j+1)%3],details[(j+2)%3]]]
    if facet in ['main_idea','inference','purpose','language_effect','structure','point_of_view','synthesise','bias','argument','positioning','perspective','compare','agreement','reliability']:
     questions={
      'main_idea':('What is the main idea?', ['Mina helps protect a garden kindly.','Arun learns from a torn lantern cover.','Ella observes birds patiently.'][j],['Winning a race is most important.','The characters want to leave their homes.']),
      'inference':('What can we infer about the main character?', ['Mina is considerate.','Arun can learn from a mistake.','Ella values careful observation.'][j],['The character never needs help.','The character refuses to try.']),
      'purpose':('What is the main purpose of this narrative?','To tell a story about solving a problem',['To give a recipe','To advertise a product']),
      'point_of_view':('Which point of view tells the story?','Third person',['First person','Second person']),
      'structure':('How is this text mainly organised?','Events and responses unfold over time',['A list of unrelated definitions','Instructions for an experiment']),
      'language_effect':('What effect do the described actions have?','They show how the character responds to a difficulty',['They give exact instructions for a machine','They describe an unrelated sporting event']),
      'synthesise':('Which idea is supported by the events?','Listening and observing can improve an initial solution',['The first solution always solves every problem','Other people never contribute useful ideas']),
      'bias':('Which claim would go beyond the evidence in this story?','This one story proves everyone will behave the same way',['A character responded to a difficulty','The story describes a particular situation']),
      'argument':('Which statement is supported by the text?','The character changes or develops an approach',['Every similar problem has been solved forever','No one else could learn from the situation']),
      'positioning':('How does the narrative encourage readers to view the character?','As someone whose learning and choices are worth considering',['As a proven expert on every topic','As someone who never changes']),
      'perspective':('Which perspective receives most attention?','The main character’s experiences and responses',['Every neighbour’s private thoughts equally','The perspective of a distant stranger']),
      'compare':('What do both passages have in common?','A character responds to a difficulty',['Both are recipes','Both describe the same person']),
      'agreement':('Which idea fits both passages?','A useful response may involve patience or help',['Mistakes should always be hidden','Listening to others is never useful']),
      'reliability':('Are these fictional narratives proof of a scientific rule?','No; they are authored stories, not a controlled investigation',['Yes; a story proves a universal rule','Yes; every character is a scientific source']),
     }
     prompt,answer,distractors=questions[facet];records.append(choice(prompt,answer,distractors,stimuli,hint='Find the specific words and events that support your choice.',representation='dual_text_comparison' if skill=='EY401' else 'text_passage'))
     # A second distinct evidence question uses the same passage, without inventing a new source.
     records.append(choice('Which detail from the passage supports your understanding?',details[j],wrong[1],stimuli,hint='Choose something the passage actually describes.',representation='text_passage'))
    elif facet in ['word_meaning','context_clue']:
     records.append(choice('In this context, what does “gently” mean?' if j==0 else 'What does “scraps” mean?' if j==1 else 'What does “watching” mean?', ['carefully and softly','small leftover pieces','watching attentively'][j],['rushing angrily','forgetting everything'],stimuli,hint='Use nearby words and actions as clues.',representation='word_context_cards'))
     records.append(choice(prompts[1],answers[1],wrong[1],stimuli,representation='text_passage'))
    else:
     for k in range(2):records.append(choice(prompts[k],answers[k],wrong[k],stimuli,hint='Locate the matching detail in the passage.',representation='text_passage'))
   add(skill,facet,records)
 for suffix in ['dictionary','synonym','etymology_register']:
  add('EY505',suffix,[choice(f'Which word has the closest meaning to “{word}” here?',synonym,[opposite,'unknown'],[text(f'Dictionary entry: {word} — {meaning}. Context: a {word} response.')],hint='Select the sense that fits the context, not just any related word.',representation='dictionary_entry') for word,synonym,opposite,meaning in [('calm','peaceful','noisy','quiet and steady'),('brief','short','lengthy','lasting a short time'),('kind','considerate','cruel','showing care'),('rapid','quick','slow','happening quickly'),('careful','cautious','reckless','taking care to avoid mistakes'),('clear','understandable','confusing','easy to understand')]])
 for suffix in ['historical_usage','register','context']:
  add('EY603',suffix,[choice('Which wording fits a formal request?',formal,[informal,'Go away.'],[text('Audience: a community committee. Purpose: request assistance.')],hint='Choose wording appropriate to audience and purpose; informal dialects are not inferior.',representation='text_comparison') for formal,informal in [('Could you assist with the garden?','Hey, help out?'),('Please consider our proposal.','Check this out!'),('We request permission to visit.','Can we pop in?'),('Thank you for your assistance.','Cheers for that!'),('We would appreciate a response.','Get back to us!'),('May we arrange a meeting?','Wanna meet?')]])
 for suffix in ['heading','caption','locate_feature']:
  add('EY206',suffix,[choice('Which heading fits this information?',heading,['Games at the park','Making a cake'],[text(body)],hint='A heading summarises the information below it.',representation='text_with_features') for heading,body in [('Caring for cats','Cats need clean water, suitable food and a safe place to rest.'),('Growing seeds','Seeds need suitable warmth, moisture and time to begin growing.'),('Using a map','A map key explains symbols. Labels help readers locate places.'),('Birds in gardens','Birds may visit quiet gardens with shelter and clean water.'),('Keeping a picnic safe','Keep food covered. Wash hands and take rubbish home.'),('Reading a clock','The short hand shows the hour. The long hand shows minutes.')]])
