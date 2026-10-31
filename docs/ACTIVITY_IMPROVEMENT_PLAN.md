# Activity improvement implementation plan

Date: 31 October 2026. Status: planned, not implemented.

Replace the draft activity experience with short, clear questions that children
answer by tapping choices, entering a number or word, or occasionally arranging
objects. Upgrade all 120 skill-map nodes using the exact exercise requirements in
[the skill checklist](ACTIVITY_SKILL_CHECKLIST.md). Keep learning inside the
existing story missions, with brief feedback and an easy supported route onward.

## Scope and development assumptions

- This is a prelaunch replacement. The user permits clearing synthetic development
  data if necessary. Do not build old question-bank readers, dual activity
  factories, staged compatibility rollouts or data-conversion machinery.
- Prefer replacing the existing draft implementation directly. Remove obsolete
  mission factory branches and tests when the replacement makes them redundant.
  Keep unrelated guest, story and assessment features intact.
- Reset incompatible synthetic learning state deliberately, rather than silently
  ignoring validation errors. Do not reset anything during the planning task.
- Keep the 120 existing skill IDs and prerequisite graph. Exercise families are
  subdivisions of those nodes, not new prerequisite nodes.
- Preserve server assessment, profile ownership, transactions and exact retries
  for the new implementation. Prelaunch status does not justify duplicate story
  rewards, lost answers or client-supplied correctness.
- Keep existing completion semantics: one confirmed correct answer or explicit
  together-completion finishes that skill's story activity. Do not require a
  multi-question test before continuing the adventure. Activity completion is
  separate from readiness and mastery.
- Do not add a separate practice hub, new adaptive/mastery algorithm, spaced
  repetition service, content CMS, generated question service or analytics system.
  The larger exercise banks are used for initial selection and wrong-answer
  replacements; completing one item does not prove every exercise was assessed.
- Preserve pending educator reviews and disabled release gates. Automated checks
  establish implementation correctness, not educational approval.

## Read and modify the active code path

Read `README.md`, `docs/IMPLEMENTATION_STATUS.md`, `docs/ACTIVITY_COVERAGE.md`,
`docs/adr/curriculum-starter-practice.md` and both curriculum pack READMEs first.
Some older feature documentation describes superseded entry points.

| Area | Existing files | Work |
| --- | --- | --- |
| Skill definitions | `content/curriculum/seed-v1.json`, `content/curriculum/years/seed-years1to6-v1.json`, their authoring packs | Keep IDs/edges; enforce bounds; reconcile objective conflicts below |
| Question generation and marking | `backend/crates/learning/src/mission.rs`, `backend/crates/learning/src/mission/catalogue.rs` | Replace starter-only catalogue with focused exercise factories and private assessors |
| Persistence and feedback | `backend/crates/api/src/mission_learning/repository.rs`, `http.rs` | Persist exercise identity; assess typed answers; issue stable replacement questions and teaching feedback |
| Wire contract | `contracts/schemas/v1.json`, `contracts/openapi.yaml`, `contracts/fixtures/cases.json` | Define typed questions, stimuli, answers and feedback; regenerate Rust/TypeScript |
| Activity interface | `frontend/src/mission-learning/activity.ts`, `activity.css` | Build the simple question shell and reusable controls |
| Answer entry and transport | `frontend/src/mission-learning/answer.ts`, `voice.ts`, `client.ts`, `validation.ts` | Typed input, optional existing dictation, schema validation and pending-command recovery |
| World integration | Find `mountMissionActivity` callers with `rg` | Keep automatic return, wrong-question replacement and save recovery working |
| Existing shared pieces | `frontend/src/learning/drag.ts`, `visuals.ts` | Reuse gestures and bundled artwork; do not rewrite the separate assessment UI |
| Tests | Learning unit tests, `backend/crates/api/src/mission_learning/repository/tests.rs`, mission-learning frontend tests/browser suite, `qa/e2e/mission-learning.mjs` | Extend the existing test structure |

## The question screen

Render in this order:

1. A short activity title and optional one-line character invitation.
2. One direct question. Emphasise the relevant words using safe text segments.
3. The necessary picture, diagram, table or passage.
4. The answer control.
5. **Check answer**, followed by optional **Hint**, **Show me**, and
   **Let's do it together**. Keep the existing put-aside action available.

