# Activity coverage

Updated 31 October 2026. All **120** draft skill-map nodes use the current
question controls in story missions: **354 exercise families** and
**3945 original authored question/prompt records**. The controls are direct
single/multiple choices, typed numbers/words, ordering, object assignment and
human-reviewed workshops. Rust owns automatic marking; no answer bank is served
to the browser. Completing one story question still finishes that skill's practice
activity, separately from readiness or mastery.

The [implementation plan](ACTIVITY_IMPROVEMENT_PLAN.md) and
[skill checklist](ACTIVITY_SKILL_CHECKLIST.md) describe the intended breadth.
The internal inventory is `content/activities/skill-exercises-v1.json`, authored
by `scripts/activity_content/` and rebuilt with `python3 scripts/build-activities.py`.
The table reports implemented families, controls and actual bank sizes. Small
finite contrasts have fewer items than open numeric families. More records do
not establish calibrated difficulty, transfer, educator approval or full-skill
assessment.

## Assessment and review limits

- Reviewed audio is not yet available: `content/activities/audio-v1.json` has no
  approved recordings. Listening, phoneme production and oral decoding retain
  explicit grown-up-supported routes. Written questions do not masquerade as
  independent listening evidence. The renderer supports allowlisted local audio
  references with a clear missing-recording fallback.
- Picture-supported spelling uses the visual target and, for less obvious long
  vowel/blend targets, word length and an initial-letter clue. Dictation never
  certifies spelling or decoding. Additional taught word sets and dialect review
  remain educator work.
- Productive writing, fluency, presentations, inquiry and collaboration use local
  drafts/scaffolds and review rubrics. Supported completion is not an actual
  recorded adult approval or an independent success. Some recognition exercises
  support these skills without assessing their productive objective.
- Reading banks use three original narratives, three paired fictional
  informational/promotional sources, and shorter Year 2 informational examples.
  Text lengths follow the node bounds, per passage for paired texts. More sources,
  genre breadth and educator-reviewed interpretation alternatives remain review
  limits; these sources do not establish full reading coverage.
- `MY305` still has the draft unit-fraction versus same-denominator wording
  ambiguity. Recognition/placement of unit fractions and comparison of proper
  same-denominator fractions are separate exercise families; alignment is pending.
- Difficulty rungs describe draft bank organisation. There is no new adaptive
  algorithm, speed scoring, spaced review or validated mastery policy.
- Icons use bundled Lucide SVGs; clocks, equal parts, number lines, graphs, grids,
  measurement, place value, composite shapes and prisms use bounded semantic
  diagrams. Passing browser checks does not replace educator, screen-reader,
  iPad Safari or child usability review.

