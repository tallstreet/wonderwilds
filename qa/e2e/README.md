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
