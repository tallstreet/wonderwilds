# Local browser checks

Human story review: `node qa/e2e/story-human-review.mjs` starts an isolated Python
review server on an ephemeral loopback port and uses a disposable progress file.
It checks blind grades, all three decisions, reload/server-restart persistence,
failed-save recovery, revision gating, JSONL export, request protections and mobile
layout. Set `CHROMIUM_PATH` as below. No actual human decisions or demo data are used.

Use the existing isolated checkout; no worktree is needed. From `frontend/`,
start `npm run dev -- --host 127.0.0.1`. From this directory run `npm ci` then
`npm test`. Chromium is available at `/usr/bin/chromium` in the prepared cloud
snapshot. Set `CHROMIUM_PATH` for a different installed browser, or
`WONDERWILDS_TEST_URL` for a different local development server.

These desktop/tablet checks verify actual Phaser startup, missions, journal,
parent export controls and the evidence limitation notice. They do not establish
a complete child playthrough, screen-reader quality or educational efficacy.
Browser automation and local socket access may require the executor's supported
sandbox escalation. No public preview is created.

Character library checks: `node character-library-smoke.mjs` against the running
frontend (default port5174; override with `WONDERWILDS_TEST_URL`). It verifies all
twelve actual artwork URLs decode, regional filters, keyboard details and desktop,
tablet and phone layouts. This page is a field guide, not a recruitment action.
`node companion-smoke.mjs` verifies journal memory/glow/reload controls with synthetic
HTTP fixtures; Rust/PostgreSQL suites provide separate authority/persistence checks.


Companion training against the real API and PostgreSQL:

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

The journey recruits the four Meadow friends through real assessed HTTP commands,
then exercises the world journal, keyboard companion/training controls, wrong
answers, recorded support, reload/resume, abandonment, Pip's level-two reaction,
persistent XP, exact request retries, repeated-practice reward prevention and a
phone-sized training UI. Real educator, child usability and screen-reader reviews
remain separate requirements. `training-browser.mjs` can also target an already
prepared **fresh QA database** via `WW_DEMO_TEST_URL`; never point it at a database
whose progress should be preserved.

Exploration quest against a fresh API/PostgreSQL profile:

```sh
WW_QA_BROWSER_TEST=qa/e2e/exploration-quest.mjs node qa/e2e/run-training-qa.mjs
```

This uses the same disposable-cluster runner and separate ports. It walks the
three-map kindness trail through actual dialogue and curriculum controls, checks
wrong answers, supported friendship, reload/resume and abandonment, then returns
to Pip and verifies the saved checklist and phone layout. Conversation acceptance
and return are local presentation state; the test verifies that neither issues
server rewards. Assessment steps and creature ownership come from the real API.

Curriculum practice inside the story world:

```sh
WW_QA_BROWSER_TEST=qa/e2e/mission-learning.mjs node qa/e2e/run-training-qa.mjs
```

This uses the same fresh synthetic cluster and ports. It accepts a story mission,
opens its practice invitation, submits a wrong one-to-one assignment, records
help, retries an uncertain save after reload and checks continued story access
and separate guest/assessed-adventure saves. The mission practice client tests
cover exact retry and stale-revision recovery; learning-engine tests cover the
five supported skills, evidence diversity and graph progression. This is not
full 120-skill activity coverage or educator approval. Keep model providers
disabled for these assessment/persistence checks.

Phaser Pip story world against a fresh API/PostgreSQL profile:

```sh
WW_QA_BROWSER_TEST=qa/e2e/story-world.mjs WW_STORY_SOURCE=bank node qa/e2e/run-training-qa.mjs
WW_QA_BROWSER_TEST=qa/e2e/story-world.mjs WW_STORY_SOURCE=workshop node qa/e2e/run-training-qa.mjs
```

Run each route separately: each command owns a new disposable cluster and ports
3002/5180. This enters `/?story=1` through the main adventure’s Missions link, walks to actual Phaser characters and resources,
and checks dialogue/choices from a schema-valid mock presentation, literal HTML
text rendering, explicit introduction acknowledgement, both wood sources,
exchange, lost-response retry after another tab advances the story, failed-refresh recovery without rolling the world back, and bridge repair. It attempts to walk
across without a server command before and after repair, then verifies Pip follows
the player across the bridge and appears once on the far bank. Phone layout,
reload, a nonplayable save-limit response, untouched assessed progress and unchanged guest-save bytes are included.
Screenshots are written under `/tmp/wonderwilds-story-world-*.png`; no persistent
demo data is used. The mock changes only initial display prose and a choice label;
all progress changes use real UI commands and the authoritative server runtime.

All three connected stories against a fresh API/PostgreSQL profile:

```sh
WW_QA_BROWSER_TEST=qa/e2e/story-series.mjs node qa/e2e/run-training-qa.mjs
```

This completes the bridge, picnic and lights through world interactions, accepts
each successor explicitly, checks pause/resume, reloads between stories, and retries
a committed exchange whose response was lost. It checks both crossing directions,
persistent picnic/lantern scenery, completed history, mobile layout and the
inspector, while verifying assessed progress remains untouched. Screenshots are
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
`WW_STORY_EXPECT_PLANS=1`, `WONDERWILDS_MODAL_PLANNER=1` and exported Modal proxy
credentials. Set `WONDERWILDS_MODAL_STORY=0` to isolate planning from scene requests.
This makes seven live inference requests and requires a healthy deployment. It
uses only the runner's disposable synthetic database, checks saved plans against
snapshots and the Missions dialog, and verifies reload stability. Authored fallback
is deliberately a failure in this opt-in mode, so availability is not mistaken for
model-quality evidence. Without the flag, the ordinary authored journey is unchanged.


Combined scene generation and mission planning with returning sessions:

```sh
WONDERWILDS_MODAL_PLANNER=1 WONDERWILDS_MODAL_STORY=1 \
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
exchange response and checks reload plus exact-body retry without duplicate progress.

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
`WW_STORY_RETURNING=1` with both provider flags set to `0` and omit
`WW_STORY_COMBINED`/`WW_STORY_EXPECT_PLANS`. Repeat both branches. An authored
rehearsal cannot satisfy combined-model acceptance.


## Long-play persistence

The disposable runner accepts `WW_LONG_PLAY=1`. With its default training test,
it executes more than 2,000 HTTP commands before exercising training, pending
support across an API restart, and an uncertain completion retried with the exact
body after another restart. No persistent demo data is used.

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
fallback and missing metadata at desktop/phone sizes in the world and inspector.
