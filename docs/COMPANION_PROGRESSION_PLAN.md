# Wonderwilds companion progression implementation plan

Updated: 29 October 2026. Status: first Meadow training delivery implemented and verified; richer milestones and regional integration remain planned. Test evidence is recorded in IMPLEMENTATION_STATUS.md.

## Purpose and scope

Build a satisfying loop of discovering friends, training them, unlocking visible growth, choosing a team and exploring together. Start with a complete Meadow experience, then connect Orchard and Cove. Extend the existing Rust authority and Phaser world rather than creating a second progression system in the browser.

This is the active gameplay implementation sequence. It supersedes the earlier frontend-first plan and the execution order for collection, progression and world integration in the broader [Wonderwilds roadmap](WONDERWILDS_IMPLEMENTATION_PLAN.md). That roadmap retains architecture, curriculum, privacy and release requirements. [Implementation status](IMPLEMENTATION_STATUS.md) records completed work and limitations; planned features below are not completion claims.

## Existing foundation

| System | Current implementation | Work remaining |
| --- | --- | --- |
| Collection | Twelve authored and illustrated friends, character field guide and four Meadow friendship quests connected | Progression presentation, discovery integration and remaining regions |
| Explorer progression | Rust XP and level thresholds at 0, 20, 50, 90 and 140; snapshot/journal projection | Pacing and child usability review |
| Friendship | Per-character points, distinct helping moments, maximum six; journal/API memory actions connected | Verified shared-activity participation and optional world roles |
| Transformations | Reversible cosmetic changes, requiring friendship and a regional story flag; journal/API controls connected | Distinct transformation artwork and human review |
| Teams | Two story-unlocked domain slots, typed team command, journal selection/removal and two world followers | Full two-slot regional browser journey; optional roles |
| Discovery | Domain selection and persistent offers | Connect offers, habitat clues and world eligibility |
| Character progression | Separate XP, sixteen Meadow training milestones, snapshot/journal controls and level-up reaction; real API/browser reload journey verified | Personal quests, roles, distinct higher-level rewards and pacing review |

The exploration library now contains 75 connected destinations with individual
landscape images for the 72 new locations. This is a browser exploration surface;
server regional eligibility, authored Orchard/Cove stories, recruitment and the
ending still require phase 6. Single front-pose friend artwork is available;
directional animation and separate transformation art remain outstanding.

### First delivery boundary

The first training implementation uses five character levels at 0, 10, 20, 30 and
40 XP, independently of Explorer XP. Four first-time training milestones per
Meadow friend each award 10 XP; replay is practice without additional XP. Rewards
currently provide a generic celebration using existing portraits, rather than the
proposed level-three role, level-four personal quest or level-five special art.
Those richer rewards remain planned below. Existing transformations retain their
original conditions. Pending training binds its recipient and a server policy
fingerprint; changed policy rejects assessment while abandonment stays available.

The local development API continues to use Meadow menu eligibility. The clearing
is an accessible world entry point, not a claim of server-verified coordinates.
The second team slot keeps its existing Orchard story prerequisite; its API and
rendering support do not unlock Orchard travel or grant the slot early.

Primary integration files: `backend/crates/domain/src/engine.rs`, `backend/crates/api/src/game/mod.rs`, `backend/crates/api/src/game/repository.rs`, `contracts/schemas/v1.json`, `content/creatures/friends.v1.json`, `content/story/progression.v1.json`, `frontend/src/backend-world.ts`, `frontend/src/backend-world-model.ts` and `frontend/src/world.ts`.

## Progression design

Maintain four separate axes: Explorer XP for adventure progress, character XP for individual growth, friendship for shared experiences, and learning evidence for demonstrated skills. Character level is not a measure of academic ability. Supported completion earns the same game rewards; hints remain available at every level. Wrong answers and time away never remove progress.

Use five character levels for the initial implementation. Store XP and derive the level from versioned content thresholds. Thresholds and reward amounts must be configured and checked against the number of available activities so every milestone is reachable without grinding.

