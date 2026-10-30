# Wonderwilds implementation plan

Updated: 31 October 2026 · Revision: integrated-LLM roadmap update

## 1. Updated baseline and purpose

The fine-tuned LLM Game Director is already integrated, as confirmed by the user. Fine-tuning and initial integration are completed work, not future prerequisites. Remove the mock Game Director milestone and the later “replace the mock with an LLM” milestone from the implementation queue.

The next milestone is a reliable, persistent, Rust-authoritative gameplay loop around the integrated director. Extend existing systems rather than rebuild them. Retain authored scenes as outage fallbacks and deterministic test fixtures; this is maintenance of existing resilience, not a new mock-director project.

This revision reconciles the project's current planning index, broader roadmap v1.2, implementation evidence, mastery ADR and long-play storage design with the user's correction. It supersedes their conflicting statements that live integration and fine-tuning are entirely future work. Existing product scope, completed work, compatibility requirements and release gates remain in force. This is a delivered Markdown revision; the source repository documents have not been overwritten.

### Evidence and status conventions

- **Completed — user-confirmed:** accepted as the implementation baseline from the user's statement.
- **Implemented — recorded evidence:** repository documentation reports delivery and checks; those historical checks were not rerun for this planning task.
- **Partial:** a delivered foundation exists, with named remaining work.
- **Unverified:** evidence is insufficient to establish current end-to-end behavior, quality or approval. Do not equate this with “not implemented.”
- **Planned:** work explicitly described as outstanding in the inspected project evidence.

Repository inspected: `/Users/gary/Documents/Development/wonderwilds`; checkout was clean at final inspection, HEAD `0ed92fe`. This was a document review with limited source-location checks, not a full code audit, live model evaluation or production verification.

## 2. Preserve delivered work

| Area                     | Baseline to preserve                                                                                                                                             | Status and limits                                                                                                                                                                                      |
| ------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Fine-tuned Game Director | Existing trained model and integration                                                                                                                           | **Completed — user-confirmed.** Model/checkpoint identity, measured quality and live browser acceptance evidence remain **unverified** in this review.                                                 |
| Live story adapter       | Optional Modal scene generation, backend-only credentials, Rust proposal validation, stale-result rejection, bounded requests/cache and authored fallback        | **Implemented — recorded evidence**, corroborated by adapter/validator source locations. Current documented scope is story-mode scene prose, not arbitrary mission creation or educational assessment. |
| Phaser frontend          | Guest adventure, accessible activities, world interactions and existing art/save identities                                                                      | **Implemented — recorded evidence.** Preserve guest save key `starwhisker.meadow.v1` and separate backend progress.                                                                                    |
| Rust and contracts       | Pure domain/learning/content crates, Axum API, closed schemas, generated Rust/TypeScript models and compatibility fixtures                                       | **Implemented — recorded evidence.** Do not introduce a second authoritative TypeScript or Node game engine.                                                                                           |
| Authoritative game loop  | Server assessment, revisions, command retries, unique rewards, support/abandonment and domain replay                                                             | **Implemented — recorded evidence.** Full regional/world eligibility remains **partial**. Pixel movement is local; coarse story transitions are server-checked.                                        |
| Persistence              | PostgreSQL transactions, profile ownership/RLS, receipts, migrations and story checkpoints                                                                       | **Implemented — recorded evidence.** Bounded long-play storage and recovery at scale remain **planned/unverified**.                                                                                    |
| Learning                 | Six assessment families, curriculum graph, deterministic experimental mastery/frontier policy and support distinctions                                           | **Partial.** Calibration, durable recomputation, broader item/context coverage and release approval remain outstanding.                                                                                |
| Creatures                | Twelve illustrated profiles; four Meadow friendships; character XP, sixteen training milestones, team controls, memories and reversible cosmetic transformations | **Implemented — recorded evidence.** Regional collection, automatic friendship participation, optional roles and distinct transformation art remain **partial/planned**.                               |
| Narrative                | Seven connected authored missions, lasting choices, escort/gathering, explicit successor acceptance, reload and compatible story saves                           | **Implemented — recorded evidence.** Persistent personalised narrative briefs and broader generated missions remain **unverified/planned**.                                                            |
| Evaluation               | Runtime-derived scenarios, Rust scene grading, reviewed datasets, model comparison tools and versioned train/validation/test preparation                         | **Implemented — recorded evidence.** Automated live-model multi-session gameplay evaluation remains **unverified**. Authored-reference passes do not establish model quality.                          |

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