Public IXL catalogues informed the task-family comparison, including
[Foundation maths](https://au.ixl.com/maths/foundation/skills) and
[Year 3 English](https://au.ixl.com/english/year-3/skills). Exact interactive
exercise mechanics and bank parity were not verified. Wonderwilds questions,
text and assessment rules are original; no IXL endorsement is implied.
All curriculum and release reviews remain pending and release is disabled.

## Implemented inventory

| Skill | Exercise families | Controls | Records |
| --- | --- | --- | --- |
| M01 | one for each; repair pairing | assign | 11 |
| M02 | tiny groups; match patterns | single_choice | 8 |
| M03 | scattered; ten frames; numeral match | number_entry | 60 |
| M04 | more; fewer; same; order | order, single_choice | 42 |
| M05 | one more; one less; missing number | number_entry, single_choice | 54 |
| M06 | missing part; two parts; split collection | assign, number_entry, single_choice | 110 |
| M07 | join; take away; picture equation | number_entry, single_choice | 135 |
| M08 | share; per recipient; fair shares | assign, number_entry, single_choice | 23 |
| M09 | continue; repeating unit; build pattern | single_choice, workshop | 14 |
| M10 | features; name shape; silhouettes | multi_choice, single_choice | 9 |
| M11 | position; movement | single_choice | 6 |
| M12 | length; mass; capacity | single_choice | 36 |
| M13 | order events; before after; day stages | order, single_choice | 18 |
| M14 | sort attribute; belongs | assign, single_choice | 10 |
| M15 | record counts; read graph; compare categories | assign, number_entry, single_choice | 51 |
| M16 | count tens ones; build number; regroup | number_entry | 63 |
| M17 | equal groups; skip count; grouped total | number_entry | 36 |
| M18 | equal parts; halves quarters; partition | single_choice | 12 |
| E01 | observed; second prompt | workshop | 2 |
| E02 | recognise word; sentence context; actions | single_choice | 18 |
| E03 | observed; second prompt | workshop | 2 |
| E04 | observed; second prompt | workshop | 2 |
| E05 | observed; second prompt | workshop | 2 |
| E06 | observed; second prompt | workshop | 2 |
| E07 | cvc; word building; sound boxes | order, text_entry | 18 |
| E08 | recognise word; sentence context | single_choice | 12 |
| E09 | first last; word order; boundaries | order, single_choice | 18 |
| E10 | sentence picture; written instruction | single_choice | 12 |
| E11 | observed; second prompt | workshop | 2 |
| E12 | retrieve | single_choice | 6 |
| E13 | observed; second prompt | workshop | 2 |
| E14 | observed; second prompt | workshop | 2 |
| E15 | observed; second prompt | workshop | 2 |
| MY101 | forward; backward; missing terms | number_entry | 36 |
| MY102 | read write; digit value; expand | number_entry | 36 |
| MY103 | symbols; greatest least; order | order, single_choice | 36 |
| MY104 | count on; make ten; addition story | number_entry | 36 |
| MY105 | count back; missing addend; take away | number_entry | 144 |
| MY106 | name shape; features; sort | assign, number_entry, single_choice | 25 |
| MY107 | length; mass; capacity; equal units | number_entry, single_choice | 48 |
| MY108 | repeating core; number pattern; create pattern | number_entry, single_choice, workshop | 17 |
| MY109 | record; read; compare; combine | assign, number_entry | 52 |
| MY110 | read clock; match digital; set time | number_entry, single_choice | 72 |
| MY201 | read write; digit value; expand | number_entry | 36 |
| MY202 | add; subtract; missing value | number_entry | 36 |
| MY203 | multiply; missing factor; apply fact | number_entry | 36 |
| MY204 | share; group; inverse | number_entry, single_choice | 75 |
| MY205 | shape half; shape quarter; collection | number_entry, single_choice | 22 |
| MY206 | length; mass; capacity; area | number_entry, single_choice | 48 |
| MY207 | read clock; match digital; set time; am pm | number_entry, single_choice | 78 |
| MY208 | classify; symmetry; transform | single_choice | 18 |
| MY209 | record; read; compare; combine | assign, number_entry | 52 |
| MY301 | read write; digit value; expand; round 10; round 100 | number_entry | 60 |
| MY302 | add; subtract; missing value | number_entry | 36 |
| MY303 | multiply; missing factor; apply fact | number_entry | 243 |
| MY304 | facts; remainder; inverse | number_entry | 36 |
| MY305 | unit fraction; place on line; compare; order | number_entry, order, single_choice | 41 |
| MY306 | area; perimeter | number_entry | 24 |
| MY307 | record; read; compare; combine | number_entry | 48 |
| MY308 | language; likelihood scale | order, single_choice | 7 |
| MY309 | grid reference; compass; map legend | number_entry, single_choice | 46 |
| MY401 | read write; digit value; expand; compare order | number_entry, order | 48 |
| MY402 | area model; standard; partial product | number_entry | 36 |
| MY403 | equivalent; missing part; locate | number_entry | 102 |
| MY404 | read model; digit value; fraction decimal | number_entry | 36 |
| MY405 | composite area; composite perimeter; decompose | number_entry, single_choice | 36 |
| MY406 | read line; complete point; trend prediction | number_entry, single_choice | 36 |
| MY407 | fraction probability; compare; experiment | number_entry, single_choice | 41 |
| MY408 | elapsed; timetable; convert | number_entry | 36 |
| MY501 | multiply; missing factor; apply fact; division; choose method | number_entry, single_choice | 60 |
| MY502 | compare; order; mixed number; locate | number_entry, order, single_choice | 138 |
| MY503 | add; subtract; multiply | number_entry | 52 |
| MY504 | read order; add; subtract | number_entry | 36 |
| MY505 | hundred model; convert; application | number_entry | 36 |
| MY506 | read point; plot point; value table; relationship | number_entry, single_choice | 70 |
| MY507 | mean; median; mode; range; choose measure | number_entry, single_choice | 60 |
| MY508 | unit cubes; prism volume; capacity | number_entry | 72 |
| MY601 | evaluate; brackets; equivalent expression | number_entry, single_choice | 36 |
| MY602 | fraction context; decimal context; percent context; representation | number_entry | 48 |
| MY603 | prime composite; divisibility; factorisation | multi_choice, single_choice | 54 |
| MY604 | equivalent ratio; missing part; unit rate | number_entry | 36 |
| MY605 | substitute; solve equation; match expression | number_entry, single_choice | 36 |
| MY606 | straight line; triangle angle; classify | number_entry, single_choice | 36 |
| MY607 | distribution; compare graphs; misleading graph | number_entry, single_choice | 36 |
| MY608 | complement; exclusive; compare | number_entry, single_choice | 51 |
| EY101 | observed; second prompt | workshop | 2 |
| EY102 | recognise word; sentence context; spell word | single_choice, workshop | 13 |
| EY103 | main idea; detail; evidence | single_choice | 18 |
| EY104 | observed; second prompt | workshop | 2 |
| EY105 | noun; verb; adjective; pronoun | single_choice | 24 |
| EY106 | cvc; word building; sound boxes; cvce; blends | order, text_entry | 30 |
| EY107 | observed; second prompt | workshop | 2 |
| EY201 | observed; second prompt | workshop | 2 |
| EY202 | inference; purpose; evidence | single_choice | 36 |
| EY203 | observed; second prompt | workshop | 2 |
| EY204 | pattern; word family; context; digraphs; long vowels; blends | order, single_choice, text_entry | 36 |
| EY205 | commas; exclamation; contraction | single_choice | 18 |
| EY206 | heading; caption; locate feature | single_choice | 18 |
| EY301 | character; setting; plot; language effect | single_choice | 24 |
| EY302 | observed; second prompt | workshop | 2 |
| EY303 | pattern; word family; context | order, text_entry | 18 |
| EY304 | sentence type; punctuation; appropriate sentence | single_choice | 18 |
| EY305 | word meaning; context clue | single_choice | 12 |
| EY306 | observed; second prompt | workshop | 2 |
| EY401 | compare; agreement; reliability | single_choice | 18 |
| EY402 | observed; second prompt | workshop | 2 |
| EY403 | pattern; word family; context | single_choice | 18 |
| EY404 | observed; second prompt | workshop | 2 |
| EY405 | simile; metaphor; personification | single_choice | 18 |
| EY406 | observed; second prompt | workshop | 2 |
| EY501 | structure; point of view; language effect; evidence | single_choice | 24 |
| EY502 | observed; second prompt | workshop | 2 |
| EY503 | tense; voice; modality | single_choice | 18 |
| EY504 | synthesise; bias; argument | single_choice | 18 |
| EY505 | dictionary; synonym; etymology register | single_choice | 18 |
| EY601 | positioning; perspective; structure; evidence | single_choice | 24 |
| EY602 | observed; second prompt | workshop | 2 |
| EY603 | historical usage; register; context | single_choice | 18 |
| EY604 | observed; second prompt | workshop | 2 |
| EY605 | observed; second prompt; scenario | single_choice, workshop | 8 |

## Manual review before release

These checks are pending; passing automated checks does not complete them.

- An educator checks every family against the skill checklist, answer keys,
  plausible distractors, vocabulary, source interpretation and difficulty.
- Children try each control on a phone/tablet: find the action, answer, correct a
  mistake, request a hint and continue after feedback without adult UI coaching.
- Check small-screen text and diagram legibility, keyboard-only use, screen-reader
  names/focus/feedback, colour-independent selection and iPad Safari touch/drag.
- Confirm phoneme/dialect recordings, consent and asset allowlisting before
  enabling listening exercises. Observe workshop review with a grown-up.
- Confirm supported completion remains distinct from independent evidence and
  story access remains available. Record issues and actual reviewer decisions
  before changing pending review or release flags.
