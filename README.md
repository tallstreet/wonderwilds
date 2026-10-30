# Wonderwilds development

The Phaser story adventure runs in `frontend/`. The Rust
workspace adds versioned contracts, pure gameplay and assessment, an experimental
mastery policy, an authored director, and a PostgreSQL-backed development API.
The expansion is **not a completed release**. See
[implementation status](docs/IMPLEMENTATION_STATUS.md) for integration and review gates.
The active work follows the [implementation plan](docs/IMPLEMENTATION_PLAN.md) and
[curriculum mission integration](docs/CURRICULUM_MISSION_PLAN.md): profile-owned
practice through the existing curriculum graph inside story missions. The
[companion progression plan](docs/COMPANION_PROGRESSION_PLAN.md) remains the
supporting specification for creature expansion.

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

Use `sh scripts/local-demo.sh` to run the app with its local authenticated session
and database. Standalone Vite requires an operator-provided session bridge.
For separate API development, run from the repository root:

```sh
WONDERWILDS_DATABASE_URL='host=/workspace/setup-tools/pgsocket port=55432 user=agent dbname=wonderwilds_dev' \
  cargo run -p wonderwilds-api --locked
```

The API listens on local port 3001. `/healthz` reports the development process;
`/readyz` probes database/schema readiness and explicitly reports that content is
not release-ready. Gameplay requests without a valid parent session return 401.
There is no test-login bypass or configured parent identity provider. Both content
modes use the same authenticated story world, PostgreSQL saves and learning UI.

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
There are exactly two content modes, selected automatically at backend startup:

- **Authored:** without Modal credentials, scenes and missions use authored content.
- **Generated:** with both `MODAL_PROXY_TOKEN_ID` and `MODAL_PROXY_TOKEN_SECRET`,
  Modal supplies scene dialogue, mission plans and repeatable neighbourhood stories.
  Invalid or unavailable output uses validated authored fallbacks.

No URL parameter or generation flag is needed. `?guest=1` and `?story=1` no longer
select modes; `/demo.html` and `/story.html` redirect to `/`. Partial credentials
produce a configuration error. Restart after changing credentials. Both modes
require the local backend and database; authored mode needs no model service.
Both ports 5173 and 3001 must be free. `WONDERWILDS_PG_BIN` can select another
local PostgreSQL binary directory. Stop the foreground launcher with Ctrl-C;
the private database stays running. To stop it as well:

```sh
/opt/homebrew/opt/postgresql@17/bin/pg_ctl -D "$PWD/.local-demo/data" stop
```

This explicitly enabled developer bridge creates a synthetic parent/profile and
24-hour session in an isolated PostgreSQL database under ignored `.local-demo/`.
Restart the launcher to renew the session. Progress persists across restarts.
A reset database gets a new profile ID so old browser saves cannot attach to it.
The database listens only on a private Unix socket; the API and browser bind
loopback. Vite checks Host, Origin and a random browser CSRF token before
forwarding commands with the operator-issued session. The Rust API still uses
normal session/ownership/CSRF verification; it has no login bypass route. Session
secrets never reach the browser and `.local-demo/` is denied by Vite's file server.
The runtime database role is not a superuser and cannot bypass RLS. Do not expose
this development bridge through a tunnel or use it with personal data.

The story world uses server-assessed learning inside mission steps. Rust owns
correctness, progress and supported completion. Older guest and assessed-adventure
interfaces are no longer app entry points. Curriculum coverage and review gates
remain as described in [implementation status](docs/IMPLEMENTATION_STATUS.md).

For real browser checks, `node qa/e2e/run-training-qa.mjs` creates and removes a
fresh synthetic database/session on ports 3002/5180. It never uses demo progress.
Set `CHROMIUM_PATH` to your installed Chromium executable if needed.

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