The model may propose prose and permitted choices. It cannot declare an answer correct, award XP, grant a creature, change inventory, establish mastery or overwrite world state. Validate all consequential commands independently of model output. Reject stale proposals and re-evaluate against current authoritative state.

Preserve the repository's separation of story mode, assessed adventure and guest saves. Connecting them requires an explicit compatible orchestration contract; do not silently merge saves or send learning records to the existing story endpoint. Story generation currently excludes education. Rust selects and grades maths challenges; any future narrative framing receives only approved, minimal context.

Retain the current three-region initial-release direction, original creatures, supported routes, accessible controls and maths/English scope. Prioritising maths does not delete implemented English work. Adventure XP, character XP, friendship and learning evidence remain distinct. Assistance may advance the adventure without falsely establishing independent mastery. Retain pending educator, art, accessibility, privacy, identity and operational release gates.

## 4. Revised priorities and dependencies

| Priority | Work package                                | Build on existing scopes              | Completion outcome                                                                                         |
| -------- | ------------------------------------------- | ------------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| 1        | P1 — Rust authority and validation          | WW-02, WW-05, WW-09, WW-13–14         | Every accepted action and model proposal respects current rules, state and ownership.                      |
| 2        | P2 — Durable long-play persistence          | WW-08–09 and long-play storage design | Profiles resume across sessions, failures and migrations without duplicate effects or unbounded hot state. |
| 3        | P3 — Adaptive maths mastery                 | WW-03, WW-06–07                       | Challenge selection adapts from trustworthy, varied evidence across sessions.                              |
| 4        | P4 — Narrative continuity                   | WW-04, WW-12–14 and story plan        | Characters, choices, quests and world consequences remain consistent across encounters and sessions.       |
| 5        | P5 — Creature progression                   | WW-04–05, WW-12 and companion plan    | Existing training grows into persistent regional collection, roles and transformations.                    |
| 6        | P6 — Automated multi-session LLM evaluation | WW-15–16, WW-19 and existing QA       | Reproducible evidence measures the actual integrated model and complete gameplay loop.                     |

P6 starts with a small baseline alongside P1; its final acceptance depends on P2–P5. It is not postponed until all features ship. P3–P5 can be developed against agreed contracts, but integrated acceptance requires P1/P2. Existing WW identifiers remain reference scopes, not a fresh queue of unfinished foundations.

### P1 — Rust-authoritative validation

**Status:** foundation implemented; complete boundary coverage **unverified**. **Owner:** backend/domain and contract maintainers.

Tasks:

- Trace the real adapter → validator → browser choice → command → commit path, identifying which checks already exist and adding only missing coverage.
- Enforce schema/bounds, request and revision binding, legal actions, present speakers, inventory/quest prerequisites, reachable destinations and reward eligibility.
- Complete authoritative regional eligibility where browser-only exploration currently exceeds server story integration. Define the consequential movement boundary without requiring server simulation of every animation frame.
- Preserve exact retries, ownership checks and rejection without partial mutation. Keep network calls outside database transactions and recheck state after generation.
- Version model, prompt, schema, rules and content references; make accepted/fallback reasons observable without credentials or child text.

**Exit evidence:** malformed/invented/stale proposals cannot alter state; illegal actions and changed-body command retries are rejected; simultaneous actions cannot double-award; every displayed accepted choice resolves to a valid command for its revision. Real-adapter browser smoke is recorded separately from fixture tests. Existing guest and training journeys still pass.

### P2 — Persistence and long-session reliability

**Status:** core persistence implemented; bounded-storage design explicitly **planned**. **Owner:** persistence/API maintainer. **Dependency:** P1 command/replay contract.

Follow the existing long-play storage design: externalise API/domain receipts and encounters, introduce explicit event/evidence counters, compact current snapshots, verified checkpoints and bounded replay suffixes. Preserve durable original responses and reward uniqueness; receipt lookup precedes stale-revision rejection. A bounded cache must not erase duplicate prevention.

Add new numbered migrations and compatible readers. Migrate synthetic old profiles transactionally; preserve pending recipients, support, revisions, friendship, appearance and exact retry results. Move mastery reconstruction to durable evidence before removing embedded history. Include profile export/deletion and recovery paths.

The assessed adventure's 2,000-command cap and story mode's 256-command cap are distinct. Keep both until each subsystem has an accepted migration and bounded-work design. Extending adventure persistence does not automatically fix story persistence. Durable history may still grow; bounded working memory is not a claim of constant database size.

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

**Status:** seven-mission authored continuity implemented; broader model continuity **unverified**. **Owner:** story/domain and director maintainers. **Dependencies:** P1 and P2.