| Level | Proposed milestone |
| --- | --- |
| 1 | Joins the collection and can accompany the player |
| 2 | New reaction or celebration animation |
| 3 | Character-specific optional exploration interaction |
| 4 | Personal quest becomes available |
| 5 | Special appearance eligibility, together with friendship and story conditions |

Existing glow transformations remain reversible and cosmetic. Do not remove transformations earned before the update or silently apply new level requirements to them. Version any new unlock conditions; gameplay abilities belong to character milestones rather than requiring a particular equipped appearance.

Retain the existing two-slot story unlocks for the first version. Three-character teams, battles, trading, breeding, paid randomness and rarity systems are outside this workstream. Personal quests, distinct roles and regional finales provide goals for training.

## Phase 1 Rules and content

Extend `content/story/progression.v1.json` with character XP thresholds, milestone rewards and training grant policy. Extend `content/creatures/friends.v1.json` with milestone references, training themes and personal quest references. Add authored training definitions under `content/activities/` and personal quests under `content/story/`.

Update the content loader and validators, including `content/story/validate.py` and `content/story/test_story.py`. Validate IDs, references, assets, prerequisites, achievable XP totals and unlock cycles. Pin progression rules/content versions for pending activities and replay.

Acceptance: all four Meadow characters have an achievable progression path to level five, and every reward has defined content and art requirements.

## Phase 2 State contracts and persistence

Extend the Rust domain with per-character XP and training milestone grants. Keep friendship independent. Record participating character IDs when a training encounter starts; the browser never submits an XP amount, earned level or completion claim.

Expand the snapshot with explorer progression, character progression, friendship, appearance eligibility, team capacity and available training. Add region/discovery projections when those phases are implemented. Define typed command payloads for training, team selection and transformations. Update `contracts/schemas/v1.json`, `contracts/openapi.yaml` and shared fixtures; regenerate `contracts/src/models.rs` and `contracts/generated/contracts.ts` using `scripts/generate-contracts.py` rather than editing generated files.

Extend `backend/crates/api/src/game/mod.rs` and the repository transaction path. Completion, XP, grants, evidence and events must commit atomically. A command retry returns its prior result; a second command for the same completed training milestone cannot grant another reward. Preserve revision-conflict handling and exact retries in `frontend/src/api/client.ts`.

Add compatible readers or an explicit migration for stored aggregates. Existing characters begin at zero character XP and level one, retaining ownership, friendship, team and transformations. Keep guest saves separate. Existing pending encounters must resume or receive a defined replacement; never award speculative historical XP. Document rollback compatibility before changing persisted state.

Acceptance: old saves load, new saves reload, duplicate and concurrent requests cannot duplicate rewards, invalid character ownership is rejected, and deterministic replay reproduces progression.

## Phase 3 Collection and character presentation

Expand the journal in `frontend/src/backend-world.ts`; extract collection/detail modules as needed instead of growing one controller indefinitely. Show owned characters, undiscovered silhouettes, habitat clues, personality, level, XP, friendship and the next unlock. Provide training, team and appearance controls with keyboard/touch access and clear focus restoration.

Add a compact explorer-level display and short recruitment/level-up celebrations with reduced-motion alternatives. Replace provisional hero substitutions in `frontend/src/backend-world-model.ts` with distinct friend portraits and sprites, using the existing asset manifest and tests. Do not expose developer revision or schema details as game rewards.

Acceptance: all four Meadow friends are recognisable, progress is understandable, and character controls work on desktop and mobile after refresh.

## Phase 4 Training loop

Add a Meadow training clearing. Reuse `frontend/src/learning/activity.ts`, its accessible renderers and the Rust assessment adapters. Themes: Pip uses lanterns and numbers; Tansy planting and grouping; Nib sorting and sharing; Flutter sequences and patterns. Themes change presentation while learner needs determine supported difficulty.

Training is a separate encounter flow from one-time story quests. Each server-issued instance binds a character, milestone, task variant and rules/content version. Verified supported or independent completion awards the configured character XP once. Incorrect attempts permit retry; abandonment awards no completion XP. Switching the active team during training does not change the bound recipient.

