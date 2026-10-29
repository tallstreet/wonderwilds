# Local browser checks

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

Run that command from the repository root in the prepared Linux workspace after
installing the frontend and `qa/e2e` dependencies. It builds the API, creates a
unique disposable synthetic database, applies migrations, issues a temporary
operator session, and starts private API/Vite processes on ports 3002/5180. It
removes the database and temporary session when finished. It does not use or
modify `.local-demo/` data or session files. Set `CHROMIUM_PATH` if needed;
`WW_QA_SKIP_BUILD=1` reuses an API binary you have already built.

The journey recruits the four Meadow friends through real assessed HTTP commands,
then exercises the world journal, keyboard companion/training controls, wrong
answers, recorded support, reload/resume, abandonment, Pip's level-two reaction,
persistent XP, exact request retries, repeated-practice reward prevention and a
phone-sized training UI. Real educator, child usability and screen-reader reviews
remain separate requirements. `training-browser.mjs` can also target an already
prepared **fresh QA database** via `WW_DEMO_TEST_URL`; never point it at a database
whose progress should be preserved.

Personal quest documents against the real API and PostgreSQL:

```sh
node qa/e2e/run-personal-quest-qa.mjs
```

This selects the personal-quest journey from the runner's fixed allowlist and
uses the same fresh database, temporary session, private ports and cleanup as
the training check. It does not read or modify persistent demo progress.
The journey recruits and trains Pip through real assessed commands, checks the
level-four unlock, and exercises keyboard narrative choices, reload/resume,
bound activities, incorrect answers, recorded support, abandonment, reflection,
ongoing handoffs and mobile alternative branches. Exact retries and revisits
must not duplicate the fixed keepsake reward. Authored episode JSON supplies the
expected nodes; the browser does not supply progress, correctness or rewards.

The finite authored episodes can be revisited and do not imply unlimited unique
generated content. These checks make no model-training, educator approval or
human accessibility claim. `WW_QA_JOURNEY` accepts only `training` or
`personal-quest`; arbitrary script paths are rejected before provisioning.
