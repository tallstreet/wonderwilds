# Wonderwilds implementation plan

Updated: 31 October 2026 · Revision: generated missions delivered; combined-model acceptance and long-play persistence next

## 1. Updated baseline and purpose

The fine-tuned LLM Game Director is already integrated, as confirmed by the user. Fine-tuning and initial integration are completed work, not future prerequisites. Remove the mock Game Director milestone and the later “replace the mock with an LLM” milestone from the implementation queue.

LLM scene generation and persisted mission planning are implemented for the seven supported story opportunities. Both-provider browser journeys now pass across the two supported branches and returning sessions. An intermittent travel timeout remains an open M1 reliability finding; the next substantial engineering milestone is long-play persistence for both story and assessed-adventure state. Extend existing systems rather than rebuild them. Retain authored scenes as outage fallbacks and deterministic test fixtures; this is maintenance of existing resilience, not a new mock-director project.

This revision reconciles the project's current planning index, broader roadmap v1.2, implementation evidence, mastery ADR and long-play storage design with the user's correction. It supersedes their conflicting statements that live integration and fine-tuning are entirely future work. Existing product scope, completed work, compatibility requirements and release gates remain in force. This repository plan incorporates the 31 October mission-planner integration and schema-constrained endpoint evidence. Historical delivery records remain unchanged.

### Evidence and status conventions

- **Completed — user-confirmed:** accepted as the implementation baseline from the user's statement.
- **Implemented — recorded evidence:** repository documentation reports delivery and checks; those historical checks were not rerun for this planning task.
- **Partial:** a delivered foundation exists, with named remaining work.
- **Unverified:** evidence is insufficient to establish current end-to-end behavior, quality or approval. Do not equate this with “not implemented.”
- **Planned:** work explicitly described as outstanding in the inspected project evidence.

This update is based on repository documentation and targeted checks of the story HTTP adapter, plan validator, persistence, browser journey and command-limit definitions. The combined-provider verification below is a new live evaluation; other historical checks were not rerun unless explicitly listed. This is not production verification.

## 2. Preserve delivered work

| Area                     | Baseline to preserve                                                                                                                                             | Status and limits                                                                                                                                                                                      |
| ------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Fine-tuned Game Director | Existing trained model and integration                                                                                                                           | **Completed — user-confirmed.** Immutable model/checkpoint identity remains **unverified**. Endpoint mechanical evaluation and a live planner browser journey are recorded below; combined-provider branch/resumption journeys have now passed; broader quality and an intermittent travel timeout remain outstanding.                                                 |
| Live story adapter       | Optional Modal scene generation, backend-only credentials, Rust proposal validation, stale-result rejection, bounded requests/cache and authored fallback        | **Implemented — recorded evidence**, corroborated by adapter/validator source locations. Current documented scope is story-mode scene prose, not arbitrary mission creation or educational assessment. |
| Mission planner | Optional Modal planning on new mission acceptance, bounded real-domain completion search and atomic saved plans | **Implemented — recorded evidence.** Seven supported opportunities; saved briefs survive reload/pause/retry and participate in export/deletion. No arbitrary mechanics or endless catalogue. |
| Phaser frontend          | Guest adventure, accessible activities, world interactions and existing art/save identities                                                                      | **Implemented — recorded evidence.** Preserve guest save key `starwhisker.meadow.v1` and separate backend progress.                                                                                    |
| Rust and contracts       | Pure domain/learning/content crates, Axum API, closed schemas, generated Rust/TypeScript models and compatibility fixtures                                       | **Implemented — recorded evidence.** Do not introduce a second authoritative TypeScript or Node game engine.                                                                                           |
| Authoritative game loop  | Server assessment, revisions, command retries, unique rewards, support/abandonment and domain replay                                                             | **Implemented — recorded evidence.** Full regional/world eligibility remains **partial**. Pixel movement is local; coarse story transitions are server-checked.                                        |
| Persistence              | PostgreSQL transactions, profile ownership/RLS, receipts, migrations and story checkpoints                                                                       | **Implemented — recorded evidence.** Bounded long-play storage and recovery at scale remain **planned/unverified**.                                                                                    |
| Learning                 | Six assessment families, curriculum graph, deterministic experimental mastery/frontier policy and support distinctions                                           | **Partial.** Calibration, durable recomputation, broader item/context coverage and release approval remain outstanding.                                                                                |
| Creatures                | Twelve illustrated profiles; four Meadow friendships; character XP, sixteen training milestones, team controls, memories and reversible cosmetic transformations | **Implemented — recorded evidence.** Regional collection, automatic friendship participation, optional roles and distinct transformation art remain **partial/planned**.                               |
| Narrative                | Seven connected authored missions, lasting choices, escort/gathering, explicit successor acceptance, reload and compatible story saves                           | **Implemented — recorded evidence.** Generated titles, premises, payoffs and milestone labels are persisted and supplied to scene generation. Broader mission catalogues and editorial prose continuity remain **planned/unverified**; personalised preferences are not established.                                                            |
| Evaluation               | Runtime-derived scenarios, Rust scene grading, reviewed datasets, model comparison tools and versioned train/validation/test preparation                         | **Implemented — recorded evidence.** Initial combined-provider branch/restart evaluation is delivered; the broader longitudinal suite remains **planned**. Authored-reference passes do not establish model quality.                          |

