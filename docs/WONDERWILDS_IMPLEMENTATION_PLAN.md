# Wonderwilds implementation plan

Version: 1.1 · Updated: 29 October 2026 · Status: broader architecture and release roadmap

The [companion progression plan](COMPANION_PROGRESSION_PLAN.md) governs the next gameplay implementation sequence and supersedes earlier collection/progression sequencing here. Use [implementation status](IMPLEMENTATION_STATUS.md) for delivered work. Existing WW task IDs remain reference scopes, not a queue to rebuild completed foundations. Live AI and fine-tuning are deferred and do not block the authored companion expansion.

## 1. Outcome and non-negotiable decisions

Build a browser-based educational adventure with original cute kittens and cartoon companions, exploration, friendship, collecting, transformations, and an authored story that grows with the learner. Design initially for Foundation–Year 2 learners, roughly ages 5–8, with young girls as the initial research audience and inclusive character choices. Ability, accessibility needs, and interests determine challenges; gender and age do not determine difficulty.

**The production backend MUST be Rust.** Retain the existing Phaser/TypeScript browser frontend. Rust owns assessment, mastery, authoritative state, rewards, persistence, authentication integration, and AI orchestration. Python may be used for offline model training/evaluation, never as a replacement gameplay backend.

Pokémon provides inspiration through discovery, companion attachment, collection completion, team choices, transformations, regional journeys, and milestone challenges. All names, creatures, art, stories, music, terminology, and mechanics implementations must be original. No copied characters, capture devices, battle interfaces, or proprietary assets.

The AI director proposes the next adventure beat; a deterministic Rust engine alone commits the next game state. Personalised variation is a goal, not a promise that every playthrough will be globally unique. Every adventure remains playable with AI disabled.

Initial release: three authored regions, an ending, twelve collectible friends, six reusable activity families, a reviewed Foundation skill slice in maths and English, adaptive sequencing, and parent-managed saves. Full Foundation coverage is a later explicitly measured gate; Years 1–2 expansion follows. Do not advertise complete curriculum coverage or proven learning gains at the initial release.

## 2. Existing project: preserve and extend

The current repository is `/Users/gary/Documents/Development/wonderwilds`, under Git with npm lockfiles and a pinned Rust workspace. The Phaser guest adventure is preserved; the local server-backed world now connects six Meadow story beats and four friendships through the real API and PostgreSQL.

| Existing material | Current baseline | Next work |
| --- | --- | --- |
| `frontend/src/world.ts` and movement modules | Playable Meadow, server-driven world changes, one follower | Team followers and regional integration |
| `frontend/src/backend-world.ts` and `backend-world-model.ts` | Server activities, journal, companion selection and summaries | Character progression, training and collection UI |
| `backend/crates/domain/src/engine.rs` | Explorer XP, friendship, teams, discovery and cosmetic transformations | Individual XP, training grants and integration |
| `backend/crates/api/src/game/` | Authenticated transactions, retries, encounter support and abandonment | Expanded commands/snapshots, training and regional eligibility |
| `contracts/` | Schemas, generated Rust/TS and shared fixtures | Compatible extensions and migration fixtures |
| `content/` | Three authored regions, twelve friends, curriculum/activity definitions | Training milestones, personal quests and remaining content reviews |
| Guest save and existing art | Separate local progress and preserved hero IDs; friend art provisional | Preserve guest data and provide distinct friend art |

Regional travel, transformations in the browser, production account flows and guest import remain outstanding. The local developer session is not production identity integration. No live model is required for the current authored adventure. Preserve the original guest save key `starwhisker.meadow.v1` and existing hero IDs.

### Migration rules

1. Preserve the current guest adventure and save key until a tested migration exists.
2. Map `missing-10`, `share-6`, and `count-1` to new skill IDs, retaining original IDs and source versions in imported records.
3. Import local story/cosmetic progress once, with a migration receipt. Never interpret editable browser evidence as verified server mastery.
4. Keep a downloadable original save; reject malformed data without destroying it. Show a clear recovery route.
5. Existing heroes and companions keep their IDs; rebranding changes display text, not save identity.
6. Future Orchard/Cove content is not evidence that those maps are ready for release. Activate only after navigation, story and content gates pass.

## 3. Game design and learning loop

### Story spine

A gentle magical storm scatters the Wonder Lantern's three lights. The player and Pip reconnect the island communities by helping friends solve practical problems. The storm creates inconvenience and mystery, not frightening danger. Fixed anchors guarantee continuity; optional side quests, creature encounters, decorations and dialogue vary.

| Chapter | Story and gameplay | Milestone / feature unlocked | Learning opportunities |
| --- | --- | --- | --- |
| Clover Meadow | Restore the bridge, share a picnic, wake a garden, discover a misplaced letter | First lantern light; Pip; journal and team slot | Counting, parts of a whole, equal sharing, sounds and labels |
| Peachy Orchard | Restore seed routes, organise a festival, reunite a courier with its delivery | Second light; gardening and habitat decoration; second team slot | Patterns, comparing measures, decoding labels, ordering instructions |
| Starlight Cove | Repair the observatory and guide friends to the lantern celebration | Third light; constellation journal; expedition remix mode | Spatial reasoning, data sorting, story sequence and comprehension |

A chapter finale is a cooperative multi-step expedition that combines practised skills. The learner can use support and still complete the chapter. Difficulty adapts within each region; story access never requires mastery of a specific academic band.

Core loop: explore → discover a need → choose a friend/tool → manipulate meaningful objects or words → receive explanatory feedback → see the world change → gain friendship/discoveries → choose the next destination. Target 10–15 minute sessions with natural stopping points; validate this in playtests rather than imposing timers.

### Collection and progression

