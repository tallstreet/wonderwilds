# Wonderwilds implementation evidence

Foundation evidence recorded 28 October 2026; integration updates dated below. Broader roadmap: [Wonderwilds implementation plan](WONDERWILDS_IMPLEMENTATION_PLAN.md) v1.2. Active gameplay sequence: [Companion progression plan](COMPANION_PROGRESSION_PLAN.md). Validation counts in earlier sections describe their original delivery, not the latest checkout.
Six subagents implemented distinct scopes; the integration owner reviewed and
validated the combined changes. Existing source, guest saves and art are preserved.
No release, deployment, curriculum approval or learning-efficacy claim is made.

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
supported ending. The 2,000-command limit remains. The concrete
[storage/checkpoint design](LONG_PLAY_STORAGE_PLAN.md) is a proposal, not an
implemented scalability claim. Account/import/lifecycle/parent-view work, human
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
story GET snapshots use the real schema, explicit speaker/unique-entry prompt,
and low reasoning; the existing story UI already refreshes after actions. Network
I/O happens outside database transactions. Rust validates every proposal, rechecks
session/ownership and state, and discards stale generations. Timeouts, invalid
responses and busy generation use authored fallback. There is one outbound request
at a time and a bounded exact-prompt cache. Command execution, persistent receipts,
learning and guest mode are unchanged. No migration or wire-schema change is needed.

Local HTTP fixtures and router tests cover valid results, malformed/oversized/error
responses, truncation, cache behavior, unavailable endpoints, authentication,
expired sessions, stale state and unchanged command retries. Live model-backed
browser play has not been exercised by this implementation; content approval and
production readiness remain pending.
