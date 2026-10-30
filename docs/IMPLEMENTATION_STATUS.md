# Wonderwilds implementation evidence

Foundation evidence recorded 28 October 2026; integration updates dated below. Broader roadmap: [Wonderwilds implementation plan](WONDERWILDS_IMPLEMENTATION_PLAN.md) v1.2. Active delivery sequence: [Implementation plan](IMPLEMENTATION_PLAN.md), with [curriculum mission integration](CURRICULUM_MISSION_PLAN.md) implemented for the first five-skill path. Validation counts in earlier sections describe their original delivery, not the latest checkout.
Six subagents implemented distinct scopes; the integration owner reviewed and
validated the combined changes. Existing source, guest saves and art are preserved.
No release, deployment, curriculum approval or learning-efficacy claim is made.

## Authored curriculum starters and accessible play controls — 31 October 2026

All 120 draft curriculum nodes now have a deterministic activity factory: the
existing five assessed maths activities, 93 additional focused answer-card
starters and 22 guided workshops needing human review. Rust owns selection,
parameters, distractors and assessment; no activity or learning progression is
LLM-generated. Original readiness requirements remain unchanged. After that path,
new starter selection follows prerequisite exposure and prefers unexplored,
earlier-year content. Correct practice or a committed worked example can record
exposure; hints/abandonment alone cannot. Exposure never marks a new skill ready
or mastered. See [activity coverage and IXL research limits](ACTIVITY_COVERAGE.md)
and [the decision record](adr/curriculum-starter-practice.md).

Mission matching now moves objects into recipient places with drag, tap or native
keyboard buttons instead of dropdowns. Delivery rows and recipient-slot grids
retain distinct layouts. Counting pieces move between labelled trays. Answer
cards support dragging, native radios and optional speech-to-text. The story host
now wires the existing Muse/Baseten recorder into these controls; transcripts
remain editable and need explicit use and a separate check. Large integers,
fractions, simple decimals and common symbols have deterministic spoken forms;
ambiguous matches preserve the answer. Recording freezes editing, and closing or
destroying the screen cancels recording and late transcripts. There is no TTS.

Guided writing/discussion tasks provide a local draft pad and optional dictation,
with explicit human-review instructions. The pad is discarded when closed and is
not submitted or assessed. Decoding, phoneme production and oral fluency use
human-observed tasks; STT is not a pronunciation or reading assessment. Supported
inline completion retains the existing atomic story/evidence transaction. New
starter events remain in the immutable journal; a bounded exploration set avoids
growing the original <=60 qualifying-observation projection. Old five-skill
checkpoints omit the empty new field and replay identically. No migration,
persistent demo reset, review approval or release activation was introduced.

Validation: aggregate repository checks passed, including Rust formatting/Clippy,
workspace tests, 155 shared contract fixtures/generated checks, content validators,
frontend checks/build and legacy fixtures. Final frontend checks include 123
unit/API tests. Learning tests validate all 120 factories across 24 seeds, private
keys, authored arithmetic examples, supported exposure, grade ordering,
reachability and legacy replay. The complete disposable PostgreSQL suite passed;
the final focused mission test also passed issuing, resuming, teaching and saving
all 115 new starters, exact retries, ownership, replay, export and deletion.
The real isolated API/browser story journey passed independent/wrong/supported
answers and lost-response recovery. Existing story smoke and six-family activity
browser checks passed. New desktop/phone checks passed real mouse/touch drags,
keyboard/tap matching, invalid drops, literal untrusted text, long-label wrapping,
mocked transcription review and cancellation. Screenshots were inspected. CI runs
the new component browser suite alongside the existing suites.

Limitations: many starters sample only one facet or one canonical example; they
are not full-objective or transfer assessments and may use shorter examples than
the curriculum pack's draft assessment ranges. Richer manipulatives, additional
representations/variants, educator calibration, child/assistive-technology
playtesting and reviewed audio remain outstanding. IXL research used public year
catalogues; interactive item-by-item parity was not verified. No live Muse/Modal
calls or child-speech-quality checks were made. Node 26.12.0 was available rather
than Node 24; the existing production bundle-size warning remains. Earlier
sections below describe their original delivery scope.

## Two content modes and a single app entry — 31 October 2026

The app now always opens the Phaser story world at `/`. Guest and older
assessed-adventure boot paths and mode-switch links have been removed from the
entry point. Legacy `?guest=1` / `?story=1` parameters have no effect; the old demo
and inspector HTML pages redirect to `/`. Supporting API/domain modules remain
for assessment and internal regression coverage, not separate playable modes.

No Modal credentials selects authored scenes and missions. Both proxy credentials
select generated scenes, mission planning and repeatable neighbourhood stories
together. Partial credentials fail configuration. Separate story/planner/exploration
environment switches are retired. `WONDERWILDS_MODAL_ENDPOINT` is the single
restricted endpoint override for all three providers. Existing validation, timeout,
authentication and authored fallback boundaries remain. Both modes require the
backend/database; no standalone guest mode is offered.

At the user's request, this checkout's `wonderwilds_demo` database was dropped,
recreated, migrated and seeded with a fresh synthetic profile/session. No new
migration or historical compatibility-removal project was needed for this change.
The reset also issues a new profile ID so old browser view state and uncertain
commands cannot attach to the fresh game. Session renewal preserves the new ID.
Earlier sections below record historical entry points and flags.

Verification: frontend check passed (strict types, lint, formatting, 121 tests and
production build); API tests passed (38, with four database tests ignored), API
Clippy and workspace formatting passed. Desktop/phone synthetic checks covered
all entry URLs and source labels. The real authored Phaser journey passed against
a disposable database, including inline learning and save recovery. The generated
neighbourhood journey passed against a loopback mock with six arrival requests,
repeat generation, malformed output, stable saves and exact retries; startup
confirmed all three providers enabled without flags. Accessible activity browser
checks passed. Demo reset/session-renewal checks confirmed a fresh stable profile
and zero story checkpoints. Screenshots were inspected and `git diff --check`
passed. Node 26 was available; live Modal inference, hosted CI and the full
standalone PostgreSQL integration suite were not rerun for this configuration/UI
change. The existing bundle-size warning remains.

## Repeatable generated neighbourhood stories — 31 October 2026

Arrival-time generation now invents a character name/personality, greeting, title,
premise, one to three sequential local investigation steps, and a payoff. The
model chooses existing character/object art; this does not generate new map or
image assets. Entering a new or completed neighbourhood requests the next story.
There is no fixed mission-catalogue limit on repeat visits across the five current
neighbourhoods. Unfinished and archived stories remain stable until completed.
Three open neighbourhood slots continue to reserve Pip's fourth slot.

Arrival generation now follows the shared Modal credential selection. Model calls run outside database locks with closed-schema
output, bounded requests, no profile/session/learning context, and authentication
plus revision checks before commit. Only supported investigation steps are exposed.
The internal generated-travel command commits content with the arrival receipt;
it is not a public browser action. Exact retries reuse saved content. Malformed,
busy or unavailable model output commits a labelled offline fallback. The hot
journal keeps only the latest content per area; previous versions remain in durable
command history. No new SQL migration, save reset or release/content approval.

Verification: repository aggregate checks passed, including 143 shared fixtures,
121 frontend tests/build and Rust/content checks. The real PostgreSQL suite passed
all four integration tests, including eight successive generated missions, rollback,
exact retries, fresh-reader consistency and full history replay. A disposable real
API/Phaser journey used a loopback mock model: six arrival requests covered generated
names/art, sequential tasks, the four-mission cap, archive/resume, completion,
reload/retry stability, invalid-output fallback and a second generated story on
return. Desktop/phone screenshots were reviewed. The provider HTTP test verifies
closed-schema requests and generated responses. Targeted regressions reject wrong
step IDs, replacement of unfinished stories and legacy-version claims of generated
travel. CI includes this mock-model journey; hosted CI was not run here.

Live generation quality and latency on the deployed model were not exercised.
Physical geography remains finite and mechanics are limited to supported local
investigation tasks. Node 26.12.0 was used instead of Node 24; the existing bundle
warning remains. Persistent demo data and the pre-existing badge-art edit were
preserved. Restart the local backend to activate the new generator.

## Story neighbourhood travel and bounded missions — 31 October 2026

Pip's playable story now supports walking onto server-offered map-edge routes.
The workshop connects to Starlight Cove, Fern Hollow, Mossy Clearing, Owl Grove
and Acorn Trail. Five new named neighbours (Marina, Fern, Moss, Orla and Hazel)
have defined accept/investigate/report missions, with provisional reused artwork.
Their journal statuses and checked tasks persist in the existing story checkpoints.

