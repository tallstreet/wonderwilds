# First milestone review

Plan: [IMPLEMENTATION_PLAN.md](IMPLEMENTATION_PLAN.md).

## Acceptance checklist

- [x] Clover Meadow is the only active map; no unfinished destination is advertised as playable.
- [x] Keyboard and touch/click controls reach all activities; paths avoid obstacles.
- [x] The bridge blocks passage until repaired; its activity remains reachable.
- [x] Bridge manipulatives support adding/removing objects and deterministic checking.
- [x] Sharing activity conserves six berries and checks equal allocation.
- [x] Sequence activity combines numerals and countable representations.
- [x] Wrong answers permit retry and useful hints, without losing story progress.
- [x] Guided completion advances story but records support accurately.
- [x] Completion changes the world and produces a clear adventure ending.
- [x] Journal/companion reward persists.
- [x] Reload restores progress and recorded learning evidence.
- [x] Invalid saves recover safely; unavailable storage leaves the game usable.
- [x] Reset is intentional; export contains meaningful evidence.
- [x] Progress view distinguishes practice/support from demonstrated mastery.
- [x] Dialog keyboard focus and small-screen layouts are usable.
- [x] Assessment/storage/progression tests and production build pass.

## Verification record

Reviewed on 28 October 2026 by the parent agent after delegated implementation.

### Automated and build checks

- Nine Node tests pass: deterministic maths, unequal sharing, unique story rewards, supported evidence, malformed saves, unavailable storage, reload restoration, saved position recovery, and gated navigation.
- Vite production build passes. A non-blocking warning reports the Phaser bundle exceeds 500 kB.

### Browser checks

- Phaser map renders after fixing an SVG data-URI loader failure discovered during review.
- Adventure cards route the kitten to the bridge, picnic, and garden. The garden route crosses the repaired bridge.
- Keyboard left movement and direct map-click movement were verified visually.
- Bridge: an incorrect check permits retry; excess planks are visible; four added planks complete the bridge.
- Picnic: 4/2 sharing is rejected; taking one back and redistributing to 3/3 succeeds. A used hint is retained in learning evidence.
- Garden: supported completion succeeds and records assisted=true and supported=true.
- All three activities award a single star, update world art, unlock the chapter celebration, and recruit a following companion.
- Journal hero selection and companion naming survive reload alongside three stars and three evidence records.
- Grown-up view correctly shows bridge without hints, picnic with support, and garden with support.
- JSON download was opened and verified: correct skills, representations, attempt counts, hints, assistance flags, and completion timestamps.
- Reset confirmation opens with “Keep my adventure” focused; cancellation preserves progress. Destructive reset was not exercised in browser QA.
- Garden and celebration dialogs were visually inspected and operated at 390×844. Normal viewport was restored afterward. Full physical tablet/touch-device testing remains outstanding.
- Storage failure and corrupt-save recovery were tested at the pure storage layer, not by altering browser storage permissions.

### Corrections requested during supervision

Obstacle-aware paths, blocked bridge until repair, preserved player position, stricter evidence validation, visible surplus planks, modal focus/keyboard handling, and base64 SVG loading.

### Remaining research

Child usability, learning transfer, and a 10–15 minute completion time require future supervised playtests and are not established by automated or adult browser checks. This milestone contains fixed quantities and reports practice evidence, not mastery. Backend, adaptive variants, and AI fine-tuning remain later phases.

Preview screenshot: [milestone-preview.png](milestone-preview.png).
