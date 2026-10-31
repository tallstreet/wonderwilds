# Activity implementation checklist for all skills

Status: planned, 31 October 2026. Implement with
[the activity implementation plan](ACTIVITY_IMPROVEMENT_PLAN.md).

Each row is required work. Semicolon-separated families become distinct stable
exercise IDs under the skill, for example `M05.one_more`, `M05.one_less` and
`M05.missing_number`. Use descriptive snake_case suffixes in the inventory.
Exercise IDs subdivide practice; they do not change the prerequisite graph.

Controls: **C** single choice, **MC** multiple choice or selectable tokens,
**N** numeric fields, **T** typed word, **O** ordering, **A** assignment/sorting,
**W** workshop with human review. Every O/A control supports tapping and keyboard;
dragging is optional. Audio means reviewed local recordings, with an explicit
human-supported fallback until those assets are ready.

Use node bounds and permitted representations from the actual curriculum JSON,
including the explicit corrections in the main plan. The regression column adds
specific requirements to the shared tests; it does not replace them. A choice
question assesses recognition of its narrow target, not the entire broad node.

## Foundation maths and extensions

| Skill | Exercise families to implement | Controls and stimulus | Required regression |
| --- | --- | --- | --- |
| M01 | Give one object to each recipient; repair a missing/duplicate pairing | A, illustrated recipients and objects | Every object and recipient used once; moving and clearing works |
| M02 | Recognise structured tiny groups; match equivalent dot patterns | C, structured groups of 1–4 | Rearrangement preserves quantity; no countdown; do not claim unobserved counting strategy |
| M03 | Count scattered objects; count ten frames; match total to numeral | C/N, movable objects and ten frames | Counts 1–20, including full and second ten frames; layout does not change answer |
| M04 | More; fewer; same quantity; order three collections | C/O, paired collections and aligned rows | Equal totals with different spacing; no size/spacing answer cue |
| M05 | One more; one less; forward/backward missing number | C/N, number paths and countable sequences | “One more than 9” has 10 correct; decade transitions and endpoint bounds |
| M06 | Find missing part; choose two parts making a whole; split a collection | C/N/A, part–whole objects | Parts sum to 2–10; swapped valid parts accepted where order irrelevant |
| M07 | Join groups; take away; match a pictured change to an equation | C/N, movable objects/change scenes | Both operations; shown removal agrees with subtraction; nonnegative totals |
| M08 | Share equally; identify fair/unfair shares; find amount per recipient | A/C/N, plates and objects | Divisible totals 2–12; reject equal-total but unequal shares |
| M09 | Continue AB/AAB/ABB patterns; find repeating unit; build a repeating pattern | C/O, object and shape patterns | New pattern is validated by its rule, not one memorised colour arrangement |
| M10 | Name a shown shape; sort by sides/corners; match everyday silhouettes | C/A, shape tiles and silhouettes | Rotated/resized shapes; no written description giving away the name |
| M11 | Identify relative position; follow one-to-three movement steps | C, object scene and grid path | Above/below/beside/inside and stated movement frame; captions equivalent to scene |
| M12 | Compare length; compare mass; compare capacity | C, aligned objects, balance and containers | Misaligned starts; larger-looking object need not be heavier or hold more |
| M13 | Order daily events; identify before/after; match events to day stages | O/C, event cards and day sequence | Only unambiguous authored order; avoid family-routine assumptions |
| M14 | Sort by one stated attribute; choose an item belonging to a group | A/C, objects and attribute cards | Distractor shares irrelevant attribute; categories clearly defined |
| M15 | Build category counts from observations; read a picture graph; compare categories | A/C/N, object and picture graphs | Each observation counted once; graph categories/totals match source |
| M16 | Count tens and ones; build a numeral; regroup ten loose objects | C/N/A, seed bundles and place-value model | 10–30 including exact tens; distinguish 12 from 21 |
| M17 | Count equal groups; skip-count by group size; complete a grouped total | C/N, planting rows and grouped objects | 2–5 groups; count groups versus count objects distractors |
| M18 | Identify equal parts; recognise halves/quarters; choose a valid partition | C, partitioned and folded wholes | Unequal pieces are incorrect despite correct piece count |

