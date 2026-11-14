# Local browser checks

Human story review: `node qa/e2e/story-human-review.mjs` starts an isolated Python
review server on an ephemeral loopback port and uses a disposable progress file.
It checks blind grades, all three decisions, reload/server-restart persistence,
failed-save recovery, revision gating, JSONL export, request protections and mobile
layout. Set `CHROMIUM_PATH` as below. No actual human decisions or demo data are used.

Live mission feedback: `node qa/e2e/mission-feedback.mjs` starts an isolated
loopback review server with synthetic capture and feedback files. It checks safe
text rendering, 1–5 scores, durable feedback across server restarts, newly appended
captures, failed-attempt filtering, JSONL export and phone layout. It does not
call a model or modify demo data. Set `CHROMIUM_PATH` for your installed browser.

Use the existing isolated checkout; no worktree is needed. From `frontend/`,
start `npm run dev -- --host 127.0.0.1`. From this directory run `npm ci` then
`npm test`. Chromium is available at `/usr/bin/chromium` in the prepared cloud
snapshot. Set `CHROMIUM_PATH` for a different installed browser, or
`WONDERWILDS_TEST_URL` for a different local development server.

These desktop/phone checks use synthetic API fixtures to verify the single story
world at `/`, retired query parameters and redirected inspector/demo URLs, plus
all scene-source statuses. No model service or database is needed for this smoke
suite. Real persistence and mission journeys use the disposable runner below.

Character library checks: `node character-library-smoke.mjs` against the running
frontend (default port5174; override with `WONDERWILDS_TEST_URL`). It verifies all
twelve actual artwork URLs decode, regional filters, keyboard details and desktop,
tablet and phone layouts. This page is a field guide, not a recruitment action.
Default authored story journey against the real API and PostgreSQL:

```sh
node qa/e2e/run-training-qa.mjs
```

Run that command from the repository root on macOS or Linux after installing
PostgreSQL 17, the pinned Rust toolchain, Node 24, and the frontend and `qa/e2e`
dependencies. `WONDERWILDS_PG_BIN` can select the PostgreSQL bin directory. It builds
the API, creates a fresh disposable private-socket cluster, applies migrations, issues a temporary
operator session, and starts private API/Vite processes on ports 3002/5180. It
stops and removes the cluster and temporary session when finished. It does not use or
modify `.local-demo/` data or session files. Set `CHROMIUM_PATH` if needed;
`WW_QA_SKIP_BUILD=1` reuses an API binary you have already built. Do not set the
internal wrapper variable `WW_QA_DATABASE_URL`; see [CI scope](../../docs/CI.md).

The default journey now runs `story-world.mjs` directly at `/`. Legacy guest,
training and inspector browser helpers are historical and no longer active app
checks. Use `WW_QA_API_PORT` and `WW_QA_VITE_PORT` to avoid busy QA ports.

Focused mission controls (with Vite running) cover mouse/touch dragging, tap and
keyboard matching, answer trays, phone layouts and mocked speech review/cancellation:

```sh
node frontend/src/mission-learning/activity.browser.mjs
```

Set `WW_ACTIVITY_TEST_URL` for another Vite origin. No provider calls are made.

Curriculum practice inside the story world:

```sh
WW_QA_BROWSER_TEST=qa/e2e/mission-learning.mjs node qa/e2e/run-training-qa.mjs
```

This uses the same fresh synthetic cluster and ports. It accepts a story mission,
opens an inline task by gathering mission supplies, submits a correct and then
wrong one-to-one assignment, records help, retries an uncertain save after reload
and completes a workshop step together. It checks atomic story advancement,
independent versus assisted evidence, desktop/phone controls and separate
guest/assessed-adventure saves. Full story journeys use `inline-step.mjs` to
complete these interactions through the supported route, including **Finish activity**
for human-reviewed workshops. The mission practice client tests
cover exact retry and stale-revision recovery; learning-engine tests cover the
five supported skills, evidence diversity and graph progression. This is not
full 120-skill activity coverage or educator approval. Keep model providers
disabled by unsetting both Modal credentials for these assessment/persistence checks.

Phaser Pip story world against a fresh API/PostgreSQL profile:

```sh
WW_QA_BROWSER_TEST=qa/e2e/story-world.mjs WW_STORY_SOURCE=bank node qa/e2e/run-training-qa.mjs
WW_QA_BROWSER_TEST=qa/e2e/story-world.mjs WW_STORY_SOURCE=workshop node qa/e2e/run-training-qa.mjs
```

Run each route separately: each command owns a new disposable cluster and ports
3002/5180. This opens the story world directly at `/`, walks to actual Phaser characters and resources,
and checks dialogue/choices from a schema-valid mock presentation, literal HTML
text rendering, explicit introduction acknowledgement, both wood sources,
exchange, lost-response retry after another tab advances the story, failed-refresh recovery without rolling the world back, and bridge repair. It attempts to walk
across without a server command before and after repair, then verifies Pip follows
the player across the bridge and appears once on the far bank. Phone layout,
reload, a nonplayable save-limit response, untouched assessed progress and unchanged guest-save bytes are included.
Screenshots are written under `/tmp/wonderwilds-story-world-*.png`; no persistent
demo data is used. The mock changes only initial display prose and a choice label;
all progress changes use real UI commands and the authoritative server runtime.
Route visits wait for either a travel choice or the saved destination when walking
automatically crosses an area exit. Travel must advance the story revision exactly
once; the second-tab recovery path also accepts automatic arrival.

All three connected stories against a fresh API/PostgreSQL profile:

```sh
WW_QA_BROWSER_TEST=qa/e2e/story-series.mjs node qa/e2e/run-training-qa.mjs
```

This completes the bridge, picnic and lights through world interactions, accepts
each successor explicitly, checks pause/resume, reloads between stories, and retries
a committed exchange whose response was lost. It checks both crossing directions,
persistent picnic/lantern scenery, completed history, mobile layout and the
retired inspector redirect, while verifying assessed progress remains untouched. Screenshots are
written to `/tmp/wonderwilds-story-series-{desktop,mobile}.png`.

Seven connected stories, including the four broader mission patterns:

```sh
WW_QA_BROWSER_TEST=qa/e2e/story-broader.mjs node qa/e2e/run-training-qa.mjs
WW_STORY_BRANCHES=alternate WW_QA_BROWSER_TEST=qa/e2e/story-broader.mjs node qa/e2e/run-training-qa.mjs
```

Run each command separately against its own disposable cluster. Both start with
the bridge, picnic and lights, then play the humming mystery, Pip's cosy corner,
Bramble's visit and the moonlight gathering. The routes choose different lasting
instrument homes, corner designs and gathering banks, and prepare seats/music in
opposite orders. The instrument is physically placed at its chosen bank; choosing
a destination alone must not render a finished decoration. The test also checks
immediate visible discovery/preference replies, paused corner progress and reload,
Bramble following the player from the workshop into the meadow without a duplicate
stationary character, present friends at the gathering, all seven completed goals,
persistent scenery, mobile layout and unchanged educational progress. The woodcutter
uses existing Cleo art so Bramble remains visually distinct. Screenshots are written
to `/tmp/wonderwilds-story-broader-{default,alternate}-{desktop,mobile}.png`.


To require a persisted model plan for each of the seven accepted missions, run
`run-training-qa.mjs` with `WW_QA_BROWSER_TEST=qa/e2e/story-broader.mjs`,
`WW_STORY_EXPECT_PLANS=1` and exported Modal proxy credentials. All three
generators run together; this makes live requests and requires a healthy deployment. It
uses only the runner's disposable synthetic database, checks saved plans against
snapshots and the Missions dialog, and verifies reload stability. Authored fallback
is deliberately a failure in this opt-in mode, so availability is not mistaken for
model-quality evidence. Without the flag, the ordinary authored journey is unchanged.


Combined scene generation and mission planning with returning sessions:

```sh
WW_STORY_COMBINED=1 WW_STORY_EXPECT_PLANS=1 \
WW_STORY_REPORT=/tmp/ww-combined-default.json \
WW_QA_BROWSER_TEST=qa/e2e/story-broader.mjs \
node qa/e2e/run-training-qa.mjs
```

Export the existing Modal proxy credentials first; never commit them or include
values in evidence. Repeat with `WW_STORY_BRANCHES=alternate` and a different report
path. Each invocation owns a new disposable database. This is an opt-in live-model
check with inference requests, not an ordinary offline CI test.

