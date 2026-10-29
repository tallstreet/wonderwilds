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
| WW-05 | Pure domain engine, unique rewards, quests, collection/team/friendship/transformations, character XP/training, compact checkpoints and durable new-command replay facts | Operational replay/recovery drills, optional roles/personal quests and authoritative world paths |
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
| WW-18 | Local reproducible install/start/migration/check helpers; GitHub Actions passed for companion PR #5 | Storage branch hosted execution, database/browser CI, deployment, observability, backup restore and rollback drills, operational owner |
| WW-19–20 | Automated foundations and synthetic checks only | Integrated QA/security review, consented pilot, educator audit and measured curriculum expansion |

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

Remaining gameplay work after that delivery: character-specific optional roles, personal quests,
distinct higher-level rewards, directional/transformation art, automatic verified
friendship participation, discovery and regional story integration, and the full
supported ending. The 2,000-command limit remains. The concrete
[storage/checkpoint design](LONG_PLAY_STORAGE_PLAN.md) is a proposal, not an
implemented scalability claim. Account/import/lifecycle/parent-view work, human
content/accessibility/security review and deployment/backup operations remain
public-release gates. No release or content approval is implied by this delivery.
The storage follow-up below supersedes that delivery's command-cap limitation.

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

## Durable storage follow-up — 29 October 2026

The PostgreSQL path now stores a compact, explicitly versioned working save and
separate durable API/domain receipts, encounters, item-novelty records and recovery
checkpoints. Ordinary requests no longer clone or rewrite full historical vectors.
At most one active encounter and eight eligible observations per skill remain in
working state. Durable item uniqueness prevents repeated practice becoming novel
after a rolling window changes. Learning policy semantics remain unchanged.

Migration `0003_game_storage.sql` adds forced-RLS tables, deletion cascades and
parent-export coverage. Legacy migration runs under the owned profile lock,
validates history and exact receipt coverage, and retains pending support and
training bindings. Current-save digests, independent revision/high-water checks,
pending-state agreement and orphan-record rejection fail closed. Tagged compact
saves omit old required history fields so pre-storage binaries reject them.

The durable path replaces the lifetime 2,000-command guard with 120 new commands
per profile per 60-second window. Exact retries return original responses before
revision/rate checks. Standalone embedded-receipt execution retains its original
guard. New domain actions record trusted replay contexts; old contexts are not
invented. `reconstruct_legacy` validates complete durable history and returns a
recovery/export candidate without overwriting current progress.

Validation completed:

- `bash scripts/check.sh`: workspace format/strict Clippy/tests, 73 shared contract
  fixtures, generated drift, content/art validators, frontend checks/build, client
  tests and legacy-save fixtures. Includes 22 passing API tests (three database
  tests ignored), 21 domain tests and 23 learning tests. Incremental mastery golden
  tests match full recomputation across support, repeated items, sessions and reloads.
- `sh scripts/check-postgres.sh`: all three real database tests, SQL ownership,
  export/deletion, repeated migrations and negative hash-drift rejection. The long
  profile reached **2,069 commands with a 6,139-byte working save and two recovery
  checkpoints**. It verifies migration/pending support, old retries, monotonic
  histories, quota bypass for retries, rate-window recovery, reconstruction,
  corrupted digests, missing progress, mismatched profile revision, orphan receipt
  rejection, ownership, export and cascaded deletion. Existing race/rollback tests
  also pass on the new repository path.
- `node qa/e2e/run-training-qa.mjs`: the real PostgreSQL/browser training journey
  passes with migration 0003, including support, reload/resume, abandonment,
  level-up, duplicate completion prevention and mobile training.
- Final strict workspace Clippy, formatting and whitespace checks passed after
  the additional database regressions. Hosted checks passed for companion PR #5;
  storage hosted execution is recorded separately after its branch is pushed.

Only disposable synthetic databases were migrated in validation. Persistent demo
data was not changed. Total durable history still grows; production budgets,
approved retention, deployed cleanup, full replay recovery and backup/restore
drills remain outstanding. See [storage decision](adr/durable-game-storage.md)
and [source review](STORAGE_REVIEW.md). This does not establish public release,
human content review or production operational readiness.
