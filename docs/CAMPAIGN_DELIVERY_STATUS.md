# Campaign delivery acceptance matrix

14 November 2026. This records the development implementation against
[CAMPAIGN_BUILD_PLAN.md](CAMPAIGN_BUILD_PLAN.md). It is not release approval.
Executable campaign coverage now extends through the ending; that does not make
every production, accessibility, animation or human-playtesting acceptance criterion
complete. The integration owner's final run results belong in
[IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md).

“Implemented” below means a concrete content definition and code path exist.
“Coverage present” means a test has assertions for that behavior; it does not claim
that its current run passed. In particular, ordinary `cargo test` does not run the
ignored PostgreSQL campaign integration test. Automated solvers establish a possible
route, not interesting balance or an audience's understanding.

## Milestone acceptance

| Milestone | Concrete implementation | Evidence and remaining acceptance work |
| --- | --- | --- |
| **M0 — executable story/content contract** | `content/campaign/TREATMENT.md` authors all 26 main-story beats. `runtime.v1.json` defines 267 actions, sixteen locations, eight keeper trials, three starter practice boards, three Rowan contests and eight post-game boards. Actions have typed predicates, costs, rewards, grant IDs, dialogue, learning joins, puzzle operations and field methods. All twelve friends have four distinct growth contexts. Vell's continued orders, local bypasses versus the central reserve, eight grounded lines, lens recovery before Illuminate, and Moss/Reed's identity are explicit. v2 remains unchanged design history. | `check-runtime.mjs` and `runtime.v1.schema.json` check references, all 26 missions, all four growth stages, free field alternatives, supply sanity and twenty-three negative mutations. Rust content loading and transition tests interpret the same JSON. The asset manifest records source/frame, dimensions, footprint, hashes and provenance. Automated validation is not narrative/educator/art approval; those reviews remain pending. |
| **M1 — saved foundation** | `domain/src/campaign/mod.rs` owns pure state and commands. `content/src/lib.rs` loads the authored definition. `api/src/campaign/` authenticates through existing identity, applies profile transactions and expected revisions, stores command receipts, and joins canonical companion grants. `0008_campaign.sql` adds profile-owned campaign checkpoint/receipt tables with forced RLS, cascade deletion and export inclusion. Content 1.0 remains immutable and resumable while new adoptions use 1.1. Adoption preserves legacy stories and defers incompatible pending work. The current entry offers campaign adoption while retaining the legacy path. | `docs/adr/campaign-authority.md` records aggregate and lock boundaries. The ignored `postgres_campaign_adoption_retry_race_canonical_learning_and_lifecycle` test covers actual persistence, ownership, retries/races, canonical grants, rollback and lifecycle. `scripts/check-postgres.sh` must execute it on a disposable database; compilation or an ordinary Rust test run is insufficient. The aggregate and full disposable PostgreSQL suite passed; see the integration evidence record. |
| **M2 — welcome, Bridge and Bloom** | Three real starter practices precede the departure; bridge observations, three reserved branches → three planks → crossing, first-help learning, two trials, seals, two travelling slots, free recovery, post-Bloom invitation, team changes and return alcove are executable. Irrigation has reversible destinations, incorrect routing, check/reset and saved state; either garden design persists. Turns use per-friend energy, hazard/phase previews, pressure, signatures, universal moves, keeper charges, items, partner bonuses, guided play and retreat. The island stays inaccessible until Ferry. | Domain tests cover all starters on opening boards, real full-campaign transitions, wrong irrigation/reset/reload and both designs, partner bonus timing, rejected learning/resource actions and replay rewards. `qa/e2e/campaign-journey.mjs` exercises the actual UI; the runner uses an isolated synthetic database. All three starters passed desktop/phone opening browser checks, including exact retry, touch, reload and recovery; the integration record lists the evidence boundary. All twelve base and twelve form identities have native directional/stride, signature, cheer and rest poses. The minimal cycles have technical renderer coverage; human playtest/art acceptance remains pending. |
| **M3 — Meadow and companion loop** | Echo has a visible three-slot reversible chime puzzle and restored warning; Flutter and all remaining starter invitations are executable. Echo licenses Resonate and opens the Orchard gate through a matching companion or free tuning fork. All twelve growth pipelines define ordered, once-only recipient milestones; camp supports level-three specialization and persistent appearance selection through canonical projection. | Coverage exists for contextual milestone order, supported joins, declined invitations, specialization/reload and full journeys. The separate public `field-guide.v1.json` covers all twelve habitats/clues/preferences. Each unowned friend has an observation plus wait/help/play approaches; the stated preference opens the existing contextual help project. A mismatch saves another clue, spends nothing and keeps the right option open. Canonical ownership hides redundant offers. Immediate deterministic eligibility satisfies the maximum-three-eligible-visits availability bound without RNG or a rerollable sequence. The field guide and complete collection have desktop/phone browser coverage. Native form poses, confirmed earned-form previews and reversible appearance transitions are implemented; audience acceptance remains pending. |
| **M4 — Orchard and resources** | Harvest and Current unlock after Echo and can run in either order; Cove requires both. Branching ridge/canal gear recovery, phase-changing mill trial, Rowan/Moss rescue and responses, reversible three-slot valve routing, four Orchard invitations, Ferry/raft opening of the first island, blueprint, fossils and copied-clue exchanges are finite actions. Three consumable recipes, a three-item capacity and carried energy give resource choices; free camps and loan tools permit recovery. All Meadow/Orchard personal growth contexts are authored. | `complete_campaign_for_every_starter_with_saved_replay` asserts the selected Orchard trial occurs first in each order, then reaches the ending and all recruits. `qa/e2e/campaign-longplay.mjs` also asserts actual Orchard order through the API. Source quantities and one-time exchange predicates are checked; full DB/browser execution results remain the integration owner's evidence. Signature is available from level one and Rally from level two, matching v2’s two-technique progression; focus/guard/rest remain universal. There are no additional equip candidates in v2 that require a separate equipment picker. Routes use saved gates and reused world backgrounds rather than fully bespoke regional layouts. |
| **M5 — Cove, crisis and ending** | Tide establishes evidence that Glimmer rescued creatures; Beacon recovers/fits the physical lens before Illuminate; the ascent has a matching-companion/free-lantern alternative. Accord requires seven restored local lines and a central eighth line, then runs craft/rhythm/gleam phases before the shared release. The constellation ending, changed collectors, sanctuary/Glimmer friendship, all twelve personal milestones and eight post-game trial variants are authored. Eight named beacon locations have persistent restored-state flags. | Full journey tests and the API long-play runner assert eight seals, all twelve recruits, all 48 milestones and the ending; the API runner additionally asserts canonical level five and restart/reload. Those assertions are stronger than content closure but still require recorded passing runs. All 24 identities have native development motion sets; human art review and higher-frame-count polish remain pending. The domain now records the Gleam signature’s one-turn extra hazard reveal; the final renderer and browser checks must establish its visible presentation. Late-game collection goals and confirmed transformation/celebration feedback are implemented. Final audience and production-art acceptance remain open. M5 is an executable development ending, not the plan's full final-art/no-placeholder release gate. |
| **M6 — audience playtest, polish and release evidence** | Dialogue is paginated, keyboard-operable HTML accompanies the Phaser world, decisions are untimed, text remains available when browser speech is unavailable, and a browser speech control can read dialogue. Existing supported learning controls are reused. Mobile, recovery and screenshot runners exist. | No educator/narrative/art/accessibility sign-off or child/parent audience playtest is claimed. Browser speech synthesis is not reviewed narration, and text fallback alone does not establish access for a five-year-old who cannot read. Balance, understanding of goals/unlocks, fatigue and session length need observation. Technical browser checks cover focus, target sizing, contrast, initial/live reduced motion and read-aloud failure paths. Texture sweeps prove bounded motion residency and no repaint/restart texture growth; the approximately 156 MiB initial decoded-texture estimate and 48 MiB initial art transfer still require physical-device review. Guided play now uses compact combat state to avoid late two-friend request timeouts. These checks do not establish screen-reader or physical-device approval. Automated scripts and source-image inspection do not satisfy M6. |
| **M7 — human paired expeditions** | The initial shared-device mode selects actual owned household profiles, independently owned companions and alternating navigator/keeper practice turns. Roles swap without changing learner attribution. Both exact, owned keepsake offers are visible before two-sided confirmation; reciprocal copies are saved separately and originals remain owned. Either player can continue solo. Migration 0009 stores actor-bound receipts/copies with RLS; 0010 preserves a monotonic household revision across participant deletion. | Real two-profile browser checks pass practice, ownership, role swaps, both offers, interrupted confirmation/exact retry, phone layout and solo continuation. Disposable tests cover races, rollback, ownership, export/deletion and the household clock. The shared board awards no academic evidence or companion XP. An explicit local-demo operator command provisions a fresh synthetic second profile. Network invitations, cross-household authentication, reconnectable network sessions and atomic transfers of original cross-profile assets remain the later sub-milestone. No solo reward requires paired play. |

