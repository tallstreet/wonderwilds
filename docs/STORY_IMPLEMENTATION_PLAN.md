# Story implementation plan

Status: authored foundation, API/browser and playable mock-driven world deliveries implemented and checked. 30 October 2026.

Design: [Mission-led generative storytelling](GENERATIVE_STORY_DESIGN.md).
The first delivery is an executable headless authored mission and deterministic
input builder. It does not replace the current assessed bridge, modify guest
saves or expose a model/provider. API/database and browser integration follow
explicitly after this foundation is verified.

## Fourth delivery: connected authored stories

Delivered on `codex/playable-story-series`, with a PR covering the playable world,
mock-provider seam, startup-address fix and this expansion. Three stories reuse
the same supported actions and locations:

1. **Pip’s bridge:** gather wood, prepare planks, repair the bridge and cross together.
2. **Pip’s picnic:** gather far-bank berries, pack a basket at the workshop, prepare
   the near-bank table, then bring Pip back across to share the picnic.
3. **Welcome lights:** gather glow pebbles, make lanterns at the workshop and light
   both banks. Near and far installations may be completed in either order.

Each completion offers the next mission explicitly. The repaired bridge, Pip’s
location, picnic and lights remain world facts; new missions do not reset old
progress. Pausing preserves supplies and milestone receipts. Completed missions
remain in compact story memory. Academic activities stay separate.

Domain subagent owns mechanics, resource conservation and strict legacy checkpoint
replay; director/API subagent owns truthful payloads/scenes, version-compatible
contracts and database verification; browser subagent owns mission-specific art
bindings, both crossing directions, visible outcomes and real three-story QA.
The integration owner reviews these changes, runs combined/regression checks and
opens the PR. No provider or paid model call is introduced.

Compatibility acceptance: validate old `pip-bridge-1` checkpoints before projecting
them into `pip-stories-2`; preserve revision and original response receipts. New
version-bound action handles must reject old ambiguous handles even at the same
revision. Never reset a save or edit an applied migration to upgrade content.
Verify old completed saves can continue, new checkpoints replay all missions,
wrong-mission recipes fail atomically, and real browser progress survives reloads.

Verification: the aggregate repository check and disposable PostgreSQL suite pass,
including a literal historical completed bridge checkpoint and original response
receipts. The real browser completes all three missions, pauses/resumes the picnic,
reloads between stories, recovers an uncertain lantern exchange with the exact
command, and checks both Pip crossing directions, persistent scenery and all three
completed memories. Desktop/tablet guest smoke and the real assessment/training
journey also pass. Local tooling used Node 26; CI retains the required Node 24.
This is a finite authored series, with no fourth mission after Welcome lights.

## Third delivery: playable world with mocked story responses

Delivered: the real Phaser world connects to the persisted story flow at
`/?story=1`, discoverable through Missions in the normal game. `/story.html` remains a diagnostic
scene view. This delivery uses a sequence of deterministic mock scene responses
for the full Pip bridge mission; no model, paid provider or training is required.

| Work | Owner | Acceptance |
| --- | --- | --- |
| S10. Mock scene provider | Director subagent | Actual compact input selects mock dialogue/choices; shared-schema and semantic validation bind speakers and legal handles; invalid output has authored fallback |
| S11. World adapter | Browser subagent | Server snapshot drives present NPCs, conversations, resource interactions, inventory, mission milestones, visible bridge state and shared crossing in Phaser |
| S12. End-to-end verification | QA subagent and integration owner | Complete actual walking/interactions with real API/database; check reload/retry, keyboard/mobile, story continuity and untouched educational/guest records |

The child walks to characters and resources, talks, chooses a current legal action
and sees the confirmed result in the world. Dialogue comes from returned scenes;
the browser does not invent conversations or grant inventory/milestones. An
authored registry maps supported character/object/location IDs to existing art and
reachable placements. Server coarse location changes occur through validated story
commands; pixel movement is a browser concern and is not server-verified.

Preserve the current retry/session protections. Do not replay a conversation as
remembered merely because it was fetched. Hide stale choices while writes or
reconciliation are pending. Existing assessed activities and guest saves keep their
own paths. This is one complete mission with multiple scripted story responses,
not arbitrary generated missions or a claim of an endless story catalogue.

### Third-delivery evidence and limits

The director subagent supplied the mock-provider seam and scene validation;
the browser subagent implemented the world adapter; a QA subagent independently
reviewed both and built the real-world journey. The integration owner reviewed
source and screenshots, resolved review findings and ran aggregate/regression
checks. The pipeline is confirmed action → saved state → compact input → mock
scene → validated snapshot → world/dialogue/mission UI. Replacing the provider
later does not give it authority to grant items, rewards or arbitrary state.

