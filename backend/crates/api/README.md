# Authored development API

This is a Rust/Axum development service, backed by actual PostgreSQL transactions.
The existing browser guest game still runs independently. Parent sign-in, cloud
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
encounters, attempts and hints. The command route also accepts `abandon` with
an encounter ID, recording unassessed abandonment and releasing its pending slot
without rewards. Closed encounters cannot accept later answers. Attempt/hint route IDs must match their command
payload. Errors use the versioned stable error code/correlation envelope. Body
limit is 32 KiB, blocking operation concurrency is 16, SQL statements/locks have
bounded timeouts, and a development aggregate accepts at most 2,000 distinct
commands without evicting idempotency receipts.

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