The journey closes the browser context and restarts the API three times: after the
original trilogy, while the corner mission is paused with a lasting choice, and
during gathering preparation. A fresh browser context resumes the same synthetic
profile/session credential; this does not test production login or session renewal.
It compares revision, full canonical scene input and persisted plans before and
after each restart, then completes all seven missions. It also loses one committed
story-action response and checks reload plus exact-body retry without duplicate progress.

Both providers must be enabled; all seven accepted missions must have persisted
model plans and the API must report accepted scene responses in each of the four sessions. The JSON summary
counts accepted/fallback/stale scene responses and accepted/fallback plans. Scene
counts include cache hits and QA reads, so they are not unique generation counts.
The browser log reports read and action-through-refresh timing distributions;
these include UI/cache effects and are not isolated provider latency or token cost.
Fallback is reported separately and never counted as successful model generation.
A run with no accepted scenes or missing plans fails, even if authored play works.
These checks establish mechanical continuity, not editorial prose approval.

For an offline rehearsal of the same restarts and retry checks, use
`WW_STORY_RETURNING=1` with both Modal credentials unset and omit
`WW_STORY_COMBINED`/`WW_STORY_EXPECT_PLANS`. Repeat both branches. An authored
rehearsal cannot satisfy combined-model acceptance.


## Long-play persistence

For story, use `WW_LONG_PLAY=1 WW_STORY_RETURNING=1
WW_QA_BROWSER_TEST=qa/e2e/story-broader.mjs node qa/e2e/run-training-qa.mjs` on one
shell line. It adds 300 travel commands before the seven-mission journey and its
three returning-session restarts. Repeat with `WW_STORY_BRANCHES=alternate`.
Leave the Modal providers disabled for storage-only QA. Set `CHROMIUM_PATH` for
local Chromium; the runner discovers PostgreSQL or accepts `WONDERWILDS_PG_BIN`.

The separate `sh scripts/check-postgres.sh` suite verifies migration, exact old
receipts, audit/recovery, missing-history refusal, snapshot sizes and ownership.
See [storage formats and recovery](../../docs/LONG_PLAY_STORAGE_PLAN.md).

Story source display regression (synthetic API responses, no database/provider):
start Vite, then run `WONDERWILDS_TEST_URL=http://127.0.0.1:5173 node qa/e2e/story-source-status.mjs`
from the root, with `CHROMIUM_PATH` when needed. Covers authored, generated,
fallback and missing metadata at desktop/phone sizes in the world and retired URL redirects.

`WONDERWILDS_TEST_URL=http://127.0.0.1:5189 node qa/e2e/dialogue-polish.mjs`
checks the mission journal and non-modal speech on desktop/phone with synthetic
story responses. Start Vite on that port and set `CHROMIUM_PATH` if needed. It
uses no database or provider and checks that only explicit acknowledgement
submits a command; reading and dismissing speech do not write progress.

`WONDERWILDS_TEST_URL=http://127.0.0.1:5189 node qa/e2e/mission-journal.mjs`
checks that the open mission heads the dialog and opens in view on desktop/phone,
completed Pip progress stays collapsed, return destinations survive reload, and
active Pip work retains priority in his areas. It uses synthetic responses and
asserts that reading the journal writes no progress. Use the same Vite and
`CHROMIUM_PATH` setup above.

### Story neighbourhood exploration

`WW_QA_BROWSER_TEST=qa/e2e/story-exploration.mjs node qa/e2e/run-training-qa.mjs`
creates a disposable PostgreSQL cluster and checks a keyboard map-edge transition,
five character missions, the four-slot policy, archive/resume with retained progress,
completion, a lost-response exact retry after reload, and phone layout. Set
`CHROMIUM_PATH` to an installed executable when needed. Never point this journey
at persistent demo progress.

`node qa/e2e/run-generated-exploration-qa.mjs` runs the same real-world journey
against a local synthetic OpenAI-compatible model endpoint and a disposable
PostgreSQL cluster. It checks generated people and multiple steps, exact retry,
archive/resume, stable reloads, malformed-model fallback and a fresh story after
returning to a completed area. It makes no paid model calls. The ordinary database
suite additionally checks eight generations, rollback and full durable replay.