- Twelve initial original friends: four per region, with distinct silhouettes, personalities, preferred habitats and assistance abilities. Pip remains the first companion; existing roster appearances remain selectable avatars.
- Befriend creatures by completing help quests and discovering habitat clues. No paid random drops, coercive streaks, real-time expiry, trading or multiplayer in the first release.
- Maintain four independent progress axes: Explorer XP (story/discoveries), character XP (individual growth), friendship (companion interactions), and skill evidence (learning). Assistance never removes story rewards.
- Example level thresholds, subject to playtest tuning: 0, 20, 50, 90, 140 XP. Award first-time story XP once. Character training awards XP once per authored milestone, with fresh practice variants and no repeated reward for an already granted milestone; see the companion plan.
- Companion transformations remain reversible cosmetic milestones from friendship and story flags; preserve existing unlocks. New character-level rewards and optional exploration abilities follow the companion plan. No creature becomes unhappy when the player leaves.
- Abilities reveal optional paths or offer manipulatives, such as lighting a clue or organising objects. They cannot bypass authority checks or count as independent learner answers.
- Team selection creates gentle strategy: choose a light, garden or pattern helper. Every essential route has a free, accessible alternative; no rare creature blocks learning.
- Journal includes silhouettes, habitat clues and guaranteed discovery paths. A configurable seeded encounter table limits repetition and guarantees access to missing friends after a bounded number of eligible expeditions.
- Player choices affect optional ordering and cosmetic outcomes. Fixed chapter anchors, prerequisite checks and an authored ending prevent incoherent or endless stories.

## 4. Curriculum: original graph and coverage process

Use IXL Foundation maths as a breadth reference, not a source of questions, taxonomy IDs, proprietary scoring, screenshots or copied wording. Use Australian Curriculum Version 9 as the alignment authority; state/territory implementation and exact content descriptions require educator review. Our node names, task designs, examples, hints and prerequisite edges are original product design.