### Latest acceptance evidence — 31 October 2026

- **Combined live journeys passed:** both branches completed seven missions with seven saved generated plans each, accepted scenes in all four sessions, three API restarts/fresh browser contexts, exact retry recovery and unchanged education. Default/alternate recorded 216/223 accepted scene responses and 10/9 fallbacks; counts include caches and QA reads. [Full results](../qa/story-eval/RESULTS_COMBINED_2026-10-31.md) retain the earlier 503 failure and an alternate travel timeout. The diagnostic rerun passed; the timeout cause remains unresolved, so M1 reliability is not unconditionally closed.

- **Planner integration delivered:** migration `0004_story_mission_plans.sql`, state-bound feasibility validation, atomic acceptance/plan/receipt persistence, forced RLS, export/deletion and exact retries. The recorded aggregate and disposable PostgreSQL checks passed.
- **Live planner journey passed:** all seven missions completed with a generated saved plan and matching Missions UI, pause/resume, reload, mobile history and unchanged educational progress. Scene generation was disabled to isolate planning; this is one default-choice journey, not combined-provider or alternate-branch acceptance.
- **Schema-constrained endpoint evaluation:** ordinary and reversed-condition planning each passed 18/18 known-family scenarios. Scenes passed 35/36; a 210-character narration exceeded the 180-character limit and was rejected. Keep local validation and fallback even with `strict: true`.
- **Timing and identity limitations:** endpoint evaluations used a 60-second timeout; the game uses six seconds. The deployed `wonderwilds` alias does not establish an immutable weight revision. These results do not establish editorial approval, unseen-mission generalisation or runtime latency acceptance.

