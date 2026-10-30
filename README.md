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
Use that exact address: `localhost:5173` is rejected by the demo session bridge.
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
story beats and four Meadow friendships. The ten story/friendship encounters are spread in pairs across Clover Meadow,
Starlight Cove, Peachy Orchard, Fern Hollow and Mossy Clearing; training is in Owl Grove.
These are browser placements for the Meadow content, not new regional story unlocks.
Missions tracks a destination; Follow path walks to the next connected exit or
nearby character. Clicking a character or pressing Space nearby opens dialogue
before starting an activity. The first exploration quest, “A trail of little
kindnesses”, links Pip, bridge repair, the Cove picnic, Tansy and the Orchard garden
before a return to Pip. Its seven-step checklist appears in Missions and Journal. Answers, support, abandonment, rewards,
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

### Optional spoken answers (Baseten hackathon)

Export `BASETEN_API_KEY` in the terminal environment before starting
`sh scripts/local-demo.sh`. Keep the key out of browser code and `VITE_*`
variables. Restart the demo after changing the key. When configured, supported
browsers show **Say my answer** for missing quantities and unambiguous single-choice
activities. Typing and the existing pieces remain available. Phonics, ordered
sequences and multi-recipient sharing do not use transcription for assessment.

The child starts a recording, finishes it (automatically at 20 seconds), reviews
or edits the transcript, chooses **Use this answer**, then **Check my idea**.
Cancellation stops the microphone and pending request. Microphone denial or
provider failure leaves the existing answer intact. Transcription never submits
an attempt automatically or determines correctness. Rust still assesses answers.
This adds audio input only, not read-aloud output or pronunciation scoring.

The local Vite bridge sends a bounded mono WAV to Baseten's OpenAI-compatible
chat endpoint using exactly `meta-models/muse-spark-open-1026`. The provider key
stays in the server environment. Host/Origin, browser CSRF, profile and the real
backend session are checked before a provider call. Only the recording and fixed
transcription instructions are sent, with no question, answer key or profile.
The application does not persist recordings or transcripts. Baseten's retention,
child-data terms and child-speech accuracy have not been verified; this remains
a local synthetic/adult-playtest feature, not approved child-data deployment.
The local bridge must not be tunneled or exposed publicly.

Voice verification (no paid requests): run the existing activity browser suite
with `CHROMIUM_PATH` as above and `node frontend/src/learning/voice-bridge.browser.mjs`.
The latter starts temporary servers on port 5191 and an ephemeral API port, uses
synthetic sessions and mocks the provider. A real account/model call is still
needed to verify hackathon access and transcription quality.

## Authored story development slice