Three neighbourhood slots reserve a fourth for Pip. Starting/resuming another
archives the oldest open neighbourhood mission without losing its checked task;
returning to its character resumes it. Completed missions remain complete. The
pure Rust domain controls adjacency, task completion, cap and archive order.
Neighbourhood actions are appended after historical bindings, empty new state is
omitted to preserve old checkpoint bytes, and older content versions cannot claim
new travel. No applied migration, persistent demo data or guest evidence changed.

The optional scene provider receives the new characters, legal actions and bounded
mission journal, and may generate dialogue. Five defined missions are shipped;
endless/model-invented missions and generated neighbourhood mission plans are not.
Pip's planner proof excludes unrelated detours. No content or release approval is
implied. Restart the local backend to use new locations; no save reset is needed.

Verification: aggregate repository checks passed (137 shared fixtures, 120 frontend
tests/build, Rust and content checks), followed by targeted checkpoint-corruption
coverage and workspace Clippy. Real PostgreSQL checks passed all four integration
tests, migration replay and hash-drift rejection. The disposable exploration browser
journey passed keyboard travel, all five missions, archive/resume, completion,
reload and exact lost-response retry. Screenshot review found and corrected phone
journal text clipping; the final exploration rerun passed an explicit dialog-width
assertion and desktop/phone screenshots were reviewed. The original Pip bank-source
world journey also passed, including shared crossing, lost-response recovery and
separate guest/education state. CI includes the new exploration journey; hosted CI
was not run here. Node 26.12.0 was used; Node 24 and live model-generated dialogue
were not verified. Existing Phaser bundle-size warnings remain. `git diff --check`
passed. The pre-existing badge artwork edit in `frontend/src/world.ts` was preserved.

## Curriculum practice inside missions — 31 October 2026

The story Missions panel now offers profile-owned curriculum practice bound to
the actual active mission and location. Rust loads all 120 Foundation/Years 1–6
graph nodes, selects a prerequisite-ready supported skill, assesses answers and
advances the practice frontier. The first path covers M01 one-to-one matching,
M03 counting, M04 comparison, M05 one-step sequences and M06 part–whole tasks,
with two representations per skill. The remaining 115 nodes are explicitly
unavailable until their activities are authored. English remains in the graph.

Migration `0006_mission_learning.sql` persists resumable activities, immutable
evidence, bounded current projections and exact original-response receipts with
profile locking, forced RLS, export and deletion. Reload preserves uncertain
commands for exact retry. Hints/examples appear only after support is committed.
Story completion and resources, assessed-adventure progress, and guest saves
remain separate. Learner history is not sent to the story provider.

The separate `provisional-mission-v1` policy requires eight distinct independent
first checks (seven correct, latest three correct), varied variants/representations,
two contexts and two verified server sessions. It represents readiness for more
practice, not validated mastery. Canonical-item replacement allows later-session
review without increasing the distinct-item count. Existing transfer-based mastery
is unchanged. Activities use original content informed by public IXL catalogue and
teaching patterns; the [integration plan](CURRICULUM_MISSION_PLAN.md) records sources
and the limits of that research.

Three lower-tier implementation subagents handled learning, API/contracts and UI;
the integration owner reviewed their work and added an independent full-path
assessment/replay regression. The aggregate `bash scripts/check.sh` passed,
including 130 shared contract fixtures, 118 frontend tests, the production build,
curriculum/story validators and guest-save compatibility. Guest desktop/tablet,
existing accessible-activity browser checks and the bridge routing/security check
passed. The new disposable-database browser journey passed wrong/correct pairing,
committed support, lost-response reload/exact retry, independent progress, phone
layout and continuing story actions without altering guest/game state. Existing
companion training and the default seven-story browser journey also passed.
The targeted real-PostgreSQL mission test passed all five skills across two
verified sessions and two story locations, with exact retry, rollback, races, RLS,
full evidence replay, unchanged assessed-adventure state, export and deletion.
The wire regression validates and assesses answers for all 60 authored variants,
including the corrected camelCase numeric-choice field. The final combined
`sh scripts/check-postgres.sh` run passed all four database integration tests,
repeat migration application and deliberate migration-hash drift rejection.
The final browser rerun also verified the ten-frame’s five-column/two-row layout.
All temporary QA clusters and servers were shut down.

Remaining work: expand the other 115 activities in prerequisite order, calibrate
learning/transfer policy, improve mission-specific placement through playtesting,
and add production identity/session renewal, retention and operator recovery.
The existing persistence CLI does not cover this new practice journal; full replay
is currently exercised by integration tests. Node 26.12.0 was used rather than the
documented Node 24. No live provider-quality claim, educator approval or release
approval is added, and the persistent `.local-demo` database was not migrated.

## Long-play persistence — 31 October 2026

Migration `0005_long_play.sql` implements durable API/domain receipts, archived
encounters and reconstruction metadata, bounded current snapshots and two verified
checkpoints for both assessed adventure and story. PostgreSQL play no longer
stops at 2,000/256 commands. Exact retries retain original responses, including
old terminal story responses; the standalone CLI retains its cap. Lazy migration
preserves supported pending training and legacy story versions. Guest saves are
unchanged.

Current reads verify checksums and the latest transition. Offline `persistence`
audit/recovery replays from the preserved origin to the latest committed revision;
missing history fails closed. Adventure audit independently recomputes learning
from retained evidence. The incremental projection is restricted to the current
no-transfer authored policy, with golden comparisons across support and sessions.
New records participate in forced RLS, export and deletion. A renewable committed
command budget and 64 KiB current-snapshot bound replace lifetime play budgets.

Verification: `bash scripts/check.sh` passed, including 123 shared contract
fixtures, frontend build/tests and legacy guest-save checks. Final API regression
coverage passed 33 tests (the three database tests are exercised separately),
with workspace Clippy, formatting, JavaScript syntax and diff checks passing.
The >2,000-command browser adventure passed supported pending training across an
API restart and a lost completion response/exact retry after another restart.
Both >256-command story branches passed seven missions, exact lost-response retry
and all three fresh API/browser returning sessions. Authored providers were used;
this storage delivery does not add live model-quality or latency evidence.
Default/alternate story read medians were 17/16 ms, with 32/20 ms p95 on this local
synthetic run. These are HTTP timings, not production capacity targets.

The final `sh scripts/check-postgres.sh` run passed all three real-database
integration tests and migration drift checks. Adventure reached revision 2,104:
hot snapshot size 1,662 → 1,672 bytes across 2,100 additional writes (66.14 s).
Story reached revision 307: 1,970 → 1,954 bytes across 300 additional writes
(3.33 s). Both retained exactly two checkpoints. Checks also passed for legacy
terminal-at-256 continuation, supported pending training migration/rollback,
exact old responses, races/unique grants, full audit and independent learning
recomputation, damaged-checkpoint recovery, refusal of missing/corrupt history,
rate limits with retry exemption, forced RLS, export and deletion. These are
local synthetic measurements, not constant total storage or production-load
claims. See [storage design and operator commands](LONG_PLAY_STORAGE_PLAN.md) for formats,
compatibility and reproduction. Tests used Node 26.12.0 rather than the documented
Node 24. The persistent `.local-demo` was not migrated. Total archive growth,
production backup operations, retention approval and human content/release review
remain open.

## Combined-model returning-session verification — 31 October 2026

The existing seven-story Phaser journey now has opt-in combined-provider QA and
an authored rehearsal mode. Both branches restart the API and open a fresh browser
context after the trilogy, during the paused corner mission, and during gathering
preparation. Checks compare the full canonical scene input, revision and saved
plans across restarts; a lost committed exchange response is recovered by reload
and exact-body retry. The same synthetic authenticated session is reused; this is
not production login/session-renewal verification.

The live gate requires seven saved generated plans and accepted scene responses in
each of four sessions. Logs distinguish accepted scenes, authored fallback and
stale results; scene counts include cache hits and diagnostic reads, not unique
inferences. Timings measure HTTP reads and UI actions through refresh, not isolated
model latency. Reproduction is in [browser QA](../qa/e2e/README.md).

API regression coverage now verifies that scene generation receives the persisted
brief, scene outage preserves that brief, and exact acceptance retries retain the
original receipt without replanning. Verification also found a duplicate scene
`response_format` field and inconsistent schema-name assertions; the duplicate
was removed while retaining the effective `story_scene` request name and schema.

Both live branches subsequently passed all seven missions and all three restarts,
with seven saved generated plans each. Default/alternate recorded 216/223 accepted
scene responses and 10/9 scene fallbacks, with no planner fallbacks in the successful
runs. Both authored rehearsals and desktop/tablet guest smoke also passed. API
checks passed 32 tests (three DB tests ignored), targeted Clippy, Rust formatting,
JavaScript syntax and diff checks. Real disposable PostgreSQL was exercised by the
browser runs; the separate DB and full aggregate suites were not rerun. Node was
26.12.0 rather than 24.

