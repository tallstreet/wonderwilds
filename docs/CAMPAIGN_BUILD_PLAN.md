# Missing Moonlight build plan

14 November 2026 · Status: planned, not implemented.

This is the active delivery plan for the authored companion adventure. It replaces
the older curriculum-first and companion-menu delivery order, while preserving
those systems. Product scope is the [v2 campaign](../content/campaign/README.md) and
its [static JSON](../content/campaign/missing-moonlight.v2.json): three regions,
eight trials, twelve companions, 26 main missions and the interlocking mechanics.
The user has requested implementation and asset creation; this document sequences
that work. No new playable system or generated asset is claimed by this plan.

## Delivery principles

- Ship complete playable journeys through the actual app, with saves and learning.
- Keep core story, missions, rules and art available without model credentials.
- Use authored JSON for content; Rust interprets a finite set of supported mechanics.
- Keep the existing checkout and preserve unrelated work and persistent demo data.
- Show progress through changed locations, new options, relationships and discovery.
- Character growth and supported story completion do not claim academic mastery.
- Start with existing artwork, then create production assets for the slice being built.
- Keep educator/content/release review states honest; development delivery is not approval.

## What exists and what must be connected

| Area | Inspected foundation | Work for this campaign |
| --- | --- | --- |
| App entry | `frontend/src/main.ts` connects `story/world.ts` | Add campaign onboarding and confirmed campaign HUD/state at this entry |
| World presentation | `frontend/src/world.ts`, story world/model, existing route and prop rendering | Bind campaign locations, gates, two companions and changed-world props |
| Story authority | `backend/crates/domain/src/story/`, API `story/` | Versioned campaign state machine and typed mechanics alongside compatible legacy readers |
| Companion rules | Domain engine, API `game/`, `frontend/src/companions.ts`, training UI | Reuse ownership/XP/team/appearance rules through the story campaign; avoid a new browser progression engine |
| Learning joins | API `mission_learning/repository.rs` and story inline actions | Bind campaign step and companion recipient; atomically commit learning, campaign and rewards |
| Persistence | Durable receipts/checkpoints, profile transactions, export/deletion | Persist campaign adoption, turns, discovery, grants and assets/content versions |
| Content | v2 design JSON and offline checker | Replace prose mechanics with executable typed definitions and full dialogue |
| Art | Three region maps, three regional character sheets, hero/prop atlases, exploration backgrounds | Audit usable frames; add missing views, poses, actors, landmarks and state variants |
| UI | Accessible learning controls, missions, bag and old companion components | Cohesive collection, care, team, trials, maps and dialogue in the current story app |

An old component's existence is not evidence that it works in the current entry
point. The older story asset manifest marks several friends missing while regional
character sheets exist; inspect actual frames before generating replacements.
The 72 exploration backgrounds are not 72 server-authorised campaign locations.

## Architecture to establish first

Keep campaign rules in a pure Rust module (proposed
`backend/crates/domain/src/campaign/`). Load immutable typed campaign definitions
through the content crate. Extend API story orchestration to run campaign commands;
retain existing story adapters for already-saved stories. Do not extend the old
fixed mission enum with dozens of unrelated special cases or create a second
JavaScript game engine.

The persisted campaign owns mission/step status, seals, location unlocks, puzzle
state, encounters, expedition energy, pouch, discovery offers and grant references.
Existing companion ownership/XP remains the canonical companion record, exposed
through a campaign projection. Learning evidence remains in its existing stream.
Document the precise aggregate boundaries and locks in an ADR before persistence
changes. Use one authenticated profile transaction for every joined completion.

New command families cover adoption/start, starter choice, travel, quest actions,
recruitment, team/loadout changes, care/camp, training, encounter start/turn/recovery
and exchange. Resolve exact names during contract work. All writes require existing
identity, ownership, Origin/CSRF, expected revision and idempotency controls.
Snapshots contain public state and legal actions, never answer keys or private tasks.

Update `contracts/schemas/v1.json`, OpenAPI, fixtures and generated consumers
together for wire changes; do not confuse the content schema with the wire schema.
Each pending activity/encounter pins its content/rules version and recipient.
Unknown content versions fail with recoverable status, not an empty reset profile.

