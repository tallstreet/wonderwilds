# Character development implementation review

29 October 2026. Governing scope: `WONDERWILDS_IMPLEMENTATION_PLAN.md`, especially WW-04, WW-05 and WW-12.

The foundation is substantially implemented. The next useful character work is individual narrative development and its presentation in the existing Meadow adventure. Rebuilding collection or introducing new character IDs would discard working foundations.

## Findings

| Area | Existing evidence | Gap and implementation direction |
| --- | --- | --- |
| Original roster | `content/creatures/friends.v1.json` defines twelve friends across three regions, silhouettes, personalities, habitats, clues and assistance | Give each friend a distinct motivation, voice and small cooperative arc. Keep the seven avatar IDs separate from collectible friend identities. |
| Friendship rules | Rust domain has recruitment, distinct helping moments, a cap, team selection and reversible story-gated transformations | The current batch extends snapshots and command handling so presentation can consume authoritative friendship, helping opportunities and transformation state. Missing optional snapshot fields must keep older clients compatible and must not be inferred from browser activity. |
| Browser presentation | Meadow has four friendship quests, server-owned collection and a journal with companion selection | Add authored personality, habitat clues and short dialogue to the journal. Preview dialogue using confirmed ownership plus server friendship and story gates; keep release content review pending. |
| Visual identity | Asset manifest contains Pip art and 23 missing friend/transformation assets | Existing Meadow sprites are provisional stand-ins. Species-specific art, atlas coverage and provenance remain prerequisites for finished character presentation. |
| Narrative depth | Current friends have brief personality labels; glow transformations share the same gates | Author observable changes in how each friend approaches a problem. Preserve support, free alternatives and cosmetic transformation rules. Avoid dialogue suggesting abandonment, affection loss or learning mastery. |
| Content compatibility | Current story manifest hashes exact source bytes and Rust reads the existing creature pack | Introduce a separately versioned development sidecar with validation. Do not silently mutate the frozen gameplay pack or imply sidecar prose changes gameplay rewards. |

## Implementation sequence

1. Author twelve character profiles with motivations, voice guidance and friendship scenes. Validate roster coverage, story references and milestone bounds; keep review status pending.
2. Present Meadow profiles and ownership-appropriate clues/support in the current journal, with accessible labels and escaped text. Keep server-owned companion selection. The development journal may preview the highest eligible authored moment from server ownership, friendship and story flags.
3. Review deterministic friendship and transformation rules for concrete defects and add regression coverage where needed.
4. Follow up with versioned friendship/transformation snapshot fields and commands, API transactions, generated-contract fixtures and browser integration. These fields and commands are now part of the integration batch; validation evidence must accompany their browser controls.
5. Produce and review distinct friend art and transformations, then test character recognition, reading burden and attachment with learners.

Steps 1–4 form the current implementation batch; step 5 remains an explicit dependency rather than a promised completed feature. Orchard and Cove profiles can be authored now without activating unfinished regions.

## Completed batch and verification

Three subagents implemented a narrative sidecar, journal presentation and domain regression coverage. `content/creatures/development.v1.json` contains twelve motivations, growth opportunities and voices, with 36 gated friendship moments. The journal integration now previews motivations and the highest eligible draft dialogue from server-confirmed friendship and story state. Missing friendship suppresses earned dialogue. Content lines were reviewed to avoid inventing specific player actions from generic helping points. Existing hashed gameplay content remains unchanged.

Validation passed: story validator; all 12 story tests; `cargo fmt --all --check`; all 13 domain tests via `cargo test -p wonderwilds-domain --locked`; and frontend `npm run check` (typecheck, lint, formatting, nine test files and production build). The build retains the existing large Phaser bundle warning. Automated Chromium desktop/tablet interaction checks passed with a synthetic API fixture. No human usability or screen-reader review was performed for these cards.

Follow-up content validation passed after the dialogue review: `python
content/story/validate.py` and `python -m unittest discover -s content/story -p
'test_*.py'` (12 tests). The sidecar activation and hashed gameplay manifest remain
unchanged. The local dialogue preview is not release content approval; new API and
browser verification results are recorded by their integration owners.

## Server integration verification

`bash scripts/check.sh` passed across the workspace, including 62 shared wire fixtures, generated-code drift, Rust formatting/Clippy/tests, content validation and frontend checks/build. `sh scripts/check-postgres.sh` passed both database-dependent tests plus migration/ownership/export/deletion checks against a disposable local database. New API regressions verify locked commands, duplicate helping moments, exact retries, serialization reload, reversible transformations and unchanged XP/inventory/learning evidence.

`node qa/e2e/companion-smoke.mjs` passed desktop and tablet memory/glow/reload flows against synthetic HTTP fixtures served to the real browser frontend. This browser test does not itself prove database persistence; the Rust aggregate roundtrip and PostgreSQL repository checks supply separate evidence. See `adr/character-progression.md` for compatibility and rollback.
