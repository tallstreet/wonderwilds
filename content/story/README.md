# Authored story pack v1

This is disabled, pending-review content for WW-04. It does not activate Orchard or Cove or change the existing guest adventure. Rust must remain the authoritative runtime; Python is an offline content validator and reference simulation, not a gameplay service.

The pack provides eighteen authored beats in three chapters, cooperative multi-step finales and an ending; twelve original creature briefs and friendship quests; separate story XP/friendship policies; reversible cosmetic transformations; authored fallbacks; and deterministic, bounded discovery. Existing avatar IDs and Pip are retained. Recruitment of Pip requires Meadow's finale; essential story beats require no creature or academic mastery. Supported completions earn the same story rewards and are explicitly not independent evidence.

Run from the repository root:

```sh
python content/story/validate.py
python -m unittest discover -s content/story -p 'test_*.py' -v
```

`manifest.v1.json` conforms to `ContentManifest@1`. Its SHA-256 covers the concatenation of each UTF-8 path, a NUL byte, and exact file bytes, in this order:

1. `content/story/adventure.v1.json`
2. `content/story/progression.v1.json`
3. `content/story/help-quests.v1.json`
4. `content/creatures/friends.v1.json`
5. `content/manifests/story-assets.json`

`story-assets.json` inventories existing map/Pip artwork using exact hashes; it explicitly marks 23 new creature/transformation assets missing. Existing assets have not received an independent provenance/rights review. No art is inferred from an unrelated atlas frame. All review states remain pending.

Integration: WW-05 should load immutable versioned story/quest data, apply prerequisite and grant-key rules in Rust transactions, and record discovery seed and pending offers. WW-06 supplies actual bounded encounters for the referenced skill/template IDs. The Python discovery function is a golden reference: port and compare vectors in Rust. Persist a declined offer rather than drawing again; a completed help quest consumes it. Every still-missing regional friend is offered within four completed eligible expeditions; this is an opportunity guarantee, not forced completion. Optional revisits do not repeat the last friend where alternatives exist.

WW-12 must supply region navigation, art and authoritative state presentation before activation. The listed finale steps are all previously completed story tasks to revisit cooperatively; completing them with support must still unlock the light. Friend abilities expose only optional helpers, with free control alternatives and support recorded. None bypasses answer or gate validation.

Migration/rollback: this pack does not read/write legacy saves, alter hero IDs or activate maps. Roll back by keeping it disabled. A future activated version must retain a compatible reader and pending-encounter replacements. Runtime idempotency and grant atomicity belong to WW-05/09 and are not proven by this offline content suite.

Outstanding acceptance: WW-02 integration gate, educator/narrative/child-safety review, original art and rights approval, activity and Rust runtime wiring, browser navigation and complete playthrough. No named human reviewer or approval is fabricated.

Contract normalisation: legacy `number_sequence` story/help tasks explicitly use
`pattern_sort` with a numeric sequence operation for M05; this is counting forwards,
not a visual repeating-shape task. Legacy `phonics_word` explicitly uses
`phonics_word_building`; E05 asks for a taught sound-to-letter connection while
E06 decodes a regular word. Assessment adapters retain those distinct semantics.

## Authored director integration (WW-13)

`backend/crates/director/src/core.rs` supplies a pure development director for all
eighteen beats through `assessment::wire::authored_encounter`. Construct `Frontier`
from trusted state with the next beat ID as its candidate ID, all completed beat
IDs, current revision, region, ephemeral request ID and recorded seed. The input
builder emits only known canonical beat facts and bounded candidate tuples. It
rejects unknown/inconsistent facts instead of forwarding child text or profiles.

`choose_with_budget(frontier, None, 0)` selects the authored path with AI disabled.
Provider bytes are bounded to 8 KiB, schema/version checked and matched to the
current revision, allowed authored tuple and exact authored dialogue. Rejected,
missing or late (2 s) proposals fall back without side effects. Child-visible text
always comes from the authored lookup. Candidate choice prefers templates absent
from the last three when an eligible alternative exists; parameter variants use a
recorded seed. The director does not choose a different academic skill.

Record `Selection.replay`; `replay` reconstructs without inference or wall clocks.
Recheck `verify_commit_revision` while holding the authoritative transaction lock.
The director neither grants story rewards nor commits evidence or a next beat;
`suggestedNextBeatId` is only a suggestion. HTTP authorization, task attempts,
atomic state/evidence/outbox writes and provider timeout enforcement belong to
API/worker integration. The pure deadline function uses caller-injected elapsed
time; it does not start a network operation or run a live timer.

Validation from root with the installed pinned Rust toolchain:

```sh
cargo test -p wonderwilds-director --locked
cargo clippy -p wonderwilds-director --all-targets --locked -- -D warnings
```

Nine tests passed, including 832 deterministic disabled-provider scenarios and
24 complete 18-beat authored journeys with timeout fallback/replay. These are
headless director tests, not browser playthroughs or proof of API transactions,
release approval, learning gains, or human accessibility validation.

## Character training development slice

`progression.v1.json` now contains the separate `character-training-1.0.0`
policy: levels at 0/10/20/30/40 character XP and four one-time training grants
for each Meadow friend. Supported completion grants the same ten XP. Replaying
an earned milestone remains practice and grants zero additional character XP.
Explorer XP, story flags, friendship, transformations and learning evidence do not
change merely because a character levels up. The normal verified encounter
start/completion/abandonment protocol binds the recipient; changing the team
cannot redirect the grant. Wrong answers and abandonment award no XP.

The new optional persisted state fields default to zero XP and no training grants
when loading pre-training saves; existing ownership, friendship and transformations
are retained. Historical quests never manufacture character XP. Pending training
encounters pin the character-training rules version. Rolling back to a pre-training
binary after new training events have been persisted is unsupported: retain a
pre-upgrade backup or use a compatible reader, rather than silently dropping
training state or receipts.

The sixteen authored milestones provide reachable level-five XP paths without
repeating a rewarded task. Their reaction IDs are presentation references, not
claims of new sprite sheets. The initial UI celebration reuses existing art;
character-specific exploration roles, personal quests, distinct higher-level
rewards and new transformation artwork remain subsequent implementation work.
The updated manifest hash covers the intentional progression policy addition;
content and educator approval remain pending and release activation stays disabled.