## Foundation English and extensions

| Skill | Exercise families to implement | Controls and stimulus | Required regression |
| --- | --- | --- | --- |
| E01 | Follow one/two spoken directions; identify next turn; practise taking turns | C/A plus W, audio and pictured objects | No written instruction substituted for listening evidence; turn-taking observed |
| E02 | Match object vocabulary; match action vocabulary; choose a word in context | C, object/action pictures | Images distinguish meanings; no reading-heavy distractors for vocabulary-only task |
| E03 | Choose a rhyme; count syllables; match syllable patterns | C/N, audio and picture cards | Reviewed pronunciations; both rhyme and syllable families present |
| E04 | Identify first/last phoneme; blend/segment 2–3 phonemes; change one sound | C/A plus W, audio and sound tokens | Manipulation targets phonemes, not letter names; spoken production human reviewed |
| E05 | Match a heard taught sound to a letter; match a letter to a heard sound | C, letter tiles and audio options | Explicit taught correspondences; letter name is not phoneme; no STT scoring |
| E06 | Match a printed decodable word to a picture; blend a taught word aloud | C plus W, word tiles and printed labels | Only taught 2–3-phoneme words; recognition is not certified oral decoding |
| E07 | Segment a spoken word into sound boxes; build/type its spelling | A/T, audio, tiles and sound boxes | Separate identities for duplicate letters; accepted taught spelling alternatives |
| E08 | Recognise taught common words; match a word across signs/contexts | C, word cards and printed signs | Explicit taught list; visual font variation does not become new evidence |
| E09 | Find first/last word; track word order; identify sentence boundaries | C/MC/O, sentence strips and word cards | Token IDs distinguish repeated words; 2–5-word sentences |
| E10 | Match a controlled sentence to a picture; follow a written short instruction | C/A, controlled sentences and instruction cards | 2–5 words; no narrated assessment sentence for independent reading |
| E11 | Recall a heard character/detail; order heard story events | C/O, narrated pictures and character cards | 2–4 events; images alone do not disclose the entire answer |
| E12 | Retrieve who/what/where from independently read text | C, controlled written text | Text remains 3–8 words; answer supported explicitly; reading aloud counts as support |
| E13 | Compose an invitation; compose a useful short message | W, optional message tiles and local draft | 2–6-word target; rubric for meaning/audience; no choice-based writing certificate |
| E14 | Discuss a character choice; give a reason for an alternative | W, picture discussion and response scaffold | Multiple defensible ideas accepted by human review; no automatic opinion marking |
| E15 | Extend a taught spelling pattern; add detail to a sentence/postcard | W with T supporting practice, sentence tiles | 4–10-word target; spelling practice kept separate from composed-message evidence |

## Year 1 maths

| Skill | Exercise families to implement | Controls and stimulus | Required regression |
| --- | --- | --- | --- |
| MY101 | Count forward/back by 1/2/5/10; fill missing terms; cross decades | C/N, number line and hundreds chart | Nonzero starting points and boundary near 120; preserve valid generated endpoints |
| MY102 | Read base-ten models; write two-digit numeral; identify tens/ones | C/N, blocks and place-value chart | 10–99; internal/terminal zero; reversed-digit distractors |
| MY103 | Compare with symbols; choose greatest/least; order two-digit numbers | C/O, place-value models and number line | Equal numbers; same tens/different ones; 19 versus 21 |
| MY104 | Count on; make ten; solve addition story/equation | C/N, ten frame and number line | Crossing ten; sum no greater than 20; operands within bounds |
| MY105 | Count back; missing-addend subtraction; solve take-away story | C/N, ten frame and number line | Distinguish starting value from amount removed; no negative results |
| MY106 | Name 2D/3D shapes; count features; sort by feature | C/N/A, shapes and object illustrations | Orientation; faces/edges/corners; flat shape versus solid |
| MY107 | Compare length/mass/capacity; measure with repeated equal units | C/N, comparison scenes and unit models | Equal units without gaps; aligned endpoints; indirect comparison |
| MY108 | Extend repeating pattern; choose core; continue/create number pattern | C/O/N, pattern tiles and number sequences | AB and more complex cores; skip rule distinct from next-value guessing |
| MY109 | Record observations; build/read tally table; interpret picture graph | A/C/N, picture graph and tally table | 2–4 categories; source observations and graph agree |
| MY110 | Read hour; read half-hour; match analogue and digital; set time | C/N, clocks with selectable time options | At 3:30 short hand is between 3 and 4; distinguish swapped hands |

