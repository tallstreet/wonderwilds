# Wonderwilds development

The existing Phaser guest adventure remains playable in `frontend/`. The Rust
workspace adds versioned contracts, pure gameplay and assessment, an experimental
mastery policy, an authored director, and a PostgreSQL-backed development API.
The expansion is **not a completed release**. See
[implementation status](docs/IMPLEMENTATION_STATUS.md) for integration and review gates.
The next gameplay work follows the [companion progression plan](docs/COMPANION_PROGRESSION_PLAN.md);
[planning navigation](docs/IMPLEMENTATION_PLAN.md) links the broader roadmap and earlier milestone evidence.

Use this existing isolated checkout; do not create a worktree unless requested.
Node 24 is available in the prepared environment. Rust is pinned by
`rust-toolchain.toml`; activate it in this cloud workspace with:

```sh
export CARGO_HOME=/workspace/.cargo RUSTUP_HOME=/workspace/.rustup
export PATH="$CARGO_HOME/bin:$PATH"
```

Install locked JavaScript dependencies with `npm ci` separately in `frontend/`,
`contracts/` and `qa/e2e/`; use `cargo fetch --locked` at the root.
For a new cloud machine, install local PostgreSQL binaries with
`sh scripts/install-postgres.sh` if they are absent. Start and migrate the local
synthetic development database using `sh scripts/migrate-postgres.sh`.
This starts a private Unix-socket cluster without a TCP listener; it is not
production database provisioning. Migrations record hashes and reject drift.

Run the guest game from `frontend/` with `npm run dev -- --host 127.0.0.1`.
In a separate terminal, run the authored API from the repository root:

```sh
WONDERWILDS_DATABASE_URL='host=/workspace/setup-tools/pgsocket port=55432 user=agent dbname=wonderwilds_dev' \
  cargo run -p wonderwilds-api --locked
```

The API listens on local port 3001. `/healthz` reports the development process;
`/readyz` probes database/schema readiness and explicitly reports that content is
not release-ready. Gameplay requests without a valid parent session return 401.
There is no test-login bypass or configured parent identity provider. The main
world remains in guest mode. An opt-in local backend demo mounts the client and
activity components (see below); account/import integration remains pending.

From the root, `bash scripts/check.sh` runs Rust format/Clippy/tests, shared
schema and generated-code checks, content validators, frontend checks/build,
client tests and legacy fixtures. `sh scripts/check-postgres.sh` separately runs
real migration, RLS, ownership, deletion, race and rollback tests in a fresh
temporary PostgreSQL cluster. It discovers local PostgreSQL tools or accepts
`WONDERWILDS_PG_BIN`, and removes its cluster after the run. Its two database tests
are deliberately ignored in the default Rust suite and explicitly executed by
this helper. With Vite running, run:

```sh
(cd qa/e2e && npm test)
node frontend/src/learning/activity.browser.mjs
```

The prepared machine has Chromium at `/usr/bin/chromium`. Browser and network
operations may need the executor's supported sandbox escalation. No public
preview or production deployment is created. See the feature READMEs under
`frontend/src/api/`, `frontend/src/learning/`, `content/`, and
`backend/crates/api/` for trust boundaries and limitations.

## Local browser + backend demo (macOS)

Install PostgreSQL 17 (`brew install postgresql@17`) and locked frontend
packages, then run `sh scripts/local-demo.sh` from the repository root.
Open <http://127.0.0.1:5173/> to play the server-backed adventure in the actual world.
Use <http://127.0.0.1:5173/?guest=1> for the original guest adventure.
The earlier `/demo.html` remains available as a diagnostic screen.
Both ports 5173 and 3001 must be free. `WONDERWILDS_PG_BIN` can select another
local PostgreSQL binary directory. Stop the foreground launcher with Ctrl-C;
the private database stays running. To stop it as well:

```sh
/opt/homebrew/opt/postgresql@17/bin/pg_ctl -D "$PWD/.local-demo/data" stop
```

This explicitly enabled developer bridge creates a synthetic parent/profile and
24-hour session in an isolated PostgreSQL database under ignored `.local-demo/`.
Restart the launcher to renew the session. Progress persists across restarts.
The database listens only on a private Unix socket; the API and browser bind
loopback. Vite checks Host, Origin and a random browser CSRF token before
forwarding commands with the operator-issued session. The Rust API still uses
normal session/ownership/CSRF verification; it has no login bypass route. Session
secrets never reach the browser and `.local-demo/` is denied by Vite's file server.
The runtime database role is not a superuser and cannot bypass RLS. Do not expose
this development bridge through a tunnel or use it with personal data.

The game world mounts the typed client and activity controls for the six Meadow
story beats and four Meadow friendships. Missions route the player to labelled
NPCs; clicking a character or pressing Space nearby opens the same activity. Answers, support, abandonment, rewards,
companion selection and learning summaries are saved by the real API. Guest
localStorage is untouched. Server flags repair the bridge, bloom the garden and update the picnic. The
journal selects a following companion; the selected team survives reloads.
Friend artwork is provisional. Camera, hero choice and movement remain local
under a separate profile-specific view key; no guest evidence is imported.
The journal now presents separate character XP/levels and sixteen authored Meadow
training milestones, with repeat practice earning no extra XP. The training
clearing and journal open the same server-assessed activities, including support,
abandonment and resume. A first-time milestone awards 10 character XP and a
level-up celebration. Character levels use thresholds 0/10/20/30/40; Explorer XP
remains separate. Team selection/removal respects existing story-owned capacity,
and the world can render both followers. The second slot retains its Orchard
unlock; this does not activate Orchard gameplay.

Friendship memory and reversible glow controls are connected in the journal.
Server-authoritative regional travel, companion exploration roles, personal
quests, distinct transformation art, director integration, identity-provider
login, save import and parent lifecycle screens remain outstanding. The 75-map
exploration library does not establish regional story eligibility. No
spatial/measurement task is reachable in Meadow. The development API retains its
2,000-command cap; see the [long-play storage design](docs/LONG_PLAY_STORAGE_PLAN.md).
Hints currently repeat the authored instruction and suggest helper support;
worked-example text and narration are not yet supplied by the API.

`qa/e2e/backend-demo.mjs` exercises the full Meadow flow against a **fresh QA
copy**, not the demo database. Use API port 3002 (`WONDERWILDS_PORT=3002`) and
Vite port 5180 (`WW_DEMO_PORT=5180 WW_DEMO_API_PORT=3002`) with the local session
config, and set `CHROMIUM_PATH` to an installed Chromium executable. It verifies
incorrect/correct answers, support persistence, abandonment, four friendships,
companion selection, reloads, mobile layout, and local access boundaries.

For the actual world training journey with local PostgreSQL installed, run
`node qa/e2e/run-training-qa.mjs`. It creates and removes its own synthetic QA
database and temporary session, using ports 3002/5180 without changing demo
progress. See [browser checks](qa/e2e/README.md) and [CI scope](docs/CI.md).