Do not show curriculum objectives, lesson paragraphs, generic drag instructions,
answer trays or duplicate question text in the default screen. Put explanatory
teaching text behind help. Use direct choices for the user's reference example:

```text
What number is ONE MORE than 9?

[ 8 ]   [ 10 ]

[ Check answer ]
```

Requirements:

- Tapping a choice selects it. A visible outline and selected label identify it.
  Tapping another choice changes the selection. Nothing submits automatically.
- Use two options for simple early contrasts; otherwise use three or four.
  Never add implausible options simply to fill a quota.
- Use native radios, buttons and labelled inputs. Enter checks a valid typed
  answer; Space/arrow keys retain their normal control behaviour.
- Controls have at least 44px targets; primary choice cards target 56px minimum
  height. At 320px width, the page has no horizontal overflow. Long passages may
  scroll vertically, with the question and answer reachable normally.
- Pictures that carry meaning have useful descriptions. Descriptions must provide
  equivalent task information without stating the correct response. Decorative
  character art has empty alternative text.
- No colour-only instructions or results. Respect reduced motion. No timers,
  score penalties, confetti covering controls, compulsory sound or speed scoring.
- Keep focus stable while editing. Announce feedback through one polite status
  region. Move focus to the next question after an explicit next action and
  restore world focus after completion.
- A confirmed correct answer closes automatically. Show a brief nonblocking
  world status such as “Nice work!”; do not add a compulsory Continue screen.

## Shared controls and contract

Use one current question format with a discriminated interaction and a bounded
stimulus. Replace the mission draft wire shapes where useful; update every
consumer and fixture together. Do not extend the unrelated legacy assessment
contract just to make it resemble the new UI.

Add `exerciseId` to the public activity. Keep `skillId`, `variantId`, `contentVersion`,
support and story binding. Use exercise IDs such as `M05.one_more` and
`MY110.read_half_hour`. The following are implementation requirements, not
permission to pass arbitrary HTML or arbitrary drawing code:

| Interaction kind | Public data and answer | Marking |
| --- | --- | --- |
| `single_choice` | 2–4 options with IDs and bounded text or asset/diagram data; answer `{kind: 'choice', choiceId}` | Compare against private correct ID |
| `multi_choice` | 2–6 options; answer `{kind: 'choices', choiceIds}` with unique IDs | Compare sets; no partial automatic success |
| `number_entry` | 1–3 labelled fields with IDs and format `integer`, `decimal` or `fraction`; answer `{kind: 'numbers', values: [{fieldId, value}]}` where value is a bounded string | Parse exactly in Rust, compare against private values and any explicitly required form |
| `text_entry` | One labelled field, maximum 120 characters; answer `{kind: 'text', value}` | Authored accepted spellings and item-specific case/punctuation policy |
| `order` | 2–8 uniquely identified cards; answer `{kind: 'order', itemIds}` | Match private accepted order; explicitly model alternatives where needed |
| `assign` | Up to 20 objects, up to 6 target IDs; answer `{kind: 'assign', assignments: [{objectId, targetId}]}` | Validate identities/uniqueness, then pairing, category or equal-sharing rule |
| `workshop` | Prompt, scaffold, rubric and human-review notice; no automatic answer command | Existing supported/not-assessed completion only |

Number strings are at most 32 characters. Reject unknown fields/IDs, duplicate
IDs, nonfinite values and invalid formats. A malformed answer produces a field
error and no incorrect learning observation. A valid wrong answer is assessed.
Whitespace trimming is permitted; do not use fuzzy matching to mark spelling.
Decimal parsing uses scaled integers; fractions use rational arithmetic with a
nonzero denominator. Accept equivalent forms unless the question explicitly asks
for a particular form. Mixed-number questions use labelled whole/numerator/
denominator fields. Units belong beside fields, not inside the required number.
Do not use JavaScript floating-point equality as an assessor.

Use a bounded stimulus union for text, passage, collection, ten frame, number
line, place-value model, shape/position scene, equal-part model, clock, measure,
grid/map, graph/table, word/sentence tokens and reviewed audio reference. A text
stimulus can carry a structured numeric expression where necessary. Add the
individual stimulus definitions when their delivery batch needs them. Use SVG/DOM
for deterministic educational diagrams. Do not build a generic scene engine.
Selectable diagram elements and sentence spans use the same choice IDs as the
answer controls. Provide an accessible textual/table equivalent where appropriate.