## Year 2 maths

| Skill | Exercise families to implement | Controls and stimulus | Required regression |
| --- | --- | --- | --- |
| MY201 | Read/write three-digit models; identify digit value; expand a number | C/N, blocks and place-value chart | 100–999; zero placeholders and value versus digit |
| MY202 | Add/subtract within 100; regroup; find a missing value | C/N, place-value chart and number line | Regrouping and no-regrouping; results 0–100 subject to node bounds |
| MY203 | Read arrays; connect groups/repeated addition; 2/3/5/10 facts | C/N, arrays and equal groups | All four named tables; row/column grouping; avoid unrelated tables in this family |
| MY204 | Equal sharing; equal grouping; connect division to multiplication | A/N, groups and arrays | Divisibility; divisor versus quotient; both interpretations of division |
| MY205 | Identify half/quarter of shape; half/quarter of collection | C/N, fraction and area models | Equal parts; divisible collection; same whole for comparisons |
| MY206 | Read ruler; compare mass/capacity; measure area with equal units | C/N, rulers, scales and unit grids | Nonzero ruler starts; units; square units versus length units |
| MY207 | Read quarter-past/to; match digital; distinguish AM/PM contexts | C/N, analogue/digital clocks | 12 boundary and hour-hand placement; explicit context for AM/PM |
| MY208 | Classify shapes/solids; identify symmetry; recognise simple transformations | C/MC, shapes and fold diagrams | Rotations/reflections; symmetry line actually maps the shape |
| MY209 | Organise data; complete bar/table entries; interpret and compare | C/N/A, bar graph and table | 3–5 categories; axes labels, totals and conclusion supported |

## Year 3 maths

| Skill | Exercise families to implement | Controls and stimulus | Required regression |
| --- | --- | --- | --- |
| MY301 | Read/write four-digit values; identify place value; round to 10/100 | C/N, chart and number line | Rounding ties and place-value crossings; zero placeholders |
| MY302 | Column addition; column subtraction; fill a missing column step | N/C, column layout and chart | Three/four digits; carrying/borrowing through zero; independent expected result |
| MY303 | Recall multiplication facts; find missing factor; apply a fact | N/C, array and table | 2–10 factors per bounds; no speed threshold or timer |
| MY304 | Division facts; quotient/remainder; match inverse fact | N/C, array and number line | Two labelled fields; remainder smaller than divisor; exact division included |
| MY305 | Recognise/place unit fractions; compare/order same-denominator proper fractions | C/O, fraction models and number line | Equal whole; distinguish the two objective facets; no false unit-fraction label |
| MY306 | Count grid area; calculate rectangle area; calculate perimeter | N/C, grid and ruler diagram | Area/perimeter distractors; displayed dimensions; square versus linear units |
| MY307 | Read scaled graphs; compare categories; combine table values | N/C, scaled bar graph and table | Scales 2–10; tick count is not frequency; categories labelled |
| MY308 | Classify chance language; place event on likelihood scale | C, event cards and scale | Impossible/certain and likely/unlikely supported by stated experiment |
| MY309 | Read grid reference; follow compass direction; interpret legend/map | C, grid map and compass | Reference order and north orientation explicit; routes stay in grid |