After starting `sh scripts/local-demo.sh`, [open the app](http://127.0.0.1:5173/).
This uses the actual Phaser world and authenticated PostgreSQL saves. Seven
connected stories are playable:

1. **Pip’s bridge:** collect wood, prepare planks, repair the visible bridge and
   cross with Pip following.
2. **Pip’s picnic:** gather berries, pack a basket at the workshop, lay out the
   picnic and bring Pip back across to share it.
3. **Welcome lights:** gather glow pebbles, make lanterns and light both banks.
4. **The humming hollow:** follow clues, rule out a false lead, discover the sound
   and choose where to place the wind instrument before telling Pip.
5. **A cosy corner for Pip:** hear Pip’s preferences, choose a leafy nook or star
   canopy, build it and enjoy the finished place together.
6. **Bramble’s first visit:** learn Bramble’s wish, check and clear the workshop
   path, confirm readiness, walk together and welcome Bramble at the bridge bank.
7. **A moonlight gathering:** choose either bank, invite both friends and prepare
   seats and a humming circle in either order, then welcome everyone.

Talk to Pip after each completion to accept the next story. Earlier world changes
remain visible, and Missions remembers the completed stories. Restart an already
running local demo after updating the backend; existing bridge saves continue
without a reset.
Missions offers paths to the current characters/resources and shows inventory and
milestones. Click a nearby character or press Space to read the returned dialogue.
Reload restores the server's location/progress. If saving is uncertain, open
Missions and use **Retry the same action**; the exact command survives reloads.

Story state and learning evidence are persisted separately. Without credentials,
a deterministic provider returns authored scenes for the seven missions through
the same validated response boundary used for Modal generation. Server state controls who is present, available actions and
bridge/milestone changes. Pixel movement remains local; server commands verify
coarse location transitions. The woodcutter temporarily uses existing Cleo art,
distinct from Bramble.
Modal credentials enable generated prose and missions in this same world.

### Walking to neighbourhood stories

In the story world, walk west from the bridge bank to the workshop, then take its
western path to Starlight Cove. Continue east through Fern Hollow, Mossy Clearing,
Owl Grove and Acorn Trail. Walking onto a labelled route at the map edge commits
its server-approved travel action. Missions also offers accessible route buttons;
bridge crossing and shared escort actions keep their existing controls.

Marina, Fern, Moss, Orla and Hazel each offer a small three-step story: accept the
request, investigate the marked object, then return to share the discovery.
The Missions journal retains their task and status through reloads. At most three
neighbourhood missions are open, reserving a fourth slot for Pip's ongoing story.
Accepting or resuming another archives the oldest open neighbourhood mission;
its checked task is preserved and visiting its friend allows resumption. Completed
missions stay completed, and Pip's story is never automatically archived.

### Repeatable AI-generated neighbourhood stories

When both Modal proxy credentials are supplied, arrivals generate neighbourhood
stories automatically. Without them, the five authored neighbourhood missions
remain available. Restart the local demo after changing credentials.

On entering a neighbourhood without unfinished work, the model invents a name,
personality, greeting, mission title/premise, one to three sequential investigation
tasks, and a payoff. It chooses from existing character and object artwork. Accept
the story, investigate its marked objects, then return to share the discoveries.
After completion, leave and return to receive another story. There is no fixed
catalogue limit on these repeat visits; the physical map catalogue remains finite.

Unfinished and archived stories keep their generated content and progress. At most
three neighbourhood missions are open, reserving Pip's fourth slot; old open
missions are archived on acceptance as before. The latest story for each of the
five areas stays in the hot journal. Older completed versions remain in durable
command history, not an ever-growing browser list.

Generation happens outside database locks and is validated before committing with
travel. Exact retries return the original saved story; reloads do not regenerate
it. Rust owns adjacency, task order, completion and the mission cap. AI generates
content for supported investigation actions, not arbitrary mechanics or rewards.
Malformed, busy or unavailable model responses use a saved, labelled offline
fallback; an unfinished fallback also remains stable until completed.

The mission journal distinguishes **AI-generated story** from **Offline story
fallback**. New geography, new image assets and production content approval are
not supplied by this feature. Existing saves and original command receipts remain
compatible. No demo-data reset or new SQL migration is needed.

For local mock/provider testing, `WONDERWILDS_MODAL_ENDPOINT` selects an
explicit backend endpoint (HTTPS, or loopback HTTP only). It receives the configured
Modal authorization header; it applies to all three generators. Leave it unset for the deployed model.

### Curriculum practice in story missions

In the story world, gathering supplies, workshop jobs, building and investigation
steps open a learning interaction as part of that action. There is no separate
practice-launch button. Generated neighbourhood steps can provide a short
`learningBridge` invitation explaining why organising discovery markers helps
with their investigation. Existing stories use authored joining text. Rust
selects a prerequisite-ready activity; generated prose never chooses answers,
assessment rules or rewards.

A correct answer commits the learning evidence and bound story action together.
**Let’s do it together** supplies a server-authored explanation and completes the
same step with supported, unassessed evidence rather than independent success.
Hints and examples remain available. Closing the panel preserves the task;
Missions offers resume and exact-save recovery. **Put this task aside** closes
it without completing the story action. Travel or another tab can invalidate a
joining point; put that task aside and reopen the current step. Unsupported
frontiers retain the normal story action. Original direct story commands remain available without an academic gate.

The combined Foundation/Years 1–6 graph contains 120 draft nodes, all with
code-authored practice. **M01 → M03 → M04/M05/M06** retain their original
assessed matching, counting, comparison, sequence and part–whole activities.
The other 115 have focused answer-card starters or guided human-review workshops.
Matching and counting support drag/tap/keyboard; answer cards support drag/tap and
optional reviewed speech-to-text. Writing workshops offer a local draft pad and
optional dictation, with human review and supported completion. There is no TTS.
These starters do not establish full-skill readiness or mastery. After the original
five-skill path, Rust offers new starters by prerequisite exposure and year band.
See the [activity inventory and research limits](docs/ACTIVITY_COVERAGE.md) and
[selection/evidence decisions](docs/adr/curriculum-starter-practice.md). New profile-owned practice progress
and exact retry receipts persist separately from story and assessed-adventure
progress. Migration `0006_mission_learning.sql` is applied by the local launcher
on restart; disposable QA must use its own database.

“Ready for another skill” is experimental practice progression, not the existing
transfer-based secure-mastery assessment. It requires varied independent first
checks across two story contexts and two authenticated server sessions; reloading
a page alone is not a new session. Restarting the local demo launcher issues a
new synthetic server session while preserving the profile. Helpers remain available without requiring independent success. Original activities use public IXL patterns as references; content
review and release gates remain pending. See the [integration plan](docs/CURRICULUM_MISSION_PLAN.md)
for the policy and remaining activity coverage.

### Optional Modal scene generation

The game HUD show the source of the displayed dialogue:
**Preauthored story**, **LLM-generated dialogue**, or **Preauthored story · LLM
fallback**. This reflects the validated scene returned by the backend, including
cached LLM scenes, rather than probing a live connection. Mission planning is
separate. Restart the backend after updating to enable source reporting; older
responses show “Story source unavailable · refresh to check”.

The backend can call the deployed `wonderwilds` model at
`https://garytafe--ep-wonderwilds-server.us-west.modal.direct/v1/chat/completions`.
It uses the real scene schema, `speakers-unique-v1` instructions and low reasoning.
This remains a development experiment: mechanical validation is not approval of
model-written prose for children. Release/content review gates stay unchanged.

With your existing proxy credentials exported in the terminal, stop the running
local-demo process with Ctrl-C, then restart it:

```sh
# MODAL_PROXY_TOKEN_ID and MODAL_PROXY_TOKEN_SECRET must already be exported.
sh scripts/local-demo.sh
```

Open [Wonderwilds](http://127.0.0.1:5173/). Generated prose applies to the story
world; assessment remains server-authored. Initial reads and the existing
post-action refresh request model prose. Credentials are backend-only; never use
`VITE_` variables for them. The model receives authored scene context and gameplay
state, not profile/session IDs, account data, learning records or free-form child
text. Do not commit credentials or put them in request logs.

The backend validates output with the existing Rust story validator. Failed,
truncated, oversized or invalid responses fall back to the authored scene. Requests
have a six-second total timeout and two-second connect timeout; a cold endpoint
may therefore fall back. One outbound generation runs at a time; busy requests
fall back immediately. Up to 32 validated scenes are cached in memory against the
exact prompt/input. Restart clears the cache, including after replacing the model
behind the same endpoint name.

Generation occurs after the read transaction releases its locks. Authentication
and state are checked again afterward; changed state discards the generated scene.
Commands and persisted retry receipts remain authored and unchanged. All actions,
rewards and progress still come from Rust. `.local-demo/api.log` contains only
coarse accepted/fallback messages for this integration, without model text or
credentials. To return to authored mode, unset both Modal proxy credentials and restart the demo.

### Optional Modal mission planning

Mission planning is enabled automatically with scene and neighbourhood generation
when both Modal proxy credentials are present.

Restart the demo after updating; its migration runner applies
`0004_story_mission_plans.sql` without resetting existing progress. In
the story world, accepting a **new** mission requests a plan using
that supported opportunity and the evaluated `dependencies-v1` prompt. The API
checks its schema, character/condition bindings, outcome coverage and a bounded
real-domain completion path before saving it atomically with acceptance. The
mission title, premise, payoff and milestone labels appear in Missions and are
supplied to subsequent scene generation. Rust still owns actions and completion;
parallel tasks remain playable in either supported order.

Plans persist across reloads, pause/resume and server restarts. Exact command
retries return the original response without generating a replacement plan. Existing
active missions keep their authored briefs; planning starts at the next new mission
acceptance. Completed mission history uses saved titles. Profile export includes
plans and profile deletion removes them. Model calls occur outside database locks,
with session/CSRF and revision checks before commit.

The six-second provider timeout, busy/unavailable endpoint, malformed output or
unproven plan all fall back to the authored mission. Validation search also has a
500 ms deadline. The planner receives authored world context and story progress, not
profile/session identifiers or educational records. Removing credentials stops future generation; saved content remains stable. Development content/release gates are unchanged.

Both scene generation and mission planning send strict JSON Schema
`response_format` constraints from the shared contracts, as well as the schema in
the prompt. Responses still pass local schema and gameplay validation: the current
endpoint has produced a scene text longer than its schema permits. See the
[structured-output evaluation](qa/story-eval/RESULTS_JSON_SCHEMA_2026-10-31.md).

This integrates model-written plans over the **seven supported opportunities**;
it does not add arbitrary mission mechanics, competing opportunities or an endless
mission catalogue. Mechanical validation does not establish prose suitability.

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
The standalone CLI retains its 256-command development limit. PostgreSQL story
play uses durable receipts and compact checkpoints, including compatible migration
of exhausted old saves. There is no browser reset endpoint.
Broader mission catalogues and shared-adventure integration remain later stages. Content review gates are unchanged.
See the [payload builder notes](backend/crates/director/src/story/README.md) for
field sources, contracts and implementation limits.

An [offline story-evaluation harness](qa/story-eval/README.md) captures real
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

The separate [mission planning evaluator](qa/story-eval/planning/README.md)
prepares 18 distinct runtime-derived planning cases and checks proposed milestone
orders with executable completion witnesses. It can explicitly call the current
Modal model without training or changing gameplay. It covers the existing authored
opportunities, not unrestricted mission invention.

The [versioned scene dataset](qa/story-eval/datasets/pilot-2026-10-30/README.md)
preserves all 411 generated candidates and independent quality judgments for future
iterations. Run `python3 qa/story-eval/dataset.py verify` to check its integrity, or
`python3 qa/story-eval/dataset.py review --archived --out qa/story-eval/experiments/pilot-review`
to regenerate the report and training/evaluation shortlists. Training remains a
separate step after human review.

The [broader input pack](qa/story-eval/datasets/broader-2026-10-30-inputs/README.md)
adds 607 reachable situations and 1,821 generation requests across all seven
missions and eight combinations of lasting choices. Its authored references pass
mechanical checks; new model candidates and quality reviews are still pending.
The older pilot uses archived checks because its content version differs; its
reviewed candidates and subsequent user approvals remain unchanged.

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
WW_QA_BROWSER_TEST=qa/e2e/story-broader.mjs node qa/e2e/run-training-qa.mjs
WW_STORY_BRANCHES=alternate WW_QA_BROWSER_TEST=qa/e2e/story-broader.mjs node qa/e2e/run-training-qa.mjs
```

Set `CHROMIUM_PATH` to your Chromium executable if required. Persistence checks run
with `sh scripts/check-postgres.sh` and also create a disposable database.