Keep answers, accepted sets, misconception rules and worked solutions private to
Rust. The public payload contains only what the child needs before checking.
Generate models with `python3 scripts/generate-contracts.py`; never hand-edit
the generated files. Add bounds in the shared schema, not just TypeScript types.

Persist the current `exerciseId` and full deterministic seed in the active task.
Derive `variantId` from content version, exercise ID and full seed; do not identify
questions using `seed % 24`. Wrong-answer selection must persist the replacement's
exercise and seed together. Existing owned-receipt checks may reconstruct a
previously issued question in the current format, but must verify the recovered
public task before marking it. Retired draft formats can be cleared from the
synthetic database. Test the existing receipt/checkpoint size limits with the
longest passages and diagrams; do not raise them without a measured need.

## Question factory and authored content

Create `content/activities/skill-exercises-v1.json` as a small internal inventory:
skill ID, exercise IDs, interaction, permitted representations, difficulty rungs,
assessment mode and authored item references. Load it into the pure learning
crate. It is not a public answer-key endpoint or a general content platform.

Split the replacement catalogue into `mission/exercises/` modules for number,
operations, fractions, measurement/space, data/algebra and English. Keep a thin
dispatcher. Return the public question, private assessment key, hints, worked
example and feedback from one deterministic factory.

For each exercise family in the checklist:

- Supply introductory, standard and stretch parameters where the objective
  supports them. Initial story selection uses introductory or standard items
  deterministically; stretch items are available to the factory/tests. Do not
  add persistent difficulty adaptation in this task.
- Automatically marked numeric families must produce at least 12 different
  semantic items across a documented seed sweep, or exhaust their valid domain
  when smaller. Cover lower/upper bounds and at least two allowed representations
  per skill where those genuinely assess the same ability.
- Authored word/sentence families need at least six distinct items per family.
  Passage skills need at least three passages of the required length, with at
  least two questions per passage collectively covering their required facets.
  Rewording names, option shuffling and cosmetic artwork do not count.
- Workshops need two distinct prompts per skill and a concrete review rubric.
- Use plausible mistake-based distractors. Remove duplicate/equivalent choices.
  Shuffle correct positions deterministically; test that no correct slot is fixed
  throughout the sample. Never expose the private correct-first authoring order.
- Store a canonical semantic fingerprint containing exercise, operands/source
  item and representation-relevant content, excluding option order and decoration.
  Different card ordering must not count as fresh evidence.
- Validate numerical relationships as well as per-field bounds: sum limits,
  nonnegative subtraction, divisible shares, consistent graph scales, equal
  fraction partitions and valid angle totals.
- Provide a strategy hint, a concrete second hint and a worked explanation tied
  to the actual issued question. Never regenerate help with a different seed.

Keep all auto-marked facets available, but classify broad skills honestly. A
grammar choice does not assess independent writing; word recognition does not
establish oral decoding; a short answer does not establish fluency or discussion.

### Resolve contradictory draft constraints

Before implementing a family, compare its objective, bounds and authoring pack.
Apply these explicit corrections together with validator fixtures and intentional
integrity-hash updates; retain pending reviews:

- `MY104`: independently bounded addends must also have a sum no greater than 20.
  `MY105`/`MY202`: reject negative results and results outside the stated range.
- `MY302`: increase the draft operand maximum to 9999 to implement “up to
  four-digit numbers”; include both three- and four-digit items.
- `MY401`: allow 1000000 to include the named million boundary.
- `MY404`/`MY504`: represent decimal bounds explicitly as scaled integers with
  precision metadata in the exercise specification; retain the node's numeric
  value range 0–9. Test hundredths/thousandths without binary float ambiguity.
- `MY305`: the objective says unit fractions but also mentions ordering fractions
  with the same denominator. Implement unit-fraction recognition/placement plus
  same-denominator proper-fraction comparison as separate exercises. Describe the
  distinction in coverage; keep the original node pending educator clarification.
- Other unresolved pedagogical contradictions remain explicit coverage limitations;
  do not silently expand a node or claim full assessment.

## Feedback and story behaviour