Sources: [IXL Foundation maths](https://au.ixl.com/maths/foundation), [ACARA Mathematics structure](https://www.australiancurriculum.edu.au/curriculum-information/understand-this-learning-area/mathematics), and [ACARA English structure](https://www.australiancurriculum.edu.au/curriculum-information/understand-this-learning-area/english). Maths spans six strands; English spans Language, Literature and Literacy. This graph is a proposed teachable decomposition, not a claim that every listed node is a formal Foundation requirement.

### Seed graph

Prerequisites in the table mean all listed nodes, unless an educator supplies an explicit alternative path. Empty prerequisites mean entry nodes. Nodes are Foundation candidates unless marked extension. Each node is decomposed into calibrated difficulty bands and multiple item families during content authoring.

| ID | Original objective | Prerequisite IDs | Natural interaction |
| --- | --- | --- | --- |
| M01 | Match one object to each recipient | — | Give each kitten a cup |
| M02 | Recognise tiny quantities without counting | — | Spot a small firefly group |
| M03 | Count collections and connect totals to numerals, gradually to 20 | M01 | Pack a requested berry basket |
| M04 | Compare and order collections | M03 | Choose sufficient picnic supplies |
| M05 | Move forward/backward through a counting sequence | M03 | Wake consecutive flowers |
| M06 | Compose and partition a small whole | M03 | Fit missing bridge planks |
| M07 | Model joining and taking away with objects | M06 | Adjust lantern fuel crystals |
| M08 | Share a small collection fairly | M01, M03 | Distribute picnic berries |
| M09 | Identify, extend and create repeating patterns | — | Arrange a lantern garland |
| M10 | Sort and describe familiar shapes | — | Fit pieces into a habitat door |
| M11 | Use position and movement language | — | Place supplies beside a tree |
| M12 | Compare length, mass and capacity directly | — | Select a suitable rope or container |
| M13 | Order familiar events and connect them to daily time | — | Prepare a festival timetable |
| M14 | Sort collections by a stated attribute | — | Organise shell discoveries |
| M15 | Gather and interpret simple category counts | M03, M14 | Plan snacks from friends' choices |
| M16 | Group tens and ones (Year 1 extension) | M03, M06 | Bundle garden seeds |
| M17 | Use equal groups and skip-counting (extension) | M05, M08 | Lay orchard planting rows |
| M18 | Partition familiar wholes equally (extension) | M08, M10 | Share a circular picnic loaf |
| E01 | Follow short spoken directions and take turns | — | Listen to Pip's route request |
| E02 | Connect useful vocabulary with objects/actions | E01 | Choose a tool from a spoken clue |
| E03 | Notice rhymes and syllables | E01 | Echo a creature's rhythm |
| E04 | Identify and manipulate sounds in spoken words | E01 | Sort deliveries by initial sound |
| E05 | Connect taught letters with their sounds | E04 | Match a sound to a letter tile |
| E06 | Blend taught sounds to decode regular words | E05 | Read decodable parcel labels |
| E07 | Segment and encode regular words with taught letters | E04, E05 | Build a sign with letter tiles |
| E08 | Recognise explicitly taught common word forms | E05 | Read recurring trail signs |
| E09 | Track print and sentence boundaries | E01 | Arrange a short message |
| E10 | Read a controlled short sentence | E06, E08, E09 | Follow a written delivery instruction |
| E11 | Recall characters and sequence events in a heard story | E01, E02 | Restore a picture storybook |
| E12 | Retrieve meaning from a short independently read text | E10 | Find which friend needs the red basket |
| E13 | Compose a short meaningful message with scaffolds | E07, E09 | Make a thank-you card |
| E14 | Discuss a character's choice and give a reason | E11 | Choose a kind response and explain |
| E15 | Extend spelling patterns and sentence detail (extension) | E10, E13 | Write an expedition postcard |

Listening comprehension and reading comprehension remain distinct. Reading an assessment aloud changes what it measures; record that support. Dialect, multilingual background and motor access must not be mistaken for low ability. Free composition, oral language and handwriting cannot be certified by a tile-choice game: provide adult-reviewed activities and explicitly mark unassessed outcomes. No microphone is required in the first release.

### Node schema and coverage ledger

`SkillNode@1` contains `id`, `subject`, `objective`, `band`, `prerequisites`, `allowedRepresentations`, `parameterBounds`, `misconceptionCodes`, `hintLadder`, `evidencePolicyId`, `contentVersion`, `reviewStatus`, and `alignmentRefs`. References hold source URL, curriculum version, exact verified descriptor code, reviewer and date. Unknown descriptor codes stay null; never invent alignment codes.

Maintain `content/curriculum/coverage.csv`: source concept reference, original node IDs, coverage status (unmapped/authored/reviewed/tested/released), excluded outcomes with rationale, representative activities, assessment limits, reviewer. A coverage audit identifies omitted Foundation concepts and adds original nodes. “All Foundation skills” is achieved only when every reviewed in-scope concept is mapped and tested, and exclusions are disclosed. IXL breadth coverage and ACARA alignment are separate columns, not interchangeable certifications.

Every released node needs at least three parameter variants across two meaningful contexts where applicable, explicit independent/support scoring, valid and invalid fixtures, and an educator-approved prerequisite rationale. Further years are separate versioned packs, never an automatic scaling of numbers alone.

## 5. Adaptive mastery specification

Implement a transparent rules baseline in Rust before experimenting with statistical learner models. Thresholds below are initial testable hypotheses, subject to educator calibration and evidence from playtests.

- States: unseen → introduced → practising → secure; optional review-needed after fresh difficulty. Secure skills leave mandatory practice selection immediately.
- Evidence is an event for each answer check, hint, worked example, abandonment and completion, not just successful completions. Store item family/variant, representation, skill, first-check correctness, support, encounter ID, and server sequence. Do not infer inability from inactivity or speed.
- Initial secure rule: at least eight eligible first checks across three variants, two representations where valid, and two sessions; at least seven correct, including the latest three; at least one novel transfer item; no support on qualifying checks. A session is server-issued; duplicate attempts and repeatedly reopening the same item do not create independent evidence.
- Repeated identical item-family/parameter combinations contribute at most once to a qualification window. Assisted completions preserve diagnostic history but do not qualify for independent mastery.
- After two unsuccessful checks, offer a concrete hint; after three, offer a worked example or simpler prerequisite. No lives lost and no negative labels.
- Select an eligible frontier node whose prerequisites are secure, or an educator-approved entry probe. Among candidates, prefer weak evidence, curriculum balance and varied contexts. Seed tie-breaking for reproducibility.
- Difficulty rises through an explicit ordered parameter/representation ladder, not arbitrary model changes. Two independent successes at a rung allow a harder probe; difficulty falls after repeated difficulty with support offered first.
- Optional revision can be offered in the journal, clearly labelled, with no mandatory repetitive drill or lost rewards. Fresh difficulties in composite tasks can trigger an offered refresher; no automatic calendar-based demotion.
- If all configured nodes are secure, offer enrichment and notify the grown-up that the current pack is complete. Never repeat mastered tasks indefinitely to fill time.
- Explanations expose why the next task was selected and distinguish “practised”, “supported”, and “secure in these activities”. No diagnostic, age-equivalence or school-grade claims.

Keep story gates and academic gates separate. Parents may choose maths, English or mixed sessions; a subject switch does not reset evidence. Calibration must measure false mastery and false non-mastery against independently reviewed transfer tasks, including accessibility and language subgroups.

## 6. Architecture and authoritative interfaces

```text
Browser: existing Phaser world + HTML activities + generated TypeScript API client
    → HTTPS /api/v1
Rust API (Axum/Tokio) → Rust domain/assessment/mastery crates → PostgreSQL
    → Rust director worker → replaceable model provider
    → approved, versioned content packs / private object storage
Offline tooling: synthetic dataset builders, educator review, evaluation, optional fine-tuning
```

Architecture map (existing core paths with optional future tooling):

```text
Cargo.toml                            # existing root workspace
backend/{crates/{domain,learning,content,api,director},migrations/}
contracts/{openapi.yaml,schemas/,fixtures/}
content/{curriculum,activities,story,creatures,manifests}/
frontend/src/{api,learning,ui}/          # add incrementally to current modules
ml/{dataset,eval,training}/
infra/                                # containers, environment and deployment definitions
qa/{e2e,fixtures,load}/
docs/{adr,privacy,runbooks}/
```

Retain the pinned Rust toolchain, dependency versions and committed Cargo.lock; update them only for a concrete requirement. PostgreSQL handles transactions, unique grants and revision checks. Start with a modular monolith and a worker using a transactional outbox; avoid microservices and extra infrastructure without measured need.

### Shared interfaces

| Contract | Required behaviour |
| --- | --- |
| `GameCommand@1` | `commandId`, `expectedRevision`, command type and typed payload; identity comes from the authenticated session, never trusted body IDs |
| `GameSnapshot@1` | Revision, content/rules versions, story flags, owned creatures, inventory, pending encounters and learner-facing summaries |
| `EncounterInstance@1` | Server ID, template/skill/version, approved parameters, permitted actions and support state; answers remain server-side |
| `LearningEvidence@1` | Server-issued event/encounter IDs, skill/variant/context, answer result, support flags, server session and order; raw child text excluded by default |
| `DomainEvent@1` | Typed immutable gameplay event with per-profile sequence, originating command, rules/content versions and deterministic outcome |
| `ContentManifest@1` | Pack/version/hash, schema range, asset IDs, dependencies, educator approval and provenance |
| `DirectorInput@1` / `DirectorProposal@1` | Bounded, versioned proposal protocol below; cannot execute commands directly |

Endpoints: `GET /api/v1/game`, `POST /api/v1/commands`, `GET /api/v1/encounters/{id}`, `POST /api/v1/encounters/{id}/attempts`, `POST /api/v1/encounters/{id}/hints`, and parent-only profile/export/delete endpoints. Attempt/hint writes use the same command ID and revision discipline. A next-encounter request can return an authored encounter immediately while a future proposal is prepared.

Errors include stable codes: `INVALID_INPUT` (400), `UNAUTHENTICATED` (401), `FORBIDDEN` (403), `NOT_FOUND` (404), `REVISION_CONFLICT` (409), `RATE_LIMITED` (429), and `UNAVAILABLE` (503). Return correlation IDs without secrets or child payloads. Retries reuse command IDs; a repeated ID with a different body is rejected. Revision conflicts fetch fresh state and ask the UI to reconcile intent, never silently overwrite.

### Deterministic engine invariants

`apply(state, command, rulesVersion, contentVersion, recordedRandomChoice) -> events | rejection` is pure. Server handlers perform authentication and load/save transactions around it. The seed/PRNG version, accepted random choices and accepted AI proposal are recorded so replay never calls a live model or wall clock. Inject timestamps for learning sessions and expiry decisions.

- Server verifies reachability, prerequisites, item validity, response correctness and reward eligibility.
- Rewards have a unique `(profile, rewardDefinition, questInstance)` grant key; repeat story instances have explicit reward policy.
- State revision, evidence, grants and outbox writes commit atomically.
- Hint requests are recorded before answers; submitted client support flags cannot make an attempt independent.
- Client movement may animate locally, but interaction eligibility uses validated region/position/path state. Teleporting a client cannot unlock story flags.
- Browser hints or answer inspection can never be fully prevented. Mastery is evidence from activities, not a high-stakes exam guarantee.
- Guest/offline practice remains available. Offline learning evidence is labelled unverified and cannot be promoted to cloud mastery by replaying local JSON.
- Retain versioned readers and golden replay fixtures. Content removal must supply a safe replacement for pending encounters.

## 7. Generative director and fine-tuning

### Authority and data flow

The learning engine chooses the skill, bounds and candidate activity families. The story engine chooses allowed beats and entities. The director chooses among those candidates and produces a short, coherent proposal. It cannot set correctness, XP, inventory, mastery, creature ownership, URLs, executable code, or arbitrary world patches.

Input includes a compact structured recap of canonical story facts, recent template/creature IDs, learner skill-state categories, reading band, available content and allowed transitions. Do not send name, email, exact date of birth, raw answers, free-text companion names, IP, account IDs or full histories. Request IDs are ephemeral correlations, not stable child identifiers. Derived progress can still be personal data; apply provider agreements and minimisation accordingly.

Illustrative valid pair; WW-02 supplies strict JSON Schemas with bounded arrays/text, closed enums and `additionalProperties: false`.

```json
{
  "schemaVersion": "1.0",
  "requestId": "req-demo-01",
  "stateRevision": 42,
  "rulesVersion": "1.0.0",
  "contentVersion": "foundation-1.0.0",
  "policyVersion": "child-director-1",
  "context": {
    "regionId": "meadow",
    "canonicalFacts": ["pip_met", "bridge_pending"],
    "recentTemplateIds": ["berry_sharing"],
    "readingBand": "supported",
    "skillStates": [{"skillId": "M06", "status": "practising"}]
  },
  "allowed": {
    "beatIds": ["help_pip_cross"],
    "creatureIds": ["pip"],
    "skillId": "M06",
    "templateIds": ["missing_quantity"],
    "parameterSets": [{"total": 7, "given": 3}],
    "nextBeatIds": ["picnic_invitation"],
    "maxDialogueCharacters": 180
  }
}
```

```json
{
  "schemaVersion": "1.0",
  "requestId": "req-demo-01",
  "stateRevision": 42,
  "contentVersion": "foundation-1.0.0",
  "beatId": "help_pip_cross",
  "creatureId": "pip",
  "encounter": {
    "templateId": "missing_quantity",
    "skillId": "M06",
    "parameters": {"total": 7, "given": 3}
  },
  "dialogue": "Pip is ready to help repair the bridge. Let's find the missing planks!",
  "suggestedNextBeatId": "picnic_invitation"
}
```

The engine computes four missing planks. Authoritative quantity instructions and assessment text are rendered from the template; free dialogue supplies flavour and must not contradict them. Initial live mode uses approved dialogue IDs if semantic safety is uncertain; unrestricted generated assessment text is excluded.

Validation pipeline: byte/timeout limits → JSON/schema validation → supported versions → request/revision match → allowlist and curriculum checks → canonical narrative checks → content safety and reading-level checks → deterministic encounter construction → atomic commit. A proposal cannot grant `suggestedNextBeatId`; completion rules decide whether it becomes eligible.

Unknown versions, unsafe text, impossible geometry, invented IDs, repeated stale proposals, unavailable providers or invalid arithmetic all fall back to authored content. No raw model output reaches the child. Deadline target: 2 seconds; one bounded retry only if time remains; circuit breaker and per-profile/global cost caps. Log acceptance/rejection codes and version IDs, not prompts by default. Compatibility: additive optional fields require tested minor-version support; semantic/breaking changes require a new major schema and explicit migration.

### Dataset and training plan

Start with authored director, then constrained-prompt baseline, then a genuine fine-tuning experiment if the baseline has measured deficits. Gameplay release is not blocked on a fine-tuned model. Model-provider selection follows a privacy, structured-output, cost, hosting-region and licence review; no provider is assumed in this plan.

`DirectorTrainingRecord@1` fields: dataset/example IDs, schema/policy/content versions, synthetic scenario seed, source/provenance/licence, input, approved target proposal, rubric scores, reviewer IDs, review date, split group, exclusion reason and integrity hash. Store rejected outputs and reasons in a separate evaluation/red-team collection, not as approved supervised targets.

1. Produce 2,000–5,000 original synthetic/author-written examples spanning story states, skill bands, repetition pressure, allowed choices and difficult constraints. This is a planning range, not an asserted sufficient dataset size.
2. Educators review pedagogy; narrative reviewers review continuity and tone; child-safety reviewers review harmful or manipulative content. Resolve disagreements before promotion.
3. Deduplicate by scenario family, template structure and semantic similarity. Split 80/10/10 by scenario/story family before generating variants; reserve entirely unseen regions and adversarial cases for a separate holdout. Never distribute near-duplicate seeds across splits.
4. Train a candidate from a licensed base with supported fine-tuning. Save data hashes, base revision, hyperparameters, seeds, evaluator version, cost, model card and rollback identifier.
5. Compare authored baseline, constrained base model and fine-tuned model on identical frozen cases. Repeat stochastic evaluations and report confidence intervals and per-category failures.
6. Deploy only if the candidate meets gates and demonstrates useful improvement. An unsuccessful experiment still delivers dataset, reproducible run and decision; retain the safer baseline.

Use synthetic and licensed adult-authored data by default. Production child data is excluded from training. Any later research use needs a separate approved purpose, consent/legal review, minimisation and deletion plan; pseudonymisation alone is insufficient. Do not claim that deleting a dataset row removes learned information from existing weights; avoid such training unless retraining/retirement obligations are resolved.

### Release evaluation gates

Use at least 1,000 frozen ordinary scenarios plus 500 adversarial cases, balanced across subjects and branches. These are release test targets, not evidence already collected.

| Measure | Required result |
| --- | --- |
| Raw model schema adherence | At least 99%; all remaining cases safely rejected |
| Accepted encounter validity | 100% deterministic constraint and answer checks |
| Unsafe, PII-seeking or coercive child-visible output | Zero observed critical failures; every known adversarial fixture blocked; no claim of absolute safety |
| Narrative consistency and age suitability | At least 95% educator/reviewer approval; no critical contradiction |
| Curriculum bounds | 100% accepted cases within engine-selected skill and parameters |
| Repetition | No duplicate encounter instance; at most one repeated template within last three beats when alternatives exist |
| Failure recovery | Every timeout, malformed output and outage fixture yields playable authored fallback |
| Response time | Non-model command p95 <300 ms at agreed pilot load; director deadline 2 s with fallback |
| Fine-tuning benefit | Pre-registered improvement on the baseline's primary deficit, no safety/learning regression, within declared cost budget |

Human pilots measure whether children understand activities and transfer skills. Offline director scores cannot demonstrate learning gains.

## 8. Security, child privacy and deployment

Treat browsers, imported saves, model outputs and authored pack uploads as untrusted inputs. Parent authentication, cross-profile authorisation and curriculum/reward validation are server-side. Parent controls require reauthentication, not a child-readable arithmetic gate. Use synthetic profiles for development/testing.

| Boundary / risk | Required control and verification |
| --- | --- |
| Child profile isolation | Ownership checks on every route/query; two-parent cross-profile negative tests |
| Session theft / CSRF | Secure HttpOnly SameSite cookies, explicit CSRF protection on writes, session expiry/rotation; invalid-origin and logout tests |
| Client reward/evidence fabrication | Server encounters, revision checks, idempotency and atomic grants; replay/race/tamper tests |
| Generated text / injection | Plain-text rendering, no HTML/URLs/tools, closed schemas, bounded inputs, output safety checks; injection fixtures |
| Content upload | Staff role checks, pack signatures/hashes, immutable versions, review before activation; malicious pack tests |
| Infrastructure exposure | Least-privilege DB roles, private storage, secret manager, TLS, restricted egress, dependency/SBOM checks; configuration verification |
| Sensitive logs | Structured allowlisted fields, no raw prompts/answers/names, retention jobs; log snapshot and canary tests |
| Availability/cost | Request/body limits, rate limits, worker concurrency, timeouts, spending caps, circuit breaker; bounded fault tests |

No ads, behavioural advertising, public profiles, open chat, location collection, biometric collection or child email login. Parent account stores contact details separately from pseudonymous child progress. Use an ability band instead of exact date of birth where possible. Self-host fonts/assets and remove unnecessary third-party calls. Review browser speech behaviour; use reviewed local/pre-generated narration when data routing is unclear.

Draft retention proposal for legal/product approval: short-lived director context in memory; operational logs 14 days; raw learning events 90 days before aggregation; progress until account deletion or an agreed inactivity expiry with parent notice; backups expire within 35 days. Implement deletion tombstones so restored backups cannot resurrect deleted profiles. Complete parent exports/deletions within a documented service target (proposed 30 days), subject to verified applicable requirements. Provide a child-readable notice and parent notice, a consent register where required, vendor inventory and incident response runbook.

As checked on 28 October 2026, OAIC describes the Australian Children's Online Privacy Code as a draft with registration planned by 10 December 2026. Do not present draft provisions as current enacted obligations; re-check at launch and assess Privacy Act/APP applicability with qualified counsel. [OAIC code status](https://www.oaic.gov.au/privacy/privacy-registers/privacy-codes/childrens-online-privacy-code), [OAIC draft announcement](https://www.oaic.gov.au/news/media-centre/oaic-releases-exposure-draft-of-the-childrens-online-privacy-code).

If launching in the US, assess COPPA applicability and verified parental consent before collecting covered child data; the FTC's 2025 amendments address third-party disclosure and retention. Geography and vendor choices must be decided before public accounts go live. This is a design requirement, not a legal compliance certification. [FTC final rule notice](https://www.ftc.gov/legal-library/browse/federal-register-notices/16-cfr-part-312-coppa-final-rule-amendments).

Deployment: static frontend through a CDN; containerised Rust API/worker; managed PostgreSQL; private content storage; separate development/staging/production accounts. Prefer Australian hosting for the initial audience, but verify every vendor/subprocessor and cross-border transfer rather than equating hosting region with compliance. Use expand/contract database migrations, compatibility-tested content activation, feature flags for AI and new regions, and rollback to a known application/content/model combination. Back up encrypted data and rehearse restoration. Proposed pilot targets: 99.5% availability, RPO 24 hours, RTO 4 hours; owners approve these before launch.

## 9. Reference work packages

These original WW work packages retain scope, interfaces and acceptance requirements for the broader release. Consult implementation status before selecting work: many foundations are already implemented, with review and integration gates still open. The companion plan replaces their immediate gameplay execution order. These packages do not require parallel agents or new worktrees; use the existing checkout unless the user requests otherwise.

Only the integration owner edits root manifests, shared contracts or shared entrypoints when concurrent tasks overlap. Feature owners expose functions/components in their own paths and submit explicit wiring instructions. Schema changes require an ADR and consumer fixture updates; never silently change another task's contract. Respect remaining dependencies and the companion plan delivery order.

Every handoff includes changed files, contract versions, exact test commands/results, fixtures, migration/rollback notes, limitations and reviewer. “Done” means deliverables checked and acceptance evidence attached; placeholders and unrun tests do not count. Security findings require concrete boundary evidence, not speculative vulnerability labels.

### WW-01 — Baseline and migration inventory

- **Depends:** none. **Scope:** `docs/adr/baseline.md`, baseline fixtures and repository setup; no gameplay redesign.
- **Interfaces:** existing save v1, current module exports and scripts.
- **Deliverables:** tracked baseline, asset/provenance inventory, captured save fixtures, dependency/lockfile decision, current regression results and known failures.
- **Acceptance:** existing adventure and saves accounted for; no existing work lost; current guest/server boundaries recorded.
- **Tests:** current frontend check command; desktop/tablet smoke; valid/corrupt/storage-blocked save fixtures. Record failures before changing code.

### WW-02 — Contracts and Rust workspace

- **Depends:** WW-01. **Scope:** `contracts/`, root Rust workspace and crate shells, architecture ADR; integration owner.
- **Interfaces:** all versioned contracts in section 6; generated TS types and Rust serde models.
- **Deliverables:** JSON Schemas, OpenAPI, fixtures, crate dependency boundaries, generation/check scripts, pinned toolchain/lockfile.
- **Acceptance:** Rust and TS accept/reject identical fixtures; unknown major versions and extra director fields rejected; no browser-only type leaks into domain crate.
- **Tests:** schema positive/negative suite, round trips, codegen drift and minimal cargo/frontend compile.

### WW-03 — Original curriculum and coverage map

- **Depends:** WW-02. **Scope:** `content/curriculum/` only.
- **Interfaces:** `SkillNode@1`, evidence policies and content manifest.
- **Deliverables:** seed DAG above, explicit difficulty ladders, curriculum crosswalk, misconception/hint metadata, educator review ledger.
- **Acceptance:** no cycles/dangling IDs; exact curriculum codes verified or marked pending; original content; released nodes have approved scope.
- **Tests:** graph reachability/cycle checks, parameter bounds and coverage-status validation; educator review of all released nodes.

### WW-04 — Story and creature content

- **Depends:** WW-02. **Scope:** `content/story/`, `content/creatures/`, asset manifest; no existing atlas edits without integration coordination.
- **Interfaces:** story graph, creature definitions, unlock/reward definitions, approved asset IDs.
- **Deliverables:** three chapter spines, twelve friends, friendship/XP tables, final expedition, fallback beats and content provenance.
- **Acceptance:** every story has a supported completion route and ending; essential routes do not require rare friends; no borrowed franchise assets.
- **Tests:** story reachability, unlock-cycle detection, bounded discovery simulation and asset reference validation.

### WW-05 — Pure Rust gameplay engine

- **Depends:** WW-02, WW-04. **Scope:** `backend/crates/domain/`.
- **Interfaces:** command → events/rejection; content versions and injected clock/randomness.
- **Deliverables:** inventory, quests, collection, levels, friendship, transformations, gates and replay implementation.
- **Acceptance:** identical inputs produce identical events; supported completions unlock story; duplicate rewards impossible within domain transitions.
- **Tests:** property tests for nonnegative inventories/monotonic revisions, duplicate commands, illegal transitions, deterministic replay and every chapter finale.

### WW-06 — Activity generation and deterministic assessment

- **Depends:** WW-02, WW-03. **Scope:** `backend/crates/learning/assessment/`, `content/activities/`.
- **Interfaces:** encounter factory, typed answer, support events, `LearningEvidence@1`.
- **Deliverables:** original generators/validators for quantity, sharing, pattern/sort, spatial/measurement, phonics/word-building, and story/message activities; reviewed hints and solution fixtures.
- **Acceptance:** generated tasks are solvable and match node bounds; wrong equal-total/unequal-group answers rejected; reading support correctly classified; subjective responses never auto-certified.
- **Tests:** exhaustive small numeric domains, invalid inputs, language answer alternatives, inaccessible representation cases and content-answer consistency.

### WW-07 — Mastery and scheduler

- **Depends:** WW-03, WW-06. **Scope:** `backend/crates/learning/mastery/` and scheduler.
- **Interfaces:** evidence stream → skill state; frontier → encounter request; explain-selection response.
- **Deliverables:** section 5 policy, versioned thresholds, replay/recompute command and simulated learner fixtures.
- **Acceptance:** assisted/repeated fixed answers cannot establish mastery; secure nodes leave mandatory practice; all-secure and cold-start states have useful outcomes.
- **Tests:** guessing, repeated items, abandoned sessions, support, subject switch, prerequisite gaps, corrupted evidence and transfer-item cases; educator calibration report.

### WW-08 — Persistence, parent identity and privacy lifecycle

- **Depends:** WW-02. **Scope:** `backend/crates/api/identity/`, storage adapters, `backend/migrations/`, `docs/privacy/`.
- **Interfaces:** profile repository, auth context, consent/retention records, export/delete job interface.
- **Deliverables:** parent-managed profiles, migrations, ownership enforcement, deletion/export flows, threat/data-flow model and vendor requirements.
- **Acceptance:** two parents cannot read/write each other's data; no child account email; deletion covers events, caches, exports and backup restoration policy.
- **Tests:** authorisation matrix, CSRF/session/logout, migrations, export completeness, deletion/revocation and synthetic restore with tombstones.

### WW-09 — Rust API and transactional integration

- **Depends:** WW-05, WW-06, WW-07, WW-08. **Scope:** `backend/crates/api/game/`, outbox and service wiring.
- **Interfaces:** `/api/v1`, command IDs/revisions, domain/learning repositories.
- **Deliverables:** authenticated endpoints, atomic state/evidence/rewards, error contracts and bounded queues.
- **Acceptance:** stale commands conflict safely; retried requests have one effect; server validates hints and answer outcomes; authored game works without AI.
- **Tests:** transaction rollback, concurrency/double-grant races, malicious body IDs, cross-profile encounter access, stale revisions and API contract suite.

### WW-10 — Frontend state adapter and save migration

- **Depends:** WW-02, WW-09. **Scope:** `frontend/src/api/`, migration module; integration owner wires `main.ts`/`progress.ts`.
- **Interfaces:** generated client, snapshot/events, save-v1 import receipt.
- **Deliverables:** guest/cloud mode selection, recoverable save migration, reconnect handling, server error UX and account/profile switching.
- **Acceptance:** existing guest save survives; import cannot create verified mastery; double-click/reconnect cannot duplicate rewards; child sees clear offline state.
- **Tests:** old/corrupt saves, storage disabled, duplicate imports, interrupted writes, logout on shared device and revision-conflict browser tests.

### WW-11 — Accessible maths and English activity UI

- **Depends:** WW-02, WW-06. **Scope:** `frontend/src/learning/`, feature DOM components; entrypoint integration reserved to owner.
- **Interfaces:** `EncounterInstance@1`, attempt/hint client adapter, keyboard equivalents.
- **Deliverables:** six activity-family renderers, manipulatives, word tiles, narration controls, hints and supported completion.
- **Acceptance:** touch/keyboard alternatives for every drag; meaningful focus/labels; no colour-only instructions; audio support does not falsely score reading mastery.
- **Tests:** keyboard-only flows, screen-reader checks, tablet targets, reduced motion, alternate answers, image/font/audio failure and deterministic fixture rendering.

### WW-12 — Regions, companions and story presentation

- **Depends:** WW-04, WW-05, WW-10, WW-11. **Scope:** frontend world feature modules and region data; coordinate edits to existing world/navigation files.
- **Interfaces:** authoritative story flags, creature roster, map geometry and content manifests.
- **Deliverables:** Orchard/Cove integration, journal collection, team abilities, transformations, world consequences and ending.
- **Acceptance:** preserve Meadow navigation and seven existing avatar IDs; all mandatory encounters reachable; assistance and hero choice do not bypass locks or erase progress.
- **Tests:** bridge/collision regressions, all heroes/regions, reload mid-quest, supported finale, portrait/landscape and collection-completion playthrough.

### WW-13 — Authored director and proposal validator

- **Depends:** WW-04, WW-06, WW-07, WW-09. **Scope:** `backend/crates/director/core/`.
- **Interfaces:** `DirectorInput@1`, `DirectorProposal@1`, deterministic encounter builder.
- **Deliverables:** safe context builder, seed-based authored planner, full validation pipeline, fallback table and replay records.
- **Acceptance:** every valid frontier/story state has a playable fallback; rejected proposals have no side effects; stale proposals cannot commit.
- **Tests:** malformed/oversized JSON, invented IDs, prompt injection, unsafe prose, arithmetic bounds, revision races and hundreds of simulated complete journeys.

### WW-14 — Live model adapter and operations controls

- **Depends:** WW-08, WW-13. **Scope:** `backend/crates/director/providers/`, provider policy ADR.
- **Interfaces:** provider-neutral async inference trait, policy/version configuration and telemetry.
- **Deliverables:** reviewed provider adapter, structured output, timeout/circuit breaker/cost caps, sanitised telemetry and AI kill switch.
- **Acceptance:** browser has no model key; provider sees only approved minimal context; outages preserve gameplay; unreviewed output never displays.
- **Tests:** mocked provider timeouts/429/invalid outputs, retry budget, payload privacy snapshots, disabled-AI end-to-end and spend-cap enforcement.

### WW-15 — Training corpus and evaluation harness

- **Depends:** WW-03, WW-04, WW-06, WW-13. **Scope:** `ml/dataset/`, `ml/eval/`.
- **Interfaces:** `DirectorTrainingRecord@1`, frozen proposal contracts and Rust validator CLI.
- **Deliverables:** versioned synthetic corpus, licences/provenance, review UI/export, held-out split manifests, adversarial suite and baseline scorecards.
- **Acceptance:** no production child records; no scenario-family leakage; evaluators call actual Rust validation; reproducible rubric and reviewer disagreement resolution.
- **Tests:** schema/hash/duplicate/split checks, planted leakage detection, safety fixtures and deterministic score aggregation.

### WW-16 — Fine-tuning experiment and model promotion

- **Depends:** WW-14, WW-15. **Scope:** `ml/training/`, model cards and promotion manifests.
- **Interfaces:** provider training adapter, dataset hash, evaluation gate report and model registry version.
- **Deliverables:** reproducible training configuration/run, cost record, comparison with baselines, go/no-go decision and rollback configuration.
- **Acceptance:** promotion only on section 7 gates; otherwise ship baseline with documented findings. Training execution requires an approved provider/budget before incurring cost.
- **Tests:** frozen holdout, unseen story families, adversarial regression, version mismatch, model rollback and reproducibility checks.

### WW-17 — Parent learning view and content expansion workflow

- **Depends:** WW-03, WW-07, WW-10, WW-11. **Scope:** `frontend/src/ui/grownups/`, content validation CLI and review documentation.
- **Interfaces:** learner summaries, selection explanations, export/delete APIs and content manifest.
- **Deliverables:** honest progress summaries, support history, subject preferences, reviewed pack import and coverage dashboard.
- **Acceptance:** legacy data labelled unverified; mastery limitations visible; new packs cannot activate with missing assets/prerequisites or unsigned review status.
- **Tests:** zero/partial/secure evidence, revoked parent session, accessibility, coverage exclusions and invalid pack rollback.

### WW-18 — Deployment, observability and recovery

- **Depends:** WW-08, WW-09, WW-13. **Scope:** `infra/`, `docs/runbooks/`, CI integration.
- **Interfaces:** container health/readiness, migrations, version manifests, redacted metrics and feature flags.
- **Deliverables:** isolated environments, reproducible builds, secret management, backup/restore, rollout/rollback and incident playbooks.
- **Acceptance:** no production data in staging; frontend works when AI worker is off; restoration meets approved RPO/RTO; outbound services documented.
- **Tests:** clean staging deployment, migration compatibility, restore/deletion tombstone drill, provider outage, DB unavailability and secret/log scans.

### WW-19 — Integrated QA, security review and pilot

- **Depends:** WW-12, WW-17, WW-18; WW-14 only for an AI-enabled release. **Scope:** `qa/`, release evidence and research protocol; production feature changes go back to owners.
- **Interfaces:** complete browser/API flows, privacy data map, curriculum review ledger and release checklist.
- **Deliverables:** cross-browser/accessibility matrix, bounded load tests, security findings with evidence, consented parent/child pilot and prioritised fixes.
- **Acceptance:** no unresolved critical/high security findings; complete offline/authored path; educator approval of released material; child testing supports understandable navigation and activities. No automatic claim of learning efficacy.
- **Tests:** current desktop Safari/Chrome/Firefox and iPad Safari; two-parent isolation, replay/tamper, all story routes, forced AI failures, focus/narration, export/delete and representative 100-concurrent-session staging load.

### WW-20 — Foundation coverage and Years 1–2 expansion

- **Depends:** WW-03, WW-06, WW-07, WW-17, WW-19. **Scope:** new versioned curriculum/activity/story packs, coverage audit and calibration reports.
- **Interfaces:** existing pack/node/evidence contracts; versioned migration only where semantics change.
- **Deliverables:** close Foundation coverage gaps first; then separate Year 1 and Year 2 packs with reviewed prerequisite bridges, original tasks and new story remixes.
- **Acceptance:** no complete-coverage claim until audited ledger is complete; outcomes requiring adult observation explicitly separated; new bands validated beyond simply increasing quantities.
- **Tests:** full pack validator, prerequisite/migration tests, unseen transfer tasks, representation coverage and educator/learner pilot for each band.

## 10. Current delivery order and integration gates

The original foundation-building waves are superseded by the implemented Rust/API/Meadow baseline. Do not restart WW-01–11 from scratch. Retain their outstanding acceptance requirements in the implementation status ledger.

| Delivery | Work | Gate |
| --- | --- | --- |
| Pip progression loop | Minimum companion plan phases 1–4 | Recruit, train, level up, see reward and reload without duplicate grants |
| Complete Meadow progression | Finish phases 1–4 for four friends | Reachable milestones, collection UI, compatible saves and sustained-play storage strategy |
| Team and friendship gameplay | Phase 5 | Two-slot team, optional roles, friendship and reversible appearances |
| Full authored journey | Phase 6 and remaining WW-12 integration | Three regions, twelve friends and supported ending |
| Public release readiness | Remaining WW-08–10, WW-17–19 requirements | Identity/lifecycle flows, reviewed content, accessibility, security and operational evidence |
| Optional model experiments | WW-14–16, separately scheduled | Provider/privacy/budget decisions, fallback and evaluation evidence; never an authored-release dependency |
| Curriculum expansion | WW-20 | Audited Foundation coverage before completeness claims; reviewed extension packs |

Every implementation gate includes compatible contracts and saves, relevant unit/integration tests, asset/content validation and browser journeys. Public release additionally requires reviewed content, privacy/security findings disposition, rollback rehearsal and an operational owner. Passing a build does not establish educational or accessibility quality.

## 11. Validation commands and release evidence

Tooling portability: follow `AGENTS.md` for the actual host. On macOS, pass the installed Cargo/rustup homes to `scripts/check.sh`; `scripts/check-postgres.sh` currently requires the prepared Linux environment or a deliberate portability update.

Current repository checks are `bash scripts/check.sh`; real PostgreSQL validation is `sh scripts/check-postgres.sh`. Contract drift is checked with `python3 scripts/generate-contracts.py --check`. Browser journeys use the isolated QA database and setup documented in the README. Run targeted companion tests as each phase lands, then the relevant full checks. Preserve locked dependencies unless a change requires updating them.

Maintain release evidence keyed by app, schema, rules, content, mastery-policy and model versions. Include test counts/results, rejected-case examples, known limits, educator approvals, privacy review date and rollback target. Existing README test counts are historical descriptions and must not be carried forward as new verification.

## 12. Decisions to resolve during implementation

Proceed with the defaults in this plan while owners resolve these at their specified gates:

- Product/education owner before companion content activation: confirm the skill slice, milestone pacing and accessibility priorities.
- Engineering owner before sustained training: define aggregate storage bounds, receipt retention and migration/rollback compatibility; retain existing Git, npm and pinned tooling.
- Education owner before released content activation: approve prerequisites, precise ACARA descriptor mappings and initial mastery thresholds.
- Privacy/legal owner before cloud pilot: launch countries, applicable obligations, consent approach, vendors and retention schedule.
- Operations owner before live AI: model provider, hosting region, per-session cost ceiling and training budget.
- Research owner before child sessions: parental consent/child assent, observation protocol, stopping conditions and data minimisation.

Success means a coherent, kind and replayable game with demonstrably correct activities, transparent adaptation, reliable child-data controls and measured user understanding. It does not require fine-tuning to outperform the baseline, nor does completing an implementation task prove educational efficacy.