For existing profiles, preserve old missions and companions. Campaign adoption is
explicit and recorded once. Resolve already-owned starters/recruits without duplicate
grants. Define mappings for old training grants before awarding any new ones; do
not infer historical XP. Defer adoption while incompatible pending work exists or
support resuming it explicitly. Add numbered migrations only when needed; never
edit applied migrations. Rollback retains compatible readers for new receipts.

Core campaign content must be explicitly authored even when Modal credentials are
present. Preserve old generated saves and the extension adapter. New live generation
must not replace the campaign, rewrite accepted missions or become a startup dependency.

## Milestone 0 — executable story and content contract

**Outcome:** the first chapter is fully authored and machine-checkable; the complete
campaign has a coherent story outline and clearly identified unfinished scenes.

- Write the full main-story treatment for review, then short playable dialogue for
  welcome, Bridge and Bloom. The earlier request for a complete prose story was
  interrupted; do not treat an undelivered manuscript as approved.
- Resolve why the collectors continue, what local repairs restore before the final
  release, why the Observatory needs all eight beacons, and how the lighthouse lens
  can be recovered before the Illuminate reward. Make these understandable on screen.
- Define Rowan's distinct companion identity without removing a recruit from the
  player's catalogue. Show Vell's intentions and consequences through concrete events.
- Make mission actions, predicates, consumed inputs, grants, choices, dialogue and
  persistent prop effects explicit JSON records. No gameplay effect should depend
  on interpreting the English instruction text.
- Separate learning invitations from tactics and creative choices. Bind tasks to
  the current curriculum frontier, not fixed age or companion level.
- Add schema/reference checks, duplicate grant detection, supply conservation and
  actual domain completion witnesses. Preserve the v2 draft as design history if
  the executable format needs a new version.
- Create an asset inventory with IDs, existing source/frame, desired poses, target
  dimensions, world footprint, milestone, status and provenance. Use original art direction.

**Acceptance:** Bridge/Bloom have complete scene/action definitions, all three
starter routes and tool alternatives are reachable, and every visible action maps
to a supported mechanic. Later missing scenes are explicitly marked unfinished.

## Milestone 1 — saved campaign foundation

**Depends on:** M0 definitions for the first chapter.

Implement content loading, campaign state/replay, typed commands and snapshots,
profile-scoped persistence, adoption and canonical companion projection. Reuse the
existing current-state/receipt storage pattern. Provide development fixtures for
fresh profiles, existing companions, saved legacy stories and pending activities.
Connect the current app to the authored campaign projection without replacing the
legacy reader. Add mission tracker, dialogue, bag and compact companion status.

**Acceptance:** choose any starter, save, reload and restart the API; the same choice
and state return. Duplicate/concurrent starter commands cannot grant two starters.
Wrong-profile access is rejected. Legacy saves remain readable. Export and deletion
include new campaign data. A transaction failure creates no partial reward.

## Milestone 2 — first complete playable loop: Bridge and Bloom

**Depends on:** M1. This is the first user-facing playtest build.

Deliver the welcome and both three-mission arcs: seven main missions in total.
Include Hazel, Rowan's introduction, starter practice/choice, a moonlight clue,
gathering and crafting, the repaired crossing, the garden investigation, irrigation
puzzle, two trials, two seals and a persistent restored garden.

Implement the initial encounter rules now: per-friend energy, signatures, universal
moves, hazard preview, pressure, tool charges, win/retry and guided play. Add one
first-help training milestone for each starter, level-two rally, care/rest, one- and
two-slot teams, recruitment, field bracing/bloom and free tool alternatives.
After Bloom, recruit an eligible unchosen starter. Keep the existing v2 recruitment
order (Nib can remain offered after Echo); do not award a second friend immediately
at the Bridge just because its second slot has opened.

Provide an optional braced supply alcove on a return visit to the bridge, so the
first slice demonstrates backtracking with a new capability. Keep the opening
island visibly unreachable until ferry access later. Allow a trial practice replay
with the new pair and no repeated milestone XP. Make at least one paired strategy
visibly different from repeatedly using the starter's signature.

**Acceptance:** choose → explore → contextual learning → train to level two → trial
→ world unlock → care → recruit → change team → revisit. All three starters work
on desktop and phone, with keyboard and supported learning. Reload during a trial,
uncertain-write retry and switching teams during a bound activity cannot duplicate
or redirect rewards. The slice must still work with all provider keys absent.

## Milestone 3 — complete Meadow and the companion loop