## Year 4 maths

| Skill | Exercise families to implement | Controls and stimulus | Required regression |
| --- | --- | --- | --- |
| MY401 | Read/write large values; expand digit value; compare/order numbers | N/C/O, place-value chart and expander | Includes 1000000 after bounds correction; internal zeroes |
| MY402 | Area-model multiplication; standard multiplication; missing partial product | N/C, area and column models | Two-digit by one-digit; regrouping; partial products sum correctly |
| MY403 | Find equivalent fraction; complete numerator/denominator; locate on line | N/C, fraction wall and number line | Exact equivalence; consistent whole; bounded nonzero denominator |
| MY404 | Read hundredths model; identify decimal place; connect fraction/decimal | N/C, hundredths grid and chart | 0.5 equals 0.50; 0.05 differs; scaled integer precision |
| MY405 | Composite area; composite perimeter; select a useful decomposition | N/C, gridded composite diagrams | Shared edges excluded from perimeter; missing lengths derivable |
| MY406 | Read line graph; complete data point; compare trends and predictions | N/C, line graph and table | Continuous ordered axis; predictions qualified; not guaranteed outcomes |
| MY407 | Fraction probability; compare chances; record a chance experiment | N/C/A, spinner and outcome table | Unequal sector areas respected; trial observations do not redefine theoretical chance |
| MY408 | Elapsed time; timetable lookup; convert time units | N/C, timeline and digital timetable | Across hour/noon boundaries; duration versus clock time |

## Year 5 maths

| Skill | Exercise families to implement | Controls and stimulus | Required regression |
| --- | --- | --- | --- |
| MY501 | Multi-digit multiplication; multi-digit division; choose/check method | N/C, columns and area model | Stated factor bounds; valid division tasks; regrouping and place alignment |
| MY502 | Compare/order unlike fractions; improper-to-mixed conversion; number-line placement | C/O/N, fraction wall and line | Equivalent fractions; mixed-number multi-fields; common-whole diagrams |
| MY503 | Add/subtract related fractions; multiply fraction by whole number | N/C, fraction model and line | Exact rational comparison; simplify only when explicitly requested |
| MY504 | Read/order thousandths; add decimals; subtract decimals | N/O, chart and line | Unequal digit counts and trailing zeroes; no binary float marking |
| MY505 | Percent of hundred model; convert fraction/decimal/percent; simple application | N/C, hundredths grid and line | Percent is rate per 100; equivalent representations; 100 percent boundary |
| MY506 | Read/plot coordinates; complete value table; interpret relationship | C/N, Cartesian grid and table | Coordinates 0–10; x/y order; axes/scale consistent |
| MY507 | Mean; median; mode; range; choose useful measure | N/C, table and dot plot | Unsorted data; no/multiple mode handled explicitly; outlier changes choice |
| MY508 | Count unit cubes; calculate prism volume; connect capacity units | N/C, prism and grid model | Cubic units; three dimensions; define conversion in task and validate |

## Year 6 maths

| Skill | Exercise families to implement | Controls and stimulus | Required regression |
| --- | --- | --- | --- |
| MY601 | Evaluate expressions; insert/select brackets; choose equivalent expression | N/C, structured expression | Multiplication/division and addition/subtraction left-to-right; no eval |
| MY602 | Real-world fraction/decimal/percent problems; choose representation | N/C, bar and hundredths models | Explicit quantities/units; exact conversion and suitable contextual rounding |
| MY603 | Prime/composite; divisibility rules; choose factorisation | C/MC, factor tree and sieve | 2 prime; perfect squares; every composite has divisor; no 1 under existing bounds |
| MY604 | Equivalent ratios; missing ratio part; unit-rate comparison | N/C, ratio bars and table | Part-to-part versus part-to-whole; compare equal units |
| MY605 | Substitute into expression; solve equation; match situation to expression | N/C, algebra tiles and balance | 1–3 variables; no arbitrary expression execution; balance maintained |
| MY606 | Missing straight-line angle; triangle angle; classify triangle | N/C, angle diagrams | Valid positive triangle angles sum 180; not-to-scale notice where needed |
| MY607 | Read distribution; compare representations; identify misleading graph | C/N, stem-and-leaf and paired graphs | Key, truncated baseline and unequal intervals; same data under both graphs |
| MY608 | Complement probability; mutually exclusive events; compare event probabilities | N/C, probability scale and Venn/outcome model | Exhaustive sample space; complement sums to one; exclusivity checked |