After starting `sh scripts/local-demo.sh`, open **Missions → Pip’s story adventure**
in the main game, or [open it directly](http://127.0.0.1:5173/?story=1).
This uses the actual Phaser world and authenticated PostgreSQL saves. Three
connected stories are playable:

1. **Pip’s bridge:** collect wood, prepare planks, repair the visible bridge and
   cross with Pip following.
2. **Pip’s picnic:** gather berries, pack a basket at the workshop, lay out the
   picnic and bring Pip back across to share it.
3. **Welcome lights:** gather glow pebbles, make lanterns and light both banks.

Talk to Pip after each completion to accept the next story. Earlier world changes
remain visible, and Missions remembers the completed stories. Restart an already
running local demo after updating the backend; existing bridge saves continue
without a reset.
Missions offers paths to the current characters/resources and shows inventory and
milestones. Click a nearby character or press Space to read the returned dialogue.
Reload restores the server's location/progress. If saving is uncertain, open
Missions and use **Retry the same action**; the exact command survives reloads.

Story saves are separate from the existing adventure, guest saves and educational
activities. A deterministic mock provider returns scenes for the three authored
missions through the same validated response boundary that a future
provider can use. Server state controls who is present, available actions and
bridge/milestone changes. Pixel movement remains local; server commands verify
coarse location transitions. The woodcutter temporarily uses existing Bramble art.
This mode does not synchronize the old assessed bridge or generate new missions.
Model generation is disabled. The earlier [/story.html inspector](http://127.0.0.1:5173/story.html)
remains available for checking scenes without walking.

The [story implementation plan](docs/STORY_IMPLEMENTATION_PLAN.md) starts with an
isolated Rust mission and deterministic model-input builder. Run Pip's bridge
journey without a model, database or browser:

```sh
cargo run -p wonderwilds-director --example story_bridge --locked -- --source bank
cargo run -p wonderwilds-director --example story_bridge --locked -- --source workshop
```

Each journey accepts the mission, obtains wood, visits the woodcutter, exchanges
wood for planks, returns, repairs the bridge and crosses with Pip. It verifies
exact command retries and checkpoint replay; final JSON contains the actual planning
input, scene input and authored output. Diagnostics go to stderr.

Add `--output-dir /tmp/wonderwilds-story-trace` to save payload/checkpoint JSON at
every revision. The directory must not already exist. Inspect/resume a captured
workshop checkpoint with:

```sh
cargo run -p wonderwilds-director --example story_bridge --locked -- \
  --resume /tmp/wonderwilds-story-trace/003-checkpoint.json
```

Use `--choose` with an exact `availableActions[].id` from that resumed scene to
execute one action; add a new output directory to retain the result. Handles are
revision-bound. Presentation acknowledgement changes revision, so rebuild choices
after it. The normal scripted journey records the woodcutter introduction explicitly.

The CLI remains a developer harness. Local checkpoints verify
replay consistency, not authenticity; never import them as trusted server progress.
Both CLI and server retain the 256-command development limit. At that limit the
browser shows a terminal view and preserves the save; there is no reset endpoint.
Broader mission planning, shared-adventure integration and model generation remain later
stages. Content review gates are unchanged.
See the [payload builder notes](backend/crates/director/src/story/README.md) for
field sources, contracts and implementation limits.

An [offline story-evaluation harness](qa/story-eval/README.md) captures 137 real
synthetic runtime situations, prepares candidate/judge requests, checks candidate
scenes with the game engine and exports human-approved examples with a held-out
split. Start without a model or database:

```sh
python3 qa/story-eval/run.py prepare --out /tmp/story-eval-run \
  --brief 'Ages 6–8; short, warm sentences; concrete next steps.'
python3 qa/story-eval/run.py grade --run /tmp/story-eval-run \
  --candidates /tmp/story-eval-run/reference-candidates.jsonl \
  --out /tmp/story-eval-graded
```

The example age brief is an editorial test target. Open the generated
`review.html` to inspect results. Model integration uses imported JSONL files;
no provider calls or training jobs run automatically. Hard checks establish scene
bindings and immediate action executability, not prose quality or new-mission
playability. The authored references are smoke-test candidates, not approved
training examples.

The [versioned scene dataset](qa/story-eval/datasets/pilot-2026-10-30/README.md)
preserves all 411 generated candidates and independent quality judgments for future
iterations. Run `python3 qa/story-eval/dataset.py verify` to check its integrity, or
`python3 qa/story-eval/dataset.py review --out qa/story-eval/experiments/pilot-review`
to regenerate the report and training/evaluation shortlists. Training remains a
separate step after human review.

Start the human review screen with `python3 qa/story-eval/review_server.py`, then
open [http://127.0.0.1:5182/](http://127.0.0.1:5182/). Approve, request revision or
reject each scene; progress saves locally and model grades appear only after your
decision saves. Download the decisions for training export. See the
[review workflow](qa/story-eval/README.md#human-review-and-export) for details.

Run story browser QA against a fresh disposable database (never `.local-demo/`):

```sh
WW_QA_BROWSER_TEST=qa/e2e/story-browser.mjs node qa/e2e/run-training-qa.mjs
WW_STORY_SOURCE=workshop WW_QA_BROWSER_TEST=qa/e2e/story-browser.mjs node qa/e2e/run-training-qa.mjs
WW_QA_BROWSER_TEST=qa/e2e/story-world.mjs node qa/e2e/run-training-qa.mjs
WW_STORY_SOURCE=workshop WW_QA_BROWSER_TEST=qa/e2e/story-world.mjs node qa/e2e/run-training-qa.mjs
WW_QA_BROWSER_TEST=qa/e2e/story-series.mjs node qa/e2e/run-training-qa.mjs
```

Set `CHROMIUM_PATH` to your Chromium executable if required. Persistence checks run
with `sh scripts/check-postgres.sh` and also create a disposable database.