Keep the server's same-skill question replacement after a wrong answer. Extend
`lastFeedback` with bounded optional `exerciseId`, `variantId` and `explanation`
fields identifying the submitted question, not the replacement.

1. Under the existing transaction, assess the issued question and record the
   check with cumulative support history.
2. On a wrong answer, save a genuinely different question of that skill and the
   submitted question's short explanation in the same receipt.
3. The UI shows the explanation before an explicit local **Try another** button
   reveals the replacement. This button sends no command. A reload may show the
   saved replacement directly, with feedback still available; it must not choose
   yet another item.
4. The explanation is committed teaching. Keep the encounter's first-check and
   support history intact; a later success is not a fresh independent first check.
5. Correct and together-completions retain atomic learning/story commits and
   automatic closure. Pending writes retain their exact body/ID across reload.
6. Keep existing walking/save recovery, stale-variant protections and legal story
   binding checks. Update them for the current format; remove obsolete historical
   factory dispatch rather than building compatibility with retired draft banks.

If the finite domain is exhausted, reuse another existing semantic item. Do not
fail a child's answer merely because there is no never-before-seen question.
Require the replacement to differ from the immediately preceding question.

## English audio and human review

Use locally bundled, reviewed recordings for listening and sound questions. Add
`content/activities/audio-v1.json` with asset ID, local path, transcript,
pronunciation/dialect note and review status. Only reviewed assets can play in
an independently marked audio exercise. No browser TTS or new external provider.

Build replay controls and missing-asset handling with synthetic audio fixtures.
Until actual recordings are supplied/reviewed, keep the affected exercises on
the human-supported route and record that limitation. Do not silently substitute
written words for a listening assessment or claim all audio content delivered.

Read-aloud of maths instructions is an access aid. Reading an independently
assessed passage is support; listening passages are the task stimulus. For this
delivery, avoid a general narration/evidence-policy rewrite: offer audio only in
explicit authored listening/sound exercises, plus the existing human-supported
routes. A general “Hear every question” feature is deferred.

Keep the existing optional answer dictation only for eligible fields. Reviewed
transcripts fill an editable answer and still need Check. Dictation cannot count
as independent spelling, phonics, decoding or oral-fluency evidence. Workshop
drafts remain local and clearly warn before discarding; do not add cloud draft
storage or claim a completed adult-review workflow.

## Implementation sequence

Complete these work packages in order. Each ends with working code and the
specified checks; do not stop after building only the reusable shell.

| Package | Exact deliverable | Acceptance |
| --- | --- | --- |
| 1 Inventory | Add exercise inventory for every checklist row; add coverage validator; reconcile the listed bounds | Exactly 120 unique known skills, no missing/extra IDs, every listed facet has an exercise or named human-review/audio limitation |
| 2 Controls | Simplify existing choice screen first; add text/number entry, ordering, assignment and multi-select controls, using synthetic questions | Real tap/keyboard/touch tests; no choice tray; no required drag; explicit Check; narrow-screen screenshots inspected |
| 3 Server and contract | Update schemas, generated models, assessors, feedback, repository and command preparation; replace draft factory dispatch | All new answer kinds assessed in Rust; public payload has no keys; exact retries and wrong-answer replacement tested against disposable PostgreSQL |
| 4 Pilot | Implement `M01`, `M03`, `M05`, `M07`, `MY110`, `EY106`, `EY103`, `E13` completely | Matching, counting, one-more, calculation, clock, spelling, passage and workshop work through real story integration; no mastery invented |
| 5 Early content | Finish all remaining Foundation, Year 1 and Year 2 checklist rows | Required item counts, bounds, answer fixtures and UI/assessment coverage pass for every row |
| 6 Later content | Finish Years 3–6 rows; add remaining diagram types and exact rational/multi-field questions | Fractions, decimals, graphs, maps, longer passages and writing scaffolds meet checklist and length requirements |
| 7 Final verification | Run aggregate, database and browser checks; update READMEs, coverage, status and ADR | Completion report lists every implemented family and any missing reviewed assets/human checks; no unsupported full-skill claims |

The pilot validates the architecture, not the entire delivery. Packages 5–7 remain
required. Keep PRs/commits focused if delivering incrementally; clearly label
partial coverage instead of presenting the pilot as all 120 upgraded skills.

## Development data reset