## Year 1 English

| Skill | Exercise families to implement | Controls and stimulus | Required regression |
| --- | --- | --- | --- |
| EY101 | Read taught decodable words in short text; match meaning; observed oral decode | C plus W, word cards and decodable text | 3–12 words; taught correspondences; oral decoding stays human reviewed |
| EY102 | Recognise high-frequency word; choose word in context; spell heard target | C/T, word cards, audio and sentence | Taught list; irregular forms; spelling cannot be solved by copying target |
| EY103 | Main idea; supporting detail; select evidence sentence | C/MC, 30–80-word text | Every answer grounded in passage; main idea distinct from one detail |
| EY104 | Capital/end-mark practice; order sentence; independently compose a sentence | C/O plus W, tiles and draft | 3–10-word target; recognise punctuation versus independent writing kept distinct |
| EY105 | Identify noun/verb/adjective/pronoun; choose word fitting a sentence | C/MC/A, sentence tokens and word sort | Word class determined in context; repeated words have unique IDs |
| EY106 | Spell CVC; spell CVCE; spell taught blends | T/A, reviewed audio, sound boxes and tiles | 2–6 letters; distinct letter identities; no dictation-based spelling success |
| EY107 | Listen then predict; order/retell; ask a relevant question | C/O plus W, 30–100-word audio text | Replay available; no written passage masquerading as listening; retell observed |

## Year 2 English

| Skill | Exercise families to implement | Controls and stimulus | Required regression |
| --- | --- | --- | --- |
| EY201 | Read multisyllabic words in passage; observed fluent/expressive reading | W, 60–150-word passage | Fluency/expression rubric; STT/timing alone never certifies reading |
| EY202 | Narrative inference; informational inference; author's purpose; evidence | C/MC, 80–200-word texts | Both genres; one defensible marked inference supported by text |
| EY203 | Plan/order recount; compose narrative beginning/middle/end; revise connective | W with O supporting practice, writing scaffold | 3–8-sentence target; sequencing choices alone do not assess composition |
| EY204 | Digraph/blend spelling; long vowels; plurals and simple suffixes | T/A, audio, word sort and tiles | 3–8-letter words; each named pattern represented |
| EY205 | Commas in lists; exclamation marks; contraction apostrophes | C/MC, 2–6-sentence contexts | Punctuation is the target; alternatives such as final list conjunction reviewed |
| EY206 | Choose heading/subheading; match caption; locate using a text feature | C, informational text layout | 2–5 features; real visual hierarchy; caption refers to shown image |

## Year 3 English

| Skill | Exercise families to implement | Controls and stimulus | Required regression |
| --- | --- | --- | --- |
| EY301 | Character/setting/plot; language effect; choose supporting evidence | C/MC, 150–400-word literary text | All named facets; evidence span matches selected interpretation |
| EY302 | Plan topic/support/conclusion; write informational paragraphs | W, writing scaffold | 2–4-paragraph target; rubric checks organisation and relevant details |
| EY303 | Build prefix/base/suffix word; spell morphological change; classify families | T/A, word tiles and word sort | 4–10 letters; valid root changes; alternatives authored explicitly |
| EY304 | Identify sentence type; choose matching punctuation; select appropriate sentence | C/A, 3–8 sentence examples | Statement/question/command/exclamation all represented |
| EY305 | Infer word meaning; select context clue; discuss alternative meanings | C/MC plus W, 60–200-word text | Context actually distinguishes meaning; discussion not auto-certified |
| EY306 | Plan short presentation; practise audience/purpose; present clearly | W, planning scaffold | 2–5-topic-sentence target; clarity/audience rubric; no microphone requirement |

