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