## Mechanics and source audit

The five field abilities now have real route/action effects. Both methods require
the corresponding licensed seal. A companion method requires a matching travelling
affinity; the tool method uses a permanent free kit, so starter choice cannot strand
the player. Each pair shares its completion flag and lasting result.

| Ability | Lasting result | Free alternative |
| --- | --- | --- |
| Brace | Open the return-visit bridge supply alcove once | `brace_kit` |
| Bloom | Open Flower Garden → Echo Hollow | `seed_kit` |
| Resonate | Open Watchtower → Orchard | `tuning_fork` |
| Ferry | Open the first bridge's island crossing | `folding_raft` |
| Illuminate | Reveal Harbour Lookout → Observatory ascent | `lantern` |

Opening projects deliberately borrow equipment before their own seals are earned.
The lens is recovered from the reachable maintenance store with an ordinary lamp
and trolley; `quest_lens` is consumed by fitting it before the Beacon trial. Neither
Beacon preparation nor the trial requires the resulting Illuminate unlock.

Supply checks preserve three quest branches → three planks → three spent planks;
one recovered mill gear → one fitted gear; one lens → one fitted lens; two fossil
rubbings → one exchanged copy, leaving a spare; and one tower-clue copy → one ridge
copy. Choice branches share completion predicates to prevent collecting both
versions. The alcove grants two raw leaves, not a finished consumable that could
bypass the three-item capacity. Original clues and companion ownership are never
consumed by NPC trades. Forage supplies are bounded baseline refreshes, not reload
rewards; first completions and replay receipts must remain tested independently.