Only do this if new active-task/checkpoint formats make existing synthetic saves
incompatible. The user has authorised that reset for this prelaunch work.

Stop the API and verify the target is the repository's local synthetic demo.
Using an operator connection, clear that profile's learning events, learning
receipts, learning progress and mission-learning command window in one transaction.
Inspect foreign keys and `command_windows` columns for deletion order. Clear the
corresponding browser pending mission-learning command as well. Leave story,
inventory, artwork and unrelated profiles alone when a targeted reset suffices.

If rebuilding the entire synthetic demo is simpler, that is authorised too;
document what was cleared and how to restart. No reset service, UI or reusable
reset framework is required. Never silently clear data on ordinary startup.
Do not edit applied SQL migrations to hide drift; add a migration only if actual
table structure changes require one.

## Required regression coverage

- **Inventory:** compare the real graph ID set with the exercise inventory and
  implemented factory set. Test every checklist facet, supported representation,
  authored item count and passage length. A factory-exists assertion is insufficient.
- **Maths:** exhaustive tiny domains and seeded boundary sweeps for larger ones;
  independently calculated expected answers, mistake distractors, equal groups,
  equivalent fractions, decimal precision, remainder fields, units, graph scales,
  clock hands, valid maps and angles. Test zero only where the node permits it.
- **English:** accepted alternatives, taught graphemes, case policies, malformed
  text, punctuation tasks, unambiguous comprehension evidence and rubric-only
  productive tasks. Test missing/unreviewed audio never manufactures success.
- **API/database:** each answer kind; reject IDs from another question; duplicate
  submission; same choice ID in different variants; stale response; lost response
  and exact retry; reload; committed support; wrong-answer replacement; together
  completion; rollback; ownership; export/deletion; journal replay.
- **Browser:** at 1280px, 390px and 320px; mouse/touch/keyboard; editing and undo;
  visible selection; Enter check; busy/pending controls; destruction/cancellation;
  feedback then replacement; automatic closure; focus; malicious text rendered
  literally; long passages and diagrams. Verify representative examples of every
  interaction and stimulus, rather than 120 nearly identical screenshot tests.
- **Usability:** provide a short manual playtest checklist for educators/adults
  and later supervised child sessions: can they identify the task, answer, change
  an answer, get help and resume play? Record actual observations only. Automated
  Chromium tests do not establish screen-reader or iPad Safari usability.

## Verification commands and completion

Use Node 24 and the pinned Rust toolchain. Run focused checks during each package,
then the aggregate suite once the full change is ready:

```sh
python3 scripts/generate-contracts.py
cargo test -p wonderwilds-learning --locked
python3 content/activities/validate.py
python3 -m unittest discover -s content/activities -p 'test_*.py'
bash scripts/check.sh
sh scripts/check-postgres.sh
```

The activities validator/test paths above are new deliverables from package 1;
wire them into `scripts/check.sh` and `.github/workflows/checks.yml`.

With Vite running at `http://127.0.0.1:5173`, and `CHROMIUM_PATH` set when needed:

```sh
npm test --prefix qa/e2e
node frontend/src/learning/activity.browser.mjs
node frontend/src/mission-learning/activity.browser.mjs
WW_QA_BROWSER_TEST=qa/e2e/mission-learning.mjs node qa/e2e/run-training-qa.mjs
node qa/e2e/run-training-qa.mjs
git diff --check
```

The QA runner uses fresh synthetic PostgreSQL on separate ports. Set
`WONDERWILDS_PG_BIN` for local PostgreSQL discovery. Run the standard authored
story journey as well as the learning journey; never run destructive fixtures on
the persistent demo. `qa/e2e`'s current `npm test` is a smoke check, not the entire
browser regression suite.

Update `docs/ACTIVITY_COVERAGE.md` from actual implemented exercise coverage,
`docs/IMPLEMENTATION_STATUS.md` with checks actually run and remaining limits,
the mission-learning README, and `docs/adr/curriculum-starter-practice.md` for the
replaced draft model. Update curriculum integrity hashes only for intentional
curriculum edits. Finish by inspecting the diff and reporting any blocked checks.

Delivery is complete when all 120 rows have the specified usable exercises or
explicit honest human-review/audio routes, the shared controls work in the story,
and the required automated checks pass. Reviewed recordings, educator approval
and child playtesting must remain separately visible if unavailable.
