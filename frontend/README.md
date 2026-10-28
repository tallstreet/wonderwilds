# Starwhisker Island — frontend milestone

A local-only Phaser 3 maths adventure for early learners. Explore Clover Meadow as an original superhero kitten, repair a bridge, share six berries fairly, and wake a flower garden by counting forwards.

## Run

Requires Node.js 20.19+ or 22.12+.

```sh
cd frontend
npm install
npm run dev
```

Open the Vite URL printed in the terminal. For a loopback-only server: `npm run dev -- --host 127.0.0.1`.

```sh
npm test
npm run build
npm run preview
```

## Code quality

All application code and tests use strict TypeScript. `src/types.ts` defines the shared encounter, activity, evidence, movement, and actor contracts. Saved JSON is treated as unknown and validated before use. Vite handles bundling; every production build runs the TypeScript checker first.

```sh
npm run typecheck     # Strict checking, including tests
npm run lint          # Oxlint; warnings fail the check
npm run lint:fix      # Apply automatic lint fixes
npm run format        # Format source, styles, HTML, and documentation with Oxfmt
npm run format:check  # Verify formatting without changing files
npm run check         # Typecheck, lint, formatting, tests, and production build
```

Tool configuration lives in `tsconfig.json`, `.oxlintrc.json`, and `.oxfmtrc.json`. Generated output, dependencies, and the npm lockfile are excluded from formatting. Tests run through Node's test runner with the `tsx` loader; run `npm ci` to install the locked dependencies on a fresh checkout.

## Play

- Click/tap the ground to walk around obstacles, or use arrow keys / WASD.
- Click a kitten or an adventure card to walk to an activity. Press Space near a kitten to interact.
- Repair the bridge before crossing to the garden. Picnic and bridge activities are initially reachable.
- Each activity has countable visual objects, retries, three hints, optional browser read-aloud, and supported completion.
- Finishing all three activities recruits Pip as a following companion. The journal lets you choose a kitten appearance and name your companion.
- Completed activities can be replayed. Replays add practice observations, never duplicate stars.

## Saving and learning evidence

Progress uses the versioned localStorage key `starwhisker.meadow.v1`. Story rewards, kitten choice, companion name, player position, and the last 200 completed practice sessions are stored in this browser. No account, server, API key, analytics service, or AI model is connected.

The page shows “Session only” if storage is blocked or full. Download the learning record from **For grown-ups** before closing that session. Browser data clearing removes saves. Reset requires an explicit confirmation and clears story and evidence together.

Each practice record includes skill ID, encounter ID, fixed variant, representation, number of answer checks, hint count, assisted-completion flag, support classification, and completion timestamp. A hint or the together option always counts as support. Retry feedback is not counted as an explicit hint; “without hints” means no hint button or worked example was used, not necessarily first-try success. Incomplete activities do not produce completion records.

Repeated success on this fixed activity is **not** a mastery judgement. The grown-up view reports observations and explicitly explains the limitations. No personal child profile is requested. Companion names are rendered as text, not HTML.

## Module boundaries

- `src/world.ts`: Phaser rendering, interactions, world consequences.
- `src/navigation.ts`: collision geometry and grid pathfinding.
- `src/art.ts`: hero roster and shared portrait helpers.
- `src/sprites.ts`: shared generated character/prop atlas frames for Phaser and HTML.
- `src/scenery.ts`: interactive props layered above the generated landscape.
- `src/content.ts`: authored content, skill metadata, deterministic answer rules. Future island data is retained but not active.
- `src/progress.ts`: validated save format, evidence integrity, story rewards, summaries.
- `src/main.ts`: accessible HTML dialogs, manipulatives, local persistence, journal, grown-up view.
- `src/style.css`: responsive page and activity presentation.
- `src/game.test.ts`: maths, progression, storage, evidence, and navigation tests.

The implementation plan is in `../docs/IMPLEMENTATION_PLAN.md`; parent review results are in `../docs/MILESTONE_REVIEW.md`.

## Validation and limits

Automated tests cover deterministic answers, unequal sharing rejection, duplicate reward prevention, supported-evidence classification, invalid save recovery, blocked storage, future-map exclusion, and repaired-bridge path routing. Production build passes; Vite warns that the Phaser bundle exceeds 500 kB. This is expected for the current single-bundle prototype, not a build failure.

Browser QA is tracked in the repository milestone review. Child playtesting and educator validation have **not** occurred. The proposed 10–15 minute play session is a research target, not an established result.

Current boundaries:

- One active map and three fixed activities, not a full curriculum.
- Activities prompt hints after repeated incorrect checks; no adaptive quantity generation yet.
- No backend, cloud sync, live model, or fine-tuning.
- Map, characters, scenery props and activity objects use a consistent generated 3D-style art set. See `docs/CONSISTENT_ART.md` for asset paths and exact prompts.
- Google Fonts is optional network-loaded presentation; local fallback fonts work without it.
- Browser speech availability and voice vary by device.
- Phaser map exploration is visual; core activity controls and quest navigation are exposed as HTML, but full screen-reader world navigation is not implemented.
- Small phones fit the complete map; tablet/desktop is the intended first playtest surface.

## Full-window play mode

The game now fills the browser viewport with a following Phaser camera. Floating controls open Missions, Journal, and Grown-ups dialogs. Selecting a mission closes the menu and routes the kitten to that activity. Activity dialogs pause walking and return to the same world position when closed. The desktop expand button requests browser fullscreen when supported; the game fills the window even without that API. Portrait screens show a closer view of the world and pan with the kitten rather than shrinking the entire map.

## Hero roster

Open **Journal** to select a playable hero: Rosie, Skye and Sunny (kittens), Poppy and Wren (birds), or Clover and Mochi (bunnies). All seven use original generated character sprites in the same shaded superhero style as the map. The selected hero appears in the world and celebration, and is stored with existing progress. Changing heroes does not alter curriculum difficulty or erase learning records. New species retain the same navigation/collision rules; birds do not bypass the bridge puzzle.

Hero selection and reload restoration were browser-checked for Poppy and Clover. Ten automated tests and the production build pass, including validation of new hero IDs and rejection of unknown IDs.

## Unified art pass

The built-in imagegen tool produced shared character and prop atlases using the map and squad as style references. Phaser and HTML draw from the same source rectangles. Bridge planks and berries remain individually counted by code, and the garden swaps between sleeping and blooming assets. Twelve tests pass, including atlas bounds, hero coverage and generated-map collision checks. The production build passes with the existing Phaser bundle-size warning.