- Assemble bounded context from committed facts: open/completed quests, lasting choices, actual NPC locations, introductions already presented, relationship milestones, inventory and unresolved threads.
- Add a versioned narrative brief and fact references where needed; summaries cannot create facts or override canonical records.
- Test callbacks after reload, return sessions, branch choices and model changes. Distinguish planned, offered, presented and completed events.
- Keep authored world anchors and valid fallback scenes. Reject absent speakers, repeated introductions, impossible inventory claims and dialogue that contradicts completed milestones.
- Treat generated mission planning as a separate extension beyond the currently evidenced scene-prose integration. Require validated operations, solvability and supported completion before enabling it.

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

**Status:** scene/dataset tooling implemented; full longitudinal harness **unverified**. **Owner:** evaluation/QA maintainer. **Dependencies:** baseline starts now; full gate follows P1–P5.

Extend the existing runtime-derived Rust grader and Python evaluation tools. Drive actual commands through persisted synthetic profiles and the integrated model adapter. Single-scene grading, authored references and model-judge recommendations remain useful but are not substitutes for complete journeys.

Proposed delivery stages:

1. Pin the integrated model/checkpoint and all prompt/content/rules/schema versions. Capture a small real-model multi-session baseline, including provider failures and fallback attribution.
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
| M1 — Trusted integrated loop       | P1 plus initial P6 baseline                                | Actual model-backed browser journey and boundary regression results; invalid outputs safely rejected/fallback.                                                |
| M2 — Durable returning player      | P2                                                         | Long-run, checkpoint, migration, restart, exact-retry and recovery evidence for both state systems.                                                           |
| M3 — Adaptive persistent adventure | P3–P5                                                      | Reviewed maths variants, coherent returning-player story and declared creature progression journeys; existing compatibility preserved.                        |
| M4 — Evaluated model gameplay      | P6 full suite                                              | Versioned report, required coverage, hard-gate passes, agreed quality/latency/cost budgets and owned remaining findings.                                      |
| M5 — Release readiness             | Existing broader roadmap gates                             | Parent identity/lifecycle/import flows, educator/content/art/accessibility review, deployment/security/backup/rollback evidence and consented pilot approval. |

These gates are planned acceptance criteria, not claims that tests have passed. Further fine-tuning is driven by measured failures and comparative evaluation; it is not a prerequisite to acknowledge the existing integration.

## 6. Immediate implementation handoff

1. Record the deployed model/checkpoint identity and reconcile stale planning statements without deleting historical test evidence.
2. Trace and test the real adapter-to-command boundary; record an actual model-backed browser journey.
3. Begin the existing long-play persistence design and a separate story-cap migration design, preserving both guards until acceptance.
4. Extend evaluation with resumed profiles immediately, then expand coverage as mastery, narrative and creature work integrates.

Each implementation handoff should include changed scope, versions, compatibility/migration effects, tests actually run, representative saved-state traces and unresolved limitations. Documentation-only work does not require rerunning the application suite. Future persistence changes require disposable real-PostgreSQL checks, not unit tests alone; future browser changes require guest, training and story regressions.

## 7. Source record and unresolved evidence

Sources inspected in the existing project: `docs/IMPLEMENTATION_PLAN.md`, `docs/WONDERWILDS_IMPLEMENTATION_PLAN.md` v1.2, `docs/IMPLEMENTATION_STATUS.md`, `docs/MILESTONE_REVIEW.md`, `docs/LONG_PLAY_STORAGE_PLAN.md`, `docs/adr/mastery-policy.md`, `README.md` and `AGENTS.md`. Source locations checked include the Modal adapter, story validator, HTTP integration and command-limit definitions. The planning index also identifies the companion and story plans as supporting specifications.

The earlier 28 October exported plan was found at `/Users/gary/Documents/Codex/2026-10-28/referenced-chatgpt-conversation-this-is-an-4/outputs/Wonderwilds_Implementation_Plan.md`. Its “no backend” baseline and seven-star/Mochi blueprint are superseded by the existing project's later implementation roadmap; they are not restored here.

The referenced conversation, “Next Implementation Milestone” (`6ae47407-7f14-83ec-a89a-59fcd2eb182d`), and the current user request establish the model-completion correction and revised priorities.

Still **unverified** in this review: exact deployed fine-tune manifest; actual live-model browser acceptance; comparative model-quality results; complete multi-session evaluation; durable bounded-storage implementation; mastery calibration; broader regional authority; shared story/education orchestration; production identity, operations and release approvals. Resolve each through evidence rather than assuming either completion or absence.