**Depends on:** M2.

Finish Echo, Flutter's friendship, the watchtower warning and Orchard transition.
Add the remaining starter offers, chime puzzle, field guide silhouettes and clues,
persistent discovery invitations, technique choices and level-three optional
habitat interactions. Add Rowan's second encounter and meaningful responses to
previous choices. Make training context-specific for all four Meadow friends.

Complete the five-level progression pipeline using fixture-backed tests of later
milestone unlocks; do not unlock Current/Beacon rewards early in live story content.
Add first final-quality evolution art and transitions for the three starters so the
pipeline can be visually assessed before producing every companion's sheet.

**Acceptance:** three seals and four possible friends, clear next destination,
recognisable world changes and a tested collection/care/training loop. Declined
recruitment survives reload, duplicate invitations cannot grant twice, and no
starter-specific gate or academic mastery requirement blocks Orchard.

## Milestone 4 — Orchard, branching routes and resource strategy

**Depends on:** M3.

Build Harvest and Current in either order, Cleo/Bramble/Patter/Twizzle recruitment,
Rowan's rescue and changed tactics, the mill automaton, valve puzzle, camp crafting,
three-slot pouch, limited supplies and energy carried between encounters.
Implement all technique effects, partner bonuses, free-signature specialisation,
phase-changing trials and recipient-bound personal quests.

Add NPC fossil and clue exchanges, their source quantities and one-time receipts.
Open ferry travel, the early bridge island and its blueprint. Ensure the two Orchard
orders produce compatible dialogue and exactly the same final access. Complete all
four Meadow personal quests and Orchard training definitions as content, not templates
whose meaningful character text is still missing.

**Acceptance:** play both region orders; recruit all four Orchard friends; recover
from depleted energy or pouch supplies without a softlock; revisit the opening island;
learn the next story clue; reach Cove. Verify resource consumption, refresh rules,
exchange retries and replay cannot create items or rewards.

## Milestone 5 — Cove, the final crisis and a complete ending

**Depends on:** M4.

Build Tide, Beacon and Accord, the lighthouse return journey, Glimmer's apparent
role and exoneration, Luna/Pebble/Sail friendships, and the three-phase Observatory
encounter. Make the eight beacons' contributions visible in the finale. Implement
Vell's confrontation, response to evidence and changed machines after the ending.
Deliver the constellation celebration and sanctuary quest recruiting Glimmer.

Complete the remaining personal quests, all twelve level-five forms, habitat
caches, final Rowan encounter and authored post-game trial variants. Give the
collection screen finite, comprehensible remaining goals.

**Acceptance:** all three starter journeys reach the ending; all twelve companions
can be recruited and reach level five, including late recruits; all eight seals and
five field abilities work; both Orchard orders and supported paths remain valid.
Earlier world changes and choices persist after restart. No placeholder dialogue,
missing final art or unreachable advertised reward remains in the shipped campaign.

## Milestone 6 — audience playtesting, polish and release evidence

**Depends on:** M2 for early feedback; full campaign sign-off follows M5.

Test short sessions with younger and older learners through the project's approved
playtesting process. Keep the same story with supported and more independent paths.
Use short dialogue pages, portraits, concrete objectives, icons plus words, replayable
instructions and untimed decisions. Do not assume reading fluency at five.
Validate read-aloud support and its fallback explicitly; voice recording is not a
substitute for narration. Leave no essential instruction solely in audio or colour.

Observe whether players understand where to go, why an encounter changed, what a
new companion offers and what a seal unlocks. Tune turn budgets, tool strength,
resource frequency and travel length from these observations. The existing solver's
24 solo witnesses establish possible solutions, not interesting balance.

Finish mobile layout, focus restoration, screen-reader alternatives, reduced motion,
loading/error recovery, artwork memory budgets and late-game save performance.
Record actual review status for curriculum, narrative, artwork, accessibility,
identity/privacy and operations. Do not manufacture approval from automated tests.

**Acceptance:** documented findings and resolved blockers; complete authored offline
campaign; all required technical checks and honest release decisions.

## Milestone 7 — human paired expeditions

**Depends on:** M5 and verified identity/grant foundations; not required for solo ending.