Sources: [integration evidence](IMPLEMENTATION_STATUS.md#mission-planner-game-integration--31-october-2026) and [schema evaluation](../qa/story-eval/RESULTS_JSON_SCHEMA_2026-10-31.md).

Historical status sections reflect earlier delivery stages. Preserve their evidence and dates; do not use older “no model” statements to undo the user's current completion confirmation. Likewise, seven connected story missions and 75 exploration destinations do not prove that the broader regional adventure or full curriculum is complete.

## 3. Architecture and product boundaries

Retain Phaser/TypeScript for presentation, Rust for authoritative gameplay and learning, PostgreSQL for durable state, and the existing provider adapter for model access. Python may support offline training/evaluation.

```text
Persisted player/world state → Rust context and legal-action builder
                                      ↓
                           Integrated fine-tuned director
                                      ↓ proposal
                           Rust validation and binding
                                      ↓
                        Browser presents validated choices
                                      ↓ player command
                       Rust rules / deterministic assessment
                                      ↓ atomic commit
                  State + evidence + rewards + durable receipts
```

The model may propose prose, permitted choices and mission briefs over supported opportunities. Accepted plans are bound to current state and checked by bounded real-domain completion search; milestone completion comes from domain receipts, not model assertions. It cannot declare an answer correct, award XP, grant a creature, change inventory, establish mastery or overwrite world state. Validate all consequential commands independently of model output. Reject stale proposals and re-evaluate against current authoritative state.

Preserve the repository's separation of story mode, assessed adventure and guest saves. Connecting them requires an explicit compatible orchestration contract; do not silently merge saves or send learning records to the existing story endpoint. Story generation currently excludes education. Rust selects and grades maths challenges; any future narrative framing receives only approved, minimal context.

Retain the current three-region initial-release direction, original creatures, supported routes, accessible controls and maths/English scope. Prioritising maths does not delete implemented English work. Adventure XP, character XP, friendship and learning evidence remain distinct. Assistance may advance the adventure without falsely establishing independent mastery. Retain pending educator, art, accessibility, privacy, identity and operational release gates.

## 4. Revised priorities and dependencies

| Priority | Work package                                | Build on existing scopes              | Completion outcome                                                                                         |
| -------- | ------------------------------------------- | ------------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| 1        | P1 — Combined-model acceptance and Rust validation          | WW-02, WW-05, WW-09, WW-13–14         | Every accepted action and model proposal respects current rules, state and ownership.                      |
| 2        | P2 — Durable long-play persistence          | WW-08–09 and long-play storage design | Profiles resume across sessions, failures and migrations without duplicate effects or unbounded hot state. |
| 3        | P3 — Adaptive maths mastery                 | WW-03, WW-06–07                       | Challenge selection adapts from trustworthy, varied evidence across sessions.                              |
| 4        | P4 — Narrative continuity                   | WW-04, WW-12–14 and story plan        | Characters, choices, quests and world consequences remain consistent across encounters and sessions.       |
| 5        | P5 — Creature progression                   | WW-04–05, WW-12 and companion plan    | Existing training grows into persistent regional collection, roles and transformations.                    |
| 6        | P6 — Automated multi-session LLM evaluation | WW-15–16, WW-19 and existing QA       | Reproducible evidence measures the actual integrated model and complete gameplay loop.                     |

P6 starts with a small baseline alongside P1; its final acceptance depends on P2–P5. It is not postponed until all features ship. P3–P5 can be developed against agreed contracts, but integrated acceptance requires P1/P2. Existing WW identifiers remain reference scopes, not a fresh queue of unfinished foundations.

### P1 — Combined-model acceptance and Rust-authoritative validation

**Status:** scene/planner validation and combined live journeys across both branches and four sessions delivered; intermittent travel timeout investigation and broader reliability coverage remain open. **Owner:** backend/domain and contract maintainers.

Tasks:

- Preserve the now-passing seven-mission browser journey with both `WONDERWILDS_MODAL_PLANNER=1` and `WONDERWILDS_MODAL_STORY=1`, covering default and alternate lasting choices. Extend existing adapter/command coverage only where missing.
- Investigate the retained intermittent first-workshop travel timeout using the new command/status diagnostics. Preserve passing restart/resume and uncertain-write recovery checks; extend deliberate provider-outage and stale-generation coverage with both providers enabled. Verify saved briefs, executable choices, history and unchanged educational progress. Measure each provider’s acceptance, fallback and latency under the actual six-second game timeout.
- Enforce schema/bounds, request and revision binding, legal actions, present speakers, inventory/quest prerequisites, reachable destinations and reward eligibility.
- Preserve authoritative coarse movement for the current story slice. Complete broader regional eligibility alongside P5 expansion; it is not a prerequisite for accepting the existing seven-mission loop. Do not require server simulation of every animation frame.
- Preserve exact retries, ownership checks and rejection without partial mutation. Keep network calls outside database transactions and recheck state after generation.
- Version model, prompt, schema, rules and content references; make accepted/fallback reasons observable without credentials or child text.

**Exit evidence:** malformed/invented/stale proposals cannot alter state; illegal actions and changed-body command retries are rejected; simultaneous actions cannot double-award; every displayed accepted choice resolves to a valid command for its revision. Both-provider browser journeys, returning-session continuity and fallback/latency results are recorded separately from fixture tests and the completed planner-only journey. Existing guest and training journeys still pass.

### P2 — Persistence and long-session reliability

**Status:** core persistence implemented; bounded-storage design explicitly **planned**. **Owner:** persistence/API maintainer. **Dependency:** P1 command/replay contract.

This is the next substantial engineering milestone; retain the open M1 timeout finding alongside the delivered combined-model evidence. Follow the existing [long-play storage design](LONG_PLAY_STORAGE_PLAN.md) for assessed-adventure state: externalise API/domain receipts and encounters, introduce explicit event/evidence counters, compact current snapshots, verified checkpoints and bounded replay suffixes. Preserve durable original responses and reward uniqueness; receipt lookup precedes stale-revision rejection. A bounded cache must not erase duplicate prevention.

Add new numbered migrations and compatible readers. Migrate synthetic old profiles transactionally; preserve pending recipients, support, revisions, friendship, appearance and exact retry results. Move mastery reconstruction to durable evidence before removing embedded history. Include profile export/deletion and recovery paths.

The assessed adventure's 2,000-command cap and story mode's 256-command cap are distinct. Keep both until each subsystem has an accepted migration and bounded-work design. Extend the design separately for story checkpoints and replay while preserving the already durable story receipts and mission plans, including their accepted revisions and content versions. Extending adventure persistence does not automatically fix story persistence. Durable history may still grow; bounded working memory is not a claim of constant database size.

**Exit evidence:** more than 2,000 adventure commands and more than 256 story commands across checkpoints and process restarts; unchanged old-command responses; no duplicate rewards/evidence; rollback on injected failures; ownership and deletion tests; migration/replay equality; restored pending encounters; measured state size and per-command work. Exercise supported rollback without overwriting newer progress.

### P3 — Adaptive maths and trustworthy mastery

**Status:** experimental policy implemented; calibration and complete integration **unverified**. **Owner:** learning maintainer with educator review. **Dependencies:** P1 assessment boundary and P2 durable evidence.

- Preserve deterministic Rust answer checking and immutable evidence: skill, item family/parameters, representation, context, session, first-check outcome and assistance.
- Integrate prerequisite/frontier selection, varied item generation, misconception feedback, graduated help, transfer probes and review after a break.
- Preserve the existing experimental `independent-transfer-v1` baseline: seven correct among the latest eight eligible checks, latest three correct, varied items, two sessions/two contexts, transfer evidence and representation requirements. Treat these as versioned hypotheses needing calibration, not proven educational thresholds.
- Keep repeated fixed practice, supported attempts, imported guest records and model assertions from qualifying as independent mastery. Preserve `legacy_unverified` imports.
- Add durable recompute/replay tooling and verify summaries against complete evidence before adopting incremental checkpoints. Expand contexts and worked examples rather than merely repeating instructions.

**Exit evidence:** scripted learners with uneven skills, repeated errors, assistance, repeated items and return visits receive appropriate targets; support never inflates mastery; reload and policy replay reproduce summaries; educator-reviewed transfer items and calibration limitations are recorded. Simulated performance is not evidence of real learning gains.

### P4 — Narrative continuity

**Status:** seven-mission continuity and persisted generated briefs implemented; canonical context and saved-brief continuity verified across both live branches/restarts; broader prose continuity **unverified**. **Owner:** story/domain and director maintainers. **Dependencies:** P1 and P2.

- Assemble bounded context from committed facts: open/completed quests, lasting choices, actual NPC locations, introductions already presented, relationship milestones, inventory and unresolved threads.
- Reuse persisted generated briefs and their content/revision bindings. Extend fact references and continuity context where evidence shows gaps; summaries cannot create facts or override canonical records.
- Test callbacks after reload, return sessions, branch choices and model changes. Distinguish planned, offered, presented and completed events.
- Keep authored world anchors and valid fallback scenes. Reject absent speakers, repeated introductions, impossible inventory claims and dialogue that contradicts completed milestones.
- Preserve the delivered planner and its feasibility checks. Broader opportunities, competing goals and new mission mechanics are later extensions requiring explicit domain support and whole-journey solvability evidence; they are not part of the completed seven-opportunity integration.

**Exit evidence:** both lasting-choice branches remain coherent across sessions; no repeated completion rewards or reset relationships; prompts reflect current state; accepted scenes preserve executable choices; critical contradictions block promotion. Story play alone does not alter assessed learning progress.

### P5 — Creature collection, evolution and progression

**Status:** first persistent training loop implemented; expansion **partial/planned**. **Owner:** domain/content and frontend maintainers. **Dependencies:** P1/P2; P3 for assessed training; P4 for story-linked progression.

- Preserve all four Meadow friendships, sixteen milestones, current level thresholds and unique training grants. Do not replace implemented values with illustrative thresholds from older plans.
- Add richer milestones, automatic friendship participation and optional exploration roles with explicit server conditions.
- Verify the Orchard-unlocked two-slot journey end to end, then integrate Orchard/Cove discovery and the supported ending. Map artwork alone is insufficient.
- Extend transformations/evolution through the existing reversible cosmetic system; retain creature identity, friendship and unlocked appearances. Distinct art and any new ability effects need their own acceptance evidence.
- Keep guaranteed routes to essential companions and support-based story completion. Avoid chance-only progression blockers and repeated-practice XP exploits.

**Exit evidence:** recruit → select → train → level → friendship/transform → regional role → reload works for the declared release roster; duplicate attempts grant nothing extra; changing teams cannot redirect a pending training reward; existing saves preserve creature identities and unlocks.

### P6 — Automated multi-session LLM gameplay evaluation

**Status:** scene/dataset tooling, live endpoint mechanical evaluations and a planner-only browser journey implemented; initial combined-provider four-session branch journeys delivered; full longitudinal harness **planned**. **Owner:** evaluation/QA maintainer. **Dependencies:** baseline starts now; full gate follows P1–P5.

Extend the existing runtime-derived Rust grader and Python evaluation tools. Drive actual commands through persisted synthetic profiles and the integrated model adapter. Single-scene grading, authored references and model-judge recommendations remain useful but are not substitutes for complete journeys.

Proposed delivery stages:

1. Record the integrated model/checkpoint when available and all prompt/content/rules/schema versions. Preserve the explicit alias-only limitation until an immutable model identity is available. Extend P1’s delivered combined-provider, both-branch multi-session baseline with broader provider-failure coverage and separate fallback attribution.
2. Add scripted player policies: independent success, repeated misconceptions, support use, exploration, abandoned tasks, inconsistent choices and long absences. Use deterministic policies for reproducibility; optional LLM players are not the grading oracle.
3. Persist, restart and resume each profile. Inject lost responses, exact retries, two-tab races, expired sessions, stale generation, outage and content migration.
4. Run an initial proposed suite of 100 synthetic profiles over at least five resumed sessions each, covering all declared playable missions and progression paths. Add dedicated runs beyond both command limits after P2 passes. Record actual completed coverage rather than counting planned runs.
5. Preserve failures as regression fixtures. Keep full journey families together in dataset splits; exclude training overlap and distinguish known-family performance from unseen-mission generalisation.

| Measure             | Required report or gate                                                                                                                                                  |
| ------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| State integrity     | Zero observed illegal committed transitions, duplicate rewards or cross-profile changes in the required suite. Any failure blocks acceptance.                            |
| Persistence         | Zero lost committed progress or replay mismatches across required restarts, migrations and retry cases.                                                                  |
| Learning            | Zero false independent mastery caused by assistance, duplicated items, imported evidence or model claims; report difficulty progression by learner policy.               |
| Narrative           | Report contradictions, repeated introductions, unresolved-thread loss and repetition per journey; zero unresolved critical contradictions in reviewed acceptance traces. |
| Playability         | Report completion, supported completion, dead ends and abandonment separately; all required supported paths complete.                                                    |
| Model reliability   | Report first-pass acceptance, invalid output, truncation, timeout, stale result, cache hits and fallback separately. Fallback never counts as a valid model generation.  |
| Experience and cost | Report latency distributions, calls/tokens/cost per session, variety and human-reviewed readability. Set numerical budgets after the baseline and before promotion.      |

Store a run manifest, synthetic profile seeds, command traces, state hashes, version identifiers, candidate provenance, failure reasons and aggregate results. Replay committed transitions without calling the model; do not assume live generations are deterministic. A calibrated human sample assesses narrative and educational suitability; a model judge cannot approve its own release. Keep frozen final-test data out of tuning.

## 5. Milestone acceptance sequence

| Gate                               | Scope                                                      | Required evidence                                                                                                                                             |
| ---------------------------------- | ---------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| M0 — Baseline reconciled           | Completed model integration and existing features retained | Status ledger, exact integration scope, version manifest and unresolved evidence list. No new mock milestone.                                                 |
| M1 — Trusted integrated loop       | P1 plus initial P6 baseline                                | Both-provider branches, restart/resume, retry recovery and response/timing evidence delivered. Open: intermittent travel timeout, detailed fallback attribution and broader failure coverage.                                                |
| M2 — Durable returning player      | P2                                                         | Long-run, checkpoint, migration, restart, exact-retry and recovery evidence for both state systems.                                                           |
| M3 — Adaptive persistent adventure | P3–P5                                                      | Reviewed maths variants, coherent returning-player story and declared creature progression journeys; existing compatibility preserved.                        |
| M4 — Evaluated model gameplay      | P6 full suite                                              | Versioned report, required coverage, hard-gate passes, agreed quality/latency/cost budgets and owned remaining findings.                                      |
| M5 — Release readiness             | Existing broader roadmap gates                             | Parent identity/lifecycle/import flows, educator/content/art/accessibility review, deployment/security/backup/rollback evidence and consented pilot approval. |

These gates distinguish delivered components from remaining acceptance: M0’s scope/status reconciliation is recorded here, with immutable model identity still unresolved; M1 has successful combined-loop evidence but retains an unresolved intermittent timeout; M2–M5 remain outstanding. Further fine-tuning is driven by measured failures and comparative evaluation; it is not a prerequisite to acknowledge the existing integration.

## 6. Immediate implementation handoff

1. **Finish M1 reliability follow-up:** preserve the successful combined-provider branch/restart/retry journeys and reproduce the retained first-workshop travel timeout with the new command diagnostics. Separate busy, transport and invalid-output fallback reasons before claiming unique-generation reliability or provider latency budgets. Investigate the known scene-length failure without weakening validation.
2. **Build M2:** implement assessed-adventure bounded storage and a separate compatible story checkpoint/replay design. Preserve both command caps until disposable-database long-run, migration, old-retry, rollback and measured-work acceptance passes. Keep migration 0004 and saved plans intact; use new numbered migrations.
3. **Grow P6 alongside delivery:** preserve versioned synthetic traces and returning-session regressions. Record the model alias honestly until immutable identity is available; do not count fallback as successful generation.
4. **Then expand P3–P5:** adaptive maths, narrative continuity, companion roles and Orchard/Cove progression. Define an explicit story/education orchestration contract before connecting their separate progress systems. Further fine-tuning follows measured failures, not a presumed missing integration milestone.

Each implementation handoff should include changed scope, versions, compatibility/migration effects, tests actually run, representative saved-state traces and unresolved limitations. Documentation-only work does not require rerunning the application suite. Future persistence changes require disposable real-PostgreSQL checks, not unit tests alone; future browser changes require guest, training and story regressions.

## 7. Source record and unresolved evidence

Sources inspected in the existing project: `docs/IMPLEMENTATION_PLAN.md`, `docs/WONDERWILDS_IMPLEMENTATION_PLAN.md` v1.2, `docs/IMPLEMENTATION_STATUS.md`, `docs/MILESTONE_REVIEW.md`, `docs/LONG_PLAY_STORAGE_PLAN.md`, `docs/adr/mastery-policy.md`, `README.md` and `AGENTS.md`. Source locations checked include the Modal adapter, story validator, HTTP integration and command-limit definitions. The planning index also identifies the companion and story plans as supporting specifications.

The earlier 28 October exported plan was found at `/Users/gary/Documents/Codex/2026-10-28/referenced-chatgpt-conversation-this-is-an-4/outputs/Wonderwilds_Implementation_Plan.md`. Its “no backend” baseline and seven-star/Mochi blueprint are superseded by the existing project's later implementation roadmap; they are not restored here.

The referenced conversation, “Next Implementation Milestone” (`6ae47407-7f14-83ec-a89a-59fcd2eb182d`), and the current user request establish the model-completion correction and revised priorities.

Current evidence also includes the 31 October mission-planner integration and schema evaluation linked above, plus targeted checks of `backend/crates/api/src/story/{http,planning,repository}.rs` and `qa/e2e/story-broader.mjs`.

Still **unverified or outstanding**: exact deployed fine-tune manifest; cause of the intermittent alternate-run travel timeout and broader combined-provider failure coverage; broader comparative/editorial quality; runtime latency and full multi-session evaluation; durable bounded-storage implementation for both state systems; mastery calibration; broader regional authority; shared story/education orchestration; production identity, operations and release approvals. The planner-only and combined-provider branch/restart journeys and persisted generated briefs are delivered evidence, not items awaiting initial implementation.