Story bag presentation: with Vite running on port 5189, run
`node qa/e2e/story-bag.mjs` (or set `WONDERWILDS_TEST_URL`). Set `CHROMIUM_PATH`
when needed. Synthetic responses cover empty and populated bags, updated quantities
after reload, desktop/phone fit, keyboard opening/dismissal and focus restoration,
Missions access and no progress writes. No demo database or provider is used.

Mission journey regression (no paid calls): set synthetic `MODAL_PROXY_TOKEN_ID`
and `MODAL_PROXY_TOKEN_SECRET`, `WONDERWILDS_MODAL_ENDPOINT=http://127.0.0.1:1/v1/chat/completions`,
and `WW_QA_BROWSER_TEST=qa/e2e/story-mission-variety.mjs`, then run
`node qa/e2e/run-training-qa.mjs`. This uses a fresh disposable database and saved
offline story fallbacks. It checks cross-area inline learning, clue replies,
current-objective tracking, phone journal destinations, reload, return/payoff and
a different story length/artwork on the next visit. Set `CHROMIUM_PATH` as needed.


### Generated project mission variety

`qa/e2e/story-mission-variety.mjs` runs through the portable disposable-database
runner. It exercises real project gathering/crafting/delivery/construction,
creative choices, inline supported learning, bag reservations, saved projects,
destination completion, reload and discovery stories at desktop/phone sizes.
Use synthetic provider credentials with an unavailable loopback endpoint so no
paid generation or persistent demo progress is involved:

```sh
MODEL_API_KEY= BASETEN_API_KEY= WONDERWILDS_MISSION_LOG= \
CHROMIUM_PATH="/path/to/Chromium" \
WW_QA_BROWSER_TEST=qa/e2e/story-mission-variety.mjs \
MODAL_PROXY_TOKEN_ID=synthetic MODAL_PROXY_TOKEN_SECRET=synthetic \
WONDERWILDS_MODAL_ENDPOINT=http://127.0.0.1:9/v1/chat/completions \
node qa/e2e/run-training-qa.mjs
```

Run disposable PostgreSQL/browser suites sequentially on macOS if concurrent
clusters exhaust the system shared-memory limit. The helper shuts down and
removes its own temporary cluster; do not stop the persistent demo database.


`qa/e2e/story-lantern-hint.mjs` separately prepares the reported Welcome lights
state in a fresh synthetic profile, verifies the woodcutter-only hint (without
route/placement instructions or guide buttons), and completes real crafting and
both lantern placements with supported learning and reload. Select it with
`WW_QA_BROWSER_TEST` in the same runner, with both Modal credential variables
empty to use authored content. It never advances the persistent demo profile.

## Authored campaign checks

`node qa/e2e/campaign-run.mjs` runs the real Bridge/Bloom browser journey with
Pip, Tansy and Nib, each in its own newly-created disposable PostgreSQL cluster.
It clears provider credentials and uses the ordinary authenticated local demo
bridge. The campaign-only Vite configuration disables automatic file reloads;
explicit browser reloads still exercise persisted application state. Default
interactive development and the existing training runner remain unchanged.
Set `WW_CAMPAIGN_STARTERS=pip` to run one starter. Use `WW_QA_SKIP_BUILD=1` only
when the API binary already matches the campaign content and contract sources.

`node qa/e2e/campaign-longplay-run.mjs` runs the three starters through both
Orchard orders, two isolated cases at a time on ports 3010/3011 and 5190/5191.
`WW_CAMPAIGN_ORDERS=harvest` selects one order. This checks the actual order of
Harvest and Current trial actions, supported learning transactions, authored puzzle
configurations, eight seals, all twelve companions at level five, forty-eight
milestones, the ending and API/browser restart persistence. Failures stop new
cases and let active disposable runners clean up. Logs and JSON evidence are
written under `/tmp/campaign-longplay-STARTER-ORDER.*`; successful screenshot
coverage is written under `/tmp/campaign-ending-STARTER-ORDER-*`.
Pip/Harvest uses actual browser supported activity controls throughout. The other
five cases use authenticated API `begin_campaign_step`/`finish_together` commands
for learning, then render the real final UI and check form/reload/phone behavior.
The three M2 journeys independently exercise each starter’s actual controls.
Set `WW_CAMPAIGN_STAGE=1.1` to keep continuation artifacts separate from the
initial campaign evidence. Longplay records guided request timings with an
eight-second budget and any guided retreats. An unbeaten board is attempted once
per team, companion-level and ability state so an optional retreat cannot starve
further exploration; later progress permits another attempt. Completion requires
Rowan's three-round relay and two contributing affinities when present in content.

