# ADR WW-01: preserve the guest prototype and establish a migration baseline

Date: 28 October 2026. Status: accepted engineering baseline; human release reviews pending.
Implementation owner: WW-01 baseline agent. Integration reviewer: root agent (review pending).
Governing plan: `docs/WONDERWILDS_IMPLEMENTATION_PLAN.md` version 1.0.

## Repository and preservation boundary

The actual cloud checkout is `/workspace/wonderwilds`, an existing Git repository
at `9e484b43eb349b9aa78342e781a3cb2f2236d524`. The planning document's earlier
non-Git `/Users/...` observation does not describe this checkout. No applicable
`AGENTS.md` was found during inspection. Initial `git status --short` was empty.
This task adds documentation and synthetic fixtures only; no game, dependency,
lockfile, art, save key or existing user files are changed. Reuse this isolated
checkout; creating another worktree is unnecessary.

The baseline is a Phaser 3 / strict TypeScript / Vite frontend. There is no Rust
backend, account system, server authority, production data store, live AI adapter
or training pipeline at this commit. Those capabilities must be implemented and
validated in subsequent plan tasks; browser observations cannot certify mastery.

## Existing adventure and interfaces

`frontend/src/progress.ts` owns `freshState`, `restoreProgress`, `readProgress`,
`writeProgress`, `completeEncounter`, `validEvidence` and `skillSummary`.
`frontend/src/content.ts` owns authored encounters, deterministic assessment and
curriculum labels. `types.ts` contains the current browser contracts, including a
Phaser actor type; future Rust domain contracts must not depend on that type.
Movement, geometry, facing and locomotion remain separate modules. `main.ts`
coordinates DOM activities, journal, grown-up view and storage; `world.ts` renders
the map. Existing saves, hero IDs and these exports remain intact.

The active reward/evidence encounter set is Meadow's bridge, picnic and garden.
Bridge: total 10 with 6 given (answer 4); picnic: 6 berries between 2 plates
(answer `[3,3]`, rejecting `[2,4]`); flower sequence answer 5. All three rewards
recruit Pip. Existing Orchard/Cove maps and movement transitions are present;
this does not establish complete learning/story chapters in those regions.

The save key is `starwhisker.meadow.v1`. Saved `Progress` has version, region,
hero, companion name, unique completed encounter IDs, up to 200 practice records,
and position. Hero IDs are `rose`, `sky`, `gold`, `poppy`, `wren`, `clover`,
`mochi`; Pip is the companion. Evidence keeps encounter/skill ID, `meadow-v1`
variant, representation, attempt/hint counts, assisted/supported flags and time.
Story rewards are unique while replay evidence can accumulate. Hints or assisted
completion always mark evidence supported. Attempt count alone does not mark a
record supported and does not imply first-check success.

Restore sanitizes known save-v1 fields, filters contradictory evidence and
unknown rewards, deduplicates rewards, limits name length, and falls back from
invalid/blocked positions. Unknown versions and malformed JSON return a fresh
in-memory state. Unavailable storage returns `available: false`; failed writes
return false. This is existing recovery behavior, not the future migration UX.

Migration mapping is `missing-10 → M06`, `share-6 → M08`, `count-1 → M05`.
Preserve original IDs, source version, original downloadable save bytes and a
one-time import receipt. Import story/cosmetics without converting editable
browser evidence into server-verified skill security. Fixtures are synthetic;
no real learner saves were available or captured.

## Dependency decision

Use npm and `frontend/package-lock.json` as the canonical installation path,
with `npm ci` for reproducibility. This matches the README and current npm
scripts. npm lockfile v3 and pnpm lockfile v9 resolve the same seven direct
packages: Phaser 3.90.0, @types/node 26.6.5, Oxfmt 0.75.0, Oxlint 1.90.0,
tsx 4.23.15, TypeScript 7.0.2 and Vite 7.3.7. This comparison does not assert
transitive graph equivalence. `pnpm-workspace.yaml` also contains an unresolved
`esbuild` build-approval placeholder. Keep both legacy pnpm files unchanged in
this baseline; do not alternate installers or regenerate either lockfile.
The integration owner can remove unused pnpm configuration in a separate scoped
change after review.