Provide fresh task variants for authored training milestones. Replaying a rewarded milestone remains available without further XP; new milestones unlock through character and story progress. Do not award XP for time elapsed, button presses or repeatedly reopening a task. Practice evidence remains labelled appropriately and does not manufacture novel-transfer evidence.

Sustained play requires bounded working storage and durable receipt/history
records, rather than simply removing the old 2,000-command guard. Preserve
duplicate prevention, pending encounters and exact retries across checkpoint
boundaries, and verify a long-running profile beyond that old cap.

The follow-up [durable storage implementation](adr/durable-game-storage.md) now
externalizes retry/history records and bounds the current PostgreSQL save. Its
rate policy replaces the lifetime cap on that path, while standalone aggregates
retain their guard. See implementation status for long-profile validation;
production storage budgets, retention and restore drills remain outstanding.

Acceptance: training raises a character to level two, displays its reward, survives reload, resumes an interrupted encounter, and cannot be farmed through duplicate requests or recipient swapping.

## Phase 5 Teams friendship and exploration roles

Expose the existing two-slot domain team through the API and collection UI. Update `frontend/src/world.ts` to render both followers with sensible spacing and motion, preserving movement/collision behaviour and reduced-motion settings. Allow selection changes and removal within server-owned capacity.

Add optional interactions: Pip lights hidden markings, Tansy blooms a dormant flower, Nib organises a treasure cache and Flutter reveals a patterned trail. Use roles to reward team choices with cosmetic discoveries or clues. Essential routes retain accessible alternatives; companions never answer an assessed question on behalf of the learner without recording support.

Connect distinct helping moments to friendship automatically from verified shared activities. Record which friends participated at completion; do not allow repeated claims for arbitrary historical quests. Connect the existing transformation action to appearance controls, retaining old unlocks.

Acceptance: changing companions makes a visible difference, two followers render correctly, friendship grants cannot repeat, and appearances persist without blocking story progress.

## Phase 6 Discovery and remaining regions

Replace Meadow-only world/API assumptions with validated regional progression. Extend quest locations and map bindings in `frontend/src/backend-world-model.ts` and server eligibility in `backend/crates/api/src/game/mod.rs`. Validate region, story prerequisites and the chosen interaction/path model on the server rather than trusting browser coordinates.

Connect the existing discovery offers and habitat clues. Persist declined offers across reloads, preserve the guaranteed first Pip encounter and bounded access to missing regional friends. Avoid rerolling rewards through refresh.

Connect Orchard first, then Cove, their eight remaining friendships, personal training paths and regional finales. Supply distinct regional character and transformation assets and validate content references before activation. Complete the full authored ending with support available.

Acceptance: a player can visit all three regions, collect all twelve friends, return to earlier areas and resume the same progression after reload.

## Delivery order and verification

Tooling portability: follow `AGENTS.md` for the actual host. On macOS, pass the installed Cargo/rustup homes to `scripts/check.sh`; `scripts/check-postgres.sh` currently requires the prepared Linux environment or a deliberate portability update.

First delivery: befriend Pip → select Pip → complete training → reach level two → see the new reaction → reload with progress intact. Implement the minimum portions of phases 1–4 for this loop, then complete those phases for all Meadow friends. Follow with phase 5 and finally phase 6. Do not postpone testing the loop until every character and region is finished.

For each delivery, cover Rust rules and replay, schema compatibility, old-save loading, HTTP ownership and idempotency, transaction rollback, frontend state reconciliation and browser interaction. Extend `qa/e2e/backend-demo.mjs` or focused companion journeys for training, abandonment/resume, collection, teams, transformations and reloads. Preserve guest regressions and verify mobile/keyboard operation.

Use `bash scripts/check.sh` for repository checks and `sh scripts/check-postgres.sh` for real database checks. Run browser tests against the isolated QA database using the existing README procedure, never the user's demo progress. Record actual results and remaining limitations in `IMPLEMENTATION_STATUS.md`; no new test pass is implied by this plan.

The completion gate is a playable and persistent companion loop with meaningful character choices. Public release continues to require the broader roadmap's identity, content review, accessibility, privacy and operational work. Live AI and model training are not dependencies of this expansion.