Start with shared-device navigator/keeper roles, explicit player/profile selection,
role swaps and independent ownership. A shared device is not permission to attribute
one learner's answer to another. Add two-sided keepsake exchange and solo continuation.
Network invitations, authenticated paired sessions, disconnection recovery and atomic
cross-profile transfers are a later sub-milestone. No random matchmaking or free-text
chat is assumed. All rewards retain a solo/NPC route.

## Asset production plan

Follow `frontend/docs/CONSISTENT_ART.md`: rounded dimensional characters, warm light,
consistent camera and readable silhouettes. Inspect existing images before reuse
or editing. Use the image generation skill for new bitmap art when production begins;
use existing SVG/CSS systems for ordinary interface controls. Generated static assets
are bundled with the game; this does not re-enable runtime AI scene generation.

| Batch | Assets | Delivery gate |
| --- | --- | --- |
| A — first slice | Audit/reuse three starter portraits and base sprites; Hazel frame; Rowan and his companion; dim/restored bridge and garden props; collector cart; camp; two seal designs; five affinity symbols; encounter board/effects | M2 |
| B — Meadow | Flutter movement/reactions; chime tower and locks; third seal; habitat clues/caches; first three Wayfinder form sheets | M3 |
| C — Orchard | Four Orchard companions' missing views/reactions; mill/automaton/pump/raft; two seal designs; fossil/rubbing/supply icons; Rowan rescue poses; Orchard forms | M4 |
| D — Cove/finale | Four Cove companions' missing views/reactions; Vell; lighthouse lens, tide refuge, Observatory machinery, sanctuary; three seal designs; finale decorations; remaining forms | M5 |

For twelve companions, target twelve base movement sets and twelve form sets, with
portrait, idle, walk, signature, cheer and rest coverage. The audit determines what
is already usable. Do not regenerate 24 complete sets automatically. Reuse compatible
frames and explicitly label missing animations during development.

Before batch generation, agree a technical atlas layout by inspecting renderer
requirements: frame sizes, directions, anchors, padding, transparent background,
world scale and filenames. Produce one starter sheet and preview it in the actual
Phaser world before scaling the pipeline. Generate scenery as separate reusable
layers/props with before/after variants, not baked-in paths or collectible counts.

Keep raw source, generation/edit prompt and reference provenance, processed asset,
frame metadata, hash, expected dimensions and review status. Inspect alpha edges,
clipping, alignment, silhouettes at phone size and reduced-motion stills. Verify
NPCs and companion forms remain identifiable without colour. Do not publish missing
or provisional artwork as final. Keep legacy hashed packs intact; use a new campaign
asset manifest and intentional versioned changes.

## Verification per delivery

| Change | Required evidence |
| --- | --- |
| Content/rules | JSON schemas, references, grant uniqueness, graph/supply checks, Rust transitions/replay, all starter and supported completion witnesses |
| Wire/API | Shared fixtures, generated model check, ownership/CSRF, stale command and exact retry coverage |
| Persistence | Fresh disposable PostgreSQL migration/RLS/rollback/race tests; old-save and pending-state fixtures; export/deletion; never use demo progress |
| Frontend | `npm run check`, actual browser journey, keyboard/touch/mobile, focus and reload/recovery screenshots |
| Activities | Existing activity browser suite plus the joined campaign outcome and support evidence |
| Art | Manifest/frame validation and visual inspection in game, not only a contact sheet |
| Full campaign | Both Orchard orders, three starters, all recruits/growth milestones, post-game, no providers, long-play save/reload |

Use `bash scripts/check.sh` for broad integration changes and
`sh scripts/check-postgres.sh` for persistence. Follow repository browser QA setup
with a fresh database and separate ports; use `CHROMIUM_PATH` and PostgreSQL binary
overrides as needed. Record actual checks and limitations in implementation status.
Inspect the final diff and run `git diff --check` for every delivery.

## Immediate implementation queue

1. Write the main-story treatment and executable welcome/Bridge/Bloom scene pack.
2. Audit starter/region art and record the first asset manifest; do not bulk-generate.
3. Add the campaign-authority/adoption ADR and pure Rust loader/state tests.
4. Implement contracts, persistence and starter choice with old-save fixtures.
5. Deliver the complete M2 loop with targeted asset production and browser evidence.

M0 and M1 are foundations, not the first declaration that the game is fun. M2 is
the first interaction test; M5 is the first full-campaign completion gate. Every
subsequent delivery should leave a resumable, coherent playable build.