The initial endpoint-503 attempt and an alternate run's first-workshop travel
timeout are retained separately. The diagnostic alternate rerun passed, but the
intermittent timeout cause remains unresolved. Combined branch/resumption coverage
is delivered; M1 reliability is not unconditionally closed. See the
[results and saved summaries](../qa/story-eval/RESULTS_COMBINED_2026-10-31.md) for
counts, timing limits, failures and reproduction. Content/release gates remain
unchanged; this is not an editorial or unseen-mission evaluation.

## Schema-constrained model requests — 31 October 2026

Both live Modal request paths now include `response_format.type = json_schema`
with a named contract schema and `strict: true`. Planning includes its referenced
`StoryNarrative` definition; scenes use the closed `StoryScene` schema. Local JSON,
binding and mechanical validation and authored fallback remain unchanged. There is
no retry that silently removes the constraint.

New planner evaluation runs default to schema constraints while legacy manifests
remain verifiable as prompt-only. Paired ordinary and reversed-condition runs each
passed 18/18, with unchanged prompts and scenarios; Bramble ordering still passes.
A new current-endpoint scene runner uses the exact live prompt and a deterministic
36-state mission/status/location/introduction sample. It passed 35/36: one narration
was 210 characters against the contract's 180-character maximum. All requests
returned JSON without transport failures. A conflicting-prompt constant probe
confirmed actual server enforcement; a simple string-length probe passed, so the
observed nested scene violation is not evidence that every string bound is ignored.
These results do not establish full JSON Schema compliance by the serving backend.

API regression tests (31 passed, three DB tests intentionally ignored), all 53
Python evaluation tests, targeted Clippy, formatting and diff checks passed.
This change does not alter persistence; the prior database integration evidence
still applies. [Results and reproduction](../qa/story-eval/RESULTS_JSON_SCHEMA_2026-10-31.md)
record the unresolved scene constraint violation. No fine-tuning was performed.

## Mission planner game integration — 31 October 2026

The opt-in API planner (`WONDERWILDS_MODAL_PLANNER=1`) now generates briefs when
accepting a new supported mission. It uses the same evaluated `dependencies-v1`
prompt files and bounded domain validator as the offline harness. Generation runs
outside database locks; session/CSRF and revision are rechecked before commit.
Migration `0004_story_mission_plans.sql` saves validated plans atomically with the
acceptance checkpoint and original-response receipt, with forced profile RLS,
export and cascading deletion. Reload, pause/resume and exact retries retain the
saved plan. Existing active missions retain their authored brief.

Both existing story clients consume saved goal/narrative/milestone projections;
optional scene generation receives the same brief. Completion labels derive only
from domain receipts. Invalid/unavailable/unproven proposals use authored fallback.
This is planning over seven authored opportunities, not arbitrary new mechanics or
an endless catalogue; model ordering does not add action gates to parallel tasks.
No model weights or content-release approvals changed.

Validation: `bash scripts/check.sh` passed, including API tests (31 passed, three
DB tests deliberately ignored), director tests and the 51 Python evaluation tests.
`sh scripts/check-postgres.sh` separately passed all three database tests and the
migration/drift checks in a disposable cluster. Added coverage includes invalid
plans, outages, session expiry, revision races, exact retries, state-bound plans,
search deadlines, persistence rollback, concurrent acceptance, RLS, history,
export and deletion. Node 26.12 was available for tooling rather than Node 24.
The ordinary seven-mission Phaser browser journey passed. After a first live
attempt correctly fell back during HTTP 503 availability failure, the recovered
endpoint generated seven validated plans and the full live browser journey passed
with `WW_STORY_EXPECT_PLANS=1`. It asserted a persisted plan and matching Missions
UI for each acceptance, seven saved plans after pause/resume, unchanged plans and
briefs after reload, all seven completed missions, mobile history and unchanged
educational progress. Scene generation was disabled to isolate planning. Local logs:
`/tmp/ww-plan-browser-live-r2.log` (seven validated plans),
`/tmp/ww-plan-browser-fallback.log` (ordinary journey),
`/tmp/ww-plan-postgres.log` and `/tmp/ww-plan-check.log`. This single default-choice
live journey is integration evidence, not an unseen-mission or editorial evaluation.

## Mission planner dependency prompt — 31 October 2026

The offline planner now defaults to the versioned `dependencies-v1` prompt,
which derives milestone ordering from capability prerequisites, follows dependencies
recursively and checks the sequence against starting facts, inventory and location.
It contains no Bramble-specific solution. `--planner-prompt baseline` preserves
the original prompt; old manifests without a prompt selector remain verifiable.

Both new 18-case runs have identical replayable scenarios and user messages to
their corresponding ordinary/reversed baselines. Only the system message changes.
The 51 Python evaluation tests and diff checks pass. No Rust, gameplay, model
weights or deployment configuration changed.

After endpoint recovery, both revised-prompt batches passed 18/18 mechanical
checks. The ordinary baseline was 18/18; the reversed-condition baseline was 17/18.
The sole improvement is `bank-series-062`: path checking and clearing now precede
Bramble's readiness, with no mechanical regressions elsewhere. All 36 responses
had valid schemas and replayable completion witnesses. Earlier 72 failed requests
remain separate HTTP 503 availability evidence, excluded from quality results.
[Comparison results](../qa/story-eval/planning/RESULTS_DEPENDENCIES_2026-10-31.md)
record controls, hashes and remaining editorial limitations. Raw evidence remains
under ignored `qa/story-eval/experiments/` directories prefixed
`planning-2026-10-31-dependencies-ordinary` and
`planning-2026-10-31-dependencies-reversed`; successful responses use `-live-r3`
and grades use `-r3-grades`. No additional fine-tuning was performed. These are
paired results on known authored families, not an unseen-mission evaluation.

## Mission planning evaluation — 31 October 2026

The separate [offline planning harness](../qa/story-eval/planning/README.md)
exports 18 distinct replay-checked situations across seven authored mission
families, including partial progress, existing planks, repaired bridge and
contrasting lasting history. Shared-schema/binding checks and bounded Rust domain
search verify ordered milestone completion and replay successful witnesses.
Model requests exclude reference answers and checkpoints; raw responses, timing,
coverage, hashes, failures and escaped review reports are preserved locally.

The current deployed `wonderwilds` alias passed 18/18 ordinary cases and 17/18
paired reversed-condition cases. The failure asks Bramble to become ready before
clearing a path that readiness requires. These are 18 situations in two prompt
variants, not 36 independent missions. All 36 responses had valid schemas; initial
503 availability failures are separately preserved and excluded from quality
measurements. [Detailed results](../qa/story-eval/planning/RESULTS_2026-10-31.md)
record settings, hashes, limitations and assistant prose-review observations.

The ordinary outputs copied 81/84 supplied milestone labels. This establishes
useful constrained planning behavior, not novel mission creativity, independent
narrative approval or generalization beyond scene-training-related families.
Prompt/context improvements and broader independent evaluation should precede a
training decision. No fine-tuning, model deployment, accepted-plan runtime,
persistence or gameplay behavior changed.

Validation: full director suite, focused planning regressions, all-target director
Clippy, 50 Python evaluation tests, Rust formatting, 18/18 authored completion
witnesses and diff checks pass. Database/browser checks were not needed for this
offline-only change. The endpoint exposes an alias, not a verified immutable
weight revision; future comparisons must preserve this limitation.

## Modal baseline and adapter evaluation — 30 October 2026

The [evaluation runner](../qa/story-eval/modal/EVALUATION.md) selects the verified
76 validation or 79 final test scenarios and uploads input prompts only. It runs
the pinned 4-bit base or a completed same-bundle adapter with identical greedy
settings, preserving raw outputs, generation errors, truncation, timing and tokens.
Adapter tensors are verified on load. Returned candidates are graded by the
existing current-runtime Rust story grader locally; invalid generations receive
no fallback credit. A comparison command checks matching suites, runtime versions
and grade integrity, reports mechanical improvements/regressions and supports
paired narrative-criterion counts from the existing judge workflow. No automatic
judge calls or human approvals are added. Partial results cannot receive a full
comparison score; recovered complete outputs can be regraded without a GPU.

Validation: 37 Python tests pass, including prompt-only selection, split integrity,
strict parsing, failure accounting and comparison drift rejection. Local Modal
preflights select both real splits; SDK CLI loading and Python compilation pass.
A six-output synthetic integration through the real Rust grader correctly rejects
malformed/truncated results and identifies one improvement and one regression.
Those fixture outputs are not model-quality measurements. No GPU inference,
baseline measurement, remote image build or adapter evaluation was launched here;
remote compatibility and actual quality/latency remain unverified.