Late Glimmer recruitment occurs after the ending at the sanctuary camp. All four
milestone story gates are then open; each action still requires the recipient and
previous milestone in order. The API pins recipient/content/rules before learning
and awards ten canonical game XP under the companion/milestone key, independent
of supported versus independent academic evidence. The pure domain's milestone
flags do not themselves establish the canonical XP result: the full API runner's
level-five assertion is the relevant joined check.

## Remaining presentation and production work

The art inventory is `content/campaign/art/manifest.v1.json`; exact source prompts,
references, original generations, alpha-cleanup edits, processed copies, frames and
hashes live beside it. All twelve static base identities and twelve static
Wayfinder forms remain available. Native motion now has **24 separate base/form
atlases with 480 drawn pose frames**. Each identity has its own four directional
idle stills, two alternating stride frames per heading, three signature frames,
three cheer frames and two rest frames. These are articulated bitmap poses; bob,
flip or scale of a single source is not counted as that coverage.

The first Pip base atlas passed actual Phaser desktop/phone inspection before the
remaining sets were generated. All 24 atlases then passed the native-frame renderer
fixture, including actual timed signature sequences returning to idle and stable
reduced-motion selection. Technical visual review covered all 480 desktop poses;
all four phone groups for every set were also inspected, with supplemental starter
review recorded in `starter-motion-phone-supplement-review.v1.json`. No source-frame clipping, identity or foot-anchor
defect was observed. The central records are
`content/campaign/art/native-motion-content-review.v1.json`,
`nonstarter-base-technical-preview.v1.json`,
and `nonstarter-wayfinder-supplement-review.v1.json`; representative screenshots are
retained under `docs/screenshots/campaign/`.

