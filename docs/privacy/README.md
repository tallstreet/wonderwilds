# Parent identity and privacy lifecycle — WW-08

Status: implementation primitives, **not production authentication**. The API
identity module supplies a synchronous PostgreSQL adapter and session checks;
it does not supply an identity provider or public account creation. HTTP
transport is owned by WW-09 and must be validated separately. The default game remains local guest play. Do not
enable cloud profiles until the integration and review gates below pass.

## Data flow and authority

An approved parent identity provider verifies the adult session. Its server-side
adapter issues independent cryptographically random session and CSRF tokens,
stores SHA-256 digests, and builds the `Session` primitive only after verifying
issuer, audience, signature, expiry and identity subject. Never accept a parent ID
from a browser request body. Configure one exact HTTPS origin. Require the bearer
digest, CSRF digest and exact Origin for every write, including logout. Set the
cookie attributes in `SESSION_COOKIE_ATTRIBUTES`; production should use a
`__Host-` cookie name, explicit expiry and no Domain attribute. Rotate session and
CSRF tokens on authentication/reauthentication and revoke the old session before
replacing it. Parent export/deletion requires `require_recent_reauthentication`.
No child email, exact birthday or free-text child name is in the profile schema.

The Rust `MemoryRepository` is a synthetic reference adapter for tests. Its
parent-ID arguments must originate from authenticated server context. It is not
durable, is not a production storage implementation, and does not establish
transaction safety. SQL migration `backend/migrations/0001_parent_profiles.sql`
supplies PostgreSQL ownership/RLS, export and deletion primitives used by
`identity::postgres::PostgresIdentity`. This adapter accepts an already configured
PostgreSQL client: use verified TLS in production, a private Unix socket for local
synthetic tests, and a blocking executor in async request handlers.

## PostgreSQL adapter integration contract

Use a dedicated non-superuser, non-`BYPASSRLS` runtime role. Migration owners and
backup operators are separate roles. Grant the runtime only required schema,
table and function privileges; PUBLIC has no table/function privileges. Session
lookup is server-side and must not expose digest tables through generic queries.
The session table deliberately has no parent-context RLS because authentication
precedes selecting a parent. Use a trusted authentication connection role with
only the session operations it needs, and no direct browser SQL interface. The
synthetic regression role grants broad privileges to test RLS boundaries; do not
copy that grant-all test role into deployment. Runtime parent-table DELETE must
be withheld except from the audited account-deletion lifecycle.
For every owned operation, open a transaction and bind the verified identity with
`SELECT set_config('app.parent_id', $1, true)`; never interpolate SQL or use a
connection-wide setting that leaks through a connection pool. Profile creation
also checks tombstones. RLS covers parent rows, profiles, progress, learning events, consent,
exports and cache. The application must not give end users SQL access or the
ability to set `app.parent_id`; RLS is defence in depth, not authentication.

Export uses an ownership-scoped, repeatable-read transaction that includes the
profile, progress, ordered learning events and all consent/revocation records.
Exclude tokens and provider credentials. Temporary export objects are private,
short-lived and parent-authorised on every download. The reference export model
uses synthetic opaque strings for events; the API adapter must use the versioned
learning contract and reviewed, allowlisted event fields.

Deletion calls `wonderwilds.delete_profile($1)` in the same authenticated
transaction after recent reauthentication. It creates a durable tombstone, queues
private export object cleanup, and cascades progress, events, consent, cache and
export jobs atomically. A worker must actually delete queued objects and mark the
outbox complete before reporting lifecycle completion. The tombstone journal and
outbox require independent replicated storage/backup policy. Deleting a parent
must call this flow for every profile before deleting the parent row: direct
parent deletion otherwise bypasses external-object cleanup and tombstones.

## Restore and retention

Before any restored database serves traffic, overlay the newest independent
deletion journal, remove all tombstoned profiles and dependent rows, reconcile
external objects/outbox, and verify a deletion canary. The insertion trigger
prevents a tombstoned ID from being recreated but is not sufficient to clean
already-restored rows. Restrict tombstone access and decide an approved lifetime
that covers every restorable backup. The Rust test simulates a tombstone overlay;
it is not a production backup/restore drill.

Proposed periods from the plan remain **pending legal/product approval**:
operational logs 14 days, raw evidence 90 days before aggregation, backups 35
days, export/deletion service target 30 days. Profile inactivity expiry and export
object lifetime need explicit decisions. No retention cron is enabled by this
change. Consent records support notice version, purpose, timestamp and revocation;
integration must stop the revoked purpose and cancel associated work. The
reference ledger alone does not enforce consent in other services.

## Vendor and release gates

Before cloud pilot: approve launch countries, identity/hosting/storage vendors,
processing agreements, cross-border transfers, child-readable and parent notices,
consent/assent requirements, retention and incident owner. External fonts and
browser speech still need review; no new third-party calls are introduced here.
No production child data enters tests, staging, director training or telemetry.
Logging must allowlist correlation/version/error codes and exclude raw answers,
tokens, parent identities, free text and prompts. Do not claim legal compliance.

Verified in the current task: PostgreSQL 17.11 migration applies and the
`0001_parent_profiles.test.sql` non-superuser test passes, covering cross-parent
reads, rejected foreign export/deletion, complete export, cascading deletion,
external-object cleanup enqueue and tombstone reinsertion rejection. The script
rolls back synthetic rows/role. It also denies foreign parent deletion (preventing
foreign-key cascade bypass) and foreign-parent profile insertion, and round-trips
stored Unicode through export. The real PostgreSQL Rust adapter test separately
passes ownership, CSRF/Origin, consent export, recent reauthentication, deletion
and logout/revocation. Run it explicitly with:

```sh
WW_IDENTITY_TEST_DATABASE_URL='host=/workspace/setup-tools/pgsocket port=55432 user=agent dbname=identity_test' cargo test -p wonderwilds-api --locked postgres_identity_ownership_export_revocation_and_delete -- --ignored
```

The database URL above identifies only a private synthetic Unix-socket cluster;
it contains no credential. Production requires a restricted runtime role and
verified TLS or an approved private socket. This is not a backup restoration drill.

For repeatable development validation, run `scripts/check-postgres.sh` from the
repository root. It uses a newly created disposable UTF8 database, applies both
migrations twice, executes the non-superuser SQL matrix and both explicit Rust
database tests, verifies that corrupted migration history is rejected, and drops
that database. Existing development data is not used. The local PostgreSQL
operator needs permission to create isolated test databases and synthetic roles.

`scripts/migrate-postgres.sh` records each complete migration's SHA-256 under an
advisory lock, committing the schema change and history row together. It defaults
to a fresh `wonderwilds_dev` database rather than adopting the earlier manually
applied `identity_test` database. Set `WONDERWILDS_DATABASE_URL` to target another
database; existing unmanaged Wonderwilds schemas are rejected and require a
separately audited bootstrap or migration to a fresh database. Applied files are
immutable: use a new numbered migration instead of changing recorded hashes.

Remaining engineering gates: real provider verification; HTTP transport and cookie
tests; transaction/concurrency tests; export/download worker; revocation wiring; retention
jobs; external deletion; independent tombstone replay; synthetic restore drill.
All are required before WW-08 is complete.