A user-launched evaluation reached the container but failed during entrypoint
import because the local `train` module was not mounted beside `evaluate.py`.
Image construction and grading-helper imports are now local-only; remote volume
handles are declared independently. An isolated entrypoint import regression
checks the same missing-sibling layout. Remote inference still needs a rerun.

## Modal QLoRA preparation — 30 October 2026

The opt-in [Modal runner](../qa/story-eval/modal/README.md) and Glimmer Axolotl
configuration are implemented. The separately recorded user acceptance covers
all 607 revised authored reference hashes; this supersedes the pending-approval
wording in the historical review below, without implying independent blind labels.
Preparation verifies acceptance hashes, reruns the current engine grader and
exports only approved unchanged scenes. All 607 pass; the approved export yields
418 training candidates and 79 held-out examples after excluding 110 overlaps.
Reserving the bank-series journey and excluding another 121 overlapping/duplicate
training records produces 221 training, 76 validation and 79 final test examples.
Eight short training examples form the smoke subset. Artifacts are local and
ignored under `qa/story-eval/experiments/glimmer-modal-r1/`; the ignored source
snapshot must also be preserved to reproduce preparation from a new checkout.

The pinned real Glimmer tokenizer verifies assistant-only loss masks and complete
untruncated targets for all splits; maximum sequence length is 1,764 tokens.
The Modal SDK imports the runner and validates its local dry-run interface.
Local environment setup and Modal image dependency installation use uv; the
image pins uv 0.13.1 and retains the model/framework version pins.
The default remote job requests one H100 for two optimizer steps, saves the
adapter and checks its tensors and generation in a fresh process. Named volumes
persist weights and run artifacts; final test answers and review notes stay local.
No cloud image build, upload, GPU allocation, training, serving or gameplay change
has run. Image compatibility, GPU memory and actual checkpoint reload remain
unverified until the remote smoke run. These authored scenes do not establish
novel-mission generalization or improved model quality.

The first user-launched Modal image build failed before training because the
non-isolated FlashAttention build could not import `wheel`. The image now
explicitly installs `setuptools` and `wheel` alongside the other build tools
before compiling CUDA extensions. The corrected remote build remains unverified.

## Authored story quality corrections — 30 October 2026

The first model quality review of 607 authored references recommended 460 keeps
and 147 revisions despite all scenes passing mechanical checks. Two implementation
subagents corrected the underlying mock prose and labels; a separate reviewer
reassessed the freshly captured set: 607 keep, zero revise/reject, with all 147
previous revisions resolved. These remain model recommendations, not approvals. Changes acknowledge partial inventory,
remaining lantern placement, completed discoveries and corner designs, distinguish
instrument pickup from travel, name reachable destinations and use a neutral
mission-pause label. Workshop dialogue avoids repeating an acknowledged introduction.
These changes do not alter mission mechanics or educational progress.

All 147 previously flagged outputs change in the new capture; shared wording
changes affect 491 outputs overall. Existing sealed inputs, reviewed snapshots and
human decisions remain unchanged. The revised capture contains the same 607
reachable situations and 1,821 generation requests with explicit parent lineage.
Fresh human decisions belong to a new review progress file. Only 102 distinct
presentation sequences span the set; deduplication and genuinely generated
alternatives remain necessary before treating it as diverse training material.

Director and contract suites, all-target director Clippy, 28 evaluation Python
tests, formatting and diff checks pass. The real-database seven-story browser
journey also passes, including reload, mobile history and separate education.

## Broader story delivery — 30 October 2026

The opt-in story adventure now has seven authored missions. The four additions
cover clue discovery and instrument placement, a choice of cosy corner designs,
Bramble’s actual escort, and a gathering with flexible invitation/preparation
order. Reusable actions, stage-specific dialogue and persistent scenery keep the
stories connected. Education and assessed progress remain separate.

Three subagents implemented domain, director/API/contracts and browser scopes;
the integration review checked cross-layer consistency and corrected premature
placement, misleading hints, delayed dialogue and inconsistent choice previews.
Both seven-mission browser journeys pass with opposite lasting choices, reloads,
real escort, mobile layout and unchanged educational progress. Existing desktop/
tablet guest smoke and real-database training browser regressions also pass.
The aggregate repository check passes, including Rust, contracts (119 fixtures),
content validators and frontend (113 tests); the final evaluation Python suite
passes 28 tests. The disposable PostgreSQL suite passes all three integration
tests and migration guards, including v2 save upgrade, exact old-response retries
and new-mission persistence. Local tooling used Node 26.12; CI remains pinned to
Node 24.

The [new input pack](../qa/story-eval/datasets/broader-2026-10-30-inputs/README.md)
contains 607 reachable states and 1,821 generation requests. All 607 authored
reference scenes pass current mechanical grading. All 21 existing frozen pilot
and approval files retain their original hashes; archived review reproduces the
accepted export of 269 training and 114 held-out examples. Historical evidence is
explicitly marked archived, not revalidated against the new content version.
New model candidates, independent quality judgments and human approvals remain
pending. Shared mission families do not establish unseen-mission generalization.

| Plan work | Current implementation | Remaining gate |
| --- | --- | --- |
| WW-01 | Baseline ADR, asset hashes/provenance inventory, synthetic save fixtures, desktop/tablet smoke | Independent rights review and child usability research |
| WW-02 | Pinned Rust workspace, closed schemas, generated serde/TS, OpenAPI, shared fixtures and drift checks; companion/training projections and team commands | Parent lifecycle API contracts expand with the selected provider |
| WW-03 | 33-node DAG, ladders, coverage/review ledgers, graph/bounds/hash validators | Educator prerequisite review, exact descriptor verification, release approval |
| WW-04 | Three chapter spines, 18 beats, 12 illustrated friends, supported ending, discovery and progression data; sixteen Meadow training milestones | Directional/transformation art, richer milestone rewards and human reviews; activation disabled |
| WW-05 | Pure domain engine, unique rewards, quests, collection/team/friendship/transformations, character XP/training and replay | Durable replay/checkpoint storage, optional roles/personal quests and authoritative world paths |
| WW-06 | Six assessment families, bounded generation, explicit wire adapters, authored prompts, honest support | Educator calibration, broader item/context coverage and approved release pack |
| WW-07 | Experimental deterministic policy, frontier/enrichment, support/difficulty advice, replay checks | Calibrated policy, context/subject balance, durable recompute CLI and window-scoped freshness |
| WW-08 | PostgreSQL ownership/RLS, sessions/CSRF primitives, consent/export/delete adapters, tombstones, migration tooling | Reviewed identity provider, HTTP lifecycle flows, cleanup/retention workers, external-object deletion and backup recovery |
| WW-09 | Axum gameplay routes, actual DB transactions, idempotency/revisions, atomic evidence/rewards/outbox, auth boundaries | Full regional/world eligibility, production roles/TLS/provider, durable worker and full integration |
| WW-10 | Typed cloud client, exact retries, profile/logout cancellation, recoverable unverified import preparation | Real import endpoint and account/profile UX; no migration UI enabled |
| WW-11 | Six accessible DOM renderers, authored presentation binding, keyboard/touch controls, support-before-narration | Entrypoint/account integration, human screen-reader/accessibility review and complete browser journeys |
| WW-12 | Meadow world, character journal, training entry point, team controls/two followers; 75-destination exploration library | Server regional/discovery/story integration, optional roles and full playable ending |
| WW-13 | Bounded proposal validation, authored-only director/fallback, exact task binding and deterministic replay | API commit integration and approved content activation |
| WW-14–16 | Offline scene evaluation with reviewed pilot and seven-mission input expansion; authored game has no model dependency | New candidate generation/review, live provider adapter, human grader calibration and gated training experiment |
| WW-17 | Existing guest parent view preserved; Rust summaries/client foundation available | New parent learning view, approved pack workflow and coverage dashboard |
| WW-18 | Local reproducible install/start/migration/check helpers and GitHub Actions configuration | Hosted CI execution, database/browser CI, deployment, observability, backup restore and rollback drills, operational owner |
| WW-19–20 | Automated foundations and synthetic checks only | Integrated QA/security review, consented pilot, educator audit and measured curriculum expansion |
| Authored story slice | Seven connected missions, discovery/choices/escort/gathering mechanics, compatible saves, mock scenes and Phaser `/?story=1`; expanded evaluation inputs | Broader catalogue/preferences, shared-adventure integration, model adapter/evaluation and human usability review |

Design-only update, 30 October 2026: the [generative story design](GENERATIVE_STORY_DESIGN.md)
proposes separate mission-planning and scene-continuation protocols, with reusable
actions, outcome-based milestones, a persistent narrative brief and a full Pip
bridge example. It also maps input fields to existing/new data sources and defines
context assembly and staged integration. Education is excluded from story generation. No model adapter,
new wire contract, operation registry or gameplay behavior is implemented by this
document; WW-14–16 and the existing authored-director limitations remain unchanged.