Pip occupies the near/far meadow bank according to server state; the woodcutter
uses existing Bramble art in the orchard workshop. Supported IDs bind to authored,
reachable placements. Wood/bridge/route interactions require walking to their
markers. Ordinary map exits are disabled in this mode. A confirmed shared crossing
animates the player and Pip over the measured bridge deck, then presents Pip on
the far bank without a duplicate follower. Existing activities stay in the main
adventure and are not a prerequisite for these story actions.

Review fixes keep the displayed world on its last confirmed current snapshot when
an old receipt is followed by a failed refresh; pending recovery blocks movement
and choices. Terminal snapshots stop rendering actionable scenery, and lifecycle
guards prevent late startup responses from mounting a departed page. Every present
NPC has a returned dialogue line across reachable states. Invalid proposals fall
back to authored scenes; this binding validation is not general prose safety or
truth verification for a future model.

Validation passed: aggregate `bash scripts/check.sh`, full frontend check with
102 tests, disposable PostgreSQL suite, both actual Phaser wood-source journeys,
existing real training and guest desktop/tablet regressions. The final expanded
bank journey also verified Missions discovery, inert HTML-looking response text,
second-tab advancement with an older exact retry and failed GET, locked recovery,
and terminal rendering. Actual bridge bounds and Pip's mid-crossing follower were
checked; desktop/mobile/crossing screenshots were inspected. Final typecheck,
format and diff checks passed. CI includes both world routes; hosted execution
and Node 24 remain unverified locally (Node 26.12.0 was used).

One concurrent database startup hit macOS shared-memory capacity; its disposable
cluster was removed. The suite passed after browser clusters stopped. Persistent
demo data was never used by QA. This remains one complete authored mission with
fixed preferences and a 256-command cap, not a chain of new missions. No model,
fine-tuning, new artwork, academic gate or release approval was introduced.

## First delivery and ownership

| Work | Owner | Acceptance |
| --- | --- | --- |
| S1. Authored story mechanics | Domain subagent | Pure Rust story state, reusable bound actions, two wood sources, exchange, repair, shared crossing; prerequisites and resources checked atomically |
| S2. Story payload contracts and builder | Payload subagent | Versioned closed contracts, generated Rust/TS, shared fixtures; deterministic planning/scene projections, authored scenes, no educational or identity fields |
| S3. Executable journey | Integration owner | Runnable CLI/example traverses real mechanics and prints real payloads; checkpoint/replay exercise uses validated state rather than invented summaries |
| S4. Independent review | Review subagent + integration owner | Verify state invariants, retries, completion conditions, continuity and payload source mapping against this plan; resolve concrete findings |
| S5. Documentation and checks | Integration owner | Commands and limitations documented, relevant suites run, diff inspected; status records delivered vs deferred work honestly |

### Required behavior

- Accept an authored mission to repair the bridge and cross with Pip.
- Milestones describe conditions: wood ready, planks ready, planks at bridge,
  bridge repaired, shared crossing. Existing compatible inventory is respected.
- A registered source can be collected only at its location and within its supply.
  Wood can come from either bridge bank or workshop.
- Exchange consumes three wood and grants three planks. Assembly consumes three
  planks and changes the bridge once. Crossing completes only at the far bank
  together with Pip. Ordinary travel alone cannot fulfill the shared outcome.
- State changes only after a successful action. Invalid actions leave state
  unchanged. Same command/body retry cannot duplicate effects; a changed body
  with the same ID or a stale new command is rejected.
- Cancel/resume preserves actual items and bridge changes without refund exploits.
  Coarse travel verifies supported route/gate transitions, not pixel movement.
- Store milestone receipts and accepted narrative separately from current inventory.
  Consumption does not erase a historical milestone. Out-of-order legitimate
  actions and existing inventory cannot strand the authored mission.
- Persisted/checkpoint input is untrusted: reject malformed or inconsistent data;
  never reset corrupt progress silently. First delivery uses a developer harness,
  not production database storage or browser-authoritative saves.
- Planning payload contains the small authored area, relevant registry bindings,
  opportunities and state. Scene payload contains current context, mission progress
  and executable actions. Sources are explicit; empty preferences use safe defaults.
- Displaying a scene does not grant items or progress. An acknowledgement can record
  presented information only; generated/unshown dialogue is not a remembered meeting.
- Model mode remains disabled. Authored scenes and action-derived labels are enough
  to complete the mission and inspect every input payload.

### Verification matrix