The M2 journey additionally checks keyboard movement, trial failure/recovery,
reload during a trial, an exact retry after a lost committed response, actual
world trail-sign interaction, recruitment/care/team changes, a return visit,
paired practice without repeated grants, phone layout and recovery when reads
fail after a confirmed write. Automated checks are technical evidence; they do
not establish educator, narrative, accessibility or child playtest approval.

`campaign-art-preview.mjs` is a separately labelled synthetic renderer fixture,
not evidence of earned companion levels. It uses standalone Vite on port 5185.

The 14 November 2026 run is recorded in [campaign-evidence.json](campaign-evidence.json):
all three M2 journeys and all six full starter/order cases passed, along with the
activity browser suite and the ten standard URL/viewport cases. The standard
suite also checks onboarding X/Escape dismissal without a campaign write. Actual
full saves contain 465–466 flags and 409–411 grants, so the earlier 256-item wire
bound was insufficient; the corrected bounded contract was exercised by every
completed case. Accepted final collection screenshots come from Tansy/Current
and both Nib cases after the card layout fix. The preserved `ending-*` images in
`docs/screenshots/campaign/` are from Nib/Harvest. Earlier Pip and Tansy/Harvest
screenshots predate that CSS fix and are not final layout evidence.

## Current legacy-reader and M6 continuation checks

Current story fixtures and journeys use `legacy-entry.mjs` to request the supported
`?adventure=earlier` reader explicitly. This preserves new onboarding coverage in
`story-source-status.mjs` while making legacy gameplay intent explicit. Both bank
and workshop routes of `story-world.mjs` were rerun against fresh PostgreSQL and
passed, including joined learning, retry/reload, shared crossing and phone layout.
The real `mission-learning.mjs` journey was also rerun: its private fixtures now
include the active NSW question bank, and it distinguishes task completion from
ten-item skill readiness and an eligible unsuccessful first check from an
independent correct check. It exposed and verifies the fixed recovery of a
resolved task after a definitive conflict while whole-skill readiness is false.
Retained tests for the already-retired inspector/assessed interfaces, such as
`story-browser.mjs` and `backend-world.mjs`, are historical; the earlier-reader
entry helper does not revive those interfaces or establish that those scripts pass.

For technical accessibility checks, run:

```sh
WW_CAMPAIGN_STARTERS=pip \
WW_QA_BROWSER_TEST=qa/e2e/campaign-accessibility.mjs \
node qa/e2e/campaign-run.mjs
```

The four controlled browser cases check unavailable narration, asynchronous
speech-service failure, synchronous speech rejection and a live change to reduced
motion. They verify written fallback, heading/launcher focus, keyboard dialogue,
narration cancellation, actionable-control contrast/target checks, visually static
reduced-motion idle rendering and preserved keyboard movement. They do not replace
native screen-reader testing, audible OS voice review, physical phone performance
measurements, consented child playtesting or production identity/privacy review.
`campaign-trial-preview.mjs` creates a real authoritative Bridge trial for semantic
progressbar and confirmed-signature/cleared-effect/celebration inspection; it does
not invent a completed campaign save.

`campaign-paired.mjs` uses `WW_QA_SECOND_PROFILE=1` in the disposable runner to
provision a second synthetic profile under the same synthetic parent. It earns
both starters and each Rowan ribbon through real authored campaign commands;
companions, XP and gift items are not seeded. It checks explicit household/profile
selection, owned companions, actor-attributed tactical turns, boundary role swaps,
reload, two separately confirmed original-backed keepsake copies, exact lost-save
retry, no companion/XP/academic-evidence transfers and solo continuation. Before
confirmation it asserts both exact offered items and stable player codes are
visible. The accepted run is recorded in `campaign-paired-evidence.json`; this is
shared-device technical evidence, not network or consented child playtesting.