## Combined validation

`bash scripts/check.sh` passed Rust formatting and Clippy with warnings denied,
Rust unit/HTTP tests, both-language shared fixtures and code-generation drift,
14 curriculum tests and 8 story tests, frontend typecheck/lint/format/build,
7 existing frontend test files, 9 client/migration tests and 4 legacy fixture
tests. Default Rust tests explicitly exclude the two database-dependent tests;
`sh scripts/check-postgres.sh` executed both against a disposable local UTF8
PostgreSQL database, along with SQL ownership/export/deletion tests, repeated
migrations and negative migration-hash drift rejection. All selected checks passed.

Chromium desktop/tablet guest smoke and six-family activity component tests
passed. Component tests include keyboard controls, safe text, duplicate guards,
support-before-audio, failed support, editable answers and destroyed pending
callbacks. These are not a complete cloud-browser playthrough. The production
build retains the existing large Phaser bundle warning.

Integration review found and fixed equal-sharing schema uniqueness, parent-delete
RLS isolation, forgotten support history in mastery replay, late profile response
body races, noncanonical quest/skill mapping, cross-origin backslash URL
normalization and configuration-only readiness reporting. These concrete fixes
have regressions; they do not substitute for the plan's production security review.

## Development and release boundary

Only local synthetic databases were used. No child data, provider key, paid model
request, training run, deployment or publication occurred. The current browser
continues to use the existing local guest adventure. The Rust API exposes a
restricted authored Meadow development slice and fails closed without sessions.
Human approvals, reviewed art and provider/operational decisions cannot be
replaced by passing synthetic tests. New packs remain drafts and do not claim
complete curriculum coverage or secure mastery from repeated fixed practice.

## Independent review follow-up

An independent code review identified two defects in the initial foundation.
Encounter abandonment now has a real idempotent server command, persisted
unassessed evidence and domain cancellation, so leaving a task does not block
the next encounter. The authored director now excludes completed beats.
See `adr/encounter-abandonment.md` for protocol compatibility, regression coverage
and rollback limits. These fixes do not complete the outstanding release gates.

## Local browser integration — 29 October 2026

An opt-in `/demo.html` development screen now mounts the existing client and
activity components against the real PostgreSQL API. It exposes the existing
Meadow slice (six story beats, four friendships), server answer checks, recorded
support, abandonment, companion selection, inventory and learner summaries.
A separate synthetic database/session is provisioned by `scripts/local-demo.sh`;
the normal API authentication path is unchanged. Guest world saves remain separate.
This does not close the regional travel, world synchronization, account/import,
parent lifecycle, director, transformation or content-review gates listed above.

### In-world local integration

The default local demo now uses the Phaser world. Ten labelled interaction
points expose all six Meadow story beats and four friendships through walking,
clicking, Space and Missions. Server completions drive bridge collision/scenery,
picnic and garden changes. The journal and grown-up view use server snapshots;
selectedCompanions is an optional snapshot field populated from the domain team,
so followers restore after reload. Existing friend art is provisional.
Closing an activity records abandonment; uncertain writes retain their exact
command for retry, and pending encounters can resume after reload. Guest mode
remains at `?guest=1` with its original storage untouched by backend play.
Movement/camera/hero preferences are local; this does not assert server-verified
world reachability or add regional travel, transformations or account workflows.

### Visual activity interaction pass

The mounted activity components now use draggable/tappable planks, berries,
plates, number and letter tiles, and illustrated story/position cards. The UI
reuses local game art, shortens instructions and preserves accessible control
names and keyboard alternatives. Pointer dragging supports touch, return-to-tray
and ordered-card rearrangement; correctness and rewards remain server-owned.
This changes presentation, not assessment rules, and still requires educator
and child usability review.

### Character progression integration — 29 October 2026

The local server journal now shows twelve authored character profiles (four Meadow friends presented), motivations and dialogue selected from confirmed friendship/story state. Server commands record distinct completed helping memories and toggle reversible cosmetic transformations; snapshots include friendship, transformation IDs and available memories. Rewards and learning evidence remain separate. The journal preserves expanded profiles and keyboard focus after saving. Glow uses accessible text/styling while distinct transformation artwork is pending.

Workspace checks, 62 shared contract fixtures, disposable PostgreSQL checks and desktop/tablet synthetic-API browser flows passed. See `CHARACTER_DEVELOPMENT_REVIEW.md` and `adr/character-progression.md`. Human content/art/accessibility review and regional integration remain pending.

### Character artwork library — 29 October 2026

Five subagents produced actual generated artwork for all twelve original friends,
a filterable `/characters.html` field guide, journal portraits and world rendering.
Three transparent regional sheets preserve each friend's species and visual
identity; inspected frame boundaries prevent wings/capes being clipped. Meadow
friend NPCs and followers now use their own art rather than borrowed hero frames.
All twelve profiles expose personality, motivation, habitat and voice guidance.
Browsing the field guide changes no story, collection or learning state.

Generation prompts/hashes and a separate development inventory are in
`content/creatures/art/`. Single front poses are available; directional animation,
separate glow artwork, human rights/content review and release pack activation
remain pending. The historical frozen story asset manifest is not rewritten.

Library validation passed: `bash scripts/check.sh`, dedicated twelve-character
PNG/hash/shared-frame regression, and Chromium checks at desktop/tablet/phone
sizes covering all artworks, regional filters, keyboard profile expansion and no
horizontal overflow. Existing desktop/tablet guest and synthetic-API companion
flows also passed after world artwork integration. The library preview is saved
as `docs/character-library-preview.png`. These checks do not replace human art or
child usability review.

## Companion progression planning update — 29 October 2026

The companion progression plan now defines the next gameplay work: per-character levels, training, collection presentation, teams, friendship/transformations and regional discovery. Existing domain systems are reused; individual character XP and training remain unimplemented. The frontend-first plan was replaced with a planning index, and obsolete foundation sequencing was removed from the broader roadmap. This documentation update adds no gameplay features or new application test results.

## Exploration library and companion training delivery — 29 October 2026

The merged exploration library provides 75 connected destinations, with individual
WebP landscape images for the 72 new maps and shared lazy-loaded previews. This
does not activate server-authoritative Orchard/Cove quests, recruitment or the
ending. Twelve original friend portraits and Meadow world sprites are already
available; the older provisional-art notes above describe earlier milestones.

The first training delivery adds separate character XP, five levels and sixteen
authored training milestones across Pip, Tansy, Nib and Flutter. Each first-time
milestone grants ten character XP; supported and independent completions receive
equal rewards. Replaying a rewarded milestone remains practice with zero extra
XP. Wrong answers and abandonment grant none. Explorer progression and learning
evidence remain separate from character level.

Server-issued training stores the recipient, rules version and policy fingerprint.
Team changes cannot redirect XP. Changed policy rejects assessment while allowing
abandonment. Existing aggregates load with zero character XP and retain ownership,
friendship and transformations. Training rewards, evidence and grant events use
the existing atomic PostgreSQL transaction and exact-command retry path.

The journal exposes levels, XP, next thresholds, training/practice and explorer
progress. A reachable Meadow clearing opens the same activity controls. A
server-confirmed level increase shows a portrait celebration with a reduced-motion
alternative. Team selection/removal uses server capacity and the world supports
two followers. The second slot retains its original Orchard story prerequisite;
its rendering is not evidence of unlocked regional gameplay.

The [GitHub Actions configuration](../.github/workflows/checks.yml) now covers the
repository's Rust, contracts, content and frontend checks. Hosted execution and
database/browser CI remain unverified; see [CI scope](CI.md).

Remaining gameplay work: character-specific optional roles, personal quests,
distinct higher-level rewards, directional/transformation art, automatic verified
friendship participation, discovery and regional story integration, and the full
supported ending. The subsequent [long-play implementation](LONG_PLAY_STORAGE_PLAN.md)
replaces the PostgreSQL lifetime command limit; archive operations remain planned. Account/import/lifecycle/parent-view work, human
content/accessibility/security review and deployment/backup operations remain
public-release gates. No release or content approval is implied by this delivery.

### Verification for this delivery

`bash scripts/check.sh` passed after adding Rust round-trip coverage for the new
contract types: workspace formatting/Clippy/tests, 73 shared contract fixtures,
generated-code drift, curriculum/story/art validators, frontend checks/build,
nested client tests and legacy saves. The workspace includes 17 domain tests and
15 API tests; two database tests are intentionally ignored by that command.

`sh scripts/check-postgres.sh` passed both real database tests and the SQL
ownership/export/deletion, repeat-migration and negative hash-drift checks in its
own disposable database. Added training coverage verifies rollback before commit,
concurrent duplicate completion, exact retry, one XP grant and persistent evidence.