## Year 4 English

| Skill | Exercise families to implement | Controls and stimulus | Required regression |
| --- | --- | --- | --- |
| EY401 | Compare two texts; distinguish agreement/difference; evaluate source reliability | C/MC, paired texts and organiser | 200–500-word bound explicit in inventory per passage; sources carry usable provenance |
| EY402 | Plan position/reasons/evidence; write persuasive text | W, writing scaffold | 3–5-paragraph target; opinion is not marked by preferred stance |
| EY403 | Spell/select homophone; distinguish confused words; use Latin/Greek roots | T/C/A, sentence and word sort | Context disambiguates homophones; 5–20-word set; no answer-visible spelling |
| EY404 | Select conjunction/clause; combine sentences; independently improve sentence | C/O plus W, sentence builder | 3–8-sentence material; multiple valid combinations; writing human reviewed |
| EY405 | Identify simile/metaphor/personification; choose meaning/effect | C/A, 3–8 examples and passage | Literal/figurative distinctions; effect answer grounded in example |
| EY406 | Plan; draft; apply feedback; revise clarity/organisation | W, staged local writing scaffold | 1–3 revision rounds; actual draft/edit evidence for human review |

## Year 5 English

| Skill | Exercise families to implement | Controls and stimulus | Required regression |
| --- | --- | --- | --- |
| EY501 | Narrative structure; point of view; effect of language; evidence | C/MC, 300–700-word text | Different viewpoints; effect is supported, not a subjective preference |
| EY502 | Plan informational/persuasive text; draft; revise vocabulary/structure | W, writing scaffold | 4–6-paragraph target; both genres across prompts; human rubric |
| EY503 | Identify/change tense; distinguish voice; interpret modal strength/effect | C/MC, 4–10-sentence contexts | Tense/voice/modality all covered; grammatical and meaning-preserving alternatives |
| EY504 | Synthesise information; identify bias; evaluate claim/evidence | C/MC, 300–700-word complex text | Evidence versus assertion; answer not inferred from topic preference |
| EY505 | Read dictionary entry; choose thesaurus word; interpret origin/register | C, entries and contextual word choices | 5–15-word set; correct sense and register rather than any synonym |

## Year 6 English

| Skill | Exercise families to implement | Controls and stimulus | Required regression |
| --- | --- | --- | --- |
| EY601 | Identify reader positioning; compare perspective; analyse language/structure evidence | C/MC, 400–800-word text and annotations | Plausible alternatives reviewed; evidence supports claimed effect |
| EY602 | Plan extended text; draft; revise; edit/polish for purpose | W, writing scaffold | 5–8-paragraph target; human rubric covers structure/language/conventions |
| EY603 | Compare historical/current usage; distinguish register; select context-appropriate language | C, 2–4 text examples | Formal/informal is context-dependent, not a ranking of dialects |
| EY604 | Form research question; compare supplied sources; organise findings/citations; present | W, 2–5 bounded supplied sources | Source attribution visible; no external browsing requirement for children |
| EY605 | Practise listening; respectful disagreement; build on a contribution | W with C supporting scenarios, discussion scaffold | 2–5 observed turns; scenario choice is not certified collaborative discussion |

## Checklist acceptance

- Inventory counts are 70 maths plus 50 English, exactly matching the graph.
- Implement and test every listed family. Workshop recognition exercises are
  supplementary; keep productive assessment human reviewed.
- Human-reviewed work uses the supported/not-assessed story route and an explicit
  rubric. Never present its completion as independent academic success.
- Reviewed-audio dependencies remain visible until recordings are available.
  Missing assets must leave a useful human-supported activity, not a broken screen.
- Publish actual per-family coverage and review limits in `ACTIVITY_COVERAGE.md`.
  Do not replace that document with an unqualified “all skills assessed” claim.