| Concern | Required evidence |
| --- | --- |
| Mechanics | Complete journey from both sources; existing wood/planks; invalid location/recipe/quantity; conservation and one repair |
| Continuity | Detour and return, cancellation/reacceptance, crossing alone then with Pip, checkpoint round-trip and corruption rejection |
| Commands | Exact retry, changed-ID-body conflict, stale revision, atomic rejection |
| Payload | Stable golden JSON, state-derived inventory/progress, revision changes after action, remembered introduction only after acknowledgement |
| Boundaries | Closed-schema positive/negative fixtures; no education, account or arbitrary text injection into model context |
| Existing behavior | Workspace format/Clippy/tests and shared contracts/generated checks; relevant content checks |

## Follow-on integration (not claimed by the first delivery)

1. Persist story state and receipts through owned-profile PostgreSQL transactions,
   with new numbered migrations/contracts as required. Add authenticated story
   endpoints and coarse-location transition rules. Run disposable-database tests
   for rollback, duplicate commands, races, RLS and ownership.
2. Add an opt-in browser story view using the new commands and saved scenes. Supply
   real placements/art/controls for workshop and crossing. Keep the existing
   educational/guest paths intact; test keyboard/touch, reload, interruption and
   browser-to-server command bindings. Do not claim pixel travel verification from
   the headless coarse-location rules.
3. Capture synthetic payloads from that integrated journey and review source
   sufficiency, context size, missing data, narrative continuity and privacy.
4. Add gated generated scenes while retaining the authored mission and fallback.
   Select/review provider separately; no paid requests or training in this delivery.
5. Add several supported opportunities and generated mission proposals only after
   feasibility validation and whole-journey diversity evidence exist. Fine-tuning
   remains optional and follows a measured prompted baseline.

## Second delivery: authenticated persistence and browser view

This delivery implements the first two follow-on stages as an opt-in `/story.html`
view. It uses the authored bridge and deterministic inputs, with model generation
still disabled. The view represents three coarse locations using accessible scene
controls. It does not claim Phaser path traversal, change the current assessed
bridge or share its inventory/rewards with this separate story slice.

| Work | Owner | Acceptance |
| --- | --- | --- |
| S6. Story HTTP contracts/application | API subagent | Authenticated GET story and POST story commands, closed contracts, current handle/revision checks, explicit acknowledgement metadata, original error/auth conventions |
| S7. PostgreSQL storage | Storage subagent | New migration, separate profile-owned story progress/receipt tables, forced RLS, transaction lock, exact original-response retries, rollback/corruption/race/deletion/export checks |
| S8. Opt-in browser | Browser subagent | Shared-schema response validation, actual server-derived scenes/progress, keyboard/touch controls, exact uncertain retry across reload, no client-granted progress |
| S9. Integration review | Integration owner and cross-review | Review source against these criteria; run disposable database and real browser journeys; record limitations and results |

### Required boundaries

- Reuse verified sessions, profile ownership and Origin/CSRF protections. No new
  login bypass, checkpoint import or reset endpoint.
- Store story separately from the existing educational/game aggregate. A story
  action cannot alter learning evidence, Explorer XP or current game progression.
- Lock the owner-scoped profile row for story writes and deletion. Save checkpoint
  and original command response together; replay a same-body/ID response even
  after later progress. Reject changed-body duplicates and stale new commands.
- Migration is additive and numbered. Corrupt stored state fails closed rather
  than silently starting over. Profile deletion cascades; export covers story data.
- Browser sends action handles and acknowledgement entry IDs, not inventory/state
  patches. Acknowledgement must not loop as scene revisions change. Exact pending
  commands survive reload separately from server progress; uncertain writes are
  retried without inventing a new ID or body.
- The existing 256-command bound remains. At exhaustion, reads expose an honest
  terminal view with no actionable choices; old exact receipt retries still work.
- Tests always use a fresh synthetic PostgreSQL cluster. Never run destructive
  fixtures against `.local-demo/` or user progress.

### Evidence to collect

HTTP auth/CSRF/schema tests; PostgreSQL rollback, races, ownership/RLS, original
retry response, corruption, export/delete and old-game isolation; browser full
mission, both material sources where practical, acknowledgement, pause/resume,
reload, uncertain retry, keyboard/mobile, safe rendering and original-game isolation.
Run the aggregate suite and inspect the final diff. Browser travel is a verified
coarse state transition selected by a control, not proof of continuous walking.

## Delivery evidence — first headless slice

S1–S5 are delivered for the headless scope above:

- [Domain runtime](../backend/crates/domain/src/story/mod.rs) owns state, actions,
  receipts, milestones, coarse routes, presentation acknowledgements and replay.
- [Payload builder](../backend/crates/director/src/story/mod.rs) emits validated
  planning/scene inputs, authored plans/scenes and revision-bound action handles.
  The schema adds nine Story definitions and sixteen fixtures while preserving
  every prior definition and fixture unchanged.
