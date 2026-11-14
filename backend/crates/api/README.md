# Authored development API

This is a Rust/Axum development service, backed by actual PostgreSQL transactions.
The browser opens the authenticated story world by default. Parent sign-in, cloud
save import, production database TLS and external identity-provider issuance are
not implemented here. No development login or public session-creation route is
exposed. Without an existing verified server session every game request returns
`UNAUTHENTICATED`; the HTTP tests supply synthetic authentication only inside
`cfg(test)`.

From the repository root, activate the pinned Rust cache and prepare the private
local PostgreSQL database with the repository setup scripts. Then:

```sh
export CARGO_HOME=/workspace/.cargo RUSTUP_HOME=/workspace/.rustup
export PATH=/workspace/.cargo/bin:$PATH
export WONDERWILDS_DATABASE_URL='host=/workspace/setup-tools/pgsocket port=55432 user=agent dbname=wonderwilds_dev'
export WONDERWILDS_ORIGIN='https://wonderwilds.local'
cargo run -p wonderwilds-api --locked
```

The server binds loopback port 3001. `/healthz` checks the process; `/readyz` probes
the database and required schema and identifies pending content review. Configured
connection strings do not establish readiness. Only private Unix-socket database
hosts are accepted; a future production transport adapter must verify database
TLS rather than using this development connector. Session cookies require
`Secure; HttpOnly; SameSite=Strict; Path=/`; write routes require the exact configured
HTTPS Origin and `X-CSRF-Token`. `X-Wonderwilds-Profile` is a selector checked against
the authenticated parent. Identity never comes from a game command body.

Implemented JSON routes match `contracts/openapi.yaml`: game snapshot, commands,
encounters, attempts, hints and the separate authored story routes described below.
The game command route also accepts `abandon` with
an encounter ID, recording unassessed abandonment and releasing its pending slot
without rewards. Closed encounters cannot accept later answers. Attempt/hint route IDs must match their command
payload. Errors use the versioned stable error code/correlation envelope. Body
limit is 32 KiB, blocking operation concurrency is 16, SQL statements/locks have
bounded timeouts, and a development aggregate accepts at most 2,000 distinct
commands without evicting idempotency receipts.

`GET /api/v1/story` returns a validated `StorySnapshot`: authoritative revision,
location, deterministic scene input, authored scene and explicit acknowledgeable
entry IDs. `POST /api/v1/story/commands` accepts only a revision-bound action handle
or scene/entry acknowledgement. Both reuse session/profile ownership; writes also
require Origin/CSRF. Only the woodcutter introduction currently records a presented
history fact. Merely fetching or rendering a scene does not acknowledge it.

Migration `0003_story_progress.sql` adds separate forced-RLS checkpoints and
original-response receipts and includes them in profile export/delete. Checkpoint
and receipt commit under the owned profile lock. Exact retries return the original
response even after later progress, so clients refresh before offering new actions.
Story revision, inventory and milestones never update assessed game progress or
learning records. At the 256-command development cap the snapshot is terminal,
omits scene/action payloads and preserves the save; old exact retries still work.
There is no story reset/import endpoint. `/` is the Phaser adventure; the retired inspector redirects there.
Modal credentials enable generation for authenticated story play.

Supplying both `MODAL_PROXY_TOKEN_ID` and `MODAL_PROXY_TOKEN_SECRET` enables
scene, neighbourhood and new-mission generation together. Without credentials,
all three use authored content; partial credentials fail startup. Old generation
flags are ignored. `WONDERWILDS_MODAL_ENDPOINT` overrides the shared endpoint for
HTTPS or loopback-only testing.
Preparation reads/replays authorized state and releases locks; generation and a
500 ms bounded domain completion search run outside the transaction. The API
rechecks session/CSRF, then the repository binds the validated plan to the exact
post-command checkpoint under the profile lock. Migration `0004_story_mission_plans.sql`
adds forced-RLS saved briefs with cascading deletion and profile export. Plan,
checkpoint and original response commit atomically. Retries check receipts first;
pause/resume and reload never regenerate a plan. Invalid/unavailable plans use
authored fallback. Snapshot projections feed saved goals, narrative and authoritative
condition labels to both clients and the optional scene provider. Mechanics and
available actions remain authored; generated ordering guides presentation rather
than adding new action gates. Changing credentials does not alter saved briefs.

