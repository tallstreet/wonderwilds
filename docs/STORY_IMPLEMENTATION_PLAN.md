# Story implementation plan

Status: first headless delivery implemented and checked. 30 October 2026.

Design: [Mission-led generative storytelling](GENERATIVE_STORY_DESIGN.md).
The first delivery is an executable headless authored mission and deterministic
input builder. It does not replace the current assessed bridge, modify guest
saves or expose a model/provider. API/database and browser integration follow
explicitly after this foundation is verified.

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

## Delivery evidence

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

No database or browser behavior changed, so new PostgreSQL/browser integration
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