- [Journey harness](../backend/crates/director/examples/story_bridge.rs) runs both
  sources, writes per-revision payloads/checkpoints, resumes a checkpoint and can
  execute a current action handle. See the root README for commands.
- [Integration exploration](../backend/crates/director/tests/story_journey.rs)
  checks reachable semantic states for valid payloads, executable handles,
  inventory conservation, present speakers and identical post-replay projections.

The domain and payload subagents implemented S1/S2. An independent review subagent
checked them and the harness against this plan; the integration owner reviewed
the code and added the reachable-state integration test. Four concrete findings
were fixed with regression coverage: an absent Pip could speak after solo return;
dialogue incorrectly credited construction of a pre-repaired bridge; paused
missions lost context; and initial context asserted an invitation before acceptance.
The builder also fails explicitly when the 256-command developer budget is exhausted.

Validation performed:

- `bash scripts/check.sh` passed: workspace formatting/Clippy/tests, 89 shared
  contract fixtures, TypeScript/generated checks, content validators and tests,
  frontend checks/build (91 tests) and four legacy compatibility tests.
- Both `story_bridge --source bank` and `--source workshop` journeys passed with
  exact retry checks and unchanged payloads after checkpoint replay.
- The independent reviewer checked saved payloads against state, manual action
  selection, stale handles, corrupted checkpoints and output-directory protection;
  a separate exploration of 82 semantic runtime states found a completion path
  from each state within the bounded scenario (excluding the command budget).
- `git diff --check` passed. Checks used pinned Rust 1.99.1 and available Node
  26.12.0; Node 24 was unavailable. The existing Phaser bundle-size warning remains.

At the first delivery, no database or browser behavior changed, so PostgreSQL/browser integration
checks were not run. Two existing database tests remain ignored by the ordinary
Rust suite. This delivery does not establish API persistence, browser movement,
production save authenticity, accessibility of new controls or model quality.

The first implementation uses one immutable authored opportunity/brief with
content-version pinning, not a generic generated-plan feasibility solver or a
persisted model-created plan. Its catalogue is authored code, and preferences are
fixed defaults. Local checkpoints replay and check consistency but are not
authenticated saves. Capability descriptions and mechanics are checked together by
tests; the broader shared content catalogue is still follow-on work. No release
review, provider call, training run or content activation occurred.

## Delivery evidence — authenticated story slice

S6–S9 are delivered for the separate scene-view scope:

- API/contracts subagent added `StoryCommand` and `StorySnapshot`, authenticated
  story routes, current-handle/acknowledgement checks and terminal snapshots.
  All pre-existing schema definitions are unchanged. Only the woodcutter greeting
  currently creates a presented-history fact, through explicit Continue.
- Storage subagent added migration 0003, forced-RLS story checkpoints/receipts,
  atomic profile-locked persistence and profile export/delete coverage. Reads
  replay saved checkpoints; neither GET nor rendering changes progress.
- Browser subagent added `/story.html`, shared-schema validation, responsive
  keyboard/touch controls and exact pending-command persistence in profile-scoped
  session storage. Server progress is never loaded from browser storage.
- The integration owner reviewed source and desktop/mobile screenshots, ran the
  real journeys and regressions, and added both wood routes to browser CI. The
  storage subagent independently reviewed API/persistence and browser recovery.

Review fixes preserve uncertain commands through authentication/validation
failures, reconnect after browser back-cache restoration, and require a current
snapshot after replaying an older receipt. Failed refresh keeps new choices
disabled. Raw action IDs were replaced with friendly labels and pause is a
secondary control after progress actions. These fixes have regression coverage.

Validation passed: `bash scripts/check.sh` (102 shared fixtures, 98 frontend tests,
workspace format/Clippy/tests, content checks, production build and four legacy
tests); disposable `scripts/check-postgres.sh` (including all three ignored database
integration tests); both source journeys with real API/PostgreSQL; existing real
training journey; desktop/tablet guest smoke and accessible activity browser tests.
Browser checks cover explicit acknowledgement, pause/resume, save-response loss,
reload/exact retry, inventory, shared crossing, keyboard/mobile controls and
unchanged assessed-game state. The final UI/recovery polish was checked again on
the workshop route. `git diff --check` passed. Local tools were pinned Rust 1.99.1
and available Node 26.12.0; Node 24 and hosted CI execution were not verified.

Limits remain explicit: one authored mission, fixed preferences, three coarse
locations, 256 accepted commands per profile, no reset/import endpoint, no Phaser
walking integration and no automatic next mission. This verifies development
integration, not human accessibility/child usability approval or model quality.
No model generation, fine-tuning, academic gate or release activation was added.
