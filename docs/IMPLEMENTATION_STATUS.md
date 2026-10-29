# Wonderwilds implementation evidence

Foundation evidence recorded 28 October 2026; integration updates dated below. Broader roadmap: [Wonderwilds implementation plan](WONDERWILDS_IMPLEMENTATION_PLAN.md) v1.2. Active gameplay sequence: [Companion progression plan](COMPANION_PROGRESSION_PLAN.md). Validation counts in earlier sections describe their original delivery, not the latest checkout.
Six subagents implemented distinct scopes; the integration owner reviewed and
validated the combined changes. Existing source, guest saves and art are preserved.
No release, deployment, curriculum approval or learning-efficacy claim is made.

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
| WW-14–16 | Not implemented; authored game has no model dependency | Provider/privacy/budget decisions, adapter, evaluation corpus and gated training experiment |
| WW-17 | Existing guest parent view preserved; Rust summaries/client foundation available | New parent learning view, approved pack workflow and coverage dashboard |
| WW-18 | Local reproducible install/start/migration/check helpers and GitHub Actions configuration | Hosted CI execution, database/browser CI, deployment, observability, backup restore and rollback drills, operational owner |
| WW-19–20 | Automated foundations and synthetic checks only | Integrated QA/security review, consented pilot, educator audit and measured curriculum expansion |
| Authored story slice | Pure Pip bridge runtime/payload builder, separate authenticated PostgreSQL saves and opt-in `/story.html` controls | Phaser/world integration, broader missions/preferences, model adapter/evaluation and human usability review |

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