Chromium guest desktop/tablet smoke, six-family activity component checks and
desktop/tablet synthetic-API friendship/glow flows passed. The real world training
journey also passed through `node qa/e2e/run-training-qa.mjs` against its own fresh
PostgreSQL database and temporary session. It recruits the four Meadow friends
through real assessed HTTP commands, then verifies keyboard selection/training,
wrong answers, recorded support, reload/resume, abandonment, supported level-two
celebration, saved XP, exact completion retries, unrewarded repeat practice and
phone-sized training. Independent and supported first milestones award equal XP.
The runner removes its database/session and does not touch persistent demo data.

Integration checks found and fixed missing Rust round-trip dispatch for new
contract fixtures and an activity stylesheet hiding the training heading. The
training context is now visible; reduced-motion celebration remains supported.
The existing large Phaser build warning remains. Hosted CI execution, human
review, sustained storage beyond the command cap, the Orchard-unlocked second-slot
browser journey and a complete three-region playthrough remain unverified.

## Maintainability refactor — 29 October 2026

The world and diagnostic demo now share a DOM-independent command session with
explicit sending, refresh and uncertain-retry states. Exact prepared commands
remain intact across retries. Journal, mission and training views have separate
rendering/binding helpers, and journal expansion/focus restoration uses creature
IDs. A validated placement manifest checks authored encounter coverage, map IDs
and coordinate bounds; regression tests check reachable approaches.

The API validates commands before decoding generated Rust types and dispatches
exhaustively. Public snapshots use typed projection builders with golden JSON
compatibility coverage. The domain engine is separated into content, state,
transitions, replay and tests while retaining its public API and saved formats.
No gameplay eligibility, rewards, content activation or persistence schema changed.

Frontend test discovery includes nested API tests. PostgreSQL and training QA now
use fresh temporary private-socket clusters on local macOS/Linux installations.
CI includes database and browser integration jobs; hosted execution remains
unverified. The existing persistent demo database is not used by these checks.

Local validation passed Rust formatting/Clippy/workspace tests, contracts and
generated-code checks, curriculum checks, frontend type/lint/format/unit/build,
character-art validation, disposable PostgreSQL integration, guest desktop/tablet,
companion journal desktop/tablet, six-family activity browser checks and the
real server-backed training journey (including reload/resume, acknowledged
abandonment, exact retries, reward prevention and mobile training). The training
test now waits for the acknowledged dialog closure before checking progress.
The aggregate check remains blocked by four pre-existing story asset hash
mismatches (Meadow, Orchard, Cove and the hero atlas); all four image files match
HEAD exactly. The story unit suite's authored-content check and the legacy
artwork-baseline check fail for that same existing mismatch. Historical manifests
were not rewritten. Guest-save compatibility tests pass. The existing large
Phaser bundle warning remains.

### CI artwork integrity follow-up — 29 October 2026

The failing hosted content check was traced to ImgBot commit `cd852f7`, which
recompressed the ten original PNGs without updating their byte inventories.
Comparing decoded RGBA dimensions and pixels against the original baseline
confirmed all ten images are unchanged visually. The legacy inventory now records
the optimization commit and retains the original baseline reference; its byte
counts/hashes and the four story asset hashes have been intentionally updated.
The dependent story pack hash was regenerated. Integrity checks remain strict,
and no content approval or release activation changed.

`bash scripts/check.sh` now passes locally in full, including story validation,
all thirteen story tests and all four legacy fixture tests. The existing Phaser
bundle-size warning remains; hosted results are tracked on PR #7.

Hosted CI also exposed an undeclared `rg` dependency in the database runner's
migration-drift log assertion, after both database tests had passed. That literal
log check now uses portable `grep -Fq`; the complete disposable database check
passes locally with the corrected assertion.

### Activity distribution — 29 October 2026

The local backend world's ten story/friendship encounters now occupy five maps
with two per map: Clover Meadow, Starlight Cove, Peachy Orchard, Fern Hollow and
Mossy Clearing. The training clearing is in Owl Grove. Welcome and bridge repair
remain reachable on the starting map before the bridge is fixed. Missions travel
to the encounter's actual destination and show map names; Explore lists encounter
counts. The existing 75-map library still has exploration-only destinations.
These browser placements retain Meadow server eligibility, prerequisites, saved
IDs and rewards; they do not activate Orchard/Cove story packs. Guest activities
retain their existing placement.

Validation: frontend check passed (80 tests, typecheck, lint, formatting and
production build); synthetic-API browser checks verified mission travel and
walking to all four relocated friends, and guest smoke passed on desktop/tablet.
`git diff --check` passed. The existing Phaser bundle-size warning remains.
No real database assessment changes were made or tested for this placement update.

### First exploration quest — 29 October 2026

“A trail of little kindnesses” now connects existing assessed content into a
seven-step journey: accept Pip’s invitation, listen to his message, repair the
bridge, share the Cove picnic, befriend Tansy in Orchard, grow the garden, and
return to Pip. Conversations precede story/friendship activities and allow the
player to leave without starting an encounter. Missions and Journal show current,
later and completed steps; existing server progress counts without replay.

Missions now tracks a target instead of teleporting. The HUD describes the next
connected exit, and Follow path walks there or approaches the character on the
current map. Players can still walk with arrows/WASD or tap exits. Explore retains
its explicit map-selection travel. Cove picnic berries and the Orchard garden
reflect server completion; newly confirmed friendships get a named celebration.

The trail uses existing server-owned activity, support, abandonment, friendship
and reward rules. Acceptance, tracked target and return conversation are validated
local preferences stored under the profile-specific view key; they are not new
server quest grants or cross-device conversation state. They cannot establish
learning evidence or creature ownership. Guest saves and content review/activation
remain unchanged. This is one authored trail, not procedural wild encounters or
an expansion of the server’s regional gameplay eligibility.

Validation: frontend typecheck/lint/format, all 87 tests and production build
passed. A new real-API browser journey passed against the disposable PostgreSQL
runner: dialogue/decline, incorrect answer without progress, support, reload/resume,
abandonment, bridge repair, actual walking between the three maps, picnic,
supported Tansy recruitment, garden, return conversation without an extra reward,
saved completion and phone layout. Desktop/tablet guest and synthetic companion
memory/glow checks passed. The local Node runtime was 26.12.0; Node 24 was not
available. The existing Phaser bundle-size warning remains; human playtesting and
cross-device conversation persistence are not established by these checks.
The updated synthetic distribution check also passed walking routes to all four
friends across the five encounter maps, including dialogue before encounter start.

### Direct character interaction — 29 October 2026

Clicking a nearby character now opens its conversation immediately; clicking from
farther away still walks over first. Space selects the nearest character within
chat range. Both paths clear pending walking interactions before opening the
conversation, so closing it cannot resume an old route and reopen the chat.
The conversation retains its action to start the server-assessed activity.

Validation passed: all 88 frontend tests/type/lint/format/build checks, the real
three-map QA journey now beginning with a direct character click and Space to
reopen chat and start the activity, desktop/tablet guest smoke, and diff checks.
The existing Phaser bundle-size warning remains.

### Optional Muse spoken answers — 30 October 2026

The local backend-world activity UI supports typed answers and optional recorded
answers for missing quantities and non-phonics single-choice tasks. A configured
`BASETEN_API_KEY` enables the local server bridge to
`meta-models/muse-spark-open-1026`. The microphone records at most 20 seconds;
transcripts are editable drafts requiring explicit application and the normal
server-assessed attempt. Existing touch/keyboard controls remain. Phonics,
ordered/multiple answers and sharing retain their existing controls. Read-aloud
output, pronunciation assessment, live story generation and fine-tuning are not
part of this change.

The local-only bridge validates binary WAV bounds, Host/Origin, CSRF/profile and
backend session before sending audio. Provider requests and responses are bounded;
recordings/transcripts are not persisted by the application. This does not establish
provider retention policy, child-speech accuracy, approved child-data processing
or a production audio API. No real provider request or child recording was used.
The authored director and gameplay/learning rules are unchanged.

Validation passed: frontend typecheck/lint/format, all 91 unit/API tests and
production build; the six-family Chromium activity suite including synthetic
microphone capture, WAV conversion, editable transcripts, cancellation/late
results, typed single-choice mapping and phone layout; desktop/tablet guest smoke;
and a temporary HTTP bridge test with synthetic sessions and mocked Baseten.
`git diff --check` passed. Checks used the available Node 26.12.0 runtime rather
than the requested Node 24. The existing Phaser bundle-size warning remains.
Live hackathon access, real speech quality and Safari/iPad recording are unverified.
No Rust, database or gameplay-contract changes required database verification.

### Authored story and payload foundation — 30 October 2026