Observed cloud tools are Node 24.19.0 and npm 11.9.0. Node satisfies the package
engine `^20.19.0 || >=22.12.0`. Future reproducible configuration should pin the
chosen Node line and the Rust toolchain selected in WW-02. No Rust version is
invented by this baseline.

## Assets and provenance

`qa/fixtures/legacy/asset-inventory.json` records path, SHA-256, byte size,
PNG dimensions and source document for all ten existing PNG assets. Source
prompts identify imagegen generation for:

- Meadow background: `frontend/docs/GENERATED_MAP.md`.
- Orchard and Cove backgrounds: `docs/MAP_ART_PROMPTS.md`.
- Front hero and prop atlases: `frontend/docs/CONSISTENT_ART.md`.
- Side/rear hero atlases: `docs/DIRECTIONAL_POSES.md`.
- Rosie front-facing walk cycle: `frontend/docs/LOCOMOTION.md`.
- Squad illustration: `frontend/docs/VISUAL_DIRECTION.md`.
- Logo: `frontend/docs/APP_LOGO.md`.

These are repository provenance records; external generated-image directories
mentioned there were not part of the checkout. No standalone project licence
or owner approval ledger was found. Ownership/licence and art review remain
pending rather than inferred from prompts. Existing atlas tests check frame
bounds/coverage, not legal provenance. Rosie has actual walk frames only when
facing down; other heroes/directions use procedural gait fallbacks. Historical
visual-direction docs describe replaced SVG rendering and must not be treated
as the current rendering contract.

Optional Google Fonts requests and browser speech are existing external/browser
behaviors. Local fallback fonts work without them. Privacy/accessibility review
and any self-hosting/narration replacement remain later scoped work.

## Verification and limitations

Current-instance evidence from 28 October 2026:

| Check                                                                                                     | Outcome                                                                                                        |
| --------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------- |
| `cd frontend && npm ci --no-audit --no-fund`                                                              | Passed, 26 packages (integration owner report)                                                                 |
| `cd frontend && npm run check`                                                                            | Typecheck, lint, formatting, tests and production build passed (integration owner report)                      |
| Existing test runner                                                                                      | 7 test files passed; 0 failed/skipped as reported by runner; historical README assertion counts are not reused |
| Production bundle                                                                                         | Passed; expected approximately 1.25 MB chunk-size warning                                                      |
| `node --import ./frontend/node_modules/tsx/dist/loader.mjs qa/fixtures/legacy/baseline.test.ts` from root | 4 tests passed, 0 failed/skipped                                                                               |
| Desktop/tablet browser smoke                                                                              | Not run: browser automation tools unavailable; historical review documents are not fresh evidence              |
| Educator, accessibility and child pilot review                                                            | Not performed; not implied by automated checks                                                                 |

The four fixture tests exercise valid save round trips, corrupt-field/malformed
JSON recovery, injected blocked/full storage and all asset hashes. Calling the
fixture test via bare `--import tsx` from frontend failed because the external
fixture path is outside that package's module-resolution scope; the documented
explicit loader command passes. An initial test fixture incorrectly called the
repaired bridge point blocked; moving that synthetic position into the
clubhouse corrected the fixture expectation without changing game code.

## Handoff and rollback

Added files: this ADR and `qa/fixtures/legacy/{README.md,baseline.test.ts,
fresh-v1.json,completed-v1.json,invalid-v1.json,malformed-save.txt,
storage-blocked.json,asset-inventory.json}`. Contract version remains legacy
save 1; no new shared contract is defined here. No runtime migration occurs.
Removing these newly added artifacts rolls back WW-01 without touching saves.
WW-02 can proceed with this captured engineering baseline; broader cross-browser/device QA,
licence approvals and human education/privacy reviews remain open gates.