The cycles are deliberately minimal: two-stride locomotion and one idle still per
heading, without a breathing/blink loop or polished four/eight-stride transitions.
Signature/cheer/rest effects and form presentation are connected through the
frontend's confirmed-state hooks. The renderer fixture is synthetic and cannot
establish canonical ownership, earned game XP, human art approval or audience
understanding. Human artwork/accessibility/release review and audience playtesting
remain pending. Active-identity lazy loading replaces an all-atlas preload;
`campaign-motion` texture residency and disposal require the separate QA cache
measurements, not an estimate from PNG file size. The measured frame contract and
processing limits are documented in `content/campaign/art/MOTION_PIPELINE.md`.

The current frontend reuses regional/exploration backgrounds and isolated props.
It presents legal actions and explicit path travel, rather than a bespoke scene
layout for every campaign location. Chime tower, tide refuge, cache variants, all eight seal designs, five affinity
icons and signature/celebration effects now have bundled assets and provenance.
Actual trial and phone-size rendering have technical coverage. Software bob/flip
is not counted as native motion; the 24 dedicated pose atlases supply that coverage.

The final field guide has desktop/phone screenshot coverage, including all twelve
owned companions. Domain coverage verifies the Gleam reveal rule; full audience
understanding of its preview remains a playtest question. Signature plus Rally
implements the two-technique set specified by v2; universal moves do not occupy
those slots, so a separate equipment picker is not counted as a missing core rule. Camp exposes play, grooming, quiet rest and remembered favourite places, with
companion-specific hints and saved feedback. Remembering a place is labelled as a
memory, not an automatic trip. Saved garden choices now render distinct layouts;
early Sail/Rowan choices and original/exchanged fossil clues have later consequences.

## Verification record boundary

The current content pass ran `node content/campaign/check-runtime.mjs` (267 actions,
22 boards, twenty-three rejected negative mutations), both historical campaign
validators, and `git diff --check`. It also inspected the actual domain, API,
frontend, migration and QA sources cited above. The content checker intentionally
overapproximates mutually exclusive branches and encounter victory when checking
reference/supply closure; it is not a runtime completion proof.

The initial integration aggregate passed the 67-test domain suite on content 1.0.
The continuation adds relay, choice-memory, paired and compact-planner regressions;
current aggregate and targeted results are recorded in the integration evidence. Its full paths cover all three
starters, both actual Orchard orders, all twelve recruits and all 48 milestone
flags, with a JSON checkpoint roundtrip each step. They do not award canonical
API XP themselves. Supply-free tactical witnesses cover every non-practice board
for every starter, including replay without repeated grants/supplies.

The initial aggregate Rust/contracts/frontend checks and disposable PostgreSQL suite
passed. All three opening UI journeys and the initial six complete campaign cases (three
starters × both Orchard orders) passed, including all twelve canonical level-five
results, all 48 milestones, API restart and browser reload. One complete case used
real supported-learning controls throughout; five used authenticated API learning
commands plus final real browser checks. Standard entry/dismissal and existing
activity browser suites also passed. See `IMPLEMENTATION_STATUS.md` for the exact
evidence boundary and `docs/screenshots/campaign/` for inspected screenshots.
The 1.1 coverage passed in two batches: four Pip/Tansy cases before the final
best-first planner change and two successful Nib reruns afterward. Each asserts the
two-affinity relay and records guided request latency; the final Nib requests peaked
below half a second with no guided retreats. The final migration-0010 paired browser
journey, native motion review and strict texture-residency checks also passed. Exact
build/batch boundaries and evidence files are in the integration record. Passing
technical checks do not close the production and human acceptance items above.
The prepared audience protocol is [CAMPAIGN_PLAYTEST.md](CAMPAIGN_PLAYTEST.md).