The [story implementation plan](STORY_IMPLEMENTATION_PLAN.md) now has an isolated
headless delivery: a pure Rust authored Pip bridge mission with two wood sources,
exchange/assembly, coarse travel, persistent milestone receipts and explicit shared
crossing; deterministic planning/scene payloads; additive Story contracts; and a
runnable journey with bounded replay checkpoints. Education and the existing
assessed bridge are unchanged. No model call is made.

The builder derives inventory, facts, progress and legal actions from actual story
state. Authored content supplies descriptions and the fixed narrative; preferences
use defaults. Rendering does not record meetings or grant progress. Explicit
presentation acknowledgements establish presented history. Revision-bound handles,
atomic rejection and exact-command retries prevent stale or duplicate effects.

Two implementation subagents and an independent reviewer worked against the plan.
Review fixed absent-character dialogue, incorrect pre-repaired-bridge credit,
missing paused-mission context and premature relationship assertions. The owner
added reachable-state payload/action integration coverage and checked compatibility
of all prior schema definitions and fixtures.

`bash scripts/check.sh` passed, including workspace format/Clippy/tests, 89 shared
fixtures/generated checks, content validators/tests, all 91 frontend tests/build
and four legacy tests. Both CLI source journeys and checkpoint/manual-action/error
checks passed. `git diff --check` passed. Rust 1.99.1 and available Node 26.12.0 were
used; Node 24 was unavailable. The existing Phaser bundle-size warning remains.

This is not a new playable browser mode. API/database storage, authenticated saves,
real world placements/movement, new accessible controls, broad content catalogues,
generated-plan validation and provider integration remain pending. Checkpoints are
local consistency-checked records, not trusted imported progress, and the developer
runtime is capped at 256 commands. No database/browser integration behavior changed
or was verified; ordinary Rust tests still ignore two existing database tests.
Content approval/release gates remain unchanged.

### Authenticated story and browser slice — 30 October 2026

The [story implementation plan](STORY_IMPLEMENTATION_PLAN.md) now includes verified
S6–S9 integration: separate story snapshot/command routes, profile-owned
PostgreSQL checkpoints and original-response receipts, and an opt-in `/story.html`
view. Sessions, CSRF/Origin, profile locks and forced RLS protect the new routes.
Story data participates in profile export/delete. The existing adventure and
educational records remain separate; the browser journey checks that they do not
change during the mission.

The view supports both wood sources, an explicit presented-introduction
acknowledgement, pause/resume, inventory/milestones and the shared crossing outcome.
It validates shared contracts and preserves exact uncertain commands across
reload/authentication failures, refreshing authoritative state after an old receipt.
Three coarse locations use keyboard/touch controls; Phaser walking and the existing
assessed bridge are not integrated. Only one authored mission exists, preferences
are defaults, and the 256-command development cap ends actionable scenes without
discarding the save. No model, next-mission generator or reset endpoint is enabled.

Three implementation subagents and the integration owner reviewed the combined
work. The full aggregate check passed (102 shared fixtures, 98 frontend tests,
content validation, workspace checks/build and legacy tests). Disposable PostgreSQL
checks passed, including the three ignored database tests, ownership/RLS, rollback,
races, corruption and lifecycle checks. Both story source journeys, existing
training, guest desktop/tablet and accessible activity browser checks passed.
Desktop/mobile story screenshots were inspected and the final recovery/label
polish was retested. CI now schedules both story routes; hosted execution and Node
24 were not verified locally (Node 26.12.0 was available). Human accessibility and
child usability research remain pending, as do all content/release approvals.

### Playable story world with mock scenes — 30 October 2026

**Missions → Pip’s story adventure** opens `/?story=1` in the actual Phaser world.
The saved story snapshot drives present NPCs, resource/route controls, conversations,
inventory and mission milestones. A deterministic mock provider consumes compact
story input and returns scene dialogue/choices through binding validation and an
authored fallback. Both wood-source paths lead through the orchard workshop and
back to the visible meadow bridge; confirmed crossing animates Pip following and
restores the correct bank after reload. The old `/story.html` is an inspector.

Story mode restricts movement to the server-confirmed bank/location and blocks
stale actions during recovery. Pixel movement is still local, not server-verified.
The woodcutter uses existing Bramble art. Main-adventure education and guest saves
are separate and unchanged; there is one complete authored mission, not automatic
successor missions. Model generation, personalized preferences and release
activation remain disabled/unimplemented.

Three subagents implemented/reviewed the provider, world adapter and browser QA;
the integration owner checked code, tests and desktop/mobile/crossing screenshots.
Aggregate checks and 102 frontend tests passed, as did disposable PostgreSQL,
both world routes, existing training and guest desktop/tablet checks. Expanded
bank QA proved discovery from Missions, response-driven safe text, second-tab
advancement, exact old-receipt retry with a failed refresh, locked recovery and
terminal rendering. Both routes are configured in CI; hosted execution and Node
24 were not verified locally. One concurrent disposable DB startup hit local
shared-memory capacity; serial retry passed without touching demo data.

### Connected story series (30 October 2026)

The story world now contains Pip’s bridge, Pip’s picnic and Welcome lights. A
completed mission offers its successor explicitly. Resources, reversible pause,
milestones, Pip’s location and prior world changes persist; the journal retains
all three completed goals. The picnic brings Pip back across the bridge. Its
food appears on the existing meadow blanket, and lights remain on both banks.
No academic activity or mastery is needed for these missions.

Content version `pip-stories-2` strictly replays and validates historical
`pip-bridge-1` checkpoints before projecting them forward. Revision and original
command receipts are preserved. Version-bound action handles prevent an old
travel choice from becoming a new mission-accept action at the same revision.
There is no save reset or modification to an applied migration.

Domain, director/API and browser subagents implemented their respective parts;
independent cross-review found and fixed partial-resource planning quantities.
The integration owner ran aggregate checks, disposable PostgreSQL including legacy
save/retry continuation, the real three-story browser journey, desktop/tablet guest
smoke and real assessment/training QA. Contracts include 107 shared fixtures.
Browser evidence covers explicit next acceptance, pause/reload, uncertain-write
retry, both crossing directions, persistent scenery/history and mobile layout.
Local Node was 26; CI is configured for Node 24. Hosted results are separate.

This remains a finite three-mission development series with a 256-command cap,
existing Bramble art for the woodcutter, local pixel movement and authoritative
coarse locations. There is no fourth mission, model generation, personalized
preferences or release approval. Restart a running demo to load the new backend;
reload alone only updates the frontend.

### Offline scene-evaluation pilot (30 October 2026)

`qa/story-eval/run.py` prepares 137 runtime-derived situations and 411 generation
requests by default. The Rust `story_eval` example exports replay-checked inputs
and grades imported candidates against the shared scene validator and domain
actions. A valid candidate keeps every legal action, present speakers and required
introduction metadata. Each choice executes independently on a cloned state with
retry/replay checks. Invalid candidates receive no authored-fallback credit.

The Python workflow emits quality-judge requests, imports evidence-based judgments,
renders escaped local HTML reviews and exports only explicitly human-approved,
hard-passing targets. Prompt/schema/content/provider metadata and hashes bind the
artifacts. Entire journeys stay together in train/evaluation splits; exact prompt
overlap with held-out scenarios is excluded from training export. The related
authored scenarios do not establish generalization to new mission structures.

The domain/director subagent implemented and tested the Rust portion; the integration
owner reviewed it, implemented the review/export workflow and addressed cross-review
findings. Director tests, formatting/Clippy and 12 Python workflow tests passed.
Actual CLI integration produced 137/137 authored-reference hard passes; an unreviewed
batch exported zero examples. These are pipeline checks, not model quality results.
No live provider calls, human-quality calibration, actual training or new-mission
generation were performed. Existing gameplay and educational state are unchanged.

### Versioned story dataset (30 October 2026)

The synthetic pilot is preserved in `qa/story-eval/datasets/pilot-2026-10-30/`:
137 replayable inputs, exact generation requests, 411 generated scenes, archived
hard checks, seven-criterion independent model judgments and recommendation notes.
The judge recommended 386 keep and 25 revise; these are not human approvals.
One preferred scene per situation yields 96 training candidates and 40 held-out
candidates after excluding one exact input overlap.

`dataset.py verify` checks file digests, complete record coverage, candidate-bound
judgments/recommendations and shortlist consistency. `dataset.py review` reruns
the current Rust grader and regenerates the report, pending human-review template
and shortlists using only repository inputs. New dataset versions can be sealed
without overwriting an existing snapshot. Disposable experiment outputs are ignored;
the frozen data, provenance and a training-experiment record template are in the repo.
Exact original session model IDs and generation commit were unavailable and remain
explicitly unknown. Sixteen Python workflow/dataset tests passed; the repository-only
review run graded all 411 candidates successfully. No actual training or upload ran.