The authored development slice uses the exact skill/template IDs from story and
friendship packs and real server assessment. It exposes Meadow menu interactions;
server-authoritative movement/path/position integration and additional-region
travel are later work. The engine still enforces quest prerequisites and ownership.
Do not describe this menu reachability policy as verified browser-world collision
integration. Story and educational content remain pending human review and must
not be activated for release.

Every command locks the owned profile row. State, learning checks/support/completion
evidence, immutable domain events, unique reward grants and outbox entries commit
in one transaction. A same-ID/same-body retry returns its original response;
changed bodies and stale new requests conflict. The persisted internal aggregate
is not the public snapshot; responses exclude answers, request receipts and session
secrets. Corrupt persisted state fails closed instead of resetting progress.
Hints and read-aloud support come from stored history. Imported browser evidence
has no route to server mastery. Learner summaries recompute the versioned mastery policy from server evidence
and canonical parameter fingerprints. Draft fixed items never claim novel
transfer, so this slice cannot fabricate secure status from supported or repeated
practice. Scheduler-to-encounter selection remains subsequent integration work. Outbox processing is not yet deployed.

Validation from repository root:

```sh
cargo test -p wonderwilds-api --locked
cargo clippy -p wonderwilds-api --all-targets --locked -- -D warnings
scripts/check-postgres.sh
```

The normal suite distinguishes ignored database tests. The database script creates
a disposable migrated UTF8 database, runs ownership/session/lifecycle checks and
explicit game transaction rollback/race tests under a non-superuser role, then
drops it. It must be allowed to access the private local socket. Do not run those
fixtures against production or a database containing user data.

`GET /api/v1/campaign` and `POST /api/v1/campaign/commands` expose the authored
Missing Moonlight campaign through the same verified session, profile, Origin and
CSRF boundary. A profile explicitly sends `adopt`; active legacy activities defer
adoption, while saved legacy stories and canonical companions remain available.
Campaign commands use their own expected revision and durable original-response
receipts. A repeated ID with a changed body conflicts. The numbered campaign
migration adds forced-RLS progress/receipts with profile export and cascading deletion.
Unsupported storage/content/rules versions return recoverable `UNAVAILABLE`, preserving
stored progress. Campaign scenes never consult a generation provider.

Campaign learning uses the existing mission-learning endpoint with
`begin_campaign_step`, an action ID and campaign revision. The server freezes the
content version, action and companion recipient. Supported and independent
completion commit the existing learning evidence, campaign step and canonical
milestone grant under the same profile lock. Changing the team cannot redirect
that reward. A no-longer-legal campaign action leaves completed learning evidence
intact. Campaign and legacy training milestones share grant aliases to prevent
duplicate XP. Companion ownership, XP and reversible Wayfinder appearance remain
in the existing canonical game aggregate and its replayable event/receipt stream.

Shared-device pairing uses `GET /api/v1/household/profiles`, `GET /api/v1/paired`
and revision-bound `POST /api/v1/paired/commands`. Only profiles owned by the
verified parent are selectable. The selected profile remains the answering learner;
role swaps change navigator/keeper permissions and never move academic evidence.
Each participant chooses an actually owned companion for a bounded, alternating
practice board. Practice grants no XP or academic evidence. Both selected players
must offer an existing whitelisted collectible and confirm the exact offers before
reciprocal keepsake copies are recorded. Originals and personal campaign progress
remain independently owned. Continuing solo preserves them. This is shared-device
play; network invitations and asset transfers remain unimplemented.

An operator can explicitly add another actual synthetic profile to the isolated
local demo with `python3 scripts/add-local-demo-player.py --create-player
--ability-band beginning` (one shell line). This command accepts only the fixed
`.local-demo` database and checks its synthetic identity issuer; it never modifies
another database or creates a profile automatically. The new profile must choose
its own starter and earn its own collectibles. Profile creation through a public
identity-provider-backed account UI remains outside the development API.