### Human review screen (30 October 2026)

`python3 qa/story-eval/review_server.py` serves a separate local review workspace
at `127.0.0.1:5182`. The UI subagent implemented readable state/scene cards,
training/evaluation queues, approve/revise/reject controls and keyboard/mobile
layouts. The integration owner implemented/reviewed persistence, grade gating,
export integration and real browser checks.

The server omits model judgments/recommendations from initial payloads and denies
grade requests until that candidate has a saved decision. Atomic local progress,
snapshot/candidate binding and a single-writer file lock preserve decisions through
reloads/restarts. Original decisions remain recorded after later edits. Optional
browser storage remembers navigation only. Revision suggestions do not edit or
approve frozen candidates; exports skip revise/reject and refuse edited approvals.
The UI can download compatible human-review JSONL. It is a single-user loopback
developer tool on macOS/Linux, not production identity or multi-user hosting.

Twenty-four Python tests passed. Real Chromium checks covered hidden grades,
approve/revise/reject, reload/restart, failed saves without premature reveal,
revision gating, exports, Host/Origin/token behavior and mobile layout. Desktop
and mobile screenshots were reviewed. No user decisions were made by the agent,
and no training or model calls were performed.

User follow-up: the user explicitly accepted all pilot model recommendations.
The separate `qa/story-eval/approvals/pilot-2026-10-30-user-acceptance/` record
preserves 17 prior decisions and records the remaining batch acceptance honestly
as model-assisted, not independent blind human labels. There are 386 approvals
and 25 revision decisions. Verified export yields 269 training examples and 114
held-out examples, excluding three approved input-overlap candidates from training.
The frozen dataset and original model judgments remain unchanged. No training ran.

The Modal story evaluator now supports an opt-in `speakers-v1` prompt-only
experiment. It derives explicit allowed speakers from the verified scene's present
characters and distinguishes them from mission/history mentions, with narration
only when no characters are present. Original prompts remain the default;
transformed prompts have distinct versions and suite hashes, and paired comparison
still requires identical suites. Offline regression checks and validation preflight
cover the variant; its effect on actual generated-story scores remains unmeasured.

The opt-in `speakers-unique-v1` evaluation variant retains the speaker constraints
and adds unique presentation entry IDs, reserved required-entry IDs, and no
repeated optional dialogue/narration. Earlier variants remain unchanged. Local
regression tests cover version/hash separation, source preservation and inherited
speaker constraints; the 76-case validation preflight passes. Actual generation
quality for this variant remains unmeasured; no GPU job was launched here.

A Modal BF16 exporter now merges completed Glimmer adapters onto the pinned
unquantized base into a separate exports directory. It checks adapter tensors,
saves full sharded weights and processor/tokenizer assets, and requires a fresh
checkpoint reload/generation check before marking success. Offline prerequisite
checks pass; GPU merge, recipe acceptance and BF16 story quality remain unverified.


### Optional Modal story endpoint integration

The development API can opt into the deployed `wonderwilds` endpoint using
`WONDERWILDS_MODAL_STORY=1` and backend-only Modal proxy credentials. Authenticated
story GET snapshots send the real `StoryScene` schema in a strict `json_schema`
response format as well as the explicit speaker/unique-entry prompt, with low
reasoning; the existing story UI already refreshes after actions. Network
I/O happens outside database transactions. Rust validates every proposal, rechecks
session/ownership and state, and discards stale generations. Timeouts, invalid
responses and busy generation use authored fallback. There is one outbound request
at a time and a bounded exact-prompt cache. Command execution, persistent receipts,
learning and guest mode are unchanged. No migration or wire-schema change is needed.

Local HTTP fixtures and router tests cover valid results, malformed/oversized/error
responses, strict response-format/schema serialization, schema-valid illegal action
bindings, unsupported-schema errors, truncation, cache behavior, unavailable
endpoints, authentication, expired sessions, stale state and unchanged command
retries. Live model-backed
browser play has not been exercised by this implementation; content approval and
production readiness remain pending. Live enforcement of the full scene schema
and structured-output latency on the deployed server remain unverified.

### Story source indicator — 31 October 2026

The game HUD and `/story.html` inspector display server-reported scene provenance:
preauthored, LLM-generated, or preauthored fallback. Failed/invalid generations and
state changes during generation report fallback. Cached valid LLM scenes count as
generated prose; the label does not claim a live connection or describe mission
plan provenance. Guest/main-adventure text is preauthored. The optional wire field
preserves historical receipts; missing metadata is displayed as unknown. No
persistence or command receipt changes are introduced.

Validation: frontend checks/build, API unit tests (35 passed, four database tests
ignored), shared contract fixtures/generated checks, synthetic desktop/phone
source-status browser checks for both views, and desktop/tablet guest smoke checks.
Node 26 was used; live provider connectivity and database integration were not
exercised for this display change.

### Mission journal and in-world replies — 31 October 2026

The playable story's Missions panel now emphasizes the current objective, a
progress checklist, inventory and illustrated destinations. Optional practice
follows the story content. Choice dialogs use character portraits and a warm
paper/forest palette. Text-only character encounters and post-action replies
without a further choice use anchored, non-modal speech bubbles. Main-adventure
characters without an available activity also use bubbles. Escape, the dismiss
button, walking away or opening a dialog clears speech; reading has no timer.
Required presentation acknowledgements remain explicit actions, and pausing is
always available in Missions even when Pip has only a casual reply.

Validation: frontend type/lint/format, 119 unit/API tests and production build;
desktop/tablet guest browser smoke; synthetic desktop/phone dialogue checks for
required acknowledgements, post-action and repeat text-only speech, escaping,
keyboard/dismissal, viewport bounds and no implicit progress writes. Desktop and
phone screenshots were inspected. No server/domain/persistence change was made;
real database story journeys and live generated prose were not rerun. Local Node
26 was used; the existing bundle-size warning remains.


### Inline learning in authored and generated mission steps — 31 October 2026

The playable story world now opens prerequisite-ready activities directly from
collect, inspect, exchange and assemble interactions. The separate practice
launch has been removed; Missions retains learning progress, resume and exact
save recovery. Discovery-marker task labels reuse the existing five maths
skills and preserve task parameters, representations and evidence fingerprints.
New generated neighbourhood requests require a short `learningBridge` invitation
relating markers to the investigation. Saved-content schemas keep it optional;
existing generated missions use authored joining text without regeneration.
No learner history or assessment keys enter the generation prompt.

A server-resolved binding freezes the legal story action, revision, mission /
neighbourhood generation, location and joining text. Correct answers execute the
bound story action in the same profile-locked transaction as learning evidence,
checkpoints and both receipt logs. **Let’s do it together** reveals a server-authored
solution and completes the action with worked-example / not-assessed evidence,
never independent success. Putting a task aside records abandonment without story
progress. Stale bindings conflict rather than silently attaching to another task.
Continue returns to the original encounter's discovery reply. Inline screens
show the narrative invitation and task; teaching explanations remain behind help
controls. The browser refreshes authoritative story state after completion and
recovery; a failed story read retains the confirmed learning receipt and requires
refresh.

The diagnostic inspector and original story commands remain compatible without
academic gates. No migration, demo reset, new skill coverage, reviewed mastery
claim or release activation is introduced. The reusable marker preparation is an
initial joining mechanism; narrative fit and frequency still need human playtesting.
See [joining decisions](adr/mission-learning-joins.md) and the updated
[curriculum mission plan](CURRICULUM_MISSION_PLAN.md).


Validation for this slice: aggregate repository checks passed (149 shared
fixtures, Rust/content checks, 121 frontend tests/build and legacy fixtures),
as did the complete disposable PostgreSQL suite and the focused final atomic
learning/story regression. Browser checks passed for inline independent/wrong/
supported answers, lost support and atomic-completion responses through reload,
exact retries, desktop/phone controls, generated multi-step stories and fallback,
and guest desktop/tablet plus the existing accessible activity controls.
Desktop and phone task screenshots were inspected. Final frontend checks and
workspace formatting/Clippy passed. Node 26.12.0 was available rather than Node
24; the existing bundle-size warning remains. No live model calls, child
playtesting, hosted CI or production content approval were performed.


Inline-learning follow-up verification: the final bridge browser journey passed,
including old-receipt reconciliation, failed-refresh locking, visible repair and
shared crossing. All seven authored stories passed on the default branch through
inline supported tasks, immediate discovery replies, pause/resume, automatic
travel, Bramble escort, gathering and mobile history. Counting/comparison pieces
now avoid ordinal labels that reveal collection sizes; the final learning tests
(32) and learning-crate Clippy passed. Concurrent entry-point and provider
configuration edits were preserved; the aggregate results above describe the
learning-integration checkpoint rather than a review of those separate changes.
